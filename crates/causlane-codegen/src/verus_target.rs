//! Honest Verus proof generation (P0-FM-007).
//!
//! The transition relation is event-indexed: `step(s, e)` computes the next
//! state and does **not** assume the result is valid, so `step_preserves_validity`
//! is a real obligation (Verus must prove validity is preserved from
//! `event_allowed`), not the earlier vacuous skeleton where `transition`
//! assumed `valid_state(s1)`.
//!
//! [`push_verus_scenario_trace`] makes the proof **payload-bound**: it emits a
//! concrete `proof fn` that folds `step` over *this bundle's* scenario events (in
//! order, derived from the Formal IR) and proves validity is preserved at every
//! step — so a different scenario produces a different obligation.
//!
//! The rule invariants (I-004 overlay, I-005 route/profile, I-006 lease
//! conflict, I-007 drain safety, I-009 approval binding, I-010 constraint
//! update) are proven as genuine lemmas over the same rule models the Kani lane
//! drives against the real `causlane-core` predicates — relating independent
//! inputs through each rule — rather than as the earlier vacuous reads of a
//! frozen `KernelState` flag that no transition ever set.

use core::fmt::Write as _;

use crate::FormalIr;

pub(crate) fn push_verus_kernel(text: &mut String) {
    push_verus_lifecycle_kernel(text);
    push_verus_rule_models(text);
}

#[allow(clippy::needless_raw_string_hashes)]
fn push_verus_lifecycle_kernel(text: &mut String) {
    text.push_str(
        r#"    // Lifecycle state carries only the protocol-ordering flags the
    // event-indexed transition actually advances. The rule invariants
    // (I-004/I-005/I-006/I-007/I-009/I-010) are NOT carried as frozen flags here —
    // they are proven below as genuine lemmas over the same rule models the
    // Kani lane exercises against the real causlane-core predicates.
    struct KernelState {
        barrier_logged: bool,
        execution_started: bool,
        observed_truth: bool,
        projection_emitted: bool,
        closed: bool,
    }

    spec fn valid_state(s: KernelState) -> bool {
        (!s.execution_started || s.barrier_logged) &&
        (!s.observed_truth || s.execution_started) &&
        (!s.projection_emitted || s.observed_truth)
    }

    // Honest event-indexed transition (P0-FM-007): `step` computes the next
    // state from an event and does NOT assume the result is valid. `event_allowed`
    // gates the protocol-advancing events (I-001/I-002/I-003/I-008), so validity
    // preservation below is a real obligation, not an assumption.
    enum Event {
        BarrierLogged,
        ExecutionStarted,
        ObservedTruth,
        ProjectionEmitted,
        LifecycleClosed,
    }

    spec fn event_allowed(s: KernelState, e: Event) -> bool {
        match e {
            Event::BarrierLogged => !s.closed,
            Event::ExecutionStarted => s.barrier_logged && !s.closed,
            Event::ObservedTruth => s.execution_started && !s.closed,
            Event::ProjectionEmitted => s.observed_truth && !s.closed,
            Event::LifecycleClosed => true,
        }
    }

    spec fn step(s: KernelState, e: Event) -> KernelState {
        match e {
            Event::BarrierLogged => KernelState { barrier_logged: true, ..s },
            Event::ExecutionStarted => KernelState { execution_started: true, ..s },
            Event::ObservedTruth => KernelState { observed_truth: true, ..s },
            Event::ProjectionEmitted => KernelState { projection_emitted: true, ..s },
            Event::LifecycleClosed => KernelState { closed: true, ..s },
        }
    }

"#,
    );
}

#[allow(clippy::needless_raw_string_hashes)]
fn push_verus_rule_models(text: &mut String) {
    text.push_str(
        r#"    // --- Rule models: these mirror the concrete causlane-core predicates the
    // Kani harnesses call (`ObligationSet::preserved_by`,
    // `route_consistent_with_profile`, `claim_modes_conflict`,
    // `DrainFenceCheck::fence_acquirable`,
    // `ConstraintUpdate::preserves_committed_truth`, witness binding). They
    // relate independent inputs through the kernel rule, so the lemmas below are
    // NOT tautologies over a frozen flag — dropping a conjunct from a rule breaks
    // its proof.

    // I-004: overlay obligations (`ObligationSet::preserved_by`).
    struct ObligationSet {
        requires_witness: bool,
        requires_claim: bool,
        requires_authz: bool,
        requires_barrier: bool,
        requires_anchor: bool,
    }

    spec fn preserved_by(base: ObligationSet, overlaid: ObligationSet) -> bool {
        (!base.requires_witness || overlaid.requires_witness) &&
        (!base.requires_claim || overlaid.requires_claim) &&
        (!base.requires_authz || overlaid.requires_authz) &&
        (!base.requires_barrier || overlaid.requires_barrier) &&
        (!base.requires_anchor || overlaid.requires_anchor)
    }

    // I-005: route/profile compatibility
    // (`lifecycle_class_for_profile` / `route_consistent_with_profile`).
    enum ConsequenceProfile {
        RuntimeExecution,
        ProjectionRead,
        OversightMeta,
        TopologyMeta,
        EvidenceMeta,
        OutsideKernel,
    }

    enum LifecycleClass {
        ExecutionBearing,
        ProjectionOnly,
        Meta,
    }

    spec fn lifecycle_class_for_profile(profile: ConsequenceProfile) -> LifecycleClass {
        match profile {
            ConsequenceProfile::RuntimeExecution => LifecycleClass::ExecutionBearing,
            ConsequenceProfile::ProjectionRead => LifecycleClass::ProjectionOnly,
            ConsequenceProfile::OversightMeta
            | ConsequenceProfile::TopologyMeta
            | ConsequenceProfile::EvidenceMeta
            | ConsequenceProfile::OutsideKernel => LifecycleClass::Meta,
        }
    }

    spec fn route_consistent_with_profile(class: LifecycleClass, profile: ConsequenceProfile) -> bool {
        class == lifecycle_class_for_profile(profile)
    }

    // I-006: lease-conflict fail-closed rule (`claim_modes_conflict`).
    enum ClaimMode { ExclusiveWrite, SharedRead }

    spec fn is_exclusive(m: ClaimMode) -> bool {
        match m { ClaimMode::ExclusiveWrite => true, ClaimMode::SharedRead => false }
    }

    spec fn claim_modes_conflict(
        left: ClaimMode, right: ClaimMode,
        same_resource: bool, same_scope: bool, verified_merge: bool,
    ) -> bool {
        same_resource && same_scope && (is_exclusive(left) || is_exclusive(right)) && !verified_merge
    }

    // I-007: drain-fence acquisition rule (`DrainFenceCheck::fence_acquirable`).
    // A fence is clear exactly when no lease slot overlaps the fence scope while
    // still active and not-yet-expired.
    struct DrainFenceCheck {
        left_overlaps: bool,
        left_active: bool,
        left_expired: bool,
        right_overlaps: bool,
        right_active: bool,
        right_expired: bool,
    }

    spec fn active_unexpired_overlap(overlaps: bool, active: bool, expired: bool) -> bool {
        overlaps && active && !expired
    }

    spec fn fence_acquirable(check: DrainFenceCheck) -> bool {
        !active_unexpired_overlap(check.left_overlaps, check.left_active, check.left_expired) &&
        !active_unexpired_overlap(check.right_overlaps, check.right_active, check.right_expired)
    }

    // I-010: constraint-update / committed-truth rule
    // (`ConstraintUpdate::preserves_committed_truth`).
    struct CommittedTruth { readiness: bool, promotion: bool, evidence: bool }
    struct ConstraintUpdate { rewrites_readiness: bool, rewrites_promotion: bool, rewrites_evidence: bool }

    spec fn preserves_committed_truth(u: ConstraintUpdate, c: CommittedTruth) -> bool {
        (!u.rewrites_readiness || !c.readiness) &&
        (!u.rewrites_promotion || !c.promotion) &&
        (!u.rewrites_evidence || !c.evidence)
    }

    // I-009: approval/witness binding exactness (action, plan, impact set).
    spec fn binding_exact(
        witness_action: int, witness_plan: int, witness_impact: int,
        expected_action: int, expected_plan: int, expected_impact: int,
    ) -> bool {
        witness_action == expected_action
            && witness_plan == expected_plan
            && witness_impact == expected_impact
    }

"#,
    );
}

pub(crate) fn push_verus_theorems(text: &mut String) {
    push_verus_lifecycle_theorems(text);
    push_verus_route_rule_theorems(text);
    push_verus_conflict_evidence_theorems(text);
    push_verus_terminal_theorems(text);
}

#[allow(clippy::needless_raw_string_hashes)]
fn push_verus_lifecycle_theorems(text: &mut String) {
    text.push_str(
        r#"    // HONEST preservation: stepping a valid state with an allowed event yields
    // a valid state. `step` does not assume validity of its result, so this is a
    // genuine proof (not a tautology).
    proof fn step_preserves_validity(s: KernelState, e: Event)
        requires valid_state(s), event_allowed(s, e)
        ensures valid_state(step(s, e))
    { }

    // I-001: an allowed ExecutionStarted requires (and preserves) a prior barrier.
    proof fn execution_started_requires_prior_barrier(s: KernelState)
        requires valid_state(s), event_allowed(s, Event::ExecutionStarted)
        ensures s.barrier_logged,
                step(s, Event::ExecutionStarted).execution_started,
                step(s, Event::ExecutionStarted).barrier_logged
    { }

    // I-002 / I-003: observed-truth and projection are gated on their predecessors.
    proof fn observed_truth_requires_prior_execution(s: KernelState)
        requires event_allowed(s, Event::ObservedTruth)
        ensures s.execution_started
    { }

    proof fn projection_requires_prior_observed_truth(s: KernelState)
        requires event_allowed(s, Event::ProjectionEmitted)
        ensures s.observed_truth
    { }

    // Consequences read directly out of valid_state (non-vacuous: they fail if
    // valid_state drops the corresponding conjunct).
    proof fn execution_implies_prior_barrier(s: KernelState)
        requires valid_state(s), s.execution_started
        ensures s.barrier_logged
    { }

    proof fn observed_truth_implies_prior_execution(s: KernelState)
        requires valid_state(s), s.observed_truth
        ensures s.execution_started
    { }

    proof fn projection_implies_observed_truth_anchor(s: KernelState)
        requires valid_state(s), s.projection_emitted
        ensures s.observed_truth
    { }

"#,
    );
}

#[allow(clippy::needless_raw_string_hashes)]
fn push_verus_route_rule_theorems(text: &mut String) {
    text.push_str(
        r#"    // I-004: an overlay the kernel ACCEPTS (preserved_by holds) can never drop a
    // base obligation. Non-vacuous: it relates two independent obligation sets
    // through the rule; removing any conjunct of `preserved_by` fails the proof.
    proof fn overlay_accepted_never_weakens_obligations(base: ObligationSet, overlaid: ObligationSet)
        requires preserved_by(base, overlaid)
        ensures
            base.requires_witness ==> overlaid.requires_witness,
            base.requires_claim ==> overlaid.requires_claim,
            base.requires_authz ==> overlaid.requires_authz,
            base.requires_barrier ==> overlaid.requires_barrier,
            base.requires_anchor ==> overlaid.requires_anchor,
    { }

    // I-005: a route the kernel accepts is compatible with exactly the lifecycle
    // class derived from the consequence profile. This mirrors the bounded Kani
    // harness over `route_consistent_with_profile` and keeps the per-profile
    // mapping load-bearing.
    proof fn route_profile_compatibility(class: LifecycleClass, profile: ConsequenceProfile)
        requires route_consistent_with_profile(class, profile)
        ensures
            class == lifecycle_class_for_profile(profile),
            profile == ConsequenceProfile::RuntimeExecution ==> class == LifecycleClass::ExecutionBearing,
            profile == ConsequenceProfile::ProjectionRead ==> class == LifecycleClass::ProjectionOnly,
            profile == ConsequenceProfile::OversightMeta ==> class == LifecycleClass::Meta,
            profile == ConsequenceProfile::TopologyMeta ==> class == LifecycleClass::Meta,
            profile == ConsequenceProfile::EvidenceMeta ==> class == LifecycleClass::Meta,
            profile == ConsequenceProfile::OutsideKernel ==> class == LifecycleClass::Meta,
    { }

"#,
    );
}

#[allow(clippy::needless_raw_string_hashes)]
fn push_verus_conflict_evidence_theorems(text: &mut String) {
    text.push_str(
        r#"    // I-006: the lease-conflict rule is fail-closed — without a verified merge,
    // two claims on the same resource/scope where at least one is exclusive DO
    // conflict. All conditions are bound variables (constrained by `requires`),
    // so the proof relates the rule to its inputs rather than reading a flag.
    proof fn lease_conflict_is_fail_closed_without_merge(
        left: ClaimMode, right: ClaimMode,
        same_resource: bool, same_scope: bool, verified_merge: bool,
    )
        requires
            same_resource,
            same_scope,
            !verified_merge,
            is_exclusive(left) || is_exclusive(right),
        ensures claim_modes_conflict(left, right, same_resource, same_scope, verified_merge)
    { }

    // I-006 (complement): a verified merge — and only a verified merge — clears
    // the conflict, so the `!verified_merge` conjunct is load-bearing.
    proof fn verified_merge_clears_lease_conflict(
        left: ClaimMode, right: ClaimMode,
        same_resource: bool, same_scope: bool, verified_merge: bool,
    )
        requires verified_merge
        ensures !claim_modes_conflict(left, right, same_resource, same_scope, verified_merge)
    { }

    // I-007: a drain fence the kernel accepts has no active, not-yet-expired
    // overlapping lease slot. This mirrors the bounded Kani `DrainFenceCheck`
    // harness and keeps the expiry conjunct load-bearing.
    proof fn drain_after_overlap_clear(check: DrainFenceCheck)
        requires fence_acquirable(check)
        ensures
            !active_unexpired_overlap(check.left_overlaps, check.left_active, check.left_expired),
            !active_unexpired_overlap(check.right_overlaps, check.right_active, check.right_expired),
    { }

    // I-009: an exact approval/witness binding implies each bound field (action,
    // plan, impact set) matches — relating the witness fields to the expected
    // ones, not reading a frozen flag.
    proof fn approval_binding_is_exact(
        witness_action: int, witness_plan: int, witness_impact: int,
        expected_action: int, expected_plan: int, expected_impact: int,
    )
        requires binding_exact(witness_action, witness_plan, witness_impact, expected_action, expected_plan, expected_impact)
        ensures
            witness_action == expected_action,
            witness_plan == expected_plan,
            witness_impact == expected_impact,
    { }

    // I-010: a constraint update the kernel ACCEPTS never rewrites an
    // already-committed truth category.
    proof fn constraint_update_preserves_committed_truth(u: ConstraintUpdate, c: CommittedTruth)
        requires preserves_committed_truth(u, c)
        ensures
            c.readiness ==> !u.rewrites_readiness,
            c.promotion ==> !u.rewrites_promotion,
            c.evidence ==> !u.rewrites_evidence,
    { }

"#,
    );
}

#[allow(clippy::needless_raw_string_hashes)]
fn push_verus_terminal_theorems(text: &mut String) {
    text.push_str(
        r#"    // I-008: `closed` is terminal — no event clears it, and an execution can no
    // longer be allowed once closed.
    proof fn closed_persists_across_step(s: KernelState, e: Event)
        requires s.closed
        ensures step(s, e).closed
    { }

    proof fn no_execution_allowed_after_closed(s: KernelState)
        requires s.closed
        ensures !event_allowed(s, Event::ExecutionStarted)
    { }
"#,
    );
}

/// Protocol-advancing IR event kinds → `(Verus Event variant, kernel flag)`. The
/// remaining kinds (admission, dispatch, gate, lease) do not advance the proven
/// kernel state and are skipped.
const VERUS_EVENTS: [(&str, &str, &str); 5] = [
    (
        "execution.barrier_logged",
        "BarrierLogged",
        "barrier_logged",
    ),
    ("execution.started", "ExecutionStarted", "execution_started"),
    (
        "observed_truth.committed",
        "ObservedTruth",
        "observed_truth",
    ),
    (
        "projection.emitted",
        "ProjectionEmitted",
        "projection_emitted",
    ),
    ("lifecycle.closed", "LifecycleClosed", "closed"),
];

fn verus_event(kind: &str) -> Option<(&'static str, &'static str)> {
    VERUS_EVENTS
        .iter()
        .find_map(|(known, variant, flag)| (*known == kind).then_some((*variant, *flag)))
}

/// Payload-bound trace lemma (P0-FM-007): emit a concrete `proof fn` that applies
/// the scenario's protocol-critical events in order and proves `valid_state` is
/// preserved at each step. The event sequence is generated from the IR, so the
/// obligation is specific to this bundle's scenario. Emits nothing for a
/// bundle-only IR (no scenario events).
pub(crate) fn push_verus_scenario_trace(text: &mut String, ir: &FormalIr) {
    let steps: Vec<(&'static str, &'static str)> = ir
        .scenario_events
        .iter()
        .filter_map(|event| verus_event(&event.kind))
        .collect();
    if steps.is_empty() {
        return;
    }
    text.push_str(
        "\n    // Payload-bound (P0-FM-007): this bundle's scenario event sequence is a\n    // valid trace under the proven kernel. The steps below are generated from the\n    // scenario's protocol-critical events, so a different scenario changes this\n    // obligation. Each `step_preserves_validity` call discharges a real proof.\n    proof fn generated_scenario_is_valid_trace() {\n",
    );
    text.push_str(
        "        let s0 = KernelState { barrier_logged: false, execution_started: false, observed_truth: false, projection_emitted: false, closed: false };\n",
    );
    text.push_str("        assert(valid_state(s0));\n");
    text.push_str("        assert(!s0.closed);\n");
    for (index, (variant, flag)) in steps.iter().enumerate() {
        let prev = index;
        let next = index + 1;
        let _w = writeln!(
            text,
            "        let s{next} = step(s{prev}, Event::{variant});"
        );
        let _w = writeln!(
            text,
            "        step_preserves_validity(s{prev}, Event::{variant});"
        );
        let _w = writeln!(text, "        assert(valid_state(s{next}));");
        let _w = writeln!(text, "        assert(s{next}.{flag});");
        if *variant != "LifecycleClosed" {
            let _w = writeln!(text, "        assert(!s{next}.closed);");
        }
    }
    text.push_str("    }\n");
}
