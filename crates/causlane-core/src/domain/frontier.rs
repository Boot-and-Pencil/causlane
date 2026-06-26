//! Safe frontier selection (M05.5).
//!
//! Selects the set of ready ops that may run concurrently right now — the
//! *frontier* — from the M05.4 [`GraphIndex`]. The guarantee is a **conflict-free
//! antichain within lane budget**:
//!   - conflict-free: no two selected ops write the same `Scope` (exact-match,
//!     consistent with `active_by_write_scope`);
//!   - antichain / no hard deps: every selected op is structurally ready (all its
//!     required facts are already produced) — it is drawn from `ready_by_lane`;
//!   - lane/resource budget: per lane, already-active + selected ≤ capacity
//!     (M05.2 `LaneCapacity::has_room`); a write scope is an exclusive resource
//!     (at most one selected writer).
//!
//! `ready_by_lane` already excludes ops that conflict with an *active* writer; the
//! novel safety this adds is the *pending-vs-pending* write conflict between two
//! ready ops. Selection is greedy in `OpId` order (deterministic for replay /
//! formal parity). Every rejected op carries a [`FrontierBlock`] reason — the seed
//! for M05.8 why-not-parallel.
//!
//! Out of scope (follow-ups): read-write conflicts (`GraphNode` carries only
//! `writes`); constraint-plane freeze/token gating inside selection (applied
//! upstream at admission; M05.7 wires runtime updates → rebuild); verified-merge
//! relaxation of the exact-scope conflict.

use std::collections::{BTreeMap, BTreeSet};

use super::{GraphIndex, LaneCapacity, LaneId, OpId, Scope};

/// Why a ready op was kept out of the frontier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrontierBlock {
    /// The op's lane is already at capacity.
    LaneAtCapacity {
        /// The lane whose budget is exhausted.
        lane: LaneId,
    },
    /// The op writes a scope already claimed by a selected op.
    WriteScopeConflict {
        /// The contended write scope.
        scope: Scope,
        /// The already-selected op that holds it.
        with: OpId,
    },
}

/// A ready op that was not admitted to the frontier, with the reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontierRejection {
    /// The op that was rejected.
    pub op_id: OpId,
    /// Why it was rejected.
    pub reason: FrontierBlock,
}

/// The outcome of frontier selection: the conflict-free antichain plus the
/// rejected ops and why (the latter feeds M05.8 why-not-parallel).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrontierSelection {
    /// The selected frontier — a conflict-free antichain within lane budget.
    pub selected: BTreeSet<OpId>,
    /// Ready ops kept out of the frontier, each with its blocking reason.
    pub rejected: Vec<FrontierRejection>,
}

/// Select a safe frontier from the ready ops in `index`, respecting each lane's
/// capacity in `lanes` (a lane absent from `lanes` is treated as unbounded).
///
/// Greedy in `OpId` order: an op joins the frontier unless its lane is at
/// capacity or one of its write scopes is already claimed by a selected op.
#[must_use]
pub fn select_frontier(
    index: &GraphIndex,
    lanes: &BTreeMap<LaneId, LaneCapacity>,
) -> FrontierSelection {
    let mut selected = BTreeSet::new();
    let mut rejected = Vec::new();
    let mut claimed_scopes: BTreeMap<Scope, OpId> = BTreeMap::new();
    let mut lane_used: BTreeMap<LaneId, u32> = BTreeMap::new();

    for node in index.ready_nodes() {
        let lane = &node.lane;
        let used = *lane_used
            .entry(lane.clone())
            .or_insert_with(|| index.active_in_lane(lane));
        let capacity = lanes.get(lane).copied().unwrap_or(LaneCapacity::Unbounded);
        if !capacity.has_room(used) {
            rejected.push(FrontierRejection {
                op_id: node.op_id.clone(),
                reason: FrontierBlock::LaneAtCapacity { lane: lane.clone() },
            });
            continue;
        }
        let conflict = node
            .writes
            .iter()
            .find_map(|scope| claimed_scopes.get(scope).map(|holder| (scope, holder)));
        if let Some((scope, holder)) = conflict {
            rejected.push(FrontierRejection {
                op_id: node.op_id.clone(),
                reason: FrontierBlock::WriteScopeConflict {
                    scope: scope.clone(),
                    with: holder.clone(),
                },
            });
            continue;
        }
        for scope in &node.writes {
            let _previous = claimed_scopes.insert(scope.clone(), node.op_id.clone());
        }
        let slot = lane_used.entry(lane.clone()).or_insert(0);
        *slot = slot.saturating_add(1);
        let _new = selected.insert(node.op_id.clone());
    }

    FrontierSelection { selected, rejected }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{select_frontier, FrontierBlock, FrontierRejection};
    use crate::domain::{
        ActionId, FactKind, GraphIndex, GraphNode, LaneCapacity, LaneId, OpId, Scope,
    };

    fn op(action: &str, index: u32) -> OpId {
        OpId(ActionId(action.to_owned()), index)
    }

    fn lane(name: &str) -> LaneId {
        LaneId(name.to_owned())
    }

    fn node(op_id: OpId, lane_name: &str, requires: &[&str], writes: &[&str]) -> GraphNode {
        GraphNode {
            op_id,
            lane: lane(lane_name),
            requires: requires.iter().map(|f| FactKind((*f).to_owned())).collect(),
            writes: writes.iter().map(|s| Scope((*s).to_owned())).collect(),
        }
    }

    /// Build an index whose every node is ready (no requires, none active).
    fn ready_index(nodes: Vec<GraphNode>) -> GraphIndex {
        let mut index = GraphIndex::new();
        for n in nodes {
            index.add_node(n);
        }
        index
    }

    #[test]
    fn disjoint_scopes_in_unbounded_lanes_all_select() {
        let index = ready_index(vec![
            node(op("a", 0), "l1", &[], &["s1"]),
            node(op("b", 0), "l2", &[], &["s2"]),
        ]);
        let out = select_frontier(&index, &BTreeMap::new());
        assert_eq!(out.selected, [op("a", 0), op("b", 0)].into_iter().collect());
        assert!(out.rejected.is_empty());
    }

    #[test]
    fn two_pending_ops_on_one_scope_select_exactly_one() {
        let index = ready_index(vec![
            node(op("a", 0), "l1", &[], &["s1"]),
            node(op("b", 0), "l1", &[], &["s1"]),
        ]);
        let out = select_frontier(&index, &BTreeMap::new());
        // Greedy in OpId order: a wins, b is rejected for the s1 conflict with a.
        assert_eq!(out.selected, [op("a", 0)].into_iter().collect());
        assert_eq!(
            out.rejected,
            vec![FrontierRejection {
                op_id: op("b", 0),
                reason: FrontierBlock::WriteScopeConflict {
                    scope: Scope("s1".to_owned()),
                    with: op("a", 0),
                },
            }]
        );
    }

    #[test]
    fn a_full_lane_rejects_its_ready_ops() {
        let mut index = GraphIndex::new();
        index.add_node(node(op("a", 0), "l1", &[], &["s1"]));
        index.add_node(node(op("b", 0), "l1", &[], &["s2"]));
        // One op already active in l1 occupies the single slot.
        index.mark_active(&op("a", 0));
        let lanes = [(lane("l1"), LaneCapacity::Bounded(1))]
            .into_iter()
            .collect();
        let out = select_frontier(&index, &lanes);
        assert!(out.selected.is_empty());
        assert_eq!(
            out.rejected,
            vec![FrontierRejection {
                op_id: op("b", 0),
                reason: FrontierBlock::LaneAtCapacity { lane: lane("l1") },
            }]
        );
    }

    /// Which non-vacuity cases a single selection exercised.
    struct CaseCoverage {
        selected: bool,
        lane_full: bool,
        scope_conflict: bool,
    }

    /// Assert the frontier of one case is a conflict-free antichain within budget
    /// with valid rejection causes; return which non-vacuity cases it exercised.
    fn check_case(index: &GraphIndex, lanes: &BTreeMap<LaneId, LaneCapacity>) -> CaseCoverage {
        let out = select_frontier(index, lanes);
        assert_eq!(
            out,
            select_frontier(index, lanes),
            "selection is not deterministic"
        );

        // (a) conflict-free + (c) antichain: each selected op is a ready node and
        // no two share a write scope.
        let mut claimed: BTreeMap<Scope, OpId> = BTreeMap::new();
        for n in index.ready_nodes() {
            if out.selected.contains(&n.op_id) {
                for scope in &n.writes {
                    assert!(
                        claimed.insert(scope.clone(), n.op_id.clone()).is_none(),
                        "two selected ops share a write scope"
                    );
                }
            }
        }
        for id in &out.selected {
            assert!(
                index.ready_nodes().into_iter().any(|n| n.op_id == *id),
                "selected op is not in the ready set"
            );
        }

        // (b) lane budget: active + selected ≤ capacity per lane.
        for (lane_id, cap) in lanes {
            let picked = index
                .ready_nodes()
                .into_iter()
                .filter(|n| out.selected.contains(&n.op_id) && n.lane == *lane_id)
                .count();
            let total = index.active_in_lane(lane_id) + u32::try_from(picked).unwrap_or(u32::MAX);
            if let LaneCapacity::Bounded(limit) = cap {
                assert!(total <= *limit, "lane budget exceeded");
            }
        }

        // (d) local maximality: every rejection has a valid cause.
        for rej in &out.rejected {
            match &rej.reason {
                FrontierBlock::WriteScopeConflict { scope, with } => {
                    assert!(
                        out.selected.contains(with),
                        "conflict holder must be selected"
                    );
                    assert!(
                        claimed.contains_key(scope),
                        "contended scope must be claimed"
                    );
                }
                FrontierBlock::LaneAtCapacity { lane: l } => {
                    assert!(
                        matches!(lanes.get(l), Some(LaneCapacity::Bounded(_))),
                        "lane-full only on a bounded lane"
                    );
                }
            }
        }

        CaseCoverage {
            selected: !out.selected.is_empty(),
            lane_full: out
                .rejected
                .iter()
                .any(|r| matches!(r.reason, FrontierBlock::LaneAtCapacity { .. })),
            scope_conflict: out
                .rejected
                .iter()
                .any(|r| matches!(r.reason, FrontierBlock::WriteScopeConflict { .. })),
        }
    }

    /// Load-bearing property: over a bounded space of ready graphs and lane
    /// budgets, the selected frontier is a conflict-free antichain within budget,
    /// every rejection has a valid cause (local maximality), and selection is
    /// deterministic.
    #[test]
    fn frontier_is_a_conflict_free_antichain_within_budget() {
        let scopes = ["s1", "s2"];
        let lane_names = ["l1", "l2"];
        let caps = [LaneCapacity::Unbounded, LaneCapacity::Bounded(1)];

        let mut saw_selection = false;
        let mut saw_lane_full = false;
        let mut saw_scope_conflict = false;

        // Exhaust a small space: ops a,b each get a lane (2) × write scope (2), and
        // each lane gets a capacity (2). A third op is pre-active in l1 to occupy
        // budget.
        for a_lane in lane_names {
            for a_scope in scopes {
                for b_lane in lane_names {
                    for b_scope in scopes {
                        for cap_l1 in caps {
                            for cap_l2 in caps {
                                let mut index = ready_index(vec![
                                    node(op("a", 0), a_lane, &[], &[a_scope]),
                                    node(op("b", 0), b_lane, &[], &[b_scope]),
                                    node(op("c", 0), "l1", &[], &["s3"]),
                                ]);
                                index.mark_active(&op("c", 0));

                                let lanes: BTreeMap<LaneId, LaneCapacity> =
                                    [(lane("l1"), cap_l1), (lane("l2"), cap_l2)]
                                        .into_iter()
                                        .collect();

                                let cov = check_case(&index, &lanes);
                                saw_selection |= cov.selected;
                                saw_lane_full |= cov.lane_full;
                                saw_scope_conflict |= cov.scope_conflict;
                            }
                        }
                    }
                }
            }
        }

        // Non-vacuity: selection happens, and both rejection causes are exercised.
        assert!(saw_selection, "no op was ever selected");
        assert!(saw_lane_full, "lane-at-capacity rejection never exercised");
        assert!(
            saw_scope_conflict,
            "write-scope-conflict rejection never exercised"
        );
    }
}
