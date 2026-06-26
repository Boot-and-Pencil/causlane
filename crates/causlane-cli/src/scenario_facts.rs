//! Conversion of a YAML replay scenario into target-neutral, **payload-bound**
//! [`AlloyScenarioFacts`] (P0-FM-003), shared by `formal verify-all`/`coverage`
//! and `formal generate`/`ir emit`.

use causlane_codegen::{
    AlloyAnchorFact, AlloyEventKind, AlloyLeaseFact, AlloyLeaseMode, AlloyScenarioEvent,
    AlloyScenarioFacts, FormalAuthzDecisionPayload, FormalBarrierPayload, FormalCapabilityPayload,
    FormalWitnessPayload,
};
use causlane_contracts::ClaimModeDto;
use causlane_replay::{
    AuthzDecisionDto, EventKindDto, ExpectedReplayResult, ReplayAuthzDecision, ReplayEvent,
    ReplayExecutionBarrier, ReplayExecutionCapability, ReplayScenario, ReplayWitnessRef,
    WitnessKindDto,
};

/// Parse a scenario YAML document and project it into [`AlloyScenarioFacts`].
///
/// # Errors
/// Returns [`causlane_replay::ReplayError`] when the YAML is invalid or its
/// scenario hash cannot be computed.
pub fn alloy_scenario_facts_from_yaml(
    yaml: &str,
) -> Result<AlloyScenarioFacts, causlane_replay::ReplayError> {
    let scenario = ReplayScenario::from_yaml_str(yaml)?;
    let scenario_hash = ReplayScenario::scenario_hash(yaml)?;
    Ok(facts_from_scenario(&scenario, scenario_hash))
}

/// Project an already-parsed scenario into [`AlloyScenarioFacts`].
#[must_use]
pub fn facts_from_scenario(scenario: &ReplayScenario, scenario_hash: String) -> AlloyScenarioFacts {
    let events = scenario
        .events
        .iter()
        .map(|event| alloy_event(event, &scenario.scenario_id))
        .collect();
    let binding_pairs = |bindings: &[causlane_replay::ScenarioBinding]| {
        bindings
            .iter()
            .map(|binding| (binding.key.clone(), binding.value.clone()))
            .collect::<Vec<_>>()
    };
    AlloyScenarioFacts {
        scenario_hash,
        action_id: scenario.action_id.clone(),
        plan_hash: scenario.plan_hash.clone(),
        expected_result: expected_result_token(scenario.expected_replay_result).to_owned(),
        expected_error_code: scenario.expected_error_code.clone(),
        formal_obligations: scenario.formal_obligations.clone(),
        predicate_id: scenario.predicate.clone(),
        subject: binding_pairs(&scenario.subject),
        circumstance: binding_pairs(&scenario.circumstance),
        events,
    }
}

fn alloy_event(event: &ReplayEvent, scenario_id: &str) -> AlloyScenarioEvent {
    let event_id = event
        .event_id
        .clone()
        .unwrap_or_else(|| format!("{scenario_id}:event"));
    AlloyScenarioEvent {
        event_id: event_id.clone(),
        kind: alloy_event_kind(event.kind),
        action_id: Some(event.action_id.clone()),
        plan_hash: event.plan_hash.clone(),
        op_index: event.execution_capability.as_ref().map(|cap| cap.op_index),
        fact_kind: event.fact_kind.clone(),
        scope: event.scope.clone(),
        anchors: event
            .anchors
            .iter()
            .map(|anchor| anchor.event_id.clone())
            .collect(),
        anchor_facts: event
            .anchors
            .iter()
            .map(|anchor| AlloyAnchorFact {
                event_id: anchor.event_id.clone(),
                fact_kind: anchor.fact_kind.clone(),
                scope: anchor.scope.clone(),
            })
            .collect(),
        leases: event
            .leases
            .iter()
            .map(|lease| AlloyLeaseFact {
                lease_id: lease.lease_id.clone(),
                resource: lease.resource.clone(),
                scope: lease.scope.clone(),
                mode: alloy_lease_mode(lease.mode),
                epoch: lease.epoch,
            })
            .collect(),
        barrier: event
            .execution_barrier
            .as_ref()
            .map(|barrier| barrier_payload(barrier, event, &event_id)),
        capability: event.execution_capability.as_ref().map(capability_payload),
        authz_decision: event
            .authz_decision
            .as_ref()
            .map(|authz| authz_payload(authz, event, &event_id)),
    }
}

fn barrier_payload(
    barrier: &ReplayExecutionBarrier,
    event: &ReplayEvent,
    event_id: &str,
) -> FormalBarrierPayload {
    FormalBarrierPayload {
        barrier_event_id: barrier
            .barrier_id
            .clone()
            .unwrap_or_else(|| event_id.to_owned()),
        action_id: barrier
            .action_id
            .clone()
            .unwrap_or_else(|| event.action_id.clone()),
        plan_hash: barrier
            .plan_hash
            .clone()
            .or_else(|| event.plan_hash.clone())
            .unwrap_or_default(),
        op_indexes: barrier.op_indexes.clone(),
        impact_set_hash: barrier.impact_set_hash.clone(),
        witnesses: barrier.witnesses.iter().map(witness_payload).collect(),
        lease_ids: barrier
            .leases
            .iter()
            .map(|lease| lease.lease_id.clone())
            .collect(),
        authz_decision_event_ids: barrier.authz_decision_refs.clone(),
    }
}

fn capability_payload(capability: &ReplayExecutionCapability) -> FormalCapabilityPayload {
    FormalCapabilityPayload {
        capability_id: capability.capability_id.clone(),
        barrier_event_id: capability.barrier_event_id.clone(),
        op_index: capability.op_index,
        lease_ids: capability.lease_ids.clone(),
    }
}

fn witness_payload(witness: &ReplayWitnessRef) -> FormalWitnessPayload {
    FormalWitnessPayload {
        witness_event_id: witness.event_id.clone(),
        requirement_id: witness.requirement_id.clone(),
        kind: witness_kind_token(witness.kind).to_owned(),
        fact_kind: witness.fact_kind.clone(),
        scope: witness.scope.clone(),
        binds_to_action_id: witness.binds_to.as_ref().map(|bind| bind.action_id.clone()),
        binds_to_plan_hash: witness.binds_to.as_ref().map(|bind| bind.plan_hash.clone()),
        binds_to_impact_set_hash: witness
            .binds_to
            .as_ref()
            .and_then(|bind| bind.impact_set_hash.clone()),
    }
}

fn authz_payload(
    authz: &ReplayAuthzDecision,
    event: &ReplayEvent,
    event_id: &str,
) -> FormalAuthzDecisionPayload {
    FormalAuthzDecisionPayload {
        decision_event_id: authz
            .decision_event_id
            .clone()
            .unwrap_or_else(|| event_id.to_owned()),
        action_id: authz
            .action_id
            .clone()
            .unwrap_or_else(|| event.action_id.clone()),
        plan_hash: authz
            .plan_hash
            .clone()
            .or_else(|| event.plan_hash.clone())
            .unwrap_or_default(),
        stage: authz.stage.clone(),
        decision: authz_decision_token(authz.decision).to_owned(),
    }
}

fn expected_result_token(result: ExpectedReplayResult) -> &'static str {
    match result {
        ExpectedReplayResult::Pass => "pass",
        ExpectedReplayResult::Fail => "fail",
    }
}

fn witness_kind_token(kind: WitnessKindDto) -> &'static str {
    match kind {
        WitnessKindDto::ObservedFact => "observed_fact",
        WitnessKindDto::GateApproval => "gate_approval",
        WitnessKindDto::AuthzDecision => "authz_decision",
        WitnessKindDto::ConstraintDecision => "constraint_decision",
        WitnessKindDto::ExternalEvidence => "external_evidence",
    }
}

fn authz_decision_token(decision: AuthzDecisionDto) -> &'static str {
    match decision {
        AuthzDecisionDto::Allow => "allow",
        AuthzDecisionDto::Deny => "deny",
    }
}

fn alloy_event_kind(kind: EventKindDto) -> AlloyEventKind {
    match kind {
        EventKindDto::ActionAdmitted => AlloyEventKind::ActionAdmitted,
        EventKindDto::ActionPlanned => AlloyEventKind::ActionPlanned,
        EventKindDto::DispatchLogged => AlloyEventKind::DispatchLogged,
        EventKindDto::ExecutionBarrierLogged => AlloyEventKind::ExecutionBarrierLogged,
        EventKindDto::ExecutionStarted => AlloyEventKind::ExecutionStarted,
        EventKindDto::ExecutionCompleted => AlloyEventKind::ExecutionCompleted,
        EventKindDto::ObservedTruthCommitted => AlloyEventKind::ObservedTruthCommitted,
        EventKindDto::ProjectionEmitted => AlloyEventKind::ProjectionEmitted,
        EventKindDto::LifecycleClosed => AlloyEventKind::LifecycleClosed,
        EventKindDto::GateApproved => AlloyEventKind::GateApproved,
        EventKindDto::GateDenied => AlloyEventKind::GateDenied,
        EventKindDto::ConstraintLeaseGranted => AlloyEventKind::ConstraintLeaseGranted,
        EventKindDto::ConstraintLeaseReleased => AlloyEventKind::ConstraintLeaseReleased,
        EventKindDto::ViolationDetected => AlloyEventKind::ViolationDetected,
        EventKindDto::AuthzDecisionRecorded => AlloyEventKind::AuthzDecisionRecorded,
        EventKindDto::DrainFenceRequested => AlloyEventKind::DrainFenceRequested,
        EventKindDto::DrainFenceAcquired => AlloyEventKind::DrainFenceAcquired,
    }
}

fn alloy_lease_mode(mode: ClaimModeDto) -> AlloyLeaseMode {
    match mode {
        ClaimModeDto::Exclusive => AlloyLeaseMode::Exclusive,
        ClaimModeDto::Shared => AlloyLeaseMode::Shared,
        ClaimModeDto::Token => AlloyLeaseMode::Token,
    }
}
