//! `scenario emit-trace` / `scenario compile` command handlers.
//!
//! `scenario compile` (M04.3) collapses the three things a scenario produces —
//! a bundle-bound trace, the target-neutral Formal IR ("formal facts"), and the
//! declared replay expectation — into ONE command over a single scenario+bundle
//! source, reusing the existing pure projections rather than re-deriving them.

use causlane_codegen::{CodegenContracts, FormalIr, FormalIrBuilder};
use causlane_contracts::CompiledDispatchBundle;
use causlane_replay::{ExpectedReplayResult, ReplayError, ReplayScenario, ReplayTrace};
use serde::Serialize;

use causlane_cli::scenario_facts::alloy_scenario_facts_from_yaml;

use crate::{read_bundle, read_file, write_file, CliError};

pub(crate) fn emit_trace_from_scenario(
    scenario_path: &str,
    out: &str,
    bundle_path: Option<&str>,
    kernel_secret: Option<&str>,
) -> Result<String, CliError> {
    let yaml = read_file(scenario_path)?;
    let scenario = ReplayScenario::from_yaml_str(&yaml)?;
    // When a bundle is supplied, stamp its hash into the emitted trace so the
    // trace is explicitly bound to that compiled bundle (P0-005). Without it the
    // trace is left unbound and only lenient verification will accept it.
    let bundle_hash = match bundle_path {
        Some(path) => Some(read_bundle(path)?.bundle_hash.0),
        None => None,
    };
    let bound = bundle_hash.is_some();
    let mut trace = scenario.to_trace_bound(bundle_hash);
    // With a kernel secret, mint valid keyed capability attestations so the emitted
    // trace passes attested verification (`replay verify --kernel-secret`) — P1-006.
    if let Some(secret) = kernel_secret {
        trace.mint_capability_attestations(secret.as_bytes())?;
    }
    let scenario_hash = ReplayScenario::scenario_hash(&yaml)?;
    write_file(out, &to_json_pretty(&trace)?)?;
    let attested = if kernel_secret.is_some() {
        " attested"
    } else {
        ""
    };
    let binding = if bound { "bundle-bound" } else { "unbound" };
    Ok(format!(
        "ok: wrote {out} [{binding}{attested}]; scenario_hash {scenario_hash}"
    ))
}

/// The scenario's declared replay expectation, emitted as a standalone manifest
/// so a contract test / CI gate can assert the expected outcome without parsing
/// the whole scenario.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ScenarioExpectation {
    pub scenario_id: String,
    pub scenario_hash: String,
    pub expected_result: ExpectedReplayResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_error_code: Option<String>,
}

/// The three artifacts a scenario compiles to, from one source.
pub(crate) struct CompiledScenario {
    pub trace: ReplayTrace,
    pub formal_ir: FormalIr,
    pub expectation: ScenarioExpectation,
}

/// Compile a scenario (YAML) against a bundle into its trace, Formal IR and
/// expectation manifest. Pure (no I/O): reuses `to_trace_bound` (bundle binding,
/// P0-005), `mint_capability_attestations` (P1-006), `alloy_scenario_facts_from_yaml`
/// and `CodegenContracts::build_ir` — the same projections `emit-trace` and
/// `formal ir emit` use, so the compiled artifacts are byte-identical to the
/// separate commands.
///
/// # Errors
/// [`CliError`] when the scenario YAML is invalid or IR construction fails.
pub(crate) fn compile_scenario(
    scenario_yaml: &str,
    bundle: &CompiledDispatchBundle,
    kernel_secret: Option<&str>,
) -> Result<CompiledScenario, CliError> {
    let scenario = ReplayScenario::from_yaml_str(scenario_yaml)?;
    let scenario_hash = ReplayScenario::scenario_hash(scenario_yaml)?;

    let mut trace = scenario.to_trace_bound(Some(bundle.bundle_hash.0.clone()));
    if let Some(secret) = kernel_secret {
        trace.mint_capability_attestations(secret.as_bytes())?;
    }

    let facts = alloy_scenario_facts_from_yaml(scenario_yaml)?;
    let formal_ir = CodegenContracts.build_ir(bundle, Some(&facts))?;

    let expectation = ScenarioExpectation {
        scenario_id: scenario.scenario_id.clone(),
        scenario_hash,
        expected_result: scenario.expected_replay_result,
        expected_error_code: scenario.expected_error_code.clone(),
    };

    Ok(CompiledScenario {
        trace,
        formal_ir,
        expectation,
    })
}

/// `scenario compile`: write the three artifacts (`<id>.trace.json`,
/// `<id>.formal-ir.json`, `<id>.expectation.json`) to `out_dir` and summarize.
pub(crate) fn compile_scenario_to_dir(
    scenario_path: &str,
    bundle_path: &str,
    out_dir: &str,
    kernel_secret: Option<&str>,
) -> Result<String, CliError> {
    let yaml = read_file(scenario_path)?;
    let bundle = read_bundle(bundle_path)?;
    let compiled = compile_scenario(&yaml, &bundle, kernel_secret)?;
    let id = &compiled.expectation.scenario_id;

    let trace_path = format!("{out_dir}/{id}.trace.json");
    let ir_path = format!("{out_dir}/{id}.formal-ir.json");
    let exp_path = format!("{out_dir}/{id}.expectation.json");

    write_file(&trace_path, &to_json_pretty(&compiled.trace)?)?;
    write_file(&ir_path, &compiled.formal_ir.to_json_pretty()?)?;
    write_file(&exp_path, &to_json_pretty(&compiled.expectation)?)?;

    Ok(format!(
        "ok: compiled {id} -> trace, formal-ir, expectation in {out_dir} \
         (bundle {}, scenario_hash {})",
        bundle.bundle_hash.0, compiled.expectation.scenario_hash
    ))
}

fn to_json_pretty<T: Serialize>(value: &T) -> Result<String, CliError> {
    serde_json::to_string_pretty(value)
        .map_err(|err| CliError::Replay(ReplayError::Decode(err.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use causlane_contracts::RegistryManifest;

    type TestResult = Result<(), CliError>;

    const REGISTRY: &str =
        include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");
    const SUCCESS: &str =
        include_str!("../fixtures/contracts/scenarios/release_promote_success.scenario.yaml");
    const INVALID: &str = include_str!(
        "../fixtures/contracts/scenarios/execution_without_barrier_invalid.scenario.yaml"
    );

    fn bundle() -> Result<CompiledDispatchBundle, CliError> {
        let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
        Ok(CompiledDispatchBundle::compile(&manifest)?)
    }

    #[test]
    fn compiles_success_to_bound_trace_ir_and_expectation() -> TestResult {
        let bundle = bundle()?;
        let compiled = compile_scenario(SUCCESS, &bundle, None)?;

        // Trace is bundle-bound and strictly verifies against the same bundle.
        assert_eq!(
            compiled.trace.bundle_hash.as_deref(),
            Some(bundle.bundle_hash.0.as_str())
        );
        compiled.trace.verify_with_bundle_strict(&bundle)?;

        // Formal IR is scenario-bound and carries the pass expectation.
        assert_eq!(
            compiled.formal_ir.scenario_hash.as_deref(),
            Some(compiled.expectation.scenario_hash.as_str())
        );
        assert_eq!(compiled.formal_ir.expected_result.as_deref(), Some("pass"));

        // Expectation manifest.
        assert_eq!(compiled.expectation.scenario_id, "release_promote_success");
        assert_eq!(
            compiled.expectation.expected_result,
            ExpectedReplayResult::Pass
        );
        assert!(compiled.expectation.expected_error_code.is_none());
        Ok(())
    }

    #[test]
    fn compiles_negative_with_expected_error_code() -> TestResult {
        let bundle = bundle()?;
        let compiled = compile_scenario(INVALID, &bundle, None)?;

        assert_eq!(
            compiled.expectation.expected_result,
            ExpectedReplayResult::Fail
        );
        assert_eq!(
            compiled.expectation.expected_error_code.as_deref(),
            Some("ExecutionWithoutBarrier")
        );
        assert_eq!(compiled.formal_ir.expected_result.as_deref(), Some("fail"));
        // The emitted trace, replayed, actually fails as the expectation declares.
        assert!(compiled.trace.verify_with_bundle(&bundle).is_err());
        Ok(())
    }
}
