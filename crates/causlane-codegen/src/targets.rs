//! Formal target generators that consume Formal IR v2.

use core::fmt::Write as _;

use crate::{artifact_header, CodegenError, FormalIr, FormalTarget, GeneratedArtifact};

const P_EVENT_NAMES: [(&str, &str); 18] = [
    ("action.admitted", "ActionAdmitted"),
    ("action.planned", "ActionPlanned"),
    ("dispatch.logged", "DispatchLogged"),
    ("execution.barrier_logged", "ExecutionBarrierLogged"),
    ("execution.started", "ExecutionStarted"),
    ("execution.completed", "ExecutionCompleted"),
    ("observed_truth.committed", "ObservedTruthCommitted"),
    ("projection.emitted", "ProjectionEmitted"),
    ("lifecycle.closed", "LifecycleClosed"),
    ("gate.approved", "GateApproved"),
    ("gate.denied", "GateDenied"),
    ("authz.decision_recorded", "AuthzDecisionRecorded"),
    ("constraint.lease_granted", "ConstraintLeaseGranted"),
    ("constraint.lease_released", "ConstraintLeaseReleased"),
    ("overlay.attached", "OverlayAttached"),
    ("constraint.updated", "ConstraintUpdated"),
    ("drain.fence_requested", "DrainFenceRequested"),
    ("drain.fence_acquired", "DrainFenceAcquired"),
];

// The monitors asserted by the generated test. Held as a list (rather than inline
// in the test string) so the test declaration can be emitted with the dynamic,
// per-scenario action-driver machine set (P1-001 part 3).
const P_MONITOR_NAMES: [&str; 16] = [
    "NoExecutionBeforeBarrier",
    "NoObservedWithoutExecution",
    "NoProjectionWithoutAnchor",
    "NoConflictingActiveLeases",
    "DrainBlocksNewMutableAdmission",
    "NoDuplicateHardExecutionForSameIdempotencyKey",
    "AuthzRevocationBeforeBarrierBlocksExecution",
    "NoStaleConstraintEpochAdmission",
    "NoEventsAfterClosed",
    "ApprovalBindingDoesNotDrift",
    "ConstraintUpdateDoesNotRewriteTruth",
    "ReplayAcceptsOnlyValidTrace",
    "CapabilityBindsToBarrier",
    "WitnessFactGrounded",
    "AnchorFactGrounded",
    "AuthzDecisionGroundsBarrier",
];

/// Generate a P protocol model from Formal IR.
///
/// # Errors
/// Returns [`CodegenError::Scenario`] when an event kind cannot be projected
/// into the generated P event universe.
#[must_use = "the generated P model must be written or checked"]
pub fn generate_p_monitor(ir: &FormalIr) -> Result<GeneratedArtifact, CodegenError> {
    let mut text = artifact_header(ir, FormalTarget::P, "protocol");
    push_p_events(&mut text);
    push_p_machines(&mut text, ir)?;
    crate::p_monitors::push_p_monitors(&mut text);
    push_p_test(&mut text, ir);
    push_contract_summary(&mut text, ir);
    Ok(GeneratedArtifact::new(
        FormalTarget::P,
        "protocol",
        ir,
        text,
    ))
}

/// Generate Verus abstract preservation proofs from Formal IR.
///
/// # Errors
/// Currently infallible; returns [`CodegenError`] for future target validation.
#[must_use = "the generated Verus proof must be written or checked"]
pub fn generate_verus_proof(ir: &FormalIr) -> Result<GeneratedArtifact, CodegenError> {
    let mut text = artifact_header(ir, FormalTarget::Verus, "proof");
    text.push_str("use vstd::prelude::*;\n\n");
    text.push_str("verus! {\n");
    crate::verus_target::push_verus_kernel(&mut text);
    crate::verus_target::push_verus_theorems(&mut text);
    crate::verus_target::push_verus_scenario_trace(&mut text, ir);
    text.push_str("}\n\n");
    text.push_str("fn main() {}\n\n");
    push_rust_contract_summary(&mut text, ir);
    Ok(GeneratedArtifact::new(
        FormalTarget::Verus,
        "proof",
        ir,
        text,
    ))
}

fn push_p_events(text: &mut String) {
    // Typed payload (P0-FM-005): events carry the action/plan/op/barrier facts
    // from the payload-bound IR, so monitors can discriminate per action/plan,
    // not just observe a kind sequence.
    text.push_str(
        "type EventPayload = (eventId: string, actionId: string, planHash: string, barrierId: string, opIndex: int, impactSetHash: string, factKind: string, factScope: string, claimEventId: string, claimFactKind: string, claimScope: string, witnessBindAction: string, witnessBindPlan: string, witnessBindImpact: string, authzDecision: string, authzRefEventId: string, authzStage: string, leaseResource: string, leaseScope: string, leaseMode: string, leaseEpoch: int, executionKey: string);\n",
    );
    for (_kind, event) in P_EVENT_NAMES {
        let _written = writeln!(text, "event {event}: EventPayload;");
    }
    text.push('\n');
}

// The fact/scope this event CLAIMS about another event (Formal IR v2): a barrier
// claims its first witness's attestation, a projection claims its first anchor's
// attestation. `WitnessFactGrounded` / `AnchorFactGrounded` refute a claim the
// referenced event never attested (P0-004). Empty triple when nothing is claimed.
fn p_resolve_claim(event: &crate::FormalEvent) -> (String, String, String) {
    event
        .barrier
        .as_ref()
        .and_then(|barrier| {
            barrier.witnesses.iter().find_map(|witness| {
                if witness.fact_kind.is_some() || witness.scope.is_some() {
                    Some((
                        witness.witness_event_id.clone(),
                        witness.fact_kind.clone().unwrap_or_default(),
                        witness.scope.clone().unwrap_or_default(),
                    ))
                } else {
                    None
                }
            })
        })
        .or_else(|| {
            event.anchors.iter().find_map(|anchor| {
                if anchor.fact_kind.is_some() || anchor.scope.is_some() {
                    Some((
                        anchor.event_id.clone(),
                        anchor.fact_kind.clone().unwrap_or_default(),
                        anchor.scope.clone().unwrap_or_default(),
                    ))
                } else {
                    None
                }
            })
        })
        .unwrap_or_default()
}

fn p_event_payload(event: &crate::FormalEvent, lease: Option<&crate::FormalLeaseFact>) -> String {
    let action = event.action_id.clone().unwrap_or_default();
    let plan = event.plan_hash.clone().unwrap_or_default();
    let (barrier_id, op_index, impact_set_hash) = if let Some(barrier) = &event.barrier {
        (
            barrier.barrier_event_id.clone(),
            barrier.op_indexes.first().copied().unwrap_or(0),
            barrier.impact_set_hash.clone(),
        )
    } else if let Some(capability) = &event.capability {
        (
            capability.barrier_event_id.clone(),
            capability.op_index,
            String::new(),
        )
    } else {
        (String::new(), 0, String::new())
    };
    // Producer attestation (Formal IR v2): the fact this event records about
    // itself, and the fact/scope it CLAIMS about another event — a barrier
    // claims its first witness's attestation, a projection claims its first
    // anchor's attestation. `WitnessFactGrounded` / `AnchorFactGrounded` refute
    // a claim the referenced event never attested (P0-004).
    let fact_kind = event.fact_kind.clone().unwrap_or_default();
    let fact_scope = event.scope.clone().unwrap_or_default();
    let (claim_event_id, claim_fact_kind, claim_scope) = p_resolve_claim(event);
    let (witness_bind_action, witness_bind_plan, witness_bind_impact) = event
        .barrier
        .as_ref()
        .and_then(|barrier| {
            barrier.witnesses.iter().find_map(|witness| {
                if witness.binds_to_action_id.is_some()
                    || witness.binds_to_plan_hash.is_some()
                    || witness.binds_to_impact_set_hash.is_some()
                {
                    Some((
                        witness.binds_to_action_id.clone().unwrap_or_default(),
                        witness.binds_to_plan_hash.clone().unwrap_or_default(),
                        witness.binds_to_impact_set_hash.clone().unwrap_or_default(),
                    ))
                } else {
                    None
                }
            })
        })
        .unwrap_or_default();
    // Authz (P0-010): the verdict this event records (on an authz decision), and
    // the first authz decision a barrier references. `AuthzDecisionGroundsBarrier`
    // refutes a barrier whose referenced decision is not an Allow.
    let authz_decision = event
        .authz_decision
        .as_ref()
        .map(|authz| authz.decision.clone())
        .unwrap_or_default();
    let authz_stage = event.authz_decision.as_ref().map_or_else(
        || {
            if event.kind == "execution.barrier_logged" {
                "execution_barrier_logged".to_owned()
            } else {
                String::new()
            }
        },
        |authz| authz.stage.clone(),
    );
    let authz_ref_event_id = event
        .barrier
        .as_ref()
        .and_then(|barrier| barrier.authz_decision_event_ids.first().cloned())
        .unwrap_or_default();
    // Lease facts (P1-001 part 2): the per-lease resource/scope/mode this send
    // represents. A lease-grant/release event is expanded into one send per lease
    // (see `push_p_machines`), so the lease monitors key by scope — a drain or an
    // exclusive lease on one scope no longer affects another scope under
    // interleaving. Empty for non-lease events.
    let (lease_resource, lease_scope, lease_mode, lease_epoch) = lease.map_or_else(
        || (String::new(), String::new(), String::new(), 0),
        |fact| {
            (
                fact.resource.clone(),
                fact.scope.clone(),
                fact.mode.clone(),
                fact.epoch,
            )
        },
    );
    let execution_key = if event.kind == "execution.started" && !plan.is_empty() {
        format!("exec:{plan}:{op_index}")
    } else {
        String::new()
    };
    format!(
        "(eventId = \"{}\", actionId = \"{}\", planHash = \"{}\", barrierId = \"{}\", opIndex = {op_index}, impactSetHash = \"{}\", factKind = \"{}\", factScope = \"{}\", claimEventId = \"{}\", claimFactKind = \"{}\", claimScope = \"{}\", witnessBindAction = \"{}\", witnessBindPlan = \"{}\", witnessBindImpact = \"{}\", authzDecision = \"{}\", authzRefEventId = \"{}\", authzStage = \"{}\", leaseResource = \"{}\", leaseScope = \"{}\", leaseMode = \"{}\", leaseEpoch = {lease_epoch}, executionKey = \"{}\")",
        p_string(&event.event_id),
        p_string(&action),
        p_string(&plan),
        p_string(&barrier_id),
        p_string(&impact_set_hash),
        p_string(&fact_kind),
        p_string(&fact_scope),
        p_string(&claim_event_id),
        p_string(&claim_fact_kind),
        p_string(&claim_scope),
        p_string(&witness_bind_action),
        p_string(&witness_bind_plan),
        p_string(&witness_bind_impact),
        p_string(&authz_decision),
        p_string(&authz_ref_event_id),
        p_string(&authz_stage),
        p_string(&lease_resource),
        p_string(&lease_scope),
        p_string(&lease_mode),
        p_string(&execution_key),
    )
}

// A lease-grant/release event carries one fact per held lease; the lease monitors
// observe these per scope, so such events are expanded into one P send per lease.
fn is_constraint_lease_event(kind: &str) -> bool {
    kind == "constraint.lease_granted" || kind == "constraint.lease_released"
}

// P1-001 part 3 (interleaving lane): emit one driver machine per distinct action,
// all created concurrently by a single bootstrap machine, so P's scheduler
// interleaves the actions' event streams. The keyed monitors (parts 1/2) then
// verify that a close/barrier/lease for one action or scope cannot satisfy or
// block another under interleaving. A single-action scenario produces exactly one
// driver and no cross-action interleaving, so its behaviour is preserved. This
// replaces the previous single sequential `ScenarioDriver` plus the empty role
// stub machines, which could not interleave anything.
fn push_p_machines(text: &mut String, ir: &FormalIr) -> Result<(), CodegenError> {
    let actions = distinct_actions(ir);
    for action in &actions {
        let machine = p_driver_name(action);
        let _written = writeln!(text, "machine {machine} {{");
        text.push_str("  start state Init {\n");
        text.push_str("    entry {\n");
        for event in ir
            .scenario_events
            .iter()
            .filter(|event| event.action_id.clone().unwrap_or_default() == *action)
        {
            let event_name = p_event_name(&event.kind)?;
            if is_constraint_lease_event(&event.kind) && !event.leases.is_empty() {
                // One send per lease so the lease monitors observe each lease's scope.
                for lease in &event.leases {
                    let _written = writeln!(
                        text,
                        "      send this, {event_name}, {};",
                        p_event_payload(event, Some(lease))
                    );
                }
            } else {
                let _written = writeln!(
                    text,
                    "      send this, {event_name}, {};",
                    p_event_payload(event, None)
                );
            }
        }
        text.push_str("    }\n");
        for (_kind, event) in P_EVENT_NAMES {
            let _written = writeln!(text, "    on {event} do {{ }}");
        }
        text.push_str("  }\n");
        text.push_str("}\n\n");
    }

    // Bootstrap (the test's `main`): create every action driver so the scheduler
    // runs them concurrently and interleaves their sends.
    text.push_str("machine ScenarioBootstrap {\n");
    text.push_str("  start state Init {\n");
    text.push_str("    entry {\n");
    for action in &actions {
        let _written = writeln!(text, "      new {}();", p_driver_name(action));
    }
    text.push_str("    }\n");
    text.push_str("  }\n");
    text.push_str("}\n\n");
    Ok(())
}

// Distinct action ids across the scenario events, in first-seen order. Events with
// no explicit action id share one default-named driver so no event is dropped.
fn distinct_actions(ir: &FormalIr) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut order = Vec::new();
    for event in &ir.scenario_events {
        let action = event.action_id.clone().unwrap_or_default();
        if seen.insert(action.clone()) {
            order.push(action);
        }
    }
    order
}

fn p_driver_name(action: &str) -> String {
    format!("ActionDriver_{}", crate::alloy::alloy_ident(action))
}

// The test entrypoint. Its NAME is stable (`release_promote_generated`) — the gate
// selects it with `--testcase release_promote_generated` for every scenario — but
// `main` is now the bootstrap that spawns the per-action drivers, and the module
// set lists those drivers.
fn push_p_test(text: &mut String, ir: &FormalIr) {
    let mut machines = vec!["ScenarioBootstrap".to_owned()];
    machines.extend(
        distinct_actions(ir)
            .iter()
            .map(|action| p_driver_name(action)),
    );
    let _written = writeln!(
        text,
        "test release_promote_generated [main=ScenarioBootstrap]: assert {} in {{ {} }};",
        P_MONITOR_NAMES.join(", "),
        machines.join(", ")
    );
}

#[allow(clippy::needless_raw_string_hashes)]
fn push_contract_summary(text: &mut String, ir: &FormalIr) {
    text.push_str("\n// Bundle-bound predicate summary.\n");
    for predicate in &ir.predicates {
        let _written = writeln!(
            text,
            "// predicate={} route={} barrier={} projection={} authz_required={}",
            predicate.predicate,
            predicate.route_id,
            predicate.barrier_policy,
            predicate.projection_policy,
            predicate.authz_required
        );
    }
    let _written = writeln!(text, "// invariants={}", ir.invariants.join(","));
}

pub(crate) fn push_rust_contract_summary(text: &mut String, ir: &FormalIr) {
    text.push_str("// Bundle-bound predicate summary.\n");
    for predicate in &ir.predicates {
        let _written = writeln!(
            text,
            "// predicate={} route={} barrier={} projection={} authz_required={}",
            predicate.predicate,
            predicate.route_id,
            predicate.barrier_policy,
            predicate.projection_policy,
            predicate.authz_required
        );
    }
    let _written = writeln!(text, "// invariants={}", ir.invariants.join(","));
}

fn p_event_name(kind: &str) -> Result<&'static str, CodegenError> {
    P_EVENT_NAMES
        .iter()
        .find_map(|(known, event)| (*known == kind).then_some(*event))
        .ok_or_else(|| {
            CodegenError::Scenario(format!("P generator does not know event kind {kind}"))
        })
}

fn p_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use crate::{
        generate_p_monitor, CodegenError, FormalAuthzDecisionPayload, FormalBarrierPayload,
        FormalCapabilityPayload, FormalEvent, FormalIr, FormalLeaseFact, FormalWitnessPayload,
        GENERATOR_VERSION,
    };
    use causlane_contracts::CANONICAL_SERIALIZATION_VERSION;

    #[test]
    fn p_witness_grounding_binds_scope_and_exact_barrier_binding() -> Result<(), CodegenError> {
        let plan_hash = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let impact_hash = "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
        let ir = FormalIr {
            schema_version: 2,
            formal_ir_hash:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            generator_version: GENERATOR_VERSION.to_owned(),
            canonical_serialization_version: CANONICAL_SERIALIZATION_VERSION,
            source_bundle_hash:
                "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_owned(),
            bundle_schema_version: 3,
            bundle_id: "test".to_owned(),
            bundle_version: "0".to_owned(),
            scenario_hash: None,
            expected_result: None,
            expected_error_code: None,
            predicates: Vec::new(),
            merge_protocols: Vec::new(),
            scenario_events: vec![
                FormalEvent {
                    event_id: "evt_readiness_ok".to_owned(),
                    kind: "gate.approved".to_owned(),
                    action_id: Some("act".to_owned()),
                    plan_hash: Some(plan_hash.to_owned()),
                    op_index: None,
                    fact_kind: Some("readiness_ok".to_owned()),
                    scope: Some("release_candidate:rc_123".to_owned()),
                    anchors: Vec::new(),
                    leases: Vec::new(),
                    barrier: None,
                    capability: None,
                    authz_decision: None,
                },
                FormalEvent {
                    event_id: "evt_barrier".to_owned(),
                    kind: "execution.barrier_logged".to_owned(),
                    action_id: Some("act".to_owned()),
                    plan_hash: Some(plan_hash.to_owned()),
                    op_index: None,
                    fact_kind: None,
                    scope: None,
                    anchors: Vec::new(),
                    leases: Vec::new(),
                    barrier: Some(FormalBarrierPayload {
                        barrier_event_id: "evt_barrier".to_owned(),
                        action_id: "act".to_owned(),
                        plan_hash: plan_hash.to_owned(),
                        op_indexes: vec![0],
                        impact_set_hash: impact_hash.to_owned(),
                        witnesses: vec![FormalWitnessPayload {
                            witness_event_id: "evt_readiness_ok".to_owned(),
                            requirement_id: "readiness_before_promotion".to_owned(),
                            kind: "gate_approval".to_owned(),
                            fact_kind: Some("readiness_ok".to_owned()),
                            scope: Some("release_candidate:other".to_owned()),
                            binds_to_action_id: Some("act".to_owned()),
                            binds_to_plan_hash: Some(plan_hash.to_owned()),
                            binds_to_impact_set_hash: Some(impact_hash.to_owned()),
                        }],
                        lease_ids: Vec::new(),
                        authz_decision_event_ids: Vec::new(),
                    }),
                    capability: None,
                    authz_decision: None,
                },
            ],
            invariants: vec!["I-009".to_owned()],
        };

        let generated = generate_p_monitor(&ir)?.text;
        assert!(generated.contains("factScope = \"release_candidate:rc_123\""));
        assert!(generated.contains("claimScope = \"release_candidate:other\""));
        assert!(generated.contains("attestedScope[p.claimEventId] == p.claimScope"));
        assert!(generated.contains("p.witnessBindPlan == p.planHash"));
        assert!(generated.contains("p.witnessBindImpact == p.impactSetHash"));
        Ok(())
    }

    fn empty_event(id: &str, kind: &str, action: &str, plan_hash: Option<&str>) -> FormalEvent {
        FormalEvent {
            event_id: id.to_owned(),
            kind: kind.to_owned(),
            action_id: Some(action.to_owned()),
            plan_hash: plan_hash.map(str::to_owned),
            op_index: None,
            fact_kind: None,
            scope: None,
            anchors: Vec::new(),
            leases: Vec::new(),
            barrier: None,
            capability: None,
            authz_decision: None,
        }
    }

    fn lifecycle_event(id: &str, kind: &str, action: &str) -> FormalEvent {
        empty_event(id, kind, action, None)
    }

    fn ir_from_events(scenario_events: Vec<FormalEvent>) -> FormalIr {
        FormalIr {
            schema_version: 2,
            formal_ir_hash:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            generator_version: GENERATOR_VERSION.to_owned(),
            canonical_serialization_version: CANONICAL_SERIALIZATION_VERSION,
            source_bundle_hash:
                "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_owned(),
            bundle_schema_version: 3,
            bundle_id: "test".to_owned(),
            bundle_version: "0".to_owned(),
            scenario_hash: None,
            expected_result: None,
            expected_error_code: None,
            predicates: Vec::new(),
            merge_protocols: Vec::new(),
            scenario_events,
            invariants: vec!["I-008".to_owned()],
        }
    }

    // P1-001 part 3: the generated P model has one driver machine per distinct
    // action, all spawned by a ScenarioBootstrap main, so P interleaves the
    // actions. A single-action scenario produces exactly one driver (no regression).
    #[test]
    fn action_sharded_drivers_one_per_action() -> Result<(), CodegenError> {
        let two = ir_from_events(vec![
            lifecycle_event("a_admit", "action.admitted", "act_a"),
            lifecycle_event("b_admit", "action.admitted", "act_b"),
            lifecycle_event("a_closed", "lifecycle.closed", "act_a"),
            lifecycle_event("b_closed", "lifecycle.closed", "act_b"),
        ]);
        let generated = generate_p_monitor(&two)?.text;
        assert!(generated.contains("machine ActionDriver_act_a {"));
        assert!(generated.contains("machine ActionDriver_act_b {"));
        assert!(generated.contains("machine ScenarioBootstrap {"));
        assert!(generated.contains("new ActionDriver_act_a();"));
        assert!(generated.contains("new ActionDriver_act_b();"));
        assert!(generated.contains("[main=ScenarioBootstrap]"));
        assert!(
            generated.contains("in { ScenarioBootstrap, ActionDriver_act_a, ActionDriver_act_b }")
        );
        // The legacy single driver and the empty role stub machines are gone.
        assert!(!generated.contains("machine ScenarioDriver"));
        assert!(!generated.contains("machine Dispatcher"));

        let one = ir_from_events(vec![
            lifecycle_event("solo_admit", "action.admitted", "act_solo"),
            lifecycle_event("solo_closed", "lifecycle.closed", "act_solo"),
        ]);
        let single = generate_p_monitor(&one)?.text;
        assert_eq!(single.matches("machine ActionDriver_").count(), 1);
        Ok(())
    }

    #[test]
    fn p_payload_carries_planned_interleaving_fields() -> Result<(), CodegenError> {
        let plan_hash = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let mut lease_event = empty_event(
            "evt_lease_epoch_2",
            "constraint.lease_granted",
            "act",
            Some(plan_hash),
        );
        lease_event.leases.push(FormalLeaseFact {
            lease_id: "lease_env".to_owned(),
            resource: "environment_write".to_owned(),
            scope: "environment:staging".to_owned(),
            mode: "exclusive".to_owned(),
            epoch: 2,
        });

        let mut authz_event = empty_event(
            "evt_authz_deny",
            "authz.decision_recorded",
            "act",
            Some(plan_hash),
        );
        authz_event.authz_decision = Some(FormalAuthzDecisionPayload {
            decision_event_id: "evt_authz_deny".to_owned(),
            action_id: "act".to_owned(),
            plan_hash: plan_hash.to_owned(),
            stage: "execution_barrier_logged".to_owned(),
            decision: "deny".to_owned(),
        });

        let mut execution_event =
            empty_event("evt_execution", "execution.started", "act", Some(plan_hash));
        execution_event.capability = Some(FormalCapabilityPayload {
            capability_id: "cap:evt_barrier:0".to_owned(),
            barrier_event_id: "evt_barrier".to_owned(),
            op_index: 0,
            lease_ids: vec!["lease_env".to_owned()],
        });

        let ir = ir_from_events(vec![lease_event, authz_event, execution_event]);
        let generated = generate_p_monitor(&ir)?.text;
        assert!(generated.contains("leaseEpoch = 2"));
        assert!(generated.contains("authzDecision = \"deny\""));
        assert!(generated.contains("authzStage = \"execution_barrier_logged\""));
        assert!(generated.contains(&format!("executionKey = \"exec:{plan_hash}:0\"")));
        assert!(generated.contains("spec NoStaleConstraintEpochAdmission"));
        assert!(generated.contains(
            "spec AuthzRevocationBeforeBarrierBlocksExecution observes AuthzDecisionRecorded"
        ));
        assert!(generated.contains("p.executionKey in executed"));
        Ok(())
    }
}
