#![forbid(unsafe_code)]
#![deny(warnings)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use causlane::core::kernel::{self, ConstraintProvider};
use causlane::core::protocol::{
    ConstraintDecision, ConstraintId, ConstraintKind, ConstraintSnapshot, ConstraintSpec,
    CorrelationId, ExecutionBarrier, FrontierBlock, LeaseTableError, PlanHashError, ResourceClaim,
    Timestamp,
};
use causlane::prelude::{
    ActionCall, ActionId, AuditEventId, ClaimMode, ConsequenceProfile, ConstraintEpoch, FactKind,
    GraphIndex, GraphNode, ImpactSetHash, KernelContracts, LaneCapacity, LaneId, LeaseId, LeaseRef,
    LeaseTable, OpId, PlanHash, PredicateId, ResourceId, Scope,
};

const PLAN_HASH: &str = "sha256:7777777777777777777777777777777777777777777777777777777777777777";
const IMPACT_SET_HASH: &str =
    "sha256:8888888888888888888888888888888888888888888888888888888888888888";

/// Summary returned by the facade/kernel operator workflow example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FacadeKernelOperatorWorkflowSummary {
    /// Action calls admitted through the public facade.
    pub admitted_actions: usize,
    /// Barrier and truth-commit policy checks over facade-visible profiles.
    pub policy_checks: usize,
    /// Ops selected into the safe frontier.
    pub frontier_selected: usize,
    /// Ready ops rejected from the safe frontier with explicit reasons.
    pub frontier_rejections: usize,
    /// Constraint decisions checked through the kernel authority.
    pub constraint_decisions: usize,
    /// Leases granted in the positive lease-table path.
    pub leases_granted: usize,
    /// Barrier validations accepted in the positive lease-table path.
    pub barrier_validations: usize,
    /// Deterministic negative controls exercised by the example.
    pub negative_controls: usize,
}

/// Error type for the facade/kernel operator workflow example.
#[derive(Debug)]
pub enum FacadeKernelOperatorWorkflowError {
    /// A static plan hash embedded in the example was malformed.
    PlanHash(PlanHashError),
    /// A deterministic check observed a different outcome from the one expected.
    UnexpectedOutcome {
        /// Check being evaluated.
        check: &'static str,
        /// Debug rendering of the unexpected value.
        actual: String,
    },
}

impl fmt::Display for FacadeKernelOperatorWorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlanHash(error) => write!(f, "invalid static plan hash: {error:?}"),
            Self::UnexpectedOutcome { check, actual } => {
                write!(f, "unexpected outcome for {check}: {actual}")
            }
        }
    }
}

impl std::error::Error for FacadeKernelOperatorWorkflowError {}

impl From<PlanHashError> for FacadeKernelOperatorWorkflowError {
    fn from(error: PlanHashError) -> Self {
        Self::PlanHash(error)
    }
}

/// Run a near-real operator workflow through the public `causlane` facade and
/// curated core/kernel modules.
#[must_use = "the example result carries verification failures"]
pub fn run_facade_kernel_operator_workflow(
) -> Result<FacadeKernelOperatorWorkflowSummary, FacadeKernelOperatorWorkflowError> {
    let admitted_actions = verify_admission_batch()?;
    let policy_checks = verify_policy_matrix()?;
    let frontier = verify_frontier_workflow()?;
    let constraint_decisions = verify_constraint_decision_matrix()?;
    let lease_summary = verify_lease_workflow()?;
    let negative_controls = verify_duplicate_lease_control()?
        + verify_conflicting_exclusive_lease_control()?
        + verify_expired_barrier_lease_control()?
        + verify_token_over_budget_deny_control()?
        + verify_token_full_wait_control()?;

    Ok(FacadeKernelOperatorWorkflowSummary {
        admitted_actions,
        policy_checks,
        frontier_selected: frontier.selected.len(),
        frontier_rejections: frontier.write_scope_conflicts + frontier.lane_capacity_rejections,
        constraint_decisions,
        leases_granted: lease_summary.leases_granted,
        barrier_validations: lease_summary.barrier_validations,
        negative_controls,
    })
}

/// Verify batch action admission through the facade-visible kernel entrypoint.
#[must_use = "admission results must be inspected"]
pub fn verify_admission_batch() -> Result<usize, FacadeKernelOperatorWorkflowError> {
    let calls = [
        action_call("act_facade_plan_release"),
        action_call("act_facade_promote_release"),
        action_call("act_facade_project_release"),
    ];

    for call in &calls {
        match kernel::admit_call(call) {
            kernel::DispatchAdmission::Accepted { action_id } if action_id == call.action_id => {}
            other => return Err(unexpected("facade admission batch", &other)),
        }
    }

    Ok(calls.len())
}

/// Verify facade-visible policy helpers for common consequence profiles.
#[must_use = "policy helper results must be inspected"]
pub fn verify_policy_matrix() -> Result<usize, FacadeKernelOperatorWorkflowError> {
    if !kernel::requires_execution_barrier(ConsequenceProfile::RuntimeExecution) {
        return Err(unexpected(
            "runtime execution barrier policy",
            &"runtime execution did not require a barrier",
        ));
    }
    if !kernel::can_commit_observed_truth(ConsequenceProfile::RuntimeExecution) {
        return Err(unexpected(
            "runtime observed-truth policy",
            &"runtime execution could not commit observed truth",
        ));
    }
    if kernel::requires_execution_barrier(ConsequenceProfile::ProjectionRead) {
        return Err(unexpected(
            "projection read barrier policy",
            &"projection read unexpectedly required a barrier",
        ));
    }
    if kernel::can_commit_observed_truth(ConsequenceProfile::ProjectionRead) {
        return Err(unexpected(
            "projection observed-truth policy",
            &"projection read unexpectedly could commit observed truth",
        ));
    }
    Ok(4)
}

/// Summary for the facade-visible frontier workflow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FacadeFrontierWorkflowSummary {
    /// Selected op ids.
    pub selected: BTreeSet<OpId>,
    /// Rejections caused by pending-vs-pending write-scope conflicts.
    pub write_scope_conflicts: usize,
    /// Rejections caused by lane capacity.
    pub lane_capacity_rejections: usize,
}

/// Verify safe frontier selection through facade-visible graph types.
#[must_use = "frontier selection results must be inspected"]
pub fn verify_frontier_workflow(
) -> Result<FacadeFrontierWorkflowSummary, FacadeKernelOperatorWorkflowError> {
    let deploy_a = op("act_frontier_deploy_a", 0);
    let deploy_b = op("act_frontier_deploy_b", 0);
    let metrics = op("act_frontier_metrics", 0);
    let active_limited = op("act_frontier_active_limited", 0);
    let waiting_limited = op("act_frontier_waiting_limited", 0);
    let fact_blocked = op("act_frontier_waiting_for_fact", 0);
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
    index.add_node(node(
        metrics.clone(),
        "observability",
        &[],
        &["report:release"],
    ));
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
    index.add_node(node(
        fact_blocked.clone(),
        "release",
        &["artifact:signed"],
        &["environment:prod:followup"],
    ));
    index.mark_active(&active_limited);

    if index
        .waiting_on_fact(&FactKind("artifact:signed".to_owned()))
        .is_none_or(|waiting| !waiting.contains(&fact_blocked))
    {
        return Err(unexpected("frontier fact blocker", &index));
    }

    let lanes = BTreeMap::from([(LaneId("limited".to_owned()), LaneCapacity::Bounded(1))]);
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
    require_not_selected(
        &selection.selected,
        &fact_blocked,
        "fact-blocked op should not be selected",
    )?;

    let write_scope_conflicts = selection
        .rejected
        .iter()
        .filter(|rejection| {
            rejection.op_id == deploy_b
                && matches!(rejection.reason, FrontierBlock::WriteScopeConflict { .. })
        })
        .count();
    let lane_capacity_rejections = selection
        .rejected
        .iter()
        .filter(|rejection| {
            rejection.op_id == waiting_limited
                && matches!(rejection.reason, FrontierBlock::LaneAtCapacity { .. })
        })
        .count();

    if write_scope_conflicts != 1 {
        return Err(unexpected(
            "frontier write-scope conflict count",
            &selection.rejected,
        ));
    }
    if lane_capacity_rejections != 1 {
        return Err(unexpected(
            "frontier lane-capacity rejection count",
            &selection.rejected,
        ));
    }

    Ok(FacadeFrontierWorkflowSummary {
        selected: selection.selected,
        write_scope_conflicts,
        lane_capacity_rejections,
    })
}

/// Verify representative constraint decisions and the `KernelContracts`
/// delegation surface.
#[must_use = "constraint decisions must be inspected"]
pub fn verify_constraint_decision_matrix() -> Result<usize, FacadeKernelOperatorWorkflowError> {
    let plan = plan_hash()?;
    let cases = [
        (
            "allowed token claim",
            token_snapshot(3, Vec::new()),
            vec![claim("rollout_slots", "pool:release", ClaimMode::Token, 2)],
            DecisionKind::Allow,
        ),
        (
            "over-budget token claim",
            token_snapshot(1, Vec::new()),
            vec![claim("rollout_slots", "pool:release", ClaimMode::Token, 2)],
            DecisionKind::Deny,
        ),
        (
            "full token budget",
            token_snapshot(
                3,
                vec![lease(
                    "lease_slots_held",
                    &plan,
                    "rollout_slots",
                    "pool:release",
                    ClaimMode::Token,
                    2,
                    None,
                )],
            ),
            vec![claim("rollout_slots", "pool:release", ClaimMode::Token, 2)],
            DecisionKind::Wait,
        ),
        (
            "restricted shared read",
            ConstraintSnapshot {
                snapshot_id: ConstraintId("snapshot_restrict".to_owned()),
                epoch: ConstraintEpoch(1),
                constraints: vec![ConstraintSpec {
                    constraint_id: ConstraintId("constraint_notify".to_owned()),
                    kind: ConstraintKind::Restrict {
                        scope: Scope("environment:prod".to_owned()),
                        note: "notify observer".to_owned(),
                    },
                }],
                active_leases: Vec::new(),
            },
            vec![claim(
                "environment",
                "environment:prod",
                ClaimMode::SharedRead,
                1,
            )],
            DecisionKind::AllowWithRestrictions,
        ),
    ];

    for (label, snapshot, claims, expected) in &cases {
        let direct = kernel::resolve_constraints(snapshot, claims, &KernelContracts);
        let delegated = KernelContracts.resolve(snapshot, claims);
        if direct != delegated {
            return Err(unexpected(label, &(direct, delegated)));
        }
        if !expected.matches(&direct) {
            return Err(unexpected(label, &direct));
        }
    }

    Ok(cases.len())
}

/// Summary for the positive lease-table workflow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FacadeLeaseWorkflowSummary {
    /// Leases granted in the positive path.
    pub leases_granted: usize,
    /// Barrier validations accepted in the positive path.
    pub barrier_validations: usize,
}

/// Verify lease grant, claim coverage and barrier validation through the facade
/// and curated kernel contract authority.
#[must_use = "lease workflow results must be inspected"]
pub fn verify_lease_workflow(
) -> Result<FacadeLeaseWorkflowSummary, FacadeKernelOperatorWorkflowError> {
    let plan = plan_hash()?;
    let action = ActionId("act_facade_promote_release".to_owned());
    let claims = vec![
        claim(
            "environment",
            "environment:prod",
            ClaimMode::ExclusiveWrite,
            1,
        ),
        claim("rollout_slots", "pool:release", ClaimMode::Token, 2),
    ];
    let leases = vec![
        lease(
            "lease_environment_prod",
            &plan,
            "environment",
            "environment:prod",
            ClaimMode::ExclusiveWrite,
            1,
            Some(100),
        ),
        lease(
            "lease_rollout_slots",
            &plan,
            "rollout_slots",
            "pool:release",
            ClaimMode::Token,
            2,
            Some(100),
        ),
    ];
    let mut table = LeaseTable::new();

    for lease in &leases {
        table
            .grant(lease.clone(), &KernelContracts)
            .map_err(|error| unexpected("positive lease grant", &error))?;
    }
    table
        .validate_claim_coverage(&action, &plan, &claims)
        .map_err(|error| unexpected("positive claim coverage", &error))?;
    table
        .validate_barrier_leases(
            &barrier(&action, &plan, leases.clone()),
            Some(Timestamp(50)),
        )
        .map_err(|error| unexpected("positive barrier validation", &error))?;

    Ok(FacadeLeaseWorkflowSummary {
        leases_granted: leases.len(),
        barrier_validations: 1,
    })
}

/// Negative control: duplicate lease ids are rejected.
#[must_use = "negative controls must be inspected"]
pub fn verify_duplicate_lease_control() -> Result<usize, FacadeKernelOperatorWorkflowError> {
    let plan = plan_hash()?;
    let mut table = LeaseTable::new();
    let lease = lease(
        "lease_duplicate",
        &plan,
        "environment",
        "environment:prod",
        ClaimMode::ExclusiveWrite,
        1,
        None,
    );
    table
        .grant(lease.clone(), &KernelContracts)
        .map_err(|error| unexpected("duplicate lease setup", &error))?;
    match table.grant(lease, &KernelContracts) {
        Err(LeaseTableError::DuplicateLease { .. }) => Ok(1),
        result => Err(unexpected("duplicate lease control", &result)),
    }
}

/// Negative control: fail-closed exclusive lease conflicts are rejected.
#[must_use = "negative controls must be inspected"]
pub fn verify_conflicting_exclusive_lease_control(
) -> Result<usize, FacadeKernelOperatorWorkflowError> {
    let plan = plan_hash()?;
    let mut table = LeaseTable::new();
    table
        .grant(
            lease(
                "lease_conflict_a",
                &plan,
                "environment",
                "environment:prod",
                ClaimMode::ExclusiveWrite,
                1,
                None,
            ),
            &KernelContracts,
        )
        .map_err(|error| unexpected("exclusive conflict setup", &error))?;
    match table.grant(
        lease(
            "lease_conflict_b",
            &plan,
            "environment",
            "environment:prod",
            ClaimMode::ExclusiveWrite,
            1,
            None,
        ),
        &KernelContracts,
    ) {
        Err(LeaseTableError::Conflict { .. }) => Ok(1),
        result => Err(unexpected("exclusive conflict control", &result)),
    }
}

/// Negative control: an expired lease cannot validate a barrier.
#[must_use = "negative controls must be inspected"]
pub fn verify_expired_barrier_lease_control() -> Result<usize, FacadeKernelOperatorWorkflowError> {
    let plan = plan_hash()?;
    let action = ActionId("act_facade_promote_release".to_owned());
    let active = lease(
        "lease_expiring",
        &plan,
        "environment",
        "environment:prod",
        ClaimMode::ExclusiveWrite,
        1,
        Some(10),
    );
    let mut table = LeaseTable::new();
    table
        .grant(active.clone(), &KernelContracts)
        .map_err(|error| unexpected("expired lease setup", &error))?;
    match table.validate_barrier_leases(&barrier(&action, &plan, vec![active]), Some(Timestamp(10)))
    {
        Err(LeaseTableError::Expired { .. }) => Ok(1),
        result => Err(unexpected("expired barrier lease control", &result)),
    }
}

/// Negative control: a claim larger than a token budget is denied.
#[must_use = "negative controls must be inspected"]
pub fn verify_token_over_budget_deny_control() -> Result<usize, FacadeKernelOperatorWorkflowError> {
    match kernel::resolve_constraints(
        &token_snapshot(1, Vec::new()),
        &[claim("rollout_slots", "pool:release", ClaimMode::Token, 2)],
        &KernelContracts,
    ) {
        ConstraintDecision::Deny { .. } => Ok(1),
        result => Err(unexpected("token over-budget deny control", &result)),
    }
}

/// Negative control: a currently full token budget makes the plan wait.
#[must_use = "negative controls must be inspected"]
pub fn verify_token_full_wait_control() -> Result<usize, FacadeKernelOperatorWorkflowError> {
    let plan = plan_hash()?;
    match kernel::resolve_constraints(
        &token_snapshot(
            3,
            vec![lease(
                "lease_slots_held",
                &plan,
                "rollout_slots",
                "pool:release",
                ClaimMode::Token,
                2,
                None,
            )],
        ),
        &[claim("rollout_slots", "pool:release", ClaimMode::Token, 2)],
        &KernelContracts,
    ) {
        ConstraintDecision::Wait { .. } => Ok(1),
        result => Err(unexpected("token full wait control", &result)),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecisionKind {
    Allow,
    Wait,
    Deny,
    AllowWithRestrictions,
}

impl DecisionKind {
    fn matches(self, decision: &ConstraintDecision) -> bool {
        matches!(
            (self, decision),
            (Self::Allow, ConstraintDecision::Allow { .. })
                | (Self::Wait, ConstraintDecision::Wait { .. })
                | (Self::Deny, ConstraintDecision::Deny { .. })
                | (
                    Self::AllowWithRestrictions,
                    ConstraintDecision::AllowWithRestrictions { .. }
                )
        )
    }
}

fn require_selected<'a>(
    selected: &BTreeSet<OpId>,
    expected: impl IntoIterator<Item = &'a OpId>,
) -> Result<(), FacadeKernelOperatorWorkflowError> {
    for op_id in expected {
        if !selected.contains(op_id) {
            return Err(unexpected("frontier expected selection", op_id));
        }
    }
    Ok(())
}

fn require_not_selected(
    selected: &BTreeSet<OpId>,
    op_id: &OpId,
    check: &'static str,
) -> Result<(), FacadeKernelOperatorWorkflowError> {
    if selected.contains(op_id) {
        return Err(unexpected(check, op_id));
    }
    Ok(())
}

fn action_call(action: &str) -> ActionCall {
    ActionCall {
        action_id: ActionId(action.to_owned()),
        predicate: PredicateId("predicate.facade.release".to_owned()),
        subject_ref: "service:checkout".to_owned(),
        circumstance_ref: "environment:prod".to_owned(),
        correlation_id: CorrelationId(format!("corr:{action}")),
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

fn plan_hash() -> Result<PlanHash, PlanHashError> {
    PlanHash::new(PLAN_HASH)
}

fn claim(resource: &str, scope: &str, mode: ClaimMode, amount: u64) -> ResourceClaim {
    ResourceClaim {
        resource: ResourceId(resource.to_owned()),
        scope: Scope(scope.to_owned()),
        mode,
        amount,
    }
}

fn lease(
    id: &str,
    plan: &PlanHash,
    resource: &str,
    scope: &str,
    mode: ClaimMode,
    amount: u64,
    expires_at: Option<u64>,
) -> LeaseRef {
    LeaseRef {
        lease_id: LeaseId(id.to_owned()),
        resource: ResourceId(resource.to_owned()),
        scope: Scope(scope.to_owned()),
        mode,
        amount,
        holder_action_id: ActionId("act_facade_promote_release".to_owned()),
        holder_plan_hash: plan.clone(),
        holder_op_index: Some(0),
        epoch: ConstraintEpoch(1),
        expires_at: expires_at.map(Timestamp),
        lease_event_id: AuditEventId(format!("evt_{id}")),
    }
}

fn barrier(action: &ActionId, plan: &PlanHash, leases: Vec<LeaseRef>) -> ExecutionBarrier {
    ExecutionBarrier {
        barrier_id: AuditEventId("evt_barrier_facade_promote".to_owned()),
        action_id: action.clone(),
        plan_hash: plan.clone(),
        op_indexes: vec![0],
        impact_set_hash: ImpactSetHash(IMPACT_SET_HASH.to_owned()),
        witnesses: Vec::new(),
        leases,
        authz_decision_refs: Vec::new(),
        constraint_snapshot_id: Some(ConstraintId("snapshot_release".to_owned())),
    }
}

fn token_snapshot(limit: u64, active_leases: Vec<LeaseRef>) -> ConstraintSnapshot {
    ConstraintSnapshot {
        snapshot_id: ConstraintId("snapshot_tokens".to_owned()),
        epoch: ConstraintEpoch(1),
        constraints: vec![ConstraintSpec {
            constraint_id: ConstraintId("constraint_rollout_slots".to_owned()),
            kind: ConstraintKind::TokenBudget {
                resource: ResourceId("rollout_slots".to_owned()),
                scope: Scope("pool:release".to_owned()),
                limit,
            },
        }],
        active_leases,
    }
}

fn unexpected(check: &'static str, actual: &impl fmt::Debug) -> FacadeKernelOperatorWorkflowError {
    FacadeKernelOperatorWorkflowError::UnexpectedOutcome {
        check,
        actual: format!("{actual:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_kernel_operator_workflow_summary_counts(
    ) -> Result<(), FacadeKernelOperatorWorkflowError> {
        let summary = run_facade_kernel_operator_workflow()?;
        assert_eq!(
            summary,
            FacadeKernelOperatorWorkflowSummary {
                admitted_actions: 3,
                policy_checks: 4,
                frontier_selected: 2,
                frontier_rejections: 2,
                constraint_decisions: 4,
                leases_granted: 2,
                barrier_validations: 1,
                negative_controls: 5,
            }
        );
        Ok(())
    }

    #[test]
    fn negative_controls_are_independently_observable(
    ) -> Result<(), FacadeKernelOperatorWorkflowError> {
        assert_eq!(verify_duplicate_lease_control()?, 1);
        assert_eq!(verify_conflicting_exclusive_lease_control()?, 1);
        assert_eq!(verify_expired_barrier_lease_control()?, 1);
        assert_eq!(verify_token_over_budget_deny_control()?, 1);
        assert_eq!(verify_token_full_wait_control()?, 1);
        Ok(())
    }

    #[test]
    fn constraint_decisions_delegate_to_kernel_contracts(
    ) -> Result<(), FacadeKernelOperatorWorkflowError> {
        assert_eq!(verify_constraint_decision_matrix()?, 4);
        Ok(())
    }
}
