#![forbid(unsafe_code)]
#![deny(warnings)]

use std::fmt;

use causlane::core::ports::{AuditLogPort, HostDispatchPort, HostEffectHandler};
use causlane::core::protocol::{
    ActionId, AuditEvent, AuditEventId, AuditEventKind, AuthzDecision, AuthzDecisionRef,
    AuthzPolicy, CorrelationId, FieldPath, HostDispatchContext, HostDispatchError,
    HostDrainOutcome, HostEffectClass, HostEffectOutcome, HostRuntimeProfile, HostTaskSpec,
    PartitionKey, PartitionRoute, PlanHash, PlanHashError, PredicateId, ProjectionReadRequest,
    RedactionPolicy, RedactionView, Timestamp, CAUSLANE_HOST_API_VERSION, MAY_PROJECT_STAGE,
};
use causlane_runtime::adapters::audit::{AuditAdapterError, InMemoryAuditLog};
use causlane_runtime::linear_host::LinearHostDispatcher;
use causlane_runtime::projection_guard::{guard_projection_read, ProjectionReadError};

const PLAN_HASH: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const ACTION_ID: &str = "reference.release.publish";
const PREDICATE_ID: &str = "projection.reference_release_status";
const ACTOR_REF: &str = "actor://reference/api";
const CORRELATION_ID: &str = "corr-reference-integration-1";
const PROJECTION_POLICY: AuthzPolicy<'static> = AuthzPolicy {
    id: "reference.integration.projection",
    version: "1",
    max_age: Some(60),
};

/// Summary returned by the runnable reference integration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceIntegrationSummary {
    /// Host tasks accepted through the API seam.
    pub submitted_tasks: usize,
    /// Host tasks executed by the worker seam.
    pub executed_tasks: usize,
    /// Events appended through the runtime audit adapter.
    pub audit_events: usize,
    /// Projection fields classified by the guarded projection read.
    pub projected_fields: usize,
    /// Projection fields redacted by default.
    pub redacted_fields: usize,
}

/// Error type for the reference integration example.
#[derive(Debug)]
pub enum ReferenceIntegrationError {
    /// A static plan hash embedded in the example was malformed.
    PlanHash(PlanHashError),
    /// Host dispatch validation, admission or execution failed.
    Dispatch(HostDispatchError),
    /// Runtime audit append failed.
    Audit(AuditAdapterError),
    /// Guarded projection read failed.
    Projection(ProjectionReadError),
    /// The deterministic worker did not return the expected drain outcome.
    UnexpectedDrain {
        /// Drain step being checked.
        step: &'static str,
        /// Actual drain outcome.
        outcome: HostDrainOutcome,
    },
}

impl fmt::Display for ReferenceIntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlanHash(error) => write!(f, "invalid static plan hash: {error:?}"),
            Self::Dispatch(error) => write!(f, "host dispatch failed: {error:?}"),
            Self::Audit(error) => write!(f, "audit append failed: {error}"),
            Self::Projection(error) => write!(f, "projection guard failed: {error:?}"),
            Self::UnexpectedDrain { step, outcome } => {
                write!(f, "unexpected drain outcome during {step}: {outcome:?}")
            }
        }
    }
}

impl std::error::Error for ReferenceIntegrationError {}

impl From<PlanHashError> for ReferenceIntegrationError {
    fn from(error: PlanHashError) -> Self {
        Self::PlanHash(error)
    }
}

impl From<HostDispatchError> for ReferenceIntegrationError {
    fn from(error: HostDispatchError) -> Self {
        Self::Dispatch(error)
    }
}

impl From<AuditAdapterError> for ReferenceIntegrationError {
    fn from(error: AuditAdapterError) -> Self {
        Self::Audit(error)
    }
}

impl From<ProjectionReadError> for ReferenceIntegrationError {
    fn from(error: ProjectionReadError) -> Self {
        Self::Projection(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReferenceIntegrationTrace {
    submitted_task_ids: Vec<String>,
    executed_task_ids: Vec<String>,
    audit_events: Vec<AuditEvent>,
    projection_view: RedactionView,
}

#[derive(Default)]
struct ReferenceWorker {
    executed_task_ids: Vec<String>,
}

impl HostEffectHandler for ReferenceWorker {
    fn execute_host_effect(
        &mut self,
        ctx: &HostDispatchContext,
        task: &HostTaskSpec,
    ) -> Result<HostEffectOutcome, HostDispatchError> {
        self.executed_task_ids.push(task.task_id.clone());
        Ok(HostEffectOutcome {
            produced_refs: vec![format!(
                "object://reference/{}/{}",
                ctx.correlation_id, task.task_id
            )],
        })
    }
}

/// Run the API+worker+audit+projection reference integration.
///
/// # Errors
/// Returns an error if any public seam rejects the sample flow.
#[must_use = "the runnable example result must be checked"]
pub fn run_reference_integration() -> Result<ReferenceIntegrationSummary, ReferenceIntegrationError>
{
    let trace = build_reference_integration()?;
    Ok(ReferenceIntegrationSummary {
        submitted_tasks: trace.submitted_task_ids.len(),
        executed_tasks: trace.executed_task_ids.len(),
        audit_events: trace.audit_events.len(),
        projected_fields: trace.projection_view.revealed.len()
            + trace.projection_view.redacted.len(),
        redacted_fields: trace.projection_view.redacted.len(),
    })
}

fn build_reference_integration() -> Result<ReferenceIntegrationTrace, ReferenceIntegrationError> {
    let ctx = host_context();
    let action = action_id();
    let plan = plan_hash()?;
    let mut audit = InMemoryAuditLog::default();
    let mut dispatcher = LinearHostDispatcher::new();

    let child_ticket = dispatcher.submit(
        &ctx,
        host_task(
            "worker.project-release",
            vec!["api.accept-release".to_owned()],
            HostEffectClass::ReadOnly,
            "idem-worker-project",
            &plan,
        ),
    )?;
    let root_ticket = dispatcher.submit(
        &ctx,
        host_task(
            "api.accept-release",
            Vec::new(),
            HostEffectClass::SoftWrite,
            "idem-api-accept",
            &plan,
        ),
    )?;

    append_event(
        &mut audit,
        event(
            "evt_reference_action_admitted",
            &action,
            AuditEventKind::ActionAdmitted,
            &ctx,
            &plan,
            1,
        ),
    )?;
    append_event(
        &mut audit,
        event(
            "evt_reference_dispatch_logged",
            &action,
            AuditEventKind::DispatchLogged,
            &ctx,
            &plan,
            2,
        ),
    )?;

    let mut worker = ReferenceWorker::default();
    drain_expected(
        "api.accept-release",
        &mut dispatcher,
        &ctx,
        &mut worker,
        "drain API task",
    )?;
    append_execution_pair(&mut audit, &action, &ctx, &plan, "api_accept", 3)?;

    drain_expected(
        "worker.project-release",
        &mut dispatcher,
        &ctx,
        &mut worker,
        "drain worker task",
    )?;
    append_execution_pair(&mut audit, &action, &ctx, &plan, "worker_project", 5)?;

    if let outcome @ (HostDrainOutcome::Executed { .. } | HostDrainOutcome::Blocked) =
        dispatcher.drain_once(&ctx, &mut worker)?
    {
        return Err(ReferenceIntegrationError::UnexpectedDrain {
            step: "verify queue idle",
            outcome,
        });
    }

    let decision = allow_projection_decision(&action, &plan, ACTOR_REF);
    append_event(
        &mut audit,
        event(
            "evt_reference_projection_authz",
            &action,
            AuditEventKind::AuthzDecisionRecorded,
            &ctx,
            &plan,
            7,
        )
        .with_authz_decision(decision.clone()),
    )?;
    let projection_view = project_release_status(&action, &plan, ACTOR_REF, &[decision])?;
    append_event(
        &mut audit,
        event(
            "evt_reference_projection_emitted",
            &action,
            AuditEventKind::ProjectionEmitted,
            &ctx,
            &plan,
            8,
        ),
    )?;
    append_event(
        &mut audit,
        event(
            "evt_reference_lifecycle_closed",
            &action,
            AuditEventKind::LifecycleClosed,
            &ctx,
            &plan,
            9,
        ),
    )?;

    Ok(ReferenceIntegrationTrace {
        submitted_task_ids: vec![child_ticket.task_id, root_ticket.task_id],
        executed_task_ids: worker.executed_task_ids,
        audit_events: audit.events().to_vec(),
        projection_view,
    })
}

fn host_context() -> HostDispatchContext {
    HostDispatchContext {
        actor_ref: ACTOR_REF.to_owned(),
        trace_id: "trace-reference-integration-1".to_owned(),
        correlation_id: CORRELATION_ID.to_owned(),
        request_id: Some("req-reference-integration-1".to_owned()),
        config_snapshot_ref: "config://reference-integration/snapshot-1".to_owned(),
        idempotency_key: Some("idem-reference-request".to_owned()),
        runtime_profile: HostRuntimeProfile::LinearOnly,
        created_at: Timestamp(1),
    }
}

fn action_id() -> ActionId {
    ActionId(ACTION_ID.to_owned())
}

fn plan_hash() -> Result<PlanHash, PlanHashError> {
    PlanHash::new(PLAN_HASH)
}

fn host_task(
    task_id: &str,
    dependencies: Vec<String>,
    effect_class: HostEffectClass,
    idempotency_key: &str,
    plan: &PlanHash,
) -> HostTaskSpec {
    HostTaskSpec {
        task_id: task_id.to_owned(),
        action_id: action_id(),
        predicate_id: PredicateId(PREDICATE_ID.to_owned()),
        subject_ref: "subject://release/reference-1".to_owned(),
        plan_hash: Some(plan.clone()),
        effect_class,
        payload_ref: Some(format!("object://payload/{task_id}")),
        dependencies,
        idempotency_key: Some(idempotency_key.to_owned()),
        partition_route: PartitionRoute::for_primary(PartitionKey("tenant:reference".to_owned())),
        host_api_version: CAUSLANE_HOST_API_VERSION.to_owned(),
    }
}

fn event(
    event_id: &str,
    action: &ActionId,
    kind: AuditEventKind,
    ctx: &HostDispatchContext,
    plan: &PlanHash,
    occurred_at: u64,
) -> AuditEvent {
    AuditEvent::new(AuditEventId(event_id.to_owned()), action.clone(), kind)
        .with_plan_hash(plan.clone())
        .with_correlation_id(CorrelationId(ctx.correlation_id.clone()))
        .with_occurred_at(Timestamp(occurred_at))
}

fn append_event(
    audit: &mut InMemoryAuditLog,
    event: AuditEvent,
) -> Result<AuditEventId, ReferenceIntegrationError> {
    Ok(audit.append(event)?)
}

fn append_execution_pair(
    audit: &mut InMemoryAuditLog,
    action: &ActionId,
    ctx: &HostDispatchContext,
    plan: &PlanHash,
    event_prefix: &str,
    first_timestamp: u64,
) -> Result<(), ReferenceIntegrationError> {
    append_event(
        audit,
        event(
            &format!("evt_reference_{event_prefix}_started"),
            action,
            AuditEventKind::ExecutionStarted,
            ctx,
            plan,
            first_timestamp,
        ),
    )?;
    append_event(
        audit,
        event(
            &format!("evt_reference_{event_prefix}_completed"),
            action,
            AuditEventKind::ExecutionCompleted,
            ctx,
            plan,
            first_timestamp + 1,
        ),
    )?;
    Ok(())
}

fn drain_expected(
    expected_task_id: &str,
    dispatcher: &mut LinearHostDispatcher,
    ctx: &HostDispatchContext,
    worker: &mut ReferenceWorker,
    step: &'static str,
) -> Result<(), ReferenceIntegrationError> {
    match dispatcher.drain_once(ctx, worker)? {
        HostDrainOutcome::Executed { task_id, .. } if task_id == expected_task_id => Ok(()),
        outcome => Err(ReferenceIntegrationError::UnexpectedDrain { step, outcome }),
    }
}

fn allow_projection_decision(action: &ActionId, plan: &PlanHash, actor: &str) -> AuthzDecisionRef {
    AuthzDecisionRef {
        decision_event_id: AuditEventId("evt_reference_projection_authz".to_owned()),
        action_id: action.clone(),
        plan_hash: plan.clone(),
        predicate_id: PREDICATE_ID.to_owned(),
        actor: actor.to_owned(),
        stage: MAY_PROJECT_STAGE.to_owned(),
        decision: AuthzDecision::Allow,
        policy_id: PROJECTION_POLICY.id.to_owned(),
        policy_version: PROJECTION_POLICY.version.to_owned(),
        issued_at: Timestamp(7),
        expires_at: Some(Timestamp(67)),
        attestation: None,
    }
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
        policy: PROJECTION_POLICY,
        now: Timestamp(8),
    };
    guard_projection_read(decisions, &req, &redaction_policy(), &fields)
}

fn projection_fields() -> Vec<FieldPath> {
    vec![
        field("release.status"),
        field("release.window"),
        field("operator.token"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use causlane::core::protocol::AuthzDenyReason;

    #[test]
    fn reference_integration_runs_api_worker_audit_projection(
    ) -> Result<(), ReferenceIntegrationError> {
        assert_eq!(
            run_reference_integration()?,
            ReferenceIntegrationSummary {
                submitted_tasks: 2,
                executed_tasks: 2,
                audit_events: 9,
                projected_fields: 3,
                redacted_fields: 1,
            }
        );
        Ok(())
    }

    #[test]
    fn worker_executes_dependency_order() -> Result<(), ReferenceIntegrationError> {
        let trace = build_reference_integration()?;
        assert_eq!(
            trace.executed_task_ids,
            vec![
                "api.accept-release".to_owned(),
                "worker.project-release".to_owned()
            ]
        );
        Ok(())
    }

    #[test]
    fn audit_indexes_are_monotonic_and_cover_flow() -> Result<(), ReferenceIntegrationError> {
        let trace = build_reference_integration()?;
        let indexes: Vec<_> = trace
            .audit_events
            .iter()
            .map(|event| event.event_index)
            .collect();
        assert_eq!(
            indexes,
            (0..trace.audit_events.len() as u64)
                .map(Some)
                .collect::<Vec<_>>()
        );

        let kinds: Vec<_> = trace.audit_events.iter().map(|event| event.kind).collect();
        for required in [
            AuditEventKind::ActionAdmitted,
            AuditEventKind::DispatchLogged,
            AuditEventKind::ExecutionStarted,
            AuditEventKind::ExecutionCompleted,
            AuditEventKind::AuthzDecisionRecorded,
            AuditEventKind::ProjectionEmitted,
            AuditEventKind::LifecycleClosed,
        ] {
            assert!(kinds.contains(&required));
        }
        Ok(())
    }

    #[test]
    fn projection_redacts_unlisted_fields() -> Result<(), ReferenceIntegrationError> {
        let trace = build_reference_integration()?;
        assert_eq!(
            trace.projection_view.revealed,
            [field("release.status"), field("release.window")]
                .into_iter()
                .collect()
        );
        assert_eq!(
            trace.projection_view.redacted,
            [field("operator.token")].into_iter().collect()
        );
        Ok(())
    }

    #[test]
    fn read_without_may_project_allow_is_denied() -> Result<(), ReferenceIntegrationError> {
        let action = action_id();
        let plan = plan_hash()?;
        let result = project_release_status(&action, &plan, ACTOR_REF, &[]);
        assert_eq!(
            result,
            Err(ProjectionReadError::Unauthorized {
                stage: MAY_PROJECT_STAGE.to_owned(),
                reason: AuthzDenyReason::Missing,
            })
        );
        Ok(())
    }

    #[test]
    fn duplicate_idempotency_key_is_suppressed() -> Result<(), ReferenceIntegrationError> {
        let ctx = host_context();
        let plan = plan_hash()?;
        let mut dispatcher = LinearHostDispatcher::new();

        let _ticket = dispatcher.submit(
            &ctx,
            host_task(
                "api.accept-release",
                Vec::new(),
                HostEffectClass::SoftWrite,
                "idem-shared",
                &plan,
            ),
        )?;
        let duplicate = dispatcher.submit(
            &ctx,
            host_task(
                "api.accept-release-duplicate",
                Vec::new(),
                HostEffectClass::SoftWrite,
                "idem-shared",
                &plan,
            ),
        );

        assert_eq!(
            duplicate,
            Err(HostDispatchError::DuplicateSuppressed {
                task_id: "api.accept-release-duplicate".to_owned(),
            })
        );
        Ok(())
    }
}
