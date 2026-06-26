use causlane_contracts::content_hash;
use serde::{Deserialize, Serialize};

use crate::{ir::FormalIr, obligations, GENERATOR_VERSION};

/// Formal verification target for a generated artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormalTarget {
    /// Alloy facts/checks.
    Alloy,
    /// P protocol monitor.
    P,
    /// Kani bounded Rust harness.
    Kani,
    /// Verus abstract proof skeleton.
    Verus,
    /// Lean4 generated proof applications.
    Lean4,
}

impl FormalTarget {
    /// Stable target token used in artifact headers and receipts.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FormalTarget::Alloy => "alloy",
            FormalTarget::P => "p",
            FormalTarget::Kani => "kani",
            FormalTarget::Verus => "verus",
            FormalTarget::Lean4 => "lean4",
        }
    }
}

/// One concrete named check that establishes a single invariant on one lane: an
/// Alloy assertion, a P spec, a Kani harness, a Verus proof fn, a Lean4 theorem
/// application, or — for the replay lane — a refuted negative control. Coverage
/// may claim a lane `passed` for an invariant only when a real obligation backs
/// it, so the matrix can never out-run the checks the artifacts actually contain
/// (P0-006).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptObligation {
    /// Invariant id this check establishes (`I-001`..`I-010`).
    pub invariant_id: String,
    /// The check identifier as it appears in the artifact (assertion / spec /
    /// harness / proof-fn name), or the control id for the replay lane.
    pub check_id: String,
}

/// A generated formal artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedArtifact {
    /// Formal target.
    pub target: FormalTarget,
    /// Artifact kind (`facts`, `monitor`, `harness`, `proof`).
    pub artifact_kind: String,
    /// Source bundle hash.
    pub source_bundle_hash: String,
    /// Optional scenario hash.
    pub scenario_hash: Option<String>,
    /// Formal IR hash used to generate this artifact.
    pub formal_ir_hash: String,
    /// Generator version.
    pub generator_version: String,
    /// Invariant ids this lane projects: the bundle/scenario-declared invariants
    /// that this target's obligation table actually carries checks for (per-lane,
    /// not the bundle-wide declared set). This is derived from the generator
    /// obligation projection for the target lane.
    pub invariant_ids: Vec<String>,
    /// Concrete per-invariant check obligations actually present in `text`
    /// (P0-006): the named checks coverage is allowed to credit this artifact
    /// with. Derived from the generator tables, filtered to what `text` contains.
    pub obligations: Vec<ReceiptObligation>,
    /// Artifact text.
    pub text: String,
    /// Hash of `text` (`sha256:...`).
    pub artifact_hash: String,
}

impl GeneratedArtifact {
    /// Construct a generated artifact and compute its text hash.
    #[must_use]
    pub fn new(target: FormalTarget, artifact_kind: &str, ir: &FormalIr, text: String) -> Self {
        let artifact_hash = generated_artifact_hash(&text);
        let obligations = obligations::present_obligations(target, &text);
        Self {
            target,
            artifact_kind: artifact_kind.to_owned(),
            source_bundle_hash: ir.source_bundle_hash.clone(),
            scenario_hash: ir.scenario_hash.clone(),
            formal_ir_hash: ir.formal_ir_hash.clone(),
            generator_version: GENERATOR_VERSION.to_owned(),
            // Per-lane honest projection: the invariants THIS target carries
            // checks for, not the bundle-wide declared set. Keeps the field from
            // claiming invariants a target does not have an obligation for.
            // Coverage credit still flows only through `obligations`.
            invariant_ids: obligations::projected_invariants(target, &ir.invariants),
            obligations,
            text,
            artifact_hash,
        }
    }
}

/// Compute the generated artifact hash (`sha256:...`).
#[must_use]
pub fn generated_artifact_hash(text: &str) -> String {
    content_hash(text.as_bytes()).0
}

pub(crate) fn artifact_header(ir: &FormalIr, target: FormalTarget, artifact_kind: &str) -> String {
    artifact_header_with_prefix(ir, target, artifact_kind, "//")
}

pub(crate) fn artifact_header_with_prefix(
    ir: &FormalIr,
    target: FormalTarget,
    artifact_kind: &str,
    prefix: &str,
) -> String {
    let scenario_hash = ir.scenario_hash.as_deref().unwrap_or("null");
    let expected_result = ir.expected_result.as_deref().unwrap_or("null");
    let expected_error_code = ir.expected_error_code.as_deref().unwrap_or("null");
    // Per-lane projection, matching `GeneratedArtifact::invariant_ids` so the
    // header the stale-check parses agrees with the receipt.
    let projected = obligations::projected_invariants(target, &ir.invariants);
    let invariant_ids = if projected.is_empty() {
        "none".to_owned()
    } else {
        projected.join(",")
    };
    [
        format!("{prefix} GENERATED by causlane-codegen; DO NOT EDIT."),
        format!("{prefix} source_bundle_hash: {}", ir.source_bundle_hash),
        format!("{prefix} formal_ir_hash: {}", ir.formal_ir_hash),
        format!("{prefix} generator_version: {GENERATOR_VERSION}"),
        format!("{prefix} target: {}", target.as_str()),
        format!("{prefix} artifact_kind: {artifact_kind}"),
        format!("{prefix} scenario_hash: {scenario_hash}"),
        format!("{prefix} expected_result: {expected_result}"),
        format!("{prefix} expected_error_code: {expected_error_code}"),
        format!("{prefix} invariant_ids: {invariant_ids}"),
        String::new(),
    ]
    .join("\n")
}
