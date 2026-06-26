//! Constraint plane domain types.

use std::collections::HashSet;

use super::{ActionId, AuditEventId, ImpactSetHash, PlanHash, Scope, WitnessRef};
use crate::contract::ConflictOracle;

/// Monotonic version of the constraint plane. A lease is only valid within the
/// epoch it was granted in (see ADR-0005 / ADR-0013).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConstraintEpoch(pub u64);

/// Opaque wall-clock instant (milliseconds since an unspecified epoch). Kept
/// out of the plan hash material; used only for lease expiry bookkeeping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(pub u64);

/// Identifies a constraint in the constraint plane.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConstraintId(pub String);

/// Identifies a resource that claims and leases are taken against.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceId(pub String);

/// Identifies a granted lease.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LeaseId(pub String);

/// Identifies an execution capability derived from a durable barrier.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CapabilityId(pub String);

/// A keyed attestation (HMAC) over a capability's canonical bytes, minted by the
/// kernel that derived the capability. Hex-encoded so it round-trips through the
/// trace JSON. The kernel holds the secret; a replay verifier configured with
/// that secret can confirm the capability was minted by the kernel, so an
/// attacker who controls the trace but not the secret cannot forge one (ADR-0013).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityAttestation(pub String);

/// How a resource is claimed: shared, exclusive, or token-counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaimMode {
    /// Shared read access; compatible with other shared readers.
    SharedRead,
    /// Exclusive write access; conflicts with all other claims.
    ExclusiveWrite,
    /// Counted token claim against a finite pool.
    Token,
}

/// A request for some amount of a resource under a given scope and mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceClaim {
    /// The resource being claimed.
    pub resource: ResourceId,
    /// The scope the claim is restricted to.
    pub scope: Scope,
    /// How the resource is claimed.
    pub mode: ClaimMode,
    /// The amount claimed (for token modes).
    pub amount: u64,
}

/// A granted hold on a resource by a specific action and plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lease {
    /// The lease's id.
    pub lease_id: LeaseId,
    /// The resource held.
    pub resource: ResourceId,
    /// The scope the lease applies to.
    pub scope: Scope,
    /// How the resource is held.
    pub mode: ClaimMode,
    /// The action holding the lease.
    pub holder_action_id: ActionId,
    /// The plan hash the holder is executing.
    pub holder_plan_hash: PlanHash,
}

/// A reference to a granted lease as recorded on an audit event / execution
/// barrier (ADR-0013). Unlike [`Lease`], this carries the epoch, expiry and the
/// id of the audit event that granted it, so replay can verify lease coverage
/// and conflict-freedom (invariant I-006).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseRef {
    /// The granted lease's id.
    pub lease_id: LeaseId,
    /// The resource held.
    pub resource: ResourceId,
    /// The scope the lease applies to.
    pub scope: Scope,
    /// How the resource is held.
    pub mode: ClaimMode,
    /// The amount held (for token modes).
    pub amount: u64,
    /// The action holding the lease.
    pub holder_action_id: ActionId,
    /// The plan hash the holder is executing.
    pub holder_plan_hash: PlanHash,
    /// The op within the plan that requires the lease, if any.
    pub holder_op_index: Option<u32>,
    /// The constraint epoch the lease was granted in.
    pub epoch: ConstraintEpoch,
    /// When the lease expires, if it is time-bounded.
    pub expires_at: Option<Timestamp>,
    /// The audit event that recorded the grant.
    pub lease_event_id: AuditEventId,
}

/// Durable write-ahead barrier authorizing execution of one or more ops
/// (ADR-0013).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionBarrier {
    /// Barrier event id.
    pub barrier_id: AuditEventId,
    /// Action this barrier authorizes.
    pub action_id: ActionId,
    /// Plan hash this barrier authorizes.
    pub plan_hash: PlanHash,
    /// Op indexes covered by this barrier.
    pub op_indexes: Vec<u32>,
    /// Planned impact set hash bound to this barrier.
    pub impact_set_hash: ImpactSetHash,
    /// Typed witnesses required by the bundle.
    pub witnesses: Vec<WitnessRef>,
    /// Leases held at barrier time.
    pub leases: Vec<LeaseRef>,
    /// Authorization decision events considered by the barrier.
    pub authz_decision_refs: Vec<AuditEventId>,
    /// Constraint snapshot id, if the constraint plane recorded one.
    pub constraint_snapshot_id: Option<ConstraintId>,
}

/// Scoped capability passed to executors instead of raw lease arrays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionCapability {
    /// Capability id.
    pub capability_id: CapabilityId,
    /// Action this capability authorizes.
    pub action_id: ActionId,
    /// Plan hash this capability authorizes.
    pub plan_hash: PlanHash,
    /// Op index authorized by this capability.
    pub op_index: u32,
    /// Barrier event id this capability derives from.
    pub barrier_event_id: AuditEventId,
    /// Lease ids covering this op.
    pub lease_ids: Vec<LeaseId>,
    /// Earliest lease expiry, if any.
    pub expires_at: Option<Timestamp>,
    /// Optional keyed attestation over [`ExecutionCapability::attestation_message`].
    /// Present when the minting kernel signed the capability; verified by replay
    /// when configured with the kernel secret.
    pub attestation: Option<CapabilityAttestation>,
}

/// Lease-table validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeaseTableError {
    /// Lease id already exists.
    DuplicateLease {
        /// Duplicate lease id.
        lease_id: LeaseId,
    },
    /// Two active leases conflict.
    Conflict {
        /// Existing lease id.
        existing: LeaseId,
        /// New lease id.
        incoming: LeaseId,
        /// Resource involved.
        resource: ResourceId,
        /// Scope involved.
        scope: Scope,
    },
    /// Release references an unknown lease id.
    UnknownRelease {
        /// Unknown lease id.
        lease_id: LeaseId,
    },
    /// Lease holder does not match expected action/plan.
    HolderMismatch {
        /// Lease id.
        lease_id: LeaseId,
    },
    /// A barrier references a lease under a different constraint epoch than the
    /// one it was granted in (the lease was silently re-pointed across an epoch
    /// boundary).
    EpochMismatch {
        /// Lease id.
        lease_id: LeaseId,
    },
    /// Lease is expired at the validation time.
    Expired {
        /// Lease id.
        lease_id: LeaseId,
    },
    /// A required claim is not covered by any active lease.
    ClaimNotCovered {
        /// Resource that lacks coverage.
        resource: ResourceId,
        /// Scope that lacks coverage.
        scope: Scope,
    },
}

/// Active lease table keyed by lease id.
///
/// `mergeable_scopes` are conflict-domain scopes a **verified** merge protocol
/// permits overlapping mutable writes on (I-006). Two otherwise-conflicting
/// exclusive leases on a mergeable scope do not conflict — this is the
/// per-protocol merge decision, resolved by the bundle-aware caller and
/// fail-closed (empty) by default.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LeaseTable {
    active: Vec<LeaseRef>,
    mergeable_scopes: HashSet<Scope>,
}

impl LeaseTable {
    /// Create an empty lease table with no mergeable scopes (fail-closed).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty lease table whose conflicts are relaxed on the given
    /// verified-merge conflict-domain scopes.
    #[must_use]
    pub fn with_mergeable_scopes(mergeable_scopes: HashSet<Scope>) -> Self {
        Self {
            active: Vec::new(),
            mergeable_scopes,
        }
    }

    /// Grant a lease, rejecting duplicate ids and active conflicts.
    ///
    /// The conflict decision (I-006) is delegated to the supplied
    /// [`ConflictOracle`] — the single kernel authority (`KernelContracts`) the
    /// replay oracle and runtime hand in — so this primitive cannot decide a
    /// conflict outside that authority, and the scope-overlap test stays the
    /// authority's [`ScopeOverlap::overlaps`](crate::ScopeOverlap::overlaps)
    /// rather than a duplicated equality here.
    ///
    /// # Errors
    /// Returns [`LeaseTableError`] if the lease cannot become active.
    #[must_use = "lease grant validation must be checked"]
    pub fn grant(
        &mut self,
        lease: LeaseRef,
        oracle: &impl ConflictOracle,
    ) -> Result<(), LeaseTableError> {
        if self
            .active
            .iter()
            .any(|active| active.lease_id == lease.lease_id)
        {
            return Err(LeaseTableError::DuplicateLease {
                lease_id: lease.lease_id,
            });
        }
        let verified_merge = self.mergeable_scopes.contains(&lease.scope);
        for active in &self.active {
            if oracle.leases_conflict(active, &lease, verified_merge) {
                return Err(LeaseTableError::Conflict {
                    existing: active.lease_id.clone(),
                    incoming: lease.lease_id,
                    resource: active.resource.clone(),
                    scope: active.scope.clone(),
                });
            }
        }
        self.active.push(lease);
        Ok(())
    }

    /// Release one active lease by id.
    ///
    /// # Errors
    /// Returns [`LeaseTableError::UnknownRelease`] if no active lease has this id.
    #[must_use = "lease release validation must be checked"]
    pub fn release(&mut self, lease_id: &LeaseId) -> Result<LeaseRef, LeaseTableError> {
        let Some(index) = self
            .active
            .iter()
            .position(|active| active.lease_id == *lease_id)
        else {
            return Err(LeaseTableError::UnknownRelease {
                lease_id: lease_id.clone(),
            });
        };
        Ok(self.active.remove(index))
    }

    /// Borrow an active lease by id.
    #[must_use]
    pub fn get(&self, lease_id: &LeaseId) -> Option<&LeaseRef> {
        self.active
            .iter()
            .find(|active| active.lease_id == *lease_id)
    }

    /// The currently-active (granted, not released) leases, in grant order.
    ///
    /// The I-007 drain-fence decision is the kernel authority's
    /// [`crate::contract::DrainSemantics::can_acquire_fence`], which is expiry-aware
    /// (an expired lease no longer blocks); callers pass this slice plus the fence
    /// acquisition time `now`, rather than testing mere lease existence.
    #[must_use]
    pub fn active_leases(&self) -> &[LeaseRef] {
        &self.active
    }

    /// Verify that barrier leases are active, non-expired, and held by the
    /// expected action/plan.
    ///
    /// # Errors
    /// Returns a [`LeaseTableError`] when any lease is invalid.
    #[must_use = "barrier lease validation must be checked"]
    pub fn validate_barrier_leases(
        &self,
        barrier: &ExecutionBarrier,
        now: Option<Timestamp>,
    ) -> Result<(), LeaseTableError> {
        for lease in &barrier.leases {
            let active = self
                .active
                .iter()
                .find(|active| active.lease_id == lease.lease_id)
                .ok_or_else(|| LeaseTableError::UnknownRelease {
                    lease_id: lease.lease_id.clone(),
                })?;
            if active.holder_action_id != barrier.action_id
                || active.holder_plan_hash != barrier.plan_hash
            {
                return Err(LeaseTableError::HolderMismatch {
                    lease_id: lease.lease_id.clone(),
                });
            }
            // The barrier must reference the lease under the epoch it was granted
            // in; a mismatch means the lease was re-pointed across a constraint
            // -plane epoch boundary (ADR-0005/0013).
            if active.epoch != lease.epoch {
                return Err(LeaseTableError::EpochMismatch {
                    lease_id: lease.lease_id.clone(),
                });
            }
            match (active.expires_at, now) {
                (Some(expiry), Some(at)) => {
                    if expiry <= at {
                        return Err(LeaseTableError::Expired {
                            lease_id: lease.lease_id.clone(),
                        });
                    }
                }
                // Fail closed: a lease that carries an expiry cannot be proven
                // fresh without an evaluation time, so reject rather than skip
                // the check (an expiring lease + a timeless barrier is exactly
                // the case that must not silently pass).
                (Some(_), None) => {
                    return Err(LeaseTableError::Expired {
                        lease_id: lease.lease_id.clone(),
                    });
                }
                (None, _) => {}
            }
        }
        Ok(())
    }

    /// Verify that every required claim is covered by an active lease.
    ///
    /// # Errors
    /// Returns [`LeaseTableError::ClaimNotCovered`] for the first uncovered claim.
    #[must_use = "claim coverage validation must be checked"]
    pub fn validate_claim_coverage(
        &self,
        action_id: &ActionId,
        plan_hash: &PlanHash,
        claims: &[ResourceClaim],
    ) -> Result<(), LeaseTableError> {
        for claim in claims {
            let covered = self.active.iter().any(|lease| {
                lease.holder_action_id == *action_id
                    && lease.holder_plan_hash == *plan_hash
                    && lease_covers_claim(lease, claim)
            });
            if !covered {
                return Err(LeaseTableError::ClaimNotCovered {
                    resource: claim.resource.clone(),
                    scope: claim.scope.clone(),
                });
            }
        }
        Ok(())
    }
}

/// Default mergeability predicate. No effects are mergeable until a verified
/// protocol is represented and selected.
#[must_use]
pub fn mergeable() -> bool {
    false
}

/// Pure conflict predicate for two claims after the caller has already resolved
/// whether resource/scope keys overlap and whether a verified merge protocol is
/// in effect.
#[must_use]
pub fn claim_modes_conflict(
    left: ClaimMode,
    right: ClaimMode,
    same_resource: bool,
    same_scope: bool,
    verified_merge: bool,
) -> bool {
    same_resource
        && same_scope
        && (left == ClaimMode::ExclusiveWrite || right == ClaimMode::ExclusiveWrite)
        && !verified_merge
}

/// Whether a lease covers a resource claim: same resource, scope and mode, with
/// at least the claimed amount. This is the single claim-coverage predicate both
/// [`LeaseTable::validate_claim_coverage`] (against active leases, with holder
/// binding) and the replay oracle (against a barrier's declared leases) consult,
/// so the two cannot diverge on what "covered" means (P1-003).
#[must_use]
pub fn lease_covers_claim(lease: &LeaseRef, claim: &ResourceClaim) -> bool {
    lease.resource == claim.resource
        && lease.scope == claim.scope
        && lease.mode == claim.mode
        && lease.amount >= claim.amount
}

#[cfg(test)]
mod tests {
    use crate::contract::KernelContracts;
    use crate::{capability_binding_matches, PlanHashError};

    use super::*;

    fn plan_hash() -> Result<PlanHash, PlanHashError> {
        PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    }

    fn lease(id: &str, plan: &PlanHash, scope: &str, mode: ClaimMode) -> LeaseRef {
        LeaseRef {
            lease_id: LeaseId(id.to_owned()),
            resource: ResourceId("release_candidate_write".to_owned()),
            scope: Scope(scope.to_owned()),
            mode,
            amount: 1,
            holder_action_id: ActionId("act".to_owned()),
            holder_plan_hash: plan.clone(),
            holder_op_index: Some(0),
            epoch: ConstraintEpoch(0),
            expires_at: None,
            lease_event_id: AuditEventId(format!("evt_{id}")),
        }
    }

    fn barrier(plan: PlanHash, leases: Vec<LeaseRef>) -> ExecutionBarrier {
        ExecutionBarrier {
            barrier_id: AuditEventId("barrier".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: plan,
            op_indexes: vec![0],
            impact_set_hash: ImpactSetHash(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_owned(),
            ),
            witnesses: vec![],
            leases,
            authz_decision_refs: vec![],
            constraint_snapshot_id: None,
        }
    }

    // I-006: a verified merge protocol (modelled as a mergeable conflict-domain
    // scope) relaxes the otherwise-fail-closed exclusive-lease conflict.
    #[test]
    fn lease_table_merge_relaxes_conflict_on_mergeable_scope() -> Result<(), PlanHashError> {
        let plan = plan_hash()?;
        let first = lease("a", &plan, "environment:staging", ClaimMode::ExclusiveWrite);
        let second = lease("b", &plan, "environment:staging", ClaimMode::ExclusiveWrite);

        // Fail-closed default: two exclusive leases on the same scope conflict.
        let mut fail_closed = LeaseTable::new();
        assert!(fail_closed.grant(first.clone(), &KernelContracts).is_ok());
        assert!(matches!(
            fail_closed.grant(second.clone(), &KernelContracts),
            Err(LeaseTableError::Conflict { .. })
        ));

        // A verified merge protocol on that scope permits the overlap.
        let permitted: HashSet<Scope> =
            std::iter::once(Scope("environment:staging".to_owned())).collect();
        let mut relaxed = LeaseTable::with_mergeable_scopes(permitted);
        assert!(relaxed.grant(first, &KernelContracts).is_ok());
        assert!(relaxed.grant(second, &KernelContracts).is_ok());
        Ok(())
    }

    #[test]
    fn claim_mode_conflict_rule_is_fail_closed_by_default() {
        assert!(claim_modes_conflict(
            ClaimMode::ExclusiveWrite,
            ClaimMode::SharedRead,
            true,
            true,
            false,
        ));
        assert!(!claim_modes_conflict(
            ClaimMode::SharedRead,
            ClaimMode::SharedRead,
            true,
            true,
            false,
        ));
        assert!(!claim_modes_conflict(
            ClaimMode::ExclusiveWrite,
            ClaimMode::ExclusiveWrite,
            true,
            true,
            true,
        ));
    }

    #[test]
    fn capability_binding_rule_is_fail_closed() {
        let matches_barrier = true;
        let matches_action = true;
        let matches_plan = true;
        let covered_op = true;
        let has_leases = true;
        assert!(capability_binding_matches(
            matches_barrier,
            matches_action,
            matches_plan,
            covered_op,
            has_leases,
        ));
        let wrong_barrier = false;
        assert!(!capability_binding_matches(
            wrong_barrier,
            matches_action,
            matches_plan,
            covered_op,
            has_leases,
        ));
        let wrong_action = false;
        assert!(!capability_binding_matches(
            matches_barrier,
            wrong_action,
            matches_plan,
            covered_op,
            has_leases,
        ));
        let wrong_plan = false;
        assert!(!capability_binding_matches(
            matches_barrier,
            matches_action,
            wrong_plan,
            covered_op,
            has_leases,
        ));
        let uncovered_op = false;
        assert!(!capability_binding_matches(
            matches_barrier,
            matches_action,
            matches_plan,
            uncovered_op,
            has_leases,
        ));
        let no_leases = false;
        assert!(!capability_binding_matches(
            matches_barrier,
            matches_action,
            matches_plan,
            covered_op,
            no_leases,
        ));
    }

    #[test]
    fn execution_capability_derives_and_validates_exact_binding() -> Result<(), PlanHashError> {
        let plan = plan_hash()?;
        let barrier = barrier(
            plan.clone(),
            vec![lease(
                "lease_a",
                &plan,
                "candidate:123",
                ClaimMode::ExclusiveWrite,
            )],
        );
        let capability = match ExecutionCapability::derive_from_barrier(&barrier, 0) {
            Ok(capability) => capability,
            Err(_err) => return Err(PlanHashError::Placeholder),
        };
        assert_eq!(capability.action_id, barrier.action_id);
        assert_eq!(capability.plan_hash, barrier.plan_hash);
        assert_eq!(capability.op_index, 0);
        assert!(capability.validate_for_barrier(&barrier).is_ok());
        assert!(ExecutionCapability::derive_from_barrier(&barrier, 1).is_err());

        let mut wrong_barrier = barrier.clone();
        wrong_barrier.barrier_id = AuditEventId("other_barrier".to_owned());
        assert!(capability.validate_for_barrier(&wrong_barrier).is_err());
        Ok(())
    }

    #[test]
    fn lease_table_rejects_conflicts_and_validates_claims() -> Result<(), PlanHashError> {
        let plan = plan_hash()?;
        let mut table = LeaseTable::new();
        assert!(table
            .grant(
                lease("lease_a", &plan, "candidate:123", ClaimMode::ExclusiveWrite),
                &KernelContracts
            )
            .is_ok());
        assert!(table
            .grant(
                lease("lease_b", &plan, "candidate:123", ClaimMode::ExclusiveWrite),
                &KernelContracts
            )
            .is_err());
        assert!(table
            .validate_claim_coverage(
                &ActionId("act".to_owned()),
                &plan,
                &[ResourceClaim {
                    resource: ResourceId("release_candidate_write".to_owned()),
                    scope: Scope("candidate:123".to_owned()),
                    mode: ClaimMode::ExclusiveWrite,
                    amount: 1,
                }],
            )
            .is_ok());
        assert!(table.release(&LeaseId("lease_a".to_owned())).is_ok());
        assert!(table.release(&LeaseId("lease_a".to_owned())).is_err());
        Ok(())
    }

    #[test]
    fn lease_table_validates_barrier_lease_holder_and_expiry() -> Result<(), PlanHashError> {
        let plan = plan_hash()?;
        let mut active = lease("lease_a", &plan, "candidate:123", ClaimMode::ExclusiveWrite);
        active.expires_at = Some(Timestamp(10));
        let mut table = LeaseTable::new();
        assert!(table.grant(active.clone(), &KernelContracts).is_ok());
        let barrier = barrier(plan, vec![active]);
        assert!(table
            .validate_barrier_leases(&barrier, Some(Timestamp(1)))
            .is_ok());
        assert!(table
            .validate_barrier_leases(&barrier, Some(Timestamp(10)))
            .is_err());
        // Fail closed: an expiring lease cannot be proven fresh without an
        // evaluation time, so a `None` clock must reject (not silently pass).
        assert!(table.validate_barrier_leases(&barrier, None).is_err());
        Ok(())
    }

    // A lease without an expiry is unaffected by a missing evaluation time.
    #[test]
    fn lease_without_expiry_passes_without_clock() -> Result<(), PlanHashError> {
        let plan = plan_hash()?;
        let active = lease("lease_b", &plan, "candidate:123", ClaimMode::ExclusiveWrite);
        let mut table = LeaseTable::new();
        assert!(table.grant(active.clone(), &KernelContracts).is_ok());
        let barrier = barrier(plan, vec![active]);
        assert!(table.validate_barrier_leases(&barrier, None).is_ok());
        Ok(())
    }

    // A barrier that references a granted lease under a different epoch is
    // rejected (the lease was re-pointed across an epoch boundary).
    #[test]
    fn lease_epoch_mismatch_is_rejected() -> Result<(), PlanHashError> {
        let plan = plan_hash()?;
        let active = lease("lease_c", &plan, "candidate:123", ClaimMode::ExclusiveWrite);
        let mut table = LeaseTable::new();
        assert!(table.grant(active.clone(), &KernelContracts).is_ok());
        let mut barrier_lease = active.clone();
        barrier_lease.epoch = ConstraintEpoch(active.epoch.0 + 1);
        let barrier = barrier(plan, vec![barrier_lease]);
        assert!(matches!(
            table.validate_barrier_leases(&barrier, None),
            Err(LeaseTableError::EpochMismatch { .. })
        ));
        Ok(())
    }
}

/// The outcome of evaluating a plan's resource claims against the constraint plane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstraintDecision {
    /// The claims are allowed; these leases must be acquired.
    Allow {
        /// The leases the plan must hold to proceed.
        required_leases: Vec<ResourceClaim>,
    },
    /// The plan must wait until the listed blockers clear.
    Wait {
        /// The constraints currently blocking the plan.
        blockers: Vec<ConstraintBlocker>,
    },
    /// The plan is denied due to the listed violations.
    Deny {
        /// The constraints the plan violates.
        violations: Vec<ConstraintViolation>,
    },
    /// The plan is allowed subject to the listed restrictions.
    AllowWithRestrictions {
        /// Human-readable restrictions imposed on the plan.
        restrictions: Vec<String>,
    },
}

/// A constraint that is temporarily preventing a plan from proceeding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintBlocker {
    /// The blocking constraint.
    pub constraint_id: ConstraintId,
    /// Why the constraint is currently blocking.
    pub reason: String,
}

/// A constraint a plan violates outright.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintViolation {
    /// The violated constraint.
    pub constraint_id: ConstraintId,
    /// Why the constraint is violated.
    pub reason: String,
}
