//! P2-004: a replay-valid MULTI-ACTION reference trace.
//!
//! A single trace can carry more than one action. `RuntimeExecution` predicates
//! require a full barrier ceremony per action, so a minimal multi-action trace
//! cannot replay-validate against them; a non-`RuntimeExecution` (`EvidenceMeta`)
//! predicate has the `admitted -> planned -> dispatch_logged -> closed` lifecycle
//! with no barrier obligation, which a multi-action history satisfies. These
//! integration tests live outside `src/tests.rs` (which is at the file-length
//! limit) and exercise only the public API.

use causlane_contracts::{CompiledDispatchBundle, RegistryManifest};
use causlane_replay::{EventKindDto, ExpectedReplayResult, ReplayError, ReplayScenario};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const REGISTRY: &str =
    include_str!("../fixtures/contracts/examples/multi_action_reference.registry.yaml");
const SCENARIO: &str =
    include_str!("../fixtures/contracts/scenarios/multi_action_reference.scenario.yaml");

fn bundle() -> Result<CompiledDispatchBundle, Box<dyn std::error::Error>> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
    Ok(CompiledDispatchBundle::compile(&manifest)?)
}

/// The reference trace carries two independent actions and replay-validates.
#[test]
fn multi_action_reference_scenario_verifies() -> TestResult {
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
        "fixture must exercise more than one action"
    );

    scenario.to_trace().verify_with_bundle(&bundle()?)?;
    Ok(())
}

/// Non-vacuity for the per-action lifecycle generalization: the secondary action's
/// own lifecycle is reduced, not just the primary's. Dropping `act_ref_b`'s admit
/// and plan leaves it starting at `dispatch.logged` — a forbidden
/// `New -> DispatchLogged` transition. Under the old single-action filter the
/// secondary action's events were ignored and this passed.
#[test]
fn secondary_action_lifecycle_is_validated() -> TestResult {
    let mut scenario = ReplayScenario::from_yaml_str(SCENARIO)?;
    scenario.events.retain(|event| {
        !(event.action_id == "act_ref_b"
            && matches!(
                event.kind,
                EventKindDto::ActionAdmitted | EventKindDto::ActionPlanned
            ))
    });

    let result = scenario.to_trace().verify_with_bundle(&bundle()?);
    assert!(
        matches!(result, Err(ReplayError::Lifecycle(_))),
        "secondary action's broken lifecycle must be rejected, got {result:?}"
    );
    Ok(())
}
