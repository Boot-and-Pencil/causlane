//! Approval as a typed, auditable action (M06.4, S06).
//!
//! An approval is not a boolean flag: it is a recorded `gate.approved` /
//! `gate.denied` decision (the [`GateApproved`](crate::domain::AuditEventKind::GateApproved) /
//! [`GateDenied`](crate::domain::AuditEventKind::GateDenied) producer events)
//! bound to the **exact** target it authorizes — the three-coordinate binding
//! `action_id` + `plan_hash` + `impact_set_hash` (ADR-0013). This module is a
//! pure, fail-closed decider: given a set of approvals and a target triple, it
//! answers whether the target is approved.
//!
//! It is an additive companion to the policy-verdict authority in
//! [`crate::domain::authz`] (ADR-0011): that gate judges a *predicate* under a
//! *policy* and is deliberately impact-agnostic; this gate judges an *approval*
//! against a *concrete planned impact set*. It deliberately does **not** delegate
//! to `classify_authz_decision` — that classifier is policy/stage/temporal-axial
//! and ref-typed to the formal-bound `AuthzDecisionRef`; an approval shares only
//! the trivial action+plan equality skeleton, which is intentionally not factored
//! out so the formal-bound classifier stays uncoupled from this non-formal type.
//!
//! The decider grounds nothing and ignores `approval_event_id` / `actor`: it
//! trusts that each [`ApprovalRef`] was faithfully extracted from a grounded
//! producer event. Grounding (the event exists, its kind matches the verb, and it
//! attests the matching triple) is enforced at the runtime/replay boundary.
//!
//! M06.5 extends each [`ApprovalRef`] with step-up / separation-of-duties / freshness
//! attributes (`issued_at`, `expires_at`, `assurance`) consumed by the stricter
//! combined gate in [`crate::domain::approval_stepup`]; the coordinate-only
//! [`approval_gate`] here is unchanged.

use super::{ActionId, AuditEventId, ImpactSetHash, PlanHash, Timestamp};

/// Whether an approval action approved or denied its target — the gate *verb*,
/// mirroring the `gate.approved` / `gate.denied` events
/// ([`AuditEventKind::GateApproved`](crate::domain::AuditEventKind::GateApproved) /
/// [`GateDenied`](crate::domain::AuditEventKind::GateDenied)). It is the verb
/// abstraction, not a policy result, and carries no taxonomy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalVerb {
    /// `gate.approved` — the actor approved the bound target.
    Approve,
    /// `gate.denied` — the actor denied the bound target.
    Deny,
}

/// Engine-agnostic authentication assurance an approval action presented — an
/// **opaque ordinal** the kernel compares as `presented >= required` via the derived
/// `Ord` (a higher value satisfies a lower requirement). The kernel fixes only the
/// ordering; which concrete factors (a session credential, a second factor, a
/// hardware-backed key, …) map to which value is the host's policy. `AssuranceLevel(0)`
/// is the weakest level, so a requirement of `AssuranceLevel(0)` demands no step-up. No
/// authentication-mechanism taxonomy lives in the kernel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssuranceLevel(pub u8);

/// A recorded approval action: a `gate.approved` / `gate.denied` decision bound to
/// the exact target it authorizes.
///
/// Additive companion to [`AuthzDecisionRef`](crate::domain::AuthzDecisionRef): it
/// records an oversight decision binding all three coordinates `action_id` +
/// `plan_hash` + `impact_set_hash`. Unlike the **optional**
/// [`WitnessBinding`](crate::domain::WitnessBinding) impact field, the impact
/// binding here is **mandatory** — a deliberate strict tightening: an oversight
/// approval with no impact binding is not expressible and therefore authorizes
/// nothing. `approval_event_id` / `actor` are carried for provenance and eventual
/// audit wiring; the coordinate decider itself reads neither (the M06.5 step-up gate
/// reads `actor`, `issued_at`, `expires_at`, and `assurance`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovalRef {
    /// The producer event that recorded this approval action.
    pub approval_event_id: AuditEventId,
    /// Whether the actor approved or denied (`gate.approved` / `gate.denied`).
    pub verdict: ApprovalVerb,
    /// The action this approval is bound to.
    pub action_id: ActionId,
    /// The plan this approval is bound to.
    pub plan_hash: PlanHash,
    /// The canonical planned-impact set this approval is bound to.
    pub impact_set_hash: ImpactSetHash,
    /// The actor who took the approval action (opaque subject id; no taxonomy).
    /// M06.5 reads this for separation-of-duties (`actor != initiator`).
    pub actor: String,
    /// M06.5: when this approval action was issued — barrier-evaluated freshness.
    pub issued_at: Timestamp,
    /// M06.5: optional expiry; an expiry at-or-before `now` is expired.
    pub expires_at: Option<Timestamp>,
    /// M06.5: the authentication assurance the approver presented (step-up).
    pub assurance: AssuranceLevel,
}

/// Why an approval gate refused to authorize a target. The first five reasons are the
/// coordinate/verb verdicts produced by [`classify_approval`]; the last five are the
/// M06.5 step-up / separation-of-duties / freshness refusals produced by
/// [`classify_approval_stepup`](crate::domain::classify_approval_stepup).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApprovalDenyReason {
    /// No approval action was bound to the target at all.
    Missing,
    /// An approval was bound to a different action.
    WrongAction,
    /// An approval matched the action but was bound to a different plan.
    WrongPlan,
    /// An approval matched action+plan but was bound to a different impact set —
    /// the hard consequences drifted, so it fails closed.
    WrongImpactSet,
    /// An exactly-bound approval explicitly denied (`gate.denied`).
    ExplicitDeny,
    /// M06.5: an exactly-bound `gate.approved` came from the action initiator
    /// (self-approval; separation-of-duties forbids it).
    SelfApproval,
    /// M06.5: an exactly-bound distinct-actor `gate.approved` presented assurance
    /// below the required step-up level.
    InsufficientAssurance,
    /// M06.5: the exact-bound distinct-actor `gate.approved` was forward-dated at
    /// `now` (issued in the future; replay parity).
    IssuedAfter,
    /// M06.5: the exact-bound distinct-actor `gate.approved` had expired at `now`.
    Expired,
    /// M06.5: the exact-bound distinct-actor `gate.approved` was older than `max_age`
    /// at `now`.
    Stale,
}

/// Outcome of the approval gate for one target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApprovalOutcome {
    /// Some approval is an exactly-bound `gate.approved` and none is an
    /// exactly-bound `gate.denied`.
    Approved,
    /// The target is not approved, for this reason.
    Denied(ApprovalDenyReason),
}

impl ApprovalOutcome {
    /// Whether the target is approved.
    #[must_use]
    pub fn is_approved(&self) -> bool {
        matches!(self, ApprovalOutcome::Approved)
    }
}

/// Classify one approval against the target triple. The verb is honored **only**
/// on an exact three-coordinate match; any coordinate mismatch is a binding deny,
/// independent of the verb (a misbound `gate.approved` authorizes nothing, and a
/// misbound `gate.denied` reports the drifted coordinate, not `ExplicitDeny`).
/// Coordinates are checked action → plan → impact, so the reported reason is the
/// outermost coordinate that drifted. Never returns `Missing` (that is an
/// aggregate of [`approval_gate`], not a per-approval verdict).
pub fn classify_approval(
    approval: &ApprovalRef,
    action: &ActionId,
    plan: &PlanHash,
    impact_set: &ImpactSetHash,
) -> Result<(), ApprovalDenyReason> {
    if approval.action_id != *action {
        return Err(ApprovalDenyReason::WrongAction);
    }
    if approval.plan_hash != *plan {
        return Err(ApprovalDenyReason::WrongPlan);
    }
    if approval.impact_set_hash != *impact_set {
        return Err(ApprovalDenyReason::WrongImpactSet);
    }
    match approval.verdict {
        ApprovalVerb::Approve => Ok(()),
        ApprovalVerb::Deny => Err(ApprovalDenyReason::ExplicitDeny),
    }
}

/// Fail-closed approval gate. A target is approved iff some recorded approval is a
/// `gate.approved` bound to it on **all three** coordinates `action_id` +
/// `plan_hash` + `impact_set_hash`, and **no** approval is an exactly-bound
/// `gate.denied` (deny-wins). Anything else denies.
///
/// It accumulates every approval's classification then decides, so the outcome is
/// **independent of the order** of `approvals`. Precedence: an exactly-bound
/// `gate.denied` outranks any approve; else an exact approve authorizes; else the
/// outermost drifted coordinate is reported (action ≻ plan ≻ impact); else the
/// unbound set denies as [`ApprovalDenyReason::Missing`].
#[must_use]
pub fn approval_gate(
    approvals: &[ApprovalRef],
    action: &ActionId,
    plan: &PlanHash,
    impact_set: &ImpactSetHash,
) -> ApprovalOutcome {
    let mut saw_explicit_deny = false;
    let mut saw_approve = false;
    let mut saw_wrong_action = false;
    let mut saw_wrong_plan = false;
    let mut saw_wrong_impact = false;
    for approval in approvals {
        match classify_approval(approval, action, plan, impact_set) {
            Ok(()) => saw_approve = true,
            Err(ApprovalDenyReason::ExplicitDeny) => saw_explicit_deny = true,
            Err(ApprovalDenyReason::WrongAction) => saw_wrong_action = true,
            Err(ApprovalDenyReason::WrongPlan) => saw_wrong_plan = true,
            Err(ApprovalDenyReason::WrongImpactSet) => saw_wrong_impact = true,
            // `classify_approval` never returns `Missing` (an aggregate decided below)
            // nor any M06.5 step-up / SoD / freshness reason (those come from
            // `classify_approval_stepup`). Named no-ops keep the match exhaustive
            // without a value-discarding wildcard.
            Err(
                ApprovalDenyReason::Missing
                | ApprovalDenyReason::SelfApproval
                | ApprovalDenyReason::InsufficientAssurance
                | ApprovalDenyReason::IssuedAfter
                | ApprovalDenyReason::Expired
                | ApprovalDenyReason::Stale,
            ) => {}
        }
    }
    if saw_explicit_deny {
        return ApprovalOutcome::Denied(ApprovalDenyReason::ExplicitDeny);
    }
    if saw_approve {
        return ApprovalOutcome::Approved;
    }
    let reason = if saw_wrong_action {
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
        approval_gate, classify_approval, ApprovalDenyReason, ApprovalOutcome, ApprovalRef,
        ApprovalVerb, AssuranceLevel,
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

    fn appr(verb: ApprovalVerb, action: &str, plan: &PlanHash, impact: &str) -> ApprovalRef {
        ApprovalRef {
            approval_event_id: AuditEventId("evt".to_owned()),
            verdict: verb,
            action_id: ActionId(action.to_owned()),
            plan_hash: plan.clone(),
            impact_set_hash: ImpactSetHash(impact.to_owned()),
            actor: "alice".to_owned(),
            issued_at: Timestamp(0),
            expires_at: None,
            assurance: AssuranceLevel(0),
        }
    }

    /// Independent oracle: recomputes the gate by a different route (membership
    /// `.any()` queries, not the accumulate loop) so a shared bug cannot hide.
    fn reference_gate(
        approvals: &[ApprovalRef],
        action: &ActionId,
        plan: &PlanHash,
        impact: &ImpactSetHash,
    ) -> ApprovalOutcome {
        let exact = |a: &ApprovalRef| {
            a.action_id == *action && a.plan_hash == *plan && a.impact_set_hash == *impact
        };
        if approvals
            .iter()
            .any(|a| exact(a) && a.verdict == ApprovalVerb::Deny)
        {
            return ApprovalOutcome::Denied(ApprovalDenyReason::ExplicitDeny);
        }
        if approvals
            .iter()
            .any(|a| exact(a) && a.verdict == ApprovalVerb::Approve)
        {
            return ApprovalOutcome::Approved;
        }
        let reason = if approvals.iter().any(|a| a.action_id != *action) {
            ApprovalDenyReason::WrongAction
        } else if approvals
            .iter()
            .any(|a| a.action_id == *action && a.plan_hash != *plan)
        {
            ApprovalDenyReason::WrongPlan
        } else if approvals
            .iter()
            .any(|a| a.action_id == *action && a.plan_hash == *plan && a.impact_set_hash != *impact)
        {
            ApprovalDenyReason::WrongImpactSet
        } else {
            ApprovalDenyReason::Missing
        };
        ApprovalOutcome::Denied(reason)
    }

    /// Load-bearing property: across the grid of {target, other} on each of
    /// action / plan / impact × {Approve, Deny}, `approval_gate` matches the
    /// independent reference oracle for every single approval and every pair, AND
    /// is order-independent (`gate(xs) == gate(reverse(xs))`). Non-vacuity: every
    /// `ApprovalDenyReason` and `Approved` is observed.
    #[test]
    fn approval_gate_matches_oracle_and_is_order_independent() -> Result<(), PlanHashError> {
        let plan1 = PlanHash::new(HASH1)?;
        let plan2 = PlanHash::new(HASH2)?;
        let action = ActionId(ACT.to_owned());
        let impact = ImpactSetHash(IMPACT.to_owned());

        let mut configs = Vec::new();
        for act in [ACT, OTHER_ACT] {
            for plan in [&plan1, &plan2] {
                for imp in [IMPACT, OTHER_IMPACT] {
                    for verb in [ApprovalVerb::Approve, ApprovalVerb::Deny] {
                        configs.push(appr(verb, act, plan, imp));
                    }
                }
            }
        }

        let mut outcomes = Vec::new();

        // Empty set denies as Missing (the only path to Missing).
        let empty = approval_gate(&[], &action, &plan1, &impact);
        assert_eq!(empty, ApprovalOutcome::Denied(ApprovalDenyReason::Missing));
        outcomes.push(empty);

        for a in &configs {
            let single = [a.clone()];
            let out = approval_gate(&single, &action, &plan1, &impact);
            assert_eq!(out, reference_gate(&single, &action, &plan1, &impact));
            outcomes.push(out);
            for b in &configs {
                let forward = [a.clone(), b.clone()];
                let backward = [b.clone(), a.clone()];
                let out = approval_gate(&forward, &action, &plan1, &impact);
                assert_eq!(
                    out,
                    approval_gate(&backward, &action, &plan1, &impact),
                    "approval_gate is order-dependent"
                );
                assert_eq!(
                    out,
                    reference_gate(&forward, &action, &plan1, &impact),
                    "approval_gate diverged from the reference oracle"
                );
                outcomes.push(out);
            }
        }

        let has = |want: &ApprovalOutcome| outcomes.iter().any(|o| o == want);
        assert!(has(&ApprovalOutcome::Approved), "Approved never produced");
        let denied = |r| ApprovalOutcome::Denied(r);
        assert!(
            has(&denied(ApprovalDenyReason::ExplicitDeny)),
            "ExplicitDeny never produced"
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

    #[test]
    fn exact_approve_authorizes() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH1)?;
        let out = approval_gate(
            &[appr(ApprovalVerb::Approve, ACT, &plan, IMPACT)],
            &ActionId(ACT.to_owned()),
            &plan,
            &ImpactSetHash(IMPACT.to_owned()),
        );
        assert_eq!(out, ApprovalOutcome::Approved);
        Ok(())
    }

    #[test]
    fn deny_wins_over_approve() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH1)?;
        let out = approval_gate(
            &[
                appr(ApprovalVerb::Approve, ACT, &plan, IMPACT),
                appr(ApprovalVerb::Deny, ACT, &plan, IMPACT),
            ],
            &ActionId(ACT.to_owned()),
            &plan,
            &ImpactSetHash(IMPACT.to_owned()),
        );
        assert_eq!(
            out,
            ApprovalOutcome::Denied(ApprovalDenyReason::ExplicitDeny)
        );
        Ok(())
    }

    // A misbound gate.denied reports the drifted coordinate (impact), never
    // ExplicitDeny — coordinates are checked before the verb.
    #[test]
    fn misbound_deny_is_wrong_impact_not_explicit_deny() -> Result<(), PlanHashError> {
        let plan = PlanHash::new(HASH1)?;
        let deny_other_impact = appr(ApprovalVerb::Deny, ACT, &plan, OTHER_IMPACT);
        assert_eq!(
            classify_approval(
                &deny_other_impact,
                &ActionId(ACT.to_owned()),
                &plan,
                &ImpactSetHash(IMPACT.to_owned()),
            ),
            Err(ApprovalDenyReason::WrongImpactSet)
        );
        Ok(())
    }
}
