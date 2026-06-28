#![forbid(unsafe_code)]
#![deny(warnings)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use causlane::core::kernel::{pair_conflict, why_not_parallel_from_index};
use causlane::core::protocol::{
    ActionId, FactKind, FrontierBlock, FrontierSelection, GraphIndex, GraphNode, LaneId,
    NotParallelReason, OpId, PairConflict, Scope, WhyNotParallel,
};
use causlane::prelude::select_frontier;

/// Summary returned by the runnable example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhyNotParallelSummary {
    /// Local cases checked by the example.
    pub checked_cases: usize,
    /// Cases with an explicit non-parallel blocker.
    pub blocked_cases: usize,
    /// Cases that demonstrate an empty why-not-parallel answer.
    pub parallelizable_cases: usize,
}

/// Pairwise pending-write conflict evidence from the example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairConflictExample {
    /// The head-to-head conflict between the two pending ops.
    pub conflict: PairConflict,
    /// The frontier-level reason for rejecting the deterministic loser.
    pub rejected_reason: NotParallelReason,
}

/// Dependency-blocker evidence before and after the required fact is produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyBlockerExample {
    /// The explanation while the op is still waiting on its fact.
    pub before: WhyNotParallel,
    /// The explanation after the fact has been produced.
    pub after: WhyNotParallel,
}

/// Active-writer blocker evidence for a conflicting write scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveWriterExample {
    /// The explanation for the blocked op.
    pub explanation: WhyNotParallel,
}

/// Error type for the why-not-parallel example composition.
#[derive(Debug)]
pub enum WhyNotParallelExampleError {
    /// The local selector or explanation returned a different outcome.
    UnexpectedOutcome(&'static str),
    /// The queried op was missing from the graph index.
    MissingExplanation(&'static str),
    /// A frontier rejection expected by the example was absent.
    MissingFrontierRejection(&'static str),
}

impl fmt::Display for WhyNotParallelExampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOutcome(label) => write!(f, "unexpected outcome: {label}"),
            Self::MissingExplanation(label) => write!(f, "missing explanation: {label}"),
            Self::MissingFrontierRejection(label) => {
                write!(f, "missing frontier rejection: {label}")
            }
        }
    }
}

impl std::error::Error for WhyNotParallelExampleError {}

/// Run the why-not-parallel example.
///
/// # Errors
/// Returns an error if any local graph/frontier/explanation case diverges from
/// the documented outcome.
#[must_use = "the runnable example result must be checked"]
pub fn run_why_not_parallel() -> Result<WhyNotParallelSummary, WhyNotParallelExampleError> {
    verify_disjoint_ready_ops_are_parallelizable()?;
    verify_pending_write_conflict()?;
    verify_dependency_blocker_transition()?;
    verify_active_writer_blocker()?;
    Ok(WhyNotParallelSummary {
        checked_cases: 4,
        blocked_cases: 3,
        parallelizable_cases: 2,
    })
}

/// Verify that two ready ops with disjoint writes are both parallelizable.
///
/// # Errors
/// Returns an error if the pair conflict, frontier or explanation outcome is
/// not the documented positive case.
#[must_use = "parallelism checks can fail and must be inspected"]
pub fn verify_disjoint_ready_ops_are_parallelizable() -> Result<(), WhyNotParallelExampleError> {
    let staging = op("act_promote_staging", 0);
    let prod = op("act_promote_prod", 0);
    let staging_writes = write_set(&["environment:staging"]);
    let prod_writes = write_set(&["environment:prod"]);
    if pair_conflict(&staging_writes, &prod_writes).is_some() {
        return Err(WhyNotParallelExampleError::UnexpectedOutcome(
            "disjoint writes reported a pair conflict",
        ));
    }

    let mut index = GraphIndex::new();
    index.add_node(node(
        staging.clone(),
        "release",
        &[],
        &["environment:staging"],
    ));
    index.add_node(node(prod.clone(), "release", &[], &["environment:prod"]));

    let selected = select_frontier(&index, &BTreeMap::new());
    require_selected(
        &selected,
        [staging.clone(), prod.clone()].into_iter().collect(),
        "disjoint ops selected together",
    )?;
    if !selected.rejected.is_empty() {
        return Err(WhyNotParallelExampleError::UnexpectedOutcome(
            "disjoint ops had frontier rejections",
        ));
    }
    require_parallelizable(&index, &staging, "staging op parallelizable")?;
    require_parallelizable(&index, &prod, "prod op parallelizable")
}

/// Verify that two pending ops writing the same scope have pair and frontier
/// explanations.
///
/// # Errors
/// Returns an error if the conflict scope or rejected reason differs from the
/// documented case.
#[must_use = "conflict checks can fail and must be inspected"]
pub fn verify_pending_write_conflict() -> Result<PairConflictExample, WhyNotParallelExampleError> {
    let first = op("act_promote_123", 0);
    let second = op("act_promote_456", 0);
    let scope = Scope("environment:staging".to_owned());
    let conflict = pair_conflict(
        &write_set(&["environment:staging"]),
        &write_set(&["environment:staging"]),
    )
    .ok_or(WhyNotParallelExampleError::UnexpectedOutcome(
        "same write scope produced no pair conflict",
    ))?;
    if conflict
        != (PairConflict::WriteScopeConflict {
            scope: scope.clone(),
        })
    {
        return Err(WhyNotParallelExampleError::UnexpectedOutcome(
            "same write scope reported the wrong pair conflict",
        ));
    }

    let mut index = GraphIndex::new();
    index.add_node(node(
        first.clone(),
        "release",
        &[],
        &["environment:staging"],
    ));
    index.add_node(node(
        second.clone(),
        "release",
        &[],
        &["environment:staging"],
    ));

    let selected = select_frontier(&index, &BTreeMap::new());
    require_selected(
        &selected,
        [first.clone()].into_iter().collect(),
        "same-scope case selected deterministic winner",
    )?;
    let frontier = frontier_block_for(&selected, &second, "same-scope rejected second op")?;
    let expected_frontier = FrontierBlock::WriteScopeConflict {
        scope: scope.clone(),
        with: first,
    };
    if frontier != expected_frontier {
        return Err(WhyNotParallelExampleError::UnexpectedOutcome(
            "same-scope case reported the wrong frontier block",
        ));
    }

    let explanation = explain_from_index(
        &index,
        &second,
        Some(&frontier),
        "same-scope rejected op explanation",
    )?;
    let rejected_reason = NotParallelReason::Frontier(expected_frontier);
    require_exact_reasons(
        &explanation,
        std::slice::from_ref(&rejected_reason),
        "same-scope rejected op reason",
    )?;
    Ok(PairConflictExample {
        conflict,
        rejected_reason,
    })
}

/// Verify that an unmet fact explains a dependency blocker and disappears after
/// the fact is produced.
///
/// # Errors
/// Returns an error if the blocked or unblocked explanation diverges.
#[must_use = "dependency checks can fail and must be inspected"]
pub fn verify_dependency_blocker_transition(
) -> Result<DependencyBlockerExample, WhyNotParallelExampleError> {
    let approval_fact = FactKind("approval:approved".to_owned());
    let op_id = op("act_release_metadata", 0);
    let mut index = GraphIndex::new();
    index.add_node(node(
        op_id.clone(),
        "release",
        &["approval:approved"],
        &["release:metadata"],
    ));

    let before = explain_from_index(&index, &op_id, None, "dependency blocker")?;
    require_exact_reasons(
        &before,
        &[NotParallelReason::BlockedOnFact {
            fact: approval_fact.clone(),
        }],
        "dependency blocker reason",
    )?;

    index.mark_produced(&approval_fact);
    let after = explain_from_index(&index, &op_id, None, "dependency unblock")?;
    if !after.is_parallelizable() {
        return Err(WhyNotParallelExampleError::UnexpectedOutcome(
            "produced fact did not unblock op",
        ));
    }
    let selected = select_frontier(&index, &BTreeMap::new());
    require_selected(
        &selected,
        [op_id].into_iter().collect(),
        "fact-produced op selected",
    )?;
    Ok(DependencyBlockerExample { before, after })
}

/// Verify that an active writer explains why a conflicting op cannot run now.
///
/// # Errors
/// Returns an error if the active writer reason is missing or incorrect.
#[must_use = "active-writer checks can fail and must be inspected"]
pub fn verify_active_writer_blocker() -> Result<ActiveWriterExample, WhyNotParallelExampleError> {
    let active = op("act_promote_active", 0);
    let blocked = op("act_promote_waiting", 0);
    let scope = Scope("environment:staging".to_owned());
    let mut index = GraphIndex::new();
    index.add_node(node(
        active.clone(),
        "release",
        &[],
        &["environment:staging"],
    ));
    index.add_node(node(
        blocked.clone(),
        "release",
        &[],
        &["environment:staging"],
    ));
    index.mark_active(&active);

    let explanation = explain_from_index(&index, &blocked, None, "active writer blocker")?;
    require_exact_reasons(
        &explanation,
        &[NotParallelReason::BlockedOnActiveScope {
            scope,
            held_by: active,
        }],
        "active writer blocker reason",
    )?;
    Ok(ActiveWriterExample { explanation })
}

fn explain_from_index(
    index: &GraphIndex,
    op_id: &OpId,
    frontier: Option<&FrontierBlock>,
    label: &'static str,
) -> Result<WhyNotParallel, WhyNotParallelExampleError> {
    why_not_parallel_from_index(index, op_id, frontier)
        .ok_or(WhyNotParallelExampleError::MissingExplanation(label))
}

fn require_parallelizable(
    index: &GraphIndex,
    op_id: &OpId,
    label: &'static str,
) -> Result<(), WhyNotParallelExampleError> {
    let explanation = explain_from_index(index, op_id, None, label)?;
    if explanation.is_parallelizable() {
        Ok(())
    } else {
        Err(WhyNotParallelExampleError::UnexpectedOutcome(label))
    }
}

fn require_exact_reasons(
    explanation: &WhyNotParallel,
    expected: &[NotParallelReason],
    label: &'static str,
) -> Result<(), WhyNotParallelExampleError> {
    if explanation.reasons == expected {
        Ok(())
    } else {
        Err(WhyNotParallelExampleError::UnexpectedOutcome(label))
    }
}

fn require_selected(
    selection: &FrontierSelection,
    expected: BTreeSet<OpId>,
    label: &'static str,
) -> Result<(), WhyNotParallelExampleError> {
    if selection.selected == expected {
        Ok(())
    } else {
        Err(WhyNotParallelExampleError::UnexpectedOutcome(label))
    }
}

fn frontier_block_for(
    selection: &FrontierSelection,
    op_id: &OpId,
    label: &'static str,
) -> Result<FrontierBlock, WhyNotParallelExampleError> {
    selection
        .rejected
        .iter()
        .find(|rejection| rejection.op_id == *op_id)
        .map(|rejection| rejection.reason.clone())
        .ok_or(WhyNotParallelExampleError::MissingFrontierRejection(label))
}

fn op(action: &str, index: u32) -> OpId {
    OpId(ActionId(action.to_owned()), index)
}

fn node(op_id: OpId, lane: &str, requires: &[&str], writes: &[&str]) -> GraphNode {
    GraphNode {
        op_id,
        lane: LaneId(lane.to_owned()),
        requires: requires
            .iter()
            .map(|fact| FactKind((*fact).to_owned()))
            .collect(),
        writes: writes
            .iter()
            .map(|scope| Scope((*scope).to_owned()))
            .collect(),
    }
}

fn write_set(scopes: &[&str]) -> BTreeSet<Scope> {
    scopes
        .iter()
        .map(|scope| Scope((*scope).to_owned()))
        .collect()
}
