//! Pure kernel rules and semantic contract traits.

pub use crate::application::use_cases::{
    admit_call, can_commit_observed_truth, requires_execution_barrier, DispatchAdmission,
};
pub use crate::contract::{
    AuthzEvaluator, CapabilityIssuer, ConflictOracle, ConstraintProvider, DrainSemantics,
    KernelContracts, LifecycleGrammar, ScopeOverlap, TruthAnchorResolver,
};
pub use crate::domain::approval::{approval_gate, classify_approval};
pub use crate::domain::approval_stepup::{approval_gate_stepup, classify_approval_stepup};
pub use crate::domain::audit::projection_anchor_source_is_observed;
pub use crate::domain::authz::{authz_gate, classify_authz_decision};
pub use crate::domain::authz_policy::decide_authz_policy;
pub use crate::domain::capability::{canonical_capability_id, capability_binding_matches};
pub use crate::domain::constraint::{claim_modes_conflict, lease_covers_claim, mergeable};
pub use crate::domain::constraint_runtime::{
    apply_capacity, apply_to_snapshot, epoch_advances, lease_current, next_epoch, truth_rewrite_of,
};
pub use crate::domain::constraint_spec::resolve_constraints;
pub use crate::domain::drain_protocol::{
    at_safe_point, drains_independent, op_admissible_during_drain,
};
pub use crate::domain::frontier::select_frontier;
pub use crate::domain::lane::lane_admits;
pub use crate::domain::lifecycle::reduce_lifecycle;
pub use crate::domain::projection_authz::read_authz_gate;
pub use crate::domain::redaction::{apply_redaction, classify_field};
pub use crate::domain::redaction_policy::compile_redaction_policy;
pub use crate::domain::routing::{lifecycle_class_for_profile, route_consistent_with_profile};
pub use crate::domain::tier::reached_tier;
pub use crate::domain::tracing::{
    trace_span_from_audit_event, trace_span_kind_from_audit_event_kind,
};
pub use crate::domain::why_not_parallel::{
    pair_conflict, reason_from_frontier, why_not_parallel, why_not_parallel_from_index,
};
pub use crate::integration::{
    validate_host_context, validate_host_effect_outcome, validate_host_submission,
    validate_host_task,
};
