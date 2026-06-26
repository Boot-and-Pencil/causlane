//! Audit event model.

use super::{
    ActionId, CorrelationId, EventHash, ExecutionBarrier, ExecutionCapability, FactKind,
    ImpactSetHash, LeaseRef, PlanHash, Scope, Timestamp,
};

/// Identifies a single entry in the audit/event journal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AuditEventId(pub String);

/// A causal reference proving that a projection is derived from a specific
/// committed observed truth (ADR-0010). Distinct from a witness: a witness
/// answers "why is this transition allowed?", an anchor answers "from which
/// observed truth is this projection built?".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TruthAnchor {
    /// The `ObservedTruthCommitted` event this projection is anchored to.
    pub event_id: AuditEventId,
    /// Action the anchored truth belongs to.
    pub action_id: ActionId,
    /// Plan hash the anchored truth was produced under.
    pub plan_hash: PlanHash,
    /// Optional fact kind the projection requires.
    pub fact_kind: Option<FactKind>,
    /// Optional scope the projection is restricted to.
    pub scope: Option<Scope>,
    /// Optional content hash pinning the exact anchored event.
    pub event_hash: Option<EventHash>,
}

/// Return whether an event kind is allowed to serve as a projection truth
/// anchor source.
#[must_use]
pub fn projection_anchor_source_is_observed(kind: AuditEventKind) -> bool {
    kind == AuditEventKind::ObservedTruthCommitted
}

/// The kind of evidence a witness reference points at (ADR-0013).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WitnessKind {
    /// A previously committed observed fact.
    ObservedFact,
    /// An oversight/gate approval event.
    GateApproval,
    /// Authorization decision evidence.
    AuthzDecision,
    /// Constraint-plane decision evidence.
    ConstraintDecision,
    /// Evidence imported from outside the audit journal.
    ExternalEvidence,
}

/// Binding metadata that ties a witness to the exact action/plan/impact it
/// authorizes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WitnessBinding {
    /// Action the witness is bound to.
    pub action_id: ActionId,
    /// Plan hash the witness is bound to.
    pub plan_hash: PlanHash,
    /// Optional impact set hash the witness is bound to.
    pub impact_set_hash: Option<ImpactSetHash>,
}

/// A typed witness reference used by execution barriers (ADR-0013).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WitnessRef {
    /// Audit event satisfying the witness requirement.
    pub event_id: AuditEventId,
    /// Required-witness id from the compiled bundle.
    pub requirement_id: String,
    /// Witness evidence kind.
    pub kind: WitnessKind,
    /// Fact kind the witness asserts, if applicable.
    pub fact_kind: Option<FactKind>,
    /// Scope the witness applies to, if applicable.
    pub scope: Option<Scope>,
    /// Optional action/plan/impact binding.
    pub binds_to: Option<WitnessBinding>,
}

/// The fact a producer event attests about itself (ADR-0010/0013): the kind of
/// fact it records and the scope that fact is restricted to. A witness ref or a
/// projection anchor must match the producer event's own attestation rather than
/// self-asserting a fact the producer never recorded (P0-004). Carried on the
/// *producer* event (e.g. `GateApproved`, `ObservedTruthCommitted`), not on the
/// referencing barrier/projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WitnessAttestation {
    /// The kind of fact this producer event records.
    pub fact_kind: FactKind,
    /// The scope the attested fact applies to.
    pub scope: Scope,
}

/// Protocol-level authorization decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthzDecision {
    /// The policy allowed the action/stage.
    Allow,
    /// The policy denied the action/stage.
    Deny,
}

/// A keyed attestation (HMAC) over an authz decision's canonical bytes, minted by
/// the authorizing PDP. Hex-encoded for the trace JSON. A replay verifier
/// configured with the PDP secret confirms the decision came from the PDP, so an
/// attacker who authored the trace cannot forge an `Allow` (ADR-0011).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthzAttestation(pub String);

/// Typed authorization evidence recorded in the audit journal (ADR-0011).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthzDecisionRef {
    /// Event id that recorded the decision.
    pub decision_event_id: AuditEventId,
    /// Action the decision is bound to.
    pub action_id: ActionId,
    /// Plan hash the decision is bound to.
    pub plan_hash: PlanHash,
    /// Predicate id the decision applies to.
    pub predicate_id: String,
    /// Actor/principal evaluated by the policy.
    pub actor: String,
    /// Lifecycle stage the decision authorizes.
    pub stage: String,
    /// Allow/deny result.
    pub decision: AuthzDecision,
    /// Stable policy id.
    pub policy_id: String,
    /// Stable policy version.
    pub policy_version: String,
    /// Decision issue time.
    pub issued_at: Timestamp,
    /// Decision expiry time, if any.
    pub expires_at: Option<Timestamp>,
    /// Optional keyed attestation over [`AuthzDecisionRef::attestation_message`].
    /// Present when the PDP signed the decision; verified by replay when
    /// configured with the PDP secret.
    pub attestation: Option<AuthzAttestation>,
}

impl AuthzDecisionRef {
    /// Canonical bytes a PDP attests over (HMAC) when issuing this decision. Each
    /// field is length-prefixed so the concatenation is injective. Excludes the
    /// attestation itself and the recording event id (which the journal, not the
    /// PDP, assigns).
    #[must_use]
    pub fn attestation_message(&self) -> Vec<u8> {
        use core::fmt::Write as _;
        let decision = match self.decision {
            AuthzDecision::Allow => "allow",
            AuthzDecision::Deny => "deny",
        };
        let issued_at = self.issued_at.0.to_string();
        let expires_at = self
            .expires_at
            .map_or_else(|| "none".to_owned(), |timestamp| timestamp.0.to_string());
        let mut msg = String::from("causlane-authz-attestation-v1");
        for field in [
            self.action_id.0.as_str(),
            self.plan_hash.as_str(),
            self.predicate_id.as_str(),
            self.actor.as_str(),
            self.stage.as_str(),
            decision,
            self.policy_id.as_str(),
            self.policy_version.as_str(),
            issued_at.as_str(),
            expires_at.as_str(),
        ] {
            let _written = write!(msg, "\u{1f}{}:{field}", field.len());
        }
        msg.into_bytes()
    }
}

/// An append-only audit/event journal entry — the single authority for observed
/// truth (ADR-0003). Carries the causal and binding metadata replay needs:
/// correlation/causation, witnesses, truth anchors, leases and the impact set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditEvent {
    /// Unique id of this journal entry.
    pub event_id: AuditEventId,
    /// Action this event belongs to.
    pub action_id: ActionId,
    /// Plan hash this event was produced under, if any.
    pub plan_hash: Option<PlanHash>,
    /// The kind of event recorded.
    pub kind: AuditEventKind,
    /// Run-scoped correlation id grouping all events of one action.
    pub correlation_id: CorrelationId,
    /// The event that directly caused this one, if any.
    pub causation_id: Option<AuditEventId>,
    /// Causal witnesses justifying a transition (e.g. a readiness fact).
    pub witnesses: Vec<AuditEventId>,
    /// Typed witness refs used by protocol-critical barriers.
    pub witness_refs: Vec<WitnessRef>,
    /// Truth anchors for projection events (see [`TruthAnchor`]).
    pub anchors: Vec<TruthAnchor>,
    /// Leases referenced by an execution barrier (ADR-0013).
    pub leases: Vec<LeaseRef>,
    /// Normalized execution barrier payload, when this event is an
    /// `ExecutionBarrierLogged` event.
    pub execution_barrier: Option<ExecutionBarrier>,
    /// Typed authorization decision payload, when this event records one.
    pub authz_decision: Option<AuthzDecisionRef>,
    /// Capability payload carried by `ExecutionStarted`.
    pub execution_capability: Option<ExecutionCapability>,
    /// Hash of the planned impact set this event is bound to (ADR-0009).
    pub impact_set_hash: Option<ImpactSetHash>,
    /// Monotonic position of this event within its journal partition.
    pub event_index: Option<u64>,
    /// The scope a drain fence event covers (I-007), when this is a
    /// `DrainFenceRequested`/`DrainFenceAcquired` event.
    pub drain_fence_scope: Option<Scope>,
    /// The fact this event attests about itself, when it is a producer of
    /// witness/anchor evidence (e.g. `GateApproved`, `ObservedTruthCommitted`).
    /// The oracle grounds referencing witness refs / projection anchors against
    /// this so they cannot self-assert a fact the producer never recorded.
    pub attested_fact: Option<WitnessAttestation>,
    /// Wall-clock time this event was recorded, when known. Used as the
    /// evaluation time for time-sensitive replay checks (e.g. authz freshness:
    /// an authz decision must be issued at-or-before, and not expired by, the
    /// barrier event's `occurred_at`).
    pub occurred_at: Option<Timestamp>,
}

impl AuditEvent {
    /// Create a minimal event. `correlation_id` defaults to the action id; use
    /// the builder methods to attach plan hash, witnesses, anchors, etc.
    #[must_use]
    pub fn new(event_id: AuditEventId, action_id: ActionId, kind: AuditEventKind) -> Self {
        let correlation_id = CorrelationId(action_id.0.clone());
        Self {
            event_id,
            action_id,
            plan_hash: None,
            kind,
            correlation_id,
            causation_id: None,
            witnesses: Vec::new(),
            witness_refs: Vec::new(),
            anchors: Vec::new(),
            leases: Vec::new(),
            execution_barrier: None,
            authz_decision: None,
            execution_capability: None,
            impact_set_hash: None,
            event_index: None,
            drain_fence_scope: None,
            attested_fact: None,
            occurred_at: None,
        }
    }

    /// Attach the plan hash this event was produced under.
    #[must_use]
    pub fn with_plan_hash(mut self, plan_hash: PlanHash) -> Self {
        self.plan_hash = Some(plan_hash);
        self
    }

    /// Override the correlation id.
    #[must_use]
    pub fn with_correlation_id(mut self, correlation_id: CorrelationId) -> Self {
        self.correlation_id = correlation_id;
        self
    }

    /// Set the directly-causing event.
    #[must_use]
    pub fn with_causation_id(mut self, causation_id: AuditEventId) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    /// Attach causal witnesses.
    #[must_use]
    pub fn with_witnesses(mut self, witnesses: Vec<AuditEventId>) -> Self {
        self.witnesses = witnesses;
        self
    }

    /// Attach typed witness refs.
    #[must_use]
    pub fn with_witness_refs(mut self, witness_refs: Vec<WitnessRef>) -> Self {
        self.witness_refs = witness_refs;
        self
    }

    /// Attach truth anchors (for projection events).
    #[must_use]
    pub fn with_anchors(mut self, anchors: Vec<TruthAnchor>) -> Self {
        self.anchors = anchors;
        self
    }

    /// Attach lease references (for barrier events).
    #[must_use]
    pub fn with_leases(mut self, leases: Vec<LeaseRef>) -> Self {
        self.leases = leases;
        self
    }

    /// Attach normalized execution barrier payload.
    #[must_use]
    pub fn with_execution_barrier(mut self, barrier: ExecutionBarrier) -> Self {
        self.execution_barrier = Some(barrier);
        self
    }

    /// Attach typed authorization decision evidence.
    #[must_use]
    pub fn with_authz_decision(mut self, authz_decision: AuthzDecisionRef) -> Self {
        self.authz_decision = Some(authz_decision);
        self
    }

    /// Attach an execution capability to an execution-start event.
    #[must_use]
    pub fn with_execution_capability(mut self, capability: ExecutionCapability) -> Self {
        self.execution_capability = Some(capability);
        self
    }

    /// Bind the event to a planned impact set.
    #[must_use]
    pub fn with_impact_set_hash(mut self, impact_set_hash: ImpactSetHash) -> Self {
        self.impact_set_hash = Some(impact_set_hash);
        self
    }

    /// Set the journal position.
    #[must_use]
    pub fn with_event_index(mut self, event_index: u64) -> Self {
        self.event_index = Some(event_index);
        self
    }

    /// Bind a drain fence event to the scope it covers (I-007).
    #[must_use]
    pub fn with_drain_fence_scope(mut self, scope: Scope) -> Self {
        self.drain_fence_scope = Some(scope);
        self
    }

    /// Record the fact this event attests about itself (witness/anchor producer).
    #[must_use]
    pub fn with_attested_fact(mut self, attested_fact: WitnessAttestation) -> Self {
        self.attested_fact = Some(attested_fact);
        self
    }

    /// Record the wall-clock time this event occurred (evaluation time for
    /// time-sensitive replay checks).
    #[must_use]
    pub fn with_occurred_at(mut self, occurred_at: Timestamp) -> Self {
        self.occurred_at = Some(occurred_at);
        self
    }
}

/// The kind of fact a journal entry records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuditEventKind {
    /// An action was admitted into dispatch.
    ActionAdmitted,
    /// An admitted action was compiled into a plan.
    ActionPlanned,
    /// A plan's dispatch was logged.
    DispatchLogged,
    /// An execution barrier (with leases) was logged.
    ExecutionBarrierLogged,
    /// Execution of the plan started.
    ExecutionStarted,
    /// Execution of the plan completed.
    ExecutionCompleted,
    /// An observed truth was committed to the journal.
    ObservedTruthCommitted,
    /// A projection derived from observed truth was emitted.
    ProjectionEmitted,
    /// The action's lifecycle was closed.
    LifecycleClosed,
    /// An oversight gate approved the action.
    GateApproved,
    /// An oversight gate denied the action.
    GateDenied,
    /// A constraint lease was granted.
    ConstraintLeaseGranted,
    /// A constraint lease was released.
    ConstraintLeaseReleased,
    /// A constraint violation was detected.
    ViolationDetected,
    /// An authorization decision was recorded.
    AuthzDecisionRecorded,
    /// A drain fence over a scope was requested (I-007).
    DrainFenceRequested,
    /// A drain fence over a scope was acquired — valid only when no lease
    /// actively overlaps the fence scope (I-007).
    DrainFenceAcquired,
}

impl AuditEventKind {
    /// Stable dotted token used at serialization/storage boundaries.
    #[must_use]
    pub const fn stable_token(self) -> &'static str {
        match self {
            Self::ActionAdmitted => "action.admitted",
            Self::ActionPlanned => "action.planned",
            Self::DispatchLogged => "dispatch.logged",
            Self::ExecutionBarrierLogged => "execution.barrier_logged",
            Self::ExecutionStarted => "execution.started",
            Self::ExecutionCompleted => "execution.completed",
            Self::ObservedTruthCommitted => "observed_truth.committed",
            Self::ProjectionEmitted => "projection.emitted",
            Self::LifecycleClosed => "lifecycle.closed",
            Self::GateApproved => "gate.approved",
            Self::GateDenied => "gate.denied",
            Self::ConstraintLeaseGranted => "constraint.lease_granted",
            Self::ConstraintLeaseReleased => "constraint.lease_released",
            Self::ViolationDetected => "violation.detected",
            Self::AuthzDecisionRecorded => "authz.decision_recorded",
            Self::DrainFenceRequested => "drain.fence_requested",
            Self::DrainFenceAcquired => "drain.fence_acquired",
        }
    }
}

/// All currently defined audit event kinds.
pub const ALL_AUDIT_EVENT_KINDS: [AuditEventKind; 17] = [
    AuditEventKind::ActionAdmitted,
    AuditEventKind::ActionPlanned,
    AuditEventKind::DispatchLogged,
    AuditEventKind::ExecutionBarrierLogged,
    AuditEventKind::ExecutionStarted,
    AuditEventKind::ExecutionCompleted,
    AuditEventKind::ObservedTruthCommitted,
    AuditEventKind::ProjectionEmitted,
    AuditEventKind::LifecycleClosed,
    AuditEventKind::GateApproved,
    AuditEventKind::GateDenied,
    AuditEventKind::ConstraintLeaseGranted,
    AuditEventKind::ConstraintLeaseReleased,
    AuditEventKind::ViolationDetected,
    AuditEventKind::AuthzDecisionRecorded,
    AuditEventKind::DrainFenceRequested,
    AuditEventKind::DrainFenceAcquired,
];

#[cfg(test)]
mod tests {
    use super::{
        projection_anchor_source_is_observed, ActionId, AuditEventId, AuditEventKind,
        AuthzDecision, AuthzDecisionRef, PlanHash, Timestamp, ALL_AUDIT_EVENT_KINDS,
    };
    use crate::PlanHashError;

    // I-003: ObservedTruthCommitted is the ONLY event kind that may source a
    // projection truth anchor; every other kind is rejected (exhaustive over the
    // event vocabulary).
    #[test]
    fn only_observed_truth_is_a_valid_anchor_source() {
        for kind in ALL_AUDIT_EVENT_KINDS {
            let expected = kind == AuditEventKind::ObservedTruthCommitted;
            assert_eq!(
                projection_anchor_source_is_observed(kind),
                expected,
                "{kind:?}"
            );
        }
    }

    #[test]
    fn every_audit_event_kind_has_one_stable_token() {
        let expected = [
            "action.admitted",
            "action.planned",
            "dispatch.logged",
            "execution.barrier_logged",
            "execution.started",
            "execution.completed",
            "observed_truth.committed",
            "projection.emitted",
            "lifecycle.closed",
            "gate.approved",
            "gate.denied",
            "constraint.lease_granted",
            "constraint.lease_released",
            "violation.detected",
            "authz.decision_recorded",
            "drain.fence_requested",
            "drain.fence_acquired",
        ];
        let actual = ALL_AUDIT_EVENT_KINDS.map(AuditEventKind::stable_token);

        assert_eq!(actual, expected);
    }

    fn authz_decision() -> Result<AuthzDecisionRef, PlanHashError> {
        Ok(AuthzDecisionRef {
            decision_event_id: AuditEventId("d".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: PlanHash::new(format!("sha256:{}", "1".repeat(PlanHash::DIGEST_LEN)))?,
            predicate_id: "pred".to_owned(),
            actor: "alice".to_owned(),
            stage: "execution_barrier_logged".to_owned(),
            decision: AuthzDecision::Allow,
            policy_id: "p".to_owned(),
            policy_version: "1".to_owned(),
            issued_at: Timestamp(0),
            expires_at: None,
            attestation: None,
        })
    }

    // The authz attestation message is deterministic and depends on each signed
    // field, so a PDP signature cannot be replayed across a different decision
    // (ADR-0011). The journal-assigned decision_event_id is intentionally NOT signed.
    #[test]
    fn authz_attestation_message_is_deterministic_and_field_sensitive() -> Result<(), PlanHashError>
    {
        let base = authz_decision()?.attestation_message();
        assert_eq!(base, authz_decision()?.attestation_message());

        let mut denied = authz_decision()?;
        denied.decision = AuthzDecision::Deny;

        let mut actor = authz_decision()?;
        actor.actor = "mallory".to_owned();

        let mut stage = authz_decision()?;
        stage.stage = "execution_started".to_owned();

        let mut expires = authz_decision()?;
        expires.expires_at = Some(Timestamp(99));

        for decision in [denied, actor, stage, expires] {
            assert_ne!(base, decision.attestation_message());
        }

        // The recording event id is journal-assigned, not signed by the PDP.
        let mut other_event = authz_decision()?;
        other_event.decision_event_id = AuditEventId("other".to_owned());
        assert_eq!(base, other_event.attestation_message());
        Ok(())
    }

    // Length-prefixed encoding is injective at field boundaries: two decisions whose
    // raw predicate_id ++ actor would concatenate identically still differ.
    #[test]
    fn authz_attestation_message_is_boundary_injective() -> Result<(), PlanHashError> {
        let mut left = authz_decision()?;
        left.predicate_id = "ab".to_owned();
        left.actor = "cd".to_owned();

        let mut right = authz_decision()?;
        right.predicate_id = "abc".to_owned();
        right.actor = "d".to_owned();

        assert_ne!(left.attestation_message(), right.attestation_message());
        Ok(())
    }
}
