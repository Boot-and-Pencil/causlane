//! Authz-evidence verification for bundle-bound replay (ADR-0011 / P0-010).
//!
//! Split from `lib.rs` for the 800-line cap. For a predicate that requires
//! authorization, every required stage must carry a prior `Allow` decision that
//! is bound to the barrier's action/plan/predicate, issued under the policy the
//! predicate declares, issued at-or-before and not expired by the barrier's
//! evaluation time. This mirrors the live [`causlane_core::authz_gate`].

use causlane_contracts::{AuthzModeDto, CompiledPredicate};
use causlane_core::{
    classify_authz_decision, AuditEvent, AuthzDecisionRef, AuthzDecisionVerdict, AuthzDenyReason,
    AuthzPolicy, ExecutionBarrier, Timestamp,
};

use crate::ReplayError;

/// Whether `decision` satisfies the configured attestation policy: trivially true
/// when no secret is configured, otherwise it must carry a valid keyed
/// attestation over its canonical bytes under `attestation_key`.
fn authz_decision_attested(decision: &AuthzDecisionRef, attestation_key: Option<&[u8]>) -> bool {
    let Some(secret) = attestation_key else {
        return true;
    };
    decision.attestation.as_ref().is_some_and(|attestation| {
        causlane_contracts::attestation::verify_attestation(
            secret,
            &decision.attestation_message(),
            &attestation.0,
        )
    })
}

/// Verify the barrier's authz evidence against the predicate's required policy.
///
/// `barrier_time` is the barrier event's `occurred_at`, used as the freshness
/// evaluation time; when absent the lane falls back to a born-expired sanity
/// check (replay cannot otherwise know elapsed time — P0-010).
pub(crate) fn validate_authz_refs<'a>(
    prior_events: impl Iterator<Item = &'a AuditEvent>,
    barrier: &ExecutionBarrier,
    predicate: &CompiledPredicate,
    barrier_time: Option<Timestamp>,
    attestation_key: Option<&[u8]>,
) -> Result<(), ReplayError> {
    if predicate.authz_policy.mode != AuthzModeDto::Required {
        return Ok(());
    }
    let prior_decisions = prior_events
        .filter(|event| barrier.authz_decision_refs.contains(&event.event_id))
        .collect::<Vec<_>>();
    for stage in &predicate.authz_policy.stages {
        let mut saw_denied = false;
        let mut saw_invalid = false;
        let mut saw_policy_mismatch = false;
        let mut saw_issued_after = false;
        let mut saw_expired = false;
        let mut saw_stale = false;
        let mut saw_allow = false;
        // The expected policy the predicate requires (P0-010); an empty id does
        // not constrain. `freshness_max_age` is the ADR-0011 freshness bound.
        let expected_policy = AuthzPolicy {
            id: &predicate.authz_policy.policy_id,
            version: &predicate.authz_policy.policy_version,
            max_age: predicate.authz_policy.freshness_max_age,
        };
        for event in &prior_decisions {
            let Some(decision) = &event.authz_decision else {
                saw_invalid = true;
                continue;
            };
            // Structural + temporal classification is the single shared kernel
            // authority (`classify_authz_decision`), so replay and the live runtime
            // cannot drift. `barrier_time = None` falls back to the born-expired
            // check (replay cannot otherwise know elapsed time — P0-010). Replay
            // keeps its own aggregate precedence and rolls wrong-binding into
            // `saw_invalid`, and layers the keyed-attestation check below.
            match classify_authz_decision(
                decision,
                stage,
                &barrier.action_id,
                &barrier.plan_hash,
                &predicate.predicate,
                expected_policy,
                barrier_time,
            ) {
                AuthzDecisionVerdict::Allow => {
                    // ADR-0011: when a PDP secret is configured, an otherwise-valid
                    // Allow must also carry a valid keyed attestation — a forged or
                    // unsigned decision (the trace author cannot produce the PDP's
                    // MAC) is treated as invalid rather than authorizing.
                    if authz_decision_attested(decision, attestation_key) {
                        saw_allow = true;
                        break;
                    }
                    saw_invalid = true;
                }
                AuthzDecisionVerdict::Deny(AuthzDenyReason::Denied) => saw_denied = true,
                AuthzDecisionVerdict::Deny(AuthzDenyReason::WrongBinding) => saw_invalid = true,
                AuthzDecisionVerdict::Deny(AuthzDenyReason::PolicyMismatch) => {
                    saw_policy_mismatch = true;
                }
                AuthzDecisionVerdict::Deny(AuthzDenyReason::IssuedAfter) => saw_issued_after = true,
                AuthzDecisionVerdict::Deny(AuthzDenyReason::Expired) => saw_expired = true,
                AuthzDecisionVerdict::Deny(AuthzDenyReason::Stale) => saw_stale = true,
                // Stage mismatch is skipped; the classifier never returns Missing
                // (the aggregate "no applicable allow" outcome is decided below).
                AuthzDecisionVerdict::Skip
                | AuthzDecisionVerdict::Deny(AuthzDenyReason::Missing) => {}
            }
        }
        if saw_allow {
            continue;
        }
        let stage = stage.clone();
        if saw_denied || saw_invalid {
            return Err(ReplayError::AuthzDecisionDenied { stage });
        }
        if saw_policy_mismatch {
            return Err(ReplayError::AuthzPolicyMismatch { stage });
        }
        if saw_issued_after {
            return Err(ReplayError::AuthzIssuedAfterBarrier { stage });
        }
        if saw_expired {
            return Err(ReplayError::AuthzDecisionExpired { stage });
        }
        if saw_stale {
            return Err(ReplayError::AuthzDecisionStale { stage });
        }
        return Err(ReplayError::AuthzDecisionMissing { stage });
    }
    Ok(())
}
