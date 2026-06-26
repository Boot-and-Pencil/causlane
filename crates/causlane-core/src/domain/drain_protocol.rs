//! Drain/fence protocol (M05.6).
//!
//! The drain protocol quiesces a region so a fence can be acquired safely. The
//! single-scope fence-acquisition rule (invariant I-007) lives in
//! [`crate::domain::drain`] / [`crate::contract::DrainSemantics::can_acquire_fence`];
//! this module adds the broader protocol *on top of* that authority without
//! duplicating it:
//!   - **domain/global targets** — a drain quiesces one [`DrainTarget::Domain`]
//!     (a scope) or everything ([`DrainTarget::Global`]);
//!   - **safe points** — [`at_safe_point`] decides when a target's region is
//!     quiesced; the domain case *delegates* to the I-007 authority, the global
//!     case is the same rule generalized to every scope;
//!   - **disjoint domains** — [`drains_independent`] decides when two drains may
//!     proceed in parallel (their targets do not overlap);
//!   - **frozen sidecars** — [`op_admissible_during_drain`] keeps a read-only
//!     sidecar admissible while the region is frozen, blocking only mutable ops
//!     that write into the drained region.
//!
//! Drain epochs reuse [`ConstraintEpoch`]; a [`DrainRequest`] governs admissions
//! only from its own epoch onward (future-only, consistent with I-010). Wiring a
//! drain into a `Freeze` constraint and rebuilding the frontier on the epoch bump
//! is M05.7; surfacing the blocking reason is M05.8.

use crate::contract::{DrainSemantics, ScopeOverlap};

use super::{ConstraintEpoch, EffectSignature, LeaseRef, Scope, Timestamp};

/// What a drain quiesces: one domain (a scope) or the whole system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrainTarget {
    /// A drain over a single scope.
    Domain(Scope),
    /// A drain over every scope.
    Global,
}

impl DrainTarget {
    /// Whether this drain covers `scope`.
    #[must_use]
    pub fn covers(&self, scope: &Scope, oracle: &impl ScopeOverlap) -> bool {
        match self {
            DrainTarget::Global => true,
            DrainTarget::Domain(target) => oracle.overlaps(target, scope),
        }
    }
}

/// A drain recorded at a constraint epoch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrainRequest {
    /// What the drain quiesces.
    pub target: DrainTarget,
    /// The constraint epoch the drain is opened in.
    pub epoch: ConstraintEpoch,
}

impl DrainRequest {
    /// A drain governs an admission only from its own epoch onward (future-only,
    /// consistent with I-010): an op admitted in an earlier epoch predates it.
    #[must_use]
    pub fn governs(&self, admission_epoch: ConstraintEpoch) -> bool {
        admission_epoch >= self.epoch
    }
}

/// Whether `target`'s region is at a safe point — quiesced, with no active,
/// non-expired lease overlapping it, so a fence may be acquired.
///
/// The domain case delegates to the I-007 authority
/// ([`DrainSemantics::can_acquire_fence`]) so there is a single rule; the global
/// case is that rule generalized to every scope (no lease is active and
/// non-expired anywhere).
#[must_use]
pub fn at_safe_point(
    target: &DrainTarget,
    active_leases: &[LeaseRef],
    now: Timestamp,
    contracts: &impl DrainSemantics,
) -> bool {
    match target {
        DrainTarget::Domain(scope) => contracts.can_acquire_fence(scope, active_leases, now),
        DrainTarget::Global => active_leases
            .iter()
            .all(|lease| lease.expires_at.is_some_and(|expiry| expiry.0 <= now.0)),
    }
}

/// Whether two drains are independent — no ordering between them, safe to acquire
/// in parallel — which holds iff their targets are disjoint. A [`DrainTarget::Global`]
/// drain touches every scope, so it is never independent of another drain.
#[must_use]
pub fn drains_independent(
    left: &DrainTarget,
    right: &DrainTarget,
    oracle: &impl ScopeOverlap,
) -> bool {
    match (left, right) {
        (DrainTarget::Global, _) | (_, DrainTarget::Global) => false,
        (DrainTarget::Domain(a), DrainTarget::Domain(b)) => !oracle.overlaps(a, b),
    }
}

/// Whether an op may still run while `target` is draining. A read-only sidecar
/// (no writes, soft impact — `!is_mutable()`) stays admissible: it is *frozen*,
/// reading committed truth and mutating nothing. A mutable op is admissible only
/// if none of its write scopes fall in the drained region.
#[must_use]
pub fn op_admissible_during_drain(
    effect: &EffectSignature,
    target: &DrainTarget,
    oracle: &impl ScopeOverlap,
) -> bool {
    if !effect.is_mutable() {
        return true;
    }
    !effect
        .writes
        .iter()
        .any(|scope| target.covers(scope, oracle))
}

#[cfg(test)]
mod tests {
    use super::{
        at_safe_point, drains_independent, op_admissible_during_drain, DrainRequest, DrainTarget,
    };
    use crate::{
        ActionId, AuditEventId, ClaimMode, ConstraintEpoch, DrainSemantics, EffectSignature,
        KernelContracts, LeaseId, LeaseRef, PlanHash, PlanHashError, ResourceId, Scope, Timestamp,
    };

    type TestResult = Result<(), PlanHashError>;

    fn plan_hash() -> Result<PlanHash, PlanHashError> {
        PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    }

    fn lease(scope_name: &str, expires_at: Option<u64>, plan: &PlanHash) -> LeaseRef {
        LeaseRef {
            lease_id: LeaseId("l".to_owned()),
            resource: ResourceId("r".to_owned()),
            scope: Scope(scope_name.to_owned()),
            mode: ClaimMode::ExclusiveWrite,
            amount: 1,
            holder_action_id: ActionId("act".to_owned()),
            holder_plan_hash: plan.clone(),
            holder_op_index: Some(0),
            epoch: ConstraintEpoch(0),
            expires_at: expires_at.map(Timestamp),
            lease_event_id: AuditEventId("evt".to_owned()),
        }
    }

    fn scope(name: &str) -> Scope {
        Scope(name.to_owned())
    }

    fn domain(name: &str) -> DrainTarget {
        DrainTarget::Domain(scope(name))
    }

    fn mutable_writing(scope_name: &str) -> EffectSignature {
        EffectSignature {
            writes: vec![scope(scope_name)],
            ..EffectSignature::projection_only()
        }
    }

    #[test]
    fn target_covers_only_its_domain_global_covers_all() {
        let oracle = KernelContracts;
        assert!(domain("s1").covers(&scope("s1"), &oracle));
        assert!(!domain("s1").covers(&scope("s2"), &oracle));
        assert!(DrainTarget::Global.covers(&scope("anything"), &oracle));
    }

    #[test]
    fn an_active_nonexpired_overlap_blocks_a_domain_safe_point() -> TestResult {
        let plan = plan_hash()?;
        let oracle = KernelContracts;
        let now = Timestamp(100);
        // Active (no expiry) overlapping lease -> not safe.
        assert!(!at_safe_point(
            &domain("s1"),
            &[lease("s1", None, &plan)],
            now,
            &oracle
        ));
        // Expired overlapping lease -> safe again.
        assert!(at_safe_point(
            &domain("s1"),
            &[lease("s1", Some(50), &plan)],
            now,
            &oracle
        ));
        // Active lease on a disjoint scope -> safe (drain is over s1).
        assert!(at_safe_point(
            &domain("s1"),
            &[lease("s2", None, &plan)],
            now,
            &oracle
        ));
        Ok(())
    }

    #[test]
    fn a_global_safe_point_requires_every_lease_expired() -> TestResult {
        let plan = plan_hash()?;
        let oracle = KernelContracts;
        let now = Timestamp(100);
        // Any active lease anywhere blocks a global drain.
        assert!(!at_safe_point(
            &DrainTarget::Global,
            &[lease("s2", None, &plan)],
            now,
            &oracle
        ));
        // All expired -> global safe.
        assert!(at_safe_point(
            &DrainTarget::Global,
            &[lease("s2", Some(10), &plan)],
            now,
            &oracle
        ));
        // No leases -> global safe.
        assert!(at_safe_point(&DrainTarget::Global, &[], now, &oracle));
        Ok(())
    }

    #[test]
    fn disjoint_domains_drain_independently_global_never() {
        let oracle = KernelContracts;
        assert!(drains_independent(&domain("s1"), &domain("s2"), &oracle));
        assert!(!drains_independent(&domain("s1"), &domain("s1"), &oracle));
        assert!(!drains_independent(
            &domain("s1"),
            &DrainTarget::Global,
            &oracle
        ));
        assert!(!drains_independent(
            &DrainTarget::Global,
            &DrainTarget::Global,
            &oracle
        ));
    }

    #[test]
    fn a_frozen_sidecar_stays_admissible_a_conflicting_writer_does_not() {
        let oracle = KernelContracts;
        // Read-only sidecar: admissible during any drain (frozen).
        assert!(op_admissible_during_drain(
            &EffectSignature::projection_only(),
            &domain("s1"),
            &oracle
        ));
        assert!(op_admissible_during_drain(
            &EffectSignature::projection_only(),
            &DrainTarget::Global,
            &oracle
        ));
        // Mutable op writing into the drained domain: not admissible.
        assert!(!op_admissible_during_drain(
            &mutable_writing("s1"),
            &domain("s1"),
            &oracle
        ));
        // Mutable op writing elsewhere: admissible during a domain drain.
        assert!(op_admissible_during_drain(
            &mutable_writing("s2"),
            &domain("s1"),
            &oracle
        ));
        // Any mutable op during a global drain: not admissible.
        assert!(!op_admissible_during_drain(
            &mutable_writing("s2"),
            &DrainTarget::Global,
            &oracle
        ));
    }

    #[test]
    fn a_drain_governs_only_admissions_from_its_epoch_onward() {
        let req = DrainRequest {
            target: DrainTarget::Global,
            epoch: ConstraintEpoch(5),
        };
        assert!(!req.governs(ConstraintEpoch(4))); // predates the drain
        assert!(req.governs(ConstraintEpoch(5))); // same epoch
        assert!(req.governs(ConstraintEpoch(6))); // later epoch
    }

    /// Load-bearing property: across the lease active/expired/overlap space,
    /// `at_safe_point`'s domain case agrees with the I-007 authority
    /// (`can_acquire_fence`) and its global case holds iff no lease is active and
    /// non-expired. Non-vacuity: both safe and unsafe outcomes occur.
    #[test]
    fn safe_point_agrees_with_the_i007_authority() -> TestResult {
        let plan = plan_hash()?;
        let oracle = KernelContracts;
        let now = Timestamp(100);
        let scopes = ["s1", "s2"];
        let expiries = [None, Some(50_u64), Some(200_u64)];
        let targets = [DrainTarget::Domain(scope("s1")), DrainTarget::Global];

        let mut saw_safe = false;
        let mut saw_unsafe = false;

        for sc_a in scopes {
            for ex_a in expiries {
                for sc_b in scopes {
                    for ex_b in expiries {
                        let leases = vec![lease(sc_a, ex_a, &plan), lease(sc_b, ex_b, &plan)];
                        for target in &targets {
                            let got = at_safe_point(target, &leases, now, &oracle);
                            let expected = match target {
                                DrainTarget::Domain(s) => oracle.can_acquire_fence(s, &leases, now),
                                DrainTarget::Global => leases
                                    .iter()
                                    .all(|l| l.expires_at.is_some_and(|e| e.0 <= now.0)),
                            };
                            assert_eq!(got, expected, "safe-point disagrees with I-007 authority");
                            saw_safe |= got;
                            saw_unsafe |= !got;
                        }
                    }
                }
            }
        }

        assert!(saw_safe, "no safe point was ever reached");
        assert!(saw_unsafe, "no unsafe point was ever reached");
        Ok(())
    }
}
