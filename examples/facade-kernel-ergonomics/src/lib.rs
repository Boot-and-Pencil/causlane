#![forbid(unsafe_code)]
#![deny(warnings)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use causlane::core::{kernel, protocol};
use causlane::prelude::{
    ActionCall, ActionId, ConsequenceProfile, FactKind, GraphIndex, GraphNode, LaneCapacity,
    LaneId, OpId, PredicateId, Scope,
};

/// Summary returned by the facade-kernel ergonomics example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FacadeKernelErgonomicsSummary {
    /// Number of action calls accepted through the facade admission path.
    pub accepted_admissions: usize,
    /// Number of consequence profiles checked for barrier policy.
    pub barrier_profiles_checked: usize,
    /// Number of ops selected into the safe frontier.
    pub frontier_selected: usize,
    /// Number of ready ops rejected from the frontier with explicit reasons.
    pub frontier_rejections: usize,
}

/// Summary for the local frontier-selection case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FacadeFrontierSummary {
    /// Selected op ids from the facade-visible frontier selection.
    pub selected: BTreeSet<OpId>,
    /// Rejections caused by pending-vs-pending write-scope conflicts.
    pub write_scope_conflicts: usize,
    /// Rejections caused by lane capacity.
    pub lane_capacity_rejections: usize,
}

/// Error type for the facade-kernel ergonomics example.
#[derive(Debug, PartialEq, Eq)]
pub enum FacadeKernelErgonomicsError {
    /// Admission returned a non-accepted result.
    AdmissionRefused(String),
    /// Admission accepted a different action id than the submitted call.
    AdmissionActionMismatch,
    /// Barrier policy did not match the documented profile split.
    BarrierPolicyMismatch(&'static str),
    /// Frontier selection diverged from the expected deterministic outcome.
    FrontierOutcome(&'static str),
}

impl fmt::Display for FacadeKernelErgonomicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AdmissionRefused(reason) => write!(f, "facade admission refused: {reason}"),
            Self::AdmissionActionMismatch => {
                f.write_str("facade admission returned a mismatched action id")
            }
            Self::BarrierPolicyMismatch(label) => {
                write!(f, "unexpected barrier policy outcome: {label}")
            }
            Self::FrontierOutcome(label) => write!(f, "unexpected frontier outcome: {label}"),
        }
    }
}

impl std::error::Error for FacadeKernelErgonomicsError {}

/// Run the facade-kernel ergonomics example.
///
/// # Errors
/// Returns an error if the facade admission, barrier policy or frontier
/// selection behavior diverges from the documented kernel contract.
#[must_use = "the runnable example result must be checked"]
pub fn run_facade_kernel_ergonomics(
) -> Result<FacadeKernelErgonomicsSummary, FacadeKernelErgonomicsError> {
    verify_facade_admission()?;
    let barrier_profiles_checked = verify_facade_barrier_policy()?;
    let frontier = verify_facade_frontier_selection()?;

    Ok(FacadeKernelErgonomicsSummary {
        accepted_admissions: 1,
        barrier_profiles_checked,
        frontier_selected: frontier.selected.len(),
        frontier_rejections: frontier.write_scope_conflicts + frontier.lane_capacity_rejections,
    })
}

/// Verify that a downstream caller can submit an [`ActionCall`] through the facade.
///
/// # Errors
/// Returns an error if admission refuses the call or changes the action id.
#[must_use = "facade admission can fail and must be inspected"]
pub fn verify_facade_admission() -> Result<(), FacadeKernelErgonomicsError> {
    let call = action_call("act_facade_promote");
    match kernel::admit_call(&call) {
        kernel::DispatchAdmission::Accepted { action_id } if action_id == call.action_id => Ok(()),
        kernel::DispatchAdmission::Accepted { .. } => {
            Err(FacadeKernelErgonomicsError::AdmissionActionMismatch)
        }
        kernel::DispatchAdmission::Waiting { reason, .. }
        | kernel::DispatchAdmission::Rejected { reason, .. } => {
            Err(FacadeKernelErgonomicsError::AdmissionRefused(reason))
        }
    }
}

/// Verify the facade-visible barrier policy helpers for common profiles.
///
/// # Errors
/// Returns an error if runtime execution does not require a barrier or if a
/// projection read unexpectedly does require one.
#[must_use = "barrier policy can fail and must be inspected"]
pub fn verify_facade_barrier_policy() -> Result<usize, FacadeKernelErgonomicsError> {
    if !kernel::requires_execution_barrier(ConsequenceProfile::RuntimeExecution) {
        return Err(FacadeKernelErgonomicsError::BarrierPolicyMismatch(
            "runtime execution should require a barrier",
        ));
    }
    if kernel::requires_execution_barrier(ConsequenceProfile::ProjectionRead) {
        return Err(FacadeKernelErgonomicsError::BarrierPolicyMismatch(
            "projection read should not require a barrier",
        ));
    }
    Ok(2)
}

/// Verify safe frontier selection through the facade-visible graph types.
///
/// # Errors
/// Returns an error if the deterministic selection, write conflict rejection or
/// lane-capacity rejection differs from the expected result.
#[must_use = "frontier selection can fail and must be inspected"]
pub fn verify_facade_frontier_selection(
) -> Result<FacadeFrontierSummary, FacadeKernelErgonomicsError> {
    let deploy_a = op("act_deploy_a", 0);
    let deploy_b = op("act_deploy_b", 0);
    let metrics = op("act_metrics", 0);
    let active_limited = op("act_active_limited", 0);
    let waiting_limited = op("act_waiting_limited", 0);
    let mut index = GraphIndex::new();

    index.add_node(node(
        deploy_a.clone(),
        "release",
        &[],
        &["environment:prod"],
    ));
    index.add_node(node(
        deploy_b.clone(),
        "release",
        &[],
        &["environment:prod"],
    ));
    index.add_node(node(metrics.clone(), "metrics", &[], &["report:release"]));
    index.add_node(node(
        active_limited.clone(),
        "limited",
        &[],
        &["queue:active"],
    ));
    index.add_node(node(
        waiting_limited.clone(),
        "limited",
        &[],
        &["queue:waiting"],
    ));
    index.mark_active(&active_limited);

    let mut lanes = BTreeMap::new();
    let _previous = lanes.insert(LaneId("limited".to_owned()), LaneCapacity::Bounded(1));
    let selection = kernel::select_frontier(&index, &lanes);

    require_selected(&selection.selected, [&deploy_a, &metrics])?;
    require_not_selected(
        &selection.selected,
        &deploy_b,
        "conflicting deploy should wait",
    )?;
    require_not_selected(
        &selection.selected,
        &waiting_limited,
        "limited lane should be at capacity",
    )?;

    let write_scope_conflicts = selection
        .rejected
        .iter()
        .filter(|rejection| {
            rejection.op_id == deploy_b
                && matches!(
                    rejection.reason,
                    protocol::FrontierBlock::WriteScopeConflict { .. }
                )
        })
        .count();
    let lane_capacity_rejections = selection
        .rejected
        .iter()
        .filter(|rejection| {
            rejection.op_id == waiting_limited
                && matches!(
                    rejection.reason,
                    protocol::FrontierBlock::LaneAtCapacity { .. }
                )
        })
        .count();

    if write_scope_conflicts != 1 {
        return Err(FacadeKernelErgonomicsError::FrontierOutcome(
            "expected one write-scope conflict",
        ));
    }
    if lane_capacity_rejections != 1 {
        return Err(FacadeKernelErgonomicsError::FrontierOutcome(
            "expected one lane-capacity rejection",
        ));
    }

    Ok(FacadeFrontierSummary {
        selected: selection.selected,
        write_scope_conflicts,
        lane_capacity_rejections,
    })
}

fn require_selected<'a>(
    selected: &BTreeSet<OpId>,
    expected: impl IntoIterator<Item = &'a OpId>,
) -> Result<(), FacadeKernelErgonomicsError> {
    for op_id in expected {
        if !selected.contains(op_id) {
            return Err(FacadeKernelErgonomicsError::FrontierOutcome(
                "expected op to be selected",
            ));
        }
    }
    Ok(())
}

fn require_not_selected(
    selected: &BTreeSet<OpId>,
    op_id: &OpId,
    label: &'static str,
) -> Result<(), FacadeKernelErgonomicsError> {
    if selected.contains(op_id) {
        return Err(FacadeKernelErgonomicsError::FrontierOutcome(label));
    }
    Ok(())
}

fn action_call(action: &str) -> ActionCall {
    ActionCall {
        action_id: ActionId(action.to_owned()),
        predicate: PredicateId("predicate.release.promote".to_owned()),
        subject_ref: "service:checkout".to_owned(),
        circumstance_ref: "environment:prod".to_owned(),
        correlation_id: protocol::CorrelationId(format!("corr:{action}")),
    }
}

fn op(action: &str, index: u32) -> OpId {
    OpId(ActionId(action.to_owned()), index)
}

fn fact(name: &str) -> FactKind {
    FactKind(name.to_owned())
}

fn node(op_id: OpId, lane: &str, requires: &[&str], writes: &[&str]) -> GraphNode {
    GraphNode {
        op_id,
        lane: LaneId(lane.to_owned()),
        requires: requires.iter().map(|item| fact(item)).collect(),
        writes: writes
            .iter()
            .map(|scope| Scope((*scope).to_owned()))
            .collect(),
    }
}
