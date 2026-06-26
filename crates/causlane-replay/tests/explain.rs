//! M04.4: replay `explain` diagnostics — exact violated invariant + causal
//! location for accepted and rejected traces. Integration tests (outside
//! `src/tests.rs`, which is at the file-length limit) over the public API.

use causlane_contracts::{CompiledDispatchBundle, RegistryManifest};
use causlane_replay::ReplayScenario;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const REGISTRY: &str = include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");
const SUCCESS: &str =
    include_str!("../fixtures/contracts/scenarios/release_promote_success.scenario.yaml");
const EXEC_WITHOUT_BARRIER: &str =
    include_str!("../fixtures/contracts/scenarios/execution_without_barrier_invalid.scenario.yaml");
const CONFLICTING_LEASES: &str =
    include_str!("../fixtures/contracts/scenarios/conflicting_leases_invalid.scenario.yaml");
const MISSING_WITNESS: &str =
    include_str!("../fixtures/contracts/scenarios/missing_witness_invalid.scenario.yaml");
const ANCHOR_WRONG_FACT: &str = include_str!(
    "../fixtures/contracts/scenarios/projection_anchor_wrong_fact_invalid.scenario.yaml"
);

fn bundle() -> Result<CompiledDispatchBundle, Box<dyn std::error::Error>> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
    Ok(CompiledDispatchBundle::compile(&manifest)?)
}

fn authz_required_bundle() -> Result<CompiledDispatchBundle, Box<dyn std::error::Error>> {
    let yaml = REGISTRY.replace(
        "mode: disabled_for_local_dev\n      allowed_in_profiles: [RuntimeExecution]\n      rationale: demo fixture without real PDP",
        "mode: required\n      stages: [execution_barrier_logged]\n      policy_id: demo-policy\n      policy_version: \"1\"",
    );
    let manifest = RegistryManifest::from_yaml_str(&yaml)?;
    Ok(CompiledDispatchBundle::compile(&manifest)?)
}

/// An accepted trace explains as accepted with no invariant/error/location, and
/// the empty causal location is omitted from the JSON.
#[test]
fn explain_accepts_valid_trace() -> TestResult {
    let scenario = ReplayScenario::from_yaml_str(SUCCESS)?;
    let explain = scenario.to_trace().verify_explain(&bundle()?);

    assert!(explain.accepted);
    assert!(explain.invariant.is_none());
    assert!(explain.error_code.is_none());
    assert!(explain.causal_location.is_empty());
    assert_eq!(explain.checked_invariants.len(), 7);

    let json = explain.to_json_pretty()?;
    assert!(json.contains("\"accepted\": true"));
    assert!(
        !json.contains("causal_location"),
        "empty causal location must be omitted, got {json}"
    );
    Ok(())
}

/// A barrier violation names the exact invariant (I-001), stable code, and the
/// offending action in the causal location and human output.
#[test]
fn explain_rejection_names_invariant_and_action() -> TestResult {
    let scenario = ReplayScenario::from_yaml_str(EXEC_WITHOUT_BARRIER)?;
    let explain = scenario.to_trace().verify_explain(&bundle()?);

    assert!(!explain.accepted);
    assert_eq!(explain.invariant.as_deref(), Some("I-001"));
    assert_eq!(
        explain.error_code.as_deref(),
        Some("ExecutionWithoutBarrier")
    );
    assert!(explain.causal_location.action_id.is_some());

    let human = explain.to_human();
    assert!(human.contains("I-001"), "human: {human}");
    assert!(human.contains("ExecutionWithoutBarrier"), "human: {human}");
    assert!(human.contains("action="), "human: {human}");

    let json = explain.to_json_pretty()?;
    assert!(json.contains("\"invariant\": \"I-001\""), "json: {json}");
    assert!(json.contains("causal_location"), "json: {json}");
    Ok(())
}

/// A lease conflict locates the contended scope (I-006) — a different causal
/// field than the barrier case, proving per-variant extraction.
#[test]
fn explain_rejection_locates_lease_conflict_scope() -> TestResult {
    let scenario = ReplayScenario::from_yaml_str(CONFLICTING_LEASES)?;
    let explain = scenario.to_trace().verify_explain(&bundle()?);

    assert!(!explain.accepted);
    assert_eq!(explain.invariant.as_deref(), Some("I-006"));
    assert_eq!(explain.error_code.as_deref(), Some("ConflictingLeases"));
    assert!(explain.causal_location.scope.is_some());
    assert!(explain.causal_location.action_id.is_none());
    Ok(())
}

/// Projection anchor attestation failures keep the I-003 error metadata aligned
/// across the structured value, JSON, and human output.
#[test]
fn explain_rejection_locates_anchor_metadata() -> TestResult {
    let scenario = ReplayScenario::from_yaml_str(ANCHOR_WRONG_FACT)?;
    let explain = scenario.to_trace().verify_explain(&bundle()?);

    assert!(!explain.accepted);
    assert_eq!(explain.invariant.as_deref(), Some("I-003"));
    assert_eq!(
        explain.error_code.as_deref(),
        Some("AnchorAttestationMismatch")
    );
    let event_id = explain
        .causal_location
        .event_id
        .as_deref()
        .ok_or("anchor rejection must name the projection event")?;
    let anchor_id = explain
        .causal_location
        .anchor_event_id
        .as_deref()
        .ok_or("anchor rejection must name the anchored event")?;

    let json = explain.to_json_pretty()?;
    assert!(
        json.contains("\"error_code\": \"AnchorAttestationMismatch\""),
        "json: {json}"
    );
    assert!(json.contains(event_id), "json: {json}");
    assert!(json.contains(anchor_id), "json: {json}");

    let human = explain.to_human();
    assert!(human.contains("I-003"), "human: {human}");
    assert!(
        human.contains("AnchorAttestationMismatch"),
        "human: {human}"
    );
    assert!(human.contains("event="), "human: {human}");
    assert!(human.contains("anchor="), "human: {human}");
    Ok(())
}

/// Witness failures report the required witness id rather than falling back to
/// an opaque error string.
#[test]
fn explain_rejection_locates_witness_requirement() -> TestResult {
    let scenario = ReplayScenario::from_yaml_str(MISSING_WITNESS)?;
    let explain = scenario.to_trace().verify_explain(&bundle()?);

    assert!(!explain.accepted);
    assert_eq!(explain.invariant.as_deref(), Some("I-009"));
    assert_eq!(
        explain.error_code.as_deref(),
        Some("RequiredWitnessMissing")
    );
    assert!(explain.causal_location.requirement_id.is_some());

    let human = explain.to_human();
    assert!(human.contains("requirement="), "human: {human}");
    Ok(())
}

/// Authz failures populate the lifecycle stage in the causal location.
#[test]
fn explain_rejection_locates_authz_stage() -> TestResult {
    let scenario = ReplayScenario::from_yaml_str(SUCCESS)?;
    let explain = scenario
        .to_trace()
        .verify_explain(&authz_required_bundle()?);

    assert!(!explain.accepted);
    assert_eq!(explain.invariant.as_deref(), Some("I-009"));
    assert_eq!(explain.error_code.as_deref(), Some("AuthzDecisionMissing"));
    assert_eq!(
        explain.causal_location.stage.as_deref(),
        Some("execution_barrier_logged")
    );

    let json = explain.to_json_pretty()?;
    assert!(
        json.contains("\"stage\": \"execution_barrier_logged\""),
        "json: {json}"
    );
    let human = explain.to_human();
    assert!(
        human.contains("stage=execution_barrier_logged"),
        "human: {human}"
    );
    Ok(())
}

/// Bundle-hash provenance failures are structural: they carry a stable code and
/// detail but no protocol invariant or causal location.
#[test]
fn explain_rejection_reports_bundle_hash_structural_error() -> TestResult {
    let bundle = bundle()?;
    let scenario = ReplayScenario::from_yaml_str(SUCCESS)?;
    let mut trace = scenario.to_trace();
    trace.bundle_hash =
        Some("sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned());
    let explain = trace.verify_explain(&bundle);

    assert!(!explain.accepted);
    assert!(explain.invariant.is_none());
    assert_eq!(explain.error_code.as_deref(), Some("BundleHashMismatch"));
    assert!(explain.causal_location.is_empty());

    let json = explain.to_json_pretty()?;
    assert!(
        json.contains("\"error_code\": \"BundleHashMismatch\""),
        "json: {json}"
    );
    assert!(!json.contains("causal_location"), "json: {json}");

    let human = explain.to_human();
    assert!(human.contains("(structural)"), "human: {human}");
    assert!(human.contains("BundleHashMismatch"), "human: {human}");
    Ok(())
}

#[test]
fn explain_accepts_trace_with_matching_bundle_hash() -> TestResult {
    let bundle = bundle()?;
    let scenario = ReplayScenario::from_yaml_str(SUCCESS)?;
    let mut trace = scenario.to_trace();
    trace.bundle_hash = Some(bundle.bundle_hash.0.clone());
    let explain = trace.verify_explain(&bundle);

    assert!(explain.accepted);
    assert!(explain.error_code.is_none());
    assert_eq!(explain.bundle_hash, bundle.bundle_hash.0);
    assert!(explain.trace_hash.starts_with("sha256:"));
    Ok(())
}
