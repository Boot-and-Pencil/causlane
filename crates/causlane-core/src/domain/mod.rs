//! Domain model.

pub mod action;
pub mod approval;
pub mod approval_stepup;
pub mod audit;
pub mod authz;
pub mod authz_policy;
pub mod capability;
pub mod constraint;
pub mod constraint_runtime;
pub mod constraint_spec;
pub mod constraint_update;
pub mod drain;
pub mod drain_protocol;
pub mod effect;
pub mod frontier;
pub mod graph_index;
pub mod hashing;
pub mod lane;
pub mod lifecycle;
pub mod overlay;
pub mod projection_authz;
pub mod redaction;
pub mod redaction_policy;
pub mod routing;
pub mod tier;
pub mod tracing;
pub mod why_not_parallel;

pub use crate::kernel::{
    apply_capacity, apply_redaction, apply_to_snapshot, approval_gate, approval_gate_stepup,
    at_safe_point, authz_gate, canonical_capability_id, capability_binding_matches,
    claim_modes_conflict, classify_approval, classify_approval_stepup, classify_authz_decision,
    classify_field, compile_redaction_policy, decide_authz_policy, drains_independent,
    epoch_advances, lane_admits, lease_covers_claim, lease_current, lifecycle_class_for_profile,
    mergeable, next_epoch, op_admissible_during_drain, pair_conflict,
    projection_anchor_source_is_observed, reached_tier, read_authz_gate, reason_from_frontier,
    reduce_lifecycle, resolve_constraints, route_consistent_with_profile, select_frontier,
    trace_span_from_audit_event, trace_span_kind_from_audit_event_kind, truth_rewrite_of,
    why_not_parallel, why_not_parallel_from_index,
};
pub use crate::protocol::{
    ActionCall, ActionId, ActionPlan, ActiveScopeHolder, ApprovalDenyReason, ApprovalOutcome,
    ApprovalRef, ApprovalRequirement, ApprovalVerb, AssuranceLevel, AuditEvent, AuditEventId,
    AuditEventKind, AuthzAttestation, AuthzDecision, AuthzDecisionRef, AuthzDecisionVerdict,
    AuthzDenyReason, AuthzGateOutcome, AuthzPolicy, AuthzPolicyId, AuthzPolicyModel,
    AuthzPolicyOutcome, BundleHash, CapabilityAttestation, CapabilityId, CapabilitySpendRefusal,
    CapabilitySpendRequest, ClaimMode, ClassifiedField, CommittedTruth, ConflictDomain,
    ConsequenceProfile, ConstraintBlocker, ConstraintDecision, ConstraintEpoch, ConstraintId,
    ConstraintKind, ConstraintSnapshot, ConstraintSpec, ConstraintUpdate, ConstraintViolation,
    ContentHash, CorrelationId, DrainFenceCheck, DrainRequest, DrainTarget, DrainedWriteScope,
    EffectSignature, EventHash, ExecutionBarrier, ExecutionCapability, ExecutionCapabilityError,
    FactKind, FieldPath, FieldVisibility, FrontierBlock, FrontierRejection, FrontierSelection,
    GraphIndex, GraphNode, ImpactHardness, ImpactSetHash, Lane, LaneAdmission, LaneCapacity,
    LaneId, LaneRejection, Lease, LeaseId, LeaseRef, LeaseTable, LeaseTableError, LifecycleClass,
    LifecycleStage, LifecycleViolation, NotParallelReason, ObligationSet, Op, OpId, PairConflict,
    PartitionKey, PartitionRoute, PlanHash, PlanHashError, PredicateId, ProjectionReadRequest,
    RedactionClass, RedactionClassPolicy, RedactionPolicy, RedactionSurface, RedactionView,
    ResourceClaim, ResourceId, RouteId, RuntimeUpdate, RuntimeUpdateKind, Scope,
    SurfaceRedactionProfile, Tier, Timestamp, TraceAttribute, TraceSpan, TraceSpanId,
    TraceSpanKind, TruthAnchor, WhyNotParallel, WhyNotParallelInputs, WitnessAttestation,
    WitnessBinding, WitnessKind, WitnessRef, ALL_AUDIT_EVENT_KINDS, MAY_PROJECT_STAGE,
};
