//! PUB1 parse-boundary property checks.
//!
//! These properties exercise public parsers and lowerers for determinism over
//! generated text. They intentionally delegate all semantic decisions to the
//! existing replay and contracts authorities.

use causlane_contracts::{CompiledDispatchBundle, RegistryManifest};
use causlane_replay::{ReplayEvent, ReplayScenario, ReplayTrace};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

const REGISTRY: &str = include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");
const SCENARIO: &str =
    include_str!("../fixtures/contracts/scenarios/release_promote_success.scenario.yaml");
const U32_EDGES: [u32; 3] = [0, 1, u32::MAX];
const U64_EDGES: [u64; 4] = [0, 1, u64::MAX - 1, u64::MAX];

fn text_input() -> impl Strategy<Value = String> {
    proptest::collection::vec(any::<char>(), 0..1024).prop_map(|chars| chars.into_iter().collect())
}

fn numeric_u32_edge() -> impl Strategy<Value = u32> {
    proptest::sample::select(U32_EDGES.to_vec())
}

fn numeric_u64_edge() -> impl Strategy<Value = u64> {
    proptest::sample::select(U64_EDGES.to_vec())
}

fn generated_error(err: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(format!("generated numeric-boundary document failed: {err}"))
}

fn apply_numeric_edges(events: &mut [ReplayEvent], value: u64, op_index: u32) {
    for event in events {
        event.occurred_at = Some(value);
        for lease in &mut event.leases {
            lease.amount = value;
            lease.epoch = value;
            lease.expires_at = Some(value);
            lease.holder_op_index = Some(op_index);
        }
        if let Some(barrier) = &mut event.execution_barrier {
            barrier.op_indexes = vec![0, op_index];
            for lease in &mut barrier.leases {
                lease.amount = value;
                lease.epoch = value;
                lease.expires_at = Some(value);
                lease.holder_op_index = Some(op_index);
            }
        }
        if let Some(authz) = &mut event.authz_decision {
            authz.issued_at = value;
            authz.expires_at = Some(value);
        }
        if let Some(capability) = &mut event.execution_capability {
            capability.op_index = op_index;
            capability.expires_at = Some(value);
        }
    }
}

fn numeric_trace_json(value: u64, op_index: u32) -> Result<String, TestCaseError> {
    let mut trace = ReplayScenario::from_yaml_str(SCENARIO)
        .map_err(generated_error)?
        .to_trace();
    apply_numeric_edges(&mut trace.events, value, op_index);
    serde_json::to_string(&trace).map_err(generated_error)
}

fn numeric_scenario_yaml(value: u64, op_index: u32) -> Result<String, TestCaseError> {
    let mut scenario = ReplayScenario::from_yaml_str(SCENARIO).map_err(generated_error)?;
    apply_numeric_edges(&mut scenario.events, value, op_index);
    serde_yaml::to_string(&scenario).map_err(generated_error)
}

fn numeric_registry_yaml(freshness: u64, version: u32) -> Result<String, TestCaseError> {
    let mut manifest = RegistryManifest::from_yaml_str(REGISTRY).map_err(generated_error)?;
    let Some(predicate) = manifest.predicates.first_mut() else {
        return Err(TestCaseError::fail("fixture registry has no predicate"));
    };
    predicate.version = version;
    predicate.authz_policy.freshness_max_age = Some(freshness);
    for protocol in &mut manifest.merge_protocols {
        protocol.version = version;
    }
    serde_yaml::to_string(&manifest).map_err(generated_error)
}

proptest! {
    #[test]
    fn replay_trace_json_parse_and_lowering_are_deterministic(input in text_input()) {
        let first = ReplayTrace::from_json_str(&input);
        let second = ReplayTrace::from_json_str(&input);
        prop_assert_eq!(&first, &second);

        if let Ok(trace) = first {
            prop_assert_eq!(trace.to_events(), trace.to_events());
        }
    }

    #[test]
    fn replay_scenario_yaml_parse_and_lowering_are_deterministic(input in text_input()) {
        let first = ReplayScenario::from_yaml_str(&input);
        let second = ReplayScenario::from_yaml_str(&input);
        prop_assert_eq!(&first, &second);

        if let Ok(scenario) = first {
            let first_trace = scenario.to_trace();
            let second_trace = scenario.to_trace();
            prop_assert_eq!(&first_trace, &second_trace);
            prop_assert_eq!(first_trace.to_events(), second_trace.to_events());
        }
    }

    #[test]
    fn registry_yaml_parse_and_compile_are_deterministic(input in text_input()) {
        let first = RegistryManifest::from_yaml_str(&input);
        let second = RegistryManifest::from_yaml_str(&input);
        prop_assert_eq!(&first, &second);

        if let Ok(manifest) = first {
            let first_bundle = CompiledDispatchBundle::compile(&manifest);
            let second_bundle = CompiledDispatchBundle::compile(&manifest);
            prop_assert_eq!(first_bundle, second_bundle);
        }
    }

    #[test]
    fn replay_trace_json_numeric_edges_parse_and_lower(
        value in numeric_u64_edge(),
        op_index in numeric_u32_edge(),
    ) {
        let json = numeric_trace_json(value, op_index)?;
        let trace = ReplayTrace::from_json_str(&json).map_err(generated_error)?;
        let first_events = trace.to_events().map_err(generated_error)?;
        let second_events = trace.to_events().map_err(generated_error)?;
        prop_assert_eq!(first_events, second_events);
    }

    #[test]
    fn replay_scenario_yaml_numeric_edges_parse_and_lower(
        value in numeric_u64_edge(),
        op_index in numeric_u32_edge(),
    ) {
        let yaml = numeric_scenario_yaml(value, op_index)?;
        let scenario = ReplayScenario::from_yaml_str(&yaml).map_err(generated_error)?;
        let first_trace = scenario.to_trace();
        let second_trace = scenario.to_trace();
        prop_assert_eq!(&first_trace, &second_trace);
        prop_assert_eq!(
            first_trace.to_events().map_err(generated_error)?,
            second_trace.to_events().map_err(generated_error)?
        );
    }

    #[test]
    fn registry_yaml_numeric_edges_parse_and_compile(
        freshness in numeric_u64_edge(),
        version in numeric_u32_edge(),
    ) {
        let yaml = numeric_registry_yaml(freshness, version)?;
        let manifest = RegistryManifest::from_yaml_str(&yaml).map_err(generated_error)?;
        let first_bundle = CompiledDispatchBundle::compile(&manifest).map_err(generated_error)?;
        let second_bundle = CompiledDispatchBundle::compile(&manifest).map_err(generated_error)?;
        prop_assert_eq!(first_bundle, second_bundle);
    }
}
