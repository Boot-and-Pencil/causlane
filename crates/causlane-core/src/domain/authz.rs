//! Deny-by-default authorization gate (ADR-0011).
//!
//! For a predicate that requires authorization, every required lifecycle stage
//! must carry an `Allow` decision bound to the exact action/plan/predicate and
//! not expired at evaluation time; anything else denies execution. This pure
//! gate is the single fail-closed authority: the **runtime** enforces it *live*
//! (before spending a barrier — see `causlane-runtime`), and it mirrors the
//! intent the replay oracle enforces *post-hoc* on recorded traces.

use super::{ActionId, AuthzDecision, AuthzDecisionRef, PlanHash, Timestamp};

/// Why the authorization gate denied a required stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthzDenyReason {
    /// No decision was present for the required stage.
    Missing,
    /// A decision explicitly denied the stage.
    Denied,
    /// A decision applied to the stage but was bound to the wrong
    /// action/plan/predicate.
    WrongBinding,
    /// A decision authorized the action/plan/stage but was issued under a
    /// different policy than the predicate requires (P0-010).
    PolicyMismatch,
    /// The only applicable allow decision had expired at evaluation time, or was
    /// born-expired (`expires_at <= issued_at`).
    Expired,
    /// The only applicable allow decision was older than the policy's freshness
    /// bound at evaluation time (ADR-0011 "fresh"), even though not expired.
    Stale,
    /// The only applicable allow decision was forward-dated (issued after the
    /// evaluation time). Mirrors the replay oracle's `AuthzIssuedAfterBarrier`
    /// so the live gate and post-hoc replay agree (ADR-0011 parity).
    IssuedAfter,
}

/// Outcome of the deny-by-default authorization gate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthzGateOutcome {
    /// Every required stage has a valid, bound, non-expired allow decision.
    Allowed,
    /// A required stage is not authorized.
    Denied {
        /// The unauthorized stage.
        stage: String,
        /// Why it was denied.
        reason: AuthzDenyReason,
    },
}

impl AuthzGateOutcome {
    /// Whether execution is authorized.
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        matches!(self, AuthzGateOutcome::Allowed)
    }
}

/// The policy a predicate requires its authz decisions to be issued under
/// (P0-010). An empty [`AuthzPolicy::id`] does not constrain the policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthzPolicy<'a> {
    /// Expected policy id.
    pub id: &'a str,
    /// Expected policy version.
    pub version: &'a str,
    /// Maximum decision age at evaluation time (ADR-0011 "fresh"); `None` imposes
    /// no freshness bound beyond the decision's own expiry.
    pub max_age: Option<u64>,
}

impl AuthzPolicy<'_> {
    /// Whether this policy constrains decisions (a non-empty id).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }
}

/// The verdict of classifying a single authz decision against one required stage.
/// This is the shared structural+temporal per-decision authority that both the
/// live gate ([`authz_gate`]) and the replay oracle evaluate, so the two cannot
/// drift; each caller aggregates verdicts under its own precedence and layers its
/// own additional checks (replay adds keyed-attestation/event-structure).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthzDecisionVerdict {
    /// The decision does not apply to the stage under evaluation.
    Skip,
    /// The decision is a valid, bound, in-policy, non-expired allow for the stage.
    Allow,
    /// The decision denies (or fails to authorize) the stage for this reason.
    Deny(AuthzDenyReason),
}

/// Classify a single authz decision for `stage` under the deny-by-default rule
/// (ADR-0011): stage filter, deny-wins, action/plan/predicate binding, policy
/// (P0-010), then temporal validity. `now` is the evaluation time; `None` means no
/// absolute time is available (replay with no barrier time) and only the
/// born-expired sanity check applies. The separate born-expired check is omitted in
/// the `Some` branch because it is subsumed there: with `now >= issued_at` a
/// born-expired decision (`expires_at <= issued_at`) also has `expires_at <= now`
/// and is caught as [`AuthzDenyReason::Expired`]; with `now < issued_at` it is
/// already [`AuthzDenyReason::IssuedAfter`].
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn classify_authz_decision(
    decision: &AuthzDecisionRef,
    stage: &str,
    action: &ActionId,
    plan: &PlanHash,
    predicate_id: &str,
    expected_policy: AuthzPolicy<'_>,
    now: Option<Timestamp>,
) -> AuthzDecisionVerdict {
    use AuthzDecisionVerdict::{Allow, Deny, Skip};
    if decision.stage != *stage {
        return Skip;
    }
    if decision.decision == AuthzDecision::Deny {
        return Deny(AuthzDenyReason::Denied);
    }
    if decision.action_id != *action
        || decision.plan_hash != *plan
        || decision.predicate_id != predicate_id
    {
        return Deny(AuthzDenyReason::WrongBinding);
    }
    // P0-010: the decision must be issued under the policy the predicate requires.
    // An empty expected policy (e.g. unset) does not constrain.
    if !expected_policy.is_empty()
        && (decision.policy_id != expected_policy.id
            || decision.policy_version != expected_policy.version)
    {
        return Deny(AuthzDenyReason::PolicyMismatch);
    }
    match now {
        Some(now) => {
            // ADR-0011 parity with replay: a forward-dated decision is not yet valid.
            if decision.issued_at.0 > now.0 {
                return Deny(AuthzDenyReason::IssuedAfter);
            }
            if decision.expires_at.is_some_and(|expiry| expiry.0 <= now.0) {
                return Deny(AuthzDenyReason::Expired);
            }
            // ADR-0011 freshness: reject a decision older than the policy bound.
            if expected_policy
                .max_age
                .is_some_and(|max| now.0.saturating_sub(decision.issued_at.0) > max)
            {
                return Deny(AuthzDenyReason::Stale);
            }
        }
        None => {
            // Born-expired sanity check: an expiry at-or-before issuance can never
            // be valid, independent of `now`.
            if decision
                .expires_at
                .is_some_and(|expiry| expiry.0 <= decision.issued_at.0)
            {
                return Deny(AuthzDenyReason::Expired);
            }
        }
    }
    Allow
}

/// Deny-by-default authorization gate (ADR-0011, fail-closed). For each required
/// stage there must be an `Allow` decision bound to `action`/`plan`/`predicate_id`,
/// issued under `expected_policy`, and not expired at `now`; otherwise the first
/// unauthorized stage is denied.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn authz_gate(
    required_stages: &[String],
    decisions: &[AuthzDecisionRef],
    action: &ActionId,
    plan: &PlanHash,
    predicate_id: &str,
    expected_policy: AuthzPolicy<'_>,
    now: Timestamp,
) -> AuthzGateOutcome {
    for stage in required_stages {
        let mut saw_denied = false;
        let mut saw_wrong_binding = false;
        let mut saw_policy_mismatch = false;
        let mut saw_expired = false;
        let mut saw_stale = false;
        let mut saw_issued_after = false;
        let mut allowed = false;
        for decision in decisions {
            match classify_authz_decision(
                decision,
                stage,
                action,
                plan,
                predicate_id,
                expected_policy,
                Some(now),
            ) {
                AuthzDecisionVerdict::Allow => {
                    allowed = true;
                    break;
                }
                AuthzDecisionVerdict::Deny(AuthzDenyReason::Denied) => saw_denied = true,
                AuthzDecisionVerdict::Deny(AuthzDenyReason::WrongBinding) => {
                    saw_wrong_binding = true;
                }
                AuthzDecisionVerdict::Deny(AuthzDenyReason::PolicyMismatch) => {
                    saw_policy_mismatch = true;
                }
                AuthzDecisionVerdict::Deny(AuthzDenyReason::Expired) => saw_expired = true,
                AuthzDecisionVerdict::Deny(AuthzDenyReason::Stale) => saw_stale = true,
                AuthzDecisionVerdict::Deny(AuthzDenyReason::IssuedAfter) => {
                    saw_issued_after = true;
                }
                // Stage mismatch is skipped; the classifier never returns Missing
                // (that aggregate "no applicable allow" outcome is decided below).
                AuthzDecisionVerdict::Skip
                | AuthzDecisionVerdict::Deny(AuthzDenyReason::Missing) => {}
            }
        }
        if allowed {
            continue;
        }
        let reason = if saw_denied {
            AuthzDenyReason::Denied
        } else if saw_wrong_binding {
            AuthzDenyReason::WrongBinding
        } else if saw_policy_mismatch {
            AuthzDenyReason::PolicyMismatch
        } else if saw_expired {
            AuthzDenyReason::Expired
        } else if saw_stale {
            AuthzDenyReason::Stale
        } else if saw_issued_after {
            AuthzDenyReason::IssuedAfter
        } else {
            AuthzDenyReason::Missing
        };
        return AuthzGateOutcome::Denied {
            stage: stage.clone(),
            reason,
        };
    }
    AuthzGateOutcome::Allowed
}

#[cfg(test)]
mod tests {
    use super::{
        authz_gate, classify_authz_decision, AuthzDecisionVerdict, AuthzDenyReason,
        AuthzGateOutcome, AuthzPolicy,
    };
    use crate::domain::{ActionId, AuditEventId, AuthzDecision, AuthzDecisionRef, Timestamp};
    use crate::PlanHash;

    /// The policy the `decision` helper issues decisions under.
    const POLICY: AuthzPolicy<'static> = AuthzPolicy {
        id: "p",
        version: "1",
        max_age: None,
    };

    fn plan() -> Result<PlanHash, crate::PlanHashError> {
        PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    }

    fn decision(
        stage: &str,
        verdict: AuthzDecision,
        action: &str,
        plan: PlanHash,
        predicate: &str,
        expires_at: Option<u64>,
    ) -> AuthzDecisionRef {
        AuthzDecisionRef {
            decision_event_id: AuditEventId("d".to_owned()),
            action_id: ActionId(action.to_owned()),
            plan_hash: plan,
            predicate_id: predicate.to_owned(),
            actor: "alice".to_owned(),
            stage: stage.to_owned(),
            decision: verdict,
            policy_id: "p".to_owned(),
            policy_version: "1".to_owned(),
            issued_at: Timestamp(0),
            expires_at: expires_at.map(Timestamp),
            attestation: None,
        }
    }

    // A bound, non-expired Allow authorizes the required stage.
    #[test]
    fn valid_allow_is_authorized() -> Result<(), crate::PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let allow = decision(
            "execution_barrier_logged",
            AuthzDecision::Allow,
            "act",
            plan()?,
            "release.promote",
            Some(100),
        );
        let outcome = authz_gate(
            &stages,
            std::slice::from_ref(&allow),
            &ActionId("act".to_owned()),
            &plan()?,
            "release.promote",
            POLICY,
            Timestamp(10),
        );
        assert_eq!(outcome, AuthzGateOutcome::Allowed);
        assert!(outcome.is_allowed());
        Ok(())
    }

    // Deny-by-default: no decision, an explicit deny, a wrong binding and an
    // expired allow are all refused with the matching reason.
    #[test]
    fn fail_closed_on_every_gap() -> Result<(), crate::PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let action = ActionId("act".to_owned());

        // Missing.
        let missing = authz_gate(
            &stages,
            &[],
            &action,
            &plan()?,
            "release.promote",
            POLICY,
            Timestamp(10),
        );
        assert!(matches!(
            missing,
            AuthzGateOutcome::Denied {
                reason: AuthzDenyReason::Missing,
                ..
            }
        ));

        // Explicit deny.
        let deny = decision(
            "execution_barrier_logged",
            AuthzDecision::Deny,
            "act",
            plan()?,
            "release.promote",
            None,
        );
        assert!(matches!(
            authz_gate(
                &stages,
                std::slice::from_ref(&deny),
                &action,
                &plan()?,
                "release.promote",
                POLICY,
                Timestamp(10)
            ),
            AuthzGateOutcome::Denied {
                reason: AuthzDenyReason::Denied,
                ..
            }
        ));

        // Wrong binding (different predicate).
        let wrong = decision(
            "execution_barrier_logged",
            AuthzDecision::Allow,
            "act",
            plan()?,
            "other.predicate",
            None,
        );
        assert!(matches!(
            authz_gate(
                &stages,
                std::slice::from_ref(&wrong),
                &action,
                &plan()?,
                "release.promote",
                POLICY,
                Timestamp(10)
            ),
            AuthzGateOutcome::Denied {
                reason: AuthzDenyReason::WrongBinding,
                ..
            }
        ));

        // Expired allow (expiry <= now).
        let expired = decision(
            "execution_barrier_logged",
            AuthzDecision::Allow,
            "act",
            plan()?,
            "release.promote",
            Some(5),
        );
        assert!(matches!(
            authz_gate(
                &stages,
                std::slice::from_ref(&expired),
                &action,
                &plan()?,
                "release.promote",
                POLICY,
                Timestamp(10)
            ),
            AuthzGateOutcome::Denied {
                reason: AuthzDenyReason::Expired,
                ..
            }
        ));
        Ok(())
    }

    // P0-010: a bound, non-expired Allow issued under a DIFFERENT policy than the
    // predicate requires is refused with PolicyMismatch.
    #[test]
    fn wrong_policy_is_refused() -> Result<(), crate::PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let allow = decision(
            "execution_barrier_logged",
            AuthzDecision::Allow,
            "act",
            plan()?,
            "release.promote",
            Some(100),
        );
        // The decision is issued under policy "p"/v"1"; the predicate requires a
        // different policy version.
        let outcome = authz_gate(
            &stages,
            std::slice::from_ref(&allow),
            &ActionId("act".to_owned()),
            &plan()?,
            "release.promote",
            AuthzPolicy {
                id: "p",
                version: "2",
                max_age: None,
            },
            Timestamp(10),
        );
        assert!(matches!(
            outcome,
            AuthzGateOutcome::Denied {
                reason: AuthzDenyReason::PolicyMismatch,
                ..
            }
        ));
        Ok(())
    }

    // ADR-0011 freshness: a bound, non-expired Allow that is older than the
    // policy's max_age at evaluation time is refused as Stale.
    #[test]
    fn stale_decision_is_refused() -> Result<(), crate::PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        // issued_at=0 (the `decision` helper); a far-future expiry so it is not
        // expired, but it is 10 old at now=10 with a max_age of 5.
        let allow = decision(
            "execution_barrier_logged",
            AuthzDecision::Allow,
            "act",
            plan()?,
            "release.promote",
            Some(1000),
        );
        let outcome = authz_gate(
            &stages,
            std::slice::from_ref(&allow),
            &ActionId("act".to_owned()),
            &plan()?,
            "release.promote",
            AuthzPolicy {
                id: "p",
                version: "1",
                max_age: Some(5),
            },
            Timestamp(10),
        );
        assert!(matches!(
            outcome,
            AuthzGateOutcome::Denied {
                reason: AuthzDenyReason::Stale,
                ..
            }
        ));
        Ok(())
    }

    // ADR-0011 parity with replay: a forward-dated Allow (issued after the
    // evaluation time) is refused as IssuedAfter — the live gate must agree with
    // the replay oracle's `AuthzIssuedAfterBarrier`.
    #[test]
    fn forward_dated_decision_is_refused() -> Result<(), crate::PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let mut allow = decision(
            "execution_barrier_logged",
            AuthzDecision::Allow,
            "act",
            plan()?,
            "release.promote",
            Some(2000),
        );
        allow.issued_at = Timestamp(1000);
        // now (10) is *before* the decision was issued (1000): not yet valid.
        let outcome = authz_gate(
            &stages,
            std::slice::from_ref(&allow),
            &ActionId("act".to_owned()),
            &plan()?,
            "release.promote",
            POLICY,
            Timestamp(10),
        );
        assert!(matches!(
            outcome,
            AuthzGateOutcome::Denied {
                reason: AuthzDenyReason::IssuedAfter,
                ..
            }
        ));
        Ok(())
    }

    // A born-expired Allow (expires_at <= issued_at) can never be valid, even at
    // a `now` between issuance and expiry — mirrors replay's no-barrier-time
    // fallback.
    #[test]
    fn born_expired_decision_is_refused() -> Result<(), crate::PlanHashError> {
        let stages = vec!["execution_barrier_logged".to_owned()];
        let mut allow = decision(
            "execution_barrier_logged",
            AuthzDecision::Allow,
            "act",
            plan()?,
            "release.promote",
            Some(5),
        );
        allow.issued_at = Timestamp(5); // expires_at == issued_at -> born-expired
        let outcome = authz_gate(
            &stages,
            std::slice::from_ref(&allow),
            &ActionId("act".to_owned()),
            &plan()?,
            "release.promote",
            POLICY,
            Timestamp(5),
        );
        assert!(matches!(
            outcome,
            AuthzGateOutcome::Denied {
                reason: AuthzDenyReason::Expired,
                ..
            }
        ));
        Ok(())
    }

    // An independent re-statement of the ADR-0011 per-decision rule, written with a
    // different control-flow shape than `classify_authz_decision`, so the grid test
    // below catches any faithful-extraction error in the dedup.
    #[allow(clippy::too_many_arguments)]
    fn reference_classify(
        decision: &AuthzDecisionRef,
        stage: &str,
        action: &ActionId,
        plan: &PlanHash,
        predicate_id: &str,
        policy: AuthzPolicy<'_>,
        now: Option<Timestamp>,
    ) -> AuthzDecisionVerdict {
        use AuthzDecisionVerdict::{Allow, Deny, Skip};
        if decision.stage != *stage {
            return Skip;
        }
        if decision.decision == AuthzDecision::Deny {
            return Deny(AuthzDenyReason::Denied);
        }
        let bound = decision.action_id == *action
            && decision.plan_hash == *plan
            && decision.predicate_id == predicate_id;
        if !bound {
            return Deny(AuthzDenyReason::WrongBinding);
        }
        let policy_ok = policy.is_empty()
            || (decision.policy_id == policy.id && decision.policy_version == policy.version);
        if !policy_ok {
            return Deny(AuthzDenyReason::PolicyMismatch);
        }
        if let Some(now) = now {
            if decision.issued_at.0 > now.0 {
                return Deny(AuthzDenyReason::IssuedAfter);
            }
            if let Some(expiry) = decision.expires_at {
                if expiry.0 <= now.0 {
                    return Deny(AuthzDenyReason::Expired);
                }
            }
            if let Some(max) = policy.max_age {
                if now.0.saturating_sub(decision.issued_at.0) > max {
                    return Deny(AuthzDenyReason::Stale);
                }
            }
        } else if let Some(expiry) = decision.expires_at {
            if expiry.0 <= decision.issued_at.0 {
                return Deny(AuthzDenyReason::Expired);
            }
        }
        Allow
    }

    // Property test (complete over a boundary-straddling grid): the shared
    // `classify_authz_decision` matches the independent reference oracle for every
    // combination of binding, policy, verdict, stage and temporal value. The six
    // example tests above only spot-check single defects; this pins the dedup.
    #[test]
    fn classify_authz_decision_matches_reference_oracle() -> Result<(), crate::PlanHashError> {
        const QUERY_STAGE: &str = "execution_barrier_logged";
        let action = ActionId("act".to_owned());
        let ph = plan()?;
        let other_action = ActionId("other".to_owned());
        let other_ph = PlanHash::new(
            "sha256:2222222222222222222222222222222222222222222222222222222222222222",
        )?;
        let predicate = "release.promote";

        // (action, plan, predicate) for the decision: all-match, then one wrong
        // component each — exercises the WrongBinding path per field.
        let bindings = [
            (action.clone(), ph.clone(), predicate),
            (other_action, ph.clone(), predicate),
            (action.clone(), other_ph, predicate),
            (action.clone(), ph.clone(), "other.predicate"),
        ];
        let policies = [
            AuthzPolicy {
                id: "",
                version: "",
                max_age: None,
            },
            AuthzPolicy {
                id: "p",
                version: "1",
                max_age: None,
            },
            AuthzPolicy {
                id: "p",
                version: "1",
                max_age: Some(5),
            },
        ];
        let nows = [
            None,
            Some(Timestamp(0)),
            Some(Timestamp(5)),
            Some(Timestamp(10)),
        ];

        let mut checked = 0_u32;
        for (dact, dplan, dpred) in bindings {
            for dstage in [QUERY_STAGE, "other_stage"] {
                for verdict in [AuthzDecision::Allow, AuthzDecision::Deny] {
                    for (dpid, dpver) in [("p", "1"), ("q", "1"), ("p", "2")] {
                        for issued in [0_u64, 5, 10] {
                            for expiry in [None, Some(3_u64), Some(5), Some(10), Some(100)] {
                                let decision = AuthzDecisionRef {
                                    decision_event_id: AuditEventId("d".to_owned()),
                                    action_id: dact.clone(),
                                    plan_hash: dplan.clone(),
                                    predicate_id: dpred.to_owned(),
                                    actor: "alice".to_owned(),
                                    stage: dstage.to_owned(),
                                    decision: verdict,
                                    policy_id: dpid.to_owned(),
                                    policy_version: dpver.to_owned(),
                                    issued_at: Timestamp(issued),
                                    expires_at: expiry.map(Timestamp),
                                    attestation: None,
                                };
                                for now in nows {
                                    for policy in policies {
                                        assert_eq!(
                                            classify_authz_decision(
                                                &decision, QUERY_STAGE, &action, &ph, predicate,
                                                policy, now,
                                            ),
                                            reference_classify(
                                                &decision, QUERY_STAGE, &action, &ph, predicate,
                                                policy, now,
                                            ),
                                            "classify vs reference mismatch: {decision:?} now={now:?} policy={policy:?}",
                                        );
                                        checked += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(checked >= 8000, "grid unexpectedly small: {checked}");
        Ok(())
    }

    // authz_gate AGGREGATION (beyond the single-stage/single-decision tests above):
    // deny-wins precedence among a stage's decisions, and the first unauthorized
    // required stage decides the outcome (ADR-0011 fail-closed).
    #[test]
    fn authz_gate_aggregates_precedence_and_stage_order() -> Result<(), crate::PlanHashError> {
        let action = ActionId("act".to_owned());
        let p = plan()?;
        let pred = "release.promote";

        // One stage, two non-allow decisions: a wrong-binding Allow plus an explicit
        // Deny. Denied outranks WrongBinding in the precedence ladder.
        let wrong = decision(
            "s",
            AuthzDecision::Allow,
            "act",
            p.clone(),
            "other.pred",
            None,
        );
        let deny = decision("s", AuthzDecision::Deny, "act", p.clone(), pred, None);
        assert!(matches!(
            authz_gate(
                &["s".to_owned()],
                &[wrong, deny],
                &action,
                &p,
                pred,
                POLICY,
                Timestamp(10)
            ),
            AuthzGateOutcome::Denied {
                reason: AuthzDenyReason::Denied,
                ..
            }
        ));

        // Two required stages, only s1 authorized: denial reports the first
        // unauthorized stage (s2), proving per-stage ordering.
        let two = vec!["s1".to_owned(), "s2".to_owned()];
        let allow_s1 = decision("s1", AuthzDecision::Allow, "act", p.clone(), pred, None);
        assert!(matches!(
            authz_gate(&two, &[allow_s1], &action, &p, pred, POLICY, Timestamp(10)),
            AuthzGateOutcome::Denied { stage, reason: AuthzDenyReason::Missing } if stage == "s2"
        ));
        Ok(())
    }
}
