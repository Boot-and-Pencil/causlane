//! Replay verifier errors and stable error codes.

use core::fmt;

use causlane_core::{LeaseTableError, PlanHashError};

/// Errors produced while loading or verifying a trace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayError {
    /// The JSON trace could not be decoded.
    Decode(String),
    /// A plan hash in the trace was not well-formed.
    BadPlanHash(String),
    /// An anchor referenced a plan hash that could not be resolved.
    UnresolvedAnchorPlanHash {
        /// The projection event whose anchor was unresolved.
        event_id: String,
    },
    /// `execution.started` appeared with no prior barrier.
    ExecutionWithoutBarrier {
        /// Action id.
        action_id: String,
        /// Plan hash.
        plan_hash: String,
    },
    /// `observed_truth.committed` appeared with no prior execution.
    ObservedWithoutExecution {
        /// Action id.
        action_id: String,
        /// Plan hash.
        plan_hash: String,
    },
    /// `projection.emitted` carried no truth anchor.
    ProjectionWithoutAnchor {
        /// Projection event id.
        event_id: String,
    },
    /// A projection anchor did not point at a prior observed truth.
    AnchorNotObservedTruth {
        /// Projection event id.
        event_id: String,
        /// The anchored event id that failed to resolve.
        anchor_event_id: String,
    },
    /// Two active exclusive-write leases overlapped on the same scope.
    ConflictingLeases {
        /// The contended scope.
        scope: String,
    },
    /// Events of one action carried inconsistent plan hashes.
    PlanHashMismatch {
        /// Action id.
        action_id: String,
    },
    /// Bundle-bound replay requires the trace to name a predicate.
    MissingTracePredicate,
    /// The trace names a predicate absent from the compiled bundle.
    UnknownPredicate {
        /// Predicate id.
        predicate: String,
    },
    /// A bundle-required execution barrier is absent.
    MissingRequiredBarrier {
        /// Action id.
        action_id: String,
    },
    /// A bundle-required dispatch log is absent before the barrier.
    MissingDispatchBeforeBarrier {
        /// Action id.
        action_id: String,
    },
    /// A barrier witness reference is absent or points to a non-prior event.
    WitnessNotPrior {
        /// Barrier event id.
        barrier_event_id: String,
        /// Witness event id.
        witness_event_id: String,
    },
    /// A required witness declared by the bundle was not satisfied.
    RequiredWitnessMissing {
        /// Requirement id.
        requirement_id: String,
    },
    /// The deprecated legacy `AuditEvent.witnesses` list on a barrier disagrees
    /// with the authoritative typed `ExecutionBarrier.witnesses` payload.
    LegacyWitnessMismatch {
        /// Barrier event id.
        barrier_event_id: String,
    },
    /// An impact-set hash in the trace was malformed.
    BadImpactSetHash {
        /// Field that was malformed.
        field: String,
    },
    /// `RuntimeExecution` barrier event did not carry normalized payload.
    MissingBarrierPayload {
        /// Barrier event id.
        event_id: String,
    },
    /// `RuntimeExecution` barrier did not bind an impact-set hash.
    MissingBarrierImpactSet {
        /// Barrier event id.
        event_id: String,
    },
    /// `execution.started` did not carry a capability.
    CapabilityMissing {
        /// Execution-start event id.
        event_id: String,
    },
    /// `execution.started` capability did not derive from the prior barrier.
    CapabilityMismatch {
        /// Execution-start event id.
        event_id: String,
        /// Validation error.
        error: String,
    },
    /// A witness ref did not match the selector declared in the bundle.
    WitnessSelectorMismatch {
        /// Requirement id.
        requirement_id: String,
    },
    /// A witness binding did not match the action/plan/impact set.
    WitnessBindingMismatch {
        /// Requirement id.
        requirement_id: String,
    },
    /// Subject/circumstance selector template did not resolve exactly.
    TemplateResolution {
        /// Template expression.
        expression: String,
        /// Resolver error.
        error: String,
    },
    /// Lease-table validation failed.
    Lease(String),
    /// Required authz decision evidence is absent.
    AuthzDecisionMissing {
        /// Stage requiring authz.
        stage: String,
    },
    /// Required authz decision was explicit deny or otherwise invalid.
    AuthzDecisionDenied {
        /// Stage requiring authz.
        stage: String,
    },
    /// A lifecycle transition was forbidden by the reducer.
    Lifecycle(String),
    /// A protocol event for an action appeared after its `lifecycle.closed`.
    EventAfterClosed {
        /// Action id.
        action_id: String,
        /// The offending event id.
        event_id: String,
    },
    /// A drain fence was acquired while a lease still actively overlapped its
    /// scope (I-007).
    DrainFenceWithActiveOverlap {
        /// The drain-fence-acquired event id.
        event_id: String,
        /// The contended fence scope.
        scope: String,
    },
    /// A required authz decision was present for the stage but had expired.
    AuthzDecisionExpired {
        /// Stage requiring authz.
        stage: String,
    },
    /// A required authz decision authorized the action/plan/stage but was issued
    /// under a different policy than the predicate requires (P0-010).
    AuthzPolicyMismatch {
        /// Stage requiring authz.
        stage: String,
    },
    /// A required authz decision was issued after the barrier it authorizes
    /// (forward-dated relative to the barrier's `occurred_at`, P0-010).
    AuthzIssuedAfterBarrier {
        /// Stage requiring authz.
        stage: String,
    },
    /// A required authz decision was older than the policy's freshness bound at
    /// the barrier's evaluation time (ADR-0011 "fresh"), though not expired.
    AuthzDecisionStale {
        /// Stage requiring authz.
        stage: String,
    },
    /// The trace declared a bundle hash that does not match the compiled bundle.
    BundleHashMismatch {
        /// Expected (compiled bundle) hash.
        expected: String,
        /// Actual (trace-declared) hash.
        actual: String,
    },
    /// Strict bundle-bound replay required the trace to carry a bundle hash, but
    /// the trace declared none (P0-005: an unbound trace must not be accepted as
    /// evidence for a specific compiled bundle).
    MissingTraceBundleHash {
        /// The compiled bundle the trace was supposed to be bound to.
        expected: String,
    },
    /// A typed witness ref claimed a fact (`fact_kind`/`scope`) the producer
    /// event did not itself attest — either the producer recorded a different
    /// fact or carried no attestation at all (P0-004: no self-attestation).
    WitnessAttestationMismatch {
        /// Requirement id whose witness was not grounded in its producer event.
        requirement_id: String,
    },
    /// A projection anchor claimed a fact the anchored observed-truth event did
    /// not attest (P0-004: a projection cannot self-assert the truth it derives
    /// from).
    AnchorAttestationMismatch {
        /// Projection event id.
        event_id: String,
        /// The anchored observed-truth event id.
        anchor_event_id: String,
    },
}

/// Stable replay error codes used by scenario matrices and CI gates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayErrorCode {
    /// Trace decode/schema error.
    Decode,
    /// Malformed plan hash.
    BadPlanHash,
    /// Anchor plan hash could not be resolved.
    UnresolvedAnchorPlanHash,
    /// `execution.started` occurred without prior barrier.
    ExecutionWithoutBarrier,
    /// `observed_truth.committed` occurred without execution.
    ObservedWithoutExecution,
    /// Projection carried no anchor.
    ProjectionWithoutAnchor,
    /// Projection anchor did not resolve to observed truth.
    AnchorNotObservedTruth,
    /// Active leases conflict.
    ConflictingLeases,
    /// Events carry inconsistent plan hashes.
    PlanHashMismatch,
    /// Bundle-bound trace omitted predicate.
    MissingTracePredicate,
    /// Trace predicate is absent from bundle.
    UnknownPredicate,
    /// Required bundle barrier is absent.
    MissingRequiredBarrier,
    /// Dispatch log is missing before barrier.
    MissingDispatchBeforeBarrier,
    /// Witness was absent or not prior.
    WitnessNotPrior,
    /// Required witness was missing.
    RequiredWitnessMissing,
    /// Legacy witness list disagrees with the typed barrier payload.
    LegacyWitnessMismatch,
    /// Malformed impact-set hash.
    BadImpactSetHash,
    /// Barrier payload missing.
    MissingBarrierPayload,
    /// Barrier impact-set hash missing.
    MissingBarrierImpactSet,
    /// Execution capability missing.
    CapabilityMissing,
    /// Execution capability mismatch.
    CapabilityMismatch,
    /// Witness selector mismatch.
    WitnessSelectorMismatch,
    /// Witness binding mismatch.
    WitnessBindingMismatch,
    /// Template resolution failed.
    TemplateResolution,
    /// Lease validation failed.
    Lease,
    /// Required authz decision missing.
    AuthzDecisionMissing,
    /// Authz decision denied/invalid.
    AuthzDecisionDenied,
    /// Lifecycle transition failed.
    Lifecycle,
    /// A protocol event appeared after `lifecycle.closed`.
    EventAfterClosed,
    /// Required authz decision had expired.
    AuthzDecisionExpired,
    /// Required authz decision was issued under the wrong policy.
    AuthzPolicyMismatch,
    /// Required authz decision was issued after the barrier it authorizes.
    AuthzIssuedAfterBarrier,
    /// Required authz decision was older than the policy's freshness bound.
    AuthzDecisionStale,
    /// Trace-declared bundle hash did not match the compiled bundle.
    BundleHashMismatch,
    /// A drain fence was acquired with an active overlapping lease.
    DrainFenceWithActiveOverlap,
    /// Strict bundle-bound replay required a trace bundle hash that was absent.
    MissingTraceBundleHash,
    /// A witness ref claimed a fact its producer event did not attest.
    WitnessAttestationMismatch,
    /// A projection anchor claimed a fact its observed-truth event did not attest.
    AnchorAttestationMismatch,
}

impl ReplayErrorCode {
    /// Return the stable string token.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Decode => "Decode",
            Self::BadPlanHash => "BadPlanHash",
            Self::UnresolvedAnchorPlanHash => "UnresolvedAnchorPlanHash",
            Self::ExecutionWithoutBarrier => "ExecutionWithoutBarrier",
            Self::ObservedWithoutExecution => "ObservedWithoutExecution",
            Self::ProjectionWithoutAnchor => "ProjectionWithoutAnchor",
            Self::AnchorNotObservedTruth => "AnchorNotObservedTruth",
            Self::ConflictingLeases => "ConflictingLeases",
            Self::PlanHashMismatch => "PlanHashMismatch",
            Self::MissingTracePredicate => "MissingTracePredicate",
            Self::UnknownPredicate => "UnknownPredicate",
            Self::MissingRequiredBarrier => "MissingRequiredBarrier",
            Self::MissingDispatchBeforeBarrier => "MissingDispatchBeforeBarrier",
            Self::WitnessNotPrior => "WitnessNotPrior",
            Self::RequiredWitnessMissing => "RequiredWitnessMissing",
            Self::LegacyWitnessMismatch => "LegacyWitnessMismatch",
            Self::BadImpactSetHash => "BadImpactSetHash",
            Self::MissingBarrierPayload => "MissingBarrierPayload",
            Self::MissingBarrierImpactSet => "MissingBarrierImpactSet",
            Self::CapabilityMissing => "CapabilityMissing",
            Self::CapabilityMismatch => "CapabilityMismatch",
            Self::WitnessSelectorMismatch => "WitnessSelectorMismatch",
            Self::WitnessBindingMismatch => "WitnessBindingMismatch",
            Self::TemplateResolution => "TemplateResolution",
            Self::Lease => "Lease",
            Self::AuthzDecisionMissing => "AuthzDecisionMissing",
            Self::AuthzDecisionDenied => "AuthzDecisionDenied",
            Self::Lifecycle => "Lifecycle",
            Self::EventAfterClosed => "EventAfterClosed",
            Self::AuthzDecisionExpired => "AuthzDecisionExpired",
            Self::AuthzPolicyMismatch => "AuthzPolicyMismatch",
            Self::AuthzIssuedAfterBarrier => "AuthzIssuedAfterBarrier",
            Self::AuthzDecisionStale => "AuthzDecisionStale",
            Self::BundleHashMismatch => "BundleHashMismatch",
            Self::DrainFenceWithActiveOverlap => "DrainFenceWithActiveOverlap",
            Self::MissingTraceBundleHash => "MissingTraceBundleHash",
            Self::WitnessAttestationMismatch => "WitnessAttestationMismatch",
            Self::AnchorAttestationMismatch => "AnchorAttestationMismatch",
        }
    }
}

impl ReplayError {
    /// Return the stable error code.
    #[must_use]
    pub fn code(&self) -> ReplayErrorCode {
        match self {
            Self::Decode(_) => ReplayErrorCode::Decode,
            Self::BadPlanHash(_) => ReplayErrorCode::BadPlanHash,
            Self::UnresolvedAnchorPlanHash { .. } => ReplayErrorCode::UnresolvedAnchorPlanHash,
            Self::ExecutionWithoutBarrier { .. } => ReplayErrorCode::ExecutionWithoutBarrier,
            Self::ObservedWithoutExecution { .. } => ReplayErrorCode::ObservedWithoutExecution,
            Self::ProjectionWithoutAnchor { .. } => ReplayErrorCode::ProjectionWithoutAnchor,
            Self::AnchorNotObservedTruth { .. } => ReplayErrorCode::AnchorNotObservedTruth,
            Self::ConflictingLeases { .. } => ReplayErrorCode::ConflictingLeases,
            Self::PlanHashMismatch { .. } => ReplayErrorCode::PlanHashMismatch,
            Self::MissingTracePredicate => ReplayErrorCode::MissingTracePredicate,
            Self::UnknownPredicate { .. } => ReplayErrorCode::UnknownPredicate,
            Self::MissingRequiredBarrier { .. } => ReplayErrorCode::MissingRequiredBarrier,
            Self::MissingDispatchBeforeBarrier { .. } => {
                ReplayErrorCode::MissingDispatchBeforeBarrier
            }
            Self::WitnessNotPrior { .. } => ReplayErrorCode::WitnessNotPrior,
            Self::RequiredWitnessMissing { .. } => ReplayErrorCode::RequiredWitnessMissing,
            Self::LegacyWitnessMismatch { .. } => ReplayErrorCode::LegacyWitnessMismatch,
            Self::BadImpactSetHash { .. } => ReplayErrorCode::BadImpactSetHash,
            Self::MissingBarrierPayload { .. } => ReplayErrorCode::MissingBarrierPayload,
            Self::MissingBarrierImpactSet { .. } => ReplayErrorCode::MissingBarrierImpactSet,
            Self::CapabilityMissing { .. } => ReplayErrorCode::CapabilityMissing,
            Self::CapabilityMismatch { .. } => ReplayErrorCode::CapabilityMismatch,
            Self::WitnessSelectorMismatch { .. } => ReplayErrorCode::WitnessSelectorMismatch,
            Self::WitnessBindingMismatch { .. } => ReplayErrorCode::WitnessBindingMismatch,
            Self::TemplateResolution { .. } => ReplayErrorCode::TemplateResolution,
            Self::Lease(_) => ReplayErrorCode::Lease,
            Self::AuthzDecisionMissing { .. } => ReplayErrorCode::AuthzDecisionMissing,
            Self::AuthzDecisionDenied { .. } => ReplayErrorCode::AuthzDecisionDenied,
            Self::Lifecycle(_) => ReplayErrorCode::Lifecycle,
            Self::EventAfterClosed { .. } => ReplayErrorCode::EventAfterClosed,
            Self::AuthzDecisionExpired { .. } => ReplayErrorCode::AuthzDecisionExpired,
            Self::AuthzPolicyMismatch { .. } => ReplayErrorCode::AuthzPolicyMismatch,
            Self::AuthzIssuedAfterBarrier { .. } => ReplayErrorCode::AuthzIssuedAfterBarrier,
            Self::AuthzDecisionStale { .. } => ReplayErrorCode::AuthzDecisionStale,
            Self::BundleHashMismatch { .. } => ReplayErrorCode::BundleHashMismatch,
            Self::DrainFenceWithActiveOverlap { .. } => {
                ReplayErrorCode::DrainFenceWithActiveOverlap
            }
            Self::MissingTraceBundleHash { .. } => ReplayErrorCode::MissingTraceBundleHash,
            Self::WitnessAttestationMismatch { .. } => ReplayErrorCode::WitnessAttestationMismatch,
            Self::AnchorAttestationMismatch { .. } => ReplayErrorCode::AnchorAttestationMismatch,
        }
    }

    /// Return the stable string token for this error.
    #[must_use]
    pub fn code_token(&self) -> &'static str {
        self.code().as_str()
    }

    /// Return the protocol invariant id (I-001..I-009) this error violates, for
    /// replay `explain` diagnostics (M04.4). Structural / loading / provenance and
    /// generic-wrapper errors are not tied to a single protocol invariant and
    /// return `None`.
    #[must_use]
    pub fn invariant(&self) -> Option<&'static str> {
        match self {
            // I-001 — execution requires a prior write-ahead barrier (and the
            // barrier-derived execution capability).
            Self::ExecutionWithoutBarrier { .. }
            | Self::MissingRequiredBarrier { .. }
            | Self::MissingDispatchBeforeBarrier { .. }
            | Self::MissingBarrierPayload { .. }
            | Self::MissingBarrierImpactSet { .. }
            | Self::CapabilityMissing { .. }
            | Self::CapabilityMismatch { .. } => Some("I-001"),
            // I-002 — observed truth requires prior execution.
            Self::ObservedWithoutExecution { .. } => Some("I-002"),
            // I-003 — projection requires an observed-truth anchor.
            Self::ProjectionWithoutAnchor { .. }
            | Self::AnchorNotObservedTruth { .. }
            | Self::UnresolvedAnchorPlanHash { .. }
            | Self::AnchorAttestationMismatch { .. } => Some("I-003"),
            // I-006 — mutable lease conflicts require a verified merge protocol.
            Self::ConflictingLeases { .. } => Some("I-006"),
            // I-007 — drain fences require prior overlapping leases to clear.
            Self::DrainFenceWithActiveOverlap { .. } => Some("I-007"),
            // I-008 — no event may mutate lifecycle after terminal close.
            Self::EventAfterClosed { .. } => Some("I-008"),
            // I-009 — witness/authz evidence must bind exact action, plan, scope.
            Self::RequiredWitnessMissing { .. }
            | Self::WitnessNotPrior { .. }
            | Self::WitnessSelectorMismatch { .. }
            | Self::WitnessBindingMismatch { .. }
            | Self::WitnessAttestationMismatch { .. }
            | Self::LegacyWitnessMismatch { .. }
            | Self::AuthzDecisionMissing { .. }
            | Self::AuthzDecisionDenied { .. }
            | Self::AuthzDecisionExpired { .. }
            | Self::AuthzPolicyMismatch { .. }
            | Self::AuthzIssuedAfterBarrier { .. }
            | Self::AuthzDecisionStale { .. } => Some("I-009"),
            // Structural / loading / provenance / generic-wrapper errors are not
            // tied to a single protocol invariant.
            Self::Decode(_)
            | Self::BadPlanHash(_)
            | Self::BadImpactSetHash { .. }
            | Self::PlanHashMismatch { .. }
            | Self::MissingTracePredicate
            | Self::UnknownPredicate { .. }
            | Self::TemplateResolution { .. }
            | Self::Lease(_)
            | Self::Lifecycle(_)
            | Self::BundleHashMismatch { .. }
            | Self::MissingTraceBundleHash { .. } => None,
        }
    }
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {self:?}", self.code_token())
    }
}

impl std::error::Error for ReplayError {}

impl From<PlanHashError> for ReplayError {
    fn from(err: PlanHashError) -> Self {
        ReplayError::BadPlanHash(format!("{err:?}"))
    }
}

impl From<LeaseTableError> for ReplayError {
    fn from(err: LeaseTableError) -> Self {
        match err {
            LeaseTableError::Conflict { scope, .. } => ReplayError::ConflictingLeases {
                scope: scope.0.clone(),
            },
            other => ReplayError::Lease(format!("{other:?}")),
        }
    }
}
