//! Effect and consequence signatures.

/// Names a region of state that effects read from, write to, or invalidate.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Scope(pub String);

/// Names a domain within which two effects may conflict.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConflictDomain(pub String);

/// Kind of observed fact produced by an action (e.g. `release_candidate_promoted`).
///
/// Used by truth anchors and witness selectors to identify *which* fact a
/// projection is derived from or a transition depends on.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FactKind(pub String);

/// Declares the reads, writes, produced/required facts and impacts of an op.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectSignature {
    /// Scopes the op reads from.
    pub reads: Vec<Scope>,
    /// Scopes the op writes to.
    pub writes: Vec<Scope>,
    /// Fact kinds the op produces.
    pub produces: Vec<String>,
    /// Fact kinds the op requires.
    pub requires: Vec<String>,
    /// Scopes the op invalidates.
    pub invalidates: Vec<Scope>,
    /// Domains within which the op may conflict with others.
    pub conflict_domains: Vec<ConflictDomain>,
    /// Whether the op's impact is soft or hard.
    pub hardness: ImpactHardness,
}

/// Whether an effect's impact is reversible (soft) or irreversible (hard).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpactHardness {
    /// Reversible / low-stakes impact.
    Soft,
    /// Irreversible / high-stakes impact.
    Hard,
}

impl EffectSignature {
    /// An empty signature describing a pure projection (no reads, writes or impacts).
    #[must_use]
    pub fn projection_only() -> Self {
        Self {
            reads: Vec::new(),
            writes: Vec::new(),
            produces: Vec::new(),
            requires: Vec::new(),
            invalidates: Vec::new(),
            conflict_domains: Vec::new(),
            hardness: ImpactHardness::Soft,
        }
    }

    /// Whether this effect mutates state (has writes or is a hard impact).
    #[must_use]
    pub fn is_mutable(&self) -> bool {
        !self.writes.is_empty() || self.hardness == ImpactHardness::Hard
    }
}

#[cfg(test)]
mod tests {
    use super::{EffectSignature, ImpactHardness, Scope};

    // is_mutable() = (has writes) OR (Hard impact) — the predicate that decides
    // whether an op mutates state (and therefore needs a barrier/lease; a read-only
    // op does not). Exhaustive over (writes empty/non-empty) x (Soft/Hard).
    #[test]
    fn is_mutable_is_writes_or_hard() {
        let read_only = EffectSignature {
            reads: vec![Scope("s".to_owned())],
            writes: Vec::new(),
            produces: Vec::new(),
            requires: Vec::new(),
            invalidates: Vec::new(),
            conflict_domains: Vec::new(),
            hardness: ImpactHardness::Soft,
        };
        assert!(!read_only.is_mutable());

        let writes_soft = EffectSignature {
            writes: vec![Scope("w".to_owned())],
            ..read_only.clone()
        };
        assert!(writes_soft.is_mutable());

        let hard_no_writes = EffectSignature {
            hardness: ImpactHardness::Hard,
            ..read_only.clone()
        };
        assert!(hard_no_writes.is_mutable());

        let writes_hard = EffectSignature {
            writes: vec![Scope("w".to_owned())],
            hardness: ImpactHardness::Hard,
            ..read_only.clone()
        };
        assert!(writes_hard.is_mutable());
    }

    // projection_only() is an empty, soft, read-only signature, so it is not mutable.
    #[test]
    fn projection_only_is_empty_and_not_mutable() {
        let projection = EffectSignature::projection_only();
        assert!(projection.reads.is_empty());
        assert!(projection.writes.is_empty());
        assert!(projection.produces.is_empty());
        assert!(projection.requires.is_empty());
        assert!(projection.invalidates.is_empty());
        assert!(projection.conflict_domains.is_empty());
        assert_eq!(projection.hardness, ImpactHardness::Soft);
        assert!(!projection.is_mutable());
    }
}
