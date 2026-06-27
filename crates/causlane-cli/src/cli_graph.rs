//! Shared CLI graph snapshot parsing and rendering DTOs.
//!
//! This is a CLI adapter shape, not a persisted kernel format. It builds the
//! existing `GraphIndex` authority and exposes small helpers used by explain and
//! graph export without duplicating scheduler decisions.

use std::collections::{BTreeMap, BTreeSet};

use causlane_core::{
    why_not_parallel_from_index, ActionId, DrainTarget, FactKind, FrontierBlock, FrontierRejection,
    GraphIndex, GraphNode, LaneCapacity, LaneId, LaneRejection, NotParallelReason, OpId, Scope,
    WhyNotParallel,
};
use noyalib::compat::serde_yaml;
use serde::{Deserialize, Serialize};

use crate::{read_file, CliError};

#[derive(Clone, Debug, Deserialize)]
struct GraphSnapshot {
    #[serde(default)]
    produced_facts: Vec<String>,
    #[serde(default)]
    active_ops: Vec<String>,
    #[serde(default)]
    lanes: Vec<LaneInput>,
    ops: Vec<GraphOpInput>,
}

#[derive(Clone, Debug, Deserialize)]
struct LaneInput {
    lane_id: String,
    capacity: LaneCapacityInput,
}

#[derive(Clone, Debug, Deserialize)]
struct GraphOpInput {
    action_id: String,
    op_index: u32,
    lane: String,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    writes: Vec<String>,
    #[serde(default)]
    witnesses: Vec<String>,
    #[serde(default)]
    leases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum LaneCapacityInput {
    Bounded(u32),
    Text(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GraphOpMeta {
    pub(crate) witnesses: Vec<String>,
    pub(crate) leases: Vec<String>,
}

pub(crate) struct GraphRuntime {
    pub(crate) index: GraphIndex,
    pub(crate) lanes: BTreeMap<LaneId, LaneCapacity>,
    pub(crate) active_ops: BTreeSet<OpId>,
    pub(crate) meta: BTreeMap<OpId, GraphOpMeta>,
}

#[derive(Serialize)]
pub(crate) struct OpRefDto {
    pub(crate) action_id: String,
    pub(crate) op_index: u32,
    pub(crate) display: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ReasonDto {
    pub(crate) kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) held_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) lane: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) constraint_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) capacity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) active: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) lane_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) op_tier: Option<String>,
}

pub(crate) fn read_graph(path: &str) -> Result<GraphRuntime, CliError> {
    let text = read_file(path)?;
    graph_from_str(path, &text)
}

pub(crate) fn graph_from_str(label: &str, text: &str) -> Result<GraphRuntime, CliError> {
    match serde_json::from_str::<GraphSnapshot>(text) {
        Ok(snapshot) => snapshot.into_runtime(),
        Err(json_err) => match serde_yaml::from_str::<GraphSnapshot>(text) {
            Ok(snapshot) => snapshot.into_runtime(),
            Err(yaml_err) => Err(input_error(format!(
                "invalid graph snapshot {label}: json: {json_err}; yaml: {yaml_err}"
            ))),
        },
    }
}

impl GraphSnapshot {
    fn into_runtime(self) -> Result<GraphRuntime, CliError> {
        let mut seen = BTreeSet::new();
        let mut nodes = Vec::new();
        let mut meta = BTreeMap::new();
        for op in self.ops {
            let op_id = OpId(ActionId(op.action_id), op.op_index);
            if !seen.insert(op_id.clone()) {
                return Err(input_error(format!(
                    "duplicate op {} in graph snapshot",
                    op_ref(&op_id)
                )));
            }
            let _previous = meta.insert(
                op_id.clone(),
                GraphOpMeta {
                    witnesses: sorted_unique(op.witnesses),
                    leases: sorted_unique(op.leases),
                },
            );
            nodes.push(GraphNode {
                op_id,
                lane: LaneId(op.lane),
                requires: op.requires.into_iter().map(FactKind).collect(),
                writes: op.writes.into_iter().map(Scope).collect(),
            });
        }

        let produced = self.produced_facts.into_iter().map(FactKind).collect();
        let active_ops = parse_active_ops(&self.active_ops, &seen)?;
        let index = GraphIndex::from_state(nodes, produced, active_ops.clone());
        let lanes = parse_lanes(self.lanes)?;
        Ok(GraphRuntime {
            index,
            lanes,
            active_ops,
            meta,
        })
    }
}

fn parse_lanes(values: Vec<LaneInput>) -> Result<BTreeMap<LaneId, LaneCapacity>, CliError> {
    let mut lanes = BTreeMap::new();
    for input in values {
        let lane = LaneId(input.lane_id);
        if lanes
            .insert(lane.clone(), parse_capacity(input.capacity)?)
            .is_some()
        {
            return Err(input_error(format!(
                "duplicate lane {} in graph snapshot",
                lane.0
            )));
        }
    }
    Ok(lanes)
}

fn parse_active_ops(
    values: &[String],
    known_ops: &BTreeSet<OpId>,
) -> Result<BTreeSet<OpId>, CliError> {
    let mut active = BTreeSet::new();
    for value in values {
        let op_id = parse_op_ref(value)?;
        if !known_ops.contains(&op_id) {
            return Err(input_error(format!(
                "active op {} is not declared in ops",
                op_ref(&op_id)
            )));
        }
        let _new = active.insert(op_id);
    }
    Ok(active)
}

fn parse_capacity(input: LaneCapacityInput) -> Result<LaneCapacity, CliError> {
    match input {
        LaneCapacityInput::Bounded(value) => Ok(LaneCapacity::Bounded(value)),
        LaneCapacityInput::Text(value) if value == "unbounded" => Ok(LaneCapacity::Unbounded),
        LaneCapacityInput::Text(value) => Err(input_error(format!(
            "unsupported lane capacity {value:?}; use a number or \"unbounded\""
        ))),
    }
}

pub(crate) fn parse_op_ref(value: &str) -> Result<OpId, CliError> {
    let Some((action, index)) = value.rsplit_once(':') else {
        return Err(input_error(format!(
            "op ref {value:?} must be formatted as <action_id>:<op_index>"
        )));
    };
    if action.is_empty() {
        return Err(input_error(format!("op ref {value:?} has empty action_id")));
    }
    let op_index = index.parse::<u32>().map_err(|err| {
        input_error(format!(
            "op ref {value:?} has invalid op_index {index:?}: {err}"
        ))
    })?;
    Ok(OpId(ActionId(action.to_owned()), op_index))
}

pub(crate) fn ensure_node<'a>(
    runtime: &'a GraphRuntime,
    op_id: &OpId,
) -> Result<&'a GraphNode, CliError> {
    runtime.index.node(op_id).ok_or_else(|| {
        input_error(format!(
            "op {} is not declared in graph snapshot",
            op_ref(op_id)
        ))
    })
}

pub(crate) fn explain_op_from_index(
    index: &GraphIndex,
    op_id: &OpId,
    frontier: Option<&FrontierBlock>,
) -> Result<WhyNotParallel, CliError> {
    why_not_parallel_from_index(index, op_id, frontier).ok_or_else(|| {
        input_error(format!(
            "op {} is not declared in graph snapshot",
            op_ref(op_id)
        ))
    })
}

pub(crate) fn frontier_block_for(
    rejections: &[FrontierRejection],
    op_id: &OpId,
) -> Option<FrontierBlock> {
    rejections
        .iter()
        .find(|rejection| rejection.op_id == *op_id)
        .map(|rejection| rejection.reason.clone())
}

pub(crate) fn write_set(node: &GraphNode) -> BTreeSet<Scope> {
    node.writes.iter().cloned().collect()
}

pub(crate) fn reason_dtos(reasons: &[NotParallelReason]) -> Vec<ReasonDto> {
    reasons.iter().map(reason_dto).collect()
}

pub(crate) fn reason_dto(reason: &NotParallelReason) -> ReasonDto {
    match reason {
        NotParallelReason::BlockedOnFact { fact } => dto("blocked_on_fact").fact(fact),
        NotParallelReason::BlockedOnActiveScope { scope, held_by } => {
            dto("active_scope_conflict").scope(scope).held_by(held_by)
        }
        NotParallelReason::Frontier(FrontierBlock::LaneAtCapacity { lane }) => {
            dto("lane_at_capacity").lane(lane)
        }
        NotParallelReason::Frontier(FrontierBlock::WriteScopeConflict { scope, with }) => {
            dto("frontier_write_scope_conflict").scope(scope).with(with)
        }
        NotParallelReason::ConstraintWait(blocker) => dto("constraint_wait")
            .constraint_id(&blocker.constraint_id.0)
            .reason(&blocker.reason),
        NotParallelReason::ConstraintDeny(violation) => dto("constraint_deny")
            .constraint_id(&violation.constraint_id.0)
            .reason(&violation.reason),
        NotParallelReason::LaneRejected(rejection) => lane_rejection_dto(*rejection),
        NotParallelReason::DrainRegion { target, scope } => {
            dto("drain_region").target(target).scope(scope)
        }
    }
}

fn lane_rejection_dto(rejection: LaneRejection) -> ReasonDto {
    match rejection {
        LaneRejection::WrongTier { lane_tier, op_tier } => dto("lane_wrong_tier")
            .lane_tier(&format!("{lane_tier:?}"))
            .op_tier(&format!("{op_tier:?}")),
        LaneRejection::CapabilityMismatch => dto("lane_capability_mismatch"),
        LaneRejection::CapacityExhausted { capacity, active } => dto("lane_capacity_exhausted")
            .capacity(capacity)
            .active(active),
    }
}

pub(crate) fn dto(kind: &'static str) -> ReasonDto {
    ReasonDto {
        kind,
        fact: None,
        scope: None,
        held_by: None,
        with: None,
        lane: None,
        constraint_id: None,
        reason: None,
        target: None,
        capacity: None,
        active: None,
        lane_tier: None,
        op_tier: None,
    }
}

impl ReasonDto {
    fn fact(mut self, value: &FactKind) -> Self {
        self.fact = Some(value.0.clone());
        self
    }

    fn scope(mut self, value: &Scope) -> Self {
        self.scope = Some(value.0.clone());
        self
    }

    fn held_by(mut self, value: &OpId) -> Self {
        self.held_by = Some(op_ref(value));
        self
    }

    fn with(mut self, value: &OpId) -> Self {
        self.with = Some(op_ref(value));
        self
    }

    fn lane(mut self, value: &LaneId) -> Self {
        self.lane = Some(value.0.clone());
        self
    }

    fn constraint_id(mut self, value: &str) -> Self {
        self.constraint_id = Some(value.to_owned());
        self
    }

    fn reason(mut self, value: &str) -> Self {
        self.reason = Some(value.to_owned());
        self
    }

    fn target(mut self, value: &DrainTarget) -> Self {
        self.target = Some(match value {
            DrainTarget::Global => "global".to_owned(),
            DrainTarget::Domain(scope) => format!("domain:{}", scope.0),
        });
        self
    }

    fn capacity(mut self, value: u32) -> Self {
        self.capacity = Some(value);
        self
    }

    fn active(mut self, value: u32) -> Self {
        self.active = Some(value);
        self
    }

    fn lane_tier(mut self, value: &str) -> Self {
        self.lane_tier = Some(value.to_owned());
        self
    }

    fn op_tier(mut self, value: &str) -> Self {
        self.op_tier = Some(value.to_owned());
        self
    }
}

pub(crate) fn format_reason(reason: &ReasonDto) -> String {
    let mut parts = vec![reason.kind.to_owned()];
    push_field(&mut parts, "fact", reason.fact.as_deref());
    push_field(&mut parts, "scope", reason.scope.as_deref());
    push_field(&mut parts, "held_by", reason.held_by.as_deref());
    push_field(&mut parts, "with", reason.with.as_deref());
    push_field(&mut parts, "lane", reason.lane.as_deref());
    push_field(&mut parts, "constraint_id", reason.constraint_id.as_deref());
    push_field(&mut parts, "reason", reason.reason.as_deref());
    push_field(&mut parts, "target", reason.target.as_deref());
    push_field(&mut parts, "lane_tier", reason.lane_tier.as_deref());
    push_field(&mut parts, "op_tier", reason.op_tier.as_deref());
    if let Some(value) = reason.capacity {
        parts.push(format!("capacity={value}"));
    }
    if let Some(value) = reason.active {
        parts.push(format!("active={value}"));
    }
    parts.join(" ")
}

fn push_field(parts: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        parts.push(format!("{key}={value}"));
    }
}

pub(crate) fn to_json(value: &impl Serialize) -> Result<String, CliError> {
    serde_json::to_string_pretty(value)
        .map_err(|err| input_error(format!("failed to serialize graph output: {err}")))
}

pub(crate) fn op_ref_dto(op_id: &OpId) -> OpRefDto {
    OpRefDto {
        action_id: op_id.0 .0.clone(),
        op_index: op_id.1,
        display: op_ref(op_id),
    }
}

pub(crate) fn op_ref(op_id: &OpId) -> String {
    format!("{}:{}", op_id.0 .0, op_id.1)
}

pub(crate) fn capacity_label(capacity: LaneCapacity) -> String {
    match capacity {
        LaneCapacity::Unbounded => "unbounded".to_owned(),
        LaneCapacity::Bounded(value) => value.to_string(),
    }
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn input_error(message: String) -> CliError {
    CliError::Usage(message)
}

#[cfg(test)]
mod tests {
    use causlane_core::LaneCapacity;

    use super::{graph_from_str, parse_op_ref};
    use crate::CliError;

    #[test]
    fn graph_parser_accepts_json_metadata_and_rejects_bad_shapes() -> Result<(), CliError> {
        let runtime = graph_from_str(
            "json",
            r#"{"produced_facts":["f1"],"active_ops":[],"lanes":[{"lane_id":"main","capacity":1}],"ops":[{"action_id":"act","op_index":0,"lane":"main","requires":["f1"],"writes":["s1"],"witnesses":["w2","w1","w1"],"leases":["l1"]}]}"#,
        )?;
        let op_id = parse_op_ref("act:0")?;
        assert!(runtime.index.node(&op_id).is_some());
        assert_eq!(runtime.meta[&op_id].witnesses, vec!["w1", "w2"]);
        assert_eq!(runtime.meta[&op_id].leases, vec!["l1"]);

        assert!(graph_from_str(
            "bad-capacity",
            r#"{ "lanes": [{"lane_id":"main","capacity":"many"}], "ops": [] }"#
        )
        .is_err());
        assert!(parse_op_ref("act").is_err());
        assert!(parse_op_ref(":0").is_err());
        assert!(parse_op_ref("act:x").is_err());
        Ok(())
    }

    #[test]
    fn graph_parser_rejects_duplicate_lanes_and_unknown_active_ops() {
        assert!(graph_from_str(
            "duplicate-lane",
            r"
lanes:
  - lane_id: main
    capacity: 1
  - lane_id: main
    capacity: 2
ops: []
"
        )
        .is_err());
        assert!(graph_from_str(
            "unknown-active",
            r"
active_ops: [ghost:0]
ops: []
"
        )
        .is_err());
    }

    #[test]
    fn graph_parser_preserves_capacity_values() -> Result<(), CliError> {
        let runtime = graph_from_str(
            "yaml",
            r"
lanes:
  - lane_id: main
    capacity: unbounded
  - lane_id: batch
    capacity: 2
ops: []
",
        )?;
        assert_eq!(
            runtime.lanes.get(&causlane_core::LaneId("main".to_owned())),
            Some(&LaneCapacity::Unbounded)
        );
        assert_eq!(
            runtime
                .lanes
                .get(&causlane_core::LaneId("batch".to_owned())),
            Some(&LaneCapacity::Bounded(2))
        );
        Ok(())
    }
}
