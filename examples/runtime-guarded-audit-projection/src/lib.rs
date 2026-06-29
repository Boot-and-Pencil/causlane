#![forbid(unsafe_code)]
#![deny(warnings)]

use std::{convert::Infallible, fmt};

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

const PLAN_HASH: &str = "sha256:3333333333333333333333333333333333333333333333333333333333333333";
const IMPACT_SET_HASH: &str =
    "sha256:4444444444444444444444444444444444444444444444444444444444444444";
const ACTION_ID: &str = "runtime.release.promote";
const PREDICATE_ID: &str = "runtime.release.promote";
const ACTOR_REF: &str = "actor://release/runtime";
const CORRELATION_ID: &str = "corr-runtime-guarded-audit-projection-1";
const EXECUTION_STAGE: &str = "execution_barrier_logged";
const EXECUTION_REF: &str = "object://runtime/release/promote";
const POLICY: AuthzPolicy<'static> = AuthzPolicy {
    id: "runtime.guard.policy",
    version: "1",
    max_age: Some(60),
};

/// Summary returned by the runtime guarded audit/projection example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeGuardedAuditProjectionSummary {
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

/// Error type for the runtime guarded audit/projection example.
#[derive(Debug)]
pub enum RuntimeGuardedAuditProjectionError {
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

impl fmt::Display for RuntimeGuardedAuditProjectionError {
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

impl std::error::Error for RuntimeGuardedAuditProjectionError {}

impl From<PlanHashError> for RuntimeGuardedAuditProjectionError {
    fn from(error: PlanHashError) -> Self {
        Self::PlanHash(error)
    }
}

impl From<AuditAdapterError> for RuntimeGuardedAuditProjectionError {
    fn from(error: AuditAdapterError) -> Self {
        Self::Audit(error)
    }
}

impl From<ProjectionReadError> for RuntimeGuardedAuditProjectionError {
    fn from(error: ProjectionReadError) -> Self {
        Self::Projection(error)
    }
}

/// Run the public runtime composition slice: authz-guarded execution, audit
/// append, trace projection and guarded projection redaction.
#[must_use = "the example result carries verification failures"]
pub fn run_runtime_guarded_audit_projection(
) -> Result<RuntimeGuardedAuditProjectionSummary, RuntimeGuardedAuditProjectionError> {
    let plan = plan_hash()?;
    let action = action_id();
    let execution_allow = execution_allow_decision(&plan);
    let projection_allow = projection_allow_decision(&plan);
    let mut barrier = execution_barrier(plan.clone());
    barrier.authz_decision_refs = vec![execution_allow.decision_event_id.clone()];
    let op = promote_op();
    let stages = required_execution_stages();
    let guarded = GuardedExecutor::new(MarkerExecutor);

    let outcome = guarded
        .call(guarded_request(
            &barrier,
            &stages,
            std::slice::from_ref(&execution_allow),
            &op,
            Timestamp(10),
        ))
        .map_err(RuntimeGuardedAuditProjectionError::GuardedExecution)?;
    if outcome.produced_refs != [EXECUTION_REF.to_owned()] {
        return Err(unexpected(
            "positive guarded execution",
            &outcome.produced_refs,
        ));
    }

    let mut audit =
        TraceProjectingAuditLog::new(InMemoryAuditLog::default(), InMemoryTraceSink::default());
    append_runtime_trace(
        &mut audit,
        &action,
        &plan,
        &barrier,
        &execution_allow,
        &projection_allow,
    )?;

    let projection_view = project_release_status(
        &action,
        &plan,
        ACTOR_REF,
        std::slice::from_ref(&projection_allow),
    )?;
    if projection_view.redacted != [field("release.operator_token")].into_iter().collect() {
        return Err(unexpected(
            "projection redaction",
            &projection_view.redacted,
        ));
    }

    let negative_controls = verify_missing_execution_authz()?
        + verify_expired_capability_is_refused()?
        + verify_projection_without_authz()?
        + verify_duplicate_audit_event_has_no_span()?;

    Ok(RuntimeGuardedAuditProjectionSummary {
        executed_ops: 1,
        produced_refs: outcome.produced_refs.len(),
        audit_events: audit.audit_log().events().len(),
        trace_spans: audit.trace_sink().spans.len(),
        projected_fields: projection_view.revealed.len() + projection_view.redacted.len(),
        redacted_fields: projection_view.redacted.len(),
        negative_controls,
    })
}

/// Negative control: missing execution authz denies before any op runs.
#[must_use = "the control result carries verification failures"]
pub fn verify_missing_execution_authz() -> Result<usize, RuntimeGuardedAuditProjectionError> {
    let plan = plan_hash()?;
    let barrier = execution_barrier(plan);
    let stages = required_execution_stages();
    let op = promote_op();
    let guarded = GuardedExecutor::new(MarkerExecutor);

    match guarded.call(guarded_request(&barrier, &stages, &[], &op, Timestamp(10))) {
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
pub fn verify_expired_capability_is_refused() -> Result<usize, RuntimeGuardedAuditProjectionError> {
    let plan = plan_hash()?;
    let allow = execution_allow_decision(&plan);
    let barrier = execution_barrier_with_expiring_lease(plan, Timestamp(10));
    let stages = required_execution_stages();
    let op = promote_op();
    let guarded = GuardedExecutor::new(MarkerExecutor);

    match guarded.call(guarded_request(
        &barrier,
        &stages,
        std::slice::from_ref(&allow),
        &op,
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
pub fn verify_projection_without_authz() -> Result<usize, RuntimeGuardedAuditProjectionError> {
    let action = action_id();
    let plan = plan_hash()?;
    match project_release_status(&action, &plan, ACTOR_REF, &[]) {
        Err(ProjectionReadError::Unauthorized { stage, reason })
            if stage == MAY_PROJECT_STAGE && reason == AuthzDenyReason::Missing =>
        {
            Ok(1)
        }
        result => Err(unexpected("projection authz control", &result)),
    }
}

/// Negative control: duplicate audit ids are rejected and do not emit telemetry.
#[must_use = "the control result carries verification failures"]
pub fn verify_duplicate_audit_event_has_no_span(
) -> Result<usize, RuntimeGuardedAuditProjectionError> {
    let plan = plan_hash()?;
    let action = action_id();
    let event = runtime_event(
        "evt_runtime_duplicate_control",
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
            if event_id == AuditEventId("evt_runtime_duplicate_control".to_owned())
                && audit.audit_log().events().len() == 1
                && audit.trace_sink().spans.len() == 1 =>
        {
            Ok(1)
        }
        other => Err(unexpected("duplicate audit id control", &other)),
    }
}

fn append_runtime_trace(
    audit: &mut TraceProjectingAuditLog<InMemoryAuditLog, InMemoryTraceSink>,
    action: &ActionId,
    plan: &PlanHash,
    barrier: &ExecutionBarrier,
    execution_allow: &AuthzDecisionRef,
    projection_allow: &AuthzDecisionRef,
) -> Result<(), AuditAdapterError> {
    let events = vec![
        runtime_event(
            "evt_runtime_barrier",
            action,
            AuditEventKind::ExecutionBarrierLogged,
            plan,
            Timestamp(10),
        )
        .with_execution_barrier(barrier.clone())
        .with_impact_set_hash(impact_hash()),
        runtime_event(
            "evt_runtime_authz_execution",
            action,
            AuditEventKind::AuthzDecisionRecorded,
            plan,
            Timestamp(9),
        )
        .with_authz_decision(execution_allow.clone()),
        runtime_event(
            "evt_runtime_execution_started",
            action,
            AuditEventKind::ExecutionStarted,
            plan,
            Timestamp(11),
        )
        .with_causation_id(AuditEventId("evt_runtime_barrier".to_owned())),
        runtime_event(
            "evt_runtime_execution_completed",
            action,
            AuditEventKind::ExecutionCompleted,
            plan,
            Timestamp(12),
        )
        .with_causation_id(AuditEventId("evt_runtime_execution_started".to_owned())),
        runtime_event(
            "evt_runtime_authz_projection",
            action,
            AuditEventKind::AuthzDecisionRecorded,
            plan,
            Timestamp(13),
        )
        .with_authz_decision(projection_allow.clone()),
        runtime_event(
            "evt_runtime_projection_emitted",
            action,
            AuditEventKind::ProjectionEmitted,
            plan,
            Timestamp(14),
        )
        .with_causation_id(AuditEventId("evt_runtime_execution_completed".to_owned())),
        runtime_event(
            "evt_runtime_lifecycle_closed",
            action,
            AuditEventKind::LifecycleClosed,
            plan,
            Timestamp(15),
        )
        .with_causation_id(AuditEventId("evt_runtime_projection_emitted".to_owned())),
    ];
    AuditLogPort::append_batch(audit, events)?;
    Ok(())
}

fn project_release_status(
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
        now: Timestamp(20),
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

struct MarkerExecutor;

impl ExecutorPort for MarkerExecutor {
    type Error = Infallible;

    fn execute(
        &self,
        _op: &Op,
        _capability: &ExecutionCapability,
    ) -> Result<Vec<String>, Self::Error> {
        Ok(vec![EXECUTION_REF.to_owned()])
    }
}

fn execution_barrier(plan: PlanHash) -> ExecutionBarrier {
    ExecutionBarrier {
        barrier_id: AuditEventId("evt_runtime_barrier".to_owned()),
        action_id: action_id(),
        plan_hash: plan.clone(),
        op_indexes: vec![0],
        impact_set_hash: impact_hash(),
        witnesses: Vec::new(),
        leases: vec![lease_ref(plan)],
        authz_decision_refs: Vec::new(),
        constraint_snapshot_id: None,
    }
}

fn execution_barrier_with_expiring_lease(
    plan: PlanHash,
    expires_at: Timestamp,
) -> ExecutionBarrier {
    let mut barrier = execution_barrier(plan);
    if let Some(lease) = barrier.leases.first_mut() {
        lease.expires_at = Some(expires_at);
    }
    barrier
}

fn lease_ref(plan: PlanHash) -> LeaseRef {
    LeaseRef {
        lease_id: LeaseId("lease-runtime-release".to_owned()),
        resource: ResourceId("resource://runtime/release".to_owned()),
        scope: Scope("release/runtime".to_owned()),
        mode: ClaimMode::ExclusiveWrite,
        amount: 1,
        holder_action_id: action_id(),
        holder_plan_hash: plan,
        holder_op_index: Some(0),
        epoch: ConstraintEpoch(0),
        expires_at: None,
        lease_event_id: AuditEventId("evt_runtime_lease_granted".to_owned()),
    }
}

fn promote_op() -> Op {
    Op {
        index: 0,
        kind: "promote".to_owned(),
        effect: EffectSignature {
            reads: Vec::new(),
            writes: vec![Scope("release/runtime".to_owned())],
            produces: vec![EXECUTION_REF.to_owned()],
            requires: Vec::new(),
            invalidates: Vec::new(),
            conflict_domains: Vec::new(),
            hardness: ImpactHardness::Hard,
        },
    }
}

fn execution_allow_decision(plan: &PlanHash) -> AuthzDecisionRef {
    authz_decision(
        "evt_runtime_authz_execution",
        EXECUTION_STAGE,
        ACTOR_REF,
        plan,
    )
}

fn projection_allow_decision(plan: &PlanHash) -> AuthzDecisionRef {
    authz_decision(
        "evt_runtime_authz_projection",
        MAY_PROJECT_STAGE,
        ACTOR_REF,
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
        expires_at: Some(Timestamp(100)),
        attestation: None,
    }
}

fn runtime_event(
    event_id: &str,
    action: &ActionId,
    kind: AuditEventKind,
    plan: &PlanHash,
    occurred_at: Timestamp,
) -> AuditEvent {
    AuditEvent::new(AuditEventId(event_id.to_owned()), action.clone(), kind)
        .with_plan_hash(plan.clone())
        .with_correlation_id(CorrelationId(CORRELATION_ID.to_owned()))
        .with_occurred_at(occurred_at)
}

fn projection_fields() -> Vec<FieldPath> {
    vec![
        field("release.status"),
        field("release.window"),
        field("release.operator_token"),
    ]
}

fn redaction_policy() -> RedactionPolicy {
    RedactionPolicy {
        revealable: [field("release.status"), field("release.window")]
            .into_iter()
            .collect(),
    }
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

fn unexpected<T: fmt::Debug>(
    check: &'static str,
    actual: &T,
) -> RuntimeGuardedAuditProjectionError {
    RuntimeGuardedAuditProjectionError::UnexpectedOutcome {
        check,
        actual: format!("{actual:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        field, run_runtime_guarded_audit_projection, verify_duplicate_audit_event_has_no_span,
        verify_expired_capability_is_refused, verify_missing_execution_authz,
        verify_projection_without_authz, RuntimeGuardedAuditProjectionError,
    };

    #[test]
    fn runtime_guarded_audit_projection_summary_counts(
    ) -> Result<(), RuntimeGuardedAuditProjectionError> {
        let summary = run_runtime_guarded_audit_projection()?;

        assert_eq!(summary.executed_ops, 1);
        assert_eq!(summary.produced_refs, 1);
        assert_eq!(summary.audit_events, 7);
        assert_eq!(summary.trace_spans, 7);
        assert_eq!(summary.projected_fields, 3);
        assert_eq!(summary.redacted_fields, 1);
        assert_eq!(summary.negative_controls, 4);
        Ok(())
    }

    #[test]
    fn negative_controls_are_independently_observable(
    ) -> Result<(), RuntimeGuardedAuditProjectionError> {
        assert_eq!(verify_missing_execution_authz()?, 1);
        assert_eq!(verify_expired_capability_is_refused()?, 1);
        assert_eq!(verify_projection_without_authz()?, 1);
        assert_eq!(verify_duplicate_audit_event_has_no_span()?, 1);
        Ok(())
    }

    #[test]
    fn projection_redacts_operator_token() -> Result<(), RuntimeGuardedAuditProjectionError> {
        let summary = run_runtime_guarded_audit_projection()?;
        assert_eq!(summary.redacted_fields, 1);
        assert_eq!(field("release.operator_token").0, "release.operator_token");
        Ok(())
    }
}
