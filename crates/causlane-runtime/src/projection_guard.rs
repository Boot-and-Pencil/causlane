//! Guarded projection read (M06.7): authorize the read at the `may_project`
//! stage, then redact (deny-by-default; redact-unless-revealable).
//!
//! Mirrors [`crate::guarded_executor::GuardedExecutor`] for the read path: a
//! reader receives a [`RedactionView`] only when a fresh, bound, actor-scoped
//! `may_project` `Allow` authorizes the read; otherwise the read is refused and
//! no view (hence no field) is produced.

use causlane_core::{
    apply_redaction, read_authz_gate, AuthzDecisionRef, AuthzDenyReason, AuthzGateOutcome,
    FieldPath, ProjectionReadRequest, RedactionPolicy, RedactionView,
};

/// Why a guarded projection read was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionReadError {
    /// The read was not authorized at the `may_project` stage (deny-by-default):
    /// the stage and reason from the read-authz gate.
    Unauthorized {
        /// The unauthorized stage (always `may_project`).
        stage: String,
        /// Why the read-authz gate denied.
        reason: AuthzDenyReason,
    },
}

/// Authorize a projection read, then compute its redaction view. Fail-closed: the
/// [`RedactionView`] is produced **only** when [`read_authz_gate`] allows the read
/// for `req.actor`; a denial returns [`ProjectionReadError::Unauthorized`] and no
/// view, so no field reaches the reader.
///
/// # Errors
/// Returns [`ProjectionReadError::Unauthorized`] when the read is not authorized
/// at the `may_project` stage.
pub fn guard_projection_read(
    decisions: &[AuthzDecisionRef],
    req: &ProjectionReadRequest<'_>,
    policy: &RedactionPolicy,
    fields: &[FieldPath],
) -> Result<RedactionView, ProjectionReadError> {
    match read_authz_gate(decisions, req) {
        AuthzGateOutcome::Allowed => Ok(apply_redaction(policy, fields)),
        AuthzGateOutcome::Denied { stage, reason } => {
            Err(ProjectionReadError::Unauthorized { stage, reason })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{guard_projection_read, ProjectionReadError};
    use causlane_core::{
        ActionId, AuditEventId, AuthzDecision, AuthzDecisionRef, AuthzDenyReason, AuthzPolicy,
        FieldPath, PlanHash, PlanHashError, ProjectionReadRequest, RedactionPolicy, RedactionView,
        Timestamp,
    };

    const HASH: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
    const PRED: &str = "projection.read_release_status";
    const POLICY: AuthzPolicy<'static> = AuthzPolicy {
        id: "p",
        version: "1",
        max_age: None,
    };

    fn allow(actor: &str, plan: &PlanHash) -> AuthzDecisionRef {
        AuthzDecisionRef {
            decision_event_id: AuditEventId("evt".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: plan.clone(),
            predicate_id: PRED.to_owned(),
            actor: actor.to_owned(),
            stage: "may_project".to_owned(),
            decision: AuthzDecision::Allow,
            policy_id: "p".to_owned(),
            policy_version: "1".to_owned(),
            issued_at: Timestamp(0),
            expires_at: Some(Timestamp(100)),
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

    fn redaction_policy() -> RedactionPolicy {
        RedactionPolicy {
            revealable: [FieldPath("name".to_owned())].into_iter().collect(),
        }
    }

    fn unauthorized_missing() -> ProjectionReadError {
        ProjectionReadError::Unauthorized {
            stage: "may_project".to_owned(),
            reason: AuthzDenyReason::Missing,
        }
    }

    // Deny-by-default: no may_project Allow -> read denied, no view produced.
    #[test]
    fn read_without_may_project_allow_is_denied() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH)?;
        let action = ActionId("act".to_owned());
        let fields = [FieldPath("name".to_owned()), FieldPath("ssn".to_owned())];
        let result = guard_projection_read(&[], &req(&action, &plan), &redaction_policy(), &fields);
        assert_eq!(result, Err(unauthorized_missing()));
        Ok(())
    }

    // CRITICAL-1: an Allow for another reader does not authorize this read.
    #[test]
    fn read_for_wrong_actor_is_denied() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH)?;
        let action = ActionId("act".to_owned());
        let fields = [FieldPath("name".to_owned())];
        let decisions = [allow("bob", &plan)];
        let result = guard_projection_read(
            &decisions,
            &req(&action, &plan),
            &redaction_policy(),
            &fields,
        );
        assert_eq!(result, Err(unauthorized_missing()));
        Ok(())
    }

    // An authorized read yields a redaction view: revealable revealed, others
    // redacted (fail-closed) — the projection is masked, not leaked.
    #[test]
    fn authorized_read_yields_redaction_view() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH)?;
        let action = ActionId("act".to_owned());
        let fields = [FieldPath("name".to_owned()), FieldPath("ssn".to_owned())];
        let decisions = [allow("alice", &plan)];
        let result = guard_projection_read(
            &decisions,
            &req(&action, &plan),
            &redaction_policy(),
            &fields,
        );
        assert_eq!(
            result,
            Ok(RedactionView {
                revealed: [FieldPath("name".to_owned())].into_iter().collect(),
                redacted: [FieldPath("ssn".to_owned())].into_iter().collect(),
            })
        );
        Ok(())
    }
}
