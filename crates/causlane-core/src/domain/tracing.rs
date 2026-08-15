//! Derived tracing model.
//!
//! Spans are a projection of causal protocol events. They are useful for observability but
//! never replace the causal protocol history as the authority for observed truth.

use super::{
    ActionId, AuthzDecision, CausalProtocolEvent, CausalProtocolEventId, CausalProtocolEventKind,
    CorrelationId, ImpactSetHash, PlanHash, Scope, Timestamp,
};

/// Stable span id derived from an causal protocol event id.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraceSpanId(pub String);

impl TraceSpanId {
    /// Build the canonical span id for a journal event.
    #[must_use]
    pub fn from_causal_protocol_event_id(event_id: &CausalProtocolEventId) -> Self {
        Self(format!("audit:{}", event_id.0))
    }
}

/// Coarse-grained category used by tracing exporters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TraceSpanKind {
    /// Admission into the action lifecycle.
    Admission,
    /// Plan compilation.
    Planning,
    /// Dispatch bookkeeping.
    Dispatch,
    /// Durable execution permission boundary.
    Barrier,
    /// Executor start/completion.
    Execution,
    /// Observed truth commit.
    Observation,
    /// Derived projection emission.
    Projection,
    /// Lifecycle closure.
    Lifecycle,
    /// Human/oversight gate decision.
    Gate,
    /// Constraint-plane lease event.
    Constraint,
    /// Policy authorization event.
    Authorization,
    /// Drain fence protocol event.
    Drain,
    /// Detected protocol or constraint violation.
    Violation,
}

/// Typed span attributes. Exporters may render these into their own wire shape,
/// but the semantic mapping lives here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceAttribute {
    /// Monotonic journal index, when the protocol-history partition supplied one.
    EventIndex(u64),
    /// Number of causal witness event ids attached to the journal event.
    WitnessCount(usize),
    /// Number of typed witness refs attached to the journal event.
    WitnessRefCount(usize),
    /// Number of truth anchors attached to a projection event.
    TruthAnchorCount(usize),
    /// Number of lease refs attached to the journal event.
    LeaseCount(usize),
    /// Planned impact set bound to the journal event.
    ImpactSetHash(ImpactSetHash),
    /// Drain fence scope carried by drain protocol events.
    DrainFenceScope(Scope),
    /// The event carried a normalized execution barrier payload.
    HasExecutionBarrier,
    /// The event carried an authorization decision payload.
    HasAuthzDecision,
    /// Allow/deny result inside the authorization decision payload.
    AuthzDecision(AuthzDecision),
    /// The event carried an execution capability payload.
    HasExecutionCapability,
    /// Operation index authorized by an execution capability.
    ExecutionCapabilityOpIndex(u32),
    /// Number of leases bound to an execution capability.
    ExecutionCapabilityLeaseCount(usize),
    /// The event carried an attested fact payload.
    HasAttestedFact,
}

/// A structured span derived from a single causal protocol event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceSpan {
    /// Span id derived from the causal protocol event id.
    pub span_id: TraceSpanId,
    /// Parent span derived from the causing causal protocol event, if any.
    pub parent_span_id: Option<TraceSpanId>,
    /// Trace id derived from the causal protocol correlation id.
    pub trace_id: CorrelationId,
    /// Action this span belongs to.
    pub action_id: ActionId,
    /// Source causal protocol event.
    pub event_id: CausalProtocolEventId,
    /// Source causal protocol event kind.
    pub event_kind: CausalProtocolEventKind,
    /// Coarse tracing category.
    pub span_kind: TraceSpanKind,
    /// Plan hash bound to the source event, if any.
    pub plan_hash: Option<PlanHash>,
    /// Event occurrence time, if the journal supplied one.
    pub occurred_at: Option<Timestamp>,
    /// Additional typed observability attributes.
    pub attributes: Vec<TraceAttribute>,
}

/// Classify an causal protocol event kind into a tracing span category.
#[must_use]
pub fn trace_span_kind_from_causal_protocol_event_kind(
    kind: CausalProtocolEventKind,
) -> TraceSpanKind {
    match kind {
        CausalProtocolEventKind::ActionAdmitted => TraceSpanKind::Admission,
        CausalProtocolEventKind::ActionPlanned => TraceSpanKind::Planning,
        CausalProtocolEventKind::DispatchLogged => TraceSpanKind::Dispatch,
        CausalProtocolEventKind::ExecutionBarrierLogged => TraceSpanKind::Barrier,
        CausalProtocolEventKind::ExecutionStarted | CausalProtocolEventKind::ExecutionCompleted => {
            TraceSpanKind::Execution
        }
        CausalProtocolEventKind::ObservedTruthCommitted => TraceSpanKind::Observation,
        CausalProtocolEventKind::ProjectionEmitted => TraceSpanKind::Projection,
        CausalProtocolEventKind::LifecycleClosed => TraceSpanKind::Lifecycle,
        CausalProtocolEventKind::GateApproved | CausalProtocolEventKind::GateDenied => {
            TraceSpanKind::Gate
        }
        CausalProtocolEventKind::ConstraintLeaseGranted
        | CausalProtocolEventKind::ConstraintLeaseReleased => TraceSpanKind::Constraint,
        CausalProtocolEventKind::ViolationDetected => TraceSpanKind::Violation,
        CausalProtocolEventKind::AuthzDecisionRecorded => TraceSpanKind::Authorization,
        CausalProtocolEventKind::DrainFenceRequested
        | CausalProtocolEventKind::DrainFenceAcquired => TraceSpanKind::Drain,
    }
}

/// Project one causal protocol event into one observability span.
#[must_use]
pub fn trace_span_from_causal_protocol_event(event: &CausalProtocolEvent) -> TraceSpan {
    TraceSpan {
        span_id: TraceSpanId::from_causal_protocol_event_id(&event.event_id),
        parent_span_id: event
            .causation_id
            .as_ref()
            .map(TraceSpanId::from_causal_protocol_event_id),
        trace_id: event.correlation_id.clone(),
        action_id: event.action_id.clone(),
        event_id: event.event_id.clone(),
        event_kind: event.kind,
        span_kind: trace_span_kind_from_causal_protocol_event_kind(event.kind),
        plan_hash: event.plan_hash.clone(),
        occurred_at: event.occurred_at,
        attributes: trace_attributes_from_causal_protocol_event(event),
    }
}

fn trace_attributes_from_causal_protocol_event(event: &CausalProtocolEvent) -> Vec<TraceAttribute> {
    let mut attributes = Vec::new();
    if let Some(event_index) = event.event_index {
        attributes.push(TraceAttribute::EventIndex(event_index));
    }
    if !event.witnesses.is_empty() {
        attributes.push(TraceAttribute::WitnessCount(event.witnesses.len()));
    }
    if !event.witness_refs.is_empty() {
        attributes.push(TraceAttribute::WitnessRefCount(event.witness_refs.len()));
    }
    if !event.anchors.is_empty() {
        attributes.push(TraceAttribute::TruthAnchorCount(event.anchors.len()));
    }
    if !event.leases.is_empty() {
        attributes.push(TraceAttribute::LeaseCount(event.leases.len()));
    }
    if let Some(impact_set_hash) = &event.impact_set_hash {
        attributes.push(TraceAttribute::ImpactSetHash(impact_set_hash.clone()));
    }
    if let Some(scope) = &event.drain_fence_scope {
        attributes.push(TraceAttribute::DrainFenceScope(scope.clone()));
    }
    if event.execution_barrier.is_some() {
        attributes.push(TraceAttribute::HasExecutionBarrier);
    }
    if let Some(authz_decision) = &event.authz_decision {
        attributes.push(TraceAttribute::HasAuthzDecision);
        attributes.push(TraceAttribute::AuthzDecision(authz_decision.decision));
    }
    if let Some(capability) = &event.execution_capability {
        attributes.push(TraceAttribute::HasExecutionCapability);
        attributes.push(TraceAttribute::ExecutionCapabilityOpIndex(
            capability.op_index,
        ));
        attributes.push(TraceAttribute::ExecutionCapabilityLeaseCount(
            capability.lease_ids.len(),
        ));
    }
    if event.attested_fact.is_some() {
        attributes.push(TraceAttribute::HasAttestedFact);
    }
    attributes
}

#[cfg(test)]
mod tests {
    use super::{
        trace_span_from_causal_protocol_event, trace_span_kind_from_causal_protocol_event_kind,
        TraceAttribute, TraceSpanId, TraceSpanKind,
    };
    use crate::{
        ActionId, AuthzDecision, AuthzDecisionRef, CausalProtocolEvent, CausalProtocolEventId,
        CausalProtocolEventKind, CorrelationId, ImpactSetHash, PlanHash, PlanHashError, Timestamp,
        ALL_CAUSAL_PROTOCOL_EVENT_KINDS,
    };

    fn event(kind: CausalProtocolEventKind) -> CausalProtocolEvent {
        CausalProtocolEvent::new(
            CausalProtocolEventId("event-1".to_owned()),
            ActionId("action-1".to_owned()),
            kind,
        )
    }

    fn plan_hash() -> Result<PlanHash, PlanHashError> {
        PlanHash::new(format!("sha256:{}", "1".repeat(PlanHash::DIGEST_LEN)))
    }

    fn authz_decision() -> Result<AuthzDecisionRef, PlanHashError> {
        Ok(AuthzDecisionRef {
            decision_event_id: CausalProtocolEventId("authz-1".to_owned()),
            action_id: ActionId("action-1".to_owned()),
            plan_hash: plan_hash()?,
            predicate_id: "predicate".to_owned(),
            actor: "alice".to_owned(),
            stage: "execution_started".to_owned(),
            decision: AuthzDecision::Allow,
            policy_id: "policy".to_owned(),
            policy_version: "1".to_owned(),
            issued_at: Timestamp(10),
            expires_at: None,
            attestation: None,
        })
    }

    #[test]
    fn every_causal_protocol_event_kind_has_a_span_kind() {
        for kind in ALL_CAUSAL_PROTOCOL_EVENT_KINDS {
            let span = trace_span_from_causal_protocol_event(&event(kind));
            assert_eq!(
                span.span_kind,
                trace_span_kind_from_causal_protocol_event_kind(kind),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn selected_event_kinds_keep_their_observability_categories() {
        let cases = [
            (
                CausalProtocolEventKind::ActionAdmitted,
                TraceSpanKind::Admission,
            ),
            (
                CausalProtocolEventKind::ActionPlanned,
                TraceSpanKind::Planning,
            ),
            (
                CausalProtocolEventKind::DispatchLogged,
                TraceSpanKind::Dispatch,
            ),
            (
                CausalProtocolEventKind::ExecutionBarrierLogged,
                TraceSpanKind::Barrier,
            ),
            (
                CausalProtocolEventKind::ExecutionStarted,
                TraceSpanKind::Execution,
            ),
            (
                CausalProtocolEventKind::ObservedTruthCommitted,
                TraceSpanKind::Observation,
            ),
            (
                CausalProtocolEventKind::ProjectionEmitted,
                TraceSpanKind::Projection,
            ),
            (CausalProtocolEventKind::GateDenied, TraceSpanKind::Gate),
            (
                CausalProtocolEventKind::ConstraintLeaseGranted,
                TraceSpanKind::Constraint,
            ),
            (
                CausalProtocolEventKind::AuthzDecisionRecorded,
                TraceSpanKind::Authorization,
            ),
            (
                CausalProtocolEventKind::DrainFenceAcquired,
                TraceSpanKind::Drain,
            ),
            (
                CausalProtocolEventKind::ViolationDetected,
                TraceSpanKind::Violation,
            ),
        ];
        for (kind, expected) in cases {
            assert_eq!(
                trace_span_kind_from_causal_protocol_event_kind(kind),
                expected
            );
        }
    }

    #[test]
    fn span_identity_and_parent_are_derived_from_protocol_causality() -> Result<(), PlanHashError> {
        let plan_hash = plan_hash()?;
        let event = event(CausalProtocolEventKind::ExecutionStarted)
            .with_plan_hash(plan_hash.clone())
            .with_correlation_id(CorrelationId("trace-1".to_owned()))
            .with_causation_id(CausalProtocolEventId("parent-event".to_owned()))
            .with_event_index(7)
            .with_occurred_at(Timestamp(99));

        let span = trace_span_from_causal_protocol_event(&event);

        assert_eq!(span.span_id, TraceSpanId("audit:event-1".to_owned()));
        assert_eq!(
            span.parent_span_id,
            Some(TraceSpanId("audit:parent-event".to_owned()))
        );
        assert_eq!(span.trace_id, CorrelationId("trace-1".to_owned()));
        assert_eq!(span.action_id, ActionId("action-1".to_owned()));
        assert_eq!(span.event_id, CausalProtocolEventId("event-1".to_owned()));
        assert_eq!(span.event_kind, CausalProtocolEventKind::ExecutionStarted);
        assert_eq!(span.plan_hash, Some(plan_hash));
        assert_eq!(span.occurred_at, Some(Timestamp(99)));
        assert!(span.attributes.contains(&TraceAttribute::EventIndex(7)));
        Ok(())
    }

    #[test]
    fn attributes_are_typed_and_only_present_when_source_fields_exist() -> Result<(), PlanHashError>
    {
        let empty =
            trace_span_from_causal_protocol_event(&event(CausalProtocolEventKind::ActionAdmitted));
        assert!(empty.attributes.is_empty());

        let event = event(CausalProtocolEventKind::AuthzDecisionRecorded)
            .with_authz_decision(authz_decision()?)
            .with_impact_set_hash(ImpactSetHash("impact-1".to_owned()));
        let span = trace_span_from_causal_protocol_event(&event);

        assert!(span.attributes.contains(&TraceAttribute::HasAuthzDecision));
        assert!(span
            .attributes
            .contains(&TraceAttribute::AuthzDecision(AuthzDecision::Allow)));
        assert!(span
            .attributes
            .contains(&TraceAttribute::ImpactSetHash(ImpactSetHash(
                "impact-1".to_owned()
            ))));
        Ok(())
    }
}
