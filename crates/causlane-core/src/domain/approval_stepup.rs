//! Step-up, separation-of-duties, and approval freshness (M06.5, S06).
//!
//! A strict, fail-closed layer over the coordinate-only approval decider in
//! [`crate::domain::approval`]. M06.4 answers *is this target bound by an approve and
//! not a deny*; M06.5 additionally demands that the authorizing approve come from an
//! actor **distinct** from the action initiator (separation-of-duties — no
//! self-approval, per `docs/07-security-and-authz.md`), at sufficient **assurance**
//! (step-up), and **fresh** at the barrier evaluation time `now` (mirroring the
//! `classify_authz_decision` freshness checks, ADR-0011).
//!
//! The central safety invariant: [`classify_approval_stepup`] returns `Ok(())`
//! **only** for an exactly-bound `gate.approved` that is *also* distinct-actor,
//! sufficiently assured, and fresh — so [`approval_gate_stepup`] can treat `Ok(())` as
//! a fully qualified authorization and a lone self-approval / under-assured / stale
//! approve can never authorize. Like the M06.4 gate it accumulates then decides, so the
//! outcome is independent of approval order.
//!
//! This module is non-formal-bound (no codegen / formal references): it adds the
//! [`AssuranceLevel`] and [`ApprovalRef`] step-up attributes and these two functions
//! without touching any formal-bound type.

use super::{
    ActionId, ApprovalDenyReason, ApprovalOutcome, ApprovalRef, ApprovalVerb, AssuranceLevel,
    ImpactSetHash, PlanHash, Timestamp,
};

/// What a barrier demands of the approvals authorizing a target (M06.5): the three
/// coordinates plus the step-up / separation-of-duties / freshness bounds, all judged
/// at `now`. Bundled into one borrow so [`approval_gate_stepup`] stays a two-argument
/// function (the coordinates, `initiator`, `required_assurance`, `max_age`, and `now`
/// always travel together as what this barrier requires).
#[derive(Clone, Copy, Debug)]
pub struct ApprovalRequirement<'a> {
    /// The action an authorizing approval must be bound to.
    pub action: &'a ActionId,
    /// The plan an authorizing approval must be bound to.
    pub plan: &'a PlanHash,
    /// The canonical planned-impact set an authorizing approval must be bound to.
    pub impact_set: &'a ImpactSetHash,
    /// The action initiator; an exactly-bound `gate.approved` from this actor is a
    /// self-approval and authorizes nothing ([`ApprovalDenyReason::SelfApproval`]).
    pub initiator: &'a str,
    /// The minimum assurance an authorizing approval must present (step-up).
    pub required_assurance: AssuranceLevel,
    /// The maximum age (at `now`) of an authorizing approval before it is stale.
    pub max_age: u64,
    /// The barrier evaluation time freshness is judged against.
    pub now: Timestamp,
}

/// Classify one approval against the full M06.5 requirement. Checks run
/// coordinate-binding (action → plan → impact, the same skeleton as
/// [`classify_approval`](crate::domain::classify_approval)) → verb. An exactly-bound
/// `gate.denied` is [`ApprovalDenyReason::ExplicitDeny`] regardless of the M06.5
/// attributes (deny-wins is unconditional). An exactly-bound `gate.approved` must then
/// clear, in order, separation-of-duties (`actor != initiator`), step-up
/// (`assurance >= required_assurance`), and freshness (not forward-dated, not expired,
/// not stale — mirroring the `classify_authz_decision` checks). It returns `Ok(())`
/// **only** when the approval is exactly bound, from a distinct actor, sufficiently
/// assured, and fresh — so the gate can treat `Ok(())` as a fully qualified
/// authorization. Never returns `Missing` (an aggregate of [`approval_gate_stepup`]).
pub fn classify_approval_stepup(
    approval: &ApprovalRef,
    req: &ApprovalRequirement<'_>,
) -> Result<(), ApprovalDenyReason> {
    if approval.action_id != *req.action {
        return Err(ApprovalDenyReason::WrongAction);
    }
    if approval.plan_hash != *req.plan {
        return Err(ApprovalDenyReason::WrongPlan);
    }
    if approval.impact_set_hash != *req.impact_set {
        return Err(ApprovalDenyReason::WrongImpactSet);
    }
    // Deny-wins is unconditional on an exact binding; the M06.5 attributes of a
    // `gate.denied` are irrelevant.
    if approval.verdict == ApprovalVerb::Deny {
        return Err(ApprovalDenyReason::ExplicitDeny);
    }
    // Exactly-bound `gate.approved`: SoD, then step-up, then freshness.
    if approval.actor == req.initiator {
        return Err(ApprovalDenyReason::SelfApproval);
    }
    if approval.assurance < req.required_assurance {
        return Err(ApprovalDenyReason::InsufficientAssurance);
    }
    // Freshness mirrors `classify_authz_decision` (ADR-0011), `Some(now)` branch: the
    // gate always has a concrete `now`, so there is no born-expired (`None`) case.
    if approval.issued_at.0 > req.now.0 {
        return Err(ApprovalDenyReason::IssuedAfter);
    }
    if approval
        .expires_at
        .is_some_and(|expiry| expiry.0 <= req.now.0)
    {
        return Err(ApprovalDenyReason::Expired);
    }
    if req.now.0.saturating_sub(approval.issued_at.0) > req.max_age {
        return Err(ApprovalDenyReason::Stale);
    }
    Ok(())
}

/// Fail-closed combined gate (M06.5). A target is authorized iff some recorded approval
/// is an exactly-bound `gate.approved` from an actor **distinct** from `req.initiator`,
/// presenting assurance `>= req.required_assurance`, and fresh at `req.now` (not
/// forward-dated, expired, or stale), **and** no approval is an exactly-bound
/// `gate.denied` (deny-wins). Anything else denies.
///
/// Like [`approval_gate`](crate::domain::approval_gate) it accumulates every approval's
/// classification then decides, so the outcome is **independent of the order** of
/// `approvals`. Crucially `saw_approve` is set **only** on a fully qualified
/// [`classify_approval_stepup`] `Ok(())`, so a lone self-approval, under-assured, or
/// stale approve can never authorize. Precedence (deny-wins first, then a qualified
/// approve, then the most actionable refusal): `ExplicitDeny` ≻ `Approved` ≻
/// `SelfApproval` ≻ `InsufficientAssurance` ≻ `IssuedAfter` ≻ `Expired` ≻ `Stale` ≻
/// `WrongAction` ≻ `WrongPlan` ≻ `WrongImpactSet` ≻ `Missing`.
#[must_use]
pub fn approval_gate_stepup(
    approvals: &[ApprovalRef],
    req: &ApprovalRequirement<'_>,
) -> ApprovalOutcome {
    let mut saw_explicit_deny = false;
    let mut saw_approve = false;
    let mut saw_self_approval = false;
    let mut saw_insufficient = false;
    let mut saw_issued_after = false;
    let mut saw_expired = false;
    let mut saw_stale = false;
    let mut saw_wrong_action = false;
    let mut saw_wrong_plan = false;
    let mut saw_wrong_impact = false;
    for approval in approvals {
        match classify_approval_stepup(approval, req) {
            Ok(()) => saw_approve = true,
            Err(ApprovalDenyReason::ExplicitDeny) => saw_explicit_deny = true,
            Err(ApprovalDenyReason::SelfApproval) => saw_self_approval = true,
            Err(ApprovalDenyReason::InsufficientAssurance) => saw_insufficient = true,
            Err(ApprovalDenyReason::IssuedAfter) => saw_issued_after = true,
            Err(ApprovalDenyReason::Expired) => saw_expired = true,
            Err(ApprovalDenyReason::Stale) => saw_stale = true,
            Err(ApprovalDenyReason::WrongAction) => saw_wrong_action = true,
            Err(ApprovalDenyReason::WrongPlan) => saw_wrong_plan = true,
            Err(ApprovalDenyReason::WrongImpactSet) => saw_wrong_impact = true,
            // `classify_approval_stepup` never returns `Missing`; this aggregate-only
            // reason is decided below.
            Err(ApprovalDenyReason::Missing) => {}
        }
    }
    if saw_explicit_deny {
        return ApprovalOutcome::Denied(ApprovalDenyReason::ExplicitDeny);
    }
    if saw_approve {
        return ApprovalOutcome::Approved;
    }
    let reason = if saw_self_approval {
        ApprovalDenyReason::SelfApproval
    } else if saw_insufficient {
        ApprovalDenyReason::InsufficientAssurance
    } else if saw_issued_after {
        ApprovalDenyReason::IssuedAfter
    } else if saw_expired {
        ApprovalDenyReason::Expired
    } else if saw_stale {
        ApprovalDenyReason::Stale
    } else if saw_wrong_action {
        ApprovalDenyReason::WrongAction
    } else if saw_wrong_plan {
        ApprovalDenyReason::WrongPlan
    } else if saw_wrong_impact {
        ApprovalDenyReason::WrongImpactSet
    } else {
        ApprovalDenyReason::Missing
    };
    ApprovalOutcome::Denied(reason)
}

#[cfg(test)]
mod tests {
    use super::{
        approval_gate_stepup, classify_approval_stepup, ApprovalDenyReason, ApprovalOutcome,
        ApprovalRef, ApprovalRequirement, ApprovalVerb, AssuranceLevel,
    };
    use crate::domain::{ActionId, AuditEventId, ImpactSetHash, Timestamp};
    use crate::{PlanHash, PlanHashError};

    const ACT: &str = "release.promote_candidate";
    const OTHER_ACT: &str = "release.rollback";
    const IMPACT: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OTHER_IMPACT: &str =
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HASH1: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
    const HASH2: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";

    /// M06.5 builder varying every step-up / `SoD` / freshness axis.
    #[allow(clippy::too_many_arguments)]
    fn appr_full(
        verb: ApprovalVerb,
        action: &str,
        plan: &PlanHash,
        impact: &str,
        actor: &str,
        issued_at: u64,
        expires_at: Option<u64>,
        assurance: u8,
    ) -> ApprovalRef {
        ApprovalRef {
            approval_event_id: AuditEventId("evt".to_owned()),
            verdict: verb,
            action_id: ActionId(action.to_owned()),
            plan_hash: plan.clone(),
            impact_set_hash: ImpactSetHash(impact.to_owned()),
            actor: actor.to_owned(),
            issued_at: Timestamp(issued_at),
            expires_at: expires_at.map(Timestamp),
            assurance: AssuranceLevel(assurance),
        }
    }

    /// Requirement with the shared M06.5 test bounds: initiator `alice`, required
    /// assurance 5, `max_age` 10, `now` 100.
    fn stepup_req<'a>(
        action: &'a ActionId,
        plan: &'a PlanHash,
        impact: &'a ImpactSetHash,
    ) -> ApprovalRequirement<'a> {
        ApprovalRequirement {
            action,
            plan,
            impact_set: impact,
            initiator: "alice",
            required_assurance: AssuranceLevel(5),
            max_age: 10,
            now: Timestamp(100),
        }
    }

    /// The M06.5 grid: a coordinate family (drives Wrong*/ExplicitDeny/Approved) plus
    /// an exact-bound family varying actor, assurance, and the temporal boundaries.
    fn stepup_configs(plan1: &PlanHash, plan2: &PlanHash) -> Vec<ApprovalRef> {
        let mut configs = Vec::new();
        for act in [ACT, OTHER_ACT] {
            for plan in [plan1, plan2] {
                for imp in [IMPACT, OTHER_IMPACT] {
                    for verb in [ApprovalVerb::Approve, ApprovalVerb::Deny] {
                        configs.push(appr_full(verb, act, plan, imp, "bob", 95, None, 5));
                    }
                }
            }
        }
        // now=100, max_age=10: fresh / age==max / age==max+1 / future / now==issued /
        // expiry<=now / expiry<now / expiry>now.
        let temporal: [(u64, Option<u64>); 8] = [
            (95, None),
            (90, None),
            (89, None),
            (101, None),
            (100, None),
            (95, Some(100)),
            (98, Some(99)),
            (95, Some(101)),
        ];
        for actor in ["alice", "bob"] {
            for assurance in [4u8, 5, 6] {
                for (issued, expires) in temporal {
                    configs.push(appr_full(
                        ApprovalVerb::Approve,
                        ACT,
                        plan1,
                        IMPACT,
                        actor,
                        issued,
                        expires,
                        assurance,
                    ));
                }
            }
        }
        configs
    }

    /// Independent oracle for the M06.5 gate: membership `.any()` queries with inline
    /// predicates (never delegating to `classify_approval_stepup`), so a shared bug
    /// cannot hide. `Approved` requires the full conjunction exact ∧ distinct ∧ assured
    /// ∧ fresh; refusal reasons are queried in gate-precedence order.
    fn reference_gate_stepup(
        approvals: &[ApprovalRef],
        req: &ApprovalRequirement<'_>,
    ) -> ApprovalOutcome {
        let exact = |a: &ApprovalRef| {
            a.action_id == *req.action
                && a.plan_hash == *req.plan
                && a.impact_set_hash == *req.impact_set
        };
        let approve_exact = |a: &ApprovalRef| exact(a) && a.verdict == ApprovalVerb::Approve;
        let distinct_assured =
            |a: &ApprovalRef| a.actor != req.initiator && a.assurance >= req.required_assurance;
        let not_future = |a: &ApprovalRef| a.issued_at.0 <= req.now.0;
        let expired = |a: &ApprovalRef| a.expires_at.is_some_and(|e| e.0 <= req.now.0);
        let stale = |a: &ApprovalRef| req.now.0.saturating_sub(a.issued_at.0) > req.max_age;
        let fresh = |a: &ApprovalRef| not_future(a) && !expired(a) && !stale(a);

        if approvals
            .iter()
            .any(|a| exact(a) && a.verdict == ApprovalVerb::Deny)
        {
            return ApprovalOutcome::Denied(ApprovalDenyReason::ExplicitDeny);
        }
        if approvals
            .iter()
            .any(|a| approve_exact(a) && distinct_assured(a) && fresh(a))
        {
            return ApprovalOutcome::Approved;
        }
        let reason = if approvals
            .iter()
            .any(|a| approve_exact(a) && a.actor == req.initiator)
        {
            ApprovalDenyReason::SelfApproval
        } else if approvals.iter().any(|a| {
            approve_exact(a) && a.actor != req.initiator && a.assurance < req.required_assurance
        }) {
            ApprovalDenyReason::InsufficientAssurance
        } else if approvals
            .iter()
            .any(|a| approve_exact(a) && distinct_assured(a) && !not_future(a))
        {
            ApprovalDenyReason::IssuedAfter
        } else if approvals
            .iter()
            .any(|a| approve_exact(a) && distinct_assured(a) && not_future(a) && expired(a))
        {
            ApprovalDenyReason::Expired
        } else if approvals.iter().any(|a| {
            approve_exact(a) && distinct_assured(a) && not_future(a) && !expired(a) && stale(a)
        }) {
            ApprovalDenyReason::Stale
        } else if approvals.iter().any(|a| a.action_id != *req.action) {
            ApprovalDenyReason::WrongAction
        } else if approvals
            .iter()
            .any(|a| a.action_id == *req.action && a.plan_hash != *req.plan)
        {
            ApprovalDenyReason::WrongPlan
        } else if approvals.iter().any(|a| {
            a.action_id == *req.action
                && a.plan_hash == *req.plan
                && a.impact_set_hash != *req.impact_set
        }) {
            ApprovalDenyReason::WrongImpactSet
        } else {
            ApprovalDenyReason::Missing
        };
        ApprovalOutcome::Denied(reason)
    }

    /// Load-bearing property (M06.5): across the grid of coordinates × verb × actor ×
    /// assurance × temporal boundaries, `approval_gate_stepup` matches the independent
    /// oracle for every single approval and every pair, is order-independent, and
    /// produces every outcome (non-vacuity). The strict-`Ok` design (C1) means no
    /// self-approval / under-assured / stale input can yield `Approved` — the oracle
    /// equality proves it, not merely per-reason non-vacuity.
    #[test]
    fn approval_gate_stepup_matches_oracle_and_is_order_independent() -> Result<(), PlanHashError> {
        let plan1 = PlanHash::new(HASH1)?;
        let plan2 = PlanHash::new(HASH2)?;
        let action = ActionId(ACT.to_owned());
        let impact = ImpactSetHash(IMPACT.to_owned());
        let req = stepup_req(&action, &plan1, &impact);

        let configs = stepup_configs(&plan1, &plan2);
        let mut outcomes = Vec::new();

        let empty = approval_gate_stepup(&[], &req);
        assert_eq!(empty, ApprovalOutcome::Denied(ApprovalDenyReason::Missing));
        outcomes.push(empty);

        for a in &configs {
            let single = [a.clone()];
            let out = approval_gate_stepup(&single, &req);
            assert_eq!(out, reference_gate_stepup(&single, &req));
            outcomes.push(out);
            for b in &configs {
                let forward = [a.clone(), b.clone()];
                let backward = [b.clone(), a.clone()];
                let out = approval_gate_stepup(&forward, &req);
                assert_eq!(
                    out,
                    approval_gate_stepup(&backward, &req),
                    "approval_gate_stepup is order-dependent"
                );
                assert_eq!(
                    out,
                    reference_gate_stepup(&forward, &req),
                    "approval_gate_stepup diverged from the reference oracle"
                );
                outcomes.push(out);
            }
        }

        let has = |want: &ApprovalOutcome| outcomes.iter().any(|o| o == want);
        let denied = |r| ApprovalOutcome::Denied(r);
        assert!(has(&ApprovalOutcome::Approved), "Approved never produced");
        assert!(
            has(&denied(ApprovalDenyReason::ExplicitDeny)),
            "ExplicitDeny never produced"
        );
        assert!(
            has(&denied(ApprovalDenyReason::SelfApproval)),
            "SelfApproval never produced"
        );
        assert!(
            has(&denied(ApprovalDenyReason::InsufficientAssurance)),
            "InsufficientAssurance never produced"
        );
        assert!(
            has(&denied(ApprovalDenyReason::IssuedAfter)),
            "IssuedAfter never produced"
        );
        assert!(
            has(&denied(ApprovalDenyReason::Expired)),
            "Expired never produced"
        );
        assert!(
            has(&denied(ApprovalDenyReason::Stale)),
            "Stale never produced"
        );
        assert!(
            has(&denied(ApprovalDenyReason::WrongAction)),
            "WrongAction never produced"
        );
        assert!(
            has(&denied(ApprovalDenyReason::WrongPlan)),
            "WrongPlan never produced"
        );
        assert!(
            has(&denied(ApprovalDenyReason::WrongImpactSet)),
            "WrongImpactSet never produced"
        );
        assert!(
            has(&denied(ApprovalDenyReason::Missing)),
            "Missing never produced"
        );
        Ok(())
    }

    // C1: `Ok(())` means fully qualified — exact-bound, distinct actor, assured, fresh.
    #[test]
    fn stepup_classifier_ok_means_fully_qualified() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH1)?;
        let action = ActionId(ACT.to_owned());
        let impact = ImpactSetHash(IMPACT.to_owned());
        let r = stepup_req(&action, &plan, &impact);
        let qualified = appr_full(
            ApprovalVerb::Approve,
            ACT,
            &plan,
            IMPACT,
            "bob",
            95,
            None,
            5,
        );
        assert_eq!(classify_approval_stepup(&qualified, &r), Ok(()));
        // Same coordinates, hardware-grade assurance, fresh — but it is the initiator.
        let self_appr = appr_full(
            ApprovalVerb::Approve,
            ACT,
            &plan,
            IMPACT,
            "alice",
            95,
            None,
            9,
        );
        assert_eq!(
            classify_approval_stepup(&self_appr, &r),
            Err(ApprovalDenyReason::SelfApproval)
        );
        Ok(())
    }

    #[test]
    fn qualified_distinct_approve_authorizes() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH1)?;
        let action = ActionId(ACT.to_owned());
        let impact = ImpactSetHash(IMPACT.to_owned());
        let r = stepup_req(&action, &plan, &impact);
        let bob = appr_full(
            ApprovalVerb::Approve,
            ACT,
            &plan,
            IMPACT,
            "bob",
            95,
            None,
            5,
        );
        assert_eq!(approval_gate_stepup(&[bob], &r), ApprovalOutcome::Approved);
        Ok(())
    }

    // C1 witness: a lone self-approval, however assured and fresh, never authorizes.
    #[test]
    fn lone_self_approval_never_authorizes() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH1)?;
        let action = ActionId(ACT.to_owned());
        let impact = ImpactSetHash(IMPACT.to_owned());
        let r = stepup_req(&action, &plan, &impact);
        let alice = appr_full(
            ApprovalVerb::Approve,
            ACT,
            &plan,
            IMPACT,
            "alice",
            95,
            None,
            9,
        );
        assert_eq!(
            approval_gate_stepup(&[alice], &r),
            ApprovalOutcome::Denied(ApprovalDenyReason::SelfApproval)
        );
        Ok(())
    }

    // C1 + H2 witness: an under-assured distinct approve plus a strong self-approval is
    // never `Approved`; SoD outranks insufficient assurance.
    #[test]
    fn under_assured_distinct_plus_self_is_not_approved() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH1)?;
        let action = ActionId(ACT.to_owned());
        let impact = ImpactSetHash(IMPACT.to_owned());
        let r = stepup_req(&action, &plan, &impact);
        let weak_bob = appr_full(
            ApprovalVerb::Approve,
            ACT,
            &plan,
            IMPACT,
            "bob",
            95,
            None,
            4,
        );
        let strong_alice = appr_full(
            ApprovalVerb::Approve,
            ACT,
            &plan,
            IMPACT,
            "alice",
            95,
            None,
            9,
        );
        assert_eq!(
            approval_gate_stepup(&[weak_bob, strong_alice], &r),
            ApprovalOutcome::Denied(ApprovalDenyReason::SelfApproval)
        );
        Ok(())
    }

    #[test]
    fn assurance_below_requirement_is_insufficient() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH1)?;
        let action = ActionId(ACT.to_owned());
        let impact = ImpactSetHash(IMPACT.to_owned());
        let r = stepup_req(&action, &plan, &impact);
        let weak = appr_full(
            ApprovalVerb::Approve,
            ACT,
            &plan,
            IMPACT,
            "bob",
            95,
            None,
            4,
        );
        assert_eq!(
            approval_gate_stepup(&[weak], &r),
            ApprovalOutcome::Denied(ApprovalDenyReason::InsufficientAssurance)
        );
        Ok(())
    }

    // now=100, max_age=10: age==10 (issued 90) is fresh; age==11 (issued 89) is stale.
    #[test]
    fn freshness_age_boundary_is_strict() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH1)?;
        let action = ActionId(ACT.to_owned());
        let impact = ImpactSetHash(IMPACT.to_owned());
        let r = stepup_req(&action, &plan, &impact);
        let at_max = appr_full(
            ApprovalVerb::Approve,
            ACT,
            &plan,
            IMPACT,
            "bob",
            90,
            None,
            5,
        );
        assert_eq!(
            approval_gate_stepup(&[at_max], &r),
            ApprovalOutcome::Approved
        );
        let over_max = appr_full(
            ApprovalVerb::Approve,
            ACT,
            &plan,
            IMPACT,
            "bob",
            89,
            None,
            5,
        );
        assert_eq!(
            approval_gate_stepup(&[over_max], &r),
            ApprovalOutcome::Denied(ApprovalDenyReason::Stale)
        );
        Ok(())
    }

    // Deny-wins still beats a fully qualified distinct approve under the M06.5 gate.
    #[test]
    fn deny_wins_over_qualified_approve_stepup() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH1)?;
        let action = ActionId(ACT.to_owned());
        let impact = ImpactSetHash(IMPACT.to_owned());
        let r = stepup_req(&action, &plan, &impact);
        let good = appr_full(
            ApprovalVerb::Approve,
            ACT,
            &plan,
            IMPACT,
            "bob",
            95,
            None,
            5,
        );
        let deny = appr_full(ApprovalVerb::Deny, ACT, &plan, IMPACT, "carol", 95, None, 5);
        assert_eq!(
            approval_gate_stepup(&[good, deny], &r),
            ApprovalOutcome::Denied(ApprovalDenyReason::ExplicitDeny)
        );
        Ok(())
    }
}
