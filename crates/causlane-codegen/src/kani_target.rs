//! Kani bounded-harness generation (split from `targets.rs` for the
//! 800-line file cap).
//!
//! Generates `#[kani::proof]` harnesses that exercise the REAL core
//! validators (capability / lease / lifecycle / overlay / constraint-update
//! truth tables) under bounded `kani::any()`, so a regression in a kernel
//! rule is a counterexample.

use crate::targets::push_rust_contract_summary;
use crate::{artifact_header, CodegenError, FormalIr, FormalTarget, GeneratedArtifact};

/// Generate Kani harnesses from Formal IR.
///
/// # Errors
/// Currently infallible; returns [`CodegenError`] for future target validation.
#[must_use = "the generated Kani harness must be written or checked"]
pub fn generate_kani_harness(ir: &FormalIr) -> Result<GeneratedArtifact, CodegenError> {
    let mut text = artifact_header(ir, FormalTarget::Kani, "harness");
    text.push_str("#![allow(dead_code)]\n\n");
    text.push_str("use causlane_core::{capability_binding_matches, claim_modes_conflict, lifecycle_class_for_profile, mergeable, projection_anchor_source_is_observed, reduce_lifecycle, route_consistent_with_profile, ActionId, AuditEventId, AuditEventKind, ClaimMode, ConstraintEpoch, ConsequenceProfile, CommittedTruth, ConstraintUpdate, DrainFenceCheck, ExecutionBarrier, ImpactSetHash, LeaseId, LeaseRef, LifecycleClass, LifecycleStage, ObligationSet, PlanHash, PlanHashError, ResourceId, Scope, Timestamp, WitnessBinding, WitnessKind, WitnessRef};\n");
    push_kani_helpers(&mut text);
    push_kani_harnesses(&mut text);
    text.push_str("fn main() {}\n\n");
    push_rust_contract_summary(&mut text, ir);
    Ok(GeneratedArtifact::new(
        FormalTarget::Kani,
        "harness",
        ir,
        text,
    ))
}

fn push_kani_helpers(text: &mut String) {
    text.push_str(
        r#"fn plan_hash_a() -> Result<PlanHash, PlanHashError> {
    PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
}

fn plan_hash_b() -> Result<PlanHash, PlanHashError> {
    PlanHash::new("sha256:2222222222222222222222222222222222222222222222222222222222222222")
}

fn lease(
    id: &str,
    plan: &PlanHash,
    resource: &str,
    scope: &str,
    mode: ClaimMode,
    op_index: Option<u32>,
    expires_at: Option<Timestamp>,
) -> LeaseRef {
    LeaseRef {
        lease_id: LeaseId(id.to_owned()),
        resource: ResourceId(resource.to_owned()),
        scope: Scope(scope.to_owned()),
        mode,
        amount: 1,
        holder_action_id: ActionId("act".to_owned()),
        holder_plan_hash: plan.clone(),
        holder_op_index: op_index,
        epoch: ConstraintEpoch(0),
        expires_at,
        lease_event_id: AuditEventId(format!("evt_{id}")),
    }
}

fn barrier(plan: &PlanHash) -> ExecutionBarrier {
    ExecutionBarrier {
        barrier_id: AuditEventId("barrier".to_owned()),
        action_id: ActionId("act".to_owned()),
        plan_hash: plan.clone(),
        op_indexes: vec![0],
        impact_set_hash: ImpactSetHash("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
        witnesses: vec![],
        leases: vec![lease("lease_a", plan, "release_candidate_write", "candidate:123", ClaimMode::ExclusiveWrite, Some(0), None)],
        authz_decision_refs: vec![],
        constraint_snapshot_id: None,
    }
}

"#,
    );
}

#[allow(clippy::too_many_lines)]
fn push_kani_harnesses(text: &mut String) {
    text.push_str(
        r#"#[cfg(kani)]
#[kani::proof]
fn lifecycle_reducer_forbidden_transitions_fail_closed() {
    assert_eq!(
        reduce_lifecycle(LifecycleStage::DispatchLogged, AuditEventKind::ExecutionBarrierLogged, ConsequenceProfile::RuntimeExecution),
        Ok(LifecycleStage::ExecutionBarrierLogged)
    );
    assert_eq!(
        reduce_lifecycle(LifecycleStage::ExecutionBarrierLogged, AuditEventKind::ExecutionStarted, ConsequenceProfile::RuntimeExecution),
        Ok(LifecycleStage::Executing)
    );
    assert_eq!(
        reduce_lifecycle(LifecycleStage::Executing, AuditEventKind::ObservedTruthCommitted, ConsequenceProfile::RuntimeExecution),
        Ok(LifecycleStage::Observed)
    );
    assert_eq!(
        reduce_lifecycle(LifecycleStage::Observed, AuditEventKind::ProjectionEmitted, ConsequenceProfile::RuntimeExecution),
        Ok(LifecycleStage::Projected)
    );
    assert!(reduce_lifecycle(LifecycleStage::DispatchLogged, AuditEventKind::ExecutionStarted, ConsequenceProfile::RuntimeExecution).is_err());
    assert!(reduce_lifecycle(LifecycleStage::DispatchLogged, AuditEventKind::ObservedTruthCommitted, ConsequenceProfile::RuntimeExecution).is_err());
    assert!(reduce_lifecycle(LifecycleStage::Closed, AuditEventKind::ExecutionStarted, ConsequenceProfile::RuntimeExecution).is_err());
    assert!(reduce_lifecycle(LifecycleStage::DispatchLogged, AuditEventKind::ObservedTruthCommitted, ConsequenceProfile::ProjectionRead).is_err());
}

#[cfg(kani)]
#[kani::proof]
fn capability_binding_rule_is_fail_closed() {
    let matches_barrier = true;
    let matches_action = true;
    let matches_plan = true;
    let covered_op = true;
    let has_leases = true;
    assert!(capability_binding_matches(
        matches_barrier,
        matches_action,
        matches_plan,
        covered_op,
        has_leases,
    ));
    let wrong_barrier = false;
    assert!(!capability_binding_matches(
        wrong_barrier,
        matches_action,
        matches_plan,
        covered_op,
        has_leases,
    ));
    let wrong_action = false;
    assert!(!capability_binding_matches(
        matches_barrier,
        wrong_action,
        matches_plan,
        covered_op,
        has_leases,
    ));
    let wrong_plan = false;
    assert!(!capability_binding_matches(
        matches_barrier,
        matches_action,
        wrong_plan,
        covered_op,
        has_leases,
    ));
    let uncovered_op = false;
    assert!(!capability_binding_matches(
        matches_barrier,
        matches_action,
        matches_plan,
        uncovered_op,
        has_leases,
    ));
    let no_leases = false;
    assert!(!capability_binding_matches(
        matches_barrier,
        matches_action,
        matches_plan,
        covered_op,
        no_leases,
    ));
}

#[cfg(kani)]
#[kani::proof]
fn plan_hash_validation_and_merge_default_are_fail_closed() {
    let Ok(_plan) = plan_hash_b() else {
        assert!(false);
        return;
    };
    assert!(PlanHash::new("sha256:TODO").is_err());
    assert!(PlanHash::new("sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").is_err());
    assert!(!mergeable());
}

#[cfg(kani)]
#[kani::proof]
fn lease_conflict_rule_is_fail_closed_without_verified_merge() {
    assert!(claim_modes_conflict(
        ClaimMode::ExclusiveWrite,
        ClaimMode::SharedRead,
        true,
        true,
        false,
    ));
    assert!(claim_modes_conflict(
        ClaimMode::SharedRead,
        ClaimMode::ExclusiveWrite,
        true,
        true,
        false,
    ));
    assert!(!claim_modes_conflict(
        ClaimMode::SharedRead,
        ClaimMode::SharedRead,
        true,
        true,
        false,
    ));
    assert!(!claim_modes_conflict(
        ClaimMode::ExclusiveWrite,
        ClaimMode::ExclusiveWrite,
        true,
        true,
        true,
    ));
    assert!(!claim_modes_conflict(
        ClaimMode::ExclusiveWrite,
        ClaimMode::ExclusiveWrite,
        false,
        true,
        false,
    ));
}

#[cfg(kani)]
#[kani::proof]
fn projection_anchor_source_kind_is_observed_truth_only() {
    assert!(projection_anchor_source_is_observed(
        AuditEventKind::ObservedTruthCommitted
    ));
    assert!(!projection_anchor_source_is_observed(
        AuditEventKind::ExecutionStarted
    ));
    assert!(!projection_anchor_source_is_observed(
        AuditEventKind::ProjectionEmitted
    ));
    assert!(!projection_anchor_source_is_observed(
        AuditEventKind::GateApproved
    ));
}

#[cfg(kani)]
#[kani::proof]
fn witness_binding_is_exact_for_action_plan_and_impact() {
    let Ok(plan) = plan_hash_a() else {
        assert!(false);
        return;
    };
    let Ok(other_plan) = plan_hash_b() else {
        assert!(false);
        return;
    };
    let impact = ImpactSetHash("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned());
    let binding = WitnessBinding {
        action_id: ActionId("act".to_owned()),
        plan_hash: plan.clone(),
        impact_set_hash: Some(impact.clone()),
    };
    let witness = WitnessRef {
        event_id: AuditEventId("approval".to_owned()),
        requirement_id: "readiness_before_promotion".to_owned(),
        kind: WitnessKind::GateApproval,
        fact_kind: None,
        scope: Some(Scope("candidate:123".to_owned())),
        binds_to: Some(binding.clone()),
    };
    let Some(actual) = witness.binds_to else {
        assert!(false);
        return;
    };
    assert_eq!(actual.action_id, ActionId("act".to_owned()));
    assert_eq!(actual.plan_hash, plan);
    assert_eq!(actual.impact_set_hash, Some(impact));
    assert_ne!(actual.plan_hash, other_plan);
}

#[cfg(kani)]
fn nondet_claim_mode() -> ClaimMode {
    if kani::any() {
        ClaimMode::ExclusiveWrite
    } else {
        ClaimMode::SharedRead
    }
}

#[cfg(kani)]
#[kani::proof]
fn capability_binding_is_exact_conjunction_nondet() {
    // Bounded nondeterminism (kani::any) over the full 2^5 binding-fact space.
    let barrier_event_matches: bool = kani::any();
    let action_matches: bool = kani::any();
    let plan_matches: bool = kani::any();
    let op_index_covered: bool = kani::any();
    let leases_present: bool = kani::any();
    let accepted = capability_binding_matches(
        barrier_event_matches,
        action_matches,
        plan_matches,
        op_index_covered,
        leases_present,
    );
    // I-001 fail-closed: a capability is accepted IFF every binding fact holds.
    // Dropping any conjunct (e.g. no longer checking the plan hash) is a
    // counterexample, so this proof is mutation-sensitive.
    assert_eq!(
        accepted,
        barrier_event_matches && action_matches && plan_matches && op_index_covered && leases_present
    );
}

#[cfg(kani)]
#[kani::proof]
fn claim_modes_conflict_matches_truth_table_nondet() {
    // Bounded nondeterminism over the mode x flag space.
    let left = nondet_claim_mode();
    let right = nondet_claim_mode();
    let same_resource: bool = kani::any();
    let same_scope: bool = kani::any();
    let verified_merge: bool = kani::any();
    let conflict = claim_modes_conflict(left, right, same_resource, same_scope, verified_merge);
    // I-006: exclusive writes on the same resource+scope conflict unless a
    // verified merge protocol applies — checked over the whole bounded space.
    let expected = same_resource
        && same_scope
        && (left == ClaimMode::ExclusiveWrite || right == ClaimMode::ExclusiveWrite)
        && !verified_merge;
    assert_eq!(conflict, expected);
}

#[cfg(kani)]
fn nondet_stage() -> LifecycleStage {
    match kani::any::<u8>() {
        0 => LifecycleStage::New,
        1 => LifecycleStage::Admitted,
        2 => LifecycleStage::Planned,
        3 => LifecycleStage::DispatchLogged,
        4 => LifecycleStage::ExecutionBarrierLogged,
        5 => LifecycleStage::Executing,
        6 => LifecycleStage::Observed,
        7 => LifecycleStage::Projected,
        _rest => LifecycleStage::Closed,
    }
}

#[cfg(kani)]
fn nondet_event() -> AuditEventKind {
    match kani::any::<u8>() {
        0 => AuditEventKind::ActionAdmitted,
        1 => AuditEventKind::ActionPlanned,
        2 => AuditEventKind::DispatchLogged,
        3 => AuditEventKind::ExecutionBarrierLogged,
        4 => AuditEventKind::ExecutionStarted,
        5 => AuditEventKind::ExecutionCompleted,
        6 => AuditEventKind::ObservedTruthCommitted,
        7 => AuditEventKind::ProjectionEmitted,
        8 => AuditEventKind::LifecycleClosed,
        9 => AuditEventKind::GateApproved,
        10 => AuditEventKind::GateDenied,
        11 => AuditEventKind::ConstraintLeaseGranted,
        12 => AuditEventKind::ConstraintLeaseReleased,
        13 => AuditEventKind::ViolationDetected,
        _rest => AuditEventKind::AuthzDecisionRecorded,
    }
}

#[cfg(kani)]
fn nondet_profile() -> ConsequenceProfile {
    match kani::any::<u8>() {
        0 => ConsequenceProfile::RuntimeExecution,
        1 => ConsequenceProfile::ProjectionRead,
        2 => ConsequenceProfile::OversightMeta,
        3 => ConsequenceProfile::TopologyMeta,
        4 => ConsequenceProfile::EvidenceMeta,
        _rest => ConsequenceProfile::OutsideKernel,
    }
}

// I-002 (bounded nondet over the real reduce_lifecycle): observed truth is
// accepted by the grammar ONLY from `Executing` under RuntimeExecution — never
// without a prior execution. Dropping that predecessor guard is a counterexample.
#[cfg(kani)]
#[kani::proof]
fn observed_truth_requires_prior_execution_nondet() {
    let stage = nondet_stage();
    let profile = nondet_profile();
    let outcome = reduce_lifecycle(stage, AuditEventKind::ObservedTruthCommitted, profile);
    if outcome.is_ok() {
        assert!(stage == LifecycleStage::Executing);
        assert!(profile == ConsequenceProfile::RuntimeExecution);
    }
}

// I-001 (bounded nondet): execution is accepted ONLY from ExecutionBarrierLogged
// under RuntimeExecution — never without a prior barrier.
#[cfg(kani)]
#[kani::proof]
fn execution_requires_prior_barrier_nondet() {
    let stage = nondet_stage();
    let profile = nondet_profile();
    let outcome = reduce_lifecycle(stage, AuditEventKind::ExecutionStarted, profile);
    if outcome.is_ok() {
        assert!(stage == LifecycleStage::ExecutionBarrierLogged);
        assert!(profile == ConsequenceProfile::RuntimeExecution);
    }
}

// I-008 (bounded nondet): `Closed` is terminal — the grammar rejects EVERY event
// from the closed stage, for every profile.
#[cfg(kani)]
#[kani::proof]
fn closed_stage_is_terminal_nondet() {
    let event = nondet_event();
    let profile = nondet_profile();
    let outcome = reduce_lifecycle(LifecycleStage::Closed, event, profile);
    assert!(outcome.is_err());
}

#[cfg(kani)]
fn nondet_obligation_set() -> ObligationSet {
    ObligationSet {
        requires_witness: kani::any(),
        requires_claim: kani::any(),
        requires_authz: kani::any(),
        requires_barrier: kani::any(),
        requires_anchor: kani::any(),
    }
}

// I-004 (bounded nondet over the real ObligationSet::preserved_by): an overlay
// preserves obligations IFF every category the base requires is still required
// after overlay — the overlaid set is a superset, never weaker. Dropping any
// conjunct from the rule is a counterexample (mutation-sensitive).
#[cfg(kani)]
#[kani::proof]
fn overlay_never_weakens_obligations_nondet() {
    let base = nondet_obligation_set();
    let overlaid = nondet_obligation_set();
    let preserved = base.preserved_by(overlaid);
    let expected = (!base.requires_witness || overlaid.requires_witness)
        && (!base.requires_claim || overlaid.requires_claim)
        && (!base.requires_authz || overlaid.requires_authz)
        && (!base.requires_barrier || overlaid.requires_barrier)
        && (!base.requires_anchor || overlaid.requires_anchor);
    assert_eq!(preserved, expected);
}

#[cfg(kani)]
fn nondet_committed_truth() -> CommittedTruth {
    CommittedTruth {
        readiness_committed: kani::any(),
        promotion_committed: kani::any(),
        evidence_committed: kani::any(),
    }
}

#[cfg(kani)]
fn nondet_constraint_update() -> ConstraintUpdate {
    ConstraintUpdate {
        rewrites_readiness: kani::any(),
        rewrites_promotion: kani::any(),
        rewrites_evidence: kani::any(),
    }
}

// I-010 (bounded nondet over the real ConstraintUpdate::preserves_committed_truth):
// a constraint update is valid IFF it rewrites no truth category that is already
// committed — committed observed truth is immutable. Dropping any conjunct from
// the rule is a counterexample (mutation-sensitive).
#[cfg(kani)]
#[kani::proof]
fn constraint_update_cannot_rewrite_committed_truth_nondet() {
    let committed = nondet_committed_truth();
    let update = nondet_constraint_update();
    let preserved = update.preserves_committed_truth(committed);
    let expected = (!update.rewrites_readiness || !committed.readiness_committed)
        && (!update.rewrites_promotion || !committed.promotion_committed)
        && (!update.rewrites_evidence || !committed.evidence_committed);
    assert_eq!(preserved, expected);
}

#[cfg(kani)]
fn nondet_lifecycle_class() -> LifecycleClass {
    match kani::any::<u8>() {
        0 => LifecycleClass::ExecutionBearing,
        1 => LifecycleClass::ProjectionOnly,
        _rest => LifecycleClass::Meta,
    }
}

// I-005 (bounded nondet over the real route_consistent_with_profile): a route's
// lifecycle class is allowed for a profile IFF it is exactly the class the
// profile routes through. Quantifies every (class, profile) pair; the `expected`
// restates the canonical mapping, so mutating lifecycle_class_for_profile is a
// counterexample.
#[cfg(kani)]
#[kani::proof]
fn route_is_allowed_only_for_matching_profile_nondet() {
    let class = nondet_lifecycle_class();
    let profile = nondet_profile();
    let consistent = route_consistent_with_profile(class, profile);
    let expected_class = match profile {
        ConsequenceProfile::RuntimeExecution => LifecycleClass::ExecutionBearing,
        ConsequenceProfile::ProjectionRead => LifecycleClass::ProjectionOnly,
        ConsequenceProfile::OversightMeta
        | ConsequenceProfile::TopologyMeta
        | ConsequenceProfile::EvidenceMeta
        | ConsequenceProfile::OutsideKernel => LifecycleClass::Meta,
    };
    assert_eq!(consistent, class == expected_class);
    // The helper and the mapping function agree.
    assert!(lifecycle_class_for_profile(profile) == expected_class);
}

#[cfg(kani)]
fn nondet_drain_fence_check() -> DrainFenceCheck {
    DrainFenceCheck {
        left_overlaps: kani::any(),
        left_active: kani::any(),
        left_expired: kani::any(),
        right_overlaps: kani::any(),
        right_active: kani::any(),
        right_expired: kani::any(),
    }
}

// I-007 (bounded nondet over the real DrainFenceCheck::fence_acquirable): a drain
// fence is acquirable IFF no lease slot is overlapping the fence scope while still
// active and not-yet-expired — an expired lease no longer blocks the drain.
// Dropping a conjunct is a counterexample (mutation-sensitive).
#[cfg(kani)]
#[kani::proof]
fn drain_fence_acquirable_only_without_active_overlap_nondet() {
    let check = nondet_drain_fence_check();
    let acquirable = check.fence_acquirable();
    let expected = (!check.left_overlaps || !check.left_active || check.left_expired)
        && (!check.right_overlaps || !check.right_active || check.right_expired);
    assert_eq!(acquirable, expected);
}

"#,
    );
}
