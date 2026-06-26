//! M07.2 graph export command handler.
//!
//! Builds one deterministic export model from the shared CLI graph snapshot,
//! then renders JSON, Mermaid or DOT. Scheduling facts come from `GraphIndex`,
//! `select_frontier` and `why_not_parallel_from_index`; this module only adapts
//! those answers for operator-facing graph output.

use std::collections::BTreeSet;

use causlane_core::{
    select_frontier, FactKind, FrontierBlock, LaneCapacity, LaneId, NotParallelReason, OpId, Scope,
};
use serde::Serialize;

use crate::cli_graph::{
    capacity_label, ensure_node, explain_op_from_index, frontier_block_for, op_ref, op_ref_dto,
    parse_op_ref, read_graph, reason_dtos, to_json, GraphRuntime, OpRefDto, ReasonDto,
};
use crate::{write_file, CliError, RunOutput};

const GRAPH_EXPORT_SCHEMA_VERSION: u32 = 1;
const FORMAT_JSON: &str = "json";
const FORMAT_MERMAID: &str = "mermaid";
const FORMAT_DOT: &str = "dot";

type NodeIds = Vec<(String, String)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExportFormat {
    Json,
    Mermaid,
    Dot,
}

#[derive(Serialize)]
pub(crate) struct GraphExportDto {
    schema_version: u32,
    command: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    focus: Option<OpRefDto>,
    lanes: Vec<LaneDto>,
    ops: Vec<GraphOpDto>,
    edges: Vec<GraphEdgeDto>,
}

#[derive(Serialize)]
struct LaneDto {
    lane_id: String,
    capacity: String,
}

#[derive(Serialize)]
struct GraphOpDto {
    op: OpRefDto,
    lane: String,
    status: &'static str,
    active: bool,
    ready: bool,
    requires: Vec<String>,
    writes: Vec<String>,
    blockers: Vec<ReasonDto>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    witnesses: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    leases: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct GraphEdgeDto {
    kind: &'static str,
    from: String,
    to: String,
    label: String,
}

pub(crate) fn export_graph(
    graph_path: &str,
    format: &str,
    focus: Option<&str>,
    out: Option<&str>,
) -> Result<RunOutput, CliError> {
    let runtime = read_graph(graph_path)?;
    let format = parse_format(format)?;
    let export = export_graph_from_runtime(&runtime, focus)?;
    let text = render_export(&export, format)?;
    if let Some(path) = out {
        write_file(path, &text)?;
        return Ok(RunOutput {
            text: format!("ok: wrote {path} ({})", format_token(format)),
            success: true,
        });
    }
    Ok(RunOutput {
        text,
        success: true,
    })
}

pub(crate) fn export_graph_model(
    graph_path: &str,
    focus: Option<&str>,
) -> Result<GraphExportDto, CliError> {
    let runtime = read_graph(graph_path)?;
    export_graph_from_runtime(&runtime, focus)
}

#[cfg(test)]
pub(crate) fn export_graph_model_from_text(
    label: &str,
    text: &str,
    focus: Option<&str>,
) -> Result<GraphExportDto, CliError> {
    let runtime = crate::cli_graph::graph_from_str(label, text)?;
    export_graph_from_runtime(&runtime, focus)
}

fn export_graph_from_runtime(
    runtime: &GraphRuntime,
    focus: Option<&str>,
) -> Result<GraphExportDto, CliError> {
    let selection = select_frontier(&runtime.index, &runtime.lanes);
    let focus_id = focus.map(parse_op_ref).transpose()?;
    if let Some(op_id) = &focus_id {
        ensure_node(runtime, op_id)?;
    }

    let mut included = BTreeSet::new();
    if let Some(op_id) = &focus_id {
        let _new = included.insert(op_id.clone());
        let frontier = frontier_block_for(&selection.rejected, op_id);
        let answer = explain_op_from_index(&runtime.index, op_id, frontier.as_ref())?;
        add_blocker_ops(runtime, &mut included, &answer.reasons);
    } else {
        included.extend(runtime.index.nodes().iter().map(|node| node.op_id.clone()));
    }

    let mut ops = Vec::new();
    let mut edges = Vec::new();
    let mut lane_ids = BTreeSet::new();
    for op_id in &included {
        let node = ensure_node(runtime, op_id)?;
        let _new = lane_ids.insert(node.lane.clone());
        let frontier = frontier_block_for(&selection.rejected, op_id);
        let answer = explain_op_from_index(&runtime.index, op_id, frontier.as_ref())?;
        let active = runtime.active_ops.contains(op_id);
        let ready = !active && answer.is_parallelizable();
        let status = if active {
            "active"
        } else if ready {
            "ready"
        } else {
            "blocked"
        };
        let meta = runtime.meta.get(op_id).cloned().unwrap_or_default();
        edges.extend(base_edges(op_id, &node.requires, &node.writes));
        edges.extend(blocker_edges(op_id, &answer.reasons));
        edges.extend(metadata_edges(op_id, &meta.witnesses, &meta.leases));
        ops.push(GraphOpDto {
            op: op_ref_dto(op_id),
            lane: node.lane.0.clone(),
            status,
            active,
            ready,
            requires: node.requires.iter().map(|fact| fact.0.clone()).collect(),
            writes: node.writes.iter().map(|scope| scope.0.clone()).collect(),
            blockers: reason_dtos(&answer.reasons),
            witnesses: meta.witnesses,
            leases: meta.leases,
        });
    }

    if focus_id.is_none() {
        lane_ids.extend(runtime.lanes.keys().cloned());
    }
    let lanes = lane_ids
        .into_iter()
        .map(|lane| lane_dto(runtime, lane))
        .collect();

    Ok(GraphExportDto {
        schema_version: GRAPH_EXPORT_SCHEMA_VERSION,
        command: "graph export",
        focus: focus_id.as_ref().map(op_ref_dto),
        lanes,
        ops,
        edges,
    })
}

fn lane_dto(runtime: &GraphRuntime, lane: LaneId) -> LaneDto {
    let capacity = runtime
        .lanes
        .get(&lane)
        .copied()
        .unwrap_or(LaneCapacity::Unbounded);
    LaneDto {
        lane_id: lane.0,
        capacity: capacity_label(capacity),
    }
}

fn add_blocker_ops(
    runtime: &GraphRuntime,
    included: &mut BTreeSet<OpId>,
    reasons: &[NotParallelReason],
) {
    for reason in reasons {
        match reason {
            NotParallelReason::BlockedOnActiveScope { held_by, .. } => {
                if runtime.index.node(held_by).is_some() {
                    let _new = included.insert(held_by.clone());
                }
            }
            NotParallelReason::Frontier(FrontierBlock::WriteScopeConflict { with, .. }) => {
                if runtime.index.node(with).is_some() {
                    let _new = included.insert(with.clone());
                }
            }
            NotParallelReason::BlockedOnFact { .. }
            | NotParallelReason::Frontier(FrontierBlock::LaneAtCapacity { .. })
            | NotParallelReason::ConstraintWait(_)
            | NotParallelReason::ConstraintDeny(_)
            | NotParallelReason::LaneRejected(_)
            | NotParallelReason::DrainRegion { .. } => {}
        }
    }
}

fn base_edges(op_id: &OpId, requires: &[FactKind], writes: &[Scope]) -> Vec<GraphEdgeDto> {
    let mut edges = Vec::new();
    for fact in requires {
        edges.push(op_edge(
            "requires_fact",
            &format!("fact:{}", fact.0),
            op_id,
            "requires",
        ));
    }
    for scope in writes {
        edges.push(raw_edge(
            "writes_scope",
            &op_endpoint(op_id),
            &format!("scope:{}", scope.0),
            "writes",
        ));
    }
    edges
}

fn blocker_edges(op_id: &OpId, reasons: &[NotParallelReason]) -> Vec<GraphEdgeDto> {
    let mut edges = Vec::new();
    for reason in reasons {
        match reason {
            NotParallelReason::BlockedOnFact { fact } => edges.push(op_edge(
                "blocked_on_fact",
                &format!("fact:{}", fact.0),
                op_id,
                "missing fact",
            )),
            NotParallelReason::BlockedOnActiveScope { scope, held_by } => edges.push(op_edge(
                "active_scope_conflict",
                &op_endpoint(held_by),
                op_id,
                &format!("active writer {}", scope.0),
            )),
            NotParallelReason::Frontier(FrontierBlock::WriteScopeConflict { scope, with }) => {
                edges.push(op_edge(
                    "frontier_write_scope_conflict",
                    &op_endpoint(with),
                    op_id,
                    &format!("pending writer {}", scope.0),
                ));
            }
            NotParallelReason::Frontier(FrontierBlock::LaneAtCapacity { lane }) => {
                edges.push(op_edge(
                    "lane_at_capacity",
                    &format!("lane:{}", lane.0),
                    op_id,
                    "capacity",
                ));
            }
            NotParallelReason::ConstraintWait(blocker) => edges.push(op_edge(
                "constraint_wait",
                &format!("constraint:{}", blocker.constraint_id.0),
                op_id,
                &blocker.reason,
            )),
            NotParallelReason::ConstraintDeny(violation) => edges.push(op_edge(
                "constraint_deny",
                &format!("constraint:{}", violation.constraint_id.0),
                op_id,
                &violation.reason,
            )),
            NotParallelReason::LaneRejected(_) => {
                edges.push(op_edge(
                    "lane_rejected",
                    "lane:rejected",
                    op_id,
                    "lane gate",
                ));
            }
            NotParallelReason::DrainRegion { target, scope } => edges.push(op_edge(
                "drain_region",
                &format!("drain:{target:?}"),
                op_id,
                &scope.0,
            )),
        }
    }
    edges
}

fn metadata_edges(op_id: &OpId, witnesses: &[String], leases: &[String]) -> Vec<GraphEdgeDto> {
    let mut edges = Vec::new();
    for witness in witnesses {
        edges.push(op_edge(
            "witness",
            &format!("witness:{witness}"),
            op_id,
            "witness",
        ));
    }
    for lease in leases {
        edges.push(op_edge("lease", &format!("lease:{lease}"), op_id, "lease"));
    }
    edges
}

fn op_edge(kind: &'static str, from: &str, to: &OpId, label: &str) -> GraphEdgeDto {
    raw_edge(kind, from, &op_endpoint(to), label)
}

fn raw_edge(kind: &'static str, from: &str, to: &str, label: &str) -> GraphEdgeDto {
    GraphEdgeDto {
        kind,
        from: from.to_owned(),
        to: to.to_owned(),
        label: label.to_owned(),
    }
}

fn op_endpoint(op_id: &OpId) -> String {
    format!("op:{}", op_ref(op_id))
}

fn parse_format(value: &str) -> Result<ExportFormat, CliError> {
    if value == FORMAT_JSON {
        Ok(ExportFormat::Json)
    } else if value == FORMAT_MERMAID {
        Ok(ExportFormat::Mermaid)
    } else if value == FORMAT_DOT {
        Ok(ExportFormat::Dot)
    } else {
        Err(CliError::Usage(format!(
            "unsupported graph export format {value:?}; use json, mermaid or dot"
        )))
    }
}

fn render_export(export: &GraphExportDto, format: ExportFormat) -> Result<String, CliError> {
    match format {
        ExportFormat::Json => to_json(export),
        ExportFormat::Mermaid => Ok(render_mermaid(export)),
        ExportFormat::Dot => Ok(render_dot(export)),
    }
}

fn render_mermaid(export: &GraphExportDto) -> String {
    let mut lines = vec!["flowchart TD".to_owned()];
    let op_ids = op_node_ids(export);
    let synthetic_ids = synthetic_node_ids(export, &op_ids);
    for (index, op) in export.ops.iter().enumerate() {
        lines.push(format!(
            "  op{index}[\"{}\"]",
            mermaid_label(&node_label(op))
        ));
    }
    for (endpoint, id) in &synthetic_ids {
        lines.push(format!("  {id}[\"{}\"]", mermaid_label(endpoint)));
    }
    for edge in &export.edges {
        let from = endpoint_id(&edge.from, &op_ids, &synthetic_ids);
        let to = endpoint_id(&edge.to, &op_ids, &synthetic_ids);
        lines.push(format!(
            "  {from} -->|{}| {to}",
            mermaid_edge_label(&format!("{} {}", edge.kind, edge.label))
        ));
    }
    lines.join("\n")
}

fn render_dot(export: &GraphExportDto) -> String {
    let mut lines = vec![
        "digraph causlane_graph {".to_owned(),
        "  rankdir=LR;".to_owned(),
    ];
    let mut declared = BTreeSet::new();
    for op in &export.ops {
        let endpoint = format!("op:{}", op.op.display);
        let _new = declared.insert(endpoint.clone());
        lines.push(format!(
            "  \"{}\" [shape=box,label=\"{}\"];",
            dot_escape(&endpoint),
            dot_escape(&node_label(op))
        ));
    }
    for edge in &export.edges {
        for endpoint in [&edge.from, &edge.to] {
            if declared.insert(endpoint.clone()) {
                lines.push(format!(
                    "  \"{}\" [shape=ellipse,label=\"{}\"];",
                    dot_escape(endpoint),
                    dot_escape(endpoint)
                ));
            }
        }
        lines.push(format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];",
            dot_escape(&edge.from),
            dot_escape(&edge.to),
            dot_escape(&format!("{} {}", edge.kind, edge.label))
        ));
    }
    lines.push("}".to_owned());
    lines.join("\n")
}

fn op_node_ids(export: &GraphExportDto) -> NodeIds {
    export
        .ops
        .iter()
        .enumerate()
        .map(|(index, op)| (format!("op:{}", op.op.display), format!("op{index}")))
        .collect()
}

fn synthetic_node_ids(export: &GraphExportDto, op_ids: &NodeIds) -> NodeIds {
    let mut endpoints = BTreeSet::new();
    for edge in &export.edges {
        if lookup_node_id(&edge.from, op_ids).is_none() {
            let _new = endpoints.insert(edge.from.clone());
        }
        if lookup_node_id(&edge.to, op_ids).is_none() {
            let _new = endpoints.insert(edge.to.clone());
        }
    }
    endpoints
        .into_iter()
        .enumerate()
        .map(|(index, endpoint)| (endpoint, format!("n{index}")))
        .collect()
}

fn endpoint_id(endpoint: &str, op_ids: &NodeIds, synthetic_ids: &NodeIds) -> String {
    lookup_node_id(endpoint, op_ids)
        .or_else(|| lookup_node_id(endpoint, synthetic_ids))
        .unwrap_or_else(|| "missing".to_owned())
}

fn lookup_node_id(endpoint: &str, ids: &NodeIds) -> Option<String> {
    ids.iter()
        .find(|(candidate, _id)| candidate == endpoint)
        .map(|(_candidate, id)| id.clone())
}

fn node_label(op: &GraphOpDto) -> String {
    let mut lines = vec![
        op.op.display.clone(),
        format!("lane={}", op.lane),
        format!("status={}", op.status),
    ];
    if !op.blockers.is_empty() {
        lines.push(format!(
            "blockers={}",
            op.blockers
                .iter()
                .map(|reason| reason.kind)
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    if !op.witnesses.is_empty() {
        lines.push(format!("witnesses={}", op.witnesses.join(",")));
    }
    if !op.leases.is_empty() {
        lines.push(format!("leases={}", op.leases.join(",")));
    }
    lines.join("\n")
}

fn mermaid_label(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\n', "<br/>")
}

fn mermaid_edge_label(value: &str) -> String {
    value.replace('|', "/")
}

fn dot_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn format_token(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Json => FORMAT_JSON,
        ExportFormat::Mermaid => FORMAT_MERMAID,
        ExportFormat::Dot => FORMAT_DOT,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        export_graph_from_runtime, parse_format, render_dot, render_mermaid, ExportFormat,
        FORMAT_JSON,
    };
    use crate::cli_graph::graph_from_str;
    use crate::CliError;

    fn graph_text() -> &'static str {
        r"
produced_facts: []
active_ops: [holder:0]
lanes:
  - lane_id: main
    capacity: unbounded
ops:
  - action_id: holder
    op_index: 0
    lane: main
    requires: []
    writes: [scope:one]
    witnesses: [w2, w1, w1]
    leases: [lease-a]
  - action_id: blocked
    op_index: 0
    lane: main
    requires: [fact:ready]
    writes: [scope:one]
  - action_id: ready
    op_index: 0
    lane: main
    requires: []
    writes: [scope:two]
  - action_id: conflict
    op_index: 0
    lane: main
    requires: []
    writes: [scope:two]
"
    }

    #[test]
    fn json_model_reports_statuses_blockers_and_metadata() -> Result<(), CliError> {
        let runtime = graph_from_str("test", graph_text())?;
        let export = export_graph_from_runtime(&runtime, None)?;
        assert_eq!(export.schema_version, 1);
        assert_eq!(export.ops.len(), 4);
        let holder = export
            .ops
            .iter()
            .find(|op| op.op.display == "holder:0")
            .ok_or_else(|| CliError::Usage("holder op not exported".to_owned()))?;
        assert_eq!(holder.status, "active");
        assert_eq!(holder.witnesses, vec!["w1", "w2"]);
        assert_eq!(holder.leases, vec!["lease-a"]);
        let blocked = export
            .ops
            .iter()
            .find(|op| op.op.display == "blocked:0")
            .ok_or_else(|| CliError::Usage("blocked op not exported".to_owned()))?;
        assert!(blocked
            .blockers
            .iter()
            .any(|reason| reason.kind == "blocked_on_fact"));
        assert!(blocked
            .blockers
            .iter()
            .any(|reason| reason.kind == "active_scope_conflict"));
        assert!(export
            .edges
            .iter()
            .any(|edge| edge.kind == "frontier_write_scope_conflict"));
        Ok(())
    }

    #[test]
    fn focused_slice_keeps_target_and_direct_evidence() -> Result<(), CliError> {
        let runtime = graph_from_str("test", graph_text())?;
        let export = export_graph_from_runtime(&runtime, Some("blocked:0"))?;
        let ops: Vec<_> = export.ops.iter().map(|op| op.op.display.as_str()).collect();
        assert_eq!(ops, vec!["blocked:0", "holder:0"]);
        assert!(export
            .edges
            .iter()
            .any(|edge| edge.kind == "active_scope_conflict"));
        assert!(export
            .edges
            .iter()
            .any(|edge| edge.kind == "blocked_on_fact"));
        Ok(())
    }

    #[test]
    fn text_renderers_are_deterministic_and_escaped() -> Result<(), CliError> {
        let runtime = graph_from_str("test", graph_text())?;
        let export = export_graph_from_runtime(&runtime, Some("holder:0"))?;
        let mermaid = render_mermaid(&export);
        assert!(mermaid.starts_with("flowchart TD"));
        assert!(mermaid.contains("witnesses=w1,w2"));
        let dot = render_dot(&export);
        assert!(dot.starts_with("digraph causlane_graph"));
        assert!(dot.contains("lease-a"));
        assert_eq!(parse_format(FORMAT_JSON)?, ExportFormat::Json);
        assert!(parse_format("svg").is_err());
        Ok(())
    }
}
