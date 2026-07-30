//! Owner contract for deterministic scenario-runtime delivery.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ContractError;

/// Stable schema for the Causlane-owned scenario-runtime provider manifest.
pub const SCENARIO_RUNTIME_PROVIDER_SCHEMA: &str = "causlane.scenario-runtime-provider.v1";

/// Reset behavior exposed to product composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioResetSemantics {
    /// Restore the selected fixture and logical clock to their immutable baseline.
    RestoreFixtureBaseline,
}

/// Typed capabilities supplied by Causlane to the shared product projection.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioRuntimeProviderManifest {
    /// Schema identity.
    pub schema_version: String,
    /// Immutable provider identity.
    pub provider_id: String,
    /// Canonical scenario-pack reference.
    pub scenario_pack_ref: String,
    /// Fixed logical clock used by the deterministic catalog.
    pub fixed_clock_millis: i64,
    /// Explicit reset semantics.
    pub reset_semantics: ScenarioResetSemantics,
    /// Whether strict replay verification is supported.
    pub replay_supported: bool,
    /// Whether runtime switching after startup is allowed.
    pub runtime_switch_supported: bool,
    /// Scenario IDs offered by the provider.
    pub scenario_ids: Vec<String>,
    /// Contract and audit evidence paths bound to the release.
    pub evidence_paths: Vec<String>,
}

impl ScenarioRuntimeProviderManifest {
    /// Validates deterministic runtime and evidence requirements.
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.schema_version != SCENARIO_RUNTIME_PROVIDER_SCHEMA {
            return Err(ContractError::Validation(
                "scenario runtime provider schema drift".to_owned(),
            ));
        }
        if self.provider_id.trim().is_empty() || self.scenario_pack_ref.trim().is_empty() {
            return Err(ContractError::Validation(
                "scenario runtime provider identity is required".to_owned(),
            ));
        }
        if !self.replay_supported {
            return Err(ContractError::Validation(
                "strict replay support is required".to_owned(),
            ));
        }
        if self.runtime_switch_supported {
            return Err(ContractError::Validation(
                "runtime world switching is forbidden".to_owned(),
            ));
        }
        require_unique_non_empty("scenario_ids", &self.scenario_ids)?;
        require_unique_non_empty("evidence_paths", &self.evidence_paths)
    }
}

fn require_unique_non_empty(role: &str, values: &[String]) -> Result<(), ContractError> {
    if values.is_empty() {
        return Err(ContractError::Validation(format!("{role} is required")));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() || !seen.insert(value) {
            return Err(ContractError::Validation(format!(
                "{role} must contain unique non-empty values"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Result<ScenarioRuntimeProviderManifest, ContractError> {
        serde_json::from_slice(include_bytes!(
            "../../../fixtures/scenario-runtime/v1/scenario-provider.json"
        ))
        .map_err(ContractError::from)
    }

    #[test]
    fn scenario_provider_fixture_is_deterministic_and_replayable() -> Result<(), ContractError> {
        let manifest = fixture()?;
        manifest.validate()?;
        assert_eq!(
            manifest.reset_semantics,
            ScenarioResetSemantics::RestoreFixtureBaseline
        );
        assert!(manifest.replay_supported);
        assert!(!manifest.runtime_switch_supported);
        Ok(())
    }

    #[test]
    fn scenario_provider_fails_closed_without_replay_or_with_runtime_switch(
    ) -> Result<(), ContractError> {
        let manifest = fixture()?;
        assert!(ScenarioRuntimeProviderManifest {
            replay_supported: false,
            ..manifest.clone()
        }
        .validate()
        .is_err());
        assert!(ScenarioRuntimeProviderManifest {
            runtime_switch_supported: true,
            ..manifest
        }
        .validate()
        .is_err());
        Ok(())
    }
}
