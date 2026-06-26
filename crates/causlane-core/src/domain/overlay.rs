//! Overlay obligation rule (invariant I-004).
//!
//! An overlay (e.g. a stricter environment or tenant profile layered over a base
//! predicate) may only **strengthen** obligations — the overlaid obligation set
//! must be a superset of, or equal to, the base. An overlay can never *weaken*
//! the base by dropping a required obligation. This is the pure kernel rule the
//! formal lanes verify (the Verus kernel's `!overlay_weakened` and the Kani
//! `overlay_never_weakens_obligations_nondet` proof both bind to it).

/// The obligation categories an overlay may strengthen but never weaken (I-004).
/// Each flag records whether that category of obligation is *required*; mapping
/// onto the bundle, these are: a typed witness requirement, a resource claim, an
/// authorization decision, an execution barrier and an anchored projection.
///
/// The five booleans are intentionally explicit obligation categories (they
/// mirror the formal I-004 truth table), not a packed enum.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObligationSet {
    /// A typed witness is required.
    pub requires_witness: bool,
    /// A resource claim is required.
    pub requires_claim: bool,
    /// An authorization decision is required.
    pub requires_authz: bool,
    /// A write-ahead execution barrier is required.
    pub requires_barrier: bool,
    /// An anchored projection (truth anchor) is required.
    pub requires_anchor: bool,
}

impl ObligationSet {
    /// I-004: whether `overlaid` preserves (is a superset of) this base set — for
    /// every category the base requires, the overlay must still require it. An
    /// overlay that clears any required obligation is *weaker* and rejected.
    #[must_use]
    pub fn preserved_by(self, overlaid: ObligationSet) -> bool {
        (!self.requires_witness || overlaid.requires_witness)
            && (!self.requires_claim || overlaid.requires_claim)
            && (!self.requires_authz || overlaid.requires_authz)
            && (!self.requires_barrier || overlaid.requires_barrier)
            && (!self.requires_anchor || overlaid.requires_anchor)
    }
}

#[cfg(test)]
mod tests {
    use super::ObligationSet;

    const NONE: ObligationSet = ObligationSet {
        requires_witness: false,
        requires_claim: false,
        requires_authz: false,
        requires_barrier: false,
        requires_anchor: false,
    };

    // An empty base is preserved by any overlay (nothing to weaken).
    #[test]
    fn empty_base_is_always_preserved() {
        let all = ObligationSet {
            requires_witness: true,
            requires_claim: true,
            requires_authz: true,
            requires_barrier: true,
            requires_anchor: true,
        };
        assert!(NONE.preserved_by(NONE));
        assert!(NONE.preserved_by(all));
    }

    // Strengthening (overlay adds obligations) is allowed; weakening is not.
    #[test]
    fn strengthening_is_allowed_weakening_is_rejected() {
        let base = ObligationSet {
            requires_witness: true,
            requires_barrier: true,
            ..NONE
        };
        // Overlay keeps both and adds authz -> preserved.
        let stronger = ObligationSet {
            requires_authz: true,
            ..base
        };
        assert!(base.preserved_by(stronger));
        // Overlay drops the witness obligation -> weaker -> rejected.
        let weaker = ObligationSet {
            requires_witness: false,
            ..base
        };
        assert!(!base.preserved_by(weaker));
    }
}
