//! Lowering helpers for replay trace DTOs.

use causlane_core::{
    ActionId, AuditEventId, AuthzAttestation, AuthzDecisionRef, CapabilityAttestation,
    CapabilityId, ConstraintEpoch, ConstraintId, ExecutionBarrier, ExecutionCapability, FactKind,
    ImpactSetHash, LeaseId, LeaseRef, PlanHash, ResourceId, Scope, Timestamp, TruthAnchor,
    WitnessBinding, WitnessRef,
};

use crate::trace::{
    ReplayAnchor, ReplayAuthzDecision, ReplayEvent, ReplayExecutionBarrier,
    ReplayExecutionCapability, ReplayLeaseRef,
};
use crate::ReplayError;

#[must_use = "hash validation errors must be handled"]
pub(crate) fn impact_hash(field: &str, value: &str) -> Result<ImpactSetHash, ReplayError> {
    validate_hash_token(field, value)?;
    Ok(ImpactSetHash(value.to_owned()))
}

#[must_use = "anchor lowering errors must be handled"]
pub(crate) fn anchors_for(
    raw: &ReplayEvent,
    event_id: &str,
    trace_plan: Option<&String>,
) -> Result<Vec<TruthAnchor>, ReplayError> {
    let mut anchors = Vec::with_capacity(raw.anchors.len());
    for anchor in &raw.anchors {
        anchors.push(anchor_for(anchor, raw, event_id, trace_plan)?);
    }
    Ok(anchors)
}

#[must_use = "witness lowering errors must be handled"]
pub(crate) fn witness_ref(raw: &crate::trace::ReplayWitnessRef) -> Result<WitnessRef, ReplayError> {
    let binds_to = match &raw.binds_to {
        Some(binding) => {
            let impact_set_hash = match &binding.impact_set_hash {
                Some(value) => Some(impact_hash("witness.binds_to.impact_set_hash", value)?),
                None => None,
            };
            Some(WitnessBinding {
                action_id: ActionId(binding.action_id.clone()),
                plan_hash: PlanHash::new(binding.plan_hash.clone())?,
                impact_set_hash,
            })
        }
        None => None,
    };
    Ok(WitnessRef {
        event_id: AuditEventId(raw.event_id.clone()),
        requirement_id: raw.requirement_id.clone(),
        kind: raw.kind.to_core(),
        fact_kind: raw.fact_kind.clone().map(FactKind),
        scope: raw.scope.clone().map(Scope),
        binds_to,
    })
}

#[must_use = "lease lowering errors must be handled"]
pub(crate) fn lease_ref(
    raw: &ReplayLeaseRef,
    event_id: &str,
    event_action: &str,
    event_plan: Option<&String>,
    trace_plan: Option<&String>,
) -> Result<LeaseRef, ReplayError> {
    let holder_plan = match &raw.holder_plan_hash {
        Some(value) => PlanHash::new(value.clone())?,
        None => plan_hash_for(event_id, event_plan, trace_plan)?,
    };
    Ok(LeaseRef {
        lease_id: LeaseId(raw.lease_id.clone()),
        resource: ResourceId(raw.resource.clone()),
        scope: Scope(raw.scope.clone()),
        mode: raw.mode.to_core(),
        amount: raw.amount,
        holder_action_id: ActionId(
            raw.holder_action_id
                .clone()
                .unwrap_or_else(|| event_action.to_owned()),
        ),
        holder_plan_hash: holder_plan,
        holder_op_index: raw.holder_op_index,
        epoch: ConstraintEpoch(raw.epoch),
        expires_at: raw.expires_at.map(Timestamp),
        lease_event_id: AuditEventId(
            raw.lease_event_id
                .clone()
                .unwrap_or_else(|| event_id.to_owned()),
        ),
    })
}

#[must_use = "barrier lowering errors must be handled"]
pub(crate) fn execution_barrier(
    raw: &ReplayExecutionBarrier,
    event_id: &str,
    event_action: &str,
    event_plan: Option<&String>,
    trace_plan: Option<&String>,
) -> Result<ExecutionBarrier, ReplayError> {
    let plan_hash = match &raw.plan_hash {
        Some(value) => PlanHash::new(value.clone())?,
        None => plan_hash_for(event_id, event_plan, trace_plan)?,
    };
    let witnesses = raw
        .witnesses
        .iter()
        .map(witness_ref)
        .collect::<Result<Vec<_>, _>>()?;
    let leases = raw
        .leases
        .iter()
        .map(|lease| lease_ref(lease, event_id, event_action, event_plan, trace_plan))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ExecutionBarrier {
        barrier_id: AuditEventId(
            raw.barrier_id
                .clone()
                .unwrap_or_else(|| event_id.to_owned()),
        ),
        action_id: ActionId(
            raw.action_id
                .clone()
                .unwrap_or_else(|| event_action.to_owned()),
        ),
        plan_hash,
        op_indexes: raw.op_indexes.clone(),
        impact_set_hash: impact_hash("execution_barrier.impact_set_hash", &raw.impact_set_hash)?,
        witnesses,
        leases,
        authz_decision_refs: raw
            .authz_decision_refs
            .iter()
            .cloned()
            .map(AuditEventId)
            .collect(),
        constraint_snapshot_id: raw.constraint_snapshot_id.clone().map(ConstraintId),
    })
}

#[must_use = "authz decision lowering errors must be handled"]
pub(crate) fn authz_decision(
    raw: &ReplayAuthzDecision,
    event_id: &str,
    event_action: &str,
    event_plan: Option<&String>,
    trace_plan: Option<&String>,
) -> Result<AuthzDecisionRef, ReplayError> {
    let plan_hash = match &raw.plan_hash {
        Some(value) => PlanHash::new(value.clone())?,
        None => plan_hash_for(event_id, event_plan, trace_plan)?,
    };
    Ok(AuthzDecisionRef {
        decision_event_id: AuditEventId(
            raw.decision_event_id
                .clone()
                .unwrap_or_else(|| event_id.to_owned()),
        ),
        action_id: ActionId(
            raw.action_id
                .clone()
                .unwrap_or_else(|| event_action.to_owned()),
        ),
        plan_hash,
        predicate_id: raw.predicate_id.clone(),
        actor: raw.actor.clone(),
        stage: raw.stage.clone(),
        decision: raw.decision.to_core(),
        policy_id: raw.policy_id.clone(),
        policy_version: raw.policy_version.clone(),
        issued_at: Timestamp(raw.issued_at),
        expires_at: raw.expires_at.map(Timestamp),
        attestation: raw.attestation.clone().map(AuthzAttestation),
    })
}

#[must_use = "execution capability lowering errors must be handled"]
pub(crate) fn execution_capability(
    raw: &ReplayExecutionCapability,
    event_id: &str,
    event_action: &str,
    event_plan: Option<&String>,
    trace_plan: Option<&String>,
) -> Result<ExecutionCapability, ReplayError> {
    let plan_hash = match &raw.plan_hash {
        Some(value) => PlanHash::new(value.clone())?,
        None => plan_hash_for(event_id, event_plan, trace_plan)?,
    };
    Ok(ExecutionCapability {
        capability_id: CapabilityId(raw.capability_id.clone()),
        action_id: ActionId(
            raw.action_id
                .clone()
                .unwrap_or_else(|| event_action.to_owned()),
        ),
        plan_hash,
        op_index: raw.op_index,
        barrier_event_id: AuditEventId(raw.barrier_event_id.clone()),
        lease_ids: raw.lease_ids.iter().cloned().map(LeaseId).collect(),
        expires_at: raw.expires_at.map(Timestamp),
        attestation: raw.attestation.clone().map(CapabilityAttestation),
    })
}

fn anchor_for(
    anchor: &ReplayAnchor,
    raw: &ReplayEvent,
    event_id: &str,
    trace_plan: Option<&String>,
) -> Result<TruthAnchor, ReplayError> {
    let anchor_plan = anchor
        .plan_hash
        .clone()
        .or_else(|| raw.plan_hash.clone())
        .or_else(|| trace_plan.cloned())
        .ok_or_else(|| ReplayError::UnresolvedAnchorPlanHash {
            event_id: event_id.to_owned(),
        })?;
    let anchor_action = anchor
        .action_id
        .clone()
        .unwrap_or_else(|| raw.action_id.clone());
    Ok(TruthAnchor {
        event_id: AuditEventId(anchor.event_id.clone()),
        action_id: ActionId(anchor_action),
        plan_hash: PlanHash::new(anchor_plan)?,
        fact_kind: anchor.fact_kind.clone().map(FactKind),
        scope: anchor.scope.clone().map(Scope),
        event_hash: None,
    })
}

fn validate_hash_token(field: &str, value: &str) -> Result<(), ReplayError> {
    if causlane_contracts::is_canonical_sha256_token(value) {
        Ok(())
    } else {
        Err(ReplayError::BadImpactSetHash {
            field: field.to_owned(),
        })
    }
}

fn plan_hash_for(
    event_id: &str,
    raw_plan: Option<&String>,
    trace_plan: Option<&String>,
) -> Result<PlanHash, ReplayError> {
    let plan = raw_plan
        .cloned()
        .or_else(|| trace_plan.cloned())
        .ok_or_else(|| ReplayError::UnresolvedAnchorPlanHash {
            event_id: event_id.to_owned(),
        })?;
    Ok(PlanHash::new(plan)?)
}

#[must_use]
pub(crate) fn default_amount() -> u64 {
    1
}
