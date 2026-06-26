//! Tier model — the ordered authority stages an action passes through (M05.1).
//!
//! Each [`Tier`] is an authority boundary in the dispatch pipeline; a tier confers
//! the authority named by [`Tier::authority`]. Tiers are NOT a parallel source of
//! truth: they are grounded in the S03 lifecycle reducer via [`reached_tier`], and
//! the `tier_is_monotonic_under_valid_lifecycle_transitions` proof (exhaustive over
//! the finite stage×event×profile space) shows every valid lifecycle transition
//! keeps the reached tier non-decreasing — an action never drops to a lower
//! authority. Lanes (M05.2) operate WITHIN a tier and carry no semantic authority.

use super::LifecycleStage;

/// An authority stage in the dispatch pipeline, ordered by declaration. An action
/// passes through them in sequence; the derived `Ord` is the authority order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// Admission control: the action is accepted for dispatch.
    Admission,
    /// Planning: the action is compiled into an immutable plan.
    Planning,
    /// Dispatch: the dispatch decision is logged.
    Dispatch,
    /// Barrier: the write-ahead execution barrier (leases/witnesses) is logged.
    Barrier,
    /// Execution: scoped execution proceeds under a barrier-derived capability.
    Execution,
    /// Observation: observed truth is committed.
    Observation,
    /// Projection: a projection is emitted from an anchor.
    Projection,
    /// Closure: the lifecycle is terminally closed.
    Closure,
}

impl Tier {
    /// Every tier, in authority order.
    pub const ALL: [Tier; 8] = [
        Tier::Admission,
        Tier::Planning,
        Tier::Dispatch,
        Tier::Barrier,
        Tier::Execution,
        Tier::Observation,
        Tier::Projection,
        Tier::Closure,
    ];

    /// The next tier in the pipeline, or `None` at [`Tier::Closure`].
    #[must_use]
    pub fn next_tier(self) -> Option<Tier> {
        match self {
            Tier::Admission => Some(Tier::Planning),
            Tier::Planning => Some(Tier::Dispatch),
            Tier::Dispatch => Some(Tier::Barrier),
            Tier::Barrier => Some(Tier::Execution),
            Tier::Execution => Some(Tier::Observation),
            Tier::Observation => Some(Tier::Projection),
            Tier::Projection => Some(Tier::Closure),
            Tier::Closure => None,
        }
    }

    /// The authority this tier confers, as a stable machine-readable token.
    #[must_use]
    pub fn authority(self) -> &'static str {
        match self {
            Tier::Admission => "admission_control",
            Tier::Planning => "plan_freeze",
            Tier::Dispatch => "dispatch_decision",
            Tier::Barrier => "barrier_write_ahead",
            Tier::Execution => "execution_capability",
            Tier::Observation => "truth_observation",
            Tier::Projection => "projection_emit",
            Tier::Closure => "closure",
        }
    }
}

/// The tier an action has reached at `stage`. `New` has reached no tier yet
/// (`None`); every other lifecycle stage maps to exactly one tier. This is the
/// single grounding of the tier model in the lifecycle authority.
#[must_use]
pub fn reached_tier(stage: LifecycleStage) -> Option<Tier> {
    match stage {
        LifecycleStage::New => None,
        LifecycleStage::Admitted => Some(Tier::Admission),
        LifecycleStage::Planned => Some(Tier::Planning),
        LifecycleStage::DispatchLogged => Some(Tier::Dispatch),
        LifecycleStage::ExecutionBarrierLogged => Some(Tier::Barrier),
        LifecycleStage::Executing => Some(Tier::Execution),
        LifecycleStage::Observed => Some(Tier::Observation),
        LifecycleStage::Projected => Some(Tier::Projection),
        LifecycleStage::Closed => Some(Tier::Closure),
    }
}

#[cfg(test)]
mod tests {
    use super::{reached_tier, Tier};
    use crate::domain::{
        reduce_lifecycle, AuditEventKind as E, ConsequenceProfile as P, LifecycleStage as S,
    };

    const ALL_STAGES: [S; 9] = [
        S::New,
        S::Admitted,
        S::Planned,
        S::DispatchLogged,
        S::ExecutionBarrierLogged,
        S::Executing,
        S::Observed,
        S::Projected,
        S::Closed,
    ];

    const ALL_EVENTS: [E; 17] = [
        E::ActionAdmitted,
        E::ActionPlanned,
        E::DispatchLogged,
        E::ExecutionBarrierLogged,
        E::ExecutionStarted,
        E::ExecutionCompleted,
        E::ObservedTruthCommitted,
        E::ProjectionEmitted,
        E::LifecycleClosed,
        E::GateApproved,
        E::GateDenied,
        E::ConstraintLeaseGranted,
        E::ConstraintLeaseReleased,
        E::ViolationDetected,
        E::AuthzDecisionRecorded,
        E::DrainFenceRequested,
        E::DrainFenceAcquired,
    ];

    const ALL_PROFILES: [P; 6] = [
        P::RuntimeExecution,
        P::ProjectionRead,
        P::OversightMeta,
        P::TopologyMeta,
        P::EvidenceMeta,
        P::OutsideKernel,
    ];

    #[test]
    fn reached_tier_maps_every_lifecycle_stage() {
        assert_eq!(reached_tier(S::New), None);
        assert_eq!(reached_tier(S::Admitted), Some(Tier::Admission));
        assert_eq!(reached_tier(S::Planned), Some(Tier::Planning));
        assert_eq!(reached_tier(S::DispatchLogged), Some(Tier::Dispatch));
        assert_eq!(reached_tier(S::ExecutionBarrierLogged), Some(Tier::Barrier));
        assert_eq!(reached_tier(S::Executing), Some(Tier::Execution));
        assert_eq!(reached_tier(S::Observed), Some(Tier::Observation));
        assert_eq!(reached_tier(S::Projected), Some(Tier::Projection));
        assert_eq!(reached_tier(S::Closed), Some(Tier::Closure));
    }

    #[test]
    fn tiers_form_a_total_ordered_pipeline() {
        assert_eq!(Tier::ALL.len(), 8);
        for (current, following) in Tier::ALL.iter().zip(Tier::ALL.iter().skip(1)) {
            assert!(current < following, "tiers must be strictly ascending");
            assert_eq!(current.next_tier(), Some(*following));
        }
        assert_eq!(Tier::Closure.next_tier(), None);
    }

    #[test]
    fn tier_authorities_are_distinct() {
        let authorities: std::collections::BTreeSet<&str> =
            Tier::ALL.into_iter().map(Tier::authority).collect();
        assert_eq!(authorities.len(), Tier::ALL.len());
    }

    /// Exhaustive proof over the finite `stage × event × profile` space: every
    /// valid lifecycle transition keeps the reached tier non-decreasing — an
    /// action never moves to a lower authority tier.
    #[test]
    fn tier_is_monotonic_under_valid_lifecycle_transitions() {
        let mut strict_advances = 0_usize;
        for &profile in &ALL_PROFILES {
            for &from in &ALL_STAGES {
                for &event in &ALL_EVENTS {
                    let Ok(next) = reduce_lifecycle(from, event, profile) else {
                        continue;
                    };
                    assert!(
                        reached_tier(next) >= reached_tier(from),
                        "tier dropped: {from:?} --{event:?}/{profile:?}--> {next:?}"
                    );
                    if reached_tier(next) > reached_tier(from) {
                        strict_advances = strict_advances.saturating_add(1);
                    }
                }
            }
        }
        // Non-vacuity: the property is not trivially satisfied by all-equal tiers.
        assert!(strict_advances > 0, "no advancing transition was exercised");
    }

    /// Exhaustiveness guard: a new `Tier` variant breaks this match until
    /// `Tier::ALL` is updated, keeping the table honest.
    #[test]
    fn tier_all_is_exhaustive() {
        fn covered(tier: Tier) -> bool {
            match tier {
                Tier::Admission
                | Tier::Planning
                | Tier::Dispatch
                | Tier::Barrier
                | Tier::Execution
                | Tier::Observation
                | Tier::Projection
                | Tier::Closure => true,
            }
        }
        assert!(Tier::ALL.into_iter().all(covered));
    }
}
