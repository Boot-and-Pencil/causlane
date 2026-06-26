//! M04.1 golden success: CONFLICT-FREE PARALLELISM.
//!
//! Two independent `release.promote_candidate` (`RuntimeExecution`) actions run
//! interleaved in one trace with DISJOINT write scopes, so their exclusive
//! leases never overlap and both barrier ceremonies complete. This is the
//! positive control for the I-006 lease no-overlap invariant — the complement
//! of `conflicting_leases_invalid`. These integration tests live outside
//! `src/tests.rs` (which is at the file-length limit) and exercise only the
//! public API.

use causlane_contracts::{CompiledDispatchBundle, RegistryManifest};
use causlane_replay::{ExpectedReplayResult, ReplayError, ReplayScenario};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const REGISTRY: &str = include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");
const SCENARIO: &str =
    include_str!("../fixtures/contracts/scenarios/conflict_free_parallelism_success.scenario.yaml");

fn bundle() -> Result<CompiledDispatchBundle, Box<dyn std::error::Error>> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
    Ok(CompiledDispatchBundle::compile(&manifest)?)
}

/// Two parallel actions on disjoint scopes replay-validate together.
#[test]
fn conflict_free_parallelism_scenario_verifies() -> TestResult {
    let scenario = ReplayScenario::from_yaml_str(SCENARIO)?;
    assert_eq!(scenario.expected_replay_result, ExpectedReplayResult::Pass);

    let actions: std::collections::BTreeSet<&str> = scenario
        .events
        .iter()
        .map(|event| event.action_id.as_str())
        .collect();
    assert_eq!(
        actions.len(),
        2,
        "fixture must exercise two parallel actions"
    );

    scenario.to_trace().verify_with_bundle(&bundle()?)?;
    Ok(())
}

/// Non-vacuity: collapse the second action's environment write onto the first
/// action's exclusive scope and the conflict checker (I-006) must reject the
/// trace. If the success fixture's scopes were not actually disjoint, this
/// mutation would be a no-op and the success above would prove nothing.
#[test]
fn overlapping_scopes_are_rejected() -> TestResult {
    let mut scenario = ReplayScenario::from_yaml_str(SCENARIO)?;
    for event in &mut scenario.events {
        if event.event_id.as_deref() == Some("evt_b_leases_granted") {
            for lease in &mut event.leases {
                if lease.lease_id == "lease_b_env_prod" {
                    lease.scope = "environment:staging".to_string();
                }
            }
        }
    }

    let result = scenario.to_trace().verify_with_bundle(&bundle()?);
    assert!(
        matches!(result, Err(ReplayError::ConflictingLeases { .. })),
        "overlapping exclusive leases must be rejected, got {result:?}"
    );
    Ok(())
}
