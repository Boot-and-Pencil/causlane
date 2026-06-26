//! Formal receipt construction shared by CLI formal commands.

use std::path::Path;

use causlane_codegen::{
    generated_artifact_hash, receipt_to_json, FormalReceipt, FormalTarget, GeneratedArtifact,
    ReceiptScope, ToolRunResult, GENERATOR_VERSION,
};

use crate::cli_shared::len_u32;
use crate::formal_artifacts::FormalArtifactPlan;

use super::io::{read_file, write_file, FormalIo};
use super::FormalServiceError;

#[derive(Clone, Copy)]
pub(super) struct GenerationWrite<'a> {
    pub(super) ir_predicate_count: usize,
    pub(super) ir_json: &'a str,
    pub(super) ir_path: &'a Path,
    pub(super) plans: &'a [FormalArtifactPlan],
    pub(super) bundle_path: &'a str,
    pub(super) scenario_path: &'a str,
    pub(super) expected_result: &'a str,
    pub(super) checked_at: &'a str,
}

pub(super) fn write_generation(
    io: &mut dyn FormalIo,
    input: GenerationWrite<'_>,
) -> Result<(), FormalServiceError> {
    write_file(io, input.ir_path, input.ir_json)?;
    let core_model_hash = optional_file_hash(io, "formal/alloy/core/causlane_core.als");
    for plan in input.plans {
        write_file(io, &plan.artifact_path, &plan.artifact.text)?;
        write_codegen_receipt(
            io,
            &plan.codegen_receipt_path,
            &plan.artifact,
            len_u32(input.ir_predicate_count),
            input.checked_at,
            &format!(
                "causlane-formal verify-all --bundle {} --scenario {}",
                input.bundle_path, input.scenario_path
            ),
            codegen_core_hash(&plan.artifact, core_model_hash.as_ref()),
        )?;
        write_tool_run_receipt(
            io,
            &plan.tool_run_receipt_path,
            &plan.artifact,
            len_u32(input.ir_predicate_count),
            input.checked_at,
            input.expected_result.to_owned(),
            codegen_core_hash(&plan.artifact, core_model_hash.as_ref()),
        )?;
    }
    Ok(())
}

pub(super) fn write_codegen_receipt(
    io: &mut dyn FormalIo,
    path: &Path,
    artifact: &GeneratedArtifact,
    predicate_count: u32,
    checked_at: &str,
    command: &str,
    core_model_hash: Option<String>,
) -> Result<(), FormalServiceError> {
    let receipt = base_receipt(
        "codegen",
        "causlane-codegen",
        artifact,
        predicate_count,
        checked_at,
        command,
        core_model_hash,
        ToolRunResult::Generated,
    );
    let json = receipt_to_json(&receipt)?;
    write_file(io, path, &json)
}

fn write_tool_run_receipt(
    io: &mut dyn FormalIo,
    path: &Path,
    artifact: &GeneratedArtifact,
    predicate_count: u32,
    checked_at: &str,
    expected_result: String,
    core_model_hash: Option<String>,
) -> Result<(), FormalServiceError> {
    let mut receipt = base_receipt(
        "tool_run",
        "causlane-formal-static-runner",
        artifact,
        predicate_count,
        checked_at,
        &format!(
            "causlane-formal static-run --target {}",
            artifact.target.as_str()
        ),
        core_model_hash,
        ToolRunResult::NotRun,
    );
    receipt.expected_result = Some(expected_result);
    let json = receipt_to_json(&receipt)?;
    write_file(io, path, &json)
}

#[allow(clippy::too_many_arguments)]
fn base_receipt(
    receipt_kind: &str,
    tool: &str,
    artifact: &GeneratedArtifact,
    predicate_count: u32,
    checked_at: &str,
    command: &str,
    core_model_hash: Option<String>,
    actual_result: ToolRunResult,
) -> FormalReceipt {
    FormalReceipt {
        schema_version: 2,
        receipt_kind: receipt_kind.to_owned(),
        artifact_kind: artifact.artifact_kind.clone(),
        target: Some(artifact.target.as_str().to_owned()),
        tool: tool.to_owned(),
        tool_version: GENERATOR_VERSION.to_owned(),
        generator_version: artifact.generator_version.clone(),
        source_bundle_hash: artifact.source_bundle_hash.clone(),
        formal_ir_hash: Some(artifact.formal_ir_hash.clone()),
        scenario_hash: artifact.scenario_hash.clone(),
        core_model_hash,
        generated_artifact_hash: artifact.artifact_hash.clone(),
        command: command.to_owned(),
        expected_result: Some("generated".to_owned()),
        actual_result,
        exit_code: None,
        invariant_ids: artifact.invariant_ids.clone(),
        obligations: artifact.obligations.clone(),
        scope: ReceiptScope {
            predicates: predicate_count,
            scenarios: u32::from(artifact.scenario_hash.is_some()),
        },
        checked_at: checked_at.to_owned(),
    }
}

fn optional_file_hash(io: &mut dyn FormalIo, path: &str) -> Option<String> {
    read_file(io, path)
        .ok()
        .map(|text| generated_artifact_hash(&text))
}

fn codegen_core_hash(
    artifact: &GeneratedArtifact,
    core_model_hash: Option<&String>,
) -> Option<String> {
    if artifact.target == FormalTarget::Alloy {
        core_model_hash.cloned()
    } else {
        None
    }
}
