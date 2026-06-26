//! JSON trace DTOs and lowering into core audit events.

use serde::{Deserialize, Serialize};

use causlane_contracts::ClaimModeDto;
use causlane_contracts::{canonical_json_hash, TemplateBindings};
use causlane_core::{
    ActionId, AuditEvent, AuditEventId, AuditEventKind, AuthzDecision, FactKind, PlanHash, Scope,
    Timestamp, WitnessAttestation, WitnessKind,
};

use crate::trace_lowering::{
    anchors_for, authz_decision, default_amount, execution_barrier, execution_capability,
    impact_hash, lease_ref, witness_ref,
};
use crate::{verify_events, ReplayError};

/// Boundary form of [`AuditEventKind`]; serde owns the dotted-token mapping so
/// no hand-written string matching is needed (and unknown tokens fail closed).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKindDto {
    /// `action.admitted`
    #[serde(rename = "action.admitted")]
    ActionAdmitted,
    /// `action.planned`
    #[serde(rename = "action.planned")]
    ActionPlanned,
    /// `dispatch.logged`
    #[serde(rename = "dispatch.logged")]
    DispatchLogged,
    /// `execution.barrier_logged`
    #[serde(rename = "execution.barrier_logged")]
    ExecutionBarrierLogged,
    /// `execution.started`
    #[serde(rename = "execution.started")]
    ExecutionStarted,
    /// `execution.completed`
    #[serde(rename = "execution.completed")]
    ExecutionCompleted,
    /// `observed_truth.committed`
    #[serde(rename = "observed_truth.committed")]
    ObservedTruthCommitted,
    /// `projection.emitted`
    #[serde(rename = "projection.emitted")]
    ProjectionEmitted,
    /// `lifecycle.closed`
    #[serde(rename = "lifecycle.closed")]
    LifecycleClosed,
    /// `gate.approved`
    #[serde(rename = "gate.approved")]
    GateApproved,
    /// `gate.denied`
    #[serde(rename = "gate.denied")]
    GateDenied,
    /// `constraint.lease_granted`
    #[serde(rename = "constraint.lease_granted")]
    ConstraintLeaseGranted,
    /// `constraint.lease_released`
    #[serde(rename = "constraint.lease_released")]
    ConstraintLeaseReleased,
    /// `violation.detected`
    #[serde(rename = "violation.detected")]
    ViolationDetected,
    /// `authz.decision_recorded`
    #[serde(rename = "authz.decision_recorded")]
    AuthzDecisionRecorded,
    /// `drain.fence_requested`
    #[serde(rename = "drain.fence_requested")]
    DrainFenceRequested,
    /// `drain.fence_acquired`
    #[serde(rename = "drain.fence_acquired")]
    DrainFenceAcquired,
}

impl EventKindDto {
    /// Map to the pure-kernel event kind.
    #[must_use]
    pub fn to_core(self) -> AuditEventKind {
        match self {
            Self::ActionAdmitted => AuditEventKind::ActionAdmitted,
            Self::ActionPlanned => AuditEventKind::ActionPlanned,
            Self::DispatchLogged => AuditEventKind::DispatchLogged,
            Self::ExecutionBarrierLogged => AuditEventKind::ExecutionBarrierLogged,
            Self::ExecutionStarted => AuditEventKind::ExecutionStarted,
            Self::ExecutionCompleted => AuditEventKind::ExecutionCompleted,
            Self::ObservedTruthCommitted => AuditEventKind::ObservedTruthCommitted,
            Self::ProjectionEmitted => AuditEventKind::ProjectionEmitted,
            Self::LifecycleClosed => AuditEventKind::LifecycleClosed,
            Self::GateApproved => AuditEventKind::GateApproved,
            Self::GateDenied => AuditEventKind::GateDenied,
            Self::ConstraintLeaseGranted => AuditEventKind::ConstraintLeaseGranted,
            Self::ConstraintLeaseReleased => AuditEventKind::ConstraintLeaseReleased,
            Self::ViolationDetected => AuditEventKind::ViolationDetected,
            Self::AuthzDecisionRecorded => AuditEventKind::AuthzDecisionRecorded,
            Self::DrainFenceRequested => AuditEventKind::DrainFenceRequested,
            Self::DrainFenceAcquired => AuditEventKind::DrainFenceAcquired,
        }
    }
}

/// Boundary form of [`AuthzDecision`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthzDecisionDto {
    /// Allow decision.
    Allow,
    /// Deny decision.
    Deny,
}

impl AuthzDecisionDto {
    /// Convert to core decision.
    #[must_use]
    pub fn to_core(self) -> AuthzDecision {
        match self {
            Self::Allow => AuthzDecision::Allow,
            Self::Deny => AuthzDecision::Deny,
        }
    }
}

/// A projection truth anchor as written in a trace document (ADR-0010).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAnchor {
    /// The observed-truth event this projection is anchored to.
    pub event_id: String,
    /// Action of the anchored truth (defaults to the event's action).
    #[serde(default)]
    pub action_id: Option<String>,
    /// Plan hash of the anchored truth (defaults to the event/trace plan hash).
    #[serde(default)]
    pub plan_hash: Option<String>,
    /// Optional required fact kind.
    #[serde(default)]
    pub fact_kind: Option<String>,
    /// Optional restricting scope.
    #[serde(default)]
    pub scope: Option<String>,
}

/// Boundary form of [`WitnessKind`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessKindDto {
    /// Observed fact evidence.
    ObservedFact,
    /// Gate approval evidence.
    GateApproval,
    /// Authorization decision evidence.
    AuthzDecision,
    /// Constraint decision evidence.
    ConstraintDecision,
    /// External evidence.
    ExternalEvidence,
}

impl WitnessKindDto {
    /// Convert to core witness kind.
    #[must_use]
    pub fn to_core(self) -> WitnessKind {
        match self {
            Self::ObservedFact => WitnessKind::ObservedFact,
            Self::GateApproval => WitnessKind::GateApproval,
            Self::AuthzDecision => WitnessKind::AuthzDecision,
            Self::ConstraintDecision => WitnessKind::ConstraintDecision,
            Self::ExternalEvidence => WitnessKind::ExternalEvidence,
        }
    }
}

/// JSON witness binding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayWitnessBinding {
    /// Bound action id.
    pub action_id: String,
    /// Bound plan hash.
    pub plan_hash: String,
    /// Bound impact set hash, if required.
    #[serde(default)]
    pub impact_set_hash: Option<String>,
}

/// JSON typed witness ref.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayWitnessRef {
    /// Evidence event id.
    pub event_id: String,
    /// Bundle required-witness id.
    pub requirement_id: String,
    /// Evidence kind.
    pub kind: WitnessKindDto,
    /// Fact kind asserted by the witness.
    #[serde(default)]
    pub fact_kind: Option<String>,
    /// Scope asserted by the witness.
    #[serde(default)]
    pub scope: Option<String>,
    /// Optional action/plan/impact binding.
    #[serde(default)]
    pub binds_to: Option<ReplayWitnessBinding>,
}

/// JSON lease reference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLeaseRef {
    /// Lease id.
    pub lease_id: String,
    /// Resource id.
    pub resource: String,
    /// Scope.
    pub scope: String,
    /// Lease mode.
    pub mode: ClaimModeDto,
    /// Amount held.
    #[serde(default = "default_amount")]
    pub amount: u64,
    /// Holder action id; defaults to the event action id.
    #[serde(default)]
    pub holder_action_id: Option<String>,
    /// Holder plan hash; defaults to the event/trace plan hash.
    #[serde(default)]
    pub holder_plan_hash: Option<String>,
    /// Holder op index.
    #[serde(default)]
    pub holder_op_index: Option<u32>,
    /// Constraint epoch.
    #[serde(default)]
    pub epoch: u64,
    /// Expiry timestamp.
    #[serde(default)]
    pub expires_at: Option<u64>,
    /// Lease grant event id; defaults to the event id.
    #[serde(default)]
    pub lease_event_id: Option<String>,
}

/// JSON execution barrier payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayExecutionBarrier {
    /// Barrier id; defaults to the event id.
    #[serde(default)]
    pub barrier_id: Option<String>,
    /// Action id; defaults to the event action id.
    #[serde(default)]
    pub action_id: Option<String>,
    /// Plan hash; defaults to event/trace plan hash.
    #[serde(default)]
    pub plan_hash: Option<String>,
    /// Covered op indexes.
    #[serde(default)]
    pub op_indexes: Vec<u32>,
    /// Bound impact-set hash.
    pub impact_set_hash: String,
    /// Typed witnesses.
    #[serde(default)]
    pub witnesses: Vec<ReplayWitnessRef>,
    /// Lease refs.
    #[serde(default)]
    pub leases: Vec<ReplayLeaseRef>,
    /// Authz decision event refs.
    #[serde(default)]
    pub authz_decision_refs: Vec<String>,
    /// Constraint snapshot id.
    #[serde(default)]
    pub constraint_snapshot_id: Option<String>,
}

/// JSON authorization decision payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayAuthzDecision {
    /// Decision event id; defaults to the containing event id.
    #[serde(default)]
    pub decision_event_id: Option<String>,
    /// Action id; defaults to the containing event action id.
    #[serde(default)]
    pub action_id: Option<String>,
    /// Plan hash; defaults to the event/trace plan hash.
    #[serde(default)]
    pub plan_hash: Option<String>,
    /// Predicate id the decision applies to.
    pub predicate_id: String,
    /// Actor/principal evaluated by the policy.
    pub actor: String,
    /// Lifecycle stage the decision authorizes.
    pub stage: String,
    /// Allow/deny result.
    pub decision: AuthzDecisionDto,
    /// Stable policy id.
    pub policy_id: String,
    /// Stable policy version.
    pub policy_version: String,
    /// Issue timestamp.
    pub issued_at: u64,
    /// Expiry timestamp, if any.
    #[serde(default)]
    pub expires_at: Option<u64>,
    /// Optional keyed attestation (hex HMAC) minted by the PDP; verified by
    /// replay when configured with the PDP secret.
    #[serde(default)]
    pub attestation: Option<String>,
}

/// JSON execution capability payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayExecutionCapability {
    /// Capability id.
    pub capability_id: String,
    /// Action id; defaults to the containing event action id.
    #[serde(default)]
    pub action_id: Option<String>,
    /// Plan hash; defaults to the event/trace plan hash.
    #[serde(default)]
    pub plan_hash: Option<String>,
    /// Op index authorized by the capability.
    pub op_index: u32,
    /// Barrier event id this capability derives from.
    pub barrier_event_id: String,
    /// Lease ids covering this op.
    #[serde(default)]
    pub lease_ids: Vec<String>,
    /// Capability expiry, if any.
    #[serde(default)]
    pub expires_at: Option<u64>,
    /// Optional keyed attestation (hex HMAC) minted by the kernel; verified by
    /// replay when configured with the kernel secret.
    #[serde(default)]
    pub attestation: Option<String>,
}

/// One event in a trace document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEvent {
    /// Stable event id (synthesized from position if absent).
    #[serde(default)]
    pub event_id: Option<String>,
    /// Event kind (dotted token).
    pub kind: EventKindDto,
    /// Action this event belongs to.
    pub action_id: String,
    /// Plan hash carried by the event, if any.
    #[serde(default)]
    pub plan_hash: Option<String>,
    /// Causal witnesses (event-id references).
    #[serde(default)]
    pub witnesses: Vec<String>,
    /// Typed witness refs.
    #[serde(default)]
    pub witness_refs: Vec<ReplayWitnessRef>,
    /// Truth anchors (for projection events).
    #[serde(default)]
    pub anchors: Vec<ReplayAnchor>,
    /// Lease refs carried by lease grant/release or barrier events.
    #[serde(default)]
    pub leases: Vec<ReplayLeaseRef>,
    /// Planned impact-set hash bound to this event.
    #[serde(default)]
    pub impact_set_hash: Option<String>,
    /// Normalized barrier payload.
    #[serde(default)]
    pub execution_barrier: Option<ReplayExecutionBarrier>,
    /// Typed authz decision payload.
    #[serde(default)]
    pub authz_decision: Option<ReplayAuthzDecision>,
    /// Execution capability payload.
    #[serde(default)]
    pub execution_capability: Option<ReplayExecutionCapability>,
    /// Fact kind recorded by evidence/truth events.
    #[serde(default)]
    pub fact_kind: Option<String>,
    /// Scope recorded by evidence/truth events.
    #[serde(default)]
    pub scope: Option<String>,
    /// Wall-clock time this event occurred, when recorded. On a barrier event it
    /// is the evaluation time for authz freshness (P0-010).
    #[serde(default)]
    pub occurred_at: Option<u64>,
}

/// A trace document: an action's recorded protocol history.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayTrace {
    /// Trace schema/content version.
    pub trace_version: String,
    /// The action the trace is about.
    pub action_id: String,
    /// Declared compiled bundle hash. When present, bundle-bound replay checks
    /// it against the compiled bundle and fails with `BundleHashMismatch`.
    #[serde(default)]
    pub bundle_hash: Option<String>,
    /// Predicate id this trace exercises. Required for bundle-bound replay.
    #[serde(default)]
    pub predicate: Option<String>,
    /// The action's plan hash (default for events/anchors that omit it).
    #[serde(default)]
    pub plan_hash: Option<String>,
    /// Subject bindings used by selector templates.
    #[serde(default)]
    pub subject: Vec<ScenarioBinding>,
    /// Circumstance bindings used by selector templates.
    #[serde(default)]
    pub circumstance: Vec<ScenarioBinding>,
    /// Per-action predicate roster for mixed-predicate traces. Empty means every
    /// action uses the trace-level `predicate`.
    #[serde(default)]
    pub actions: Vec<ActionSpec>,
    /// The ordered events.
    pub events: Vec<ReplayEvent>,
}

/// Scenario expectation token.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedReplayResult {
    /// Scenario should pass replay.
    Pass,
    /// Scenario should fail replay.
    Fail,
}

/// Simple key/value binding used by scenarios without introducing dynamic JSON.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioBinding {
    /// Binding key.
    pub key: String,
    /// Binding value.
    pub value: String,
}

/// Per-action predicate declaration for a mixed-predicate trace.
///
/// A trace normally binds every action to the single trace-level `predicate`.
/// When a trace carries actions of different predicates — e.g. a
/// `RuntimeExecution` producer plus a `ProjectionRead` reader that anchors its
/// projection on the producer's observed truth — each non-primary action
/// declares its own predicate here so the lifecycle reducer resolves the right
/// consequence profile per action. Actions absent from this roster fall back to
/// the trace-level predicate, so single-predicate traces need no roster.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionSpec {
    /// The action this declaration is for.
    pub action_id: String,
    /// The predicate that action exercises (must exist in the bundle).
    pub predicate: String,
    /// Optional per-action plan hash (documentation/override; events still carry
    /// their own `plan_hash`).
    #[serde(default)]
    pub plan_hash: Option<String>,
}

/// YAML scenario document that can emit a trace fixture.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayScenario {
    /// Scenario schema/content version.
    pub scenario_version: String,
    /// Stable scenario id.
    pub scenario_id: String,
    /// Action id.
    pub action_id: String,
    /// Predicate id.
    pub predicate: String,
    /// Plan hash.
    pub plan_hash: String,
    /// Subject bindings used by templates/docs.
    #[serde(default)]
    pub subject: Vec<ScenarioBinding>,
    /// Circumstance bindings used by templates/docs.
    #[serde(default)]
    pub circumstance: Vec<ScenarioBinding>,
    /// Expected replay result.
    pub expected_replay_result: ExpectedReplayResult,
    /// Stable expected replay error code for negative scenarios.
    #[serde(default)]
    pub expected_error_code: Option<String>,
    /// Formal obligations exercised by this scenario.
    #[serde(default)]
    pub formal_obligations: Vec<String>,
    /// Per-action predicate roster for mixed-predicate traces. Empty means every
    /// action uses the trace-level `predicate`.
    #[serde(default)]
    pub actions: Vec<ActionSpec>,
    /// Ordered trace events.
    pub events: Vec<ReplayEvent>,
}

impl ReplayTrace {
    /// Parse a trace from a JSON string.
    ///
    /// # Errors
    /// Returns [`ReplayError::Decode`] if the JSON is invalid or carries an
    /// unknown event kind.
    #[must_use = "the parsed trace must be used"]
    pub fn from_json_str(json: &str) -> Result<Self, ReplayError> {
        serde_json::from_str(json).map_err(|err| ReplayError::Decode(err.to_string()))
    }

    /// Mint a valid keyed attestation into every `execution_capability` payload
    /// under `secret` (P1-006), so an attested replay (`--kernel-secret`) accepts
    /// the emitted trace. Used by `scenario emit-trace --kernel-secret` to produce
    /// a positive attested fixture without a separate minting tool.
    ///
    /// # Errors
    /// Returns [`ReplayError`] if a capability payload cannot be lowered.
    #[must_use = "minting can fail; the result must be checked"]
    pub fn mint_capability_attestations(&mut self, secret: &[u8]) -> Result<(), ReplayError> {
        let trace_plan = self.plan_hash.clone();
        for event in &mut self.events {
            let Some(raw) = event.execution_capability.clone() else {
                continue;
            };
            let event_id = event.event_id.clone().unwrap_or_default();
            let core = crate::trace_lowering::execution_capability(
                &raw,
                &event_id,
                &event.action_id,
                event.plan_hash.as_ref(),
                trace_plan.as_ref(),
            )?;
            let tag = causlane_contracts::attestation::attest(secret, &core.attestation_message());
            if let Some(capability) = &mut event.execution_capability {
                capability.attestation = Some(tag);
            }
        }
        Ok(())
    }

    /// Lower the trace into typed core [`AuditEvent`]s, validating every plan
    /// hash and resolving anchor defaults.
    ///
    /// # Errors
    /// Returns [`ReplayError::BadPlanHash`] for a malformed plan hash, or
    /// [`ReplayError::UnresolvedAnchorPlanHash`] when an anchor's plan hash
    /// cannot be defaulted.
    #[allow(clippy::too_many_lines)]
    #[must_use = "the lowered events must be used"]
    pub fn to_events(&self) -> Result<Vec<AuditEvent>, ReplayError> {
        let mut events = Vec::with_capacity(self.events.len());
        for (index, raw) in self.events.iter().enumerate() {
            let event_id = raw
                .event_id
                .clone()
                .unwrap_or_else(|| format!("{}#{index}", self.action_id));

            let plan_hash = match &raw.plan_hash {
                Some(value) => Some(PlanHash::new(value.clone())?),
                None => None,
            };

            let anchors = anchors_for(raw, &event_id, self.plan_hash.as_ref())?;
            let witness_refs = raw
                .witness_refs
                .iter()
                .map(witness_ref)
                .collect::<Result<Vec<_>, _>>()?;
            let leases = raw
                .leases
                .iter()
                .map(|lease| {
                    lease_ref(
                        lease,
                        &event_id,
                        &raw.action_id,
                        raw.plan_hash.as_ref(),
                        self.plan_hash.as_ref(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let event_id_value = event_id.clone();
            let mut event = AuditEvent::new(
                AuditEventId(event_id_value),
                ActionId(raw.action_id.clone()),
                raw.kind.to_core(),
            )
            .with_witnesses(raw.witnesses.iter().cloned().map(AuditEventId).collect())
            .with_witness_refs(witness_refs)
            .with_anchors(anchors)
            .with_leases(leases)
            .with_event_index(index as u64);
            if let Some(value) = &raw.impact_set_hash {
                event = event.with_impact_set_hash(impact_hash("event.impact_set_hash", value)?);
            }
            if let Some(raw_barrier) = &raw.execution_barrier {
                event = event.with_execution_barrier(execution_barrier(
                    raw_barrier,
                    &event_id,
                    &raw.action_id,
                    raw.plan_hash.as_ref(),
                    self.plan_hash.as_ref(),
                )?);
            }
            if let Some(raw_authz) = &raw.authz_decision {
                event = event.with_authz_decision(authz_decision(
                    raw_authz,
                    &event_id,
                    &raw.action_id,
                    raw.plan_hash.as_ref(),
                    self.plan_hash.as_ref(),
                )?);
            }
            if let Some(raw_capability) = &raw.execution_capability {
                event = event.with_execution_capability(execution_capability(
                    raw_capability,
                    &event_id,
                    &raw.action_id,
                    raw.plan_hash.as_ref(),
                    self.plan_hash.as_ref(),
                )?);
            }
            if matches!(
                raw.kind,
                EventKindDto::DrainFenceRequested | EventKindDto::DrainFenceAcquired
            ) {
                if let Some(scope) = &raw.scope {
                    event = event.with_drain_fence_scope(Scope(scope.clone()));
                }
            } else if let (Some(fact), Some(scope)) = (&raw.fact_kind, &raw.scope) {
                // A producer event that records both a fact_kind and a scope is
                // attesting its own truth (gate.approved / observed_truth.committed);
                // the oracle grounds witness refs / anchors against this (P0-004).
                event = event.with_attested_fact(WitnessAttestation {
                    fact_kind: FactKind(fact.clone()),
                    scope: Scope(scope.clone()),
                });
            }
            if let Some(hash) = plan_hash {
                event = event.with_plan_hash(hash);
            }
            if let Some(occurred_at) = raw.occurred_at {
                event = event.with_occurred_at(Timestamp(occurred_at));
            }
            events.push(event);
        }
        Ok(events)
    }

    /// Build exact selector-template bindings from this trace.
    #[must_use]
    pub fn template_bindings(&self) -> TemplateBindings {
        TemplateBindings::from_pairs(
            self.subject
                .iter()
                .map(|binding| (binding.key.clone(), binding.value.clone())),
            self.circumstance
                .iter()
                .map(|binding| (binding.key.clone(), binding.value.clone())),
        )
    }

    /// Load and verify a trace in one step.
    ///
    /// # Errors
    /// Any [`ReplayError`] from loading or verification.
    #[must_use = "the verification result must be used"]
    pub fn verify(&self) -> Result<(), ReplayError> {
        verify_events(&self.to_events()?)
    }
}

impl ReplayScenario {
    /// Parse a scenario from YAML.
    ///
    /// # Errors
    /// Returns [`ReplayError::Decode`] if the YAML does not match the scenario schema.
    #[must_use = "the parsed scenario must be used"]
    pub fn from_yaml_str(yaml: &str) -> Result<Self, ReplayError> {
        serde_yaml::from_str(yaml).map_err(|err| ReplayError::Decode(err.to_string()))
    }

    /// Compute the scenario hash over canonical typed scenario material.
    ///
    /// # Errors
    /// Returns [`ReplayError::Decode`] if the YAML does not match the scenario
    /// schema or canonical JSON serialization fails.
    #[must_use = "scenario hash errors must be handled"]
    pub fn scenario_hash(yaml: &str) -> Result<String, ReplayError> {
        let scenario = Self::from_yaml_str(yaml)?;
        canonical_json_hash(&scenario).map_err(|err| ReplayError::Decode(err.to_string()))
    }

    /// Convert this scenario into a replay trace.
    #[must_use]
    pub fn to_trace(&self) -> ReplayTrace {
        ReplayTrace {
            trace_version: self.scenario_version.clone(),
            action_id: self.action_id.clone(),
            bundle_hash: None,
            predicate: Some(self.predicate.clone()),
            plan_hash: Some(self.plan_hash.clone()),
            subject: self.subject.clone(),
            circumstance: self.circumstance.clone(),
            actions: self.actions.clone(),
            events: self.events.clone(),
        }
    }

    /// Convert this scenario into a replay trace bound to `bundle_hash`.
    ///
    /// When `bundle_hash` is `Some`, the emitted trace declares its binding to a
    /// specific compiled bundle, so strict verification
    /// ([`ReplayTrace::verify_with_bundle_strict`]) accepts it; when `None` the
    /// trace is left unbound, exactly as [`Self::to_trace`].
    #[must_use]
    pub fn to_trace_bound(&self, bundle_hash: Option<String>) -> ReplayTrace {
        ReplayTrace {
            bundle_hash,
            ..self.to_trace()
        }
    }

    /// Convert this scenario into pretty trace JSON.
    ///
    /// # Errors
    /// Returns [`ReplayError::Decode`] if serialization fails.
    #[must_use = "the generated trace JSON must be written or returned"]
    pub fn to_trace_json_pretty(&self) -> Result<String, ReplayError> {
        self.to_trace_json_pretty_bound(None)
    }

    /// Convert this scenario into pretty trace JSON bound to `bundle_hash`.
    ///
    /// # Errors
    /// Returns [`ReplayError::Decode`] if serialization fails.
    #[must_use = "the generated trace JSON must be written or returned"]
    pub fn to_trace_json_pretty_bound(
        &self,
        bundle_hash: Option<String>,
    ) -> Result<String, ReplayError> {
        serde_json::to_string_pretty(&self.to_trace_bound(bundle_hash))
            .map_err(|err| ReplayError::Decode(err.to_string()))
    }
}
