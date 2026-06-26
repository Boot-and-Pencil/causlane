//! Lane model — capacity/capability/fairness slots within a tier (M05.2).
//!
//! A [`Lane`] is bound to exactly one [`Tier`] and selects *where* already-allowed
//! work runs: it carries a concurrency [`LaneCapacity`], an optional provided
//! capability class, and a fairness weight. A lane confers **no semantic
//! authority** — [`lane_admits`] can only place work that is already at the
//! lane's tier (the tier authority gate runs first), so it can never admit
//! cross-tier work or bypass a lifecycle guard. `lane_never_grants_cross_tier_authority`
//! proves this exhaustively over `Tier × Tier`. Fairness *ordering* is a runtime
//! scheduling concern; the kernel only carries the weight.

use super::Tier;

/// Unique lane identifier.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LaneId(pub String);

/// A lane's concurrency budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LaneCapacity {
    /// No concurrency limit.
    Unbounded,
    /// At most `n` actions may be active in the lane at once.
    Bounded(u32),
}

impl LaneCapacity {
    /// Whether the lane has room for one more action given `active` already in it.
    #[must_use]
    pub fn has_room(self, active: u32) -> bool {
        match self {
            LaneCapacity::Unbounded => true,
            LaneCapacity::Bounded(limit) => active < limit,
        }
    }
}

/// A capacity/capability/fairness slot inside a [`Tier`]. Carries no authority.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Lane {
    /// Unique id.
    pub lane_id: LaneId,
    /// The tier this lane operates within; a lane only hosts work at this tier.
    pub tier: Tier,
    /// Concurrency budget.
    pub capacity: LaneCapacity,
    /// The capability class this lane provides (`None` = general-purpose).
    pub capability: Option<String>,
    /// Relative fairness share for runtime scheduling (no authority).
    pub fairness_weight: u32,
}

impl Lane {
    /// Whether this lane can satisfy an action's capability requirement. An action
    /// that requires nothing (`None`) runs in any lane; an action requiring `req`
    /// runs only in a lane that provides exactly `req`.
    #[must_use]
    pub fn accepts_capability(&self, op_requires: Option<&str>) -> bool {
        match op_requires {
            None => true,
            Some(required) => self.capability.as_deref() == Some(required),
        }
    }
}

/// Why a lane refused an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaneRejection {
    /// The action's tier does not match the lane's tier (the authority gate).
    WrongTier {
        /// The lane's tier.
        lane_tier: Tier,
        /// The action's tier.
        op_tier: Tier,
    },
    /// The lane does not provide the capability the action requires.
    CapabilityMismatch,
    /// The lane is at its concurrency budget.
    CapacityExhausted {
        /// The lane's bound.
        capacity: u32,
        /// Active actions in the lane.
        active: u32,
    },
}

/// Whether a lane admits an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaneAdmission {
    /// The lane places the action.
    Admit,
    /// The lane refused the action.
    Reject(LaneRejection),
}

/// Decide whether `lane` admits an action currently at `op_tier` requiring
/// `op_requires`, given `active` actions already in the lane. The tier authority
/// gate runs FIRST: an action whose tier differs from the lane's is always
/// rejected with [`LaneRejection::WrongTier`], so a lane can never grant
/// cross-tier authority. Only same-tier, capability-compatible work within
/// capacity is admitted.
#[must_use]
pub fn lane_admits(
    lane: &Lane,
    active: u32,
    op_tier: Tier,
    op_requires: Option<&str>,
) -> LaneAdmission {
    if op_tier != lane.tier {
        return LaneAdmission::Reject(LaneRejection::WrongTier {
            lane_tier: lane.tier,
            op_tier,
        });
    }
    if !lane.accepts_capability(op_requires) {
        return LaneAdmission::Reject(LaneRejection::CapabilityMismatch);
    }
    match lane.capacity {
        LaneCapacity::Unbounded => LaneAdmission::Admit,
        LaneCapacity::Bounded(capacity) => {
            if active < capacity {
                LaneAdmission::Admit
            } else {
                LaneAdmission::Reject(LaneRejection::CapacityExhausted { capacity, active })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{lane_admits, Lane, LaneAdmission, LaneCapacity, LaneId, LaneRejection};
    use crate::domain::Tier;

    const CAPABILITIES: [Option<&str>; 3] = [None, Some("a"), Some("b")];

    fn lane(tier: Tier, capacity: LaneCapacity, capability: Option<&str>) -> Lane {
        Lane {
            lane_id: LaneId("L".to_owned()),
            tier,
            capacity,
            capability: capability.map(str::to_owned),
            fairness_weight: 1,
        }
    }

    #[test]
    fn capacity_has_room_at_the_bound() {
        assert!(LaneCapacity::Unbounded.has_room(1_000_000));
        assert!(LaneCapacity::Bounded(2).has_room(0));
        assert!(LaneCapacity::Bounded(2).has_room(1));
        assert!(!LaneCapacity::Bounded(2).has_room(2));
        assert!(!LaneCapacity::Bounded(0).has_room(0));
    }

    #[test]
    fn capability_matching_is_exact() {
        let general = lane(Tier::Execution, LaneCapacity::Unbounded, None);
        assert!(general.accepts_capability(None));
        assert!(!general.accepts_capability(Some("gpu")));
        let gpu = lane(Tier::Execution, LaneCapacity::Unbounded, Some("gpu"));
        assert!(gpu.accepts_capability(None));
        assert!(gpu.accepts_capability(Some("gpu")));
        assert!(!gpu.accepts_capability(Some("cpu")));
    }

    #[test]
    fn rejections_name_the_specific_cause() {
        let l = lane(Tier::Execution, LaneCapacity::Bounded(1), Some("gpu"));
        assert_eq!(
            lane_admits(&l, 0, Tier::Barrier, None),
            LaneAdmission::Reject(LaneRejection::WrongTier {
                lane_tier: Tier::Execution,
                op_tier: Tier::Barrier,
            })
        );
        assert_eq!(
            lane_admits(&l, 0, Tier::Execution, Some("cpu")),
            LaneAdmission::Reject(LaneRejection::CapabilityMismatch)
        );
        assert_eq!(
            lane_admits(&l, 1, Tier::Execution, Some("gpu")),
            LaneAdmission::Reject(LaneRejection::CapacityExhausted {
                capacity: 1,
                active: 1,
            })
        );
        assert_eq!(
            lane_admits(&l, 0, Tier::Execution, Some("gpu")),
            LaneAdmission::Admit
        );
    }

    /// No semantic authority: exhaustive over `Tier × Tier` (bounded over capacity
    /// / active / capability), an admit always implies the action was already at
    /// the lane's tier, and any cross-tier action is always rejected as
    /// `WrongTier` — the lane can never bypass the tier authority.
    #[test]
    fn lane_never_grants_cross_tier_authority() {
        let mut admits = 0_usize;
        let mut cross_tier_rejections = 0_usize;
        let capacities = [
            LaneCapacity::Unbounded,
            LaneCapacity::Bounded(0),
            LaneCapacity::Bounded(1),
            LaneCapacity::Bounded(2),
        ];
        for &lane_tier in &Tier::ALL {
            for &op_tier in &Tier::ALL {
                for capacity in capacities {
                    for active in 0_u32..=3 {
                        for &lane_cap in &CAPABILITIES {
                            for &op_req in &CAPABILITIES {
                                let l = lane(lane_tier, capacity, lane_cap);
                                let result = lane_admits(&l, active, op_tier, op_req);
                                if op_tier == lane_tier {
                                    assert!(
                                        !matches!(
                                            result,
                                            LaneAdmission::Reject(LaneRejection::WrongTier { .. })
                                        ),
                                        "same-tier work spuriously rejected as WrongTier"
                                    );
                                } else {
                                    assert!(
                                        matches!(
                                            result,
                                            LaneAdmission::Reject(LaneRejection::WrongTier { .. })
                                        ),
                                        "cross-tier work not rejected — lane bypassed tier authority"
                                    );
                                    cross_tier_rejections = cross_tier_rejections.saturating_add(1);
                                }
                                if matches!(result, LaneAdmission::Admit) {
                                    assert_eq!(op_tier, lane_tier);
                                    assert!(l.accepts_capability(op_req));
                                    assert!(l.capacity.has_room(active));
                                    admits = admits.saturating_add(1);
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(admits > 0, "no admitting case exercised");
        assert!(
            cross_tier_rejections > 0,
            "no cross-tier rejection exercised"
        );
    }

    #[test]
    fn rejection_variants_are_exhaustive() {
        fn covered(rejection: LaneRejection) -> bool {
            match rejection {
                LaneRejection::WrongTier { .. }
                | LaneRejection::CapabilityMismatch
                | LaneRejection::CapacityExhausted { .. } => true,
            }
        }
        assert!(covered(LaneRejection::CapabilityMismatch));
    }
}
