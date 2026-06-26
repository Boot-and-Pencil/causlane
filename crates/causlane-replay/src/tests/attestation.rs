//! Keyed-attestation replay tests (ADR-0011 / ADR-0013), split from `tests.rs`
//! for the 800-line file cap. Reuses the parent module's fixtures via `super::`.

use super::{authz_required_bundle, insert_authz_decision, set_barrier_occurred_at, TRACE};
use crate::{AuthzDecisionDto, ReplayError, ReplayTrace};

// ADR-0011 authz attestation: with a PDP secret configured, an Allow decision is
// only honored if it carries a valid keyed attestation. A trace author can write
// an Allow-shaped payload but cannot produce the PDP's MAC, so forged/unsigned
// allows are refused — deny-by-default is no longer defeatable by self-supply.
#[test]
fn bundle_mode_attested_requires_valid_authz_attestation() -> Result<(), Box<dyn std::error::Error>>
{
    let secret = b"trusted-root-secret";
    let bundle = authz_required_bundle()?;

    // A correctly PDP-signed (and capability-signed) trace is accepted.
    let mut signed = ReplayTrace::from_json_str(TRACE)?;
    insert_authz_decision(&mut signed, AuthzDecisionDto::Allow);
    set_barrier_occurred_at(&mut signed, 5); // issued_at=1 <= 5 < expires_at=10
    sign_trace(&mut signed, secret)?;
    assert!(signed.verify_with_bundle_attested(&bundle, secret).is_ok());

    // An Allow attested by an attacker key the verifier does not hold is refused.
    let mut forged = ReplayTrace::from_json_str(TRACE)?;
    insert_authz_decision(&mut forged, AuthzDecisionDto::Allow);
    set_barrier_occurred_at(&mut forged, 5);
    sign_trace(&mut forged, b"attacker-secret")?;
    assert!(matches!(
        forged.verify_with_bundle_attested(&bundle, secret),
        Err(ReplayError::AuthzDecisionDenied { .. })
    ));

    // An unsigned Allow is refused once a secret is configured.
    let mut unsigned = ReplayTrace::from_json_str(TRACE)?;
    insert_authz_decision(&mut unsigned, AuthzDecisionDto::Allow);
    set_barrier_occurred_at(&mut unsigned, 5);
    assert!(matches!(
        unsigned.verify_with_bundle_attested(&bundle, secret),
        Err(ReplayError::AuthzDecisionDenied { .. })
    ));
    Ok(())
}

// Sign every capability and authz decision in the trace with `secret`, computing
// each tag over the lowered artifact's canonical message (1:1 with the DTO list).
fn sign_trace(trace: &mut ReplayTrace, secret: &[u8]) -> Result<(), ReplayError> {
    let events = trace.to_events()?;
    for (raw, lowered) in trace.events.iter_mut().zip(events.iter()) {
        if let (Some(dto), Some(capability)) =
            (&mut raw.execution_capability, &lowered.execution_capability)
        {
            dto.attestation = Some(causlane_contracts::attestation::attest(
                secret,
                &capability.attestation_message(),
            ));
        }
        if let (Some(dto), Some(decision)) = (&mut raw.authz_decision, &lowered.authz_decision) {
            dto.attestation = Some(causlane_contracts::attestation::attest(
                secret,
                &decision.attestation_message(),
            ));
        }
    }
    Ok(())
}
