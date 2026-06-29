//! Runtime authz/audit/projection fuzz target for the M12.5 API validation loop.
//!
//! The target drives public runtime APIs with small byte-selected variants. It is
//! an invariant harness only: existing kernel/runtime authorities decide authz,
//! capability spend, audit append and projection redaction semantics.

#![no_main]

use std::convert::Infallible;

use causlane::core::ports::{AuditLogPort, ExecutorPort};
use causlane::core::protocol::{
    ActionId, AuditEvent, AuditEventId, AuditEventKind, AuthzDecision, AuthzDecisionRef,
    AuthzPolicy, CapabilitySpendRefusal, ClaimMode, ConstraintEpoch, CorrelationId,
    EffectSignature, ExecutionBarrier, ExecutionCapability, FieldPath, ImpactHardness,
    ImpactSetHash, LeaseId, LeaseRef, Op, PlanHash, ProjectionReadRequest, RedactionPolicy,
    ResourceId, Scope, Timestamp, MAY_PROJECT_STAGE,
};
use causlane_runtime::adapters::audit::{AuditAdapterError, InMemoryAuditLog};
use causlane_runtime::adapters::tracing::{InMemoryTraceSink, TraceProjectingAuditLog};
use causlane_runtime::guarded_executor::{
    ExecutorService, GuardedExecutionRequest, GuardedExecutor, SpendError,
};
use causlane_runtime::projection_guard::guard_projection_read;
use libfuzzer_sys::fuzz_target;

const PLAN_HASH: &str = "sha256:5555555555555555555555555555555555555555555555555555555555555555";
const IMPACT_SET_HASH: &str =
    "sha256:6666666666666666666666666666666666666666666666666666666666666666";
const ACTION_ID: &str = "runtime.fuzz.promote";
const PREDICATE_ID: &str = "runtime.fuzz.promote";
const ACTOR_REF: &str = "actor://runtime/fuzz";
const OTHER_ACTOR_REF: &str = "actor://runtime/other";
const CORRELATION_ID: &str = "corr-runtime-fuzz-1";
const EXECUTION_STAGE: &str = "execution_barrier_logged";
const EXECUTION_REF: &str = "object://runtime/fuzz/promote";
const POLICY: AuthzPolicy<'static> = AuthzPolicy {
    id: "runtime.fuzz.policy",
    version: "1",
    max_age: Some(60),
};

fuzz_target!(|data: &[u8]| {
    run_case(data);
});

fn run_case(data: &[u8]) {
    let Ok(plan) = PlanHash::new(PLAN_HASH) else {
        return;
    };
    let input = Input::new(data);
    exercise_guarded_execution(&input, &plan);
    exercise_audit_trace_projection(&input, &plan);
    exercise_projection_guard(&input, &plan);
}

fn exercise_guarded_execution(input: &Input<'_>, plan: &PlanHash) {
    let mut barrier = execution_barrier(plan.clone(), input.lease_expiry());
    let decisions = input.execution_decisions(plan);
    barrier.authz_decision_refs = decisions
        .iter()
        .map(|decision| decision.decision_event_id.clone())
        .collect();
    let op = promote_op(input.op_index());
    let stages = [EXECUTION_STAGE.to_owned()];
    let guarded = GuardedExecutor::new(MarkerExecutor);

    let result = guarded.call(GuardedExecutionRequest {
        barrier: &barrier,
        predicate_id: PREDICATE_ID,
        required_stages: &stages,
        decisions: &decisions,
        expected_policy: POLICY,
        now: Timestamp(10),
        op: &op,
    });

    if input.execution_should_run() {
        assert!(
            result.is_ok(),
            "valid guarded execution refused: {result:?}"
        );
        if let Ok(outcome) = result {
            assert_eq!(outcome.produced_refs, [EXECUTION_REF.to_owned()]);
        }
    } else {
        let refused_as_expected = match &result {
            Err(SpendError::Unauthorized(_denied)) => true,
            Err(SpendError::Capability(_capability_error)) => true,
            Err(SpendError::CapabilityRefused(_refusal)) => true,
            Ok(_outcome) => false,
            Err(SpendError::Execute(_executor_error)) => false,
        };
        assert!(
            refused_as_expected,
            "invalid guarded execution ran or reached executor error: {result:?}"
        );
    }

    if input.lease_is_expired() && input.execution_decision_is_valid() && input.op_index() == 0 {
        assert!(matches!(
            guarded.call(GuardedExecutionRequest {
                barrier: &barrier,
                predicate_id: PREDICATE_ID,
                required_stages: &stages,
                decisions: &decisions,
                expected_policy: POLICY,
                now: Timestamp(10),
                op: &promote_op(0),
            }),
            Err(SpendError::CapabilityRefused(
                CapabilitySpendRefusal::Expired {
                    expires_at: _expires_at,
                    now: _now
                }
            ))
        ));
    }
}

fn exercise_audit_trace_projection(input: &Input<'_>, plan: &PlanHash) {
    let action = action_id();
    let first = runtime_event(
        "evt_runtime_fuzz_first",
        &action,
        AuditEventKind::ExecutionStarted,
        plan,
        Timestamp(10),
    );
    let second_id = if input.duplicate_audit_id() {
        "evt_runtime_fuzz_first"
    } else {
        "evt_runtime_fuzz_second"
    };
    let second = runtime_event(
        second_id,
        &action,
        AuditEventKind::ExecutionCompleted,
        plan,
        Timestamp(11),
    )
    .with_causation_id(AuditEventId("evt_runtime_fuzz_first".to_owned()));
    let mut audit =
        TraceProjectingAuditLog::new(InMemoryAuditLog::default(), InMemoryTraceSink::default());

    assert_eq!(
        AuditLogPort::append(&mut audit, first),
        Ok(AuditEventId("evt_runtime_fuzz_first".to_owned()))
    );
    assert_eq!(audit.audit_log().events().len(), 1);
    assert_eq!(audit.trace_sink().spans.len(), 1);

    let result = AuditLogPort::append(&mut audit, second);
    if input.duplicate_audit_id() {
        assert_eq!(
            result,
            Err(AuditAdapterError::DuplicateEventId {
                event_id: AuditEventId("evt_runtime_fuzz_first".to_owned())
            })
        );
        assert_eq!(audit.audit_log().events().len(), 1);
        assert_eq!(audit.trace_sink().spans.len(), 1);
    } else {
        assert_eq!(
            result,
            Ok(AuditEventId("evt_runtime_fuzz_second".to_owned()))
        );
        assert_eq!(audit.audit_log().events().len(), 2);
        assert_eq!(audit.trace_sink().spans.len(), 2);
    }
}

fn exercise_projection_guard(input: &Input<'_>, plan: &PlanHash) {
    let action = action_id();
    let fields = input.projection_fields();
    let policy = input.redaction_policy(&fields);
    let decisions = input.projection_decisions(plan);
    let req = ProjectionReadRequest {
        action: &action,
        plan,
        predicate_id: PREDICATE_ID,
        actor: ACTOR_REF,
        policy: POLICY,
        now: Timestamp(10),
    };
    let result = guard_projection_read(&decisions, &req, &policy, &fields);

    if input.projection_decision_is_valid() {
        assert!(result.is_ok(), "valid projection read refused: {result:?}");
        if let Ok(view) = result {
            assert_eq!(view.revealed.len() + view.redacted.len(), fields.len());
            for field in &fields {
                let reveal = policy.revealable.contains(field);
                assert_eq!(view.revealed.contains(field), reveal);
                assert_eq!(view.redacted.contains(field), !reveal);
            }
        }
    } else {
        assert!(result.is_err());
    }
}

struct Input<'a> {
    data: &'a [u8],
}

impl<'a> Input<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn byte(&self, index: usize) -> u8 {
        match self.data.get(index) {
            Some(byte) => *byte,
            None => 0,
        }
    }

    fn execution_decision_case(&self) -> u8 {
        self.byte(0) % 6
    }

    fn lease_case(&self) -> u8 {
        self.byte(1) % 4
    }

    fn op_index(&self) -> u32 {
        if self.byte(2) % 3 == 0 {
            1
        } else {
            0
        }
    }

    fn projection_decision_case(&self) -> u8 {
        self.byte(3) % 6
    }

    fn duplicate_audit_id(&self) -> bool {
        self.byte(4) % 2 == 0
    }

    fn field_count(&self) -> usize {
        usize::from((self.byte(5) % 4) + 1)
    }

    fn reveal_mask(&self) -> u8 {
        self.byte(6)
    }

    fn execution_decision_is_valid(&self) -> bool {
        self.execution_decision_case() == 1
    }

    fn projection_decision_is_valid(&self) -> bool {
        self.projection_decision_case() == 1
    }

    fn lease_is_expired(&self) -> bool {
        matches!(self.lease_case(), 1 | 2)
    }

    fn execution_should_run(&self) -> bool {
        self.execution_decision_is_valid() && !self.lease_is_expired() && self.op_index() == 0
    }

    fn lease_expiry(&self) -> Option<Timestamp> {
        match self.lease_case() {
            0 => None,
            1 => Some(Timestamp(9)),
            2 => Some(Timestamp(10)),
            3 => Some(Timestamp(11)),
            unexpected_case => {
                let _unexpected_case = unexpected_case;
                Some(Timestamp(11))
            }
        }
    }

    fn execution_decisions(&self, plan: &PlanHash) -> Vec<AuthzDecisionRef> {
        match self.execution_decision_case() {
            0 => Vec::new(),
            1 => vec![authz_decision(
                "evt_runtime_fuzz_exec_authz",
                EXECUTION_STAGE,
                ACTOR_REF,
                PREDICATE_ID,
                AuthzDecision::Allow,
                plan,
            )],
            2 => vec![authz_decision(
                "evt_runtime_fuzz_exec_authz",
                EXECUTION_STAGE,
                ACTOR_REF,
                PREDICATE_ID,
                AuthzDecision::Deny,
                plan,
            )],
            3 => vec![authz_decision(
                "evt_runtime_fuzz_exec_authz",
                "wrong_stage",
                ACTOR_REF,
                PREDICATE_ID,
                AuthzDecision::Allow,
                plan,
            )],
            4 => vec![authz_decision(
                "evt_runtime_fuzz_exec_authz",
                EXECUTION_STAGE,
                ACTOR_REF,
                "runtime.fuzz.other",
                AuthzDecision::Allow,
                plan,
            )],
            5 => {
                let mut decision = authz_decision(
                    "evt_runtime_fuzz_exec_authz",
                    EXECUTION_STAGE,
                    ACTOR_REF,
                    PREDICATE_ID,
                    AuthzDecision::Allow,
                    plan,
                );
                decision.expires_at = Some(Timestamp(5));
                vec![decision]
            }
            unexpected_case => {
                let _unexpected_case = unexpected_case;
                Vec::new()
            }
        }
    }

    fn projection_decisions(&self, plan: &PlanHash) -> Vec<AuthzDecisionRef> {
        match self.projection_decision_case() {
            0 => Vec::new(),
            1 => vec![authz_decision(
                "evt_runtime_fuzz_projection_authz",
                MAY_PROJECT_STAGE,
                ACTOR_REF,
                PREDICATE_ID,
                AuthzDecision::Allow,
                plan,
            )],
            2 => vec![authz_decision(
                "evt_runtime_fuzz_projection_authz",
                MAY_PROJECT_STAGE,
                OTHER_ACTOR_REF,
                PREDICATE_ID,
                AuthzDecision::Allow,
                plan,
            )],
            3 => vec![authz_decision(
                "evt_runtime_fuzz_projection_authz",
                EXECUTION_STAGE,
                ACTOR_REF,
                PREDICATE_ID,
                AuthzDecision::Allow,
                plan,
            )],
            4 => vec![authz_decision(
                "evt_runtime_fuzz_projection_authz",
                MAY_PROJECT_STAGE,
                ACTOR_REF,
                "runtime.fuzz.other",
                AuthzDecision::Allow,
                plan,
            )],
            5 => vec![authz_decision(
                "evt_runtime_fuzz_projection_authz",
                MAY_PROJECT_STAGE,
                ACTOR_REF,
                PREDICATE_ID,
                AuthzDecision::Deny,
                plan,
            )],
            unexpected_case => {
                let _unexpected_case = unexpected_case;
                Vec::new()
            }
        }
    }

    fn projection_fields(&self) -> Vec<FieldPath> {
        const FIELDS: [&str; 4] = [
            "release.status",
            "release.window",
            "release.operator_token",
            "release.internal_note",
        ];
        FIELDS
            .iter()
            .take(self.field_count())
            .map(|field| FieldPath((*field).to_owned()))
            .collect()
    }

    fn redaction_policy(&self, fields: &[FieldPath]) -> RedactionPolicy {
        RedactionPolicy {
            revealable: fields
                .iter()
                .enumerate()
                .filter_map(|(index, field)| {
                    if (self.reveal_mask() & (1 << index)) == 0 {
                        None
                    } else {
                        Some(field.clone())
                    }
                })
                .collect(),
        }
    }
}

struct MarkerExecutor;

impl ExecutorPort for MarkerExecutor {
    type Error = Infallible;

    fn execute(
        &self,
        op: &Op,
        capability: &ExecutionCapability,
    ) -> Result<Vec<String>, Self::Error> {
        let _observed_op_index = op.index;
        let _observed_capability_id = &capability.capability_id;
        Ok(vec![EXECUTION_REF.to_owned()])
    }
}

fn execution_barrier(plan: PlanHash, lease_expiry: Option<Timestamp>) -> ExecutionBarrier {
    ExecutionBarrier {
        barrier_id: AuditEventId("evt_runtime_fuzz_barrier".to_owned()),
        action_id: action_id(),
        plan_hash: plan.clone(),
        op_indexes: vec![0],
        impact_set_hash: impact_hash(),
        witnesses: Vec::new(),
        leases: vec![LeaseRef {
            lease_id: LeaseId("lease-runtime-fuzz".to_owned()),
            resource: ResourceId("resource://runtime/fuzz".to_owned()),
            scope: Scope("runtime/fuzz".to_owned()),
            mode: ClaimMode::ExclusiveWrite,
            amount: 1,
            holder_action_id: action_id(),
            holder_plan_hash: plan,
            holder_op_index: Some(0),
            epoch: ConstraintEpoch(0),
            expires_at: lease_expiry,
            lease_event_id: AuditEventId("evt_runtime_fuzz_lease".to_owned()),
        }],
        authz_decision_refs: Vec::new(),
        constraint_snapshot_id: None,
    }
}

fn promote_op(index: u32) -> Op {
    Op {
        index,
        kind: "promote".to_owned(),
        effect: EffectSignature {
            reads: Vec::new(),
            writes: vec![Scope("runtime/fuzz".to_owned())],
            produces: vec![EXECUTION_REF.to_owned()],
            requires: Vec::new(),
            invalidates: Vec::new(),
            conflict_domains: Vec::new(),
            hardness: ImpactHardness::Hard,
        },
    }
}

fn authz_decision(
    event_id: &str,
    stage: &str,
    actor: &str,
    predicate_id: &str,
    decision: AuthzDecision,
    plan: &PlanHash,
) -> AuthzDecisionRef {
    AuthzDecisionRef {
        decision_event_id: AuditEventId(event_id.to_owned()),
        action_id: action_id(),
        plan_hash: plan.clone(),
        predicate_id: predicate_id.to_owned(),
        actor: actor.to_owned(),
        stage: stage.to_owned(),
        decision,
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

fn action_id() -> ActionId {
    ActionId(ACTION_ID.to_owned())
}

fn impact_hash() -> ImpactSetHash {
    ImpactSetHash(IMPACT_SET_HASH.to_owned())
}
