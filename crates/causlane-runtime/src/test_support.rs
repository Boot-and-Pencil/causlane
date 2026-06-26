//! Shared runtime test fixtures.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use causlane_core::{
    ActionId, AuditEventId, AuthzDecision, AuthzDecisionRef, AuthzPolicy, ClaimMode,
    ConstraintEpoch, EffectSignature, ExecutionBarrier, ExecutionCapability, ExecutorPort,
    ImpactHardness, ImpactSetHash, LeaseId, LeaseRef, Op, PlanHash, PlanHashError, ResourceId,
    Scope, Timestamp,
};

use crate::guarded_executor::GuardedExecutionJob;

/// Authz policy used by guarded execution adapter fixtures.
pub const POLICY: AuthzPolicy<'static> = AuthzPolicy {
    id: "p",
    version: "1",
    max_age: None,
};

/// Executor that marks successful entry with op/capability indexes.
pub struct MarkerExecutor;

impl ExecutorPort for MarkerExecutor {
    type Error = core::convert::Infallible;

    fn execute(
        &self,
        op: &Op,
        capability: &ExecutionCapability,
    ) -> Result<Vec<String>, Self::Error> {
        Ok(vec![format!(
            "executed:{}:{}",
            op.index, capability.op_index
        )])
    }
}

/// Executor that records successful entry while returning the standard marker.
#[derive(Clone, Default)]
pub struct RecordingExecutor {
    calls: Arc<AtomicUsize>,
}

impl RecordingExecutor {
    /// Number of times [`ExecutorPort::execute`] was reached.
    #[must_use]
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ExecutorPort for RecordingExecutor {
    type Error = core::convert::Infallible;

    fn execute(
        &self,
        op: &Op,
        capability: &ExecutionCapability,
    ) -> Result<Vec<String>, Self::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec![format!(
            "executed:{}:{}",
            op.index, capability.op_index
        )])
    }
}

/// Executor that counts whether denied/refused jobs reached the execution seam.
#[derive(Clone, Default)]
pub struct CountingExecutor {
    calls: Arc<AtomicUsize>,
}

impl CountingExecutor {
    /// Number of times [`ExecutorPort::execute`] was reached.
    #[must_use]
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ExecutorPort for CountingExecutor {
    type Error = core::convert::Infallible;

    fn execute(
        &self,
        _op: &Op,
        _capability: &ExecutionCapability,
    ) -> Result<Vec<String>, Self::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec!["unexpected".to_owned()])
    }
}

/// Stable plan hash fixture.
#[must_use = "test fixture plan hashes are fallible and must be handled"]
pub fn plan() -> Result<PlanHash, PlanHashError> {
    PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
}

/// Execution barrier fixture for op 0.
#[must_use]
pub fn barrier(plan: PlanHash) -> ExecutionBarrier {
    ExecutionBarrier {
        barrier_id: AuditEventId("barrier".to_owned()),
        action_id: ActionId("act".to_owned()),
        plan_hash: plan.clone(),
        op_indexes: vec![0],
        impact_set_hash: ImpactSetHash(
            "sha256:2222222222222222222222222222222222222222222222222222222222222222".to_owned(),
        ),
        witnesses: Vec::new(),
        leases: vec![LeaseRef {
            lease_id: LeaseId("lease".to_owned()),
            resource: ResourceId("resource".to_owned()),
            scope: Scope("scope".to_owned()),
            mode: ClaimMode::ExclusiveWrite,
            amount: 1,
            holder_action_id: ActionId("act".to_owned()),
            holder_plan_hash: plan,
            holder_op_index: Some(0),
            epoch: ConstraintEpoch(0),
            expires_at: None,
            lease_event_id: AuditEventId("lease-event".to_owned()),
        }],
        authz_decision_refs: Vec::new(),
        constraint_snapshot_id: None,
    }
}

/// Execution barrier fixture whose first lease expires at `expires_at`.
#[must_use]
pub fn barrier_lease_expiring(plan: PlanHash, expires_at: Timestamp) -> ExecutionBarrier {
    let mut barrier = barrier(plan);
    if let Some(lease) = barrier.leases.first_mut() {
        lease.expires_at = Some(expires_at);
    }
    barrier
}

/// Allow decision fixture matching [`POLICY`].
#[must_use]
pub fn allow(plan: PlanHash) -> AuthzDecisionRef {
    AuthzDecisionRef {
        decision_event_id: AuditEventId("decision".to_owned()),
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

/// Hard effect op fixture matching the barrier.
#[must_use]
pub fn op() -> Op {
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

/// Guarded execution job fixture.
#[must_use]
pub fn job(
    barrier: ExecutionBarrier,
    decisions: Vec<AuthzDecisionRef>,
    now: Timestamp,
) -> GuardedExecutionJob {
    GuardedExecutionJob {
        barrier,
        predicate_id: "release.promote".to_owned(),
        required_stages: vec!["execution_barrier_logged".to_owned()],
        decisions,
        expected_policy: POLICY.into(),
        now,
        op: op(),
    }
}
