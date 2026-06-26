//! Public facade for `causlane`.
//!
//! This crate stays small: use [`prelude`] for common imports and [`core`] for
//! explicit access to curated `causlane-core` API layers.

#![forbid(unsafe_code)]
#![deny(warnings)]

/// Curated access to the kernel crate's public API layers.
///
/// Testing helpers and lower-level implementation modules remain available only
/// to crates that intentionally depend on `causlane-core`.
///
/// ```compile_fail
/// use causlane::core::testing;
/// ```
pub mod core {
    pub use causlane_core::{kernel, ports, prelude, protocol};
}

/// Common imports from `causlane-core`.
pub mod prelude {
    pub use causlane_core::prelude::{
        admit_call, can_commit_observed_truth, claim_modes_conflict, mergeable,
        requires_execution_barrier, select_frontier, ActionCall, ActionId, ActionPlan, AuditEvent,
        AuditEventId, AuditEventKind, AuditLogPort, BundleHash, ClaimMode, ConsequenceProfile,
        ConstraintEpoch, ConstraintProviderPort, ExecutionBarrier, ExecutorPort, FactKind,
        GraphIndex, GraphNode, ImpactSetHash, KernelContracts, LaneCapacity, LaneId, LeaseId,
        LeaseRef, LeaseTable, Op, OpId, PlanHash, PlannerPort, PredicateId, ResourceId, Scope,
    };
}
