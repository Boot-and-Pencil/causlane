use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use causlane_contracts::{CompiledDispatchBundle, ContractError, RegistryManifest};

use super::{
    CoverageRequest, FormalIo, FormalIoError, FormalOrchestrator, FormalServiceError,
    GenerateAllRequest, StaleCheckAllRequest, VerifyAllRequest,
};

const REGISTRY: &str =
    include_str!("../../../fixtures/contracts/examples/release_promote.registry.yaml");
const SCENARIO: &str =
    include_str!("../../../fixtures/contracts/scenarios/release_promote_success.scenario.yaml");
const SCENARIO_PATH: &str = "../../contracts/scenarios/release_promote_success.scenario.yaml";

#[derive(Default)]
struct MemoryIo {
    files: BTreeMap<PathBuf, String>,
}

impl MemoryIo {
    fn insert(&mut self, path: impl Into<PathBuf>, content: impl Into<String>) {
        self.files.insert(path.into(), content.into());
    }

    fn read_written(&self, path: impl AsRef<Path>) -> Option<&str> {
        self.files.get(path.as_ref()).map(String::as_str)
    }
}

impl FormalIo for MemoryIo {
    fn read_to_string(&mut self, path: &Path) -> Result<String, FormalIoError> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| FormalIoError::new(format!("missing {}", path.display())))
    }

    fn create_dir_all(&mut self, _path: &Path) -> Result<(), FormalIoError> {
        Ok(())
    }

    fn write_string(&mut self, path: &Path, content: &str) -> Result<(), FormalIoError> {
        self.insert(path.to_path_buf(), content.to_owned());
        Ok(())
    }

    fn read_dir_paths(&mut self, path: &Path) -> Result<Option<Vec<PathBuf>>, FormalIoError> {
        let paths = self
            .files
            .keys()
            .filter(|candidate| candidate.parent() == Some(path))
            .cloned()
            .collect::<Vec<_>>();
        if paths.is_empty() {
            Ok(None)
        } else {
            Ok(Some(paths))
        }
    }

    fn checked_at_token(&self) -> String {
        "unix:0".to_owned()
    }
}

#[derive(Debug)]
enum TestError {
    Contract,
    Formal,
    Missing,
}

impl From<ContractError> for TestError {
    fn from(_err: ContractError) -> Self {
        Self::Contract
    }
}

impl From<FormalServiceError> for TestError {
    fn from(_err: FormalServiceError) -> Self {
        Self::Formal
    }
}

fn demo_io() -> Result<(MemoryIo, String), TestError> {
    let mut io = MemoryIo::default();
    io.insert(SCENARIO_PATH, SCENARIO);
    let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
    let bundle = CompiledDispatchBundle::compile(&manifest)?;
    let bundle_path = "target/test/release_promote.bundle.json".to_owned();
    io.insert(&bundle_path, bundle.to_json_pretty()?);
    Ok((io, bundle_path))
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
    let (mut io, bundle_path) = demo_io()?;
    let generated = FormalOrchestrator.generate_all(
        &mut io,
        GenerateAllRequest {
            bundle_path: &bundle_path,
            scenario_path: Some(SCENARIO_PATH),
            artifact_dir: "formal",
            receipt_dir: "formal/receipts",
        },
    )?;
    require(generated.contains("alloy=sha256:"))?;
    require(generated.contains("lean4=sha256:"))?;

    let checked = FormalOrchestrator.stale_check_all(
        &mut io,
        StaleCheckAllRequest {
            bundle_path: &bundle_path,
            scenario_path: Some(SCENARIO_PATH),
            artifact_dir: "formal",
            receipt_dir: "formal/receipts",
        },
    )?;
    require(checked.contains("all generated targets fresh"))?;
    require(checked.contains("alloy, p, kani, verus, lean4"))
}

#[test]
fn verify_all_then_coverage_rederive_writes_report() -> Result<(), TestError> {
    let (mut io, bundle_path) = demo_io()?;
    let verify = FormalOrchestrator.verify_all(
        &mut io,
        VerifyAllRequest {
            bundle_path: &bundle_path,
            scenario_path: SCENARIO_PATH,
            artifact_dir: "formal",
            receipt_dir: "formal/receipts",
            coverage_path: "target/coverage.json",
        },
    )?;
    require(verify.contains("formal artifacts generated"))?;

    let rederive = FormalOrchestrator.coverage(
        &mut io,
        CoverageRequest {
            bundle_path: &bundle_path,
            scenario_path: SCENARIO_PATH,
            artifact_dir: "formal",
            receipt_dir: "formal/receipts",
            coverage_path: "target/coverage.json",
        },
    )?;
    require(rederive.contains("coverage derived from receipts"))?;
    let report = io
        .read_written("target/coverage.json")
        .ok_or(TestError::Missing)?;
    require(report.contains("\"invariant_coverage\""))
}
