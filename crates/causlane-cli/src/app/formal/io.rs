//! I/O port and loading helpers for the shared formal service.

use core::fmt;
use std::path::{Path, PathBuf};

use causlane_codegen::{AlloyScenarioFacts, CodegenContracts, FormalIr, FormalIrBuilder};
use causlane_contracts::CompiledDispatchBundle;
use causlane_replay::ReplayScenario;

use crate::cli_shared::FileError;
use crate::cli_shared::{read_file_with, safe_scenario_stem};
use crate::formal_artifacts::{ir_path_for, plan_artifacts, FormalArtifactPlan};
use crate::scenario_facts::{alloy_scenario_facts_from_yaml, facts_from_scenario};

use super::FormalServiceError;

/// Typed error returned by the formal service I/O boundary.
#[derive(Debug, Clone)]
pub struct FormalIoError {
    message: String,
}

impl FormalIoError {
    /// Build an adapter error from a displayable message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for FormalIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// File-system and clock boundary required by formal orchestration.
pub trait FormalIo {
    /// Read a UTF-8 text file.
    fn read_to_string(&mut self, path: &Path) -> Result<String, FormalIoError>;
    /// Create a directory and its parents.
    fn create_dir_all(&mut self, path: &Path) -> Result<(), FormalIoError>;
    /// Write UTF-8 text to a file.
    fn write_string(&mut self, path: &Path, content: &str) -> Result<(), FormalIoError>;
    /// Return immediate directory entries, or `None` when the directory is absent.
    fn read_dir_paths(&mut self, path: &Path) -> Result<Option<Vec<PathBuf>>, FormalIoError>;
    /// Return the checked-at token to embed in receipts/reports.
    fn checked_at_token(&self) -> String;
}

pub(super) struct LoadedFormalScenario {
    pub(super) bundle: CompiledDispatchBundle,
    pub(super) facts: AlloyScenarioFacts,
    pub(super) ir: FormalIr,
    pub(super) ir_path: PathBuf,
    pub(super) plans: Vec<FormalArtifactPlan>,
}

pub(super) fn load_bundle_scenario_and_plans(
    io: &mut dyn FormalIo,
    bundle_path: &str,
    scenario_path: &str,
    artifact_dir: &str,
    receipt_dir: &str,
) -> Result<LoadedFormalScenario, FormalServiceError> {
    let bundle = read_bundle(io, bundle_path)?;
    let (scenario, facts) = read_scenario(io, scenario_path)?;
    let ir = CodegenContracts.build_ir(&bundle, Some(&facts))?;
    let stem = safe_scenario_stem(&scenario.scenario_id);
    let ir_path = ir_path_for(artifact_dir, &stem);
    let plans = plan_artifacts(&bundle, &facts, &ir, artifact_dir, receipt_dir, &stem)?;
    Ok(LoadedFormalScenario {
        bundle,
        facts,
        ir,
        ir_path,
        plans,
    })
}

pub(super) fn load_facts(
    io: &mut dyn FormalIo,
    scenario_path: Option<&str>,
) -> Result<Option<AlloyScenarioFacts>, FormalServiceError> {
    match scenario_path {
        Some(path) => Ok(Some(alloy_scenario_facts_from_yaml(&read_file(io, path)?)?)),
        None => Ok(None),
    }
}

pub(super) fn scenario_stem(
    io: &mut dyn FormalIo,
    scenario_path: Option<&str>,
) -> Result<String, FormalServiceError> {
    let path = scenario_path.ok_or_else(|| {
        FormalServiceError::Usage("--scenario is required for all-target generation".to_owned())
    })?;
    let scenario = ReplayScenario::from_yaml_str(&read_file(io, path)?)?;
    Ok(safe_scenario_stem(&scenario.scenario_id))
}

pub(super) fn scenario_hash_from_file(
    io: &mut dyn FormalIo,
    path: &str,
) -> Result<String, FormalServiceError> {
    let yaml = read_file(io, path)?;
    ReplayScenario::scenario_hash(&yaml).map_err(FormalServiceError::from)
}

pub(super) fn read_file(
    io: &mut dyn FormalIo,
    path: impl AsRef<Path>,
) -> Result<String, FormalServiceError> {
    Ok(read_file_with(path, |path: &Path| io.read_to_string(path))?)
}

pub(super) fn write_file(
    io: &mut dyn FormalIo,
    path: impl AsRef<Path>,
    content: &str,
) -> Result<(), FormalServiceError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            io.create_dir_all(parent).map_err(|err| FileError::Write {
                path: parent.display().to_string(),
                message: err.to_string(),
            })?;
        }
    }
    Ok(io
        .write_string(path, content)
        .map_err(|err| FileError::Write {
            path: path.display().to_string(),
            message: err.to_string(),
        })?)
}

pub(super) fn read_bundle(
    io: &mut dyn FormalIo,
    path: &str,
) -> Result<CompiledDispatchBundle, FormalServiceError> {
    let json = read_file(io, path)?;
    Ok(CompiledDispatchBundle::from_json_str(&json)?)
}

pub(super) fn read_scenario(
    io: &mut dyn FormalIo,
    path: &str,
) -> Result<(ReplayScenario, AlloyScenarioFacts), FormalServiceError> {
    let yaml = read_file(io, path)?;
    let scenario = ReplayScenario::from_yaml_str(&yaml)?;
    let scenario_hash = ReplayScenario::scenario_hash(&yaml)?;
    let facts = facts_from_scenario(&scenario, scenario_hash);
    Ok((scenario, facts))
}
