//! Replay `explain` diagnostics (M04.4).
//!
//! A [`ReplayExplain`] is the structured, serializable answer to "why did this
//! trace pass or fail replay?". On rejection it carries the exact violated
//! protocol invariant (I-001..I-009, via [`ReplayError::invariant`]), the stable
//! error code, and the causal location (which event / action / scope) — so a
//! devtool or CI gate can point at the precise failing step rather than a Debug
//! string. It reuses the same verify path as [`crate::ReplayVerdict`].

use causlane_contracts::CompiledDispatchBundle;
use serde::Serialize;

use crate::{ReplayError, ReplayTrace};

/// Where in the trace a replay rejection was located. Every field is optional —
/// each error variant populates the pointers it actually carries (an execution
/// barrier error knows the `action_id`; an anchor error knows the projection and
/// anchored `event_id`s; a lease conflict knows the contended `scope`). Empty
/// fields are omitted from the serialized form.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CausalLocation {
    /// The action whose lifecycle the failing event belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    /// The offending event id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// The anchored (observed-truth) event id, for projection-anchor errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_event_id: Option<String>,
    /// The barrier event id, for witness/barrier errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barrier_event_id: Option<String>,
    /// The witness event id, for witness-not-prior errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness_event_id: Option<String>,
    /// The requirement id, for required-witness errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement_id: Option<String>,
    /// The contended scope, for lease/drain errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// The plan hash bound to the failing event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_hash: Option<String>,
    /// The lifecycle stage, for authz errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
}

impl CausalLocation {
    /// Whether no causal pointer was captured (used to omit the field when empty).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Extract the causal pointers a rejection carries.
    #[must_use]
    pub fn from_error(err: &ReplayError) -> Self {
        use ReplayError as E;
        match err {
            E::ExecutionWithoutBarrier {
                action_id,
                plan_hash,
            }
            | E::ObservedWithoutExecution {
                action_id,
                plan_hash,
            } => Self {
                action_id: Some(action_id.clone()),
                plan_hash: Some(plan_hash.clone()),
                ..Self::default()
            },
            E::PlanHashMismatch { action_id }
            | E::MissingRequiredBarrier { action_id }
            | E::MissingDispatchBeforeBarrier { action_id } => Self {
                action_id: Some(action_id.clone()),
                ..Self::default()
            },
            E::EventAfterClosed {
                action_id,
                event_id,
            } => Self {
                action_id: Some(action_id.clone()),
                event_id: Some(event_id.clone()),
                ..Self::default()
            },
            E::UnresolvedAnchorPlanHash { event_id }
            | E::ProjectionWithoutAnchor { event_id }
            | E::MissingBarrierPayload { event_id }
            | E::MissingBarrierImpactSet { event_id }
            | E::CapabilityMissing { event_id }
            | E::CapabilityMismatch { event_id, .. } => Self {
                event_id: Some(event_id.clone()),
                ..Self::default()
            },
            E::AnchorNotObservedTruth {
                event_id,
                anchor_event_id,
            }
            | E::AnchorAttestationMismatch {
                event_id,
                anchor_event_id,
            } => Self {
                event_id: Some(event_id.clone()),
                anchor_event_id: Some(anchor_event_id.clone()),
                ..Self::default()
            },
            E::DrainFenceWithActiveOverlap { event_id, scope } => Self {
                event_id: Some(event_id.clone()),
                scope: Some(scope.clone()),
                ..Self::default()
            },
            E::ConflictingLeases { scope } => Self {
                scope: Some(scope.clone()),
                ..Self::default()
            },
            E::WitnessNotPrior {
                barrier_event_id,
                witness_event_id,
            } => Self {
                barrier_event_id: Some(barrier_event_id.clone()),
                witness_event_id: Some(witness_event_id.clone()),
                ..Self::default()
            },
            E::LegacyWitnessMismatch { barrier_event_id } => Self {
                barrier_event_id: Some(barrier_event_id.clone()),
                ..Self::default()
            },
            E::RequiredWitnessMissing { requirement_id }
            | E::WitnessSelectorMismatch { requirement_id }
            | E::WitnessBindingMismatch { requirement_id }
            | E::WitnessAttestationMismatch { requirement_id } => Self {
                requirement_id: Some(requirement_id.clone()),
                ..Self::default()
            },
            E::AuthzDecisionMissing { stage }
            | E::AuthzDecisionDenied { stage }
            | E::AuthzDecisionExpired { stage }
            | E::AuthzPolicyMismatch { stage }
            | E::AuthzIssuedAfterBarrier { stage }
            | E::AuthzDecisionStale { stage } => Self {
                stage: Some(stage.clone()),
                ..Self::default()
            },
            // Free-form / loading / provenance errors carry their detail in the
            // explain's `error_detail`, not a structured location.
            E::Decode(_)
            | E::BadPlanHash(_)
            | E::BadImpactSetHash { .. }
            | E::MissingTracePredicate
            | E::UnknownPredicate { .. }
            | E::TemplateResolution { .. }
            | E::Lease(_)
            | E::Lifecycle(_)
            | E::BundleHashMismatch { .. }
            | E::MissingTraceBundleHash { .. } => Self::default(),
        }
    }
}

/// A structured explanation of a replay outcome (M04.4). Serializable for CLI
/// `--json` / contract-test assertions; [`Self::to_human`] renders a terse
/// diagnostic for humans.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReplayExplain {
    /// Whether the trace was accepted.
    pub accepted: bool,
    /// The violated protocol invariant id (e.g. `I-001`) when rejected for an
    /// invariant-bearing reason; `None` when accepted or for structural errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invariant: Option<String>,
    /// Stable error code when rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Human-readable error detail when rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<String>,
    /// Where the rejection was located in the trace.
    #[serde(skip_serializing_if = "CausalLocation::is_empty")]
    pub causal_location: CausalLocation,
    /// Invariants the oracle checks for a bundle-bound trace.
    pub checked_invariants: Vec<String>,
    /// Source compiled bundle hash.
    pub bundle_hash: String,
    /// Content hash of the trace document.
    pub trace_hash: String,
}

impl ReplayExplain {
    /// Serialize as pretty JSON.
    ///
    /// # Errors
    /// Returns [`ReplayError::Decode`] if serialization fails.
    #[must_use = "the serialized explain must be used"]
    pub fn to_json_pretty(&self) -> Result<String, ReplayError> {
        serde_json::to_string_pretty(self).map_err(|err| ReplayError::Decode(err.to_string()))
    }

    /// Render a terse human-readable diagnostic.
    #[must_use]
    pub fn to_human(&self) -> String {
        if self.accepted {
            return format!(
                "ok: trace accepted; {} invariants checked against bundle {}",
                self.checked_invariants.len(),
                self.bundle_hash
            );
        }
        let invariant = self.invariant.as_deref().unwrap_or("(structural)");
        let code = self.error_code.as_deref().unwrap_or("(unknown)");
        let mut out = format!("rejected: {invariant} violated [{code}]");
        let loc = &self.causal_location;
        let mut parts = Vec::new();
        if let Some(value) = &loc.action_id {
            parts.push(format!("action={value}"));
        }
        if let Some(value) = &loc.event_id {
            parts.push(format!("event={value}"));
        }
        if let Some(value) = &loc.anchor_event_id {
            parts.push(format!("anchor={value}"));
        }
        if let Some(value) = &loc.barrier_event_id {
            parts.push(format!("barrier={value}"));
        }
        if let Some(value) = &loc.witness_event_id {
            parts.push(format!("witness={value}"));
        }
        if let Some(value) = &loc.requirement_id {
            parts.push(format!("requirement={value}"));
        }
        if let Some(value) = &loc.scope {
            parts.push(format!("scope={value}"));
        }
        if let Some(value) = &loc.stage {
            parts.push(format!("stage={value}"));
        }
        if !parts.is_empty() {
            out.push_str("\n  at ");
            out.push_str(&parts.join(", "));
        }
        if let Some(detail) = &self.error_detail {
            out.push_str("\n  ");
            out.push_str(detail);
        }
        out
    }
}

impl ReplayTrace {
    /// Verify this trace against a compiled bundle and return a structured
    /// [`ReplayExplain`] (M04.4): the exact violated invariant, stable code and
    /// causal location on rejection, or an acceptance summary. Same verify path
    /// as [`Self::verify_verdict`].
    #[must_use = "the explain output must be used"]
    pub fn verify_explain(&self, bundle: &CompiledDispatchBundle) -> ReplayExplain {
        let outcome = self.verify_replay_outcome(bundle);
        let (invariant, error_code, error_detail, causal_location) = match outcome.error {
            Some(metadata) => {
                let causal_location = CausalLocation::from_error(&metadata.source);
                (
                    metadata.invariant,
                    Some(metadata.stable_error_code),
                    Some(metadata.detail),
                    causal_location,
                )
            }
            None => (None, None, None, CausalLocation::default()),
        };

        ReplayExplain {
            accepted: outcome.accepted,
            invariant,
            error_code,
            error_detail,
            causal_location,
            checked_invariants: outcome.checked_invariants,
            bundle_hash: outcome.bundle_hash,
            trace_hash: outcome.trace_hash,
        }
    }
}
