//! Testing-oriented re-exports for deterministic examples.

pub use crate::kernel::KernelContracts;
pub use crate::protocol::{
    ActionCall, ActionId, BundleHash, CausalProtocolEvent, CausalProtocolEventId,
    CausalProtocolEventKind, ConstraintEpoch, ExecutionBarrier, ImpactSetHash, LeaseId, LeaseRef,
    PlanHash, ResourceId, Scope,
};
