//! Minimal, dependency-free HMAC-SHA-256 (RFC 2104 / FIPS 198-1).
//!
//! Built on the in-tree [`super::sha256`] so the offline build stays free of
//! C-backed crypto crates (same rationale as `sha256.rs`). Used to bind audit
//! artifacts (execution capabilities, authz decisions) to a kernel/PDP secret so
//! an attacker who controls a replay trace — but not the secret — cannot forge a
//! valid attestation. Correctness is pinned by the RFC 4231 test vectors below.
//!
//! All buffer access is checked (iterator-based), never panic-prone indexing.

use core::fmt::Write as _;

use crate::sha256::sha256;

const BLOCK: usize = 64;

/// Compute HMAC-SHA-256 of `message` under `key`, returning the raw 32-byte tag.
#[must_use]
pub(crate) fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    // Keys longer than the block size are hashed down first (RFC 2104).
    let mut block = [0u8; BLOCK];
    if key.len() > BLOCK {
        let digest = sha256(key);
        for (slot, byte) in block.iter_mut().zip(digest.iter()) {
            *slot = *byte;
        }
    } else {
        for (slot, byte) in block.iter_mut().zip(key.iter()) {
            *slot = *byte;
        }
    }

    let mut ipad = [0u8; BLOCK];
    let mut opad = [0u8; BLOCK];
    for ((i_slot, o_slot), byte) in ipad.iter_mut().zip(opad.iter_mut()).zip(block.iter()) {
        *i_slot = byte ^ 0x36;
        *o_slot = byte ^ 0x5c;
    }

    let mut inner = Vec::with_capacity(BLOCK + message.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(message);
    let inner_digest = sha256(&inner);

    let mut outer = Vec::with_capacity(BLOCK + 32);
    outer.extend_from_slice(&opad);
    outer.extend_from_slice(&inner_digest);
    sha256(&outer)
}

/// HMAC-SHA-256 as 64 lowercase hex chars.
#[must_use]
pub(crate) fn hmac_sha256_hex(key: &[u8], message: &[u8]) -> String {
    let tag = hmac_sha256(key, message);
    let mut out = String::with_capacity(64);
    for byte in tag {
        let _written = write!(out, "{byte:02x}");
    }
    out
}

/// Constant-time-ish verification that `tag_hex` is the HMAC of `message` under
/// `key`. Compares the recomputed hex with no early return on the first
/// differing byte, so timing does not leak the matching prefix.
#[must_use]
pub(crate) fn hmac_sha256_verify(key: &[u8], message: &[u8], tag_hex: &str) -> bool {
    let expected = hmac_sha256_hex(key, message);
    if expected.len() != tag_hex.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in expected.bytes().zip(tag_hex.bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::{hmac_sha256_hex, hmac_sha256_verify};

    // RFC 4231 Test Case 1: key = 0x0b * 20, data = "Hi There".
    #[test]
    fn rfc4231_case1() {
        let key = [0x0bu8; 20];
        assert_eq!(
            hmac_sha256_hex(&key, b"Hi There"),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7",
        );
    }

    // RFC 4231 Test Case 2: key = "Jefe", data = "what do ya want for nothing?".
    #[test]
    fn rfc4231_case2() {
        assert_eq!(
            hmac_sha256_hex(b"Jefe", b"what do ya want for nothing?"),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843",
        );
    }

    // RFC 4231 Test Case 6: key longer than the block size (131 * 0xaa).
    #[test]
    fn rfc4231_case6_long_key() {
        let key = [0xaau8; 131];
        assert_eq!(
            hmac_sha256_hex(
                &key,
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            ),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54",
        );
    }

    #[test]
    fn verify_accepts_matching_and_rejects_tampered() {
        let key = b"kernel-secret";
        let tag = hmac_sha256_hex(key, b"payload");
        assert!(hmac_sha256_verify(key, b"payload", &tag));
        assert!(!hmac_sha256_verify(key, b"payload!", &tag)); // wrong message
        assert!(!hmac_sha256_verify(b"other-secret", b"payload", &tag)); // wrong key
        let tampered = tag.replacen(|c: char| c != '0', "0", 1);
        assert!(!hmac_sha256_verify(key, b"payload", &tampered)); // tampered tag
    }
}
