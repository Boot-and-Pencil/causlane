//! Alloy fact generation from compiled bundles and replay scenarios.

use core::fmt::Write as _;
use std::collections::{BTreeMap, BTreeSet};

use causlane_contracts::{
    BarrierPolicyDto, CompiledDispatchBundle, ConsequenceProfileDto, LifecycleClassDto,
    ProjectionPolicyDto,
};

use crate::alloy_bindings::{ApprovalFacts, CapabilityFacts};
use crate::{artifact_header, build_formal_ir, CodegenError, FormalTarget, GeneratedArtifact};

/// Scenario projection consumed by the Alloy facts generator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlloyScenarioFacts {
    /// Scenario content hash (`sha256:...`).
    pub scenario_hash: String,
    /// Scenario action id.
    pub action_id: String,
    /// Scenario plan hash.
    pub plan_hash: String,
    /// Expected replay result (`pass` or `fail`).
    pub expected_result: String,
    /// Expected stable error code for negative scenarios.
    pub expected_error_code: Option<String>,
    /// Invariant ids exercised by the scenario.
    pub formal_obligations: Vec<String>,
    /// Predicate id, used to resolve the bundle's verified-merge protocols into
    /// the scope-keyed `mergeable` relation (P0-005).
    pub predicate_id: String,
    /// Subject bindings (`key`, `value`) for resolving conflict-domain scopes.
    pub subject: Vec<(String, String)>,
    /// Circumstance bindings (`key`, `value`) for resolving conflict-domain scopes.
    pub circumstance: Vec<(String, String)>,
    /// Ordered scenario events.
    pub events: Vec<AlloyScenarioEvent>,
}

/// One scenario event projected into Alloy (payload-bound — P0-FM-003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlloyScenarioEvent {
    /// Event id.
    pub event_id: String,
    /// Event kind.
    pub kind: AlloyEventKind,
    /// Action this event belongs to.
    pub action_id: Option<String>,
    /// Event plan hash.
    pub plan_hash: Option<String>,
    /// Op index, when the event binds one.
    pub op_index: Option<u32>,
    /// Fact kind this event attests about itself (producer attestation, P0-004).
    pub fact_kind: Option<String>,
    /// Scope the attested fact applies to.
    pub scope: Option<String>,
    /// Event anchor ids (core-model anchor relation).
    pub anchors: Vec<String>,
    /// Structured projection anchors with their claimed fact (producer
    /// grounding, P0-004 / Formal IR v2). Parallel to `anchors`; carries each
    /// anchor's observed-event id and the `fact_kind` the projection claims.
    pub anchor_facts: Vec<AlloyAnchorFact>,
    /// Lease grants attached to this event.
    pub leases: Vec<AlloyLeaseFact>,
    /// Barrier payload (for `execution.barrier_logged`).
    pub barrier: Option<crate::ir::FormalBarrierPayload>,
    /// Capability payload (for `execution.started`).
    pub capability: Option<crate::ir::FormalCapabilityPayload>,
    /// Authz decision payload (for `authz.decision_recorded`).
    pub authz_decision: Option<crate::ir::FormalAuthzDecisionPayload>,
}

/// One projection truth anchor projected into Alloy, with the fact it claims
/// about the observed-truth event it points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlloyAnchorFact {
    /// The observed-truth event id this anchor points at.
    pub event_id: String,
    /// Fact kind the projection claims for the anchored truth, if any.
    pub fact_kind: Option<String>,
    /// Scope the projection claims for the anchored truth, if any.
    pub scope: Option<String>,
}

/// One lease grant projected into Alloy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlloyLeaseFact {
    /// Lease id.
    pub lease_id: String,
    /// Resource id.
    pub resource: String,
    /// Scope token.
    pub scope: String,
    /// Claim mode.
    pub mode: AlloyLeaseMode,
    /// Constraint epoch the lease was granted in.
    pub epoch: u64,
}

/// Event kind projected into Alloy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlloyEventKind {
    /// `action.admitted`.
    ActionAdmitted,
    /// `action.planned`.
    ActionPlanned,
    /// `dispatch.logged`.
    DispatchLogged,
    /// `execution.barrier_logged`.
    ExecutionBarrierLogged,
    /// `execution.started`.
    ExecutionStarted,
    /// `execution.completed`.
    ExecutionCompleted,
    /// `observed_truth.committed`.
    ObservedTruthCommitted,
    /// `projection.emitted`.
    ProjectionEmitted,
    /// `lifecycle.closed`.
    LifecycleClosed,
    /// `gate.approved`.
    GateApproved,
    /// `gate.denied`.
    GateDenied,
    /// `constraint.lease_granted`.
    ConstraintLeaseGranted,
    /// `constraint.lease_released`.
    ConstraintLeaseReleased,
    /// `violation.detected`.
    ViolationDetected,
    /// `authz.decision_recorded`.
    AuthzDecisionRecorded,
    /// `drain.fence_requested`.
    DrainFenceRequested,
    /// `drain.fence_acquired`.
    DrainFenceAcquired,
}

/// Lease mode projected into Alloy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlloyLeaseMode {
    /// Exclusive write.
    Exclusive,
    /// Shared read.
    Shared,
    /// Token/quota.
    Token,
}

/// Generate Alloy facts from a compiled dispatch bundle.
///
/// # Errors
/// Currently infallible in practice; returns [`CodegenError`] for future
/// generator validation failures.
#[must_use = "the generated artifact must be written or checked"]
pub fn generate_alloy_facts(
    bundle: &CompiledDispatchBundle,
    scenario_hash: Option<&str>,
) -> Result<GeneratedArtifact, CodegenError> {
    generate_alloy_facts_internal(bundle, scenario_hash, None)
}

/// Generate Alloy facts from a compiled dispatch bundle and scenario projection.
///
/// # Errors
/// Returns [`CodegenError::Scenario`] when the scenario projection contains a
/// malformed hash or inconsistent event/anchor/lease ids.
#[must_use = "the generated artifact must be written or checked"]
pub fn generate_alloy_facts_with_scenario(
    bundle: &CompiledDispatchBundle,
    scenario: &AlloyScenarioFacts,
) -> Result<GeneratedArtifact, CodegenError> {
    generate_alloy_facts_internal(bundle, Some(&scenario.scenario_hash), Some(scenario))
}

fn generate_alloy_facts_internal(
    bundle: &CompiledDispatchBundle,
    _scenario_hash: Option<&str>,
    scenario: Option<&AlloyScenarioFacts>,
) -> Result<GeneratedArtifact, CodegenError> {
    let ir = build_formal_ir(bundle, scenario)?;
    let module_name = format!("generated/{}", alloy_ident(&bundle.body.bundle_id));
    let predicate_sigs = bundle
        .body
        .predicates
        .iter()
        .map(|predicate| predicate_sig(&predicate.predicate))
        .collect::<Vec<_>>();

    let mut text = String::new();
    text.push_str(&artifact_header(&ir, FormalTarget::Alloy, "facts"));
    let _written = write!(text, "module {module_name}\n\n");
    if scenario.is_some() {
        text.push_str("open core/causlane_core\n\n");
    }
    push_bundle_facts(&mut text, bundle, &predicate_sigs);
    if let Some(facts) = scenario {
        push_scenario_facts(&mut text, facts, bundle, &predicate_sigs)?;
    }

    Ok(GeneratedArtifact::new(
        FormalTarget::Alloy,
        "facts",
        &ir,
        text,
    ))
}

fn push_bundle_facts(
    text: &mut String,
    bundle: &CompiledDispatchBundle,
    predicate_sigs: &[String],
) {
    text.push_str("abstract sig Predicate {}\n");
    text.push_str("abstract sig ConsequenceProfile {}\n");
    text.push_str("abstract sig LifecycleClass {}\n");
    text.push_str("abstract sig BarrierPolicy {}\n");
    text.push_str("abstract sig ProjectionPolicy {}\n");
    text.push_str("abstract sig Invariant {}\n");
    text.push_str("abstract sig ScenarioResult {}\n");
    text.push_str("one sig RuntimeExecution, ProjectionRead, OversightMeta, TopologyMeta, EvidenceMeta, OutsideKernel extends ConsequenceProfile {}\n");
    text.push_str("one sig ExecutionBearing, ProjectionOnly, Meta extends LifecycleClass {}\n");
    text.push_str("one sig StrictWriteAhead, NoBarrier extends BarrierPolicy {}\n");
    text.push_str("one sig AnchoredProjection, NoProjectionPolicy extends ProjectionPolicy {}\n\n");
    text.push_str("one sig I_001, I_002, I_003, I_004, I_005, I_006, I_007, I_008, I_009, I_010 extends Invariant {}\n");
    text.push_str("one sig ExpectedPass, ExpectedFail extends ScenarioResult {}\n\n");
    for sig in predicate_sigs {
        let _written = writeln!(text, "one sig {sig} extends Predicate {{}}");
    }
    text.push('\n');
    text.push_str("one sig BundleFacts {\n");
    text.push_str("  predicates: set Predicate,\n");
    text.push_str("  runtimeExecution: set Predicate,\n");
    text.push_str("  executionBearing: set Predicate,\n");
    text.push_str("  strictBarrier: set Predicate,\n");
    text.push_str("  anchoredProjection: set Predicate,\n");
    text.push_str("  requiresWitness: set Predicate,\n");
    text.push_str("  requiresClaim: set Predicate,\n");
    text.push_str("  authzRequired: set Predicate,\n");
    text.push_str("  hasScenarioRef: set Predicate,\n");
    text.push_str("  formalObligation: Predicate -> Invariant,\n");
    // I-006 / P0-005: the verified-merge relation is scope-keyed (the conflict-domain
    // scopes a verified, applicable merge protocol permits overlap on), resolved per
    // scenario from the bundle by `causlane_contracts::resolve_mergeable_scopes` and
    // pinned in the scenario facts. An exclusive-lease conflict is relaxed ONLY on a
    // scope in this set, never globally.
    text.push_str("  mergeable: set LeaseScope\n");
    text.push_str("}\n\n");
    text.push_str("fact CompiledBundleFacts {\n");
    push_bundle_set(text, "predicates", predicate_sigs);
    push_bundle_set(
        text,
        "runtimeExecution",
        &filtered_predicates(bundle, |predicate| {
            predicate.consequence_profile == ConsequenceProfileDto::RuntimeExecution
        }),
    );
    push_bundle_set(
        text,
        "executionBearing",
        &filtered_predicates(bundle, |predicate| {
            predicate.lifecycle_class == LifecycleClassDto::ExecutionBearing
        }),
    );
    push_bundle_set(
        text,
        "strictBarrier",
        &filtered_predicates(bundle, |predicate| {
            predicate.barrier_policy == BarrierPolicyDto::StrictWriteAhead
        }),
    );
    push_bundle_set(
        text,
        "anchoredProjection",
        &filtered_predicates(bundle, |predicate| {
            predicate.projection_policy == ProjectionPolicyDto::Anchored
        }),
    );
    push_bundle_set(
        text,
        "requiresWitness",
        &filtered_predicates(bundle, |predicate| !predicate.required_witnesses.is_empty()),
    );
    push_bundle_set(
        text,
        "requiresClaim",
        &filtered_predicates(bundle, |predicate| !predicate.claims.is_empty()),
    );
    push_bundle_set(
        text,
        "authzRequired",
        &filtered_predicates(bundle, |predicate| {
            predicate.authz_decision_selector.required
        }),
    );
    push_bundle_set(
        text,
        "hasScenarioRef",
        &filtered_predicates(bundle, |predicate| !predicate.scenario_refs.is_empty()),
    );
    push_formal_obligation_relation(text, bundle);
    // `BundleFacts.mergeable` is scope-keyed and scenario-derived (pinned in
    // GeneratedScenarioFacts); for the bundle-only model there are no LeaseScope
    // atoms, so it is empty by construction.
    text.push_str("}\n");
}

fn push_formal_obligation_relation(text: &mut String, bundle: &CompiledDispatchBundle) {
    let pairs = bundle
        .body
        .predicates
        .iter()
        .flat_map(|predicate| {
            let predicate_sig = predicate_sig(&predicate.predicate);
            predicate
                .formal_obligations
                .iter()
                .map(move |invariant| format!("{predicate_sig} -> {}", invariant_sig(invariant)))
        })
        .collect::<Vec<_>>();
    if pairs.is_empty() {
        text.push_str("  no BundleFacts.formalObligation\n");
    } else {
        let _written = writeln!(
            text,
            "  BundleFacts.formalObligation = {}",
            pairs.join(" + ")
        );
    }
}

fn push_scenario_facts(
    text: &mut String,
    scenario: &AlloyScenarioFacts,
    bundle: &CompiledDispatchBundle,
    predicate_sigs: &[String],
) -> Result<(), CodegenError> {
    validate_hash("scenario_hash", &scenario.scenario_hash)?;
    validate_hash("scenario.plan_hash", &scenario.plan_hash)?;
    let event_sigs = scenario_event_sigs(scenario)?;
    let lease_sigs = scenario_lease_sigs(scenario)?;
    let mergeable_scope_sigs = crate::alloy_merge::resolve_mergeable_scope_sigs(bundle, scenario)?;
    // P1-002: one Action sig per distinct event action_id, one Plan sig per distinct
    // plan hash. A single-action scenario degenerates to one of each (unchanged).
    let action_sigs = crate::alloy_events::distinct_action_sigs(scenario);
    let base_plan_sigs = crate::alloy_events::distinct_plan_sigs(scenario)?;
    let result_sig = expected_result_sig(&scenario.expected_result)?;
    text.push_str("\n// Scenario facts projected from scenario.yaml.\n");
    for action_sig in &action_sigs {
        let _written = writeln!(text, "one sig {action_sig} extends Action {{}}");
    }
    for plan_sig in &base_plan_sigs {
        let _written = writeln!(text, "one sig {plan_sig} extends PlanHash {{}}");
    }
    text.push_str("one sig GeneratedScenarioExpectation { expected: one ScenarioResult }\n");
    for event_sig in &event_sigs {
        let _written = writeln!(text, "one sig {event_sig} extends Event {{}}");
    }
    push_lease_signatures(text, &lease_sigs);
    let capability_facts = crate::alloy_bindings::collect_capability_facts(scenario);
    let approval_facts = crate::alloy_bindings::collect_approval_facts(scenario);
    let fact_grounding_facts = crate::alloy_bindings::collect_fact_grounding_facts(scenario);
    let authz_facts = crate::alloy_authz::collect_authz_facts(scenario);
    let drain_facts = crate::alloy_drain::collect_drain_facts(scenario);
    crate::alloy_bindings::push_capability_signatures(text, &capability_facts);
    crate::alloy_bindings::push_approval_signatures(text, &approval_facts);
    crate::alloy_bindings::push_fact_grounding_signatures(text, &fact_grounding_facts);
    crate::alloy_authz::push_authz_signatures(text, &authz_facts);
    crate::alloy_drain::push_drain_signatures(text, &drain_facts);
    // PlanHash set = distinct base plan sigs ∪ approval extra plan sigs (deduped).
    let mut plan_sigs = base_plan_sigs.clone();
    for extra in approval_facts.extra_plan_sigs() {
        if !plan_sigs.contains(extra) {
            plan_sigs.push(extra.clone());
        }
    }
    text.push_str("\nfact GeneratedScenarioFacts {\n");
    push_exact_set(text, "Action", &action_sigs);
    push_exact_set(text, "PlanHash", &plan_sigs);
    push_bundle_set(text, "predicates", predicate_sigs);
    push_exact_set(text, "Event", &event_sigs);
    let _written = writeln!(
        text,
        "  GeneratedScenarioExpectation.expected = {result_sig}"
    );
    crate::alloy_events::push_event_fields(text, scenario, &event_sigs)?;
    push_lease_fields(text, &lease_sigs);
    push_exact_set(text, "BundleFacts.mergeable", &mergeable_scope_sigs);
    text.push_str("}\n\n");
    crate::alloy_bindings::push_capability_fact_block(text, &capability_facts);
    crate::alloy_bindings::push_approval_fact_block(text, &approval_facts);
    crate::alloy_bindings::push_fact_grounding_fact_block(text, &fact_grounding_facts);
    crate::alloy_authz::push_authz_fact_block(text, &authz_facts);
    crate::alloy_drain::push_drain_fact_block(text, &drain_facts);
    push_generated_checks(
        text,
        scenario.events.len(),
        action_sigs.len(),
        plan_sigs.len(),
        &lease_sigs,
        &mergeable_scope_sigs,
        &capability_facts,
        &approval_facts,
        &fact_grounding_facts,
        &authz_facts,
        &drain_facts,
        bundle,
    );
    Ok(())
}

fn expected_result_sig(expected_result: &str) -> Result<&'static str, CodegenError> {
    if expected_result == "pass" {
        return Ok("ExpectedPass");
    }
    if expected_result == "fail" {
        return Ok("ExpectedFail");
    }
    Err(CodegenError::Scenario(format!(
        "unknown expected_result {expected_result}"
    )))
}

fn scenario_event_sigs(scenario: &AlloyScenarioFacts) -> Result<Vec<String>, CodegenError> {
    if scenario.events.is_empty() {
        return Err(CodegenError::Scenario(
            "scenario must contain at least one event".to_owned(),
        ));
    }
    let mut seen = BTreeSet::new();
    let mut sigs = Vec::with_capacity(scenario.events.len());
    for event in &scenario.events {
        if !seen.insert(event.event_id.clone()) {
            return Err(CodegenError::Scenario(format!(
                "duplicate event id {}",
                event.event_id
            )));
        }
        sigs.push(format!("Event_{}", alloy_ident(&event.event_id)));
    }
    Ok(sigs)
}

/// A lease projected into Alloy, with its grant event and — when the scenario
/// releases it — the release event that ends its active window. The active
/// window `[grant .. release)` is what makes `GeneratedNoExclusiveConflicts`
/// interval-aware (P0-FM-004), mirroring `LeaseTable::grant` which checks a new
/// grant only against still-active leases.
struct LeaseSig {
    sig: String,
    lease: AlloyLeaseFact,
    grant_event_sig: String,
    release_event_sig: Option<String>,
}

fn scenario_lease_sigs(scenario: &AlloyScenarioFacts) -> Result<Vec<LeaseSig>, CodegenError> {
    // Map each lease id to the first event that releases it, so a granted lease
    // can carry its active-window upper bound.
    let mut release_by_lease = BTreeMap::new();
    for event in &scenario.events {
        if event.kind != AlloyEventKind::ConstraintLeaseReleased {
            continue;
        }
        let event_sig = format!("Event_{}", alloy_ident(&event.event_id));
        for lease in &event.leases {
            release_by_lease
                .entry(lease.lease_id.clone())
                .or_insert_with(|| event_sig.clone());
        }
    }
    let mut seen = BTreeSet::new();
    let mut leases = Vec::new();
    for event in &scenario.events {
        if event.kind != AlloyEventKind::ConstraintLeaseGranted {
            continue;
        }
        let event_sig = format!("Event_{}", alloy_ident(&event.event_id));
        for lease in &event.leases {
            if !seen.insert(lease.lease_id.clone()) {
                return Err(CodegenError::Scenario(format!(
                    "duplicate lease id {}",
                    lease.lease_id
                )));
            }
            leases.push(LeaseSig {
                sig: format!("Lease_{}", alloy_ident(&lease.lease_id)),
                lease: lease.clone(),
                grant_event_sig: event_sig.clone(),
                release_event_sig: release_by_lease.get(&lease.lease_id).cloned(),
            });
        }
    }
    Ok(leases)
}

fn push_lease_signatures(text: &mut String, leases: &[LeaseSig]) {
    text.push_str("\nabstract sig LeaseMode {}\n");
    text.push_str("one sig ExclusiveLease, SharedLease, TokenLease extends LeaseMode {}\n");
    text.push_str("sig Resource {}\n");
    text.push_str("sig LeaseScope {}\n");
    // `leaseEvent` is the grant; `releaseEvent` (lone) is the release that ends
    // the active window. A lease with no release stays active to the trace end.
    text.push_str(
        "sig Lease { leaseEvent: one Event, releaseEvent: lone Event, resource: one Resource, scope: one LeaseScope, mode: one LeaseMode }\n",
    );
    let mut resources = BTreeSet::new();
    let mut scopes = BTreeSet::new();
    for lease in leases {
        resources.insert(resource_sig(&lease.lease.resource));
        scopes.insert(scope_sig(&lease.lease.scope));
    }
    for resource in resources {
        let _written = writeln!(text, "one sig {resource} extends Resource {{}}");
    }
    for scope in scopes {
        let _written = writeln!(text, "one sig {scope} extends LeaseScope {{}}");
    }
    for lease in leases {
        let _written = writeln!(text, "one sig {} extends Lease {{}}", lease.sig);
    }
}

fn push_lease_fields(text: &mut String, leases: &[LeaseSig]) {
    let lease_sigs = leases
        .iter()
        .map(|lease| lease.sig.clone())
        .collect::<Vec<_>>();
    push_exact_set(text, "Lease", &lease_sigs);
    let resources = leases
        .iter()
        .map(|lease| resource_sig(&lease.lease.resource))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let scopes = leases
        .iter()
        .map(|lease| scope_sig(&lease.lease.scope))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    push_exact_set(text, "Resource", &resources);
    push_exact_set(text, "LeaseScope", &scopes);
    for lease in leases {
        let mode = lease_mode_sig(lease.lease.mode);
        let resource = resource_sig(&lease.lease.resource);
        let scope = scope_sig(&lease.lease.scope);
        let _written = writeln!(
            text,
            "  {}.leaseEvent = {}",
            lease.sig, lease.grant_event_sig
        );
        match &lease.release_event_sig {
            Some(release) => {
                let _written = writeln!(text, "  {}.releaseEvent = {release}", lease.sig);
            }
            None => {
                let _written = writeln!(text, "  no {}.releaseEvent", lease.sig);
            }
        }
        let _written = writeln!(text, "  {}.resource = {resource}", lease.sig);
        let _written = writeln!(text, "  {}.scope = {scope}", lease.sig);
        let _written = writeln!(text, "  {}.mode = {mode}", lease.sig);
    }
}

#[allow(clippy::too_many_arguments)]
fn push_generated_checks(
    text: &mut String,
    event_count: usize,
    action_count: usize,
    plan_count: usize,
    leases: &[LeaseSig],
    mergeable_scope_sigs: &[String],
    capability_facts: &CapabilityFacts,
    approval_facts: &ApprovalFacts,
    fact_grounding_facts: &crate::alloy_bindings::FactGroundingFacts,
    authz_facts: &crate::alloy_authz::AuthzFacts,
    drain_facts: &crate::alloy_drain::DrainFacts,
    bundle: &CompiledDispatchBundle,
) {
    // I-006, interval + merge aware (P0-FM-004). Mirrors causlane-core
    // `leases_conflict`/`claim_modes_conflict`: two leases on the same
    // resource+scope conflict when at least ONE is exclusive (not both), their
    // active windows `[grant .. release)` overlap, and no verified merge protocol
    // is in effect. The active-window test mirrors `LeaseTable::grant`, which
    // checks a new grant only against still-active leases:
    //   - `a` granted by the same event as `b` (simultaneously held), or
    //   - `a` granted before `b` (`a.leaseEvent in b.leaseEvent.hb`) and `a` not
    //     yet released at `b`'s grant (no release, or release not before the grant).
    // `BundleFacts.mergeable` is the scope-keyed verified-merge relation (P0-005):
    // an exclusive-lease conflict is relaxed ONLY when the overlapping scope is in
    // `mergeable` (a verified, applicable merge protocol permits it), never globally.
    // The set is empty unless a merge protocol is Verified AND applicable, matching
    // `resolve_mergeable_scopes`' fail-closed default on the lease path.
    text.push_str("pred GeneratedNoExclusiveConflicts {\n");
    text.push_str("  no disj a, b: Lease |\n");
    text.push_str("    (a.mode = ExclusiveLease or b.mode = ExclusiveLease) and\n");
    text.push_str("    a.resource = b.resource and a.scope = b.scope and\n");
    text.push_str("    a.scope not in BundleFacts.mergeable and\n");
    text.push_str("    (a.leaseEvent = b.leaseEvent or\n");
    text.push_str("     (a.leaseEvent in b.leaseEvent.hb and\n");
    text.push_str("      not (some a.releaseEvent and a.releaseEvent in b.leaseEvent.hb)))\n");
    text.push_str("}\n\n");
    text.push_str("assert GeneratedTraceSatisfiesCore {\n");
    text.push_str("  Enforced and GeneratedNoExclusiveConflicts\n");
    text.push_str("}\n\n");
    text.push_str("assert GeneratedRuntimePredicatesHaveBarriers {\n");
    text.push_str("  all p: BundleFacts.runtimeExecution | p in BundleFacts.executionBearing and p in BundleFacts.strictBarrier\n");
    text.push_str("}\n\n");
    // Fail-closed proof: when no verified merge protocol applies the relation is
    // empty. Emitted only for such scenarios (a scenario that genuinely exercises a
    // verified merge has a non-empty `mergeable`, so this would not hold there).
    if mergeable_scope_sigs.is_empty() {
        text.push_str("assert GeneratedMergeableDefaultEmpty { no BundleFacts.mergeable }\n\n");
    }
    crate::alloy_bindings::push_binding_assertions(text, capability_facts, approval_facts);
    crate::alloy_bindings::push_fact_grounding_assertion(text, fact_grounding_facts);
    crate::alloy_authz::push_authz_assertion(text, authz_facts);
    crate::alloy_drain::push_drain_assertion(text);
    let mut scope = generated_scope(event_count, action_count, plan_count, leases, bundle);
    scope.push_str(&crate::alloy_bindings::binding_scope_suffix(
        capability_facts,
        approval_facts,
    ));
    scope.push_str(&crate::alloy_bindings::fact_grounding_scope_suffix(
        fact_grounding_facts,
    ));
    scope.push_str(&crate::alloy_authz::authz_scope_suffix(authz_facts));
    scope.push_str(&crate::alloy_drain::drain_scope_suffix(drain_facts));
    let _written = writeln!(text, "check GeneratedTraceSatisfiesCore for {scope}");
    let _written = writeln!(
        text,
        "check GeneratedRuntimePredicatesHaveBarriers for {scope}"
    );
    if mergeable_scope_sigs.is_empty() {
        let _written = writeln!(text, "check GeneratedMergeableDefaultEmpty for {scope}");
    }
    crate::alloy_bindings::push_binding_checks(text, &scope, capability_facts, approval_facts);
    crate::alloy_bindings::push_fact_grounding_check(text, &scope, fact_grounding_facts);
    crate::alloy_authz::push_authz_check(text, &scope, authz_facts);
    let _written = writeln!(text, "check GeneratedDrainFenceClear for {scope}");
}

fn generated_scope(
    event_count: usize,
    action_count: usize,
    plan_count: usize,
    leases: &[LeaseSig],
    bundle: &CompiledDispatchBundle,
) -> String {
    let resource_count = leases
        .iter()
        .map(|lease| lease.lease.resource.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let scope_count = leases
        .iter()
        .map(|lease| lease.lease.scope.clone())
        .collect::<BTreeSet<_>>()
        .len();
    format!(
        "exactly {event_count} Event, exactly {action_count} Action, exactly {plan_count} PlanHash, exactly {} Predicate, exactly {} Lease, exactly {resource_count} Resource, exactly {scope_count} LeaseScope",
        bundle.body.predicates.len(),
        leases.len()
    )
}

fn filtered_predicates(
    bundle: &CompiledDispatchBundle,
    keep: impl Fn(&causlane_contracts::CompiledPredicate) -> bool,
) -> Vec<String> {
    bundle
        .body
        .predicates
        .iter()
        .filter(|predicate| keep(predicate))
        .map(|predicate| predicate_sig(&predicate.predicate))
        .collect()
}

fn push_bundle_set(text: &mut String, name: &str, items: &[String]) {
    let _written = writeln!(text, "  BundleFacts.{name} = {}", set_expr(items));
}

pub(crate) fn push_exact_set(text: &mut String, name: &str, items: &[String]) {
    let _written = writeln!(text, "  {name} = {}", set_expr(items));
}

pub(crate) fn validate_hash(field: &str, value: &str) -> Result<(), CodegenError> {
    if causlane_contracts::is_canonical_sha256_token(value) {
        Ok(())
    } else {
        Err(CodegenError::Scenario(format!(
            "{field} must be sha256:<64 lowercase hex>"
        )))
    }
}

fn predicate_sig(predicate: &str) -> String {
    format!("Pred_{}", alloy_ident(predicate))
}

fn invariant_sig(invariant: &str) -> String {
    format!("I_{}", invariant.replace("I-", ""))
}

pub(crate) fn event_kind_sig(kind: AlloyEventKind) -> &'static str {
    match kind {
        AlloyEventKind::ActionAdmitted => "ActionAdmitted",
        AlloyEventKind::ActionPlanned => "ActionPlanned",
        AlloyEventKind::DispatchLogged => "DispatchLogged",
        AlloyEventKind::ExecutionBarrierLogged => "ExecutionBarrierLogged",
        AlloyEventKind::ExecutionStarted => "ExecutionStarted",
        AlloyEventKind::ExecutionCompleted => "ExecutionCompleted",
        AlloyEventKind::ObservedTruthCommitted => "ObservedTruthCommitted",
        AlloyEventKind::ProjectionEmitted => "ProjectionEmitted",
        AlloyEventKind::LifecycleClosed => "LifecycleClosed",
        AlloyEventKind::GateApproved => "GateApproved",
        AlloyEventKind::GateDenied => "GateDenied",
        AlloyEventKind::ConstraintLeaseGranted => "ConstraintLeaseGranted",
        AlloyEventKind::ConstraintLeaseReleased => "ConstraintLeaseReleased",
        AlloyEventKind::ViolationDetected => "ViolationDetected",
        AlloyEventKind::AuthzDecisionRecorded => "AuthzDecisionRecorded",
        AlloyEventKind::DrainFenceRequested => "DrainFenceRequested",
        AlloyEventKind::DrainFenceAcquired => "DrainFenceAcquired",
    }
}

fn lease_mode_sig(mode: AlloyLeaseMode) -> &'static str {
    match mode {
        AlloyLeaseMode::Exclusive => "ExclusiveLease",
        AlloyLeaseMode::Shared => "SharedLease",
        AlloyLeaseMode::Token => "TokenLease",
    }
}

fn resource_sig(resource: &str) -> String {
    format!("Resource_{}", alloy_ident(resource))
}

pub(crate) fn scope_sig(scope: &str) -> String {
    format!("LeaseScope_{}", alloy_ident(scope))
}

pub(crate) fn alloy_ident(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_digit())
    {
        out.insert(0, '_');
    }
    out
}

pub(crate) fn set_expr(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(" + ")
    }
}
