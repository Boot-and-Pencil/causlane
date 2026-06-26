//! Graph indexes over the op graph (M05.4).
//!
//! Four deterministic (`BTreeMap`) indexes classify every op node by the keys a
//! frontier selector (M05.5) queries:
//!   - `wait_by_fact`         — ops blocked on a required fact not yet produced;
//!   - `wait_by_scope`        — ops blocked because an active op writes their scope;
//!   - `active_by_write_scope`— active ops by the scope they write (conflict source);
//!   - `ready_by_lane`        — structurally-ready ops grouped by their lane.
//!
//! "Structurally ready" here means all required facts are produced and no active
//! op writes a conflicting scope; the constraint-plane (M05.3) and lane-capacity
//! (M05.2) budgets are layered on top by frontier selection. The index is
//! maintained INCREMENTALLY (`mark_produced` / `mark_active` / `mark_complete`
//! recompute only the affected nodes); `incremental_matches_full_rebuild` proves
//! the incremental state always equals [`GraphIndex::from_state`] from scratch.

use std::collections::{BTreeMap, BTreeSet};

use super::{ActionId, FactKind, LaneId, Scope};

/// Identity of an op: its action plus the op's index within the plan.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OpId(pub ActionId, pub u32);

/// An op-graph node: its lane, the facts it requires, and the scopes it writes
/// (derived from the op's effect signature and its lane assignment).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNode {
    /// The op this node is for.
    pub op_id: OpId,
    /// The lane the op is assigned to.
    pub lane: LaneId,
    /// Fact kinds the op requires before it can run.
    pub requires: Vec<FactKind>,
    /// Scopes the op writes.
    pub writes: Vec<Scope>,
}

/// Deterministic indexes over the op graph with incremental maintenance.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphIndex {
    nodes: BTreeMap<OpId, GraphNode>,
    produced: BTreeSet<FactKind>,
    active: BTreeSet<OpId>,
    wait_by_fact: BTreeMap<FactKind, BTreeSet<OpId>>,
    wait_by_scope: BTreeMap<Scope, BTreeSet<OpId>>,
    active_by_write_scope: BTreeMap<Scope, BTreeSet<OpId>>,
    ready_by_lane: BTreeMap<LaneId, BTreeSet<OpId>>,
}

/// Remove `op_id` from every set in `map`, dropping any set left empty.
fn drop_op<K: Ord>(map: &mut BTreeMap<K, BTreeSet<OpId>>, op_id: &OpId) {
    map.retain(|_, set| {
        set.remove(op_id);
        !set.is_empty()
    });
}

impl GraphIndex {
    /// An empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the index from a full state: the node set, the produced facts, and
    /// the active op set. Active ops are indexed first so non-active nodes see
    /// the complete `active_by_write_scope` when classified.
    #[must_use]
    pub fn from_state(
        nodes: impl IntoIterator<Item = GraphNode>,
        produced: BTreeSet<FactKind>,
        active: BTreeSet<OpId>,
    ) -> Self {
        let mut index = GraphIndex {
            produced,
            active,
            ..Self::default()
        };
        for node in nodes {
            let _previous = index.nodes.insert(node.op_id.clone(), node);
        }
        let active_ids: Vec<OpId> = index.active.iter().cloned().collect();
        for op_id in &active_ids {
            index.recompute(op_id);
        }
        let waiting_ids: Vec<OpId> = index
            .nodes
            .keys()
            .filter(|op_id| !index.active.contains(*op_id))
            .cloned()
            .collect();
        for op_id in &waiting_ids {
            index.recompute(op_id);
        }
        index
    }

    /// A full rebuild from this index's own base state (`incremental == rebuilt`
    /// is the load-bearing invariant).
    #[must_use]
    pub fn rebuilt(&self) -> GraphIndex {
        GraphIndex::from_state(
            self.nodes.values().cloned(),
            self.produced.clone(),
            self.active.clone(),
        )
    }

    /// Add a node to the graph (its classification is computed immediately).
    pub fn add_node(&mut self, node: GraphNode) {
        let op_id = node.op_id.clone();
        let _previous = self.nodes.insert(op_id.clone(), node);
        self.recompute(&op_id);
    }

    /// Record that `fact` is now produced; reclassify ops that required it.
    pub fn mark_produced(&mut self, fact: &FactKind) {
        let _new = self.produced.insert(fact.clone());
        for op_id in self.nodes_requiring(fact) {
            self.recompute(&op_id);
        }
    }

    /// Mark `op_id` active; index its write scopes and reclassify ops that would
    /// now conflict with it.
    pub fn mark_active(&mut self, op_id: &OpId) {
        if !self.nodes.contains_key(op_id) {
            return;
        }
        let _new = self.active.insert(op_id.clone());
        self.recompute(op_id);
        for affected in self.nodes_writing_any(&self.write_scopes(op_id), op_id) {
            self.recompute(&affected);
        }
    }

    /// Remove `op_id` from the graph (it completed); reclassify ops that wrote
    /// the same scopes, which may now be unblocked.
    pub fn mark_complete(&mut self, op_id: &OpId) {
        let scopes = self.write_scopes(op_id);
        let _was_active = self.active.remove(op_id);
        let _node = self.nodes.remove(op_id);
        self.remove_from_indexes(op_id);
        for affected in self.nodes_writing_any(&scopes, op_id) {
            self.recompute(&affected);
        }
    }

    /// Ops blocked waiting for `fact` to be produced.
    #[must_use]
    pub fn waiting_on_fact(&self, fact: &FactKind) -> Option<&BTreeSet<OpId>> {
        self.wait_by_fact.get(fact)
    }

    /// Ops blocked because an active op writes `scope`.
    #[must_use]
    pub fn waiting_on_scope(&self, scope: &Scope) -> Option<&BTreeSet<OpId>> {
        self.wait_by_scope.get(scope)
    }

    /// Active ops writing `scope`.
    #[must_use]
    pub fn active_writers(&self, scope: &Scope) -> Option<&BTreeSet<OpId>> {
        self.active_by_write_scope.get(scope)
    }

    /// Structurally-ready ops in `lane`.
    #[must_use]
    pub fn ready_in_lane(&self, lane: &LaneId) -> Option<&BTreeSet<OpId>> {
        self.ready_by_lane.get(lane)
    }

    /// All currently-ready nodes in `OpId` order — the frontier-selection
    /// candidates (M05.5).
    #[must_use]
    pub fn ready_nodes(&self) -> Vec<&GraphNode> {
        let ready: BTreeSet<&OpId> = self.ready_by_lane.values().flatten().collect();
        ready
            .into_iter()
            .filter_map(|op_id| self.nodes.get(op_id))
            .collect()
    }

    /// The graph node for `op_id`, if present.
    #[must_use]
    pub fn node(&self, op_id: &OpId) -> Option<&GraphNode> {
        self.nodes.get(op_id)
    }

    /// All graph nodes in deterministic `OpId` order.
    #[must_use]
    pub fn nodes(&self) -> Vec<&GraphNode> {
        self.nodes.values().collect()
    }

    /// Required facts for `op_id` that have not yet been produced.
    #[must_use]
    pub fn unmet_facts_for(&self, op_id: &OpId) -> Vec<FactKind> {
        self.nodes
            .get(op_id)
            .map(|node| {
                node.requires
                    .iter()
                    .filter(|fact| !self.produced.contains(*fact))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Active ops holding write scopes also written by `op_id`, in deterministic
    /// `(scope, holder)` order. The queried op never blocks itself.
    #[must_use]
    pub fn active_scope_holders_for(&self, op_id: &OpId) -> Vec<(Scope, OpId)> {
        let mut holders = Vec::new();
        let Some(node) = self.nodes.get(op_id) else {
            return holders;
        };
        for scope in &node.writes {
            if let Some(active) = self.active_by_write_scope.get(scope) {
                for holder in active {
                    if holder != op_id {
                        holders.push((scope.clone(), holder.clone()));
                    }
                }
            }
        }
        holders
    }

    /// Count of active ops assigned to `lane` (the lane's occupied capacity).
    #[must_use]
    pub fn active_in_lane(&self, lane: &LaneId) -> u32 {
        let count = self
            .active
            .iter()
            .filter(|op_id| self.nodes.get(op_id).is_some_and(|node| node.lane == *lane))
            .count();
        u32::try_from(count).unwrap_or(u32::MAX)
    }

    fn write_scopes(&self, op_id: &OpId) -> Vec<Scope> {
        self.nodes
            .get(op_id)
            .map(|node| node.writes.clone())
            .unwrap_or_default()
    }

    fn nodes_requiring(&self, fact: &FactKind) -> Vec<OpId> {
        self.nodes
            .values()
            .filter(|node| node.requires.iter().any(|f| f == fact))
            .map(|node| node.op_id.clone())
            .collect()
    }

    fn nodes_writing_any(&self, scopes: &[Scope], except: &OpId) -> Vec<OpId> {
        self.nodes
            .values()
            .filter(|node| {
                node.op_id != *except && node.writes.iter().any(|scope| scopes.contains(scope))
            })
            .map(|node| node.op_id.clone())
            .collect()
    }

    fn remove_from_indexes(&mut self, op_id: &OpId) {
        drop_op(&mut self.wait_by_fact, op_id);
        drop_op(&mut self.wait_by_scope, op_id);
        drop_op(&mut self.active_by_write_scope, op_id);
        drop_op(&mut self.ready_by_lane, op_id);
    }

    /// Re-classify a single op: clear its index entries, then place it by its
    /// current state (active → write-scope index; otherwise blocked-by-fact /
    /// blocked-by-scope / ready).
    fn recompute(&mut self, op_id: &OpId) {
        self.remove_from_indexes(op_id);
        let Some(node) = self.nodes.get(op_id).cloned() else {
            return;
        };
        if self.active.contains(op_id) {
            for scope in &node.writes {
                self.active_by_write_scope
                    .entry(scope.clone())
                    .or_default()
                    .insert(op_id.clone());
            }
            return;
        }
        let mut blocked = false;
        for fact in &node.requires {
            if !self.produced.contains(fact) {
                blocked = true;
                self.wait_by_fact
                    .entry(fact.clone())
                    .or_default()
                    .insert(op_id.clone());
            }
        }
        for scope in &node.writes {
            if self.active_by_write_scope.contains_key(scope) {
                blocked = true;
                self.wait_by_scope
                    .entry(scope.clone())
                    .or_default()
                    .insert(op_id.clone());
            }
        }
        if !blocked {
            self.ready_by_lane
                .entry(node.lane.clone())
                .or_default()
                .insert(op_id.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{GraphIndex, GraphNode, OpId};
    use crate::domain::{ActionId, FactKind, LaneId, Scope};

    fn op(action: &str, index: u32) -> OpId {
        OpId(ActionId(action.to_owned()), index)
    }

    fn fact(name: &str) -> FactKind {
        FactKind(name.to_owned())
    }

    fn lane(name: &str) -> LaneId {
        LaneId(name.to_owned())
    }

    fn scope(name: &str) -> Scope {
        Scope(name.to_owned())
    }

    fn node(op_id: OpId, lane_name: &str, requires: &[&str], writes: &[&str]) -> GraphNode {
        GraphNode {
            op_id,
            lane: lane(lane_name),
            requires: requires.iter().map(|f| fact(f)).collect(),
            writes: writes.iter().map(|s| scope(s)).collect(),
        }
    }

    #[test]
    fn a_node_waiting_on_a_fact_becomes_ready_when_it_is_produced() {
        let mut index = GraphIndex::new();
        index.add_node(node(op("b", 0), "lane1", &["f1"], &["s1"]));

        assert!(index.waiting_on_fact(&fact("f1")).is_some());
        assert!(index.ready_in_lane(&lane("lane1")).is_none());

        index.mark_produced(&fact("f1"));
        assert!(index.waiting_on_fact(&fact("f1")).is_none());
        assert!(index.ready_in_lane(&lane("lane1")).is_some());
    }

    #[test]
    fn an_active_writer_blocks_a_conflicting_op_on_its_scope() {
        let mut index = GraphIndex::new();
        index.add_node(node(op("a", 0), "lane1", &[], &["s1"]));
        index.add_node(node(op("b", 0), "lane2", &[], &["s1"]));

        // Both are ready until one goes active.
        assert!(index.ready_in_lane(&lane("lane1")).is_some());
        index.mark_active(&op("a", 0));

        // a now holds s1; b is blocked on s1 and no longer ready.
        assert!(index.active_writers(&scope("s1")).is_some());
        assert!(index.waiting_on_scope(&scope("s1")).is_some());
        assert!(index.ready_in_lane(&lane("lane2")).is_none());

        // a completes -> b unblocks.
        index.mark_complete(&op("a", 0));
        assert!(index.active_writers(&scope("s1")).is_none());
        assert!(index.ready_in_lane(&lane("lane2")).is_some());
    }

    /// Load-bearing invariant: after every incremental event the index equals a
    /// full rebuild from its own base state, across a graph that exercises all
    /// four indexes (a fact dependency, a write-scope conflict, multiple lanes).
    #[test]
    fn incremental_matches_full_rebuild() {
        type Event = Box<dyn Fn(&mut GraphIndex)>;

        let mut index = GraphIndex::new();
        let mut saw_wait_fact = false;
        let mut saw_wait_scope = false;
        let mut saw_active = false;
        let mut saw_ready = false;

        let events: Vec<Event> = vec![
            Box::new(|g| g.add_node(node(op("a", 0), "compute", &[], &["s1"]))),
            Box::new(|g| g.add_node(node(op("b", 0), "compute", &["f1"], &["s2"]))),
            Box::new(|g| g.add_node(node(op("c", 0), "io", &[], &["s1"]))),
            Box::new(|g| g.add_node(node(op("d", 0), "io", &["f1"], &["s1"]))),
            Box::new(|g| g.mark_active(&op("a", 0))),
            Box::new(|g| g.mark_produced(&fact("f1"))),
            Box::new(|g| g.mark_active(&op("b", 0))),
            Box::new(|g| g.mark_complete(&op("a", 0))),
            Box::new(|g| g.mark_complete(&op("b", 0))),
        ];

        for apply in &events {
            apply(&mut index);
            assert_eq!(
                index,
                index.rebuilt(),
                "incremental index diverged from a full rebuild"
            );
            saw_wait_fact |= index.waiting_on_fact(&fact("f1")).is_some();
            saw_wait_scope |= index.waiting_on_scope(&scope("s1")).is_some();
            saw_active |= index.active_writers(&scope("s1")).is_some();
            saw_ready |= index.ready_in_lane(&lane("compute")).is_some();
        }

        // Non-vacuity: every index was populated at some point.
        assert!(saw_wait_fact, "wait_by_fact never populated");
        assert!(saw_wait_scope, "wait_by_scope never populated");
        assert!(saw_active, "active_by_write_scope never populated");
        assert!(saw_ready, "ready_by_lane never populated");
    }

    #[test]
    fn from_state_is_deterministic() {
        let nodes = vec![
            node(op("a", 0), "lane1", &[], &["s1"]),
            node(op("b", 0), "lane1", &["f1"], &["s2"]),
        ];
        let built = GraphIndex::from_state(nodes.clone(), BTreeSet::default(), BTreeSet::default());
        let again = GraphIndex::from_state(nodes, BTreeSet::default(), BTreeSet::default());
        assert_eq!(built, again);
    }

    #[test]
    fn read_only_explain_accessors_surface_index_causes() {
        let mut index = GraphIndex::new();
        index.add_node(node(op("a", 0), "lane1", &[], &["s1"]));
        index.add_node(node(op("b", 0), "lane1", &["f1", "f2"], &["s1"]));
        index.mark_active(&op("a", 0));
        index.mark_produced(&fact("f2"));

        assert_eq!(
            index
                .nodes()
                .into_iter()
                .map(|n| n.op_id.clone())
                .collect::<Vec<_>>(),
            vec![op("a", 0), op("b", 0)]
        );
        assert_eq!(
            index.node(&op("b", 0)).map(|n| n.lane.clone()),
            Some(lane("lane1"))
        );
        assert_eq!(index.unmet_facts_for(&op("b", 0)), vec![fact("f1")]);
        assert_eq!(
            index.active_scope_holders_for(&op("b", 0)),
            vec![(scope("s1"), op("a", 0))]
        );
        assert!(index.active_scope_holders_for(&op("a", 0)).is_empty());
    }
}
