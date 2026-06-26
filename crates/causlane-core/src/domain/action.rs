//! Action grammar domain types.

use core::fmt;

use super::EffectSignature;

/// Identifies an action grammar entry (a verb in the dispatch vocabulary).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionId(pub String);

/// Identifies a predicate (the typed shape an action operates over).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PredicateId(pub String);

/// Canonical plan hash: `sha256:` followed by 64 lowercase hex characters.
///
/// The inner value is private so that a [`PlanHash`] can only exist in a
/// well-formed state. Construct one via [`PlanHash::new`]; the placeholder
/// `sha256:TODO` is rejected (see ADR-0009).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlanHash(String);

/// Reasons a string is not a valid [`PlanHash`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanHashError {
    /// The value did not start with the required `sha256:` prefix.
    MissingPrefix,
    /// The digest part was the `TODO` placeholder.
    Placeholder,
    /// The digest part did not have exactly 64 characters.
    BadLength {
        /// Expected digest length (64).
        expected: usize,
        /// Actual digest length found.
        got: usize,
    },
    /// The digest part contained a non lowercase-hex character.
    NonHex,
}

impl PlanHash {
    /// The fixed digest length (SHA-256 as lowercase hex).
    pub const DIGEST_LEN: usize = 64;

    /// Parse and validate a canonical plan hash of the form
    /// `sha256:<64 lowercase hex>`.
    ///
    /// # Errors
    /// Returns [`PlanHashError`] if the prefix, length, or alphabet is wrong,
    /// or if the digest is the `TODO` placeholder.
    #[must_use = "a validated PlanHash must be used; ignoring it discards the validation"]
    pub fn new(value: impl Into<String>) -> Result<Self, PlanHashError> {
        let value = value.into();
        let digest = value
            .strip_prefix("sha256:")
            .ok_or(PlanHashError::MissingPrefix)?;
        if digest == "TODO" {
            return Err(PlanHashError::Placeholder);
        }
        if digest.len() != Self::DIGEST_LEN {
            return Err(PlanHashError::BadLength {
                expected: Self::DIGEST_LEN,
                got: digest.len(),
            });
        }
        if !digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(PlanHashError::NonHex);
        }
        Ok(Self(value))
    }

    /// Borrow the canonical string form (`sha256:...`).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PlanHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Identifier of a route derived from a consequence profile.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RouteId(pub String);

/// Run-scoped id grouping all events of a single action invocation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CorrelationId(pub String);

/// An inbound request to perform an action against a subject and circumstance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionCall {
    /// The action being invoked.
    pub action_id: ActionId,
    /// The predicate the action operates over.
    pub predicate: PredicateId,
    /// Reference to the subject the action acts on.
    pub subject_ref: String,
    /// Reference to the circumstance the action is evaluated in.
    pub circumstance_ref: String,
    /// Run-scoped correlation id for this invocation.
    pub correlation_id: CorrelationId,
}

/// A compiled plan for an action call: its consequence profile and ordered ops.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionPlan {
    /// The action this plan was compiled for.
    pub action_id: ActionId,
    /// The predicate this plan was compiled for.
    pub predicate: PredicateId,
    /// Canonical hash of this plan (ADR-0009).
    pub plan_hash: PlanHash,
    /// The consequence profile classifying this plan's lifecycle.
    pub consequence_profile: ConsequenceProfile,
    /// Ordered operations the plan will execute.
    pub ops: Vec<Op>,
}

/// A single planned operation within an [`ActionPlan`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Op {
    /// Position of this op within the plan.
    pub index: u32,
    /// The operation kind (an executor-specific verb).
    pub kind: String,
    /// The effect signature describing this op's reads, writes and impacts.
    pub effect: EffectSignature,
}

/// Classifies the kind of consequence an action has, driving its lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConsequenceProfile {
    /// Hard-effect path requiring a barrier and observed truth.
    RuntimeExecution,
    /// Derived-read path producing a projection, no barrier.
    ProjectionRead,
    /// Oversight meta path.
    OversightMeta,
    /// Topology meta path.
    TopologyMeta,
    /// Evidence meta path.
    EvidenceMeta,
    /// Action whose consequences fall outside the kernel's responsibility.
    OutsideKernel,
}

impl fmt::Display for ConsequenceProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::{PlanHash, PlanHashError};

    // A canonical `sha256:<64 lowercase hex>` is accepted and round-trips (ADR-0009).
    #[test]
    fn canonical_plan_hash_is_accepted() -> Result<(), PlanHashError> {
        let valid = format!("sha256:{}", "1".repeat(PlanHash::DIGEST_LEN));
        assert_eq!(PlanHash::new(valid.clone())?.as_str(), valid);
        Ok(())
    }

    // Every malformed shape is rejected with its specific reason (fail-closed,
    // ADR-0009 / P1-004): the validator is the single authority for plan-hash form.
    #[test]
    fn malformed_plan_hashes_are_rejected() {
        assert!(matches!(
            PlanHash::new("deadbeef"),
            Err(PlanHashError::MissingPrefix)
        ));
        assert!(matches!(
            PlanHash::new("sha256:TODO"),
            Err(PlanHashError::Placeholder)
        ));
        assert!(matches!(
            PlanHash::new(format!("sha256:{}", "1".repeat(63))),
            Err(PlanHashError::BadLength {
                expected: 64,
                got: 63
            })
        ));
        // 64 chars but uppercase hex is not the canonical lowercase alphabet.
        assert!(matches!(
            PlanHash::new(format!("sha256:{}", "A".repeat(64))),
            Err(PlanHashError::NonHex)
        ));
        // 64 chars but a non-hex letter.
        assert!(matches!(
            PlanHash::new(format!("sha256:{}", "z".repeat(64))),
            Err(PlanHashError::NonHex)
        ));
    }
}
