//! M04.1 golden success: PROJECTION + READ-ONLY SIDECAR (mixed-predicate traces).
//!
//! A `RuntimeExecution` producer commits observed truth; a `ProjectionRead` reader
//! projects that truth, anchored to the producer's `observed_truth.committed`,
//! without executing or committing anything. The reader resolves to its own
//! `ProjectionRead` profile via the scenario's per-action `actions` roster. These
//! integration tests live outside `src/tests.rs` (at the file-length limit) and
//! exercise only the public API.

use causlane_contracts::{CompiledDispatchBundle, RegistryManifest};
use causlane_replay::{ExpectedReplayResult, ReplayError, ReplayScenario};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const REGISTRY: &str =
    include_str!("../fixtures/contracts/examples/projection_readonly.registry.yaml");
const SIDECAR: &str =
    include_str!("../fixtures/contracts/scenarios/read_only_sidecar_success.scenario.yaml");
const PROJECTION: &str =
    include_str!("../fixtures/contracts/scenarios/projection_success.scenario.yaml");

fn bundle() -> Result<CompiledDispatchBundle, Box<dyn std::error::Error>> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
    Ok(CompiledDispatchBundle::compile(&manifest)?)
}

/// The read-only sidecar projects the producer's observed truth and verifies.
#[test]
fn read_only_sidecar_scenario_verifies() -> TestResult {
    let scenario = ReplayScenario::from_yaml_str(SIDECAR)?;
    assert_eq!(scenario.expected_replay_result, ExpectedReplayResult::Pass);

    let actions: std::collections::BTreeSet<&str> = scenario
        .events
        .iter()
        .map(|event| event.action_id.as_str())
        .collect();
    assert_eq!(actions.len(), 2, "fixture must carry producer + reader");
    assert_eq!(
        scenario.actions.len(),
        2,
        "fixture must declare a per-action predicate roster"
    );

    scenario.to_trace().verify_with_bundle(&bundle()?)?;
    Ok(())
}

/// The focused projection success scenario verifies.
#[test]
fn projection_scenario_verifies() -> TestResult {
    let scenario = ReplayScenario::from_yaml_str(PROJECTION)?;
    assert_eq!(scenario.expected_replay_result, ExpectedReplayResult::Pass);
    scenario.to_trace().verify_with_bundle(&bundle()?)?;
    Ok(())
}

/// Non-vacuity for the per-action predicate roster: drop the roster and the reader
/// is reduced against the trace-level `RuntimeExecution` profile, under which a
/// `dispatch.logged -> projection.emitted` reader lifecycle is forbidden. If the
/// roster were not load-bearing, the sidecar would verify without it.
#[test]
fn sidecar_without_action_roster_is_rejected() -> TestResult {
    let mut scenario = ReplayScenario::from_yaml_str(SIDECAR)?;
    scenario.actions.clear();

    let result = scenario.to_trace().verify_with_bundle(&bundle()?);
    assert!(
        matches!(result, Err(ReplayError::Lifecycle(_))),
        "reader reduced as RuntimeExecution must be rejected, got {result:?}"
    );
    Ok(())
}

/// Non-vacuity for the sidecar's anchor grounding: point the reader's projection
/// anchor at a fact the producer never observed and replay must refute it. Proves
/// the sidecar projection is genuinely grounded in the producer's truth (P0-004),
/// not self-asserted.
#[test]
fn sidecar_projection_with_ungrounded_fact_is_rejected() -> TestResult {
    let mut scenario = ReplayScenario::from_yaml_str(SIDECAR)?;
    for event in &mut scenario.events {
        if event.event_id.as_deref() == Some("evt_read_projection") {
            for anchor in &mut event.anchors {
                anchor.fact_kind = Some("fabricated_fact".to_string());
            }
        }
    }

    let result = scenario.to_trace().verify_with_bundle(&bundle()?);
    assert!(
        matches!(result, Err(ReplayError::AnchorAttestationMismatch { .. })),
        "ungrounded sidecar anchor must be rejected, got {result:?}"
    );
    Ok(())
}
