//! why-not-parallel — machine-readable blocker/rationale (M05.8).
//!
//! Closes the S05 exit gate: a dispatcher can explain `ready`, `blocked`, and
//! *why-not-parallel*. This module is an **aggregator** — it unifies the already
//! typed causes minted by M05.2–M05.7 into one machine-readable vocabulary and
//! **re-derives nothing**. Every [`NotParallelReason`] variant wraps an existing
//! typed cause by value:
//!   - [`FrontierBlock`] (M05.5 — lane at capacity / pending write-scope conflict);
//!   - [`ConstraintBlocker`] / [`ConstraintViolation`] (M05.3/M05.7 — `Wait` / `Deny`);
//!   - [`LaneRejection`] (M05.2 — wrong tier / capability mismatch / capacity);
//!   - [`DrainTarget`] (M05.6 — a write scope inside a draining region);
//!   - blocked-on-unproduced-fact and blocked-on-active-writer-scope (M05.4 index).
//!
//! "Machine-readable" for the Rust-first kernel is the typed enum itself
//! (exhaustive, pattern-matchable). `causlane-core` has no `serde`; a JSON surface
//! is a downstream concern (the replay/CLI crates, like `ReplayExplain`). The
//! aggregators consume *outputs* of `select_frontier` / `resolve_constraints` /
//! `op_admissible_during_drain` / [`crate::domain::graph_index::GraphIndex`]
//! queries, so they cannot disagree with those deciders and add no scheduling
//! logic.

use std::collections::BTreeSet;

use super::{
    ConstraintBlocker, ConstraintDecision, ConstraintViolation, DrainTarget, FactKind,
    FrontierBlock, FrontierRejection, GraphIndex, LaneRejection, OpId, Scope,
};

/// One machine-readable reason an op cannot run *now*, wrapping the existing typed
/// cause that produced it. M05.8 aggregates these — it never re-derives them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotParallelReason {
    /// Frontier selection kept the op out (lane at capacity, or its write scope is
    /// already claimed by a selected op). Wraps the M05.5 [`FrontierBlock`].
    Frontier(FrontierBlock),
    /// The constraint plane says *wait* (an active exclusive lease, or token-budget
    /// exhaustion). Wraps a `ConstraintDecision::Wait` blocker.
    ConstraintWait(ConstraintBlocker),
    /// The constraint plane says *deny* (a `Freeze`, or an oversized token claim).
    /// Wraps a `ConstraintDecision::Deny` violation.
    ConstraintDeny(ConstraintViolation),
    /// The lane authority rejected the op (wrong tier, capability mismatch, or
    /// capacity exhausted). Wraps the M05.2 [`LaneRejection`].
    LaneRejected(LaneRejection),
    /// A mutable op writes into a region being drained (M05.6), inadmissible until
    /// the drain completes.
    DrainRegion {
        /// The drain that quiesces the op's write region.
        target: DrainTarget,
        /// One of the op's write scopes that falls inside the drained region.
        scope: Scope,
    },
    /// The op is structurally not-yet-ready: a required fact has not been produced
    /// (M05.4 `waiting_on_fact`).
    BlockedOnFact {
        /// The unproduced fact the op requires.
        fact: FactKind,
    },
    /// The op writes a scope held by an *active* writer (M05.4 `waiting_on_scope` /
    /// `active_writers`) — distinct from `Frontier(WriteScopeConflict)`, which is a
    /// pending-vs-pending conflict between two ready ops.
    BlockedOnActiveScope {
        /// The contended write scope.
        scope: Scope,
        /// An active op currently writing the scope.
        held_by: OpId,
    },
}

/// A write scope of an op that falls inside an active drain region (a named carrier
/// for the evidence the caller already holds — not a new cause).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrainedWriteScope {
    /// The drain quiescing the region.
    pub target: DrainTarget,
    /// The op's write scope inside it.
    pub scope: Scope,
}

/// A write scope of an op currently held by an active writer (a named carrier — not
/// a new cause).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveScopeHolder {
    /// The contended write scope.
    pub scope: Scope,
    /// The active op writing it.
    pub held_by: OpId,
}

/// The already-computed blocking causes for one op, as the dispatcher holds them.
/// Each field is an *output* of an existing decider; [`why_not_parallel`] unions
/// them without recomputing anything.
#[derive(Clone, Debug)]
pub struct WhyNotParallelInputs<'a> {
    /// This op's `FrontierBlock` from `select_frontier`, if it was rejected.
    pub frontier: Option<&'a FrontierBlock>,
    /// The plan's `ConstraintDecision` from `resolve_constraints` (only `Wait` /
    /// `Deny` contribute; `Allow` / `AllowWithRestrictions` do not bar parallelism).
    pub constraint: Option<&'a ConstraintDecision>,
    /// A lane rejection from the lane authority gate, if any.
    pub lane: Option<&'a LaneRejection>,
    /// The op's write scopes inside an active drain region.
    pub drained_writes: &'a [DrainedWriteScope],
    /// Facts the op still waits on (from the graph index).
    pub unmet_facts: &'a [FactKind],
    /// Active writers holding scopes the op writes.
    pub active_scope_holders: &'a [ActiveScopeHolder],
}

/// The complete why-not-parallel answer for one op: every reason it is held back.
/// Empty `reasons` ⟺ the op *is* parallelizable now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhyNotParallel {
    /// The op being explained.
    pub op_id: OpId,
    /// Every reason the op is held back (zero ⟺ it can run now).
    pub reasons: Vec<NotParallelReason>,
}

impl WhyNotParallel {
    /// Whether the op can run now (no blocking reason was aggregated).
    #[must_use]
    pub fn is_parallelizable(&self) -> bool {
        self.reasons.is_empty()
    }
}

/// Why two specific ops cannot be selected together — the pairwise "why-not-parallel".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PairConflict {
    /// Both ops write the same scope; only one may be selected (M05.5 exact-scope
    /// conflict — overlapping mutable writes conflict by default, TD-010).
    WriteScopeConflict {
        /// The contended write scope.
        scope: Scope,
    },
}

/// Lift a single frontier rejection into the why-not-parallel vocabulary. Total and
/// information-preserving: the [`FrontierBlock`] is wrapped, never re-derived.
#[must_use]
pub fn reason_from_frontier(rejection: &FrontierRejection) -> NotParallelReason {
    NotParallelReason::Frontier(rejection.reason.clone())
}

/// Aggregate every already-computed blocking cause for `op_id` into one
/// machine-readable answer. Reasons are pushed in a fixed order (facts, active
/// scopes, constraint, drain, lane, frontier) so the result is deterministic for
/// replay parity. `Allow` / `AllowWithRestrictions` contribute nothing — a
/// restriction is not a bar to parallelism.
#[must_use]
pub fn why_not_parallel(op_id: OpId, inputs: &WhyNotParallelInputs<'_>) -> WhyNotParallel {
    let mut reasons = Vec::new();
    for fact in inputs.unmet_facts {
        reasons.push(NotParallelReason::BlockedOnFact { fact: fact.clone() });
    }
    for holder in inputs.active_scope_holders {
        reasons.push(NotParallelReason::BlockedOnActiveScope {
            scope: holder.scope.clone(),
            held_by: holder.held_by.clone(),
        });
    }
    if let Some(decision) = inputs.constraint {
        match decision {
            ConstraintDecision::Wait { blockers } => {
                for blocker in blockers {
                    reasons.push(NotParallelReason::ConstraintWait(blocker.clone()));
                }
            }
            ConstraintDecision::Deny { violations } => {
                for violation in violations {
                    reasons.push(NotParallelReason::ConstraintDeny(violation.clone()));
                }
            }
            // AllowWithRestrictions does not bar parallelism; its restrictions are
            // surfaced by the constraint plane, not here.
            ConstraintDecision::Allow { .. } | ConstraintDecision::AllowWithRestrictions { .. } => {
            }
        }
    }
    for drained in inputs.drained_writes {
        reasons.push(NotParallelReason::DrainRegion {
            target: drained.target.clone(),
            scope: drained.scope.clone(),
        });
    }
    if let Some(lane) = inputs.lane {
        reasons.push(NotParallelReason::LaneRejected(*lane));
    }
    if let Some(block) = inputs.frontier {
        reasons.push(NotParallelReason::Frontier(block.clone()));
    }
    WhyNotParallel { op_id, reasons }
}

/// Build a why-not-parallel answer from the read-only graph index evidence plus
/// an optional frontier rejection. This is a thin adapter over
/// [`why_not_parallel`]; it does not recompute readiness or scheduling.
#[must_use]
pub fn why_not_parallel_from_index(
    index: &GraphIndex,
    op_id: &OpId,
    frontier: Option<&FrontierBlock>,
) -> Option<WhyNotParallel> {
    index.node(op_id)?;
    let unmet_facts = index.unmet_facts_for(op_id);
    let active_scope_holders: Vec<ActiveScopeHolder> = index
        .active_scope_holders_for(op_id)
        .into_iter()
        .map(|(scope, held_by)| ActiveScopeHolder { scope, held_by })
        .collect();
    let inputs = WhyNotParallelInputs {
        frontier,
        constraint: None,
        lane: None,
        drained_writes: &[],
        unmet_facts: &unmet_facts,
        active_scope_holders: &active_scope_holders,
    };
    Some(why_not_parallel(op_id.clone(), &inputs))
}

/// Explain why two ops cannot be selected together. The only head-to-head bar at
/// S05 is a shared write scope (TD-010); lane budget is a per-lane property, not a
/// pairwise one, so it is surfaced per-op via `Frontier(LaneAtCapacity)`, not here.
/// Returns the first shared scope in deterministic order, or `None`.
#[must_use]
pub fn pair_conflict(
    left_writes: &BTreeSet<Scope>,
    right_writes: &BTreeSet<Scope>,
) -> Option<PairConflict> {
    left_writes
        .intersection(right_writes)
        .next()
        .map(|scope| PairConflict::WriteScopeConflict {
            scope: scope.clone(),
        })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        pair_conflict, reason_from_frontier, why_not_parallel, why_not_parallel_from_index,
        ActiveScopeHolder, DrainedWriteScope, NotParallelReason, PairConflict, WhyNotParallel,
        WhyNotParallelInputs,
    };
    use crate::{
        ActionId, ConstraintBlocker, ConstraintDecision, ConstraintId, ConstraintViolation,
        DrainTarget, FactKind, FrontierBlock, FrontierRejection, GraphIndex, GraphNode, LaneId,
        LaneRejection, OpId, Scope,
    };

    fn op(name: &str) -> OpId {
        OpId(ActionId(name.to_owned()), 0)
    }

    fn scope(name: &str) -> Scope {
        Scope(name.to_owned())
    }

    fn fact(name: &str) -> FactKind {
        FactKind(name.to_owned())
    }

    fn lane(name: &str) -> LaneId {
        LaneId(name.to_owned())
    }

    fn node(op_id: OpId, lane_name: &str, requires: &[&str], writes: &[&str]) -> GraphNode {
        GraphNode {
            op_id,
            lane: lane(lane_name),
            requires: requires.iter().map(|f| fact(f)).collect(),
            writes: writes.iter().map(|s| scope(s)).collect(),
        }
    }

    fn blocker(id: &str, reason: &str) -> ConstraintBlocker {
        ConstraintBlocker {
            constraint_id: ConstraintId(id.to_owned()),
            reason: reason.to_owned(),
        }
    }

    fn violation(id: &str, reason: &str) -> ConstraintViolation {
        ConstraintViolation {
            constraint_id: ConstraintId(id.to_owned()),
            reason: reason.to_owned(),
        }
    }

    fn inputs<'a>(
        frontier: Option<&'a FrontierBlock>,
        constraint: Option<&'a ConstraintDecision>,
        lane: Option<&'a LaneRejection>,
        drained_writes: &'a [DrainedWriteScope],
        unmet_facts: &'a [FactKind],
        active_scope_holders: &'a [ActiveScopeHolder],
    ) -> WhyNotParallelInputs<'a> {
        WhyNotParallelInputs {
            frontier,
            constraint,
            lane,
            drained_writes,
            unmet_facts,
            active_scope_holders,
        }
    }

    fn none_inputs<'a>() -> WhyNotParallelInputs<'a> {
        inputs(None, None, None, &[], &[], &[])
    }

    #[test]
    fn reason_from_frontier_wraps_the_block_verbatim() {
        let block = FrontierBlock::LaneAtCapacity {
            lane: LaneId("l1".to_owned()),
        };
        let rejection = FrontierRejection {
            op_id: op("a"),
            reason: block.clone(),
        };
        assert_eq!(
            reason_from_frontier(&rejection),
            NotParallelReason::Frontier(block)
        );
    }

    #[test]
    fn an_op_with_no_cause_is_parallelizable() {
        let out = why_not_parallel(op("a"), &none_inputs());
        assert!(out.reasons.is_empty());
        assert!(out.is_parallelizable());
    }

    #[test]
    fn allow_and_restrictions_contribute_no_reason() {
        let allow = ConstraintDecision::Allow {
            required_leases: Vec::new(),
        };
        let restricted = ConstraintDecision::AllowWithRestrictions {
            restrictions: vec!["noted".to_owned()],
        };
        for decision in [&allow, &restricted] {
            let case = inputs(None, Some(decision), None, &[], &[], &[]);
            assert!(why_not_parallel(op("a"), &case).is_parallelizable());
        }
    }

    #[test]
    fn a_lane_rejection_is_surfaced() {
        let rejection = LaneRejection::CapabilityMismatch;
        let case = inputs(None, None, Some(&rejection), &[], &[], &[]);
        let out = why_not_parallel(op("a"), &case);
        assert_eq!(
            out.reasons,
            vec![NotParallelReason::LaneRejected(rejection)]
        );
    }

    #[test]
    fn pair_conflict_names_a_shared_write_scope() {
        let left: BTreeSet<Scope> = [scope("s1"), scope("s2")].into_iter().collect();
        let right: BTreeSet<Scope> = [scope("s2"), scope("s3")].into_iter().collect();
        assert_eq!(
            pair_conflict(&left, &right),
            Some(PairConflict::WriteScopeConflict { scope: scope("s2") })
        );
    }

    #[test]
    fn disjoint_writes_have_no_pair_conflict() {
        let left: BTreeSet<Scope> = [scope("s1")].into_iter().collect();
        let right: BTreeSet<Scope> = [scope("s2")].into_iter().collect();
        assert_eq!(pair_conflict(&left, &right), None);
    }

    #[test]
    fn index_wrapper_uses_graph_causes_and_frontier_verbatim() -> Result<(), &'static str> {
        let mut index = GraphIndex::new();
        index.add_node(node(op("a"), "lane1", &[], &["s1"]));
        index.add_node(node(op("b"), "lane1", &["f1"], &["s1"]));
        index.mark_active(&op("a"));
        let frontier = FrontierBlock::LaneAtCapacity {
            lane: lane("lane1"),
        };

        let out = why_not_parallel_from_index(&index, &op("b"), Some(&frontier))
            .ok_or("op must be present")?;
        assert_eq!(
            out.reasons,
            vec![
                NotParallelReason::BlockedOnFact { fact: fact("f1") },
                NotParallelReason::BlockedOnActiveScope {
                    scope: scope("s1"),
                    held_by: op("a"),
                },
                NotParallelReason::Frontier(frontier),
            ]
        );
        assert!(why_not_parallel_from_index(&index, &op("missing"), None).is_none());
        Ok(())
    }

    /// Assert the aggregation is value-faithful (every supplied cause appears as
    /// exactly one reason carrying its values, nothing invented or dropped) and
    /// sound (`is_parallelizable()` ⟺ zero causes). Returns the produced answer.
    fn assert_faithful(op_id: OpId, case: &WhyNotParallelInputs<'_>) -> WhyNotParallel {
        let out = why_not_parallel(op_id, case);

        for fact in case.unmet_facts {
            assert!(out
                .reasons
                .iter()
                .any(|r| matches!(r, NotParallelReason::BlockedOnFact { fact: f } if f == fact)));
        }
        for holder in case.active_scope_holders {
            assert!(out.reasons.iter().any(|r| matches!(
                r,
                NotParallelReason::BlockedOnActiveScope { scope, held_by }
                    if *scope == holder.scope && *held_by == holder.held_by
            )));
        }
        for drained in case.drained_writes {
            assert!(out.reasons.iter().any(|r| matches!(
                r,
                NotParallelReason::DrainRegion { target, scope }
                    if *target == drained.target && *scope == drained.scope
            )));
        }
        if let Some(lane) = case.lane {
            assert!(out
                .reasons
                .iter()
                .any(|r| matches!(r, NotParallelReason::LaneRejected(x) if x == lane)));
        }
        if let Some(block) = case.frontier {
            assert!(out
                .reasons
                .iter()
                .any(|r| matches!(r, NotParallelReason::Frontier(b) if b == block)));
        }
        let constraint_count = match case.constraint {
            Some(ConstraintDecision::Wait { blockers }) => {
                for b in blockers {
                    assert!(out
                        .reasons
                        .iter()
                        .any(|r| matches!(r, NotParallelReason::ConstraintWait(x) if x == b)));
                }
                blockers.len()
            }
            Some(ConstraintDecision::Deny { violations }) => {
                for v in violations {
                    assert!(out
                        .reasons
                        .iter()
                        .any(|r| matches!(r, NotParallelReason::ConstraintDeny(x) if x == v)));
                }
                violations.len()
            }
            Some(
                ConstraintDecision::Allow { .. } | ConstraintDecision::AllowWithRestrictions { .. },
            )
            | None => 0,
        };

        let expected = case.unmet_facts.len()
            + case.active_scope_holders.len()
            + case.drained_writes.len()
            + usize::from(case.lane.is_some())
            + usize::from(case.frontier.is_some())
            + constraint_count;
        assert_eq!(
            out.reasons.len(),
            expected,
            "a cause was invented or dropped"
        );
        assert_eq!(
            out.is_parallelizable(),
            expected == 0,
            "soundness: parallelizable iff no cause"
        );
        out
    }

    /// Load-bearing property: across the product of all input categories the
    /// aggregation is value-faithful and sound, and every reason variant is
    /// produced at least once (non-vacuity).
    #[test]
    fn aggregation_is_faithful_and_sound() {
        let frontiers = [
            None,
            Some(FrontierBlock::LaneAtCapacity {
                lane: LaneId("l1".to_owned()),
            }),
        ];
        let constraints = [
            None,
            Some(ConstraintDecision::Wait {
                blockers: vec![blocker("c1", "busy")],
            }),
            Some(ConstraintDecision::Deny {
                violations: vec![violation("c2", "frozen")],
            }),
            Some(ConstraintDecision::Allow {
                required_leases: Vec::new(),
            }),
        ];
        let lanes = [None, Some(LaneRejection::CapabilityMismatch)];
        let drains = [
            Vec::new(),
            vec![DrainedWriteScope {
                target: DrainTarget::Global,
                scope: scope("s1"),
            }],
        ];
        let facts = [Vec::new(), vec![FactKind("f1".to_owned())]];
        let actives = [
            Vec::new(),
            vec![ActiveScopeHolder {
                scope: scope("s2"),
                held_by: op("h"),
            }],
        ];

        let mut saw_frontier = false;
        let mut saw_wait = false;
        let mut saw_deny = false;
        let mut saw_lane = false;
        let mut saw_drain = false;
        let mut saw_fact = false;
        let mut saw_active = false;

        for frontier in &frontiers {
            for constraint in &constraints {
                for lane in &lanes {
                    for drained in &drains {
                        for unmet in &facts {
                            for holders in &actives {
                                let case = inputs(
                                    frontier.as_ref(),
                                    constraint.as_ref(),
                                    lane.as_ref(),
                                    drained,
                                    unmet,
                                    holders,
                                );
                                let out = assert_faithful(op("a"), &case);
                                for reason in &out.reasons {
                                    match reason {
                                        NotParallelReason::Frontier(_) => saw_frontier = true,
                                        NotParallelReason::ConstraintWait(_) => saw_wait = true,
                                        NotParallelReason::ConstraintDeny(_) => saw_deny = true,
                                        NotParallelReason::LaneRejected(_) => saw_lane = true,
                                        NotParallelReason::DrainRegion { .. } => saw_drain = true,
                                        NotParallelReason::BlockedOnFact { .. } => saw_fact = true,
                                        NotParallelReason::BlockedOnActiveScope { .. } => {
                                            saw_active = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        assert!(saw_frontier, "frontier reason never produced");
        assert!(saw_wait, "constraint-wait reason never produced");
        assert!(saw_deny, "constraint-deny reason never produced");
        assert!(saw_lane, "lane-rejected reason never produced");
        assert!(saw_drain, "drain-region reason never produced");
        assert!(saw_fact, "blocked-on-fact reason never produced");
        assert!(saw_active, "blocked-on-active-scope reason never produced");
    }
}
