//! Lean4 proof applications generated from Formal IR v2.

use core::fmt::Write as _;

use crate::{
    artifact_header_with_prefix, CodegenError, FormalEvent, FormalIr, FormalPredicate,
    FormalTarget, GeneratedArtifact,
};

const ACTION_ADMITTED: &str = "action.admitted";
const ACTION_PLANNED: &str = "action.planned";
const DISPATCH_LOGGED: &str = "dispatch.logged";
const EXECUTION_BARRIER_LOGGED: &str = "execution.barrier_logged";
const EXECUTION_STARTED: &str = "execution.started";
const EXECUTION_COMPLETED: &str = "execution.completed";
const OBSERVED_TRUTH_COMMITTED: &str = "observed_truth.committed";
const PROJECTION_EMITTED: &str = "projection.emitted";
const LIFECYCLE_CLOSED: &str = "lifecycle.closed";
const GATE_APPROVED: &str = "gate.approved";
const GATE_DENIED: &str = "gate.denied";
const AUTHZ_DECISION_RECORDED: &str = "authz.decision_recorded";
const CONSTRAINT_LEASE_GRANTED: &str = "constraint.lease_granted";
const CONSTRAINT_LEASE_RELEASED: &str = "constraint.lease_released";
const OVERLAY_ATTACHED: &str = "overlay.attached";
const CONSTRAINT_UPDATED: &str = "constraint.updated";
const DRAIN_FENCE_REQUESTED: &str = "drain.fence_requested";
const DRAIN_FENCE_ACQUIRED: &str = "drain.fence_acquired";

const LEAN_KIND_NAMES: &[(&str, &str)] = &[
    (ACTION_ADMITTED, "actionAdmitted"),
    (ACTION_PLANNED, "actionPlanned"),
    (DISPATCH_LOGGED, "dispatchLogged"),
    (EXECUTION_BARRIER_LOGGED, "executionBarrierLogged"),
    (EXECUTION_STARTED, "executionStarted"),
    (EXECUTION_COMPLETED, "executionCompleted"),
    (OBSERVED_TRUTH_COMMITTED, "observedTruthCommitted"),
    (PROJECTION_EMITTED, "projectionEmitted"),
    (LIFECYCLE_CLOSED, "lifecycleClosed"),
    (GATE_APPROVED, "gateApproved"),
    (GATE_DENIED, "gateDenied"),
    (AUTHZ_DECISION_RECORDED, "authzDecisionRecorded"),
    (CONSTRAINT_LEASE_GRANTED, "constraintLeaseGranted"),
    (CONSTRAINT_LEASE_RELEASED, "constraintLeaseReleased"),
    (OVERLAY_ATTACHED, "overlayAttached"),
    (CONSTRAINT_UPDATED, "constraintUpdated"),
    (DRAIN_FENCE_REQUESTED, "drainFenceRequested"),
    (DRAIN_FENCE_ACQUIRED, "drainFenceAcquired"),
];

const LEAN_CONSEQUENCE_PROFILE_NAMES: &[(&str, &str)] = &[
    ("RuntimeExecution", "runtimeExecution"),
    ("ProjectionRead", "projectionRead"),
    ("OversightMeta", "oversightMeta"),
    ("TopologyMeta", "topologyMeta"),
    ("EvidenceMeta", "evidenceMeta"),
    ("OutsideKernel", "outsideKernel"),
];

const LEAN_LIFECYCLE_CLASS_NAMES: &[(&str, &str)] = &[
    ("execution_bearing", "executionBearing"),
    ("projection_only", "projectionOnly"),
    ("meta", "metaOnly"),
];

/// Generate Lean4 theorem applications from scenario-bound Formal IR.
///
/// # Errors
/// Returns [`CodegenError::Scenario`] when an event kind cannot be projected
/// into the Lean event universe.
#[must_use = "the generated Lean4 proof must be written or checked"]
pub fn generate_lean4_proof(ir: &FormalIr) -> Result<GeneratedArtifact, CodegenError> {
    let mut text = artifact_header_with_prefix(ir, FormalTarget::Lean4, "proof", "--");
    text.push_str("import CauslaneFormal.Core\n\n");
    text.push_str("namespace CauslaneFormal\n");
    let namespace = safe_namespace(&ir.bundle_id);
    let _written = writeln!(text, "namespace {namespace}\n");
    push_events(&mut text, ir)?;
    push_predicate_routes(&mut text, ir)?;
    push_theorems(&mut text, ir);
    let _written = writeln!(text, "end {namespace}");
    text.push_str("end CauslaneFormal\n");
    Ok(GeneratedArtifact::new(
        FormalTarget::Lean4,
        "proof",
        ir,
        text,
    ))
}

fn push_events(text: &mut String, ir: &FormalIr) -> Result<(), CodegenError> {
    for (index, event) in ir.scenario_events.iter().enumerate() {
        push_event_def(text, index, event)?;
    }
    text.push_str("def generatedTrace : List Event := [\n");
    for index in 0..ir.scenario_events.len() {
        let comma = if index + 1 == ir.scenario_events.len() {
            ""
        } else {
            ","
        };
        let _written = writeln!(text, "  evt_{index}{comma}");
    }
    text.push_str("]\n\n");
    Ok(())
}

fn push_predicate_routes(text: &mut String, ir: &FormalIr) -> Result<(), CodegenError> {
    text.push_str("def generatedPredicateRoutes : List PredicateRoute := [\n");
    for (index, predicate) in ir.predicates.iter().enumerate() {
        push_predicate_route(text, index, ir.predicates.len(), predicate)?;
    }
    text.push_str("]\n\n");
    Ok(())
}

fn push_predicate_route(
    text: &mut String,
    index: usize,
    len: usize,
    predicate: &FormalPredicate,
) -> Result<(), CodegenError> {
    let comma = if index + 1 == len { "" } else { "," };
    let _written = writeln!(text, "  {{");
    let _written = writeln!(
        text,
        "    predicateId := {},",
        lean_string(&predicate.predicate)
    );
    let _written = writeln!(text, "    routeId := {},", lean_string(&predicate.route_id));
    let _written = writeln!(
        text,
        "    consequenceProfile := ConsequenceProfile.{},",
        lean_consequence_profile(&predicate.consequence_profile)?
    );
    let _written = writeln!(
        text,
        "    lifecycleClass := LifecycleClass.{}",
        lean_lifecycle_class(&predicate.lifecycle_class)?
    );
    let _written = writeln!(text, "  }}{comma}");
    Ok(())
}

/// Emit one `def evt_N : Event := {…}` block. Local bindings collapse the anchor /
/// barrier / witness accessor chains so each field is a single `writeln!`; the
/// emitted bytes are identical to the inlined form.
fn push_event_def(
    text: &mut String,
    index: usize,
    event: &FormalEvent,
) -> Result<(), CodegenError> {
    let anchor = event.anchors.first();
    let barrier = event.barrier.as_ref();
    let witness = barrier.and_then(|barrier| barrier.witnesses.first());
    let _written = writeln!(text, "def evt_{index} : Event := {{");
    let _written = writeln!(text, "  eventId := {},", lean_string(&event.event_id));
    let _written = writeln!(text, "  kind := EventKind.{},", lean_kind(&event.kind)?);
    let _written = writeln!(
        text,
        "  actionId := {},",
        lean_option(event.action_id.as_deref())
    );
    let _written = writeln!(
        text,
        "  planHash := {},",
        lean_option(event.plan_hash.as_deref())
    );
    let _written = writeln!(text, "  opIndex := {},", lean_nat_option(event.op_index));
    let _written = writeln!(
        text,
        "  factKind := {},",
        lean_option(event.fact_kind.as_deref())
    );
    let _written = writeln!(text, "  scope := {},", lean_option(event.scope.as_deref()));
    let _written = writeln!(
        text,
        "  anchorEventId := {},",
        lean_option(anchor.map(|anchor| anchor.event_id.as_str()))
    );
    let _written = writeln!(
        text,
        "  anchorFactKind := {},",
        lean_option(anchor.and_then(|anchor| anchor.fact_kind.as_deref()))
    );
    let _written = writeln!(
        text,
        "  anchorScope := {},",
        lean_option(anchor.and_then(|anchor| anchor.scope.as_deref()))
    );
    let _written = writeln!(
        text,
        "  barrierEventId := {},",
        lean_option(barrier.map(|barrier| barrier.barrier_event_id.as_str()))
    );
    let _written = writeln!(
        text,
        "  barrierRef := {},",
        lean_option(
            event
                .capability
                .as_ref()
                .map(|capability| capability.barrier_event_id.as_str())
        )
    );
    let _written = writeln!(
        text,
        "  impactSetHash := {},",
        lean_option(barrier.map(|barrier| barrier.impact_set_hash.as_str()))
    );
    let _written = writeln!(
        text,
        "  witnessBindAction := {},",
        lean_option(witness.and_then(|witness| witness.binds_to_action_id.as_deref()))
    );
    let _written = writeln!(
        text,
        "  witnessBindPlan := {},",
        lean_option(witness.and_then(|witness| witness.binds_to_plan_hash.as_deref()))
    );
    let _written = writeln!(
        text,
        "  witnessBindImpact := {}",
        lean_option(witness.and_then(|witness| witness.binds_to_impact_set_hash.as_deref()))
    );
    text.push_str("}\n\n");
    Ok(())
}

fn push_theorems(text: &mut String, ir: &FormalIr) {
    push_theorem(
        text,
        "valid_trace_execution_started_has_prior_barrier",
        &execution_started_has_prior_barrier(ir),
    );
    push_theorem(
        text,
        "valid_trace_observed_truth_has_prior_execution",
        &observed_truth_has_prior_execution(ir),
    );
    push_theorem(
        text,
        "projection_anchor_soundness",
        &projection_anchor_soundness(ir),
    );
    push_theorem(text, "closed_is_terminal", &closed_is_terminal(ir));
    push_theorem(text, "witness_exact_binding", &witness_exact_binding(ir));
    push_theorem(
        text,
        "overlay_monotonicity",
        "overlayMonotonicityHolds = true",
    );
    push_theorem(
        text,
        "route_profile_compatibility",
        "allRoutesConsistentWithProfiles generatedPredicateRoutes = true",
    );
    push_theorem(
        text,
        "lease_conflict_fail_closed",
        "leaseConflictFailClosedHolds = true",
    );
    push_theorem(
        text,
        "verified_merge_algebra",
        "verifiedMergeClearsConflictHolds = true",
    );
    push_theorem(
        text,
        "drain_after_overlap_clear",
        "drainAfterOverlapClearHolds = true",
    );
    push_theorem(
        text,
        "constraint_update_future_only",
        "constraintUpdateFutureOnlyHolds = true",
    );
}

fn push_theorem(text: &mut String, name: &str, proposition: &str) {
    let _written = writeln!(text, "theorem {name} :");
    let _written = writeln!(text, "  {proposition} := by");
    text.push_str("  native_decide\n\n");
}

fn execution_started_has_prior_barrier(ir: &FormalIr) -> String {
    let Some(exec_idx) = first_kind(&ir.scenario_events, EXECUTION_STARTED) else {
        return "True".to_owned();
    };
    let exec = &ir.scenario_events[exec_idx];
    let barrier_idx = exec
        .capability
        .as_ref()
        .and_then(|capability| prior_event_by_id(ir, exec_idx, &capability.barrier_event_id))
        .or_else(|| prior_kind(&ir.scenario_events, exec_idx, EXECUTION_BARRIER_LOGGED));
    let Some(barrier_idx) = barrier_idx else {
        return "False".to_owned();
    };
    let barrier_ref = exec
        .capability
        .as_ref()
        .map(|capability| capability.barrier_event_id.as_str())
        .or_else(|| {
            ir.scenario_events[barrier_idx]
                .barrier
                .as_ref()
                .map(|barrier| barrier.barrier_event_id.as_str())
        });
    let mut clauses = vec![
        event_kind_clause(barrier_idx, "executionBarrierLogged"),
        event_kind_clause(exec_idx, "executionStarted"),
        format!("{barrier_idx} < {exec_idx}"),
        format!(
            "eventActionAt generatedTrace {barrier_idx} = eventActionAt generatedTrace {exec_idx}"
        ),
        format!("eventPlanAt generatedTrace {barrier_idx} = eventPlanAt generatedTrace {exec_idx}"),
    ];
    if let Some(barrier_ref) = barrier_ref {
        clauses.push(format!(
            "eventBarrierRefAt generatedTrace {exec_idx} = some {}",
            lean_string(barrier_ref)
        ));
    }
    join_conjunction(&clauses)
}

fn observed_truth_has_prior_execution(ir: &FormalIr) -> String {
    let Some(observed_idx) = first_kind(&ir.scenario_events, OBSERVED_TRUTH_COMMITTED) else {
        return "True".to_owned();
    };
    let Some(exec_idx) = prior_kind(&ir.scenario_events, observed_idx, EXECUTION_STARTED) else {
        return "False".to_owned();
    };
    join_conjunction(&[
        event_kind_clause(exec_idx, "executionStarted"),
        event_kind_clause(observed_idx, "observedTruthCommitted"),
        format!("{exec_idx} < {observed_idx}"),
        format!(
            "eventActionAt generatedTrace {exec_idx} = eventActionAt generatedTrace {observed_idx}"
        ),
        format!(
            "eventPlanAt generatedTrace {exec_idx} = eventPlanAt generatedTrace {observed_idx}"
        ),
    ])
}

fn projection_anchor_soundness(ir: &FormalIr) -> String {
    let Some(projection_idx) = first_kind(&ir.scenario_events, PROJECTION_EMITTED) else {
        return "True".to_owned();
    };
    let projection = &ir.scenario_events[projection_idx];
    let anchor_id = projection
        .anchors
        .first()
        .map(|anchor| anchor.event_id.as_str());
    let anchor_idx = anchor_id
        .and_then(|event_id| prior_event_by_id(ir, projection_idx, event_id))
        .or_else(|| {
            prior_kind(
                &ir.scenario_events,
                projection_idx,
                OBSERVED_TRUTH_COMMITTED,
            )
        });
    let Some(anchor_idx) = anchor_idx else {
        return "False".to_owned();
    };
    let mut clauses = vec![
        event_kind_clause(anchor_idx, "observedTruthCommitted"),
        event_kind_clause(projection_idx, "projectionEmitted"),
        format!("{anchor_idx} < {projection_idx}"),
        format!(
            "eventFactAt generatedTrace {anchor_idx} = eventAnchorFactAt generatedTrace {projection_idx}"
        ),
        format!(
            "eventScopeAt generatedTrace {anchor_idx} = eventAnchorScopeAt generatedTrace {projection_idx}"
        ),
    ];
    if let Some(anchor_id) = anchor_id {
        clauses.push(format!(
            "eventAnchorEventIdAt generatedTrace {projection_idx} = some {}",
            lean_string(anchor_id)
        ));
    }
    join_conjunction(&clauses)
}

fn closed_is_terminal(ir: &FormalIr) -> String {
    let Some(closed_idx) = first_kind(&ir.scenario_events, LIFECYCLE_CLOSED) else {
        return "True".to_owned();
    };
    join_conjunction(&[
        event_kind_clause(closed_idx, "lifecycleClosed"),
        format!("noLifecycleMutationsAfter generatedTrace {closed_idx} = true"),
    ])
}

fn witness_exact_binding(ir: &FormalIr) -> String {
    let Some(barrier_idx) = ir.scenario_events.iter().position(|event| {
        event
            .barrier
            .as_ref()
            .is_some_and(|barrier| !barrier.witnesses.is_empty())
    }) else {
        return "True".to_owned();
    };
    join_conjunction(&[
        event_kind_clause(barrier_idx, "executionBarrierLogged"),
        format!(
            "eventWitnessBindActionAt generatedTrace {barrier_idx} = eventActionAt generatedTrace {barrier_idx}"
        ),
        format!(
            "eventWitnessBindPlanAt generatedTrace {barrier_idx} = eventPlanAt generatedTrace {barrier_idx}"
        ),
        format!(
            "eventWitnessBindImpactAt generatedTrace {barrier_idx} = eventImpactAt generatedTrace {barrier_idx}"
        ),
    ])
}

fn event_kind_clause(index: usize, kind: &str) -> String {
    format!("eventKindAt generatedTrace {index} = some EventKind.{kind}")
}

fn first_kind(events: &[FormalEvent], kind: &str) -> Option<usize> {
    events.iter().position(|event| event.kind == kind)
}

fn prior_kind(events: &[FormalEvent], before: usize, kind: &str) -> Option<usize> {
    events
        .iter()
        .take(before)
        .rposition(|event| event.kind == kind)
}

fn prior_event_by_id(ir: &FormalIr, before: usize, event_id: &str) -> Option<usize> {
    ir.scenario_events
        .iter()
        .take(before)
        .rposition(|event| event.event_id == event_id)
}

fn join_conjunction(clauses: &[String]) -> String {
    if clauses.is_empty() {
        return "True".to_owned();
    }
    clauses.join(" /\\\n  ")
}

fn safe_namespace(raw: &str) -> String {
    let mut name = String::from("Generated_");
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            name.push(ch);
        } else {
            name.push('_');
        }
    }
    name
}

fn lean_kind(kind: &str) -> Result<&'static str, CodegenError> {
    lean_name(LEAN_KIND_NAMES, kind, "event kind")
}

fn lean_consequence_profile(token: &str) -> Result<&'static str, CodegenError> {
    lean_name(
        LEAN_CONSEQUENCE_PROFILE_NAMES,
        token,
        "consequence_profile token",
    )
}

fn lean_lifecycle_class(token: &str) -> Result<&'static str, CodegenError> {
    lean_name(LEAN_LIFECYCLE_CLASS_NAMES, token, "lifecycle_class token")
}

fn lean_name(
    table: &'static [(&'static str, &'static str)],
    token: &str,
    label: &str,
) -> Result<&'static str, CodegenError> {
    table
        .iter()
        .find_map(|(source, lean_name)| (*source == token).then_some(*lean_name))
        .ok_or_else(|| CodegenError::Scenario(format!("unsupported Lean4 {label} {token}")))
}

fn lean_nat_option(value: Option<u32>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| format!("some {value}"))
}

fn lean_option(value: Option<&str>) -> String {
    value.map_or_else(
        || "none".to_owned(),
        |value| format!("some {}", lean_string(value)),
    )
}

fn lean_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}
