//! Protocol data types for dispatch, audit, constraints and host dispatch.

pub use crate::domain::action::{
    ActionCall, ActionId, ActionPlan, ConsequenceProfile, CorrelationId, Op, PlanHash,
    PlanHashError, PredicateId, RouteId,
};
pub use crate::domain::approval::{
    ApprovalDenyReason, ApprovalOutcome, ApprovalRef, ApprovalVerb, AssuranceLevel,
};
pub use crate::domain::approval_stepup::ApprovalRequirement;
pub use crate::domain::audit::{
    AuditEvent, AuditEventId, AuditEventKind, AuthzAttestation, AuthzDecision, AuthzDecisionRef,
    TruthAnchor, WitnessAttestation, WitnessBinding, WitnessKind, WitnessRef,
    ALL_AUDIT_EVENT_KINDS,
};
pub use crate::domain::authz::{
    AuthzDecisionVerdict, AuthzDenyReason, AuthzGateOutcome, AuthzPolicy,
};
pub use crate::domain::authz_policy::{AuthzPolicyId, AuthzPolicyModel, AuthzPolicyOutcome};
pub use crate::domain::capability::{
    CapabilitySpendRefusal, CapabilitySpendRequest, ExecutionCapabilityError,
};
pub use crate::domain::constraint::{
    CapabilityAttestation, CapabilityId, ClaimMode, ConstraintBlocker, ConstraintDecision,
    ConstraintEpoch, ConstraintId, ConstraintViolation, ExecutionBarrier, ExecutionCapability,
    Lease, LeaseId, LeaseRef, LeaseTable, LeaseTableError, ResourceClaim, ResourceId, Timestamp,
};
pub use crate::domain::constraint_runtime::{RuntimeUpdate, RuntimeUpdateKind};
pub use crate::domain::constraint_spec::{ConstraintKind, ConstraintSnapshot, ConstraintSpec};
pub use crate::domain::constraint_update::{CommittedTruth, ConstraintUpdate};
pub use crate::domain::drain::DrainFenceCheck;
pub use crate::domain::drain_protocol::{DrainRequest, DrainTarget};
pub use crate::domain::effect::{ConflictDomain, EffectSignature, FactKind, ImpactHardness, Scope};
pub use crate::domain::frontier::{FrontierBlock, FrontierRejection, FrontierSelection};
pub use crate::domain::graph_index::{GraphIndex, GraphNode, OpId};
pub use crate::domain::hashing::{BundleHash, ContentHash, EventHash, ImpactSetHash};
pub use crate::domain::lane::{Lane, LaneAdmission, LaneCapacity, LaneId, LaneRejection};
pub use crate::domain::lifecycle::{LifecycleStage, LifecycleViolation};
pub use crate::domain::overlay::ObligationSet;
pub use crate::domain::projection_authz::{ProjectionReadRequest, MAY_PROJECT_STAGE};
pub use crate::domain::redaction::{FieldPath, FieldVisibility, RedactionPolicy, RedactionView};
pub use crate::domain::redaction_policy::{
    ClassifiedField, RedactionClass, RedactionClassPolicy, RedactionSurface,
    SurfaceRedactionProfile,
};
pub use crate::domain::routing::LifecycleClass;
pub use crate::domain::tier::Tier;
pub use crate::domain::tracing::{TraceAttribute, TraceSpan, TraceSpanId, TraceSpanKind};
pub use crate::domain::why_not_parallel::{
    ActiveScopeHolder, DrainedWriteScope, NotParallelReason, PairConflict, WhyNotParallel,
    WhyNotParallelInputs,
};
pub use crate::integration::{
    HostDispatchContext, HostDispatchError, HostDispatchTicket, HostDispatcherCapabilities,
    HostDrainOutcome, HostEffectClass, HostEffectOutcome, HostRuntimeProfile, HostTaskSpec,
    PartitionKey, PartitionRoute, CAUSLANE_HOST_API_VERSION,
};
