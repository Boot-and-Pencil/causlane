#![forbid(unsafe_code)]
#![deny(warnings)]

use std::fmt;

use causlane::core::ports::{AuditLogPort, HostDispatchPort, HostEffectHandler};
use causlane::core::protocol::{
    ActionId, AuditEvent, AuditEventId, AuditEventKind, CorrelationId, HostDispatchContext,
    HostDispatchError, HostDrainOutcome, HostEffectClass, HostEffectOutcome, HostRuntimeProfile,
    HostTaskSpec, PartitionKey, PartitionRoute, PlanHash, PlanHashError, PredicateId, Timestamp,
    CAUSLANE_HOST_API_VERSION,
};
use causlane_runtime::adapters::audit::{AuditAdapterError, InMemoryAuditLog};
use causlane_runtime::linear_host::LinearHostDispatcher;

const PLAN_HASH: &str = "sha256:3333333333333333333333333333333333333333333333333333333333333333";
const ACTION_ID: &str = "reference.release.orchestrate";
const PREDICATE_ID: &str = "release.orchestration.step";
const ACTOR_REF: &str = "actor://release/orchestrator";
const CORRELATION_ID: &str = "corr-release-orchestration-1";
const PACKAGES: [&str; 8] = [
    "causlane-core",
    "causlane-formal",
    "causlane-contracts",
    "causlane-runtime",
    "causlane-replay",
    "causlane-codegen",
    "causlane",
    "causlane-cli",
];
const RELEASE_TASKS: [ReleaseTaskId; 7] = [
    ReleaseTaskId::CiFmt,
    ReleaseTaskId::CiCheck,
    ReleaseTaskId::CiClippy,
    ReleaseTaskId::CiTest,
    ReleaseTaskId::PackageFileListReview,
    ReleaseTaskId::PublishDryRunPlan,
    ReleaseTaskId::DownstreamSmokePlan,
];
#[cfg(test)]
const PUBLISH_UPLOAD_TASK_ID: &str = "publish.upload";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReleaseTaskId {
    CiFmt,
    CiCheck,
    CiClippy,
    CiTest,
    PackageFileListReview,
    PublishDryRunPlan,
    DownstreamSmokePlan,
}

impl ReleaseTaskId {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CiFmt => "ci.fmt",
            Self::CiCheck => "ci.check",
            Self::CiClippy => "ci.clippy",
            Self::CiTest => "ci.test",
            Self::PackageFileListReview => "package.file-list-review",
            Self::PublishDryRunPlan => "publish.dry-run-plan",
            Self::DownstreamSmokePlan => "downstream.smoke-plan",
        }
    }

    fn dependencies(self) -> Vec<String> {
        match self {
            Self::CiFmt => Vec::new(),
            Self::CiCheck => vec![Self::CiFmt.as_str().to_owned()],
            Self::CiClippy => vec![Self::CiCheck.as_str().to_owned()],
            Self::CiTest => vec![Self::CiClippy.as_str().to_owned()],
            Self::PackageFileListReview => vec![Self::CiTest.as_str().to_owned()],
            Self::PublishDryRunPlan => vec![Self::PackageFileListReview.as_str().to_owned()],
            Self::DownstreamSmokePlan => vec![Self::PublishDryRunPlan.as_str().to_owned()],
        }
    }
}

/// Summary returned by the runnable release-orchestration example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseOrchestrationSummary {
    /// Host tasks accepted through the release orchestration seam.
    pub submitted_tasks: usize,
    /// Host tasks executed by the deterministic release worker.
    pub executed_tasks: usize,
    /// Events appended through the runtime audit adapter.
    pub audit_events: usize,
    /// Package file lists represented by the review step.
    pub reviewed_packages: usize,
    /// Package dry-runs represented by the dry-run planning step.
    pub dry_run_packages: usize,
}

/// Error type for the release orchestration example.
#[derive(Debug)]
pub enum ReleaseOrchestrationError {
    /// A static plan hash embedded in the example was malformed.
    PlanHash(PlanHashError),
    /// Host dispatch validation, admission or execution failed.
    Dispatch(HostDispatchError),
    /// Runtime audit append failed.
    Audit(AuditAdapterError),
    /// The deterministic worker did not return the expected drain outcome.
    UnexpectedDrain {
        /// Drain step being checked.
        step: &'static str,
        /// Actual drain outcome.
        outcome: HostDrainOutcome,
    },
}

impl fmt::Display for ReleaseOrchestrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlanHash(error) => write!(f, "invalid static plan hash: {error:?}"),
            Self::Dispatch(error) => write!(f, "host dispatch failed: {error:?}"),
            Self::Audit(error) => write!(f, "audit append failed: {error}"),
            Self::UnexpectedDrain { step, outcome } => {
                write!(f, "unexpected drain outcome during {step}: {outcome:?}")
            }
        }
    }
}

impl std::error::Error for ReleaseOrchestrationError {}

impl From<PlanHashError> for ReleaseOrchestrationError {
    fn from(error: PlanHashError) -> Self {
        Self::PlanHash(error)
    }
}

impl From<HostDispatchError> for ReleaseOrchestrationError {
    fn from(error: HostDispatchError) -> Self {
        Self::Dispatch(error)
    }
}

impl From<AuditAdapterError> for ReleaseOrchestrationError {
    fn from(error: AuditAdapterError) -> Self {
        Self::Audit(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReleaseOrchestrationTrace {
    submitted_task_ids: Vec<String>,
    executed_task_ids: Vec<String>,
    audit_events: Vec<AuditEvent>,
    reviewed_packages: Vec<String>,
    dry_run_packages: Vec<String>,
}

#[derive(Default)]
struct ReleaseWorker {
    executed_task_ids: Vec<String>,
    reviewed_packages: Vec<String>,
    dry_run_packages: Vec<String>,
}

impl HostEffectHandler for ReleaseWorker {
    fn execute_host_effect(
        &mut self,
        ctx: &HostDispatchContext,
        task: &HostTaskSpec,
    ) -> Result<HostEffectOutcome, HostDispatchError> {
        self.executed_task_ids.push(task.task_id.clone());
        if task.task_id == ReleaseTaskId::PackageFileListReview.as_str() {
            self.reviewed_packages = package_names();
        }
        if task.task_id == ReleaseTaskId::PublishDryRunPlan.as_str() {
            self.dry_run_packages = package_names();
        }
        Ok(HostEffectOutcome {
            produced_refs: vec![format!("release://{}/{}", ctx.correlation_id, task.task_id)],
        })
    }
}

/// Run the bounded CI/CD release orchestration example.
///
/// # Errors
/// Returns an error if any public seam rejects the sample flow.
#[must_use = "the runnable example result must be checked"]
pub fn run_release_orchestration() -> Result<ReleaseOrchestrationSummary, ReleaseOrchestrationError>
{
    let trace = build_release_orchestration()?;
    Ok(ReleaseOrchestrationSummary {
        submitted_tasks: trace.submitted_task_ids.len(),
        executed_tasks: trace.executed_task_ids.len(),
        audit_events: trace.audit_events.len(),
        reviewed_packages: trace.reviewed_packages.len(),
        dry_run_packages: trace.dry_run_packages.len(),
    })
}

fn build_release_orchestration() -> Result<ReleaseOrchestrationTrace, ReleaseOrchestrationError> {
    let ctx = host_context();
    let action = action_id();
    let plan = plan_hash()?;
    let mut audit = InMemoryAuditLog::default();
    let mut dispatcher = LinearHostDispatcher::new();

    let mut submitted_task_ids = Vec::new();
    for task in RELEASE_TASKS.iter().rev() {
        let ticket = dispatcher.submit(&ctx, release_task(*task, &plan))?;
        submitted_task_ids.push(ticket.task_id);
    }

    append_event(
        &mut audit,
        event(
            "evt_release_orchestration_admitted",
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
            "evt_release_orchestration_dispatch",
            &action,
            AuditEventKind::DispatchLogged,
            &ctx,
            &plan,
            2,
        ),
    )?;

    let mut worker = ReleaseWorker::default();
    for (index, task) in RELEASE_TASKS.iter().enumerate() {
        drain_expected(
            *task,
            &mut dispatcher,
            &ctx,
            &mut worker,
            "drain release task",
        )?;
        append_execution_pair(
            &mut audit,
            &action,
            &ctx,
            &plan,
            *task,
            3 + (index as u64 * 2),
        )?;
    }

    if let outcome @ (HostDrainOutcome::Executed { .. } | HostDrainOutcome::Blocked) =
        dispatcher.drain_once(&ctx, &mut worker)?
    {
        return Err(ReleaseOrchestrationError::UnexpectedDrain {
            step: "verify release queue idle",
            outcome,
        });
    }

    append_event(
        &mut audit,
        event(
            "evt_release_orchestration_closed",
            &action,
            AuditEventKind::LifecycleClosed,
            &ctx,
            &plan,
            17,
        ),
    )?;

    Ok(ReleaseOrchestrationTrace {
        submitted_task_ids,
        executed_task_ids: worker.executed_task_ids,
        audit_events: audit.events().to_vec(),
        reviewed_packages: worker.reviewed_packages,
        dry_run_packages: worker.dry_run_packages,
    })
}

fn host_context() -> HostDispatchContext {
    HostDispatchContext {
        actor_ref: ACTOR_REF.to_owned(),
        trace_id: "trace-release-orchestration-1".to_owned(),
        correlation_id: CORRELATION_ID.to_owned(),
        request_id: Some("req-release-orchestration-1".to_owned()),
        config_snapshot_ref: "config://release-orchestration/snapshot-1".to_owned(),
        idempotency_key: Some("idem-release-request".to_owned()),
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

fn release_task(task: ReleaseTaskId, plan: &PlanHash) -> HostTaskSpec {
    host_task(
        task.as_str(),
        task.dependencies(),
        HostEffectClass::ReadOnly,
        &format!("idem-{}", task.as_str()),
        plan,
    )
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
        subject_ref: "subject://release/workspace-0.0.1".to_owned(),
        plan_hash: Some(plan.clone()),
        effect_class,
        payload_ref: Some(format!("object://release/{task_id}")),
        dependencies,
        idempotency_key: Some(idempotency_key.to_owned()),
        partition_route: PartitionRoute::for_primary(PartitionKey("release:workspace".to_owned())),
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
) -> Result<AuditEventId, ReleaseOrchestrationError> {
    Ok(audit.append(event)?)
}

fn append_execution_pair(
    audit: &mut InMemoryAuditLog,
    action: &ActionId,
    ctx: &HostDispatchContext,
    plan: &PlanHash,
    task: ReleaseTaskId,
    first_timestamp: u64,
) -> Result<(), ReleaseOrchestrationError> {
    let event_prefix = task.as_str().replace(['.', '-'], "_");
    append_event(
        audit,
        event(
            &format!("evt_release_{event_prefix}_started"),
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
            &format!("evt_release_{event_prefix}_completed"),
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
    expected_task: ReleaseTaskId,
    dispatcher: &mut LinearHostDispatcher,
    ctx: &HostDispatchContext,
    worker: &mut ReleaseWorker,
    step: &'static str,
) -> Result<(), ReleaseOrchestrationError> {
    match dispatcher.drain_once(ctx, worker)? {
        HostDrainOutcome::Executed { task_id, .. } if task_id == expected_task.as_str() => Ok(()),
        outcome => Err(ReleaseOrchestrationError::UnexpectedDrain { step, outcome }),
    }
}

fn package_names() -> Vec<String> {
    PACKAGES
        .iter()
        .map(|package| (*package).to_owned())
        .collect()
}

#[cfg(test)]
fn expected_task_order() -> Vec<String> {
    RELEASE_TASKS
        .iter()
        .map(|task| task.as_str().to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_orchestration_runs_ci_release_graph() -> Result<(), ReleaseOrchestrationError> {
        assert_eq!(
            run_release_orchestration()?,
            ReleaseOrchestrationSummary {
                submitted_tasks: RELEASE_TASKS.len(),
                executed_tasks: RELEASE_TASKS.len(),
                audit_events: 17,
                reviewed_packages: PACKAGES.len(),
                dry_run_packages: PACKAGES.len(),
            }
        );
        Ok(())
    }

    #[test]
    fn release_tasks_execute_in_dependency_order() -> Result<(), ReleaseOrchestrationError> {
        let trace = build_release_orchestration()?;
        assert_eq!(trace.executed_task_ids, expected_task_order());
        Ok(())
    }

    #[test]
    fn review_and_dry_run_wait_for_ci_prerequisites() -> Result<(), ReleaseOrchestrationError> {
        let ctx = host_context();
        let plan = plan_hash()?;
        let mut dispatcher = LinearHostDispatcher::new();
        let mut worker = ReleaseWorker::default();

        let _review = dispatcher.submit(
            &ctx,
            host_task(
                ReleaseTaskId::PackageFileListReview.as_str(),
                vec![ReleaseTaskId::CiTest.as_str().to_owned()],
                HostEffectClass::ReadOnly,
                "idem-review",
                &plan,
            ),
        )?;
        let _dry_run = dispatcher.submit(
            &ctx,
            host_task(
                ReleaseTaskId::PublishDryRunPlan.as_str(),
                vec![ReleaseTaskId::PackageFileListReview.as_str().to_owned()],
                HostEffectClass::ReadOnly,
                "idem-dry-run",
                &plan,
            ),
        )?;

        assert_eq!(
            dispatcher.drain_once(&ctx, &mut worker)?,
            HostDrainOutcome::Blocked
        );
        assert!(worker.executed_task_ids.is_empty());
        Ok(())
    }

    #[test]
    fn audit_events_cover_release_gate_sequence() -> Result<(), ReleaseOrchestrationError> {
        let trace = build_release_orchestration()?;
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

        let started = trace
            .audit_events
            .iter()
            .filter(|event| event.kind == AuditEventKind::ExecutionStarted)
            .count();
        let completed = trace
            .audit_events
            .iter()
            .filter(|event| event.kind == AuditEventKind::ExecutionCompleted)
            .count();
        assert_eq!(started, RELEASE_TASKS.len());
        assert_eq!(completed, RELEASE_TASKS.len());
        assert_eq!(
            trace.audit_events.first().map(|event| event.kind),
            Some(AuditEventKind::ActionAdmitted)
        );
        assert_eq!(
            trace.audit_events.last().map(|event| event.kind),
            Some(AuditEventKind::LifecycleClosed)
        );
        Ok(())
    }

    #[test]
    fn duplicate_release_attempt_is_suppressed() -> Result<(), ReleaseOrchestrationError> {
        let ctx = host_context();
        let plan = plan_hash()?;
        let mut dispatcher = LinearHostDispatcher::new();

        let _first = dispatcher.submit(
            &ctx,
            host_task(
                ReleaseTaskId::CiFmt.as_str(),
                Vec::new(),
                HostEffectClass::ReadOnly,
                "idem-release-attempt",
                &plan,
            ),
        )?;
        let duplicate_task_id = format!("{}.duplicate", ReleaseTaskId::CiFmt.as_str());
        let duplicate = dispatcher.submit(
            &ctx,
            host_task(
                &duplicate_task_id,
                Vec::new(),
                HostEffectClass::ReadOnly,
                "idem-release-attempt",
                &plan,
            ),
        );

        assert_eq!(
            duplicate,
            Err(HostDispatchError::DuplicateSuppressed {
                task_id: duplicate_task_id,
            })
        );
        Ok(())
    }

    #[test]
    fn forbidden_publish_upload_is_rejected() -> Result<(), ReleaseOrchestrationError> {
        let ctx = host_context();
        let plan = plan_hash()?;
        let mut dispatcher = LinearHostDispatcher::new();
        let upload = dispatcher.submit(
            &ctx,
            host_task(
                PUBLISH_UPLOAD_TASK_ID,
                vec![ReleaseTaskId::PublishDryRunPlan.as_str().to_owned()],
                HostEffectClass::Forbidden,
                "idem-forbidden-upload",
                &plan,
            ),
        );

        assert_eq!(
            upload,
            Err(HostDispatchError::ForbiddenEffect {
                task_id: PUBLISH_UPLOAD_TASK_ID.to_owned(),
            })
        );
        Ok(())
    }
}
