//! Drain fence acquisition rule (invariant I-007).
//!
//! A drain fence over a scope may be acquired only when no lease *actively*
//! overlaps that scope, where "active" means granted, not released, **and not yet
//! expired** at the fence's acquisition time. This is the expiry-aware rule the
//! kernel authority exposes ([`crate::contract::DrainSemantics::can_acquire_fence`]),
//! which the replay oracle routes through. The bounded [`DrainFenceCheck`] models
//! the same rule over a fixed pair of lease slots so the Kani lane can prove it
//! over its full input space.

/// A fixed pair of lease slots' `(overlaps-fence, still-active, expired)` status,
/// used to prove the I-007 rule over a bounded space: a fence is acquirable iff no
/// slot is overlapping the fence scope while still actively (granted, not released)
/// and not-yet-expired holding it. An expired overlapping lease no longer blocks.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrainFenceCheck {
    /// The first lease slot overlaps the fence scope.
    pub left_overlaps: bool,
    /// The first lease slot is still active (granted, not released).
    pub left_active: bool,
    /// The first lease slot has expired at the fence's acquisition time.
    pub left_expired: bool,
    /// The second lease slot overlaps the fence scope.
    pub right_overlaps: bool,
    /// The second lease slot is still active.
    pub right_active: bool,
    /// The second lease slot has expired at the fence's acquisition time.
    pub right_expired: bool,
}

impl DrainFenceCheck {
    /// I-007 (expiry-aware): a fence is acquirable iff neither slot is overlapping
    /// the fence scope while still active and not-yet-expired — an expired lease no
    /// longer blocks the drain.
    #[must_use]
    pub fn fence_acquirable(self) -> bool {
        (!self.left_overlaps || !self.left_active || self.left_expired)
            && (!self.right_overlaps || !self.right_active || self.right_expired)
    }
}

#[cfg(test)]
mod tests {
    use super::DrainFenceCheck;

    // A fence is blocked by an active, non-expired overlapping lease, and only that;
    // an expired overlapping lease no longer blocks.
    #[test]
    fn fence_blocked_only_by_active_nonexpired_overlap() {
        let clear = DrainFenceCheck {
            left_overlaps: false,
            left_active: true,
            left_expired: false,
            right_overlaps: true,
            right_active: false,
            right_expired: false,
        };
        assert!(clear.fence_acquirable());
        let blocked = DrainFenceCheck {
            left_overlaps: true,
            left_active: true,
            left_expired: false,
            right_overlaps: false,
            right_active: false,
            right_expired: false,
        };
        assert!(!blocked.fence_acquirable());
        // The same overlapping, active lease no longer blocks once it has expired.
        let expired = DrainFenceCheck {
            left_overlaps: true,
            left_active: true,
            left_expired: true,
            right_overlaps: false,
            right_active: false,
            right_expired: false,
        };
        assert!(expired.fence_acquirable());
    }
}
