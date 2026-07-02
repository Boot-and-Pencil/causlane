//! Per-lane formal coverage obligations (P0-006).
//!
//! A coverage cell may claim `passed`/`covered` for an invariant on a lane only
//! when a **concrete named check** establishes it: an Alloy assertion, a P spec,
//! a Kani harness, a Verus proof fn, a Lean4 theorem application, or — for the
//! executable replay lane — a refuted negative control the gate runs every time.
//! These tables are the
//! single source of truth for that mapping.
//!
//! [`present_obligations`] keeps only the candidate checks whose `check_id`
//! literally appears in a generated artifact's text, so a conditionally-emitted
//! check (or a renamed one) can never let the coverage matrix out-run the checks
//! the artifact actually contains. The check ids below were taken from the
//! generators in this crate (`alloy.rs`/`alloy_bindings.rs`, `targets.rs`,
//! `kani_target.rs`, `verus_target.rs`, `lean4_target.rs`); the
//! `obligations_match_generators`
//! tests assert every candidate is present in freshly generated artifacts so the
//! tables cannot silently drift out of the overclaim they prevent.

use causlane_contracts::ACTIVE_INVARIANT_IDS;

use crate::{FormalTarget, ReceiptObligation};

/// Every invariant the coverage matrix reports on, in stable display order.
pub(crate) const ALL_INVARIANTS: &[&str] = ACTIVE_INVARIANT_IDS;

// Alloy generated checks (in the bundle/scenario facts artifact, not the generic
// base model). `GeneratedTraceSatisfiesCore` asserts `Enforced` (the base-model
// I-001/I-002/I-003/I-008 lifecycle guards) together with `GeneratedNoExclusiveConflicts`
// (I-006). `GeneratedDrainFenceClear` is the I-007 *structural* check (a drain fence
// over a scope must have no active overlapping exclusive lease) — emitted for every
// scenario (vacuous over `DrainFence = none` when the trace has no drain), so the
// positive artifact still carries it; replay remains the temporal/expiry authority.
// The binding assertions are payload-bound and emitted only when the
// bundle carries the relevant facts (hence kept honest by `present_obligations`).
const ALLOY: &[(&str, &str)] = &[
    ("I-001", "GeneratedTraceSatisfiesCore"),
    ("I-002", "GeneratedTraceSatisfiesCore"),
    ("I-003", "GeneratedTraceSatisfiesCore"),
    ("I-003", "GeneratedAnchorFactGrounded"),
    ("I-006", "GeneratedTraceSatisfiesCore"),
    ("I-007", "GeneratedDrainFenceClear"),
    ("I-008", "GeneratedTraceSatisfiesCore"),
    ("I-009", "GeneratedApprovalBindingHolds"),
    ("I-009", "GeneratedWitnessFactGrounded"),
    ("I-009", "GeneratedAnchorFactGrounded"),
];

// P specs (each `spec NAME observes ...` with an explicit assert).
const P: &[(&str, &str)] = &[
    ("I-001", "NoExecutionBeforeBarrier"),
    ("I-002", "NoObservedWithoutExecution"),
    ("I-003", "NoProjectionWithoutAnchor"),
    ("I-003", "AnchorFactGrounded"),
    ("I-006", "NoConflictingActiveLeases"),
    ("I-007", "DrainBlocksNewMutableAdmission"),
    ("I-008", "NoEventsAfterClosed"),
    ("I-009", "WitnessFactGrounded"),
    ("I-009", "AnchorFactGrounded"),
    ("I-009", "AuthzDecisionGroundsBarrier"),
    ("I-010", "ConstraintUpdateDoesNotRewriteTruth"),
];

// Kani harnesses (`#[kani::proof] fn NAME`) — one per invariant, each exercising
// the real core predicate under bounded nondeterminism.
const KANI: &[(&str, &str)] = &[
    ("I-001", "execution_requires_prior_barrier_nondet"),
    ("I-002", "observed_truth_requires_prior_execution_nondet"),
    (
        "I-003",
        "projection_anchor_source_kind_is_observed_truth_only",
    ),
    ("I-004", "overlay_never_weakens_obligations_nondet"),
    ("I-005", "route_is_allowed_only_for_matching_profile_nondet"),
    (
        "I-006",
        "lease_conflict_rule_is_fail_closed_without_verified_merge",
    ),
    (
        "I-007",
        "drain_fence_acquirable_only_without_active_overlap_nondet",
    ),
    ("I-008", "closed_stage_is_terminal_nondet"),
    (
        "I-009",
        "witness_binding_is_exact_for_action_plan_and_impact",
    ),
    (
        "I-010",
        "constraint_update_cannot_rewrite_committed_truth_nondet",
    ),
];

// Verus proof fns. The Verus lane is now always run and blocking (the
// LANE_REALITY exception was dropped 2026-06-21), so these gate every
// check-verification-full run; they are grounded in concrete proof fns so the matrix
// reports `passed` only where Verus actually carries a proof.
const VERUS: &[(&str, &str)] = &[
    ("I-001", "execution_started_requires_prior_barrier"),
    ("I-002", "observed_truth_requires_prior_execution"),
    ("I-003", "projection_requires_prior_observed_truth"),
    ("I-004", "overlay_accepted_never_weakens_obligations"),
    ("I-005", "route_profile_compatibility"),
    ("I-006", "lease_conflict_is_fail_closed_without_merge"),
    ("I-007", "drain_after_overlap_clear"),
    ("I-008", "closed_persists_across_step"),
    ("I-009", "approval_binding_is_exact"),
    ("I-010", "constraint_update_preserves_committed_truth"),
];

// Lean4 theorem applications generated from the scenario-bound Formal IR. The
// Lean4 lane is always run and blocking in check-verification-full, and every credited
// row is grounded in a named theorem that must compile under `lake env lean`.
const LEAN4: &[(&str, &str)] = &[
    ("I-001", "valid_trace_execution_started_has_prior_barrier"),
    ("I-002", "valid_trace_observed_truth_has_prior_execution"),
    ("I-003", "projection_anchor_soundness"),
    ("I-004", "overlay_monotonicity"),
    ("I-005", "route_profile_compatibility"),
    ("I-006", "lease_conflict_fail_closed"),
    ("I-006", "verified_merge_algebra"),
    ("I-007", "drain_after_overlap_clear"),
    ("I-008", "closed_is_terminal"),
    ("I-009", "witness_exact_binding"),
    ("I-010", "constraint_update_future_only"),
];

// Replay is the executable oracle, not a generated artifact: it exercises these
// invariants every gate run via the positive trace and the refuted negative
// control named here (see `contracts/scenarios/*_invalid.scenario.yaml`).
const REPLAY: &[(&str, &str)] = &[
    ("I-001", "execution_without_barrier_invalid"),
    ("I-002", "observed_without_execution_invalid"),
    ("I-003", "projection_without_anchor_invalid"),
    ("I-003", "projection_anchor_wrong_fact_invalid"),
    ("I-003", "projection_anchor_wrong_scope_invalid"),
    ("I-006", "conflicting_leases_invalid"),
    ("I-007", "drain_with_active_lease_invalid"),
    ("I-008", "event_after_closed_invalid"),
    ("I-009", "approval_wrong_plan_invalid"),
    ("I-009", "approval_wrong_impact_invalid"),
    ("I-009", "witness_wrong_scope_invalid"),
];

fn candidate_checks(target: FormalTarget) -> &'static [(&'static str, &'static str)] {
    match target {
        FormalTarget::Alloy => ALLOY,
        FormalTarget::P => P,
        FormalTarget::Kani => KANI,
        FormalTarget::Verus => VERUS,
        FormalTarget::Lean4 => LEAN4,
    }
}

/// The invariant ids a target's obligation table actually covers, intersected
/// with the bundle/scenario-declared set (`ir.invariants`), in stable display
/// order. Unlike the bundle-wide declared set (identical across every lane),
/// this is honest about which invariants *this* lane carries checks for.
#[must_use]
pub(crate) fn projected_invariants(target: FormalTarget, declared: &[String]) -> Vec<String> {
    let table = candidate_checks(target);
    ALL_INVARIANTS
        .iter()
        .filter(|inv| declared.iter().any(|d| d == *inv))
        .filter(|inv| table.iter().any(|(i, _)| i == *inv))
        .map(|inv| (*inv).to_owned())
        .collect()
}

fn to_obligations(pairs: &[(&str, &str)]) -> Vec<ReceiptObligation> {
    pairs
        .iter()
        .map(|(invariant, check)| ReceiptObligation {
            invariant_id: (*invariant).to_owned(),
            check_id: (*check).to_owned(),
        })
        .collect()
}

/// The obligations a target's generator emits that are **actually present** in
/// `text`. Conditional or renamed checks are dropped rather than claimed.
#[must_use]
pub(crate) fn present_obligations(target: FormalTarget, text: &str) -> Vec<ReceiptObligation> {
    candidate_checks(target)
        .iter()
        .filter(|(_, check)| text.contains(check))
        .map(|(invariant, check)| ReceiptObligation {
            invariant_id: (*invariant).to_owned(),
            check_id: (*check).to_owned(),
        })
        .collect()
}

/// All candidate obligations for a generated target (used by the canonical
/// declared matrix; the binary prefers the per-artifact [`present_obligations`]).
#[must_use]
pub(crate) fn all_obligations(target: FormalTarget) -> Vec<ReceiptObligation> {
    to_obligations(candidate_checks(target))
}

/// Replay-lane obligations, grounded in the gate-run negative controls.
#[must_use]
pub(crate) fn replay_obligations() -> Vec<ReceiptObligation> {
    to_obligations(REPLAY)
}
