//! Runtime constraint updates (M05.7).
//!
//! A runtime update changes the constraint plane while a system is live:
//! capacity, quota, freeze, or rate-limit. Each update is applied **at a new
//! epoch** and is **future-only** — it governs admissions from its epoch onward,
//! never rewrites already-committed observed truth, and forces a frontier rebuild
//! (old-epoch leases are stale at the new epoch).
//!
//! This builds **on top of** the I-010 truth-preservation authority
//! ([`crate::domain::constraint_update`] / `ConstraintUpdate::preserves_committed_truth`)
//! without duplicating or altering it: a runtime update rewrites no truth category
//! by construction ([`truth_rewrite_of`]), so it preserves committed truth for any
//! committed state. It reuses the M05.3 snapshot/epoch/`ConstraintKind` model, the
//! M05.2 `LaneCapacity`, and feeds the M05.4/M05.5 rebuild
//! (`GraphIndex::from_state` + `select_frontier`) via [`apply_capacity`].

use std::collections::BTreeMap;

use super::{
    ConstraintEpoch, ConstraintId, ConstraintKind, ConstraintSnapshot, ConstraintSpec,
    ConstraintUpdate, LaneCapacity, LaneId, LeaseRef, ResourceId, Scope,
};

/// A runtime change to the constraint plane (the four update kinds).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeUpdateKind {
    /// Change a lane's concurrency budget (applied to the lane registry, not the
    /// snapshot — see [`apply_capacity`]).
    Capacity {
        /// The lane whose capacity changes.
        lane: LaneId,
        /// The new capacity.
        capacity: LaneCapacity,
    },
    /// Set a token budget (quota) for a resource within a scope.
    Quota {
        /// The resource the budget applies to.
        resource: ResourceId,
        /// The scope the budget applies to.
        scope: Scope,
        /// The new maximum total token amount.
        limit: u64,
    },
    /// Freeze a scope (no new exclusive-write claims may be admitted on it).
    Freeze {
        /// The scope to freeze.
        scope: Scope,
    },
    /// Limit admissions of a resource/scope per epoch. Enforcement is a follow-up
    /// (no constraint-plane primitive yet); the update still bumps the epoch and is
    /// future-only.
    RateLimit {
        /// The resource the rate limit applies to.
        resource: ResourceId,
        /// The scope the rate limit applies to.
        scope: Scope,
        /// The maximum admissions allowed per epoch.
        max_per_epoch: u64,
    },
}

/// A runtime update applied at a constraint epoch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeUpdate {
    /// What the update changes.
    pub kind: RuntimeUpdateKind,
    /// The epoch the update takes effect at.
    pub applied_at: ConstraintEpoch,
}

impl RuntimeUpdate {
    /// A runtime update governs an admission only from its own epoch onward
    /// (future-only, consistent with I-010): an op admitted in an earlier epoch
    /// predates the update.
    #[must_use]
    pub fn governs(&self, admission_epoch: ConstraintEpoch) -> bool {
        admission_epoch >= self.applied_at
    }
}

/// The epoch a constraint update bumps the plane to (monotonic, saturating).
#[must_use]
pub fn next_epoch(current: ConstraintEpoch) -> ConstraintEpoch {
    ConstraintEpoch(current.0.saturating_add(1))
}

/// Whether `new` is a strictly later constraint epoch than `old`.
#[must_use]
pub fn epoch_advances(old: ConstraintEpoch, new: ConstraintEpoch) -> bool {
    new > old
}

/// Whether a lease is current at `epoch` — a lease is valid only within the epoch
/// it was granted in (ADR-0005/0013), so an old-epoch lease is stale after a bump.
#[must_use]
pub fn lease_current(lease: &LeaseRef, epoch: ConstraintEpoch) -> bool {
    lease.epoch == epoch
}

/// The observed-truth rewrite mask of a runtime constraint update — empty, because
/// an update changes the constraint plane, not committed truth. Feeding this to the
/// I-010 authority (`ConstraintUpdate::preserves_committed_truth`) is therefore true
/// for any committed state: a runtime update never rewrites truth.
#[must_use]
pub fn truth_rewrite_of(_kind: &RuntimeUpdateKind) -> ConstraintUpdate {
    ConstraintUpdate {
        rewrites_readiness: false,
        rewrites_promotion: false,
        rewrites_evidence: false,
    }
}

/// Apply a runtime update to a snapshot, producing a new snapshot at the next
/// epoch. Quota upserts the token budget; Freeze adds the freeze (idempotently);
/// Capacity (lane-side) and `RateLimit` leave the snapshot constraints unchanged.
/// Stale-epoch leases are dropped (none are current at the new epoch).
#[must_use]
pub fn apply_to_snapshot(
    snapshot: &ConstraintSnapshot,
    kind: &RuntimeUpdateKind,
) -> ConstraintSnapshot {
    let epoch = next_epoch(snapshot.epoch);
    let mut constraints = snapshot.constraints.clone();
    match kind {
        RuntimeUpdateKind::Quota {
            resource,
            scope,
            limit,
        } => upsert_token_budget(&mut constraints, resource, scope, *limit),
        RuntimeUpdateKind::Freeze { scope } => add_freeze_if_absent(&mut constraints, scope),
        RuntimeUpdateKind::Capacity { .. } | RuntimeUpdateKind::RateLimit { .. } => {}
    }
    let active_leases = snapshot
        .active_leases
        .iter()
        .filter(|lease| lease_current(lease, epoch))
        .cloned()
        .collect();
    ConstraintSnapshot {
        snapshot_id: ConstraintId(format!("snapshot@e{}", epoch.0)),
        epoch,
        constraints,
        active_leases,
    }
}

/// Apply a capacity update to a lane registry, producing the updated registry that
/// `select_frontier` (M05.5) is re-run against. Non-capacity kinds are no-ops.
#[must_use]
pub fn apply_capacity(
    lanes: &BTreeMap<LaneId, LaneCapacity>,
    kind: &RuntimeUpdateKind,
) -> BTreeMap<LaneId, LaneCapacity> {
    let mut updated = lanes.clone();
    if let RuntimeUpdateKind::Capacity { lane, capacity } = kind {
        let _previous = updated.insert(lane.clone(), *capacity);
    }
    updated
}

fn upsert_token_budget(
    constraints: &mut Vec<ConstraintSpec>,
    resource: &ResourceId,
    scope: &Scope,
    limit: u64,
) {
    for spec in constraints.iter_mut() {
        if let ConstraintKind::TokenBudget {
            resource: r,
            scope: s,
            limit: existing,
        } = &mut spec.kind
        {
            if *r == *resource && *s == *scope {
                *existing = limit;
                return;
            }
        }
    }
    constraints.push(ConstraintSpec {
        constraint_id: ConstraintId(format!("quota:{}:{}", resource.0, scope.0)),
        kind: ConstraintKind::TokenBudget {
            resource: resource.clone(),
            scope: scope.clone(),
            limit,
        },
    });
}

fn add_freeze_if_absent(constraints: &mut Vec<ConstraintSpec>, scope: &Scope) {
    let present = constraints
        .iter()
        .any(|spec| matches!(&spec.kind, ConstraintKind::Freeze { scope: s } if s == scope));
    if !present {
        constraints.push(ConstraintSpec {
            constraint_id: ConstraintId(format!("freeze:{}", scope.0)),
            kind: ConstraintKind::Freeze {
                scope: scope.clone(),
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        apply_capacity, apply_to_snapshot, epoch_advances, lease_current, next_epoch,
        truth_rewrite_of, RuntimeUpdate, RuntimeUpdateKind,
    };
    use crate::{
        select_frontier, ActionId, AuditEventId, ClaimMode, CommittedTruth, ConstraintEpoch,
        ConstraintId, ConstraintKind, ConstraintSnapshot, ConstraintSpec, FactKind, GraphIndex,
        GraphNode, LaneCapacity, LaneId, LeaseId, LeaseRef, OpId, PlanHash, PlanHashError,
        ResourceId, Scope,
    };

    type TestResult = Result<(), PlanHashError>;

    fn plan_hash() -> Result<PlanHash, PlanHashError> {
        PlanHash::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
    }

    fn lease_at(scope_name: &str, epoch: ConstraintEpoch, plan: &PlanHash) -> LeaseRef {
        LeaseRef {
            lease_id: LeaseId("l".to_owned()),
            resource: ResourceId("r".to_owned()),
            scope: Scope(scope_name.to_owned()),
            mode: ClaimMode::ExclusiveWrite,
            amount: 1,
            holder_action_id: ActionId("act".to_owned()),
            holder_plan_hash: plan.clone(),
            holder_op_index: Some(0),
            epoch,
            expires_at: None,
            lease_event_id: AuditEventId("evt".to_owned()),
        }
    }

    fn snapshot(
        epoch: ConstraintEpoch,
        constraints: Vec<ConstraintSpec>,
        active_leases: Vec<LeaseRef>,
    ) -> ConstraintSnapshot {
        ConstraintSnapshot {
            snapshot_id: ConstraintId("snap".to_owned()),
            epoch,
            constraints,
            active_leases,
        }
    }

    #[test]
    fn epoch_bumps_are_monotonic() {
        let e = ConstraintEpoch(3);
        assert_eq!(next_epoch(e), ConstraintEpoch(4));
        assert!(epoch_advances(e, next_epoch(e)));
        assert!(!epoch_advances(next_epoch(e), e));
        assert!(!epoch_advances(e, e));
    }

    #[test]
    fn a_lease_is_current_only_in_its_own_epoch() -> TestResult {
        let plan = plan_hash()?;
        let l = lease_at("s1", ConstraintEpoch(2), &plan);
        assert!(lease_current(&l, ConstraintEpoch(2)));
        assert!(!lease_current(&l, ConstraintEpoch(3)));
        Ok(())
    }

    #[test]
    fn an_update_governs_only_admissions_from_its_epoch_onward() {
        let u = RuntimeUpdate {
            kind: RuntimeUpdateKind::Freeze {
                scope: Scope("s1".to_owned()),
            },
            applied_at: ConstraintEpoch(5),
        };
        assert!(!u.governs(ConstraintEpoch(4)));
        assert!(u.governs(ConstraintEpoch(5)));
        assert!(u.governs(ConstraintEpoch(6)));
    }

    #[test]
    fn a_quota_update_upserts_the_limit_without_duplicating() {
        let resource = ResourceId("res".to_owned());
        let scope = Scope("s1".to_owned());
        let existing = ConstraintSpec {
            constraint_id: ConstraintId("q".to_owned()),
            kind: ConstraintKind::TokenBudget {
                resource: resource.clone(),
                scope: scope.clone(),
                limit: 1,
            },
        };
        let snap = snapshot(ConstraintEpoch(1), vec![existing], Vec::new());
        let out = apply_to_snapshot(
            &snap,
            &RuntimeUpdateKind::Quota {
                resource,
                scope,
                limit: 9,
            },
        );
        assert_eq!(out.constraints.len(), 1);
        let updated = out.constraints.into_iter().next().map(|spec| spec.kind);
        assert!(matches!(
            updated,
            Some(ConstraintKind::TokenBudget { limit: 9, .. })
        ));
    }

    #[test]
    fn a_freeze_update_is_idempotent() {
        let scope = Scope("s1".to_owned());
        let snap = snapshot(ConstraintEpoch(1), Vec::new(), Vec::new());
        let once = apply_to_snapshot(
            &snap,
            &RuntimeUpdateKind::Freeze {
                scope: scope.clone(),
            },
        );
        assert_eq!(once.constraints.len(), 1);
        // Re-applying to a snapshot that already carries the freeze adds no duplicate.
        let twice = apply_to_snapshot(&once, &RuntimeUpdateKind::Freeze { scope });
        assert_eq!(twice.constraints.len(), 1);
    }

    /// Load-bearing property: every update kind bumps the epoch monotonically,
    /// preserves committed truth for any committed state (reusing the I-010
    /// authority), and drops stale-epoch leases; constraint-resident kinds
    /// (Quota/Freeze) change the constraints while Capacity/`RateLimit` do not.
    #[test]
    fn apply_preserves_truth_bumps_epoch_and_drops_stale_leases() -> TestResult {
        let plan = plan_hash()?;
        let resource = ResourceId("res".to_owned());
        let scope = Scope("s1".to_owned());
        let kinds = [
            RuntimeUpdateKind::Capacity {
                lane: LaneId("l1".to_owned()),
                capacity: LaneCapacity::Bounded(2),
            },
            RuntimeUpdateKind::Quota {
                resource: resource.clone(),
                scope: scope.clone(),
                limit: 5,
            },
            RuntimeUpdateKind::Freeze {
                scope: scope.clone(),
            },
            RuntimeUpdateKind::RateLimit {
                resource,
                scope,
                max_per_epoch: 4,
            },
        ];

        let mut saw_changed = false;
        let mut saw_unchanged = false;

        for kind in &kinds {
            let snap = snapshot(
                ConstraintEpoch(3),
                Vec::new(),
                vec![lease_at("s1", ConstraintEpoch(3), &plan)],
            );
            let out = apply_to_snapshot(&snap, kind);

            assert!(epoch_advances(snap.epoch, out.epoch), "epoch must advance");
            assert_eq!(out.epoch, next_epoch(snap.epoch));
            assert!(
                out.active_leases.is_empty(),
                "stale-epoch leases must be dropped"
            );

            // Committed truth preserved for every committed state (I-010 reuse).
            for bits in 0_u8..8 {
                let committed = CommittedTruth {
                    readiness_committed: bits & 1 != 0,
                    promotion_committed: bits & 2 != 0,
                    evidence_committed: bits & 4 != 0,
                };
                assert!(truth_rewrite_of(kind).preserves_committed_truth(committed));
            }

            if out.constraints == snap.constraints {
                saw_unchanged = true;
            } else {
                saw_changed = true;
            }
        }

        assert!(saw_changed, "quota/freeze must change the constraints");
        assert!(
            saw_unchanged,
            "capacity/rate-limit must not change constraints"
        );
        Ok(())
    }

    /// A capacity update, applied via `apply_capacity`, makes `select_frontier`
    /// (M05.5) rebuild a tighter frontier — the runtime-update → frontier-rebuild path.
    #[test]
    fn a_capacity_update_rebuilds_a_tighter_frontier() {
        let mut index = GraphIndex::new();
        index.add_node(GraphNode {
            op_id: OpId(ActionId("a".to_owned()), 0),
            lane: LaneId("l1".to_owned()),
            requires: Vec::<FactKind>::new(),
            writes: vec![Scope("s1".to_owned())],
        });
        index.add_node(GraphNode {
            op_id: OpId(ActionId("b".to_owned()), 0),
            lane: LaneId("l1".to_owned()),
            requires: Vec::<FactKind>::new(),
            writes: vec![Scope("s2".to_owned())],
        });

        let lanes: BTreeMap<LaneId, LaneCapacity> =
            [(LaneId("l1".to_owned()), LaneCapacity::Unbounded)]
                .into_iter()
                .collect();
        let before = select_frontier(&index, &lanes);
        assert_eq!(before.selected.len(), 2);

        let tightened = apply_capacity(
            &lanes,
            &RuntimeUpdateKind::Capacity {
                lane: LaneId("l1".to_owned()),
                capacity: LaneCapacity::Bounded(1),
            },
        );
        let after = select_frontier(&index, &tightened);
        assert_eq!(after.selected.len(), 1);
    }
}
