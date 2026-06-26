//! Executor adapters for the semantic [`ExecutorPort`] seam.

use causlane_core::{ExecutionCapability, ExecutorPort, Op};

/// An executor adapter that performs no side effects and returns no results.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoopExecutor;

impl ExecutorPort for NoopExecutor {
    type Error = core::convert::Infallible;

    fn execute(
        &self,
        _op: &Op,
        _capability: &ExecutionCapability,
    ) -> Result<Vec<String>, Self::Error> {
        Ok(Vec::new())
    }
}

/// Adapts a closure to [`ExecutorPort`] for local composition and tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionExecutor<F> {
    handler: F,
}

impl<F> FunctionExecutor<F> {
    /// Create a closure-backed executor adapter.
    #[must_use]
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F, Error> ExecutorPort for FunctionExecutor<F>
where
    F: Fn(&Op, &ExecutionCapability) -> Result<Vec<String>, Error>,
{
    type Error = Error;

    fn execute(
        &self,
        op: &Op,
        capability: &ExecutionCapability,
    ) -> Result<Vec<String>, Self::Error> {
        (self.handler)(op, capability)
    }
}

#[cfg(test)]
mod tests {
    use super::{FunctionExecutor, NoopExecutor};
    use causlane_core::{
        ActionId, AuditEventId, ClaimMode, ConstraintEpoch, EffectSignature, ExecutionBarrier,
        ExecutionCapability, ExecutionCapabilityError, ExecutorPort, ImpactHardness, ImpactSetHash,
        LeaseId, LeaseRef, Op, PlanHash, PlanHashError, ResourceId, Scope,
    };
    use core::convert::Infallible;

    #[derive(Debug, PartialEq, Eq)]
    enum TestError {
        PlanHash,
        Capability,
    }

    impl From<PlanHashError> for TestError {
        fn from(_error: PlanHashError) -> Self {
            Self::PlanHash
        }
    }

    impl From<ExecutionCapabilityError> for TestError {
        fn from(_error: ExecutionCapabilityError) -> Self {
            Self::Capability
        }
    }

    fn plan() -> Result<PlanHash, PlanHashError> {
        PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    }

    fn op(index: u32) -> Op {
        Op {
            index,
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

    fn barrier(plan: PlanHash) -> ExecutionBarrier {
        ExecutionBarrier {
            barrier_id: AuditEventId("barrier".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: plan.clone(),
            op_indexes: vec![0],
            impact_set_hash: ImpactSetHash("impact".to_owned()),
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

    #[test]
    fn noop_executor_returns_no_refs_under_derived_capability() -> Result<(), TestError> {
        let barrier = barrier(plan()?);
        let capability = ExecutionCapability::derive_from_barrier(&barrier, 0)?;

        assert_eq!(NoopExecutor.execute(&op(0), &capability), Ok(Vec::new()));
        Ok(())
    }

    #[test]
    fn function_executor_receives_exact_op_and_capability() -> Result<(), TestError> {
        let barrier = barrier(plan()?);
        let capability = ExecutionCapability::derive_from_barrier(&barrier, 0)?;
        let executor = FunctionExecutor::new(
            |op: &Op, capability: &ExecutionCapability| -> Result<Vec<String>, Infallible> {
                Ok(vec![format!(
                    "op:{} capability:{}",
                    op.index, capability.op_index
                )])
            },
        );

        assert_eq!(
            executor.execute(&op(0), &capability),
            Ok(vec!["op:0 capability:0".to_owned()])
        );
        Ok(())
    }
}
