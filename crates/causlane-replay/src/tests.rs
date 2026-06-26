//! Tests for the trace loader and protocol verifier.

use causlane_contracts::{
    CompiledDispatchBundle, ContractError, RegistryManifest, TemplateBindings,
};
use causlane_core::{
    ActionId, AuditEvent, AuditEventId, AuditEventKind, ClaimMode, ConstraintEpoch, LeaseId,
    LeaseRef, PlanHash, PredicateId, ResourceId, Scope,
};

use crate::{
    verify_events, AuthzDecisionDto, EventKindDto, ExpectedReplayResult, ReplayAuthzDecision,
    ReplayError, ReplayEvent, ReplayScenario, ReplayTrace,
};

const TRACE: &str = include_str!("../fixtures/contracts/examples/release_promote.trace.json");
const REGISTRY: &str = include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");
const SCENARIO_SUCCESS: &str =
    include_str!("../fixtures/contracts/scenarios/release_promote_success.scenario.yaml");
const SCENARIO_EXEC_WITHOUT_BARRIER: &str =
    include_str!("../fixtures/contracts/scenarios/execution_without_barrier_invalid.scenario.yaml");
const SCENARIO_PROJECTION_WITHOUT_ANCHOR: &str =
    include_str!("../fixtures/contracts/scenarios/projection_without_anchor_invalid.scenario.yaml");
const SCENARIO_CONFLICTING_LEASES: &str =
    include_str!("../fixtures/contracts/scenarios/conflicting_leases_invalid.scenario.yaml");
const SCENARIO_MISSING_WITNESS: &str =
    include_str!("../fixtures/contracts/scenarios/missing_witness_invalid.scenario.yaml");
const SCENARIO_APPROVAL_WRONG_PLAN: &str =
    include_str!("../fixtures/contracts/scenarios/approval_wrong_plan_invalid.scenario.yaml");
const SCENARIO_EXEC_WITHOUT_CAPABILITY: &str = include_str!(
    "../fixtures/contracts/scenarios/execution_without_capability_invalid.scenario.yaml"
);
const SCENARIO_WITNESS_WRONG_SCOPE: &str =
    include_str!("../fixtures/contracts/scenarios/witness_wrong_scope_invalid.scenario.yaml");
const SCENARIO_EVENT_AFTER_CLOSED: &str =
    include_str!("../fixtures/contracts/scenarios/event_after_closed_invalid.scenario.yaml");
const SCENARIO_DRAIN_WITH_ACTIVE_LEASE: &str =
    include_str!("../fixtures/contracts/scenarios/drain_with_active_lease_invalid.scenario.yaml");
const SCENARIO_WITNESS_WRONG_FACT_KIND: &str = include_str!(
    "../fixtures/contracts/scenarios/witness_event_wrong_fact_kind_invalid.scenario.yaml"
);
const SCENARIO_ANCHOR_WRONG_FACT: &str = include_str!(
    "../fixtures/contracts/scenarios/projection_anchor_wrong_fact_invalid.scenario.yaml"
);

fn demo_bundle() -> Result<CompiledDispatchBundle, ContractError> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
    CompiledDispatchBundle::compile(&manifest)
}

fn authz_required_bundle() -> Result<CompiledDispatchBundle, ContractError> {
    let yaml = REGISTRY.replace(
        "mode: disabled_for_local_dev\n      allowed_in_profiles: [RuntimeExecution]\n      rationale: demo fixture without real PDP",
        "mode: required\n      stages: [execution_barrier_logged]\n      policy_id: demo-policy\n      policy_version: \"1\"",
    );
    let manifest = RegistryManifest::from_yaml_str(&yaml)?;
    CompiledDispatchBundle::compile(&manifest)
}

fn authz_fresh_bundle() -> Result<CompiledDispatchBundle, ContractError> {
    let yaml = REGISTRY.replace(
        "mode: disabled_for_local_dev\n      allowed_in_profiles: [RuntimeExecution]\n      rationale: demo fixture without real PDP",
        "mode: required\n      stages: [execution_barrier_logged]\n      policy_id: demo-policy\n      policy_version: \"1\"\n      freshness_max_age: 5",
    );
    let manifest = RegistryManifest::from_yaml_str(&yaml)?;
    CompiledDispatchBundle::compile(&manifest)
}

fn merge_applicable_bundle() -> Result<CompiledDispatchBundle, ContractError> {
    // Declare the bundle's verified merge protocol applicable to the predicate's
    // `promote_release` op so its conflict domains become mergeable (I-006).
    let yaml = REGISTRY.replace(
        "    scenario_refs:",
        "    merge_protocol_applicability:\n      - protocol_id: append_only_release_log_v1\n        applies_to: promote_release\n    scenario_refs:",
    );
    let manifest = RegistryManifest::from_yaml_str(&yaml)?;
    CompiledDispatchBundle::compile(&manifest)
}

// I-006 (per-protocol merge): without an applicable verified protocol the
// mergeable set is empty (fail-closed); with one, the effect's conflict domains
// resolve into the mergeable set and overlapping exclusive leases are permitted.
#[test]
fn merge_protocol_relaxes_lease_conflict_domains() -> Result<(), Box<dyn std::error::Error>> {
    let bindings = TemplateBindings::from_pairs(
        [
            ("release_candidate_id".to_owned(), "rc_123".to_owned()),
            ("target_environment".to_owned(), "staging".to_owned()),
        ],
        std::iter::empty::<(String, String)>(),
    );
    let predicate_id = PredicateId("release.promote_candidate".to_owned());
    let missing = || ReplayError::UnknownPredicate {
        predicate: predicate_id.0.clone(),
    };

    let plain = demo_bundle()?;
    let plain_pred = plain.predicate(&predicate_id).ok_or_else(missing)?;
    assert!(crate::resolve_mergeable_scopes(&plain, plain_pred, &bindings)?.is_empty());

    let applicable = merge_applicable_bundle()?;
    let applicable_pred = applicable.predicate(&predicate_id).ok_or_else(missing)?;
    let scopes = crate::resolve_mergeable_scopes(&applicable, applicable_pred, &bindings)?;
    assert!(scopes.contains(&Scope("environment:staging".to_owned())));
    assert!(scopes.contains(&Scope("release_candidate:rc_123".to_owned())));
    Ok(())
}

// I-007: the replay oracle refutes a drain fence acquired while a lease still
// actively overlaps the fence scope.
#[test]
fn drain_fence_with_active_lease_is_refuted() -> Result<(), Box<dyn std::error::Error>> {
    let scenario = ReplayScenario::from_yaml_str(SCENARIO_DRAIN_WITH_ACTIVE_LEASE)?;
    let result = scenario.to_trace().verify_with_bundle(&demo_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::DrainFenceWithActiveOverlap { .. })
    ));
    Ok(())
}

fn plan_hash(digit: char) -> Result<PlanHash, ReplayError> {
    let digest: String = (0..64).map(|_| digit).collect();
    let parsed = PlanHash::new(format!("sha256:{digest}"))?;
    Ok(parsed)
}

fn event(id: &str, kind: AuditEventKind, plan: &PlanHash) -> AuditEvent {
    AuditEvent::new(
        AuditEventId(id.to_owned()),
        ActionId("act".to_owned()),
        kind,
    )
    .with_plan_hash(plan.clone())
}

fn exclusive_lease(lease_id: &str, scope: &str, plan: &PlanHash) -> LeaseRef {
    LeaseRef {
        lease_id: LeaseId(lease_id.to_owned()),
        resource: ResourceId("resource".to_owned()),
        scope: Scope(scope.to_owned()),
        mode: ClaimMode::ExclusiveWrite,
        amount: 1,
        holder_action_id: ActionId("act".to_owned()),
        holder_plan_hash: plan.clone(),
        holder_op_index: None,
        epoch: ConstraintEpoch(0),
        expires_at: None,
        lease_event_id: AuditEventId("lease_evt".to_owned()),
    }
}

#[test]
fn example_trace_verifies() -> Result<(), ReplayError> {
    let trace = ReplayTrace::from_json_str(TRACE)?;
    trace.verify()?;
    Ok(())
}

#[test]
fn example_trace_verifies_with_bundle() -> Result<(), Box<dyn std::error::Error>> {
    let trace = ReplayTrace::from_json_str(TRACE)?;
    trace.verify_with_bundle(&demo_bundle()?)?;
    Ok(())
}

#[test]
fn success_scenario_emits_bundle_verified_trace() -> Result<(), Box<dyn std::error::Error>> {
    let scenario = ReplayScenario::from_yaml_str(SCENARIO_SUCCESS)?;
    assert_eq!(scenario.expected_replay_result, ExpectedReplayResult::Pass);
    assert!(ReplayScenario::scenario_hash(SCENARIO_SUCCESS)?.starts_with("sha256:"));

    let trace = scenario.to_trace();
    assert_eq!(
        trace.predicate.as_deref(),
        Some("release.promote_candidate")
    );
    trace.verify_with_bundle(&demo_bundle()?)?;
    Ok(())
}

#[test]
fn invalid_scenarios_fail_as_catalogued() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = demo_bundle()?;
    assert_scenario_fails(
        SCENARIO_EXEC_WITHOUT_BARRIER,
        |err| matches!(err, ReplayError::ExecutionWithoutBarrier { .. }),
        &bundle,
    )?;
    assert_scenario_fails(
        SCENARIO_PROJECTION_WITHOUT_ANCHOR,
        |err| matches!(err, ReplayError::ProjectionWithoutAnchor { .. }),
        &bundle,
    )?;
    assert_scenario_fails(
        SCENARIO_CONFLICTING_LEASES,
        |err| matches!(err, ReplayError::ConflictingLeases { .. }),
        &bundle,
    )?;
    assert_scenario_fails(
        SCENARIO_MISSING_WITNESS,
        |err| matches!(err, ReplayError::RequiredWitnessMissing { .. }),
        &bundle,
    )?;
    assert_scenario_fails(
        SCENARIO_APPROVAL_WRONG_PLAN,
        |err| matches!(err, ReplayError::WitnessBindingMismatch { .. }),
        &bundle,
    )?;
    assert_scenario_fails(
        SCENARIO_EXEC_WITHOUT_CAPABILITY,
        |err| matches!(err, ReplayError::CapabilityMissing { .. }),
        &bundle,
    )?;
    assert_scenario_fails(
        SCENARIO_WITNESS_WRONG_SCOPE,
        |err| matches!(err, ReplayError::WitnessSelectorMismatch { .. }),
        &bundle,
    )?;
    assert_scenario_fails(
        SCENARIO_WITNESS_WRONG_FACT_KIND,
        |err| matches!(err, ReplayError::WitnessAttestationMismatch { .. }),
        &bundle,
    )?;
    assert_scenario_fails(
        SCENARIO_ANCHOR_WRONG_FACT,
        |err| matches!(err, ReplayError::AnchorAttestationMismatch { .. }),
        &bundle,
    )?;
    Ok(())
}

fn assert_scenario_fails(
    yaml: &str,
    matches_expected: fn(&ReplayError) -> bool,
    bundle: &CompiledDispatchBundle,
) -> Result<(), Box<dyn std::error::Error>> {
    let scenario = ReplayScenario::from_yaml_str(yaml)?;
    assert_eq!(scenario.expected_replay_result, ExpectedReplayResult::Fail);
    match scenario.to_trace().verify_with_bundle(bundle) {
        Err(err) if matches_expected(&err) => {
            if let Some(expected_code) = scenario.expected_error_code {
                assert_eq!(err.code_token(), expected_code);
            }
            Ok(())
        }
        Err(err) => {
            Err(std::io::Error::other(format!("unexpected scenario error: {err:?}")).into())
        }
        Ok(()) => Err(std::io::Error::other("scenario unexpectedly passed").into()),
    }
}

#[test]
fn bundle_mode_rejects_missing_required_witness() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    for event in &mut trace.events {
        if event.kind == EventKindDto::ExecutionBarrierLogged {
            event.witnesses.clear();
            if let Some(barrier) = &mut event.execution_barrier {
                barrier.witnesses.clear();
            }
        }
    }
    let result = trace.verify_with_bundle(&demo_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::RequiredWitnessMissing { .. })
    ));
    Ok(())
}

#[test]
fn bundle_mode_rejects_missing_barrier_payload() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    for event in &mut trace.events {
        if event.kind == EventKindDto::ExecutionBarrierLogged {
            event.execution_barrier = None;
        }
    }
    let result = trace.verify_with_bundle(&demo_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::MissingBarrierPayload { .. })
    ));
    Ok(())
}

#[test]
fn bundle_mode_rejects_execution_without_capability() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    for event in &mut trace.events {
        if event.kind == EventKindDto::ExecutionStarted {
            event.execution_capability = None;
        }
    }
    let result = trace.verify_with_bundle(&demo_bundle()?);
    assert!(matches!(result, Err(ReplayError::CapabilityMissing { .. })));
    Ok(())
}

#[test]
fn bundle_mode_rejects_missing_required_authz_decision() -> Result<(), Box<dyn std::error::Error>> {
    let trace = ReplayTrace::from_json_str(TRACE)?;
    let result = trace.verify_with_bundle(&authz_required_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::AuthzDecisionMissing { .. })
    ));
    Ok(())
}

#[test]
fn bundle_mode_rejects_denied_authz_decision() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    insert_authz_decision(&mut trace, AuthzDecisionDto::Deny);
    let result = trace.verify_with_bundle(&authz_required_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::AuthzDecisionDenied { .. })
    ));
    Ok(())
}

#[test]
fn bundle_mode_accepts_bound_allow_authz_decision() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    insert_authz_decision(&mut trace, AuthzDecisionDto::Allow);
    trace.verify_with_bundle(&authz_required_bundle()?)?;
    Ok(())
}

// P0-010: an Allow decision that binds the right action/plan/stage but is issued
// under a DIFFERENT policy than the predicate requires is rejected, not accepted.
#[test]
fn bundle_mode_rejects_wrong_policy_authz_decision() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    insert_authz_decision_with_policy(&mut trace, AuthzDecisionDto::Allow, "attacker-policy", "1");
    let result = trace.verify_with_bundle(&authz_required_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::AuthzPolicyMismatch { .. })
    ));
    Ok(())
}

// P0-010 freshness: with the barrier carrying an `occurred_at`, an Allow whose
// expiry has passed by the barrier's evaluation time is rejected (decision
// expires_at=10 but the barrier was evaluated at t=20).
#[test]
fn bundle_mode_rejects_expired_authz_at_barrier_time() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    insert_authz_decision(&mut trace, AuthzDecisionDto::Allow);
    set_barrier_occurred_at(&mut trace, 20);
    let result = trace.verify_with_bundle(&authz_required_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::AuthzDecisionExpired { .. })
    ));
    Ok(())
}

// P0-010 freshness: a decision issued AFTER the barrier it authorizes (decision
// issued_at=1, barrier evaluated at t=0) is forward-dated and rejected.
#[test]
fn bundle_mode_rejects_authz_issued_after_barrier() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    insert_authz_decision(&mut trace, AuthzDecisionDto::Allow);
    set_barrier_occurred_at(&mut trace, 0);
    let result = trace.verify_with_bundle(&authz_required_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::AuthzIssuedAfterBarrier { .. })
    ));
    Ok(())
}

// ADR-0011 freshness: a non-expired Allow that is older than the policy's
// freshness_max_age at the barrier's evaluation time is stale (decision
// issued_at=1, barrier at t=8 -> age 7 > max_age 5, but expires_at=10 > 8).
#[test]
fn bundle_mode_rejects_stale_authz_decision() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    insert_authz_decision(&mut trace, AuthzDecisionDto::Allow);
    set_barrier_occurred_at(&mut trace, 8);
    let result = trace.verify_with_bundle(&authz_fresh_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::AuthzDecisionStale { .. })
    ));
    Ok(())
}

// ADR-0013: leases are time-bounded authority. A barrier that crosses execution
// while the leases it relies on have already expired (lease expires_at=5, barrier
// evaluated at t=10) must be rejected. Previously the lease check was called with
// `now: None`, making `LeaseTableError::Expired` dead code on the replay path.
#[test]
fn bundle_mode_rejects_expired_barrier_lease() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    set_lease_expiry(&mut trace, 5);
    set_barrier_occurred_at(&mut trace, 10);
    let result = trace.verify_with_bundle(&demo_bundle()?);
    assert!(
        matches!(&result, Err(ReplayError::Lease(msg)) if msg.contains("Expired")),
        "expected a lease-expiry rejection, got {result:?}",
    );
    Ok(())
}

// Control: the same trace with leases that expire AFTER the barrier evaluation
// time is still accepted, so the rejection above is about expiry, not the new
// time-binding itself.
#[test]
fn bundle_mode_accepts_fresh_barrier_lease() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    set_lease_expiry(&mut trace, 100);
    set_barrier_occurred_at(&mut trace, 10);
    assert!(trace.verify_with_bundle(&demo_bundle()?).is_ok());
    Ok(())
}

// ADR-0013 capability attestation: with a kernel secret configured, only a
// capability carrying a valid keyed attestation over its canonical bytes is
// accepted. An attacker who authored the trace but lacks the secret cannot mint
// a spendable capability, even though the structural id still matches.
#[test]
fn bundle_mode_attested_accepts_signed_and_rejects_forgery(
) -> Result<(), Box<dyn std::error::Error>> {
    let secret = b"kernel-secret-v1";
    let bundle = demo_bundle()?;

    // The canonical message is computed from the lowered capability.
    let base = ReplayTrace::from_json_str(TRACE)?;
    let message = base
        .to_events()?
        .iter()
        .find_map(|event| event.execution_capability.as_ref())
        .ok_or("demo trace carries a capability")?
        .attestation_message();
    let valid_tag = causlane_contracts::attestation::attest(secret, &message);

    // A correctly-signed capability is accepted under the kernel secret.
    let mut signed = ReplayTrace::from_json_str(TRACE)?;
    set_capability_attestation(&mut signed, Some(&valid_tag));
    assert!(signed.verify_with_bundle_attested(&bundle, secret).is_ok());

    // A missing attestation is rejected once a secret is configured.
    let missing = ReplayTrace::from_json_str(TRACE)?;
    assert!(matches!(
        missing.verify_with_bundle_attested(&bundle, secret),
        Err(ReplayError::CapabilityMismatch { .. })
    ));

    // An attacker without the secret cannot produce a tag the kernel accepts.
    let attacker_tag = causlane_contracts::attestation::attest(b"attacker-secret", &message);
    let mut forged = ReplayTrace::from_json_str(TRACE)?;
    set_capability_attestation(&mut forged, Some(&attacker_tag));
    assert!(matches!(
        forged.verify_with_bundle_attested(&bundle, secret),
        Err(ReplayError::CapabilityMismatch { .. })
    ));

    // Back-compat: with no secret configured, the unsigned demo trace verifies.
    assert!(base.verify_with_bundle(&bundle).is_ok());
    Ok(())
}

fn set_capability_attestation(trace: &mut ReplayTrace, tag: Option<&str>) {
    for event in &mut trace.events {
        if let Some(capability) = &mut event.execution_capability {
            capability.attestation = tag.map(ToOwned::to_owned);
        }
    }
}

fn set_lease_expiry(trace: &mut ReplayTrace, at: u64) {
    for event in &mut trace.events {
        for lease in &mut event.leases {
            lease.expires_at = Some(at);
        }
        if let Some(barrier) = &mut event.execution_barrier {
            for lease in &mut barrier.leases {
                lease.expires_at = Some(at);
            }
        }
        // The derived capability expiry tracks the leases' min expiry, so keep
        // it consistent or the capability check would fail for an unrelated
        // reason instead of (or before) the lease check we are exercising.
        if let Some(capability) = &mut event.execution_capability {
            capability.expires_at = Some(at);
        }
    }
}

fn set_barrier_occurred_at(trace: &mut ReplayTrace, at: u64) {
    for event in &mut trace.events {
        if event.kind == EventKindDto::ExecutionBarrierLogged {
            event.occurred_at = Some(at);
        }
    }
}

fn insert_authz_decision(trace: &mut ReplayTrace, decision: AuthzDecisionDto) {
    insert_authz_decision_with_policy(trace, decision, "demo-policy", "1");
}

fn insert_authz_decision_with_policy(
    trace: &mut ReplayTrace,
    decision: AuthzDecisionDto,
    policy_id: &str,
    policy_version: &str,
) {
    let plan_hash = trace.plan_hash.clone();
    let event = ReplayEvent {
        event_id: Some("evt_authz".to_owned()),
        kind: EventKindDto::AuthzDecisionRecorded,
        action_id: trace.action_id.clone(),
        plan_hash,
        witnesses: Vec::new(),
        witness_refs: Vec::new(),
        anchors: Vec::new(),
        leases: Vec::new(),
        impact_set_hash: None,
        execution_barrier: None,
        authz_decision: Some(ReplayAuthzDecision {
            decision_event_id: None,
            action_id: None,
            plan_hash: None,
            predicate_id: "release.promote_candidate".to_owned(),
            actor: "alice".to_owned(),
            stage: "execution_barrier_logged".to_owned(),
            decision,
            policy_id: policy_id.to_owned(),
            policy_version: policy_version.to_owned(),
            issued_at: 1,
            expires_at: Some(10),
            attestation: None,
        }),
        execution_capability: None,
        fact_kind: None,
        scope: None,
        occurred_at: None,
    };
    let barrier_position = trace
        .events
        .iter()
        .position(|event| event.kind == EventKindDto::ExecutionBarrierLogged);
    if let Some(position) = barrier_position {
        trace.events.insert(position, event);
    } else {
        trace.events.push(event);
    }
    for event in &mut trace.events {
        if let Some(barrier) = &mut event.execution_barrier {
            barrier.authz_decision_refs.push("evt_authz".to_owned());
        }
    }
}

#[test]
fn bundle_mode_rejects_wrong_witness_scope() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    for event in &mut trace.events {
        if let Some(barrier) = &mut event.execution_barrier {
            if let Some(witness) = barrier.witnesses.first_mut() {
                witness.scope = Some(String::new());
            }
        }
    }
    let result = trace.verify_with_bundle(&demo_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::WitnessSelectorMismatch { .. })
    ));
    Ok(())
}

#[test]
fn bundle_mode_rejects_wrong_witness_binding() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    for event in &mut trace.events {
        if let Some(barrier) = &mut event.execution_barrier {
            if let Some(witness) = barrier.witnesses.first_mut() {
                if let Some(binding) = &mut witness.binds_to {
                    binding.action_id = "other_action".to_owned();
                }
            }
        }
    }
    let result = trace.verify_with_bundle(&demo_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::WitnessBindingMismatch { .. })
    ));
    Ok(())
}

#[test]
fn bundle_mode_rejects_nonempty_but_wrong_witness_scope() -> Result<(), Box<dyn std::error::Error>>
{
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    for event in &mut trace.events {
        if let Some(barrier) = &mut event.execution_barrier {
            if let Some(witness) = barrier.witnesses.first_mut() {
                witness.scope = Some("release_candidate:other".to_owned());
            }
        }
    }
    let result = trace.verify_with_bundle(&demo_bundle()?);
    assert!(matches!(
        result,
        Err(ReplayError::WitnessSelectorMismatch { .. })
    ));
    Ok(())
}

#[test]
fn bundle_mode_rejects_missing_claim_lease() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    for event in &mut trace.events {
        if let Some(barrier) = &mut event.execution_barrier {
            barrier
                .leases
                .retain(|lease| lease.resource != "release_candidate_write");
        }
    }
    let result = trace.verify_with_bundle(&demo_bundle()?);
    assert!(matches!(result, Err(ReplayError::Lease(_))));
    Ok(())
}

#[test]
fn rejects_unknown_event_kind() {
    let json = r#"{ "trace_version": "0.1.0", "action_id": "a",
        "events": [ { "kind": "bogus.kind", "action_id": "a" } ] }"#;
    let parsed = ReplayTrace::from_json_str(json);
    assert!(matches!(parsed, Err(ReplayError::Decode(_))));
}

#[test]
fn rejects_todo_plan_hash() {
    let json = r#"{ "trace_version": "0.1.0", "action_id": "a", "plan_hash": "sha256:TODO",
        "events": [ { "kind": "action.planned", "action_id": "a", "plan_hash": "sha256:TODO" } ] }"#;
    let result = ReplayTrace::from_json_str(json).and_then(|trace| trace.verify());
    assert!(matches!(result, Err(ReplayError::BadPlanHash(_))));
}

#[test]
fn detects_execution_without_barrier() -> Result<(), ReplayError> {
    let plan = plan_hash('1')?;
    let events = vec![
        event("e0", AuditEventKind::ActionPlanned, &plan),
        event("e1", AuditEventKind::ExecutionStarted, &plan),
    ];
    assert!(matches!(
        verify_events(&events),
        Err(ReplayError::ExecutionWithoutBarrier { .. })
    ));
    Ok(())
}

#[test]
fn detects_observed_without_execution() -> Result<(), ReplayError> {
    let plan = plan_hash('1')?;
    let events = vec![
        event("e0", AuditEventKind::ExecutionBarrierLogged, &plan),
        event("e1", AuditEventKind::ObservedTruthCommitted, &plan),
    ];
    assert!(matches!(
        verify_events(&events),
        Err(ReplayError::ObservedWithoutExecution { .. })
    ));
    Ok(())
}

#[test]
fn detects_projection_without_anchor() -> Result<(), ReplayError> {
    let plan = plan_hash('1')?;
    let events = vec![event("e0", AuditEventKind::ProjectionEmitted, &plan)];
    assert!(matches!(
        verify_events(&events),
        Err(ReplayError::ProjectionWithoutAnchor { .. })
    ));
    Ok(())
}

#[test]
fn detects_plan_hash_mismatch() -> Result<(), ReplayError> {
    let plan_a = plan_hash('1')?;
    let plan_b = plan_hash('2')?;
    let events = vec![
        event("e0", AuditEventKind::ActionPlanned, &plan_a),
        event("e1", AuditEventKind::DispatchLogged, &plan_b),
    ];
    assert!(matches!(
        verify_events(&events),
        Err(ReplayError::PlanHashMismatch { .. })
    ));
    Ok(())
}

#[test]
fn detects_conflicting_exclusive_leases() -> Result<(), ReplayError> {
    let plan = plan_hash('1')?;
    let events = vec![
        event("e0", AuditEventKind::ConstraintLeaseGranted, &plan).with_leases(vec![
            exclusive_lease("lease_a", "environment:staging", &plan),
        ]),
        event("e1", AuditEventKind::ConstraintLeaseGranted, &plan).with_leases(vec![
            exclusive_lease("lease_b", "environment:staging", &plan),
        ]),
    ];
    assert!(matches!(
        verify_events(&events),
        Err(ReplayError::ConflictingLeases { .. })
    ));
    Ok(())
}

#[test]
fn detects_release_by_wrong_lease_id() -> Result<(), ReplayError> {
    let plan = plan_hash('1')?;
    let events = vec![
        event("e0", AuditEventKind::ConstraintLeaseGranted, &plan).with_leases(vec![
            exclusive_lease("lease_a", "environment:staging", &plan),
        ]),
        event("e1", AuditEventKind::ConstraintLeaseReleased, &plan).with_leases(vec![
            exclusive_lease("lease_b", "environment:staging", &plan),
        ]),
    ];
    assert!(matches!(verify_events(&events), Err(ReplayError::Lease(_))));
    Ok(())
}

#[test]
fn bundle_hash_mismatch_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = demo_bundle()?;
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    trace.bundle_hash =
        Some("sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned());
    assert!(matches!(
        trace.verify_with_bundle(&bundle),
        Err(ReplayError::BundleHashMismatch { .. })
    ));
    Ok(())
}

#[test]
fn declared_bundle_hash_match_is_accepted() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = demo_bundle()?;
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    trace.bundle_hash = Some(bundle.bundle_hash.0.clone());
    assert!(trace.verify_with_bundle(&bundle).is_ok());
    Ok(())
}

#[test]
fn verify_verdict_reports_acceptance_and_hashes() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = demo_bundle()?;
    let trace = ReplayTrace::from_json_str(TRACE)?;
    let verdict = trace.verify_verdict(&bundle);
    assert!(verdict.accepted);
    assert!(verdict.stable_error_code.is_none());
    assert_eq!(verdict.bundle_hash, bundle.bundle_hash.0);
    assert!(verdict.trace_hash.starts_with("sha256:"));
    assert!(verdict.checked_invariants.iter().any(|id| id == "I-001"));
    Ok(())
}

#[test]
fn verify_verdict_carries_stable_error_code_on_rejection() -> Result<(), Box<dyn std::error::Error>>
{
    let bundle = demo_bundle()?;
    let mut trace = ReplayTrace::from_json_str(TRACE)?;
    trace.bundle_hash =
        Some("sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned());
    let verdict = trace.verify_verdict(&bundle);
    assert!(!verdict.accepted);
    assert_eq!(
        verdict.stable_error_code.as_deref(),
        Some("BundleHashMismatch")
    );
    Ok(())
}

#[test]
fn event_after_closed_is_rejected_with_dedicated_code() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = demo_bundle()?;
    assert_scenario_fails(
        SCENARIO_EVENT_AFTER_CLOSED,
        |err| matches!(err, ReplayError::EventAfterClosed { .. }),
        &bundle,
    )?;
    Ok(())
}

mod attestation;
