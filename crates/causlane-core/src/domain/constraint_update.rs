//! Constraint-update / committed-truth rule (invariant I-010).
//!
//! Observed truth is the single source of truth (ADR-0003) and is immutable once
//! committed. A constraint-plane update may freely change constraint state, but
//! it must **never rewrite an observed-truth fact that is already committed**.
//! This is the pure kernel rule the formal lanes verify (the Verus kernel's
//! `!truth_rewritten` and the Kani `constraint_update_cannot_rewrite_committed_truth_nondet`
//! proof both bind to it).

/// Which observed-truth fact categories are already committed to the journal.
/// Committed truth is immutable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommittedTruth {
    /// A readiness fact has been committed.
    pub readiness_committed: bool,
    /// A promotion fact has been committed.
    pub promotion_committed: bool,
    /// An external-evidence fact has been committed.
    pub evidence_committed: bool,
}

/// The observed-truth fact categories a constraint update would rewrite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstraintUpdate {
    /// The update would rewrite a readiness fact.
    pub rewrites_readiness: bool,
    /// The update would rewrite a promotion fact.
    pub rewrites_promotion: bool,
    /// The update would rewrite an external-evidence fact.
    pub rewrites_evidence: bool,
}

impl ConstraintUpdate {
    /// I-010: whether this update preserves committed truth — it rewrites no fact
    /// category that is already committed. An update that would rewrite any
    /// already-committed category is rejected.
    #[must_use]
    pub fn preserves_committed_truth(self, committed: CommittedTruth) -> bool {
        // For each category: either the update does not rewrite it, or it is not
        // already committed — i.e. never rewrite a committed truth fact.
        (!self.rewrites_readiness || !committed.readiness_committed)
            && (!self.rewrites_promotion || !committed.promotion_committed)
            && (!self.rewrites_evidence || !committed.evidence_committed)
    }
}

#[cfg(test)]
mod tests {
    use super::{CommittedTruth, ConstraintUpdate};

    const NOTHING_COMMITTED: CommittedTruth = CommittedTruth {
        readiness_committed: false,
        promotion_committed: false,
        evidence_committed: false,
    };

    // Before anything is committed, any update is allowed (nothing to rewrite).
    #[test]
    fn update_allowed_when_no_truth_committed() {
        let update = ConstraintUpdate {
            rewrites_readiness: true,
            rewrites_promotion: true,
            rewrites_evidence: true,
        };
        assert!(update.preserves_committed_truth(NOTHING_COMMITTED));
    }

    // An update that does not touch committed categories is allowed; one that
    // rewrites a committed category is rejected.
    #[test]
    fn update_rejected_only_when_it_rewrites_committed_truth() {
        let committed = CommittedTruth {
            readiness_committed: true,
            ..NOTHING_COMMITTED
        };
        // Rewrites a different (uncommitted) category -> allowed.
        let other = ConstraintUpdate {
            rewrites_readiness: false,
            rewrites_promotion: true,
            rewrites_evidence: false,
        };
        assert!(other.preserves_committed_truth(committed));
        // Rewrites the committed readiness fact -> rejected.
        let rewrite = ConstraintUpdate {
            rewrites_readiness: true,
            rewrites_promotion: false,
            rewrites_evidence: false,
        };
        assert!(!rewrite.preserves_committed_truth(committed));
    }
}
