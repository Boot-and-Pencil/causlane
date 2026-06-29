#![forbid(unsafe_code)]
#![deny(warnings)]

use std::{collections::BTreeSet, convert::Infallible, fmt};

use causlane::core::ports::{AuditLogPort, ExecutorPort};
use causlane::core::protocol::{
    ActionId, AuditEvent, AuditEventId, AuditEventKind, AuthzDecision, AuthzDecisionRef,
    AuthzDenyReason, AuthzPolicy, CapabilitySpendRefusal, ClaimMode, ConstraintEpoch,
    CorrelationId, EffectSignature, ExecutionBarrier, ExecutionCapability, FieldPath,
    ImpactHardness, ImpactSetHash, LeaseId, LeaseRef, Op, PlanHash, PlanHashError,
    ProjectionReadRequest, RedactionPolicy, RedactionView, ResourceId, Scope, Timestamp,
    MAY_PROJECT_STAGE,
};
use causlane_runtime::adapters::audit::{AuditAdapterError, InMemoryAuditLog};
use causlane_runtime::adapters::tracing::{InMemoryTraceSink, TraceProjectingAuditLog};
use causlane_runtime::guarded_executor::{
    ExecutorService, GuardedExecutionRequest, GuardedExecutor, SpendError,
};
use causlane_runtime::projection_guard::{guard_projection_read, ProjectionReadError};

const PLAN_HASH: &str = "sha256:5555555555555555555555555555555555555555555555555555555555555555";
const IMPACT_SET_HASH: &str =
    "sha256:6666666666666666666666666666666666666666666666666666666666666666";
const ACTION_ID: &str = "runtime.operator.rollout";
const PREDICATE_ID: &str = "runtime.operator.rollout";
const ACTOR_REF: &str = "actor://operator/release-manager";
const OTHER_ACTOR_REF: &str = "actor://operator/support";
const CORRELATION_ID: &str = "corr-runtime-operator-workflow-1";
const EXECUTION_STAGE: &str = "execution_barrier_logged";
const POLICY: AuthzPolicy<'static> = AuthzPolicy {
    id: "runtime.operator.policy",
    version: "1",
    max_age: Some(120),
};

/// Summary returned by the runtime operator workflow example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOperatorWorkflowSummary {
    /// Guarded operations that reached the executor.
    pub executed_ops: usize,
    /// Object/fact refs produced by the executor.
    pub produced_refs: usize,
    /// Audit events appended through the tracing audit wrapper.
    pub audit_events: usize,
    /// Trace spans projected from successful audit appends.
    pub trace_spans: usize,
    /// Projection fields classified by the guarded projection read.
    pub projected_fields: usize,
    /// Projection fields redacted by default.
    pub redacted_fields: usize,
    /// Deterministic negative controls exercised by the example.
    pub negative_controls: usize,
}

/// Error type for the runtime operator workflow example.
#[derive(Debug)]
pub enum RuntimeOperatorWorkflowError {
    /// A static plan hash embedded in the example was malformed.
    PlanHash(PlanHashError),
    /// Runtime audit append failed.
    Audit(AuditAdapterError),
    /// Guarded projection read failed.
    Projection(ProjectionReadError),
    /// Guarded execution failed in the positive path.
    GuardedExecution(SpendError<Infallible>),
    /// A deterministic check observed a different outcome from the one expected.
    UnexpectedOutcome {
        /// Check being evaluated.
        check: &'static str,
        /// Debug rendering of the unexpected value.
        actual: String,
    },
}

impl fmt::Display for RuntimeOperatorWorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlanHash(error) => write!(f, "invalid static plan hash: {error:?}"),
            Self::Audit(error) => write!(f, "audit append failed: {error}"),
            Self::Projection(error) => write!(f, "projection guard failed: {error:?}"),
            Self::GuardedExecution(error) => {
                write!(f, "guarded execution failed unexpectedly: {error:?}")
            }
            Self::UnexpectedOutcome { check, actual } => {
                write!(f, "unexpected outcome for {check}: {actual}")
            }
        }
    }
}

impl std::error::Error for RuntimeOperatorWorkflowError {}

impl From<PlanHashError> for RuntimeOperatorWorkflowError {
    fn from(error: PlanHashError) -> Self {
        Self::PlanHash(error)
    }
}

impl From<AuditAdapterError> for RuntimeOperatorWorkflowError {
    fn from(error: AuditAdapterError) -> Self {
        Self::Audit(error)
    }
}

impl From<ProjectionReadError> for RuntimeOperatorWorkflowError {
    fn from(error: ProjectionReadError) -> Self {
        Self::Projection(error)
    }
}

/// Run a realistic runtime host workflow over guarded execution, audit trace
/// projection and guarded dashboard projection redaction.
#[must_use = "the example result carries verification failures"]
pub fn run_runtime_operator_workflow(
) -> Result<RuntimeOperatorWorkflowSummary, RuntimeOperatorWorkflowError> {
    let plan = plan_hash()?;
    let action = action_id();
    let execution_allow = execution_allow_decision(&plan);
    let projection_allow = projection_allow_decision(ACTOR_REF, &plan);
    let mut barrier = execution_barrier(plan.clone());
    barrier.authz_decision_refs = vec![execution_allow.decision_event_id.clone()];
    let ops = rollout_ops();
    let stages = required_execution_stages();
    let guarded = GuardedExecutor::new(RolloutExecutor);
    let mut produced_refs = Vec::new();

    for op in &ops {
        let outcome = guarded
            .call(guarded_request(
                &barrier,
                &stages,
                std::slice::from_ref(&execution_allow),
                op,
                Timestamp(30),
            ))
            .map_err(RuntimeOperatorWorkflowError::GuardedExecution)?;
        let expected = produced_ref(op.index);
        if outcome.produced_refs != [expected.clone()] {
            return Err(unexpected(
                "positive multi-op guarded execution",
                &outcome.produced_refs,
            ));
        }
        produced_refs.push(expected);
    }

    let mut audit =
        TraceProjectingAuditLog::new(InMemoryAuditLog::default(), InMemoryTraceSink::default());
    append_operator_trace(
        &mut audit,
        &action,
        &plan,
        &barrier,
        &execution_allow,
        &projection_allow,
        &ops,
    )?;

    let projection_view = project_operator_dashboard(
        &action,
        &plan,
        ACTOR_REF,
        std::slice::from_ref(&projection_allow),
    )?;
    if projection_view.redacted != field_set(&["dashboard.operator_token", "dashboard.deploy_key"])
    {
        return Err(unexpected(
            "dashboard projection redaction",
            &projection_view.redacted,
        ));
    }

    let negative_controls = verify_missing_execution_authz()?
        + verify_expired_capability_is_refused()?
        + verify_projection_without_authz()?
        + verify_projection_for_wrong_actor()?
        + verify_duplicate_audit_event_has_no_span()?;

    Ok(RuntimeOperatorWorkflowSummary {
        executed_ops: ops.len(),
        produced_refs: produced_refs.len(),
        audit_events: audit.audit_log().events().len(),
        trace_spans: audit.trace_sink().spans.len(),
        projected_fields: projection_view.revealed.len() + projection_view.redacted.len(),
        redacted_fields: projection_view.redacted.len(),
        negative_controls,
    })
}

/// Negative control: missing execution authz denies before any op runs.
#[must_use = "the control result carries verification failures"]
pub fn verify_missing_execution_authz() -> Result<usize, RuntimeOperatorWorkflowError> {
    let plan = plan_hash()?;
    let barrier = execution_barrier(plan);
    let stages = required_execution_stages();
    let ops = rollout_ops();
    let op = first_op(&ops)?;
    let guarded = GuardedExecutor::new(RolloutExecutor);

    match guarded.call(guarded_request(&barrier, &stages, &[], op, Timestamp(30))) {
        Err(SpendError::Unauthorized(denied))
            if denied.stage == EXECUTION_STAGE && denied.reason == AuthzDenyReason::Missing =>
        {
            Ok(1)
        }
        result => Err(unexpected("missing execution authz control", &result)),
    }
}

/// Negative control: an expired lease-derived capability is refused at spend time.
#[must_use = "the control result carries verification failures"]
pub fn verify_expired_capability_is_refused() -> Result<usize, RuntimeOperatorWorkflowError> {
    let plan = plan_hash()?;
    let allow = execution_allow_decision(&plan);
    let barrier = execution_barrier_with_expiring_leases(plan, Timestamp(10));
    let stages = required_execution_stages();
    let ops = rollout_ops();
    let op = first_op(&ops)?;
    let guarded = GuardedExecutor::new(RolloutExecutor);

    match guarded.call(guarded_request(
        &barrier,
        &stages,
        std::slice::from_ref(&allow),
        op,
        Timestamp(10),
    )) {
        Err(SpendError::CapabilityRefused(CapabilitySpendRefusal::Expired { expires_at, now }))
            if expires_at == Timestamp(10) && now == Timestamp(10) =>
        {
            Ok(1)
        }
        result => Err(unexpected("expired capability control", &result)),
    }
}

/// Negative control: projection without a `may_project` allow is fail-closed.
#[must_use = "the control result carries verification failures"]
pub fn verify_projection_without_authz() -> Result<usize, RuntimeOperatorWorkflowError> {
    let action = action_id();
    let plan = plan_hash()?;
    match project_operator_dashboard(&action, &plan, ACTOR_REF, &[]) {
        Err(ProjectionReadError::Unauthorized { stage, reason })
            if stage == MAY_PROJECT_STAGE && reason == AuthzDenyReason::Missing =>
        {
            Ok(1)
        }
        result => Err(unexpected("projection authz control", &result)),
    }
}

/// Negative control: another actor's projection allow is not reusable.
#[must_use = "the control result carries verification failures"]
pub fn verify_projection_for_wrong_actor() -> Result<usize, RuntimeOperatorWorkflowError> {
    let action = action_id();
    let plan = plan_hash()?;
    let other_actor_allow = projection_allow_decision(OTHER_ACTOR_REF, &plan);
    match project_operator_dashboard(
        &action,
        &plan,
        ACTOR_REF,
        std::slice::from_ref(&other_actor_allow),
    ) {
        Err(ProjectionReadError::Unauthorized { stage, reason })
            if stage == MAY_PROJECT_STAGE && reason == AuthzDenyReason::Missing =>
        {
            Ok(1)
        }
        result => Err(unexpected("projection wrong-actor control", &result)),
    }
}

/// Negative control: duplicate audit ids are rejected and do not emit telemetry.
#[must_use = "the control result carries verification failures"]
pub fn verify_duplicate_audit_event_has_no_span() -> Result<usize, RuntimeOperatorWorkflowError> {
    let plan = plan_hash()?;
    let action = action_id();
    let event = runtime_event(
        "evt_runtime_operator_duplicate_control",
        &action,
        AuditEventKind::ExecutionStarted,
        &plan,
        Timestamp(1),
    );
    let mut audit =
        TraceProjectingAuditLog::new(InMemoryAuditLog::default(), InMemoryTraceSink::default());

    AuditLogPort::append(&mut audit, event.clone())?;
    let result = AuditLogPort::append(&mut audit, event);

    match result {
        Err(AuditAdapterError::DuplicateEventId { event_id })
            if event_id == AuditEventId("evt_runtime_operator_duplicate_control".to_owned())
                && audit.audit_log().events().len() == 1
                && audit.trace_sink().spans.len() == 1 =>
        {
            Ok(1)
        }
        other => Err(unexpected("duplicate audit id control", &other)),
    }
}

fn append_operator_trace(
    audit: &mut TraceProjectingAuditLog<InMemoryAuditLog, InMemoryTraceSink>,
    action: &ActionId,
    plan: &PlanHash,
    barrier: &ExecutionBarrier,
    execution_allow: &AuthzDecisionRef,
    projection_allow: &AuthzDecisionRef,
    ops: &[Op],
) -> Result<(), AuditAdapterError> {
    let mut events = vec![
        runtime_event(
            "evt_runtime_operator_barrier",
            action,
            AuditEventKind::ExecutionBarrierLogged,
            plan,
            Timestamp(20),
        )
        .with_execution_barrier(barrier.clone())
        .with_impact_set_hash(impact_hash()),
        runtime_event(
            "evt_runtime_operator_authz_execution",
            action,
            AuditEventKind::AuthzDecisionRecorded,
            plan,
            Timestamp(19),
        )
        .with_authz_decision(execution_allow.clone()),
        runtime_event(
            "evt_runtime_operator_authz_projection",
            action,
            AuditEventKind::AuthzDecisionRecorded,
            plan,
            Timestamp(31),
        )
        .with_authz_decision(projection_allow.clone()),
    ];

    for op in ops {
        let started = format!("evt_runtime_operator_op{}_started", op.index);
        let completed = format!("evt_runtime_operator_op{}_completed", op.index);
        events.push(
            runtime_event(
                started.clone(),
                action,
                AuditEventKind::ExecutionStarted,
                plan,
                Timestamp(21 + u64::from(op.index) * 2),
            )
            .with_causation_id(AuditEventId("evt_runtime_operator_barrier".to_owned())),
        );
        events.push(
            runtime_event(
                completed,
                action,
                AuditEventKind::ExecutionCompleted,
                plan,
                Timestamp(22 + u64::from(op.index) * 2),
            )
            .with_causation_id(AuditEventId(started)),
        );
    }

    events.extend([
        runtime_event(
            "evt_runtime_operator_truth",
            action,
            AuditEventKind::ObservedTruthCommitted,
            plan,
            Timestamp(28),
        )
        .with_causation_id(AuditEventId(
            "evt_runtime_operator_op2_completed".to_owned(),
        )),
        runtime_event(
            "evt_runtime_operator_projection",
            action,
            AuditEventKind::ProjectionEmitted,
            plan,
            Timestamp(32),
        )
        .with_causation_id(AuditEventId("evt_runtime_operator_truth".to_owned())),
        runtime_event(
            "evt_runtime_operator_closed",
            action,
            AuditEventKind::LifecycleClosed,
            plan,
            Timestamp(33),
        )
        .with_causation_id(AuditEventId("evt_runtime_operator_projection".to_owned())),
    ]);

    AuditLogPort::append_batch(audit, events)?;
    Ok(())
}

fn project_operator_dashboard(
    action: &ActionId,
    plan: &PlanHash,
    actor: &str,
    decisions: &[AuthzDecisionRef],
) -> Result<RedactionView, ProjectionReadError> {
    let fields = projection_fields();
    let req = ProjectionReadRequest {
        action,
        plan,
        predicate_id: PREDICATE_ID,
        actor,
        policy: POLICY,
        now: Timestamp(40),
    };
    guard_projection_read(decisions, &req, &redaction_policy(), &fields)
}

fn guarded_request<'a>(
    barrier: &'a ExecutionBarrier,
    stages: &'a [String],
    decisions: &'a [AuthzDecisionRef],
    op: &'a Op,
    now: Timestamp,
) -> GuardedExecutionRequest<'a> {
    GuardedExecutionRequest {
        barrier,
        predicate_id: PREDICATE_ID,
        required_stages: stages,
        decisions,
        expected_policy: POLICY,
        now,
        op,
    }
}

struct RolloutExecutor;

impl ExecutorPort for RolloutExecutor {
    type Error = Infallible;

    fn execute(
        &self,
        op: &Op,
        _capability: &ExecutionCapability,
    ) -> Result<Vec<String>, Self::Error> {
        Ok(vec![produced_ref(op.index)])
    }
}

fn execution_barrier(plan: PlanHash) -> ExecutionBarrier {
    ExecutionBarrier {
        barrier_id: AuditEventId("evt_runtime_operator_barrier".to_owned()),
        action_id: action_id(),
        plan_hash: plan.clone(),
        op_indexes: vec![0, 1, 2],
        impact_set_hash: impact_hash(),
        witnesses: Vec::new(),
        leases: rollout_leases(&plan),
        authz_decision_refs: Vec::new(),
        constraint_snapshot_id: None,
    }
}

fn execution_barrier_with_expiring_leases(
    plan: PlanHash,
    expires_at: Timestamp,
) -> ExecutionBarrier {
    let mut barrier = execution_barrier(plan);
    for lease in &mut barrier.leases {
        lease.expires_at = Some(expires_at);
    }
    barrier
}

fn rollout_leases(plan: &PlanHash) -> Vec<LeaseRef> {
    vec![
        lease_ref(plan, 0, "runtime/release/window"),
        lease_ref(plan, 1, "runtime/release/canary"),
        lease_ref(plan, 2, "runtime/release/traffic"),
    ]
}

fn lease_ref(plan: &PlanHash, op_index: u32, scope: &str) -> LeaseRef {
    LeaseRef {
        lease_id: LeaseId(format!("lease-runtime-operator-{op_index}")),
        resource: ResourceId(format!("resource://{scope}")),
        scope: Scope(scope.to_owned()),
        mode: ClaimMode::ExclusiveWrite,
        amount: 1,
        holder_action_id: action_id(),
        holder_plan_hash: plan.clone(),
        holder_op_index: Some(op_index),
        epoch: ConstraintEpoch(0),
        expires_at: None,
        lease_event_id: AuditEventId(format!("evt_runtime_operator_lease_{op_index}")),
    }
}

fn rollout_ops() -> Vec<Op> {
    vec![
        rollout_op(
            0,
            "validate_window",
            &["runtime/release/calendar"],
            &["runtime/release/window"],
            &[],
        ),
        rollout_op(
            1,
            "prepare_canary",
            &["runtime/release/window"],
            &["runtime/release/canary"],
            &[produced_ref(0)],
        ),
        rollout_op(
            2,
            "promote_canary",
            &["runtime/release/canary"],
            &["runtime/release/traffic"],
            &[produced_ref(1)],
        ),
    ]
}

fn rollout_op(index: u32, kind: &str, reads: &[&str], writes: &[&str], requires: &[String]) -> Op {
    Op {
        index,
        kind: kind.to_owned(),
        effect: EffectSignature {
            reads: reads
                .iter()
                .map(|scope| Scope((*scope).to_owned()))
                .collect(),
            writes: writes
                .iter()
                .map(|scope| Scope((*scope).to_owned()))
                .collect(),
            produces: vec![produced_ref(index)],
            requires: requires.to_vec(),
            invalidates: Vec::new(),
            conflict_domains: Vec::new(),
            hardness: ImpactHardness::Hard,
        },
    }
}

fn produced_ref(index: u32) -> String {
    match index {
        0 => "object://runtime/operator/window-accepted".to_owned(),
        1 => "object://runtime/operator/canary-ready".to_owned(),
        2 => "object://runtime/operator/traffic-promoted".to_owned(),
        other => format!("object://runtime/operator/op-{other}"),
    }
}

fn first_op(ops: &[Op]) -> Result<&Op, RuntimeOperatorWorkflowError> {
    ops.first()
        .ok_or_else(|| unexpected("runtime operator workflow op inventory", &ops.len()))
}

fn execution_allow_decision(plan: &PlanHash) -> AuthzDecisionRef {
    authz_decision(
        "evt_runtime_operator_authz_execution",
        EXECUTION_STAGE,
        ACTOR_REF,
        plan,
    )
}

fn projection_allow_decision(actor: &str, plan: &PlanHash) -> AuthzDecisionRef {
    authz_decision(
        "evt_runtime_operator_authz_projection",
        MAY_PROJECT_STAGE,
        actor,
        plan,
    )
}

fn authz_decision(event_id: &str, stage: &str, actor: &str, plan: &PlanHash) -> AuthzDecisionRef {
    AuthzDecisionRef {
        decision_event_id: AuditEventId(event_id.to_owned()),
        action_id: action_id(),
        plan_hash: plan.clone(),
        predicate_id: PREDICATE_ID.to_owned(),
        actor: actor.to_owned(),
        stage: stage.to_owned(),
        decision: AuthzDecision::Allow,
        policy_id: POLICY.id.to_owned(),
        policy_version: POLICY.version.to_owned(),
        issued_at: Timestamp(0),
        expires_at: Some(Timestamp(120)),
        attestation: None,
    }
}

fn runtime_event(
    event_id: impl Into<String>,
    action: &ActionId,
    kind: AuditEventKind,
    plan: &PlanHash,
    occurred_at: Timestamp,
) -> AuditEvent {
    AuditEvent::new(AuditEventId(event_id.into()), action.clone(), kind)
        .with_plan_hash(plan.clone())
        .with_correlation_id(CorrelationId(CORRELATION_ID.to_owned()))
        .with_occurred_at(occurred_at)
}

fn projection_fields() -> Vec<FieldPath> {
    vec![
        field("dashboard.release"),
        field("dashboard.canary_percent"),
        field("dashboard.error_budget"),
        field("dashboard.operator_token"),
        field("dashboard.deploy_key"),
    ]
}

fn redaction_policy() -> RedactionPolicy {
    RedactionPolicy {
        revealable: field_set(&[
            "dashboard.release",
            "dashboard.canary_percent",
            "dashboard.error_budget",
        ]),
    }
}

fn field_set(paths: &[&str]) -> BTreeSet<FieldPath> {
    paths.iter().map(|path| field(path)).collect()
}

fn field(path: &str) -> FieldPath {
    FieldPath(path.to_owned())
}

fn required_execution_stages() -> Vec<String> {
    vec![EXECUTION_STAGE.to_owned()]
}

fn plan_hash() -> Result<PlanHash, PlanHashError> {
    PlanHash::new(PLAN_HASH)
}

fn action_id() -> ActionId {
    ActionId(ACTION_ID.to_owned())
}

fn impact_hash() -> ImpactSetHash {
    ImpactSetHash(IMPACT_SET_HASH.to_owned())
}

fn unexpected<T: fmt::Debug>(check: &'static str, actual: &T) -> RuntimeOperatorWorkflowError {
    RuntimeOperatorWorkflowError::UnexpectedOutcome {
        check,
        actual: format!("{actual:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        field, run_runtime_operator_workflow, verify_duplicate_audit_event_has_no_span,
        verify_expired_capability_is_refused, verify_missing_execution_authz,
        verify_projection_for_wrong_actor, verify_projection_without_authz,
        RuntimeOperatorWorkflowError,
    };

    #[test]
    fn runtime_operator_workflow_summary_counts() -> Result<(), RuntimeOperatorWorkflowError> {
        let summary = run_runtime_operator_workflow()?;

        assert_eq!(summary.executed_ops, 3);
        assert_eq!(summary.produced_refs, 3);
        assert_eq!(summary.audit_events, 12);
        assert_eq!(summary.trace_spans, 12);
        assert_eq!(summary.projected_fields, 5);
        assert_eq!(summary.redacted_fields, 2);
        assert_eq!(summary.negative_controls, 5);
        Ok(())
    }

    #[test]
    fn negative_controls_are_independently_observable() -> Result<(), RuntimeOperatorWorkflowError>
    {
        assert_eq!(verify_missing_execution_authz()?, 1);
        assert_eq!(verify_expired_capability_is_refused()?, 1);
        assert_eq!(verify_projection_without_authz()?, 1);
        assert_eq!(verify_projection_for_wrong_actor()?, 1);
        assert_eq!(verify_duplicate_audit_event_has_no_span()?, 1);
        Ok(())
    }

    #[test]
    fn projection_redacts_operator_fields() -> Result<(), RuntimeOperatorWorkflowError> {
        let summary = run_runtime_operator_workflow()?;
        assert_eq!(summary.redacted_fields, 2);
        assert_eq!(field("dashboard.deploy_key").0, "dashboard.deploy_key");
        Ok(())
    }
}
