//! Keyed attestation over audit artifacts (HMAC-SHA-256).
//!
//! The minting kernel/PDP holds a secret and tags a capability's (or authz
//! decision's) canonical bytes; a replay verifier configured with that secret
//! recomputes the tag and rejects any artifact whose tag is absent or wrong. An
//! attacker who controls a replay trace but not the secret cannot forge a valid
//! attestation, which is what upgrades the capability/authz binding from
//! structural-only to cryptographic (ADR-0011, ADR-0013).
//!
//! The wire form is a lowercase-hex HMAC string carried on the artifact. The
//! kernel-side secret is supplied out of band (verifier config), not embedded in
//! the bundle or trace.

use crate::hmac::{hmac_sha256_hex, hmac_sha256_verify};

/// Compute the attestation tag (hex HMAC) for `message` under `secret`. Used by
/// a minting kernel and by tests/fixtures that produce signed artifacts.
#[must_use]
pub fn attest(secret: &[u8], message: &[u8]) -> String {
    hmac_sha256_hex(secret, message)
}

/// Verify that `tag_hex` is a valid attestation of `message` under `secret`.
#[must_use]
pub fn verify_attestation(secret: &[u8], message: &[u8], tag_hex: &str) -> bool {
    hmac_sha256_verify(secret, message, tag_hex)
}

#[cfg(test)]
mod tests {
    use super::{attest, verify_attestation};

    #[test]
    fn round_trips_and_rejects_forgery() {
        let secret = b"kernel-secret-v1";
        let message = b"causlane-capability-attestation-v1\x1f3:cap";
        let tag = attest(secret, message);
        assert!(verify_attestation(secret, message, &tag));
        assert!(!verify_attestation(b"attacker-guess", message, &tag));
        assert!(!verify_attestation(secret, b"tampered", &tag));
    }
}
