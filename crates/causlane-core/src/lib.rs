//! Pure domain/application kernel for causlane.
//!
//! This crate should not depend on async runtimes, databases, HTTP, workflow engines,
//! policy engines or observability frameworks.

#![forbid(unsafe_code)]
#![deny(warnings)]

pub mod application;
pub mod contract;
pub mod domain;
pub mod integration;
pub mod kernel;
pub mod ports;
pub mod prelude;
pub mod protocol;
pub mod testing;

pub use kernel::{
    admit_call, apply_capacity, apply_redaction, apply_to_snapshot, approval_gate,
    approval_gate_stepup, at_safe_point, authz_gate, can_commit_observed_truth,
    canonical_capability_id, capability_binding_matches, claim_modes_conflict, classify_approval,
    classify_approval_stepup, classify_authz_decision, classify_field, compile_redaction_policy,
    decide_authz_policy, drains_independent, epoch_advances, lane_admits, lease_covers_claim,
    lease_current, lifecycle_class_for_profile, mergeable, next_epoch, op_admissible_during_drain,
    pair_conflict, projection_anchor_source_is_observed, reached_tier, read_authz_gate,
    reason_from_frontier, reduce_lifecycle, requires_execution_barrier, resolve_constraints,
    route_consistent_with_profile, select_frontier, trace_span_from_audit_event,
    trace_span_kind_from_audit_event_kind, truth_rewrite_of, validate_host_task, why_not_parallel,
    why_not_parallel_from_index, AuthzEvaluator, CapabilityIssuer, ConflictOracle,
    ConstraintProvider, DispatchAdmission, DrainSemantics, KernelContracts, LifecycleGrammar,
    ScopeOverlap, TruthAnchorResolver,
};
pub use ports::{
    AuditLogPort, ConstraintProviderPort, ExecutorPort, HostDispatchPort, HostEffectHandler,
    PlannerPort, ProjectionPort,
};
pub use protocol::{
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
    GraphIndex, GraphNode, HostDispatchContext, HostDispatchError, HostDispatchTicket,
    HostDispatcherCapabilities, HostDrainOutcome, HostEffectClass, HostEffectOutcome,
    HostRuntimeProfile, HostTaskSpec, ImpactHardness, ImpactSetHash, Lane, LaneAdmission,
    LaneCapacity, LaneId, LaneRejection, Lease, LeaseId, LeaseRef, LeaseTable, LeaseTableError,
    LifecycleClass, LifecycleStage, LifecycleViolation, NotParallelReason, ObligationSet, Op, OpId,
    PairConflict, PartitionKey, PartitionRoute, PlanHash, PlanHashError, PredicateId,
    ProjectionReadRequest, RedactionClass, RedactionClassPolicy, RedactionPolicy, RedactionSurface,
    RedactionView, ResourceClaim, ResourceId, RouteId, RuntimeUpdate, RuntimeUpdateKind, Scope,
    SurfaceRedactionProfile, Tier, Timestamp, TraceAttribute, TraceSpan, TraceSpanId,
    TraceSpanKind, TruthAnchor, WhyNotParallel, WhyNotParallelInputs, WitnessAttestation,
    WitnessBinding, WitnessKind, WitnessRef, ALL_AUDIT_EVENT_KINDS, CAUSLANE_HOST_API_VERSION,
    MAY_PROJECT_STAGE,
};
