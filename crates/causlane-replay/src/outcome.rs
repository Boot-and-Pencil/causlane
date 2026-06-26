//! Shared bundle-bound replay outcome diagnostics.

use causlane_contracts::{canonical_json_hash, CompiledDispatchBundle};
use serde::Serialize;

use crate::{ReplayError, ReplayTrace};

/// Invariants the replay oracle checks for a bundle-bound trace. I-007 (drain
/// fence vs active leases) is enforced in `verify_events` and reported here.
const CHECKED_INVARIANTS: [&str; 7] = [
    "I-001", "I-002", "I-003", "I-006", "I-007", "I-008", "I-009",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplayErrorMetadata {
    pub(crate) source: ReplayError,
    pub(crate) stable_error_code: String,
    pub(crate) invariant: Option<String>,
    pub(crate) detail: String,
}

impl ReplayErrorMetadata {
    fn from_error(err: ReplayError) -> Self {
        Self {
            stable_error_code: err.code_token().to_owned(),
            invariant: err.invariant().map(ToOwned::to_owned),
            detail: err.to_string(),
            source: err,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplayOutcome {
    pub(crate) accepted: bool,
    pub(crate) error: Option<ReplayErrorMetadata>,
    pub(crate) checked_invariants: Vec<String>,
    pub(crate) bundle_hash: String,
    pub(crate) trace_hash: String,
}

impl ReplayOutcome {
    fn from_verify_result(
        result: Result<(), ReplayError>,
        checked_invariants: Vec<String>,
        bundle_hash: String,
        trace_hash: String,
    ) -> Self {
        match result {
            Ok(()) => Self {
                accepted: true,
                error: None,
                checked_invariants,
                bundle_hash,
                trace_hash,
            },
            Err(err) => Self {
                accepted: false,
                error: Some(ReplayErrorMetadata::from_error(err)),
                checked_invariants,
                bundle_hash,
                trace_hash,
            },
        }
    }
}

fn checked_invariants() -> Vec<String> {
    CHECKED_INVARIANTS
        .iter()
        .map(|invariant| (*invariant).to_owned())
        .collect()
}

impl ReplayTrace {
    pub(crate) fn verify_replay_outcome(&self, bundle: &CompiledDispatchBundle) -> ReplayOutcome {
        ReplayOutcome::from_verify_result(
            self.verify_with_bundle(bundle),
            checked_invariants(),
            bundle.bundle_hash.0.clone(),
            canonical_json_hash(self).unwrap_or_default(),
        )
    }

    /// Verify and return a structured, serializable verdict (§7.4 `ReplayOracle`).
    #[must_use = "the verdict must be used"]
    pub fn verify_verdict(&self, bundle: &CompiledDispatchBundle) -> ReplayVerdict {
        let ReplayOutcome {
            accepted,
            error,
            checked_invariants,
            bundle_hash,
            trace_hash,
        } = self.verify_replay_outcome(bundle);
        let (stable_error_code, error_detail) = match error {
            Some(metadata) => (Some(metadata.stable_error_code), Some(metadata.detail)),
            None => (None, None),
        };

        ReplayVerdict {
            accepted,
            stable_error_code,
            error_detail,
            checked_invariants,
            bundle_hash,
            trace_hash,
        }
    }
}

/// A serializable verdict from verifying a trace against a compiled bundle
/// (§7.4). Suitable for embedding in receipts and coverage provenance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReplayVerdict {
    /// Whether the trace was accepted by the oracle.
    pub accepted: bool,
    /// Stable error code when rejected (`None` when accepted).
    pub stable_error_code: Option<String>,
    /// Human-readable error detail when rejected.
    pub error_detail: Option<String>,
    /// Invariants the oracle checks for a bundle-bound trace.
    pub checked_invariants: Vec<String>,
    /// Source compiled bundle hash.
    pub bundle_hash: String,
    /// Content hash of the trace document.
    pub trace_hash: String,
}
