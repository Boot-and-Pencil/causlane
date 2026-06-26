//! Read/projection authorization (M06.7, S06) — deny-by-default at the
//! `may_project` stage (`docs/07-security-and-authz.md`: "No projection of
//! sensitive truth without projection authz").
//!
//! This is **not** a new authority. It reuses the ADR-0011 deny-by-default gate
//! [`authz_gate`] verbatim, pinned to the single required stage
//! [`MAY_PROJECT_STAGE`], so the read path and the execution path share one
//! fail-closed classifier and cannot drift. The only additions are a named
//! request carrier and an actor-scoping filter.
//!
//! Unlike the execution path, a read is authorized **for a specific reader**:
//! [`read_authz_gate`] first restricts the decisions to those issued for
//! `req.actor`, so a `may_project` `Allow` minted for another actor cannot
//! authorize this reader — it is invisible to the gate, which then denies
//! `Missing`. `authz_gate` itself does not bind `actor`; this filter is what
//! makes the read path actor-scoped.
//!
//! Non-formal-bound: no codegen / formal references.

use super::{
    authz_gate, ActionId, AuthzDecisionRef, AuthzGateOutcome, AuthzPolicy, PlanHash, Timestamp,
};

/// The canonical lifecycle stage a projection read must be authorized at.
pub const MAY_PROJECT_STAGE: &str = "may_project";

/// The coordinates a projection read is authorized against. Bundled into one
/// borrow so [`read_authz_gate`] stays a two-argument function (these always
/// travel together as what is being read, by whom, under which policy, at what
/// time).
#[derive(Clone, Copy, Debug)]
pub struct ProjectionReadRequest<'a> {
    /// The projection action being read.
    pub action: &'a ActionId,
    /// The plan the projection was produced under.
    pub plan: &'a PlanHash,
    /// The projection predicate id the `may_project` decision must bind.
    pub predicate_id: &'a str,
    /// The reader; only `may_project` decisions issued for this actor authorize
    /// the read (separation from other readers' decisions).
    pub actor: &'a str,
    /// The policy the `may_project` decision must be issued under (P0-010).
    pub policy: AuthzPolicy<'a>,
    /// The evaluation time freshness is judged against.
    pub now: Timestamp,
}

/// Fail-closed read/projection authorization gate (ADR-0011, deny-by-default). A
/// projection read is allowed iff some `may_project` decision **issued for
/// `req.actor`** is a valid, exactly-bound, in-policy, non-expired `Allow`;
/// anything else denies with the same precedence as the execution-side gate.
///
/// Delegates to [`authz_gate`] over the single required stage
/// [`MAY_PROJECT_STAGE`] so the read path and the execution path cannot diverge,
/// after restricting `decisions` to `req.actor` — the gate does not itself bind
/// the requesting actor, so a decision for another reader is filtered out and the
/// gate denies `Missing`.
#[must_use]
pub fn read_authz_gate(
    decisions: &[AuthzDecisionRef],
    req: &ProjectionReadRequest<'_>,
) -> AuthzGateOutcome {
    let required = [MAY_PROJECT_STAGE.to_owned()];
    let for_reader: Vec<AuthzDecisionRef> = decisions
        .iter()
        .filter(|decision| decision.actor == req.actor)
        .cloned()
        .collect();
    authz_gate(
        &required,
        &for_reader,
        req.action,
        req.plan,
        req.predicate_id,
        req.policy,
        req.now,
    )
}

#[cfg(test)]
mod tests {
    use super::{read_authz_gate, ProjectionReadRequest, MAY_PROJECT_STAGE};
    use crate::domain::{
        ActionId, AuditEventId, AuthzDecision, AuthzDecisionRef, AuthzDenyReason, AuthzGateOutcome,
        AuthzPolicy, Timestamp,
    };
    use crate::{PlanHash, PlanHashError};

    const HASH: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
    const PRED: &str = "projection.read_release_status";
    const POLICY: AuthzPolicy<'static> = AuthzPolicy {
        id: "p",
        version: "1",
        max_age: None,
    };

    fn decision_ref(
        actor: &str,
        stage: &str,
        verdict: AuthzDecision,
        predicate_id: &str,
        plan: &PlanHash,
        expires_at: Option<u64>,
    ) -> AuthzDecisionRef {
        AuthzDecisionRef {
            decision_event_id: AuditEventId("evt".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: plan.clone(),
            predicate_id: predicate_id.to_owned(),
            actor: actor.to_owned(),
            stage: stage.to_owned(),
            decision: verdict,
            policy_id: "p".to_owned(),
            policy_version: "1".to_owned(),
            issued_at: Timestamp(0),
            expires_at: expires_at.map(Timestamp),
            attestation: None,
        }
    }

    fn req<'a>(action: &'a ActionId, plan: &'a PlanHash) -> ProjectionReadRequest<'a> {
        ProjectionReadRequest {
            action,
            plan,
            predicate_id: PRED,
            actor: "alice",
            policy: POLICY,
            now: Timestamp(10),
        }
    }

    fn denied(reason: AuthzDenyReason) -> AuthzGateOutcome {
        AuthzGateOutcome::Denied {
            stage: MAY_PROJECT_STAGE.to_owned(),
            reason,
        }
    }

    #[test]
    fn read_allow_authorizes() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH)?;
        let action = ActionId("act".to_owned());
        let decisions = [decision_ref(
            "alice",
            MAY_PROJECT_STAGE,
            AuthzDecision::Allow,
            PRED,
            &plan,
            Some(100),
        )];
        assert_eq!(
            read_authz_gate(&decisions, &req(&action, &plan)),
            AuthzGateOutcome::Allowed
        );
        Ok(())
    }

    #[test]
    fn read_missing_is_denied() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH)?;
        let action = ActionId("act".to_owned());
        assert_eq!(
            read_authz_gate(&[], &req(&action, &plan)),
            denied(AuthzDenyReason::Missing)
        );
        Ok(())
    }

    // An Allow on a different stage is invisible to the may_project gate.
    #[test]
    fn read_wrong_stage_does_not_authorize() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH)?;
        let action = ActionId("act".to_owned());
        let decisions = [decision_ref(
            "alice",
            "execution_barrier_logged",
            AuthzDecision::Allow,
            PRED,
            &plan,
            Some(100),
        )];
        assert_eq!(
            read_authz_gate(&decisions, &req(&action, &plan)),
            denied(AuthzDenyReason::Missing)
        );
        Ok(())
    }

    // CRITICAL-1 witness: a may_project Allow issued for another reader does not
    // authorize this one (the requesting actor is bound by the wrapper's filter).
    #[test]
    fn read_for_wrong_actor_is_denied() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH)?;
        let action = ActionId("act".to_owned());
        let decisions = [decision_ref(
            "bob",
            MAY_PROJECT_STAGE,
            AuthzDecision::Allow,
            PRED,
            &plan,
            Some(100),
        )];
        assert_eq!(
            read_authz_gate(&decisions, &req(&action, &plan)),
            denied(AuthzDenyReason::Missing)
        );
        Ok(())
    }

    #[test]
    fn read_wrong_binding_is_denied() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH)?;
        let action = ActionId("act".to_owned());
        let decisions = [decision_ref(
            "alice",
            MAY_PROJECT_STAGE,
            AuthzDecision::Allow,
            "projection.other",
            &plan,
            Some(100),
        )];
        assert_eq!(
            read_authz_gate(&decisions, &req(&action, &plan)),
            denied(AuthzDenyReason::WrongBinding)
        );
        Ok(())
    }

    // issued_at 0, expires_at 5, now 10 -> expired.
    #[test]
    fn read_expired_is_denied() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH)?;
        let action = ActionId("act".to_owned());
        let decisions = [decision_ref(
            "alice",
            MAY_PROJECT_STAGE,
            AuthzDecision::Allow,
            PRED,
            &plan,
            Some(5),
        )];
        assert_eq!(
            read_authz_gate(&decisions, &req(&action, &plan)),
            denied(AuthzDenyReason::Expired)
        );
        Ok(())
    }

    // An explicit deny for this reader wins over default-missing (deny-wins).
    #[test]
    fn read_explicit_deny_is_denied() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH)?;
        let action = ActionId("act".to_owned());
        let decisions = [decision_ref(
            "alice",
            MAY_PROJECT_STAGE,
            AuthzDecision::Deny,
            PRED,
            &plan,
            Some(100),
        )];
        assert_eq!(
            read_authz_gate(&decisions, &req(&action, &plan)),
            denied(AuthzDenyReason::Denied)
        );
        Ok(())
    }
}
