use serde::{Deserialize, Serialize};

use crate::{error::CodegenError, ReceiptObligation};

/// Recorded outcome of a tool run, parsed from a receipt at the serde
/// boundary so coverage derivation branches on a typed value rather than a raw
/// string. Unknown tokens decode to [`ToolRunResult::Unknown`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolRunResult {
    /// Tool ran and the expected result held.
    Pass,
    /// Tool ran and the expected result did not hold.
    Fail,
    /// Lane intentionally not run under the active profile.
    NonBlockingSkipped,
    /// Tool unavailable on this platform.
    UnsupportedOnPlatform,
    /// Negative-control artifact refuted as expected.
    ExpectedFailRefuted,
    /// Negative-control artifact not refuted as expected.
    ExpectedFailNotRefuted,
    /// Generated but no tool has run yet.
    NotRun,
    /// Artifact drifted from its receipt.
    Stale,
    /// No artifact was generated.
    NotGenerated,
    /// Artifact freshly generated, awaiting a tool run.
    Generated,
    /// Legacy token for an awaiting tool run.
    PendingToolRun,
    /// Any token not recognised by this generator version.
    #[serde(other)]
    Unknown,
}

/// Typed scope summary for formal receipts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptScope {
    /// Number of predicates projected into the artifact.
    pub predicates: u32,
    /// Number of scenarios projected into the artifact.
    pub scenarios: u32,
}

/// Receipt shape used by `formal stale-check`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalReceipt {
    /// Receipt schema version.
    pub schema_version: u32,
    /// Receipt kind (`codegen`, `tool_run`, `stale_check`, etc.).
    pub receipt_kind: String,
    /// Artifact kind (`alloy`, `p`, `kani`, `verus`, `lean4`, etc.).
    pub artifact_kind: String,
    /// Optional formal target (`alloy`, `p`, `kani`, `verus`, `lean4`).
    #[serde(default)]
    pub target: Option<String>,
    /// Formal tool or generator.
    pub tool: String,
    /// Tool version.
    pub tool_version: String,
    /// Generator version.
    pub generator_version: String,
    /// Source bundle hash.
    pub source_bundle_hash: String,
    /// Optional formal IR hash.
    #[serde(default)]
    pub formal_ir_hash: Option<String>,
    /// Optional scenario hash.
    pub scenario_hash: Option<String>,
    /// Optional core model hash.
    pub core_model_hash: Option<String>,
    /// Generated artifact hash.
    pub generated_artifact_hash: String,
    /// Command that produced or checked the artifact.
    pub command: String,
    /// Expected tool/check result.
    pub expected_result: Option<String>,
    /// Actual tool/check result (typed at the serde boundary).
    pub actual_result: ToolRunResult,
    /// Process exit code of the tool run, when a tool was actually invoked.
    /// `None` for codegen receipts and for tool-run receipts that have not yet
    /// been refreshed by a real tool invocation. A non-zero code forces the
    /// derived coverage status to `fail` (P0-FM-002).
    #[serde(default)]
    pub exit_code: Option<i64>,
    /// Invariant ids covered by this receipt.
    #[serde(default)]
    pub invariant_ids: Vec<String>,
    /// Concrete per-invariant check obligations the generated artifact carries
    /// (P0-006). Coverage credits a generated lane for an invariant only when a
    /// matching obligation is present, so the matrix is grounded in real checks.
    #[serde(default)]
    pub obligations: Vec<ReceiptObligation>,
    /// Scope description.
    pub scope: ReceiptScope,
    /// Timestamp token supplied by the caller.
    pub checked_at: String,
}

/// Serialize a formal receipt as pretty JSON.
///
/// # Errors
/// Returns [`CodegenError::Receipt`] if serialization fails.
#[must_use = "the receipt JSON must be written or returned"]
pub fn receipt_to_json(receipt: &FormalReceipt) -> Result<String, CodegenError> {
    serde_json::to_string_pretty(receipt).map_err(|err| CodegenError::Receipt(err.to_string()))
}
