//! Formal coverage report derivation from generated artifacts and receipts.

use std::path::Path;

use causlane_codegen::{
    generated_artifact_hash, invariant_matrix, ArtifactFacts, CodegenContracts, CodegenError,
    CoverageMeta, CoverageReporter, FormalReceipt, FormalTarget, ReceiptObligation, ReportStatus,
};
use causlane_contracts::CompiledDispatchBundle;

use crate::formal_artifacts::FormalArtifactPlan;

use super::io::{read_file, write_file, FormalIo};
use super::negative::collect_negative_controls;
use super::FormalServiceError;

struct ArtifactDerivation {
    target: FormalTarget,
    artifact_kind: String,
    path: String,
    artifact_hash: String,
    codegen_receipt_path: String,
    tool_run_receipt_path: String,
    tool_run: Option<FormalReceipt>,
    receipt_parse_failed: bool,
    on_disk_hash: Option<String>,
}

#[derive(Clone, Copy)]
pub(super) struct CoverageInput<'a> {
    pub(super) bundle: &'a CompiledDispatchBundle,
    pub(super) facts: &'a causlane_codegen::AlloyScenarioFacts,
    pub(super) ir: &'a causlane_codegen::FormalIr,
    pub(super) ir_path: &'a Path,
    pub(super) plans: &'a [FormalArtifactPlan],
    pub(super) coverage_path: &'a str,
    pub(super) checked_at: &'a str,
}

pub(super) fn derive_and_write_coverage(
    io: &mut dyn FormalIo,
    input: CoverageInput<'_>,
) -> Result<ReportStatus, FormalServiceError> {
    let derivations: Vec<ArtifactDerivation> = input
        .plans
        .iter()
        .map(|plan| {
            let (tool_run, receipt_parse_failed) =
                read_tool_run_receipt(io, &plan.tool_run_receipt_path);
            ArtifactDerivation {
                target: plan.artifact.target,
                artifact_kind: plan.artifact.artifact_kind.clone(),
                path: plan.artifact_path.display().to_string(),
                artifact_hash: plan.artifact.artifact_hash.clone(),
                codegen_receipt_path: plan.codegen_receipt_path.display().to_string(),
                tool_run_receipt_path: plan.tool_run_receipt_path.display().to_string(),
                tool_run,
                receipt_parse_failed,
                on_disk_hash: on_disk_artifact_hash(io, &plan.artifact_path),
            }
        })
        .collect();
    let artifact_facts: Vec<ArtifactFacts> = derivations
        .iter()
        .map(|derivation| ArtifactFacts {
            target: derivation.target,
            artifact_kind: &derivation.artifact_kind,
            path: &derivation.path,
            artifact_hash: &derivation.artifact_hash,
            codegen_receipt_path: &derivation.codegen_receipt_path,
            tool_run_receipt_path: &derivation.tool_run_receipt_path,
            tool_run: derivation.tool_run.as_ref(),
            receipt_parse_failed: derivation.receipt_parse_failed,
            on_disk_hash: derivation.on_disk_hash.as_deref(),
        })
        .collect();
    let negative_controls = collect_negative_controls(io, "contracts/scenarios", input.bundle)?;
    let meta = CoverageMeta {
        checked_at: input.checked_at.to_owned(),
        source_bundle_hash: input.bundle.bundle_hash.0.clone(),
        formal_ir_hash: input.ir.formal_ir_hash.clone(),
        scenario_hash: Some(input.facts.scenario_hash.clone()),
        formal_ir_path: input.ir_path.display().to_string(),
    };
    let obligations_for = |target: FormalTarget| -> Vec<ReceiptObligation> {
        input
            .plans
            .iter()
            .find(|plan| plan.artifact.target == target)
            .map(|plan| plan.artifact.obligations.clone())
            .unwrap_or_default()
    };
    let declared = invariant_matrix(
        &obligations_for(FormalTarget::Alloy),
        &obligations_for(FormalTarget::P),
        &obligations_for(FormalTarget::Kani),
        &obligations_for(FormalTarget::Verus),
        &obligations_for(FormalTarget::Lean4),
    );
    let report = CodegenContracts.build_coverage_report(
        &meta,
        &artifact_facts,
        &declared,
        negative_controls,
    );
    let report_json = serde_json::to_string_pretty(&report)
        .map_err(|err| FormalServiceError::Codegen(CodegenError::Receipt(err.to_string())))?;
    write_file(io, Path::new(input.coverage_path), &report_json)?;
    Ok(report.status)
}

fn read_tool_run_receipt(io: &mut dyn FormalIo, path: &Path) -> (Option<FormalReceipt>, bool) {
    match read_file(io, path) {
        Ok(json) => match serde_json::from_str::<FormalReceipt>(&json) {
            Ok(receipt) => (Some(receipt), false),
            Err(_err) => (None, true),
        },
        Err(_err) => (None, false),
    }
}

fn on_disk_artifact_hash(io: &mut dyn FormalIo, path: &Path) -> Option<String> {
    read_file(io, path)
        .ok()
        .map(|text| generated_artifact_hash(&text))
}
