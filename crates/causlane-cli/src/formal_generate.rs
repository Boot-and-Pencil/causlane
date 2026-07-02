//! Formal codegen CLI adapter.

use causlane_cli::app::formal::{
    EmitIrRequest, FormalOrchestrator, GenerateAllRequest, GenerateArtifactRequest,
    StaleCheckAllRequest, StaleCheckRequest,
};
use causlane_codegen::FormalTarget;

use crate::{CliError, StdFormalIo};

pub(crate) fn generate_alloy(
    bundle_path: &str,
    out: &str,
    scenario_path: Option<&str>,
    receipt_path: Option<&str>,
) -> Result<String, CliError> {
    let mut io = StdFormalIo;
    Ok(FormalOrchestrator.generate_alloy(
        &mut io,
        GenerateArtifactRequest {
            target: FormalTarget::Alloy,
            bundle_path,
            scenario_path,
            out_path: out,
            receipt_path,
        },
    )?)
}

pub(crate) fn generate_single(
    target: FormalTarget,
    bundle_path: &str,
    scenario_path: Option<&str>,
    out: &str,
    receipt_path: Option<&str>,
) -> Result<String, CliError> {
    let mut io = StdFormalIo;
    Ok(FormalOrchestrator.generate_single(
        &mut io,
        GenerateArtifactRequest {
            target,
            bundle_path,
            scenario_path,
            out_path: out,
            receipt_path,
        },
    )?)
}

pub(crate) fn generate_all(
    bundle_path: &str,
    scenario_path: Option<&str>,
    artifact_dir: &str,
    receipt_dir: &str,
) -> Result<String, CliError> {
    let mut io = StdFormalIo;
    Ok(FormalOrchestrator.generate_all(
        &mut io,
        GenerateAllRequest {
            bundle_path,
            scenario_path,
            artifact_dir,
            receipt_dir,
        },
    )?)
}

pub(crate) fn check_generated(
    bundle_path: &str,
    generated_path: &str,
    scenario_path: Option<&str>,
    receipt_path: Option<&str>,
) -> Result<String, CliError> {
    let mut io = StdFormalIo;
    Ok(FormalOrchestrator.stale_check(
        &mut io,
        StaleCheckRequest {
            bundle_path,
            generated_path,
            scenario_path,
            receipt_path,
        },
    )?)
}

pub(crate) fn stale_check_all(
    bundle_path: &str,
    scenario_path: Option<&str>,
    artifact_dir: &str,
    receipt_dir: &str,
) -> Result<String, CliError> {
    let mut io = StdFormalIo;
    Ok(FormalOrchestrator.stale_check_all(
        &mut io,
        StaleCheckAllRequest {
            bundle_path,
            scenario_path,
            artifact_dir,
            receipt_dir,
        },
    )?)
}

pub(crate) fn emit_ir(
    bundle_path: &str,
    scenario_path: Option<&str>,
    out: &str,
) -> Result<String, CliError> {
    let mut io = StdFormalIo;
    Ok(FormalOrchestrator.emit_ir(
        &mut io,
        EmitIrRequest {
            bundle_path,
            scenario_path,
            out_path: out,
        },
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use causlane_contracts::{CompiledDispatchBundle, ContractError, RegistryManifest};
    use std::path::PathBuf;

    const REGISTRY: &str =
        include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");

    #[derive(Debug)]
    enum TestError {
        Contract,
        Cli,
        Missing,
    }

    impl From<ContractError> for TestError {
        fn from(_err: ContractError) -> Self {
            Self::Contract
        }
    }

    impl From<crate::CliError> for TestError {
        fn from(_err: crate::CliError) -> Self {
            Self::Cli
        }
    }

    fn output_dir(name: &str) -> PathBuf {
        PathBuf::from(format!("../../target/causlane-cli-tests/{name}"))
    }

    fn write_demo_bundle(path: &str) -> Result<(), TestError> {
        let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
        let bundle = CompiledDispatchBundle::compile(&manifest)?;
        crate::write_file(path, &bundle.to_json_pretty()?)?;
        Ok(())
    }

    fn require(condition: bool) -> Result<(), TestError> {
        if condition {
            Ok(())
        } else {
            Err(TestError::Missing)
        }
    }

    #[test]
    fn generate_all_then_stale_check_all_covers_every_target() -> Result<(), TestError> {
        let dir = output_dir("formal-generate-all-targets");
        let bundle = dir.join("release_promote.bundle.json");
        let artifact_dir = dir.join("verification/formal-full");
        let receipt_dir = dir.join("receipts");

        let bundle_path = bundle.display().to_string();
        write_demo_bundle(&bundle_path)?;
        let scenario_path = "../../contracts/scenarios/release_promote_success.scenario.yaml";
        let artifact_dir_path = artifact_dir.display().to_string();
        let receipt_dir_path = receipt_dir.display().to_string();

        let generated = generate_all(
            &bundle_path,
            Some(scenario_path),
            &artifact_dir_path,
            &receipt_dir_path,
        )?;
        require(generated.contains("alloy=sha256:"))?;
        require(generated.contains("lean4=sha256:"))?;

        let checked = stale_check_all(
            &bundle_path,
            Some(scenario_path),
            &artifact_dir_path,
            &receipt_dir_path,
        )?;
        require(checked.contains("all generated targets fresh"))?;
        require(checked.contains("alloy, p, kani, verus, lean4"))
    }
}
