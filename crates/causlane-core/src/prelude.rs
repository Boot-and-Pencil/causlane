//! Common imports for users of the Causlane kernel.

pub use crate::kernel::{
    admit_call, can_commit_observed_truth, claim_modes_conflict, mergeable,
    requires_execution_barrier, select_frontier, KernelContracts,
};
pub use crate::ports::{AuditLogPort, ConstraintProviderPort, ExecutorPort, PlannerPort};
pub use crate::protocol::{
    ActionCall, ActionId, ActionPlan, AuditEvent, AuditEventId, AuditEventKind, BundleHash,
    ClaimMode, ConsequenceProfile, ConstraintEpoch, ExecutionBarrier, FactKind, GraphIndex,
    GraphNode, ImpactSetHash, LaneCapacity, LaneId, LeaseId, LeaseRef, LeaseTable, Op, OpId,
    PlanHash, PredicateId, ResourceId, Scope,
};
