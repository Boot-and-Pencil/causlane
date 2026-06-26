//! M07.1 `explain` / `why-*` command handlers.
//!
//! This module is an I/O and rendering adapter. It loads a small typed CLI graph
//! snapshot, delegates readiness/frontier causes to `causlane-core`, and renders
//! those typed answers without re-deriving scheduler decisions.

use causlane_core::{pair_conflict, select_frontier, PairConflict};
use serde::Serialize;

use crate::cli_graph::{
    ensure_node, explain_op_from_index, format_reason, frontier_block_for, op_ref_dto,
    parse_op_ref, read_graph, reason_dtos, to_json, write_set, GraphRuntime, OpRefDto, ReasonDto,
};
use crate::{CliError, RunOutput};

#[derive(Serialize)]
struct WhyBlockedDto {
    command: &'static str,
    op: OpRefDto,
    status: &'static str,
    ready: bool,
    blocked: bool,
    reasons: Vec<ReasonDto>,
}

#[derive(Serialize)]
struct PairConflictDto {
    kind: &'static str,
    scope: String,
}

#[derive(Serialize)]
struct WhyNotParallelDto {
    command: &'static str,
    left: OpRefDto,
    right: OpRefDto,
    can_run_parallel: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pair_conflict: Option<PairConflictDto>,
    left_reasons: Vec<ReasonDto>,
    right_reasons: Vec<ReasonDto>,
}

pub(crate) fn why_blocked(graph_path: &str, op: &str, json: bool) -> Result<RunOutput, CliError> {
    let runtime = read_graph(graph_path)?;
    why_blocked_from_runtime(&runtime, op, json)
}

fn why_blocked_from_runtime(
    runtime: &GraphRuntime,
    op: &str,
    json: bool,
) -> Result<RunOutput, CliError> {
    let op_id = parse_op_ref(op)?;
    ensure_node(runtime, &op_id)?;
    let selection = select_frontier(&runtime.index, &runtime.lanes);
    let frontier = frontier_block_for(&selection.rejected, &op_id);
    let answer = explain_op_from_index(&runtime.index, &op_id, frontier.as_ref())?;
    let is_active = runtime.active_ops.contains(&op_id);
    let dto = WhyBlockedDto {
        command: "why-blocked",
        op: op_ref_dto(&op_id),
        status: if is_active {
            "active"
        } else if answer.is_parallelizable() {
            "ready"
        } else {
            "blocked"
        },
        ready: !is_active && answer.is_parallelizable(),
        blocked: !answer.is_parallelizable(),
        reasons: reason_dtos(&answer.reasons),
    };
    render_why_blocked(&dto, json)
}

pub(crate) fn why_not_parallel(
    graph_path: &str,
    left: &str,
    right: &str,
    json: bool,
) -> Result<RunOutput, CliError> {
    let runtime = read_graph(graph_path)?;
    why_not_parallel_from_runtime(&runtime, left, right, json)
}

fn why_not_parallel_from_runtime(
    runtime: &GraphRuntime,
    left: &str,
    right: &str,
    json: bool,
) -> Result<RunOutput, CliError> {
    let left_id = parse_op_ref(left)?;
    let right_id = parse_op_ref(right)?;
    let left_node = ensure_node(runtime, &left_id)?;
    let right_node = ensure_node(runtime, &right_id)?;
    let selection = select_frontier(&runtime.index, &runtime.lanes);

    let left_frontier = frontier_block_for(&selection.rejected, &left_id);
    let right_frontier = frontier_block_for(&selection.rejected, &right_id);
    let left_answer = explain_op_from_index(&runtime.index, &left_id, left_frontier.as_ref())?;
    let right_answer = explain_op_from_index(&runtime.index, &right_id, right_frontier.as_ref())?;
    let pair = pair_conflict(&write_set(left_node), &write_set(right_node));
    let can_run_parallel =
        pair.is_none() && left_answer.is_parallelizable() && right_answer.is_parallelizable();
    let dto = WhyNotParallelDto {
        command: "why-not-parallel",
        left: op_ref_dto(&left_id),
        right: op_ref_dto(&right_id),
        can_run_parallel,
        pair_conflict: pair.map(pair_conflict_dto),
        left_reasons: reason_dtos(&left_answer.reasons),
        right_reasons: reason_dtos(&right_answer.reasons),
    };
    render_why_not_parallel(&dto, json)
}

fn pair_conflict_dto(conflict: PairConflict) -> PairConflictDto {
    match conflict {
        PairConflict::WriteScopeConflict { scope } => PairConflictDto {
            kind: "write_scope_conflict",
            scope: scope.0,
        },
    }
}

fn render_why_blocked(dto: &WhyBlockedDto, json: bool) -> Result<RunOutput, CliError> {
    let text = if json {
        to_json(dto)?
    } else if dto.reasons.is_empty() {
        format!("{}: {} is {}", dto.status, dto.op.display, dto.status)
    } else {
        format!(
            "blocked: {}\n{}",
            dto.op.display,
            format_reasons(&dto.reasons)
        )
    };
    Ok(RunOutput {
        text,
        success: true,
    })
}

fn render_why_not_parallel(dto: &WhyNotParallelDto, json: bool) -> Result<RunOutput, CliError> {
    let text = if json {
        to_json(dto)?
    } else if dto.can_run_parallel {
        format!(
            "ok: {} and {} can run in parallel",
            dto.left.display, dto.right.display
        )
    } else {
        let mut lines = vec![format!(
            "not parallel: {} and {}",
            dto.left.display, dto.right.display
        )];
        if let Some(conflict) = &dto.pair_conflict {
            lines.push(format!("  - {} scope={}", conflict.kind, conflict.scope));
        }
        append_reasons(&mut lines, "left", &dto.left_reasons);
        append_reasons(&mut lines, "right", &dto.right_reasons);
        lines.join("\n")
    };
    Ok(RunOutput {
        text,
        success: true,
    })
}

fn append_reasons(lines: &mut Vec<String>, side: &str, reasons: &[ReasonDto]) {
    for line in reason_lines(reasons) {
        lines.push(format!("  - {side}: {line}"));
    }
}

fn format_reasons(reasons: &[ReasonDto]) -> String {
    reason_lines(reasons)
        .into_iter()
        .map(|line| format!("  - {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn reason_lines(reasons: &[ReasonDto]) -> Vec<String> {
    reasons.iter().map(format_reason).collect()
}

#[cfg(test)]
mod tests {
    use crate::cli_graph::{dto, graph_from_str, op_ref_dto, parse_op_ref, reason_dto};
    use crate::CliError;
    use causlane_core::{
        ConstraintBlocker, ConstraintId, ConstraintViolation, DrainTarget, FrontierBlock, LaneId,
        LaneRejection, NotParallelReason, Scope, Tier,
    };

    use super::{
        render_why_blocked, render_why_not_parallel, why_blocked_from_runtime,
        why_not_parallel_from_runtime, PairConflictDto, WhyBlockedDto, WhyNotParallelDto,
    };

    fn graph_text() -> &'static str {
        r"
produced_facts: []
active_ops: []
lanes:
  - lane_id: main
    capacity: unbounded
ops:
  - action_id: act_a
    op_index: 0
    lane: main
    requires: []
    writes: [scope:one]
  - action_id: act_b
    op_index: 0
    lane: main
    requires: [fact:ready]
    writes: [scope:two]
  - action_id: act_c
    op_index: 0
    lane: main
    requires: []
    writes: [scope:one]
  - action_id: act_d
    op_index: 0
    lane: main
    requires: []
    writes: [scope:three]
"
    }

    #[test]
    fn why_blocked_reports_fact_and_frontier_causes() -> Result<(), CliError> {
        let runtime = graph_from_str("test", graph_text())?;
        let blocked = why_blocked_from_runtime(&runtime, "act_b:0", true)?;
        assert!(blocked.text.contains("\"status\": \"blocked\""));
        assert!(blocked.text.contains("\"kind\": \"blocked_on_fact\""));

        let frontier = why_blocked_from_runtime(&runtime, "act_c:0", true)?;
        assert!(frontier
            .text
            .contains("\"kind\": \"frontier_write_scope_conflict\""));
        Ok(())
    }

    #[test]
    fn why_not_parallel_reports_pair_conflict_and_disjoint_success() -> Result<(), CliError> {
        let runtime = graph_from_str("test", graph_text())?;
        let conflict = why_not_parallel_from_runtime(&runtime, "act_a:0", "act_c:0", true)?;
        assert!(conflict.text.contains("\"can_run_parallel\": false"));
        assert!(conflict.text.contains("\"kind\": \"write_scope_conflict\""));

        let disjoint = why_not_parallel_from_runtime(&runtime, "act_a:0", "act_d:0", true)?;
        assert!(disjoint.text.contains("\"can_run_parallel\": true"));
        Ok(())
    }

    #[test]
    fn reason_mapping_covers_non_graph_variants() {
        let wait = reason_dto(&NotParallelReason::ConstraintWait(ConstraintBlocker {
            constraint_id: ConstraintId("c1".to_owned()),
            reason: "busy".to_owned(),
        }));
        assert_eq!(wait.kind, "constraint_wait");
        assert_eq!(wait.constraint_id.as_deref(), Some("c1"));
        assert_eq!(wait.reason.as_deref(), Some("busy"));

        let deny = reason_dto(&NotParallelReason::ConstraintDeny(ConstraintViolation {
            constraint_id: ConstraintId("c2".to_owned()),
            reason: "frozen".to_owned(),
        }));
        assert_eq!(deny.kind, "constraint_deny");
        assert_eq!(deny.constraint_id.as_deref(), Some("c2"));

        let drain = reason_dto(&NotParallelReason::DrainRegion {
            target: DrainTarget::Domain(Scope("env".to_owned())),
            scope: Scope("env:staging".to_owned()),
        });
        assert_eq!(drain.kind, "drain_region");
        assert_eq!(drain.target.as_deref(), Some("domain:env"));

        let lane_full = reason_dto(&NotParallelReason::LaneRejected(
            LaneRejection::CapacityExhausted {
                capacity: 2,
                active: 2,
            },
        ));
        assert_eq!(lane_full.capacity, Some(2));
        assert_eq!(lane_full.active, Some(2));

        let wrong_tier = reason_dto(&NotParallelReason::LaneRejected(LaneRejection::WrongTier {
            lane_tier: Tier::Execution,
            op_tier: Tier::Planning,
        }));
        assert_eq!(wrong_tier.kind, "lane_wrong_tier");
        assert_eq!(wrong_tier.lane_tier.as_deref(), Some("Execution"));

        let frontier = reason_dto(&NotParallelReason::Frontier(
            FrontierBlock::LaneAtCapacity {
                lane: LaneId("main".to_owned()),
            },
        ));
        assert_eq!(frontier.kind, "lane_at_capacity");
        assert_eq!(frontier.lane.as_deref(), Some("main"));
    }

    #[test]
    fn human_renderers_cover_ready_blocked_and_pair_outputs() -> Result<(), CliError> {
        let op = op_ref_dto(&parse_op_ref("act:0")?);
        let blocked = WhyBlockedDto {
            command: "why-blocked",
            op,
            status: "blocked",
            ready: false,
            blocked: true,
            reasons: vec![dto("blocked_on_fact")],
        };
        assert!(render_why_blocked(&blocked, false)?
            .text
            .contains("blocked: act:0"));

        let ready = WhyBlockedDto {
            command: "why-blocked",
            op: op_ref_dto(&parse_op_ref("ready:0")?),
            status: "ready",
            ready: true,
            blocked: false,
            reasons: Vec::new(),
        };
        assert!(render_why_blocked(&ready, false)?
            .text
            .contains("ready: ready:0 is ready"));

        let pair = WhyNotParallelDto {
            command: "why-not-parallel",
            left: op_ref_dto(&parse_op_ref("left:0")?),
            right: op_ref_dto(&parse_op_ref("right:0")?),
            can_run_parallel: false,
            pair_conflict: Some(PairConflictDto {
                kind: "write_scope_conflict",
                scope: "scope:one".to_owned(),
            }),
            left_reasons: vec![dto("lane_at_capacity")],
            right_reasons: Vec::new(),
        };
        assert!(render_why_not_parallel(&pair, false)?
            .text
            .contains("not parallel: left:0 and right:0"));

        let ok = WhyNotParallelDto {
            can_run_parallel: true,
            pair_conflict: None,
            left_reasons: Vec::new(),
            right_reasons: Vec::new(),
            ..pair
        };
        assert!(render_why_not_parallel(&ok, false)?
            .text
            .contains("can run in parallel"));
        Ok(())
    }
}
