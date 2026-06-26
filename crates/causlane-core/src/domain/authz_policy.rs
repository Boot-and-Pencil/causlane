//! Authorization policy model (M06.1) — the owned policy entity over the
//! deny-by-default gate.
//!
//! The per-decision/aggregate authority lives in [`crate::domain::authz`]
//! (ADR-0011, I-009): [`classify_authz_decision`](super::classify_authz_decision)
//! and the deny-wins [`authz_gate`]. This module adds the
//! *owned policy entity* a runtime holds — the core analogue of the
//! `AuthzPolicyManifest` DTO in `causlane-contracts` — and a default-deny decider
//! that **delegates** the per-decision verdict to [`authz_gate`] so the live gate,
//! this model, and replay cannot drift. It re-derives no binding, freshness, or
//! deny-precedence logic.
//!
//! It is engine-agnostic: it prescribes no authorization paradigm
//! (`RBAC`/`ABAC`/`ReBAC`) and embeds no engine (`Cedar` is M06.2;
//! `Casbin`/`AuthZEN`/`OpenFGA` are M06.3). It carries only policy identity,
//! version, the lifecycle stages it authorizes, and a freshness bound — the same
//! vocabulary the gate and the `AuthzDecisionRef.stage` already use.

use super::{
    authz_gate, ActionId, AuthzDecisionRef, AuthzDenyReason, AuthzGateOutcome, AuthzPolicy,
    PlanHash, Timestamp,
};

/// Typed authorization-policy identifier (P0-010) — a newtype so a policy id can
/// never be confused with another string id (mirrors [`crate::domain::ConstraintId`]).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AuthzPolicyId(pub String);

/// An owned, engine-agnostic authorization-policy entity (ADR-0011, M06.1) — the
/// core analogue of the `causlane-contracts` `AuthzPolicyManifest` DTO.
///
/// Deny-by-default: a stage not listed in [`AuthzPolicyModel::stages`] is
/// unauthorized (no implicit fall-through), and an admitted stage's decisions must
/// satisfy the kernel gate under this policy's id/version + freshness bound (the
/// borrowed [`AuthzPolicy`] view it projects via [`AuthzPolicyModel::expected`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthzPolicyModel {
    /// Stable policy identity decisions must be issued under (P0-010).
    pub id: AuthzPolicyId,
    /// Stable policy version decisions must carry (P0-010).
    pub version: String,
    /// The lifecycle stages this policy authorizes (the same string tokens the
    /// gate and `AuthzDecisionRef.stage` use, e.g. `execution_barrier_logged`).
    /// A stage absent here is deny-by-default.
    pub stages: Vec<String>,
    /// Maximum decision age at evaluation time (ADR-0011 "fresh"); `None` imposes
    /// no freshness bound beyond a decision's own expiry. Projected onto the gate.
    pub freshness_max_age: Option<u64>,
}

impl AuthzPolicyModel {
    /// The borrowed classifier/gate view (P0-010 id/version + freshness bound) this
    /// policy projects onto [`authz_gate`]. The single source of the policy bound.
    #[must_use]
    pub fn expected(&self) -> AuthzPolicy<'_> {
        AuthzPolicy {
            id: &self.id.0,
            version: &self.version,
            max_age: self.freshness_max_age,
        }
    }

    /// Whether this policy authorizes `stage` (membership in [`AuthzPolicyModel::stages`]).
    /// Fail-closed: a stage not listed is not authorized.
    #[must_use]
    pub fn admits_stage(&self, stage: &str) -> bool {
        self.stages.iter().any(|listed| listed == stage)
    }
}

/// The outcome of evaluating a stage against an [`AuthzPolicyModel`] and the
/// decisions for it. Deny-by-default: anything that is not an explicit,
/// stage-authorized, gate-accepted allow is a denial.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthzPolicyOutcome {
    /// The policy authorizes this stage and the kernel gate accepted it.
    Allowed,
    /// The policy does not list this stage among the stages it authorizes
    /// (deny-by-default; decided before any decision is classified).
    StageNotAuthorized,
    /// The policy admits the stage but the decisions did not authorize it; carries
    /// the kernel gate's reason (deny-wins precedence is the gate's, not re-derived).
    DecisionDenied(AuthzDenyReason),
}

/// Deny-by-default policy decider (ADR-0011, M06.1). Fail-closed in two ordered
/// steps:
///   1. the policy must authorize `stage`            (policy gate — precedence);
///   2. the decisions must satisfy [`authz_gate`] for that stage under this
///      policy's [`AuthzPolicyModel::expected`] view at `now`   (I-009 + temporal).
///
/// Step 2 is delegated to the shared kernel gate (binding, deny-wins, P0-010, and
/// freshness/expiry), so this model cannot drift from the live gate or replay.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn decide_authz_policy(
    policy: &AuthzPolicyModel,
    decisions: &[AuthzDecisionRef],
    stage: &str,
    action: &ActionId,
    plan: &PlanHash,
    predicate_id: &str,
    now: Timestamp,
) -> AuthzPolicyOutcome {
    if !policy.admits_stage(stage) {
        return AuthzPolicyOutcome::StageNotAuthorized;
    }
    let required = [stage.to_owned()];
    match authz_gate(
        &required,
        decisions,
        action,
        plan,
        predicate_id,
        policy.expected(),
        now,
    ) {
        AuthzGateOutcome::Allowed => AuthzPolicyOutcome::Allowed,
        AuthzGateOutcome::Denied { reason, .. } => AuthzPolicyOutcome::DecisionDenied(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        authz_gate, decide_authz_policy, AuthzDenyReason, AuthzGateOutcome, AuthzPolicyId,
        AuthzPolicyModel, AuthzPolicyOutcome,
    };
    use crate::domain::{ActionId, AuditEventId, AuthzDecision, AuthzDecisionRef, Timestamp};
    use crate::{PlanHash, PlanHashError};

    const LISTED: &str = "execution_barrier_logged";
    const PRED: &str = "release.promote";

    fn plan() -> Result<PlanHash, PlanHashError> {
        PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    }

    fn policy(stages: &[&str], max_age: Option<u64>) -> AuthzPolicyModel {
        AuthzPolicyModel {
            id: AuthzPolicyId("p".to_owned()),
            version: "1".to_owned(),
            stages: stages.iter().map(|s| (*s).to_owned()).collect(),
            freshness_max_age: max_age,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn dec(
        stage: &str,
        verdict: AuthzDecision,
        predicate: &str,
        issued: u64,
        expires: Option<u64>,
        policy_version: &str,
        plan: PlanHash,
    ) -> AuthzDecisionRef {
        AuthzDecisionRef {
            decision_event_id: AuditEventId("d".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: plan,
            predicate_id: predicate.to_owned(),
            actor: "alice".to_owned(),
            stage: stage.to_owned(),
            decision: verdict,
            policy_id: "p".to_owned(),
            policy_version: policy_version.to_owned(),
            issued_at: Timestamp(issued),
            expires_at: expires.map(Timestamp),
            attestation: None,
        }
    }

    #[test]
    fn admits_stage_is_membership() {
        let p = policy(&[LISTED], None);
        assert!(p.admits_stage(LISTED));
        assert!(!p.admits_stage("other_stage"));
    }

    #[test]
    fn a_real_lifecycle_stage_with_a_valid_allow_is_allowed() -> Result<(), PlanHashError> {
        let p = policy(&[LISTED], None);
        let allow = vec![dec(
            LISTED,
            AuthzDecision::Allow,
            PRED,
            0,
            Some(1000),
            "1",
            plan()?,
        )];
        let out = decide_authz_policy(
            &p,
            &allow,
            LISTED,
            &ActionId("act".to_owned()),
            &plan()?,
            PRED,
            Timestamp(10),
        );
        assert_eq!(out, AuthzPolicyOutcome::Allowed);
        Ok(())
    }

    #[test]
    fn an_unlisted_stage_precedes_decision_classification() -> Result<(), PlanHashError> {
        // Policy lists only LISTED; query a stage it does not authorize, with a
        // decision that WOULD authorize that stage. The policy gate denies first.
        let p = policy(&[LISTED], None);
        let would_allow = vec![dec(
            "other_stage",
            AuthzDecision::Allow,
            PRED,
            0,
            Some(1000),
            "1",
            plan()?,
        )];
        let out = decide_authz_policy(
            &p,
            &would_allow,
            "other_stage",
            &ActionId("act".to_owned()),
            &plan()?,
            PRED,
            Timestamp(10),
        );
        assert_eq!(out, AuthzPolicyOutcome::StageNotAuthorized);
        Ok(())
    }

    #[test]
    fn a_listed_stage_delegates_deny_wins_to_the_gate() -> Result<(), PlanHashError> {
        // A wrong-binding allow plus an explicit deny: the gate's deny-wins ladder
        // reports Denied; the policy decider surfaces it as DecisionDenied(Denied).
        let p = policy(&[LISTED], None);
        let decisions = vec![
            dec(
                LISTED,
                AuthzDecision::Allow,
                "other.pred",
                0,
                None,
                "1",
                plan()?,
            ),
            dec(LISTED, AuthzDecision::Deny, PRED, 0, None, "1", plan()?),
        ];
        let out = decide_authz_policy(
            &p,
            &decisions,
            LISTED,
            &ActionId("act".to_owned()),
            &plan()?,
            PRED,
            Timestamp(10),
        );
        assert_eq!(
            out,
            AuthzPolicyOutcome::DecisionDenied(AuthzDenyReason::Denied)
        );
        Ok(())
    }

    #[test]
    fn a_listed_stage_with_no_decision_is_decision_denied_missing() -> Result<(), PlanHashError> {
        let p = policy(&[LISTED], None);
        let out = decide_authz_policy(
            &p,
            &[],
            LISTED,
            &ActionId("act".to_owned()),
            &plan()?,
            PRED,
            Timestamp(10),
        );
        assert_eq!(
            out,
            AuthzPolicyOutcome::DecisionDenied(AuthzDenyReason::Missing)
        );
        Ok(())
    }

    /// The decision sets the grid exercises (built once per `PlanHash`): a valid
    /// allow, an explicit deny, a wrong-binding allow, an expired allow, a
    /// wrong-policy allow, a deny-wins pair, and the empty (missing) set.
    fn decision_sets(ph: &PlanHash) -> Vec<Vec<AuthzDecisionRef>> {
        vec![
            vec![dec(
                LISTED,
                AuthzDecision::Allow,
                PRED,
                0,
                Some(1000),
                "1",
                ph.clone(),
            )],
            vec![dec(
                LISTED,
                AuthzDecision::Deny,
                PRED,
                0,
                None,
                "1",
                ph.clone(),
            )],
            vec![dec(
                LISTED,
                AuthzDecision::Allow,
                "other.pred",
                0,
                None,
                "1",
                ph.clone(),
            )],
            vec![dec(
                LISTED,
                AuthzDecision::Allow,
                PRED,
                0,
                Some(5),
                "1",
                ph.clone(),
            )],
            vec![dec(
                LISTED,
                AuthzDecision::Allow,
                PRED,
                0,
                Some(1000),
                "2",
                ph.clone(),
            )],
            vec![
                dec(
                    LISTED,
                    AuthzDecision::Allow,
                    "other.pred",
                    0,
                    None,
                    "1",
                    ph.clone(),
                ),
                dec(LISTED, AuthzDecision::Deny, PRED, 0, None, "1", ph.clone()),
            ],
            Vec::new(),
        ]
    }

    /// Assert the policy decider relates to the kernel gate exactly for one case
    /// (precedence + mirrors-gate + soundness); return the outcome for non-vacuity.
    fn check_case(
        p: &AuthzPolicyModel,
        set: &[AuthzDecisionRef],
        query: &str,
        action: &ActionId,
        ph: &PlanHash,
        now: Timestamp,
    ) -> AuthzPolicyOutcome {
        let admitted = p.admits_stage(query);
        let required = [query.to_owned()];
        let gate = authz_gate(&required, set, action, ph, PRED, p.expected(), now);
        let out = decide_authz_policy(p, set, query, action, ph, PRED, now);
        if admitted {
            match &gate {
                AuthzGateOutcome::Allowed => assert_eq!(out, AuthzPolicyOutcome::Allowed),
                AuthzGateOutcome::Denied { reason, .. } => {
                    assert_eq!(out, AuthzPolicyOutcome::DecisionDenied(reason.clone()));
                }
            }
        } else {
            assert_eq!(out, AuthzPolicyOutcome::StageNotAuthorized);
        }
        match &out {
            AuthzPolicyOutcome::Allowed => assert!(admitted && gate.is_allowed()),
            AuthzPolicyOutcome::StageNotAuthorized => assert!(!admitted),
            AuthzPolicyOutcome::DecisionDenied(_) => assert!(admitted),
        }
        out
    }

    /// Load-bearing property: across a grid of (stage listed-or-not) × decision
    /// sets × temporal boundaries, the policy decider relates to the kernel gate
    /// exactly: an unauthorized stage denies before classification (precedence,
    /// independent of the decisions); an authorized stage mirrors `authz_gate`;
    /// `Allowed` implies the stage was authorized AND the gate allowed; a decision
    /// reason is only ever reported for an authorized stage. Non-vacuity: each
    /// outcome variant occurs. now=3 is fresh, now=5 hits age == `max_age`, now=10
    /// is the stale boundary.
    #[test]
    fn policy_model_is_fail_closed_with_gate_precedence() -> Result<(), PlanHashError> {
        let p = policy(&[LISTED], Some(5));
        let action = ActionId("act".to_owned());
        let ph = plan()?;
        let sets = decision_sets(&ph);
        let nows = [Timestamp(3), Timestamp(5), Timestamp(10)];

        let mut saw_allowed = false;
        let mut saw_stage_not = false;
        let mut saw_decision_denied = false;

        for query in [LISTED, "other_stage"] {
            for set in &sets {
                for now in nows {
                    match check_case(&p, set, query, &action, &ph, now) {
                        AuthzPolicyOutcome::Allowed => saw_allowed = true,
                        AuthzPolicyOutcome::StageNotAuthorized => saw_stage_not = true,
                        AuthzPolicyOutcome::DecisionDenied(_) => saw_decision_denied = true,
                    }
                }
            }
        }

        assert!(saw_allowed, "no Allowed outcome was produced");
        assert!(saw_stage_not, "no StageNotAuthorized outcome was produced");
        assert!(
            saw_decision_denied,
            "no DecisionDenied outcome was produced"
        );
        Ok(())
    }
}
