//! P1-002: the Alloy generator emits a distinct `Action`/`Plan` atom per action,
//! rather than collapsing a multi-action scenario onto one atom.

use causlane_codegen::{
    generate_alloy_facts_with_scenario, AlloyEventKind, AlloyScenarioEvent, AlloyScenarioFacts,
};
use causlane_contracts::{CompiledDispatchBundle, RegistryManifest};

const REGISTRY: &str = include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");
const SHA_A: &str = "sha256:a241276dc389e9197710cd415072e850036429799d636873f47ae8a1bc44d47b";
const SHA_B: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";

fn event(
    event_id: &str,
    kind: AlloyEventKind,
    action: &str,
    plan: Option<&str>,
) -> AlloyScenarioEvent {
    AlloyScenarioEvent {
        event_id: event_id.to_owned(),
        kind,
        action_id: Some(action.to_owned()),
        plan_hash: plan.map(ToOwned::to_owned),
        op_index: None,
        fact_kind: None,
        scope: None,
        anchors: Vec::new(),
        anchor_facts: Vec::new(),
        leases: Vec::new(),
        barrier: None,
        capability: None,
        authz_decision: None,
    }
}

#[test]
fn multi_action_scenario_emits_distinct_action_and_plan_atoms(
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
    let bundle = CompiledDispatchBundle::compile(&manifest)?;

    // Two actions (readiness + promotion), each admitted and planned under its own
    // plan hash, in one scenario.
    let scenario = AlloyScenarioFacts {
        scenario_hash: SHA_A.to_owned(),
        action_id: "act_readiness".to_owned(),
        plan_hash: SHA_A.to_owned(),
        expected_result: "pass".to_owned(),
        expected_error_code: None,
        formal_obligations: Vec::new(),
        predicate_id: "release.promote_candidate".to_owned(),
        subject: Vec::new(),
        circumstance: Vec::new(),
        events: vec![
            event(
                "evt_r_admit",
                AlloyEventKind::ActionAdmitted,
                "act_readiness",
                None,
            ),
            event(
                "evt_r_plan",
                AlloyEventKind::ActionPlanned,
                "act_readiness",
                Some(SHA_A),
            ),
            event(
                "evt_p_admit",
                AlloyEventKind::ActionAdmitted,
                "act_promote",
                None,
            ),
            event(
                "evt_p_plan",
                AlloyEventKind::ActionPlanned,
                "act_promote",
                Some(SHA_B),
            ),
        ],
    };

    let text = generate_alloy_facts_with_scenario(&bundle, &scenario)?.text;

    // Two distinct Action sigs and two distinct Plan sigs — not collapsed.
    assert!(text.contains("one sig Action_act_readiness extends Action {}"));
    assert!(text.contains("one sig Action_act_promote extends Action {}"));
    assert!(text.contains("Action = Action_act_readiness + Action_act_promote"));
    assert!(text.contains("one sig Plan_sha256_a241"));
    assert!(text.contains("one sig Plan_sha256_1111"));
    // The bounded scope counts both actions and both plans.
    assert!(text.contains("exactly 2 Action, exactly 2 PlanHash"));
    // Per-event binding: the promotion events name the promotion action/plan.
    assert!(text.contains("Event_evt_p_admit.action = Action_act_promote"));
    assert!(text.contains("Event_evt_p_plan.planHash = Plan_sha256_1111"));

    Ok(())
}
