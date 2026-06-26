//! Guarded execution: wire [`AuthzGuard`] into the barrier-spend path.
//!
//! Closes the gap where `AuthzGuard` existed but had no caller (TZ-008/P0-007):
//! a [`GuardedExecutor`] authorizes a barrier — deny-by-default — *before* it
//! derives or spends an execution capability, so no op runs under an unauthorized
//! barrier (ADR-0011, ADR-0013). This is the runtime composition seam the docs
//! reference; it is intentionally a thin wrapper, not a full dispatcher.

use causlane_core::{
    AuthzDecisionRef, AuthzPolicy, CapabilityIssuer, CapabilitySpendRefusal,
    CapabilitySpendRequest, ExecutionBarrier, ExecutionCapabilityError, ExecutorPort,
    KernelContracts, Op, Timestamp,
};

use crate::authz::{AuthzDenied, AuthzGuard};

/// Guarded execution request consumed by the service-shaped executor seam.
#[derive(Clone, Copy, Debug)]
pub struct GuardedExecutionRequest<'a> {
    /// Barrier that must authorize the op.
    pub barrier: &'a ExecutionBarrier,
    /// Predicate whose authz policy applies.
    pub predicate_id: &'a str,
    /// Required authorization stages for this barrier.
    pub required_stages: &'a [String],
    /// Authz decisions available to the barrier.
    pub decisions: &'a [AuthzDecisionRef],
    /// Expected policy identity and freshness.
    pub expected_policy: AuthzPolicy<'a>,
    /// Spend/evaluation time.
    pub now: Timestamp,
    /// Operation to execute.
    pub op: &'a Op,
}

/// Owned authorization policy identity/freshness bound for queued guarded jobs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedAuthzPolicy {
    /// Expected policy id.
    pub id: String,
    /// Expected policy version.
    pub version: String,
    /// Maximum decision age at evaluation time.
    pub max_age: Option<u64>,
}

impl ExpectedAuthzPolicy {
    /// Create an owned policy bound.
    #[must_use]
    pub fn new(id: impl Into<String>, version: impl Into<String>, max_age: Option<u64>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            max_age,
        }
    }

    /// Borrow this owned bound as the kernel authz policy view.
    #[must_use]
    pub fn as_policy(&self) -> AuthzPolicy<'_> {
        AuthzPolicy {
            id: &self.id,
            version: &self.version,
            max_age: self.max_age,
        }
    }
}

impl From<AuthzPolicy<'_>> for ExpectedAuthzPolicy {
    fn from(policy: AuthzPolicy<'_>) -> Self {
        Self::new(policy.id, policy.version, policy.max_age)
    }
}

/// Owned guarded execution job that can cross runtime adapter boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardedExecutionJob {
    /// Barrier that must authorize the op.
    pub barrier: ExecutionBarrier,
    /// Predicate whose authz policy applies.
    pub predicate_id: String,
    /// Required authorization stages for this barrier.
    pub required_stages: Vec<String>,
    /// Authz decisions available to the barrier.
    pub decisions: Vec<AuthzDecisionRef>,
    /// Expected policy identity and freshness.
    pub expected_policy: ExpectedAuthzPolicy,
    /// Spend/evaluation time.
    pub now: Timestamp,
    /// Operation to execute.
    pub op: Op,
}

impl GuardedExecutionJob {
    /// Borrow this owned job as a guarded execution request.
    #[must_use]
    pub fn as_request(&self) -> GuardedExecutionRequest<'_> {
        GuardedExecutionRequest {
            barrier: &self.barrier,
            predicate_id: &self.predicate_id,
            required_stages: &self.required_stages,
            decisions: &self.decisions,
            expected_policy: self.expected_policy.as_policy(),
            now: self.now,
            op: &self.op,
        }
    }
}

impl From<GuardedExecutionRequest<'_>> for GuardedExecutionJob {
    fn from(request: GuardedExecutionRequest<'_>) -> Self {
        Self {
            barrier: request.barrier.clone(),
            predicate_id: request.predicate_id.to_owned(),
            required_stages: request.required_stages.to_vec(),
            decisions: request.decisions.to_vec(),
            expected_policy: request.expected_policy.into(),
            now: request.now,
            op: request.op.clone(),
        }
    }
}

/// Result of a guarded execution service call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionOutcome {
    /// Fact/object references produced by the executed op.
    pub produced_refs: Vec<String>,
}

/// Dependency-free service-shaped executor interface.
pub trait ExecutorService {
    /// Error type returned by service calls.
    type Error;

    /// Execute one guarded request.
    fn call(&self, request: GuardedExecutionRequest<'_>) -> Result<ExecutionOutcome, Self::Error>;
}

/// Why spending a barrier through a [`GuardedExecutor`] failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpendError<E> {
    /// Authorization was denied before any op ran (deny-by-default).
    Unauthorized(AuthzDenied),
    /// The capability could not be derived from the barrier.
    Capability(ExecutionCapabilityError),
    /// The derived capability was refused at spend time (M06.6 enforcement).
    CapabilityRefused(CapabilitySpendRefusal),
    /// The inner executor failed.
    Execute(E),
}

/// Wraps an [`ExecutorPort`] so a barrier is authorized before it is spent.
pub struct GuardedExecutor<E> {
    executor: E,
    guard: AuthzGuard,
}

impl<E: ExecutorPort> GuardedExecutor<E> {
    /// Wrap `executor` behind the deny-by-default authorization guard.
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            guard: AuthzGuard,
        }
    }

    /// Authorize `barrier` for `required_stages`, and only on success derive the
    /// capability for `op` and execute it. Authorization happens first, so a
    /// denial returns before any capability is derived or any op runs.
    ///
    /// # Errors
    /// Returns [`SpendError::Unauthorized`] when authorization is denied (no op
    /// runs), [`SpendError::Capability`] if the capability cannot be derived,
    /// [`SpendError::CapabilityRefused`] if the derived capability is not admissible
    /// for this op at `now` (M06.6), or [`SpendError::Execute`] if the inner executor
    /// fails.
    #[allow(clippy::too_many_arguments)]
    pub fn spend_barrier(
        &self,
        barrier: &ExecutionBarrier,
        predicate_id: &str,
        required_stages: &[String],
        decisions: &[AuthzDecisionRef],
        expected_policy: AuthzPolicy<'_>,
        now: Timestamp,
        op: &Op,
    ) -> Result<Vec<String>, SpendError<E::Error>> {
        self.execute_guarded(GuardedExecutionRequest {
            barrier,
            predicate_id,
            required_stages,
            decisions,
            expected_policy,
            now,
            op,
        })
    }

    fn execute_guarded(
        &self,
        request: GuardedExecutionRequest<'_>,
    ) -> Result<Vec<String>, SpendError<E::Error>> {
        // Deny-by-default: authorize BEFORE deriving/spending the capability.
        self.guard
            .authorize_barrier(
                request.barrier,
                request.predicate_id,
                request.required_stages,
                request.decisions,
                request.expected_policy,
                request.now,
            )
            .map_err(SpendError::Unauthorized)?;
        let capability = KernelContracts
            .derive_capability(request.barrier, request.op.index)
            .map_err(SpendError::Capability)?;
        // M06.6: the worker spends only a capability that admits this exact op and is
        // live at `now`. This enforces the lease-derived expiry the authorization gate
        // does not check (authz judges the decision's freshness, not the lease's), and
        // becomes load-bearing once a capability is carried across a trust boundary
        // rather than minted in-process.
        capability
            .spend_admits(CapabilitySpendRequest {
                barrier: request.barrier,
                requested_op: request.op.index,
                now: request.now,
            })
            .map_err(SpendError::CapabilityRefused)?;
        self.executor
            .execute(request.op, &capability)
            .map_err(SpendError::Execute)
    }
}

impl<E: ExecutorPort> ExecutorService for GuardedExecutor<E> {
    type Error = SpendError<E::Error>;

    fn call(&self, request: GuardedExecutionRequest<'_>) -> Result<ExecutionOutcome, Self::Error> {
        self.execute_guarded(request)
            .map(|produced_refs| ExecutionOutcome { produced_refs })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ExecutionOutcome, ExecutorService, GuardedExecutionJob, GuardedExecutionRequest,
        GuardedExecutor, SpendError,
    };
    use causlane_core::{
        ActionId, AuditEventId, AuthzDecision, AuthzDecisionRef, AuthzPolicy,
        CapabilitySpendRefusal, ClaimMode, ConstraintEpoch, EffectSignature, ExecutionBarrier,
        ExecutionCapability, ExecutorPort, ImpactHardness, ImpactSetHash, LeaseId, LeaseRef, Op,
        PlanHash, PlanHashError, ResourceId, Scope, Timestamp,
    };
    use core::convert::Infallible;

    const POLICY: AuthzPolicy<'static> = AuthzPolicy {
        id: "p",
        version: "1",
        max_age: None,
    };

    /// Returns a distinctive marker when (and only when) `execute` is reached, so
    /// a denied barrier — which never reaches the executor — is observable as the
    /// absence of the marker rather than via interior mutability.
    struct MarkerExecutor;

    impl ExecutorPort for MarkerExecutor {
        type Error = Infallible;
        fn execute(
            &self,
            _op: &Op,
            _capability: &ExecutionCapability,
        ) -> Result<Vec<String>, Self::Error> {
            Ok(vec!["executed".to_owned()])
        }
    }

    fn plan() -> Result<PlanHash, PlanHashError> {
        PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    }

    fn barrier(plan: PlanHash) -> ExecutionBarrier {
        ExecutionBarrier {
            barrier_id: AuditEventId("barrier".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: plan.clone(),
            op_indexes: vec![0],
            impact_set_hash: ImpactSetHash(
                "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                    .to_owned(),
            ),
            witnesses: Vec::new(),
            leases: vec![LeaseRef {
                lease_id: LeaseId("l".to_owned()),
                resource: ResourceId("r".to_owned()),
                scope: Scope("s".to_owned()),
                mode: ClaimMode::ExclusiveWrite,
                amount: 1,
                holder_action_id: ActionId("act".to_owned()),
                holder_plan_hash: plan,
                holder_op_index: Some(0),
                epoch: ConstraintEpoch(0),
                expires_at: None,
                lease_event_id: AuditEventId("evt_leases".to_owned()),
            }],
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

    fn op() -> Op {
        Op {
            index: 0,
            kind: "promote".to_owned(),
            effect: EffectSignature {
                reads: Vec::new(),
                writes: Vec::new(),
                produces: Vec::new(),
                requires: Vec::new(),
                invalidates: Vec::new(),
                conflict_domains: Vec::new(),
                hardness: ImpactHardness::Hard,
            },
        }
    }

    fn request<'a>(
        barrier: &'a ExecutionBarrier,
        stages: &'a [String],
        decisions: &'a [AuthzDecisionRef],
        op: &'a Op,
        now: Timestamp,
    ) -> GuardedExecutionRequest<'a> {
        GuardedExecutionRequest {
            barrier,
            predicate_id: "release.promote",
            required_stages: stages,
            decisions,
            expected_policy: POLICY,
            now,
            op,
        }
    }

    #[test]
    fn owned_job_borrows_back_to_equivalent_request() -> Result<(), PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let barrier = barrier(plan()?);
        let decisions = vec![allow(plan()?)];
        let op = op();

        let borrowed = request(&barrier, &stages, &decisions, &op, Timestamp(10));
        let job = GuardedExecutionJob::from(borrowed);
        let round_trip = job.as_request();

        assert_eq!(round_trip.barrier, &barrier);
        assert_eq!(round_trip.predicate_id, "release.promote");
        assert_eq!(round_trip.required_stages, stages.as_slice());
        assert_eq!(round_trip.decisions, decisions.as_slice());
        assert_eq!(round_trip.expected_policy, POLICY);
        assert_eq!(round_trip.now, Timestamp(10));
        assert_eq!(round_trip.op, &op);
        Ok(())
    }

    // Deny-by-default: an unauthorized barrier returns Unauthorized and never
    // reaches the executor (no "executed" marker is produced).
    #[test]
    fn barrier_cannot_be_spent_without_authorization() -> Result<(), PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let guarded = GuardedExecutor::new(MarkerExecutor);
        let result = guarded.spend_barrier(
            &barrier(plan()?),
            "release.promote",
            &stages,
            &[],
            POLICY,
            Timestamp(10),
            &op(),
        );
        assert!(matches!(result, Err(SpendError::Unauthorized(_))));
        Ok(())
    }

    // The `barrier` fixture with the single lease's `expires_at` set, so the derived
    // capability inherits that lease-derived expiry.
    fn barrier_lease_expiring(plan: PlanHash, expires_at: Timestamp) -> ExecutionBarrier {
        let mut barrier = barrier(plan);
        if let Some(lease) = barrier.leases.first_mut() {
            lease.expires_at = Some(expires_at);
        }
        barrier
    }

    // M06.6: even with a valid Allow, a capability whose lease-derived expiry is at or
    // before `now` is refused at the spend seam and the op never executes (no marker).
    // This is the only spend refusal reachable end-to-end: a freshly derived capability
    // is always barrier-bound and op-exact, so NotBoundToBarrier / OpMismatch cannot
    // arise at the seam (those are covered at the core layer).
    #[test]
    fn expired_capability_is_refused_and_does_not_execute() -> Result<(), PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let guarded = GuardedExecutor::new(MarkerExecutor);
        let result = guarded.spend_barrier(
            &barrier_lease_expiring(plan()?, Timestamp(10)),
            "release.promote",
            &stages,
            std::slice::from_ref(&allow(plan()?)),
            POLICY,
            Timestamp(10),
            &op(),
        );
        assert_eq!(
            result,
            Err(SpendError::CapabilityRefused(
                CapabilitySpendRefusal::Expired {
                    expires_at: Timestamp(10),
                    now: Timestamp(10),
                }
            ))
        );
        Ok(())
    }

    // A bound, non-expired allow authorizes the barrier and the op runs exactly
    // once (the executor marker is produced).
    #[test]
    fn authorized_barrier_runs_the_op() -> Result<(), PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let guarded = GuardedExecutor::new(MarkerExecutor);
        let result = guarded.spend_barrier(
            &barrier(plan()?),
            "release.promote",
            &stages,
            std::slice::from_ref(&allow(plan()?)),
            POLICY,
            Timestamp(10),
            &op(),
        );
        assert_eq!(result, Ok(vec!["executed".to_owned()]));
        Ok(())
    }

    #[test]
    fn service_call_executes_authorized_barrier() -> Result<(), PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let barrier = barrier(plan()?);
        let decisions = vec![allow(plan()?)];
        let op = op();
        let guarded = GuardedExecutor::new(MarkerExecutor);

        let result = guarded.call(request(&barrier, &stages, &decisions, &op, Timestamp(10)));

        assert_eq!(
            result,
            Ok(ExecutionOutcome {
                produced_refs: vec!["executed".to_owned()]
            })
        );
        Ok(())
    }

    #[test]
    fn service_call_refuses_unauthorized_without_execution() -> Result<(), PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let barrier = barrier(plan()?);
        let decisions = Vec::new();
        let op = op();
        let guarded = GuardedExecutor::new(MarkerExecutor);

        let result = guarded.call(request(&barrier, &stages, &decisions, &op, Timestamp(10)));

        assert!(matches!(result, Err(SpendError::Unauthorized(_))));
        Ok(())
    }

    #[test]
    fn service_call_refuses_expired_capability_without_execution() -> Result<(), PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let barrier = barrier_lease_expiring(plan()?, Timestamp(10));
        let decisions = vec![allow(plan()?)];
        let op = op();
        let guarded = GuardedExecutor::new(MarkerExecutor);

        let result = guarded.call(request(&barrier, &stages, &decisions, &op, Timestamp(10)));

        assert_eq!(
            result,
            Err(SpendError::CapabilityRefused(
                CapabilitySpendRefusal::Expired {
                    expires_at: Timestamp(10),
                    now: Timestamp(10),
                }
            ))
        );
        Ok(())
    }

    #[test]
    fn legacy_spend_and_service_call_are_equivalent() -> Result<(), PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let barrier = barrier(plan()?);
        let decisions = vec![allow(plan()?)];
        let op = op();
        let guarded = GuardedExecutor::new(MarkerExecutor);

        let legacy = guarded.spend_barrier(
            &barrier,
            "release.promote",
            &stages,
            &decisions,
            POLICY,
            Timestamp(10),
            &op,
        );
        let service = guarded
            .call(request(&barrier, &stages, &decisions, &op, Timestamp(10)))
            .map(|outcome| outcome.produced_refs);

        assert_eq!(legacy, service);
        Ok(())
    }
}
