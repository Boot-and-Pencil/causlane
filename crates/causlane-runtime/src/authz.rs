//! Live, deny-by-default authorization enforcement.
//!
//! The replay oracle verifies authorization *post-hoc* on recorded traces; the
//! runtime must enforce it *live* — an unauthorized barrier may never be spent.
//! [`AuthzGuard`] is the enforcement point: the runtime calls
//! [`AuthzGuard::authorize_barrier`] before deriving/spending an execution
//! capability, routing through the single kernel authority
//! [`causlane_core::KernelContracts`] (`AuthzEvaluator`, which delegates to the
//! pure `authz_gate`) so live enforcement and the replay oracle evaluate the same
//! gate (ADR-0011, fail-closed).

use causlane_core::{
    AuthzDecisionRef, AuthzDenyReason, AuthzEvaluator, AuthzGateOutcome, AuthzPolicy,
    ExecutionBarrier, KernelContracts, Timestamp,
};

/// The runtime refused to execute an unauthorized barrier (ADR-0011).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthzDenied {
    /// The unauthorized lifecycle stage.
    pub stage: String,
    /// Why the stage was denied.
    pub reason: AuthzDenyReason,
}

/// A runtime guard that enforces deny-by-default authorization before an
/// execution barrier may be spent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AuthzGuard;

impl AuthzGuard {
    /// Authorize `barrier` for every `required_stage` at `now`, deny-by-default.
    /// The runtime must call this and propagate the error before executing any
    /// op under the barrier.
    ///
    /// # Errors
    /// Returns [`AuthzDenied`] for the first required stage lacking a bound,
    /// non-expired allow decision.
    #[allow(clippy::too_many_arguments)]
    pub fn authorize_barrier(
        &self,
        barrier: &ExecutionBarrier,
        predicate_id: &str,
        required_stages: &[String],
        decisions: &[AuthzDecisionRef],
        expected_policy: AuthzPolicy<'_>,
        now: Timestamp,
    ) -> Result<(), AuthzDenied> {
        // Route the decision through the single kernel authority so live
        // enforcement and the replay oracle evaluate the same gate.
        match KernelContracts.evaluate_authz(
            required_stages,
            decisions,
            &barrier.action_id,
            &barrier.plan_hash,
            predicate_id,
            expected_policy,
            now,
        ) {
            AuthzGateOutcome::Allowed => Ok(()),
            AuthzGateOutcome::Denied { stage, reason } => Err(AuthzDenied { stage, reason }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AuthzGuard;
    use causlane_core::{
        ActionId, AuditEventId, AuthzDecision, AuthzDecisionRef, AuthzDenyReason, AuthzPolicy,
        ExecutionBarrier, ImpactSetHash, PlanHash, PlanHashError, Timestamp,
    };

    /// The policy the `allow` helper issues decisions under.
    const POLICY: AuthzPolicy<'static> = AuthzPolicy {
        id: "p",
        version: "1",
        max_age: None,
    };

    fn plan() -> Result<PlanHash, PlanHashError> {
        PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    }

    fn barrier(plan: PlanHash) -> ExecutionBarrier {
        ExecutionBarrier {
            barrier_id: AuditEventId("barrier".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: plan,
            op_indexes: vec![0],
            impact_set_hash: ImpactSetHash(
                "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                    .to_owned(),
            ),
            witnesses: Vec::new(),
            leases: Vec::new(),
            authz_decision_refs: Vec::new(),
            constraint_snapshot_id: None,
        }
    }

    fn allow(plan: PlanHash) -> AuthzDecisionRef {
        AuthzDecisionRef {
            decision_event_id: AuditEventId("d".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: plan,
            predicate_id: "release.promote".to_owned(),
            actor: "alice".to_owned(),
            stage: "execution_barrier_logged".to_owned(),
            decision: AuthzDecision::Allow,
            policy_id: "p".to_owned(),
            policy_version: "1".to_owned(),
            issued_at: Timestamp(0),
            expires_at: Some(Timestamp(100)),
            attestation: None,
        }
    }

    // The guard authorizes a barrier with a bound, non-expired allow.
    #[test]
    fn authorizes_barrier_with_valid_allow() -> Result<(), PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let result = AuthzGuard.authorize_barrier(
            &barrier(plan()?),
            "release.promote",
            &stages,
            std::slice::from_ref(&allow(plan()?)),
            POLICY,
            Timestamp(10),
        );
        assert!(result.is_ok());
        Ok(())
    }

    // Deny-by-default: with no decision present, the runtime refuses to execute.
    #[test]
    fn refuses_barrier_without_authorization() -> Result<(), PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let result = AuthzGuard.authorize_barrier(
            &barrier(plan()?),
            "release.promote",
            &stages,
            &[],
            POLICY,
            Timestamp(10),
        );
        assert_eq!(
            result,
            Err(super::AuthzDenied {
                stage: "execution_barrier_logged".to_owned(),
                reason: AuthzDenyReason::Missing,
            })
        );
        Ok(())
    }

    // An expired allow is refused live (expiry <= now), even though it exists.
    #[test]
    fn refuses_expired_allow() -> Result<(), PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let mut expired = allow(plan()?);
        expired.expires_at = Some(Timestamp(5));
        let result = AuthzGuard.authorize_barrier(
            &barrier(plan()?),
            "release.promote",
            &stages,
            std::slice::from_ref(&expired),
            POLICY,
            Timestamp(10),
        );
        assert!(matches!(
            result,
            Err(super::AuthzDenied {
                reason: AuthzDenyReason::Expired,
                ..
            })
        ));
        Ok(())
    }
}
