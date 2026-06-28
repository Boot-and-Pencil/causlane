#![forbid(unsafe_code)]
#![deny(warnings)]

use std::convert::Infallible;
use std::fmt;

use causlane::core::kernel::{CapabilityIssuer, DispatchAdmission};
use causlane::core::protocol::{
    ConflictDomain, CorrelationId, EffectSignature, ExecutionCapability, ExecutionCapabilityError,
    ImpactHardness, LeaseTableError, PlanHashError, ResourceClaim, Timestamp, TruthAnchor,
    WitnessAttestation,
};
use causlane::prelude::{
    admit_call, requires_execution_barrier, ActionCall, ActionId, ActionPlan, AuditEvent,
    AuditEventId, AuditEventKind, AuditLogPort, ClaimMode, ConsequenceProfile, ConstraintEpoch,
    ExecutionBarrier, ExecutorPort, FactKind, ImpactSetHash, KernelContracts, LeaseId, LeaseRef,
    LeaseTable, Op, PlanHash, PredicateId, ResourceId, Scope,
};

const PLAN_DIGEST: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const IMPACT_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// Summary returned by the runnable example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimpleLocalSummary {
    /// Number of audit events replay-verified by the example.
    pub event_count: usize,
    /// Produced references returned by the local executor.
    pub produced_refs: Vec<String>,
}

/// Error type for the local example composition.
#[derive(Debug)]
pub enum SimpleLocalError {
    /// A static plan hash embedded in the example was malformed.
    PlanHash(PlanHashError),
    /// The admitted call was not accepted by the kernel.
    AdmissionRejected,
    /// The runtime-execution plan unexpectedly did not require a barrier.
    BarrierNotRequired,
    /// The local plan had no operation to execute.
    MissingPlanOp,
    /// Local lease-table validation failed.
    Lease(LeaseTableError),
    /// Capability derivation failed.
    Capability(ExecutionCapabilityError),
    /// Replay rejected the local audit trail.
    Replay(causlane_replay::ReplayError),
}

impl fmt::Display for SimpleLocalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlanHash(error) => write!(f, "invalid static plan hash: {error:?}"),
            Self::AdmissionRejected => f.write_str("local action admission was rejected"),
            Self::BarrierNotRequired => {
                f.write_str("runtime-execution plan did not require a barrier")
            }
            Self::MissingPlanOp => f.write_str("local plan had no operation to execute"),
            Self::Lease(error) => write!(f, "lease validation failed: {error:?}"),
            Self::Capability(error) => write!(f, "capability derivation failed: {error:?}"),
            Self::Replay(error) => write!(f, "replay verification failed: {error}"),
        }
    }
}

impl std::error::Error for SimpleLocalError {}

impl From<PlanHashError> for SimpleLocalError {
    fn from(error: PlanHashError) -> Self {
        Self::PlanHash(error)
    }
}

impl From<LeaseTableError> for SimpleLocalError {
    fn from(error: LeaseTableError) -> Self {
        Self::Lease(error)
    }
}

impl From<ExecutionCapabilityError> for SimpleLocalError {
    fn from(error: ExecutionCapabilityError) -> Self {
        Self::Capability(error)
    }
}

impl From<causlane_replay::ReplayError> for SimpleLocalError {
    fn from(error: causlane_replay::ReplayError) -> Self {
        Self::Replay(error)
    }
}

impl From<Infallible> for SimpleLocalError {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

#[derive(Default)]
struct InMemoryAudit {
    events: Vec<AuditEvent>,
}

impl InMemoryAudit {
    fn events(self) -> Vec<AuditEvent> {
        self.events
    }
}

impl AuditLogPort for InMemoryAudit {
    type Error = Infallible;

    fn append_batch(&mut self, events: Vec<AuditEvent>) -> Result<Vec<AuditEventId>, Self::Error> {
        let ids = events.iter().map(|event| event.event_id.clone()).collect();
        self.events.extend(events);
        Ok(ids)
    }

    fn append(&mut self, event: AuditEvent) -> Result<AuditEventId, Self::Error> {
        let id = event.event_id.clone();
        self.events.push(event);
        Ok(id)
    }
}

struct NoopExecutor;

impl ExecutorPort for NoopExecutor {
    type Error = Infallible;

    fn execute(
        &self,
        op: &Op,
        _capability: &ExecutionCapability,
    ) -> Result<Vec<String>, Self::Error> {
        Ok(vec![format!("produced:{}:ok", op.kind)])
    }
}

/// Build the local audit trail used by the example.
///
/// # Errors
/// Returns an error if local kernel checks reject the sample flow.
#[must_use = "example event construction can fail and must be checked"]
pub fn simple_local_events() -> Result<Vec<AuditEvent>, SimpleLocalError> {
    let (events, _produced_refs) = build_simple_local()?;
    Ok(events)
}

/// Run the local example and verify its audit trail through replay.
///
/// # Errors
/// Returns an error if local composition or replay verification fails.
#[must_use = "the runnable example result must be checked"]
pub fn run_simple_local() -> Result<SimpleLocalSummary, SimpleLocalError> {
    let (events, produced_refs) = build_simple_local()?;
    causlane_replay::verify_events(&events)?;
    Ok(SimpleLocalSummary {
        event_count: events.len(),
        produced_refs,
    })
}

fn build_simple_local() -> Result<(Vec<AuditEvent>, Vec<String>), SimpleLocalError> {
    let mut audit = InMemoryAudit::default();
    let call = action_call();
    let action_id = call.action_id.clone();
    let plan = action_plan(&call)?;
    let plan_hash = plan.plan_hash.clone();
    let impact_set_hash = impact_set_hash();
    let claim = resource_claim();
    let lease = lease_ref(&action_id, &plan_hash, &claim);
    let barrier = execution_barrier(&action_id, &plan_hash, &impact_set_hash, lease.clone());

    match admit_call(&call) {
        DispatchAdmission::Accepted {
            action_id: accepted,
        } if accepted == action_id => {}
        DispatchAdmission::Accepted { .. }
        | DispatchAdmission::Waiting { .. }
        | DispatchAdmission::Rejected { .. } => return Err(SimpleLocalError::AdmissionRejected),
    }
    if !requires_execution_barrier(plan.consequence_profile) {
        return Err(SimpleLocalError::BarrierNotRequired);
    }

    let mut leases = LeaseTable::new();
    leases.grant(lease.clone(), &KernelContracts)?;
    leases.validate_claim_coverage(&action_id, &plan_hash, &[claim])?;
    let capability = KernelContracts.derive_capability(&barrier, 0)?;
    let first_op = plan.ops.first().ok_or(SimpleLocalError::MissingPlanOp)?;
    let produced_refs = NoopExecutor.execute(first_op, &capability)?;

    append(
        &mut audit,
        event("evt_admitted", &action_id, AuditEventKind::ActionAdmitted),
    )?;
    append(
        &mut audit,
        event("evt_planned", &action_id, AuditEventKind::ActionPlanned)
            .with_plan_hash(plan_hash.clone()),
    )?;
    append(
        &mut audit,
        event("evt_dispatch", &action_id, AuditEventKind::DispatchLogged)
            .with_plan_hash(plan_hash.clone()),
    )?;
    append(
        &mut audit,
        event(
            "evt_lease_granted",
            &action_id,
            AuditEventKind::ConstraintLeaseGranted,
        )
        .with_plan_hash(plan_hash.clone())
        .with_leases(vec![lease]),
    )?;
    append(
        &mut audit,
        event(
            "evt_barrier",
            &action_id,
            AuditEventKind::ExecutionBarrierLogged,
        )
        .with_plan_hash(plan_hash.clone())
        .with_execution_barrier(barrier),
    )?;
    append(
        &mut audit,
        event("evt_started", &action_id, AuditEventKind::ExecutionStarted)
            .with_plan_hash(plan_hash.clone())
            .with_execution_capability(capability),
    )?;
    append(
        &mut audit,
        event(
            "evt_completed",
            &action_id,
            AuditEventKind::ExecutionCompleted,
        )
        .with_plan_hash(plan_hash.clone()),
    )?;
    append(
        &mut audit,
        event(
            "evt_observed_truth",
            &action_id,
            AuditEventKind::ObservedTruthCommitted,
        )
        .with_plan_hash(plan_hash.clone())
        .with_attested_fact(WitnessAttestation {
            fact_kind: FactKind("release_candidate_promoted".to_owned()),
            scope: Scope("environment:staging".to_owned()),
        }),
    )?;
    append(
        &mut audit,
        event(
            "evt_projection",
            &action_id,
            AuditEventKind::ProjectionEmitted,
        )
        .with_plan_hash(plan_hash.clone())
        .with_anchors(vec![TruthAnchor {
            event_id: AuditEventId("evt_observed_truth".to_owned()),
            action_id: action_id.clone(),
            plan_hash: plan_hash.clone(),
            fact_kind: Some(FactKind("release_candidate_promoted".to_owned())),
            scope: Some(Scope("environment:staging".to_owned())),
            event_hash: None,
        }]),
    )?;
    append(
        &mut audit,
        event("evt_closed", &action_id, AuditEventKind::LifecycleClosed).with_plan_hash(plan_hash),
    )?;

    Ok((audit.events(), produced_refs))
}

fn append(audit: &mut InMemoryAudit, event: AuditEvent) -> Result<(), SimpleLocalError> {
    let _id = audit.append(event)?;
    Ok(())
}

fn event(id: &str, action_id: &ActionId, kind: AuditEventKind) -> AuditEvent {
    AuditEvent::new(AuditEventId(id.to_owned()), action_id.clone(), kind)
        .with_correlation_id(CorrelationId("simple-local-run".to_owned()))
        .with_occurred_at(Timestamp(1))
}

fn action_call() -> ActionCall {
    ActionCall {
        action_id: ActionId("release.promote".to_owned()),
        predicate: PredicateId("release_candidate".to_owned()),
        subject_ref: "release_candidate:rc-1".to_owned(),
        circumstance_ref: "environment:staging".to_owned(),
        correlation_id: CorrelationId("simple-local-run".to_owned()),
    }
}

fn action_plan(call: &ActionCall) -> Result<ActionPlan, SimpleLocalError> {
    Ok(ActionPlan {
        action_id: call.action_id.clone(),
        predicate: call.predicate.clone(),
        plan_hash: plan_hash()?,
        consequence_profile: ConsequenceProfile::RuntimeExecution,
        ops: vec![Op {
            index: 0,
            kind: "promote_release_candidate".to_owned(),
            effect: EffectSignature {
                reads: vec![Scope(call.subject_ref.clone())],
                writes: vec![Scope(call.circumstance_ref.clone())],
                produces: vec!["release_candidate_promoted".to_owned()],
                requires: Vec::new(),
                invalidates: Vec::new(),
                conflict_domains: vec![ConflictDomain("environment".to_owned())],
                hardness: ImpactHardness::Hard,
            },
        }],
    })
}

fn plan_hash() -> Result<PlanHash, PlanHashError> {
    PlanHash::new(format!("sha256:{PLAN_DIGEST}"))
}

fn impact_set_hash() -> ImpactSetHash {
    ImpactSetHash(format!("sha256:{IMPACT_DIGEST}"))
}

fn resource_claim() -> ResourceClaim {
    ResourceClaim {
        resource: ResourceId("environment".to_owned()),
        scope: Scope("environment:staging".to_owned()),
        mode: ClaimMode::ExclusiveWrite,
        amount: 1,
    }
}

fn lease_ref(action_id: &ActionId, plan_hash: &PlanHash, claim: &ResourceClaim) -> LeaseRef {
    LeaseRef {
        lease_id: LeaseId("lease_environment_staging".to_owned()),
        resource: claim.resource.clone(),
        scope: claim.scope.clone(),
        mode: claim.mode,
        amount: claim.amount,
        holder_action_id: action_id.clone(),
        holder_plan_hash: plan_hash.clone(),
        holder_op_index: Some(0),
        epoch: ConstraintEpoch(1),
        expires_at: None,
        lease_event_id: AuditEventId("evt_lease_granted".to_owned()),
    }
}

fn execution_barrier(
    action_id: &ActionId,
    plan_hash: &PlanHash,
    impact_set_hash: &ImpactSetHash,
    lease: LeaseRef,
) -> ExecutionBarrier {
    ExecutionBarrier {
        barrier_id: AuditEventId("evt_barrier".to_owned()),
        action_id: action_id.clone(),
        plan_hash: plan_hash.clone(),
        op_indexes: vec![0],
        impact_set_hash: impact_set_hash.clone(),
        witnesses: Vec::new(),
        leases: vec![lease],
        authz_decision_refs: Vec::new(),
        constraint_snapshot_id: None,
    }
}
