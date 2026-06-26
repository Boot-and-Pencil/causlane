//! Shared formal orchestration service.

mod coverage;
mod error;
mod io;
mod negative;
mod receipt;

#[cfg(test)]
mod tests;

use std::path::Path;

use causlane_codegen::{
    CodegenContracts, FormalIrBuilder, FormalTarget, GeneratedArtifact, ReportStatus, StaleChecker,
};

use crate::cli_shared::len_u32;
use crate::formal_artifacts::{
    artifact_for, artifact_path_for, codegen_receipt_path_for, ir_path_for, plan_artifacts,
    FORMAL_TARGETS,
};

use coverage::{derive_and_write_coverage, CoverageInput};
pub use error::FormalServiceError;
use io::{
    load_bundle_scenario_and_plans, load_facts, read_bundle, read_file, scenario_hash_from_file,
    scenario_stem, write_file,
};
pub use io::{FormalIo, FormalIoError};
use receipt::{write_codegen_receipt, write_generation, GenerationWrite};

/// Shared formal orchestration entrypoint.
#[derive(Debug, Default, Clone, Copy)]
pub struct FormalOrchestrator;

/// Generate one formal target.
#[derive(Clone, Copy)]
pub struct GenerateArtifactRequest<'a> {
    /// Formal target to generate.
    pub target: FormalTarget,
    /// Path to the compiled dispatch bundle JSON.
    pub bundle_path: &'a str,
    /// Optional replay scenario YAML path.
    pub scenario_path: Option<&'a str>,
    /// Output path for the generated artifact.
    pub out_path: &'a str,
    /// Optional output path for the codegen receipt.
    pub receipt_path: Option<&'a str>,
}

/// Generate all formal targets under the stable artifact layout.
#[derive(Clone, Copy)]
pub struct GenerateAllRequest<'a> {
    /// Path to the compiled dispatch bundle JSON.
    pub bundle_path: &'a str,
    /// Required replay scenario YAML path for all-target generation.
    pub scenario_path: Option<&'a str>,
    /// Root directory for generated formal artifacts.
    pub artifact_dir: &'a str,
    /// Root directory for generated formal receipts.
    pub receipt_dir: &'a str,
}

/// Check one generated artifact against the bundle and optional receipt.
#[derive(Clone, Copy)]
pub struct StaleCheckRequest<'a> {
    /// Path to the compiled dispatch bundle JSON.
    pub bundle_path: &'a str,
    /// Path to the generated artifact being checked.
    pub generated_path: &'a str,
    /// Optional replay scenario YAML path used to bind the artifact.
    pub scenario_path: Option<&'a str>,
    /// Optional codegen receipt path.
    pub receipt_path: Option<&'a str>,
}

/// Check all generated artifacts and codegen receipts under the stable layout.
#[derive(Clone, Copy)]
pub struct StaleCheckAllRequest<'a> {
    /// Path to the compiled dispatch bundle JSON.
    pub bundle_path: &'a str,
    /// Required replay scenario YAML path for all-target stale checks.
    pub scenario_path: Option<&'a str>,
    /// Root directory for generated formal artifacts.
    pub artifact_dir: &'a str,
    /// Root directory for generated formal receipts.
    pub receipt_dir: &'a str,
}

/// Emit the target-neutral Formal IR.
#[derive(Clone, Copy)]
pub struct EmitIrRequest<'a> {
    /// Path to the compiled dispatch bundle JSON.
    pub bundle_path: &'a str,
    /// Optional replay scenario YAML path.
    pub scenario_path: Option<&'a str>,
    /// Output path for the Formal IR JSON.
    pub out_path: &'a str,
}

/// Generate formal artifacts, codegen receipts, not-run tool receipts and coverage.
#[derive(Clone, Copy)]
pub struct VerifyAllRequest<'a> {
    /// Path to the compiled dispatch bundle JSON.
    pub bundle_path: &'a str,
    /// Replay scenario YAML path.
    pub scenario_path: &'a str,
    /// Root directory for generated formal artifacts.
    pub artifact_dir: &'a str,
    /// Root directory for generated formal receipts.
    pub receipt_dir: &'a str,
    /// Output path for the derived coverage report.
    pub coverage_path: &'a str,
}

/// Re-derive coverage from existing tool-run receipts.
#[derive(Clone, Copy)]
pub struct CoverageRequest<'a> {
    /// Path to the compiled dispatch bundle JSON.
    pub bundle_path: &'a str,
    /// Replay scenario YAML path.
    pub scenario_path: &'a str,
    /// Root directory for generated formal artifacts.
    pub artifact_dir: &'a str,
    /// Root directory for generated formal receipts.
    pub receipt_dir: &'a str,
    /// Output path for the derived coverage report.
    pub coverage_path: &'a str,
}

impl FormalOrchestrator {
    /// Generate an Alloy artifact and optional codegen receipt.
    pub fn generate_alloy(
        self,
        io: &mut dyn FormalIo,
        request: GenerateArtifactRequest<'_>,
    ) -> Result<String, FormalServiceError> {
        let artifact = Self::write_single_artifact(io, &request)?;
        if let Some(path) = request.receipt_path {
            Ok(format!(
                "ok: wrote {}; receipt {path}; generated_artifact_hash {}",
                request.out_path, artifact.artifact_hash
            ))
        } else {
            Ok(format!(
                "ok: wrote {}; generated_artifact_hash {}",
                request.out_path, artifact.artifact_hash
            ))
        }
    }

    /// Generate a non-Alloy formal artifact and optional codegen receipt.
    pub fn generate_single(
        self,
        io: &mut dyn FormalIo,
        request: GenerateArtifactRequest<'_>,
    ) -> Result<String, FormalServiceError> {
        let artifact = Self::write_single_artifact(io, &request)?;
        Ok(format!(
            "ok: wrote {}; target {}; generated_artifact_hash {}",
            request.out_path,
            artifact.target.as_str(),
            artifact.artifact_hash
        ))
    }

    /// Generate Formal IR plus every formal target and codegen receipt.
    pub fn generate_all(
        self,
        io: &mut dyn FormalIo,
        request: GenerateAllRequest<'_>,
    ) -> Result<String, FormalServiceError> {
        let bundle = read_bundle(io, request.bundle_path)?;
        let stem = scenario_stem(io, request.scenario_path)?;
        let Some(facts) = load_facts(io, request.scenario_path)? else {
            return Err(FormalServiceError::Usage(
                "--scenario is required for all-target generation".to_owned(),
            ));
        };
        let ir = CodegenContracts.build_ir(&bundle, Some(&facts))?;
        let ir_path = ir_path_for(request.artifact_dir, &stem);
        write_file(io, &ir_path, &ir.to_json_pretty()?)?;
        let plans = plan_artifacts(
            &bundle,
            &facts,
            &ir,
            request.artifact_dir,
            request.receipt_dir,
            &stem,
        )?;

        let mut summary = Vec::new();
        let checked_at = io.checked_at_token();
        for plan in plans {
            write_file(io, &plan.artifact_path, &plan.artifact.text)?;
            let command = format!(
                "causlane formal generate all --bundle {}",
                request.bundle_path
            );
            write_codegen_receipt(
                io,
                &plan.codegen_receipt_path,
                &plan.artifact,
                len_u32(bundle.body.predicates.len()),
                &checked_at,
                &command,
                None,
            )?;
            summary.push(format!(
                "{}={}",
                plan.artifact.target.as_str(),
                plan.artifact.artifact_hash
            ));
        }
        Ok(format!(
            "ok: generated all targets under {}; {}",
            request.artifact_dir,
            summary.join(" ")
        ))
    }

    /// Check one generated artifact for freshness.
    pub fn stale_check(
        self,
        io: &mut dyn FormalIo,
        request: StaleCheckRequest<'_>,
    ) -> Result<String, FormalServiceError> {
        let bundle = read_bundle(io, request.bundle_path)?;
        let generated = read_file(io, request.generated_path)?;
        let receipt = match request.receipt_path {
            Some(path) => Some(read_file(io, path)?),
            None => None,
        };
        let scenario_hash = match request.scenario_path {
            Some(path) => Some(scenario_hash_from_file(io, path)?),
            None => None,
        };
        CodegenContracts.stale_check(
            &bundle,
            &generated,
            receipt.as_deref(),
            scenario_hash.as_deref(),
        )?;
        Ok(format!(
            "ok: generated artifact is fresh for bundle {}",
            bundle.bundle_hash.0
        ))
    }

    /// Check every generated target and codegen receipt for freshness.
    pub fn stale_check_all(
        self,
        io: &mut dyn FormalIo,
        request: StaleCheckAllRequest<'_>,
    ) -> Result<String, FormalServiceError> {
        let bundle = read_bundle(io, request.bundle_path)?;
        let scenario_hash = match request.scenario_path {
            Some(path) => Some(scenario_hash_from_file(io, path)?),
            None => None,
        };
        let stem = scenario_stem(io, request.scenario_path)?;

        let mut checked = Vec::new();
        for target in FORMAL_TARGETS {
            let artifact_path = artifact_path_for(request.artifact_dir, target, &stem);
            let receipt_path = codegen_receipt_path_for(request.receipt_dir, target, &stem);
            let generated = read_file(io, &artifact_path)?;
            let receipt = read_file(io, &receipt_path).ok();
            CodegenContracts.stale_check(
                &bundle,
                &generated,
                receipt.as_deref(),
                scenario_hash.as_deref(),
            )?;
            checked.push(target.as_str());
        }
        Ok(format!(
            "ok: all generated targets fresh for bundle {} ({})",
            bundle.bundle_hash.0,
            checked.join(", ")
        ))
    }

    /// Emit target-neutral Formal IR JSON.
    pub fn emit_ir(
        self,
        io: &mut dyn FormalIo,
        request: EmitIrRequest<'_>,
    ) -> Result<String, FormalServiceError> {
        let bundle = read_bundle(io, request.bundle_path)?;
        let scenario_facts = load_facts(io, request.scenario_path)?;
        let ir = CodegenContracts.build_ir(&bundle, scenario_facts.as_ref())?;
        write_file(io, request.out_path, &ir.to_json_pretty()?)?;
        Ok(format!(
            "ok: wrote {}; formal_ir_hash {}; source_bundle_hash {}",
            request.out_path, ir.formal_ir_hash, ir.source_bundle_hash
        ))
    }

    /// Generate artifacts, not-run receipts and preliminary coverage.
    pub fn verify_all(
        self,
        io: &mut dyn FormalIo,
        request: VerifyAllRequest<'_>,
    ) -> Result<String, FormalServiceError> {
        let loaded = load_bundle_scenario_and_plans(
            io,
            request.bundle_path,
            request.scenario_path,
            request.artifact_dir,
            request.receipt_dir,
        )?;
        let checked_at = checked_at_token(io);
        write_generation(
            io,
            GenerationWrite {
                ir_predicate_count: loaded.ir.predicates.len(),
                ir_json: &loaded.ir.to_json_pretty()?,
                ir_path: &loaded.ir_path,
                plans: &loaded.plans,
                bundle_path: request.bundle_path,
                scenario_path: request.scenario_path,
                expected_result: &loaded.facts.expected_result,
                checked_at: &checked_at,
            },
        )?;
        let status = derive_and_write_coverage(
            io,
            CoverageInput {
                bundle: &loaded.bundle,
                facts: &loaded.facts,
                ir: &loaded.ir,
                ir_path: &loaded.ir_path,
                plans: &loaded.plans,
                coverage_path: request.coverage_path,
                checked_at: &checked_at,
            },
        )?;
        Ok(format!(
            "ok: formal artifacts generated; preliminary coverage {} (status={}); run the tool lanes \
             and `causlane-formal coverage` to finalise",
            request.coverage_path,
            report_status_token(status)
        ))
    }

    /// Re-derive coverage from existing tool-run receipts.
    pub fn coverage(
        self,
        io: &mut dyn FormalIo,
        request: CoverageRequest<'_>,
    ) -> Result<String, FormalServiceError> {
        let loaded = load_bundle_scenario_and_plans(
            io,
            request.bundle_path,
            request.scenario_path,
            request.artifact_dir,
            request.receipt_dir,
        )?;
        let checked_at = checked_at_token(io);
        let status = derive_and_write_coverage(
            io,
            CoverageInput {
                bundle: &loaded.bundle,
                facts: &loaded.facts,
                ir: &loaded.ir,
                ir_path: &loaded.ir_path,
                plans: &loaded.plans,
                coverage_path: request.coverage_path,
                checked_at: &checked_at,
            },
        )?;
        Ok(format!(
            "ok: coverage derived from receipts; {} (status={})",
            request.coverage_path,
            report_status_token(status)
        ))
    }

    fn write_single_artifact(
        io: &mut dyn FormalIo,
        request: &GenerateArtifactRequest<'_>,
    ) -> Result<GeneratedArtifact, FormalServiceError> {
        let bundle = read_bundle(io, request.bundle_path)?;
        let facts = load_facts(io, request.scenario_path)?;
        let ir = CodegenContracts.build_ir(&bundle, facts.as_ref())?;
        let artifact = artifact_for(request.target, &bundle, &ir, facts.as_ref())?;
        write_file(io, request.out_path, &artifact.text)?;
        if let Some(path) = request.receipt_path {
            let command = single_artifact_command(request);
            let checked_at = io.checked_at_token();
            write_codegen_receipt(
                io,
                Path::new(path),
                &artifact,
                len_u32(bundle.body.predicates.len()),
                &checked_at,
                &command,
                None,
            )?;
        }
        Ok(artifact)
    }
}

fn single_artifact_command(request: &GenerateArtifactRequest<'_>) -> String {
    if request.target == FormalTarget::Alloy {
        let scenario_command = match request.scenario_path {
            Some(scenario) => format!(" --scenario {scenario}"),
            None => String::new(),
        };
        format!(
            "causlane formal generate alloy --bundle {} --out {}{scenario_command} --receipt {}",
            request.bundle_path,
            request.out_path,
            request.receipt_path.unwrap_or_default()
        )
    } else {
        format!(
            "causlane formal generate {} --bundle {} --out {} --receipt {}",
            request.target.as_str(),
            request.bundle_path,
            request.out_path,
            request.receipt_path.unwrap_or_default()
        )
    }
}

fn checked_at_token(io: &mut dyn FormalIo) -> String {
    io.checked_at_token()
}

fn report_status_token(status: ReportStatus) -> &'static str {
    match status {
        ReportStatus::Pass => "pass",
        ReportStatus::Fail => "fail",
    }
}
