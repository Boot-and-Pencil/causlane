//! Facade/kernel fuzz target for the M12.5 API validation loop.
//!
//! The target drives public `causlane` facade and curated core APIs. It checks
//! consistency properties over admission, policy helpers, constraint delegation
//! and frontier outputs without replacing the kernel authorities.

#![no_main]

use std::collections::{BTreeMap, BTreeSet};

use causlane::core::kernel::{self, ConstraintProvider};
use causlane::core::protocol::{
    ConstraintDecision, ConstraintId, ConstraintKind, ConstraintSnapshot, ConstraintSpec,
    CorrelationId, FrontierBlock, FrontierRejection, ResourceClaim,
};
use causlane::prelude::{
    ActionCall, ActionId, ClaimMode, ConsequenceProfile, ConstraintEpoch, FactKind, GraphIndex,
    GraphNode, KernelContracts, LaneCapacity, LaneId, OpId, PredicateId, ResourceId, Scope,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    run_case(data);
});

fn run_case(data: &[u8]) {
    let input = Input::new(data);
    exercise_admission_and_policy(&input);
    exercise_constraint_delegation(&input);
    exercise_frontier_consistency(&input);
}

fn exercise_admission_and_policy(input: &Input<'_>) {
    let call = ActionCall {
        action_id: ActionId(format!("facade.fuzz.action.{}", input.byte(0) % 8)),
        predicate: PredicateId("facade.fuzz.predicate".to_owned()),
        subject_ref: "service:fuzz".to_owned(),
        circumstance_ref: "environment:fuzz".to_owned(),
        correlation_id: CorrelationId(format!("corr-facade-fuzz-{}", input.byte(1))),
    };

    let admission = kernel::admit_call(&call);
    assert!(
        matches!(
            admission,
            kernel::DispatchAdmission::Accepted { ref action_id } if action_id == &call.action_id
        ),
        "facade admission unexpectedly refused or changed id: {admission:?}"
    );

    assert!(kernel::requires_execution_barrier(
        ConsequenceProfile::RuntimeExecution
    ));
    assert!(kernel::can_commit_observed_truth(
        ConsequenceProfile::RuntimeExecution
    ));
    assert!(!kernel::requires_execution_barrier(
        ConsequenceProfile::ProjectionRead
    ));
    assert!(!kernel::can_commit_observed_truth(
        ConsequenceProfile::ProjectionRead
    ));
}

fn exercise_constraint_delegation(input: &Input<'_>) {
    let limit = u64::from(input.byte(2) % 8);
    let amount = u64::from(input.byte(3) % 10);
    let claims = vec![ResourceClaim {
        resource: ResourceId("facade-fuzz-slots".to_owned()),
        scope: Scope("pool:fuzz".to_owned()),
        mode: ClaimMode::Token,
        amount,
    }];
    let snapshot = ConstraintSnapshot {
        snapshot_id: ConstraintId("snapshot_facade_fuzz".to_owned()),
        epoch: ConstraintEpoch(1),
        constraints: vec![ConstraintSpec {
            constraint_id: ConstraintId("constraint_facade_fuzz_slots".to_owned()),
            kind: ConstraintKind::TokenBudget {
                resource: ResourceId("facade-fuzz-slots".to_owned()),
                scope: Scope("pool:fuzz".to_owned()),
                limit,
            },
        }],
        active_leases: Vec::new(),
    };

    let direct = kernel::resolve_constraints(&snapshot, &claims, &KernelContracts);
    let delegated = KernelContracts.resolve(&snapshot, &claims);
    assert_eq!(direct, delegated);
    if amount > limit {
        assert!(matches!(direct, ConstraintDecision::Deny { .. }));
    }
}

fn exercise_frontier_consistency(input: &Input<'_>) {
    let mut index = GraphIndex::new();
    let produced = FactKind("fact:available".to_owned());
    if input.produce_fact() {
        index.mark_produced(&produced);
    }

    for slot in 0_u32..6 {
        let op_id = op(slot);
        index.add_node(GraphNode {
            op_id: op_id.clone(),
            lane: input.lane(slot),
            requires: input.requires(slot, &produced),
            writes: vec![input.scope(slot)],
        });
        if input.active(slot) {
            index.mark_active(&op_id);
        }
    }

    let lanes = input.lanes();
    let selection = kernel::select_frontier(&index, &lanes);
    assert_selected_ops_are_structurally_ready(&index, &selection.selected);
    assert_selected_write_scopes_are_unique(&index, &selection.selected);
    assert_selected_lanes_respect_capacity(&index, &lanes, &selection.selected);
    assert_rejections_are_self_consistent(&index, &selection.selected, &selection.rejected);
}

fn assert_selected_ops_are_structurally_ready(index: &GraphIndex, selected: &BTreeSet<OpId>) {
    for op_id in selected {
        assert!(
            index.node(op_id).is_some(),
            "selected op is missing from graph"
        );
        assert!(
            index.unmet_facts_for(op_id).is_empty(),
            "selected op has unmet fact requirements"
        );
        assert!(
            index.active_scope_holders_for(op_id).is_empty(),
            "selected op conflicts with an active writer"
        );
    }
}

fn assert_selected_write_scopes_are_unique(index: &GraphIndex, selected: &BTreeSet<OpId>) {
    let mut scopes = BTreeSet::new();
    for op_id in selected {
        let Some(node) = index.node(op_id) else {
            continue;
        };
        for scope in &node.writes {
            assert!(
                scopes.insert(scope.clone()),
                "selected frontier contains duplicate write scope"
            );
        }
    }
}

fn assert_selected_lanes_respect_capacity(
    index: &GraphIndex,
    lanes: &BTreeMap<LaneId, LaneCapacity>,
    selected: &BTreeSet<OpId>,
) {
    let mut selected_by_lane: BTreeMap<LaneId, u32> = BTreeMap::new();
    for op_id in selected {
        if let Some(node) = index.node(op_id) {
            *selected_by_lane.entry(node.lane.clone()).or_insert(0) += 1;
        }
    }
    for (lane, selected_count) in selected_by_lane {
        if let Some(LaneCapacity::Bounded(limit)) = lanes.get(&lane) {
            let total = index.active_in_lane(&lane).saturating_add(selected_count);
            assert!(total <= *limit, "selected lane exceeds bounded capacity");
        }
    }
}

fn assert_rejections_are_self_consistent(
    index: &GraphIndex,
    selected: &BTreeSet<OpId>,
    rejections: &[FrontierRejection],
) {
    for rejection in rejections {
        let Some(rejected_node) = index.node(&rejection.op_id) else {
            assert!(false, "rejected op is missing from graph");
            return;
        };
        match &rejection.reason {
            FrontierBlock::LaneAtCapacity { lane } => {
                assert_eq!(lane, &rejected_node.lane);
            }
            FrontierBlock::WriteScopeConflict { scope, with } => {
                assert!(selected.contains(with));
                let Some(holder_node) = index.node(with) else {
                    assert!(false, "write-scope conflict holder is missing from graph");
                    return;
                };
                assert!(rejected_node.writes.contains(scope));
                assert!(holder_node.writes.contains(scope));
            }
        }
    }
}

fn op(index: u32) -> OpId {
    OpId(ActionId(format!("facade.fuzz.op.{index}")), index)
}

struct Input<'a> {
    data: &'a [u8],
}

impl<'a> Input<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn byte(&self, index: usize) -> u8 {
        self.data.get(index).copied().unwrap_or(0)
    }

    fn produce_fact(&self) -> bool {
        self.byte(4) % 2 == 0
    }

    fn active(&self, slot: u32) -> bool {
        self.byte(5 + slot as usize) % 5 == 0
    }

    fn lane(&self, slot: u32) -> LaneId {
        if self.byte(11 + slot as usize) % 2 == 0 {
            LaneId("release".to_owned())
        } else {
            LaneId("limited".to_owned())
        }
    }

    fn scope(&self, slot: u32) -> Scope {
        Scope(format!("environment:{}", self.byte(17 + slot as usize) % 3))
    }

    fn requires(&self, slot: u32, produced: &FactKind) -> Vec<FactKind> {
        if self.byte(23 + slot as usize) % 3 == 0 {
            vec![produced.clone()]
        } else {
            Vec::new()
        }
    }

    fn lanes(&self) -> BTreeMap<LaneId, LaneCapacity> {
        let limit = u32::from(self.byte(29) % 3);
        if self.byte(30) % 2 == 0 {
            BTreeMap::from([(LaneId("limited".to_owned()), LaneCapacity::Bounded(limit))])
        } else {
            BTreeMap::new()
        }
    }
}
