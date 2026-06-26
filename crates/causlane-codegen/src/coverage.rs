//! Derived formal coverage report (P0-FM-002).
//!
//! The coverage report is a *derived* artifact: every artifact status is
//! computed from the codegen / tool-run receipts and the real process exit
//! codes, never patched to `pass` after the fact. A failing tool run can never
//! be upgraded to a passing coverage status, and a negative control that the
//! replay oracle fails to refute fails the whole report.
//!
//! Statuses are modelled as typed enums; the raw tool-run token is parsed into
//! [`ToolRunResult`] at the serde boundary, so derivation branches on variants
//! rather than raw strings, and per-target accumulation uses a typed struct
//! rather than a string-keyed map.
//!
//! Provenance (H5/M6): the receipts this report derives from are **not
//! cryptographically signed**, so this report is evidence of "the last real tool
//! run on a formal-capable host reported X", not a signed proof. The publication
//! authority is re-deriving it by re-running `tools/formal-verify-all` on a
//! formal-capable host; a committed/loaded receipt must not be presented as proof
//! on its own. See `docs/formal/09-formal-evidence-provenance-and-trust-policy.md`.

use serde::Serialize;

use crate::{obligations, FormalReceipt, FormalTarget, ReceiptObligation, ToolRunResult};

/// Coverage report schema version. Bumped from 1 to 2 when artifact entries
/// gained `exit_code` + `derived_from` provenance and the full status enum.
pub const COVERAGE_SCHEMA_VERSION: u32 = 2;

/// Per-artifact coverage status (P0-FM-002 status policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStatus {
    /// Tool ran and the expected result held.
    Pass,
    /// Tool ran and the expected result did not hold (or non-zero exit).
    Fail,
    /// Negative-control artifact refuted exactly as expected.
    ExpectedFailRefuted,
    /// Negative-control artifact that was not refuted as expected.
    ExpectedFailNotRefuted,
    /// The artifact on disk no longer matches its receipt.
    Stale,
    /// No artifact was generated for this lane.
    NotGenerated,
    /// The artifact was generated but no tool has run against it yet.
    NotRun,
    /// The lane is intentionally not run under this profile.
    NonBlockingSkipped,
    /// The tool is unavailable on this platform.
    UnsupportedOnPlatform,
    /// The receipt could not be parsed / is internally inconsistent.
    InvalidReceipt,
}

impl ArtifactStatus {
    /// `true` when this status counts as positive assurance for the gate.
    #[must_use]
    pub fn holds(self) -> bool {
        matches!(
            self,
            ArtifactStatus::Pass
                | ArtifactStatus::NonBlockingSkipped
                | ArtifactStatus::ExpectedFailRefuted
        )
    }
}

/// A lane-status token used in [`InvariantCoverage`] cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LaneStatus {
    /// A real, discriminating check that actually ran and held.
    Passed,
    /// Refuted by a generated expected-fail artifact.
    NegativeControl,
    /// No concrete runtime hook in this slice; other lanes still cover it.
    NotApplicable,
    /// Generated but the tool run did not (yet) prove this lane.
    PendingToolRun,
    /// Optional proof/spec facet, not a release gate.
    NonBlockingSpec,
}

impl LaneStatus {
    fn proves(self) -> bool {
        matches!(self, LaneStatus::Passed | LaneStatus::NegativeControl)
    }
}

/// Rolled-up status for a single invariant row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvariantStatus {
    /// At least one lane proves the invariant.
    Covered,
    /// Covered, with one or more deferred (non-blocking) lanes.
    CoveredWithDeferredLane,
    /// A lane that should prove the invariant did not (tool run failed).
    Blocked,
    /// No concrete lane applies in this slice.
    NotApplicable,
}

/// Verdict for a negative control executed through the replay oracle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NegativeControlStatus {
    /// Refuted by replay with the declared (or any, if none declared) code.
    RefutedByReplay,
    /// Refuted, but with a different code than declared.
    WrongCode,
    /// Replay accepted a trace that should have failed — control is broken.
    UnexpectedPass,
    /// No declared code and not executed (catalogue only).
    Catalogued,
}

impl NegativeControlStatus {
    fn refuted(self) -> bool {
        matches!(self, NegativeControlStatus::RefutedByReplay)
    }
}

/// Overall report status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    /// Every artifact holds and every negative control was refuted.
    Pass,
    /// At least one artifact or negative control did not hold.
    Fail,
}

/// Map a recorded tool-run result onto a derived artifact status.
fn status_from_result(result: ToolRunResult) -> ArtifactStatus {
    match result {
        ToolRunResult::Pass => ArtifactStatus::Pass,
        ToolRunResult::Fail => ArtifactStatus::Fail,
        ToolRunResult::NonBlockingSkipped => ArtifactStatus::NonBlockingSkipped,
        ToolRunResult::UnsupportedOnPlatform => ArtifactStatus::UnsupportedOnPlatform,
        ToolRunResult::ExpectedFailRefuted => ArtifactStatus::ExpectedFailRefuted,
        ToolRunResult::ExpectedFailNotRefuted => ArtifactStatus::ExpectedFailNotRefuted,
        ToolRunResult::NotRun | ToolRunResult::Generated | ToolRunResult::PendingToolRun => {
            ArtifactStatus::NotRun
        }
        ToolRunResult::Stale => ArtifactStatus::Stale,
        ToolRunResult::NotGenerated => ArtifactStatus::NotGenerated,
        ToolRunResult::Unknown => ArtifactStatus::InvalidReceipt,
    }
}

/// Derived artifact status per formal target (typed replacement for a
/// string-keyed map).
#[derive(Debug, Default, Clone, Copy)]
struct TargetStatuses {
    alloy: Option<ArtifactStatus>,
    p: Option<ArtifactStatus>,
    kani: Option<ArtifactStatus>,
    verus: Option<ArtifactStatus>,
    lean4: Option<ArtifactStatus>,
}

impl TargetStatuses {
    fn set(&mut self, target: FormalTarget, status: ArtifactStatus) {
        match target {
            FormalTarget::Alloy => self.alloy = Some(status),
            FormalTarget::P => self.p = Some(status),
            FormalTarget::Kani => self.kani = Some(status),
            FormalTarget::Verus => self.verus = Some(status),
            FormalTarget::Lean4 => self.lean4 = Some(status),
        }
    }

    fn holds(status: Option<ArtifactStatus>) -> bool {
        status.is_some_and(ArtifactStatus::holds)
    }
}

/// Inputs needed to derive a single artifact's coverage status.
#[derive(Debug, Clone)]
pub struct ArtifactFacts<'a> {
    /// Formal target.
    pub target: FormalTarget,
    /// Artifact kind (`facts`/`monitor`/`harness`/`proof`).
    pub artifact_kind: &'a str,
    /// Path to the generated artifact.
    pub path: &'a str,
    /// Generated artifact hash recorded at generation time.
    pub artifact_hash: &'a str,
    /// Path to the codegen receipt.
    pub codegen_receipt_path: &'a str,
    /// Path to the tool-run receipt.
    pub tool_run_receipt_path: &'a str,
    /// Parsed tool-run receipt, when present and decodable.
    pub tool_run: Option<&'a FormalReceipt>,
    /// `true` when a tool-run receipt file existed but failed to parse.
    pub receipt_parse_failed: bool,
    /// Hash of the artifact file as it currently exists on disk, when readable.
    pub on_disk_hash: Option<&'a str>,
}

/// A derived per-artifact coverage entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactReport {
    /// Formal target.
    pub target: FormalTarget,
    /// Artifact kind.
    pub artifact_kind: String,
    /// Path to the generated artifact.
    pub path: String,
    /// Generated artifact hash.
    pub artifact_hash: String,
    /// Path to the codegen receipt.
    pub codegen_receipt: String,
    /// Path to the tool-run receipt.
    pub tool_run_receipt: String,
    /// Derived status.
    pub status: ArtifactStatus,
    /// Process exit code that contributed to the status, when known.
    pub exit_code: Option<i64>,
    /// Receipt paths the status was derived from (provenance).
    pub derived_from: Vec<String>,
}

/// A per-invariant lane coverage row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InvariantCoverage {
    /// Invariant id (`I-001`..`I-010`).
    pub invariant_id: String,
    /// Executable replay oracle lane status.
    pub replay: LaneStatus,
    /// Alloy lane status.
    pub alloy: LaneStatus,
    /// P lane status.
    pub p: LaneStatus,
    /// Kani lane status.
    pub kani: LaneStatus,
    /// Verus lane status.
    pub verus: LaneStatus,
    /// Lean4 lane status.
    pub lean4: LaneStatus,
    /// Rolled-up row status.
    pub status: InvariantStatus,
    /// The concrete checks backing the claimed lanes (`lane:check_id`), so the
    /// row's coverage is auditable rather than asserted (P0-006).
    #[serde(default)]
    pub check_ids: Vec<String>,
}

/// A negative-control verdict produced by the replay oracle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NegativeControl {
    /// Scenario id.
    pub scenario: String,
    /// Declared expected stable error code.
    pub expected_error_code: Option<String>,
    /// Verdict.
    pub status: NegativeControlStatus,
    /// Actual stable error code observed (if the trace was refuted).
    pub actual_error_code: Option<String>,
}

/// A fully derived formal coverage report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormalCoverageReport {
    /// Coverage schema version.
    pub schema_version: u32,
    /// Overall derived status.
    pub status: ReportStatus,
    /// Timestamp token supplied by the caller.
    pub checked_at: String,
    /// Source compiled bundle hash.
    pub source_bundle_hash: String,
    /// Formal IR hash.
    pub formal_ir_hash: String,
    /// Optional scenario hash.
    pub scenario_hash: Option<String>,
    /// Path to the emitted formal IR.
    pub formal_ir_path: String,
    /// Per-artifact derived coverage.
    pub artifacts: Vec<ArtifactReport>,
    /// Per-invariant lane coverage (reconciled with artifact reality).
    pub invariant_coverage: Vec<InvariantCoverage>,
    /// Negative-control verdicts from the replay oracle.
    pub negative_controls: Vec<NegativeControl>,
}

/// Report-level metadata threaded through [`build_report`].
#[derive(Debug, Clone)]
pub struct CoverageMeta {
    /// Timestamp token.
    pub checked_at: String,
    /// Source bundle hash.
    pub source_bundle_hash: String,
    /// Formal IR hash.
    pub formal_ir_hash: String,
    /// Optional scenario hash.
    pub scenario_hash: Option<String>,
    /// Path to the emitted formal IR.
    pub formal_ir_path: String,
}

/// Derive the coverage status of a single artifact from its receipts and the
/// real exit code. This never returns `Pass` for a non-zero exit code.
#[must_use]
pub fn derive_artifact_status(facts: &ArtifactFacts) -> ArtifactStatus {
    if facts.receipt_parse_failed {
        return ArtifactStatus::InvalidReceipt;
    }
    let Some(receipt) = facts.tool_run else {
        return ArtifactStatus::NotRun;
    };
    if let Some(disk) = facts.on_disk_hash {
        if disk != receipt.generated_artifact_hash {
            return ArtifactStatus::Stale;
        }
    }
    // A non-zero exit code can never be a pass, regardless of recorded text.
    if let Some(code) = receipt.exit_code {
        if code != 0 {
            return ArtifactStatus::Fail;
        }
    }
    status_from_result(receipt.actual_result)
}

fn reconcile_cell(cell: LaneStatus, target_status: Option<ArtifactStatus>) -> LaneStatus {
    if cell.proves() && !TargetStatuses::holds(target_status) {
        LaneStatus::PendingToolRun
    } else {
        cell
    }
}

fn roll_up_invariant(cells: &[LaneStatus]) -> InvariantStatus {
    if cells.iter().any(|cell| cell.proves()) {
        InvariantStatus::Covered
    } else if cells.iter().all(|cell| {
        matches!(
            cell,
            LaneStatus::NotApplicable | LaneStatus::NonBlockingSpec
        )
    }) {
        InvariantStatus::NotApplicable
    } else {
        InvariantStatus::Blocked
    }
}

/// Reconcile declared per-invariant lane cells against real artifact statuses.
///
/// A declared `Passed` / `NegativeControl` cell whose underlying artifact did
/// not actually hold is downgraded to `PendingToolRun`, which prevents the
/// invariant matrix from claiming assurance the tools did not produce.
fn reconcile_invariants(
    declared: &[InvariantCoverage],
    statuses: TargetStatuses,
) -> Vec<InvariantCoverage> {
    declared
        .iter()
        .map(|row| {
            // The replay lane is exercised separately (prerequisite replay +
            // negative controls), so it is not gated on a generated artifact.
            let replay = row.replay;
            let alloy = reconcile_cell(row.alloy, statuses.alloy);
            let p = reconcile_cell(row.p, statuses.p);
            let kani = reconcile_cell(row.kani, statuses.kani);
            let verus = reconcile_cell(row.verus, statuses.verus);
            let lean4 = reconcile_cell(row.lean4, statuses.lean4);
            let status = roll_up_invariant(&[replay, alloy, p, kani, verus, lean4]);
            InvariantCoverage {
                invariant_id: row.invariant_id.clone(),
                replay,
                alloy,
                p,
                kani,
                verus,
                lean4,
                status,
                check_ids: row.check_ids.clone(),
            }
        })
        .collect()
}

/// Overall report status: `Pass` only when every artifact holds and every
/// negative control was refuted by the replay oracle. Cannot be upgraded.
#[must_use]
pub fn overall_status(
    artifacts: &[ArtifactReport],
    negative_controls: &[NegativeControl],
) -> ReportStatus {
    let artifacts_hold = artifacts.iter().all(|artifact| artifact.status.holds());
    let negatives_hold = negative_controls
        .iter()
        .all(|control| control.status.refuted());
    if artifacts_hold && negatives_hold {
        ReportStatus::Pass
    } else {
        ReportStatus::Fail
    }
}

/// Assemble a derived coverage report from the per-artifact facts, the declared
/// invariant lane matrix and the negative-control verdicts.
#[must_use]
pub fn build_report(
    meta: &CoverageMeta,
    artifacts: &[ArtifactFacts],
    declared_invariants: &[InvariantCoverage],
    negative_controls: Vec<NegativeControl>,
) -> FormalCoverageReport {
    let mut artifact_reports = Vec::with_capacity(artifacts.len());
    let mut statuses = TargetStatuses::default();
    for facts in artifacts {
        let status = derive_artifact_status(facts);
        let exit_code = facts.tool_run.and_then(|receipt| receipt.exit_code);
        statuses.set(facts.target, status);
        artifact_reports.push(ArtifactReport {
            target: facts.target,
            artifact_kind: facts.artifact_kind.to_owned(),
            path: facts.path.to_owned(),
            artifact_hash: facts.artifact_hash.to_owned(),
            codegen_receipt: facts.codegen_receipt_path.to_owned(),
            tool_run_receipt: facts.tool_run_receipt_path.to_owned(),
            status,
            exit_code,
            derived_from: vec![
                facts.codegen_receipt_path.to_owned(),
                facts.tool_run_receipt_path.to_owned(),
            ],
        });
    }
    let invariant_coverage = reconcile_invariants(declared_invariants, statuses);
    let status = overall_status(&artifact_reports, &negative_controls);
    FormalCoverageReport {
        schema_version: COVERAGE_SCHEMA_VERSION,
        status,
        checked_at: meta.checked_at.clone(),
        source_bundle_hash: meta.source_bundle_hash.clone(),
        formal_ir_hash: meta.formal_ir_hash.clone(),
        scenario_hash: meta.scenario_hash.clone(),
        formal_ir_path: meta.formal_ir_path.clone(),
        artifacts: artifact_reports,
        invariant_coverage,
        negative_controls,
    }
}

/// Build the declared per-invariant lane matrix from the **concrete obligations**
/// each lane carries (P0-006). Every generated lane (alloy/p/kani **and the
/// always-on proof lanes Verus/Lean4**) is declared `Passed` for an invariant only
/// when a real check/proof obligation backs it; any lane without an obligation is
/// `NotApplicable` — so the matrix can never claim assurance a check does not
/// provide. The reconcile pass downgrades any declared `Passed` cell whose tool run
/// did not hold, so a proof failure is never silently absorbed. The `replay` column
/// is grounded in the gate-run negative controls.
///
/// These are still *declared* expectations: [`build_report`] reconciles the
/// generated lanes against real tool-run statuses, downgrading any `Passed` cell
/// whose tool run did not hold.
#[must_use]
pub fn invariant_matrix(
    alloy: &[ReceiptObligation],
    p: &[ReceiptObligation],
    kani: &[ReceiptObligation],
    verus: &[ReceiptObligation],
    lean4: &[ReceiptObligation],
) -> Vec<InvariantCoverage> {
    let replay_obligations = obligations::replay_obligations();
    obligations::ALL_INVARIANTS
        .iter()
        .map(|invariant| {
            let mut check_ids = Vec::new();
            let mut cell = |lane: &str, obligations: &[ReceiptObligation], present: LaneStatus| {
                let checks = checks_for(obligations, invariant);
                if checks.is_empty() {
                    LaneStatus::NotApplicable
                } else {
                    for check_id in checks {
                        check_ids.push(format!("{lane}:{check_id}"));
                    }
                    present
                }
            };
            let replay = cell("replay", &replay_obligations, LaneStatus::Passed);
            let alloy = cell("alloy", alloy, LaneStatus::Passed);
            let p = cell("p", p, LaneStatus::Passed);
            let kani = cell("kani", kani, LaneStatus::Passed);
            let verus = cell("verus", verus, LaneStatus::Passed);
            let lean4 = cell("lean4", lean4, LaneStatus::Passed);
            InvariantCoverage {
                invariant_id: (*invariant).to_owned(),
                replay,
                alloy,
                p,
                kani,
                verus,
                lean4,
                // Placeholder; build_report's reconcile pass recomputes the
                // rolled-up status from the reconciled cells.
                status: InvariantStatus::Covered,
                check_ids,
            }
        })
        .collect()
}

fn checks_for<'a>(obligations: &'a [ReceiptObligation], invariant: &str) -> Vec<&'a str> {
    obligations
        .iter()
        .filter(|obligation| obligation.invariant_id == invariant)
        .map(|obligation| obligation.check_id.as_str())
        .collect()
}

/// The canonical declared lane matrix (all candidate checks present). The
/// `verify-all` binary prefers [`invariant_matrix`] with the per-artifact
/// obligations actually generated for the bundle; this is the static view used
/// by tests and tools.
#[must_use]
pub fn default_invariant_matrix() -> Vec<InvariantCoverage> {
    invariant_matrix(
        &obligations::all_obligations(FormalTarget::Alloy),
        &obligations::all_obligations(FormalTarget::P),
        &obligations::all_obligations(FormalTarget::Kani),
        &obligations::all_obligations(FormalTarget::Verus),
        &obligations::all_obligations(FormalTarget::Lean4),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_report, default_invariant_matrix, derive_artifact_status, invariant_matrix,
        overall_status, ArtifactFacts, ArtifactReport, ArtifactStatus, CoverageMeta,
        InvariantCoverage, InvariantStatus, LaneStatus, NegativeControl, NegativeControlStatus,
        ReportStatus,
    };
    use crate::{FormalReceipt, FormalTarget, ReceiptObligation, ReceiptScope, ToolRunResult};

    fn tool_run_receipt(
        actual_result: ToolRunResult,
        exit_code: Option<i64>,
        hash: &str,
    ) -> FormalReceipt {
        FormalReceipt {
            schema_version: 2,
            receipt_kind: "tool_run".to_owned(),
            artifact_kind: "facts".to_owned(),
            target: Some("alloy".to_owned()),
            tool: "alloy".to_owned(),
            tool_version: "6.2.0".to_owned(),
            generator_version: "0.0.0".to_owned(),
            source_bundle_hash: "sha256:aa".to_owned(),
            formal_ir_hash: Some("sha256:bb".to_owned()),
            scenario_hash: None,
            core_model_hash: None,
            generated_artifact_hash: hash.to_owned(),
            command: "alloy run".to_owned(),
            expected_result: Some("pass".to_owned()),
            actual_result,
            exit_code,
            invariant_ids: vec!["I-001".to_owned()],
            obligations: Vec::new(),
            scope: ReceiptScope {
                predicates: 1,
                scenarios: 1,
            },
            checked_at: "unix:0".to_owned(),
        }
    }

    fn facts<'a>(
        target: FormalTarget,
        receipt: Option<&'a FormalReceipt>,
        on_disk_hash: Option<&'a str>,
    ) -> ArtifactFacts<'a> {
        ArtifactFacts {
            target,
            artifact_kind: "facts",
            path: "formal/x.als",
            artifact_hash: "sha256:cc",
            codegen_receipt_path: "formal/receipts/x.codegen.json",
            tool_run_receipt_path: "formal/receipts/x.tool-run.json",
            tool_run: receipt,
            receipt_parse_failed: false,
            on_disk_hash,
        }
    }

    #[test]
    fn pass_when_tool_passed_and_exit_zero() {
        let receipt = tool_run_receipt(ToolRunResult::Pass, Some(0), "sha256:dd");
        let status = derive_artifact_status(&facts(
            FormalTarget::Alloy,
            Some(&receipt),
            Some("sha256:dd"),
        ));
        assert_eq!(status, ArtifactStatus::Pass);
    }

    #[test]
    fn nonzero_exit_forces_fail_even_if_text_says_pass() {
        // actual_result=Pass but a non-zero exit must win: coverage cannot be
        // greened over a real failure.
        let receipt = tool_run_receipt(ToolRunResult::Pass, Some(1), "sha256:dd");
        let status = derive_artifact_status(&facts(
            FormalTarget::Alloy,
            Some(&receipt),
            Some("sha256:dd"),
        ));
        assert_eq!(status, ArtifactStatus::Fail);
    }

    #[test]
    fn missing_receipt_is_not_run() {
        assert_eq!(
            derive_artifact_status(&facts(FormalTarget::Alloy, None, None)),
            ArtifactStatus::NotRun
        );
    }

    #[test]
    fn on_disk_drift_is_stale() {
        let receipt = tool_run_receipt(ToolRunResult::Pass, Some(0), "sha256:dd");
        let status = derive_artifact_status(&facts(
            FormalTarget::Alloy,
            Some(&receipt),
            Some("sha256:ee"),
        ));
        assert_eq!(status, ArtifactStatus::Stale);
    }

    #[test]
    fn unparseable_receipt_is_invalid() {
        let mut f = facts(FormalTarget::Alloy, None, None);
        f.receipt_parse_failed = true;
        assert_eq!(derive_artifact_status(&f), ArtifactStatus::InvalidReceipt);
    }

    #[test]
    fn unknown_token_is_invalid_receipt() {
        let receipt = tool_run_receipt(ToolRunResult::Unknown, Some(0), "sha256:dd");
        let status = derive_artifact_status(&facts(
            FormalTarget::Alloy,
            Some(&receipt),
            Some("sha256:dd"),
        ));
        assert_eq!(status, ArtifactStatus::InvalidReceipt);
    }

    fn report_artifact(target: FormalTarget, status: ArtifactStatus) -> ArtifactReport {
        ArtifactReport {
            target,
            artifact_kind: "facts".to_owned(),
            path: "x".to_owned(),
            artifact_hash: "sha256:cc".to_owned(),
            codegen_receipt: "c".to_owned(),
            tool_run_receipt: "t".to_owned(),
            status,
            exit_code: Some(0),
            derived_from: Vec::new(),
        }
    }

    #[test]
    fn overall_fail_if_any_artifact_failed() {
        let artifacts = vec![
            report_artifact(FormalTarget::Alloy, ArtifactStatus::Pass),
            report_artifact(FormalTarget::P, ArtifactStatus::Fail),
        ];
        assert_eq!(overall_status(&artifacts, &[]), ReportStatus::Fail);
    }

    #[test]
    fn overall_fail_if_negative_control_not_refuted() {
        let artifacts = vec![report_artifact(FormalTarget::Alloy, ArtifactStatus::Pass)];
        let negatives = vec![NegativeControl {
            scenario: "x_invalid".to_owned(),
            expected_error_code: Some("ConflictingLeases".to_owned()),
            status: NegativeControlStatus::UnexpectedPass,
            actual_error_code: None,
        }];
        assert_eq!(overall_status(&artifacts, &negatives), ReportStatus::Fail);
    }

    #[test]
    fn overall_pass_when_everything_holds() {
        let artifacts = vec![
            report_artifact(FormalTarget::Alloy, ArtifactStatus::Pass),
            report_artifact(FormalTarget::Verus, ArtifactStatus::NonBlockingSkipped),
        ];
        let negatives = vec![NegativeControl {
            scenario: "x_invalid".to_owned(),
            expected_error_code: Some("ConflictingLeases".to_owned()),
            status: NegativeControlStatus::RefutedByReplay,
            actual_error_code: Some("ConflictingLeases".to_owned()),
        }];
        assert_eq!(overall_status(&artifacts, &negatives), ReportStatus::Pass);
    }

    #[test]
    fn build_report_downgrades_lane_when_artifact_failed() {
        // A failed P tool run must downgrade a declared "passed" P lane cell and
        // fail the report — the matrix cannot claim assurance tools did not give.
        let pass_receipt = tool_run_receipt(ToolRunResult::Pass, Some(0), "sha256:cc");
        let fail_receipt = tool_run_receipt(ToolRunResult::Fail, Some(1), "sha256:cc");
        let artifact_facts = vec![
            facts(FormalTarget::Alloy, Some(&pass_receipt), Some("sha256:cc")),
            facts(FormalTarget::P, Some(&fail_receipt), Some("sha256:cc")),
        ];
        let declared = vec![InvariantCoverage {
            invariant_id: "I-001".to_owned(),
            replay: LaneStatus::Passed,
            alloy: LaneStatus::Passed,
            p: LaneStatus::Passed,
            kani: LaneStatus::NotApplicable,
            verus: LaneStatus::NonBlockingSpec,
            lean4: LaneStatus::NonBlockingSpec,
            status: InvariantStatus::Covered,
            check_ids: Vec::new(),
        }];
        let report = build_report(
            &CoverageMeta {
                checked_at: "unix:0".to_owned(),
                source_bundle_hash: "sha256:aa".to_owned(),
                formal_ir_hash: "sha256:bb".to_owned(),
                scenario_hash: None,
                formal_ir_path: "ir".to_owned(),
            },
            &artifact_facts,
            &declared,
            Vec::new(),
        );
        assert_eq!(report.status, ReportStatus::Fail);
        let row = report
            .invariant_coverage
            .iter()
            .find(|row| row.invariant_id == "I-001");
        assert_eq!(row.map(|row| row.p), Some(LaneStatus::PendingToolRun));
        assert_eq!(row.map(|row| row.alloy), Some(LaneStatus::Passed));
    }

    #[test]
    fn default_matrix_covers_ten_invariants() {
        let matrix = default_invariant_matrix();
        assert_eq!(matrix.len(), 10);
        assert!(matrix.iter().any(|row| row.invariant_id == "I-001"));
        assert!(matrix.iter().any(|row| row.invariant_id == "I-010"));
    }

    #[test]
    fn invariant_matrix_keeps_all_checks_for_one_lane() {
        let matrix = invariant_matrix(
            &[
                ReceiptObligation {
                    invariant_id: "I-009".to_owned(),
                    check_id: "GeneratedApprovalBindingHolds".to_owned(),
                },
                ReceiptObligation {
                    invariant_id: "I-009".to_owned(),
                    check_id: "GeneratedWitnessFactGrounded".to_owned(),
                },
                ReceiptObligation {
                    invariant_id: "I-009".to_owned(),
                    check_id: "GeneratedAnchorFactGrounded".to_owned(),
                },
            ],
            &[],
            &[],
            &[],
            &[],
        );
        let row = matrix.iter().find(|row| row.invariant_id == "I-009");

        if let Some(row) = row {
            assert_eq!(row.alloy, LaneStatus::Passed);
            assert!(row
                .check_ids
                .contains(&"alloy:GeneratedApprovalBindingHolds".to_owned()));
            assert!(row
                .check_ids
                .contains(&"alloy:GeneratedWitnessFactGrounded".to_owned()));
            assert!(row
                .check_ids
                .contains(&"alloy:GeneratedAnchorFactGrounded".to_owned()));
        } else {
            assert!(row.is_some(), "I-009 row exists");
        }
    }
}
