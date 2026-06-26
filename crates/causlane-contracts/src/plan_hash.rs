//! Canonical plan-hash material and content hashing (ADR-0009).
//!
//! The material is a stable, serializable projection of everything that must
//! influence a plan's identity. Runtime-only facts (event ids, timestamps,
//! granted leases, observed results) are deliberately excluded.

use serde::{Deserialize, Serialize};

use causlane_core::{ContentHash, ImpactSetHash, PlanHash};

use crate::canonical::{byte_hash, canonical_json_hash};
use crate::ContractError;

/// Hash an opaque content blob into a `sha256:...` [`ContentHash`].
#[must_use]
pub fn content_hash(bytes: &[u8]) -> ContentHash {
    ContentHash(byte_hash(bytes))
}

/// Whether `value` is a canonical content-hash token: `sha256:` followed by
/// exactly 64 **lowercase** hex digits. The hasher always mints lowercase
/// (`{byte:02x}`), so accepting uppercase would let two distinct strings denote
/// the same digest. This is the single hash-token validator the contract, codegen
/// and replay layers all consult (P1-004).
#[must_use]
pub fn is_canonical_sha256_token(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .chars()
                .all(|ch| ch.is_ascii_digit() || ('a'..='f').contains(&ch))
    })
}

/// The stable material a [`PlanHash`] is computed over (ADR-0009). All fields
/// are owned/primitive so the pure kernel needs no serialization support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanHashMaterial {
    /// Version of the hashing schema itself.
    pub hash_schema_version: u32,
    /// Bundle identity the plan was compiled against.
    pub bundle_id: String,
    /// Bundle content version.
    pub bundle_version: String,
    /// Bundle content hash (`sha256:...`).
    pub bundle_hash: String,
    /// Planner identity.
    pub planner_id: String,
    /// Planner version.
    pub planner_version: String,
    /// Planner build fingerprint.
    pub planner_fingerprint: String,
    /// The action being planned.
    pub action_id: String,
    /// Predicate id.
    pub predicate: String,
    /// Predicate schema version.
    pub predicate_version: u32,
    /// Content hash of the subject snapshot.
    pub subject_fingerprint: String,
    /// Content hash of the circumstance snapshot.
    pub circumstance_fingerprint: String,
    /// Consequence profile (rendered token).
    pub consequence_profile: String,
    /// Lifecycle class (rendered token).
    pub lifecycle_class: String,
    /// Route id derived from the profile.
    pub route_id: String,
    /// Ordered ops.
    pub ops: Vec<CanonicalOp>,
    /// Planned impacts (the same set [`impact_set_hash`] digests).
    pub planned_impacts: Vec<CanonicalImpact>,
    /// Ids of required witnesses.
    pub required_witnesses: Vec<String>,
    /// Required resource claims.
    pub required_claims: Vec<CanonicalClaim>,
    /// Barrier policy token.
    pub barrier_policy: String,
    /// Projection policy token.
    pub projection_policy: String,
}

/// A canonical op inside [`PlanHashMaterial`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalOp {
    /// Op index within the plan.
    pub index: u32,
    /// Op kind token.
    pub kind: String,
    /// Effect signature.
    pub effect: CanonicalEffect,
}

/// A canonical effect signature inside [`CanonicalOp`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalEffect {
    /// Read scopes.
    pub reads: Vec<String>,
    /// Write scopes.
    pub writes: Vec<String>,
    /// Produced fact kinds.
    pub produces: Vec<String>,
    /// Required fact kinds.
    pub requires: Vec<String>,
    /// Invalidated scopes.
    pub invalidates: Vec<String>,
    /// Conflict domains.
    pub conflict_domains: Vec<String>,
    /// Hardness token (`soft` / `hard`).
    pub hardness: String,
}

/// A canonical planned impact (the unit [`impact_set_hash`] digests).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalImpact {
    /// Affected scope.
    pub scope: String,
    /// Hardness token (`soft` / `hard`).
    pub hardness: String,
}

/// A canonical resource claim inside [`PlanHashMaterial`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalClaim {
    /// Resource id.
    pub resource: String,
    /// Claim scope.
    pub scope: String,
    /// Claim mode token.
    pub mode: String,
    /// Claim amount.
    pub amount: u64,
}

impl PlanHashMaterial {
    /// Compute the canonical plan hash for this material.
    ///
    /// # Errors
    /// Returns [`ContractError::Json`] if serialization fails, or
    /// [`ContractError::PlanHash`] if the computed digest is malformed (which
    /// would indicate a bug, not bad input).
    #[must_use = "the computed plan hash must be used"]
    pub fn compute_plan_hash(&self) -> Result<PlanHash, ContractError> {
        let plan_hash = PlanHash::new(canonical_json_hash(self)?)?;
        Ok(plan_hash)
    }
}

/// Compute the impact-set hash that approvals/barriers bind to (I-009).
///
/// # Errors
/// Returns [`ContractError::Json`] if serialization fails.
#[must_use = "the computed impact-set hash must be used"]
pub fn impact_set_hash(impacts: &[CanonicalImpact]) -> Result<ImpactSetHash, ContractError> {
    Ok(ImpactSetHash(canonical_json_hash(&impacts)?))
}

#[cfg(test)]
mod tests {
    use super::is_canonical_sha256_token;

    #[test]
    fn canonical_token_is_lowercase_64_hex() {
        let lower = format!("sha256:{}", "a".repeat(64));
        assert!(is_canonical_sha256_token(&lower));
    }

    #[test]
    fn rejects_uppercase_prefix_todo_and_wrong_length() {
        assert!(!is_canonical_sha256_token(&format!(
            "sha256:{}",
            "A".repeat(64)
        )));
        assert!(!is_canonical_sha256_token(&format!(
            "SHA256:{}",
            "a".repeat(64)
        )));
        assert!(!is_canonical_sha256_token("sha256:TODO"));
        assert!(!is_canonical_sha256_token(&format!(
            "sha256:{}",
            "a".repeat(63)
        )));
        assert!(!is_canonical_sha256_token(&format!(
            "sha256:{}",
            "a".repeat(65)
        )));
        assert!(!is_canonical_sha256_token(&"a".repeat(64)));
    }
}
