//! Criterion baseline suite for dispatch, replay and audit hot paths.

#![allow(missing_docs)]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::convert::Infallible;
use std::fmt::Debug;
use std::hint::black_box;
use std::time::Duration;

use causlane::core::protocol::{
    AuthzDecision, AuthzDecisionRef, AuthzPolicy, CorrelationId, EffectSignature,
    ExecutionCapability, FieldPath, ImpactHardness, ProjectionReadRequest, RedactionPolicy,
    Timestamp, MAY_PROJECT_STAGE,
};
use causlane::prelude::{
    select_frontier, ActionId, AuditEvent, AuditEventId, AuditEventKind, AuditLogPort, ClaimMode,
    ConstraintEpoch, ExecutionBarrier, ExecutorPort, FactKind, GraphIndex, GraphNode,
    ImpactSetHash, KernelContracts, LaneCapacity, LaneId, LeaseId, LeaseRef, LeaseTable, Op, OpId,
    PlanHash, ResourceId, Scope,
};
use causlane_contracts::{
    examples::release_promote_impacts, examples::release_promote_plan_material, impact_set_hash,
    CompiledDispatchBundle, PlanHashMaterial, RegistryManifest,
};
use causlane_replay::{ReplayScenario, ReplayTrace};
use causlane_runtime::adapters::audit::InMemoryAuditLog;
use causlane_runtime::adapters::tracing::{InMemoryTraceSink, TraceProjectingAuditLog};
use causlane_runtime::guarded_executor::{
    ExecutorService, GuardedExecutionRequest, GuardedExecutor,
};
use causlane_runtime::projection_guard::guard_projection_read;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

const REGISTRY_YAML: &str =
    include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");
const SCENARIO_YAML: &str =
    include_str!("../fixtures/contracts/scenarios/release_promote_success.scenario.yaml");
const RUNTIME_LANE: &str = "runtime-execution";
const RUNTIME_SCALE_OPS: u32 = 512;
const RUNTIME_SCALE_PREDICATE: &str = "runtime.scale.release.promote";
const RUNTIME_SCALE_ACTOR: &str = "actor://runtime/scale";
const RUNTIME_SCALE_STAGE: &str = "execution_barrier_logged";
const RUNTIME_SCALE_POLICY: AuthzPolicy<'static> = AuthzPolicy {
    id: "runtime.scale.guard",
    version: "1",
    max_age: Some(60),
};

struct BenchFixture {
    bundle: CompiledDispatchBundle,
    bundle_json: String,
    graph_index: GraphIndex,
    impact_hash: ImpactSetHash,
    lanes: BTreeMap<LaneId, LaneCapacity>,
    lease: LeaseRef,
    plan_material: PlanHashMaterial,
    runtime_scale: RuntimeScaleFixture,
    trace: ReplayTrace,
}

impl BenchFixture {
    fn new() -> Self {
        let manifest = parse_registry(REGISTRY_YAML);
        let bundle = compile_bundle(&manifest);
        let bundle_json = require(bundle.to_json_pretty(), "release promote bundle serializes");
        let scenario = parse_scenario(SCENARIO_YAML);
        let trace = parse_trace(&scenario, bundle.bundle_hash.0.clone());
        let plan_material = release_promote_plan_material(&bundle.bundle_hash.0);
        let plan_hash = require(
            plan_material.compute_plan_hash(),
            "release promote plan hash computes",
        );
        let impact_hash = require(
            impact_set_hash(&release_promote_impacts()),
            "release promote impact hash computes",
        );
        let lease = build_lease(plan_hash.clone());
        let runtime_scale = RuntimeScaleFixture::new(&plan_hash, &impact_hash);

        Self {
            bundle,
            bundle_json,
            graph_index: ready_conflict_index(),
            impact_hash,
            lanes: runtime_lanes(),
            lease,
            plan_material,
            runtime_scale,
            trace,
        }
    }

    fn barrier_event(&self) -> AuditEvent {
        let barrier = ExecutionBarrier {
            barrier_id: audit_event_id("barrier-1"),
            action_id: action_id(),
            plan_hash: self.lease.holder_plan_hash.clone(),
            op_indexes: vec![0],
            impact_set_hash: self.impact_hash.clone(),
            witnesses: Vec::new(),
            leases: vec![self.lease.clone()],
            authz_decision_refs: Vec::new(),
            constraint_snapshot_id: None,
        };

        AuditEvent::new(
            audit_event_id("barrier-1"),
            action_id(),
            AuditEventKind::ExecutionBarrierLogged,
        )
        .with_plan_hash(self.lease.holder_plan_hash.clone())
        .with_impact_set_hash(self.impact_hash.clone())
        .with_leases(vec![self.lease.clone()])
        .with_execution_barrier(barrier)
    }
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_millis(300))
}

fn dispatch_baseline_benches(criterion: &mut Criterion) {
    let fixture = BenchFixture::new();
    let mut group = criterion.benchmark_group("dispatch_baseline_bench_suite");

    group.bench_function("registry_normalize_from_yaml", |bencher| {
        bencher.iter(|| parse_registry(black_box(REGISTRY_YAML)));
    });
    group.bench_function("plan_hash_release_promote", |bencher| {
        bencher.iter(|| {
            require(
                fixture.plan_material.compute_plan_hash(),
                "release promote plan hash computes",
            )
        });
    });
    group.bench_function("bundle_load_from_json", |bencher| {
        bencher.iter(|| parse_bundle_json(black_box(&fixture.bundle_json)));
    });
    group.bench_function("replay_verify_with_bundle", |bencher| {
        bencher.iter(|| {
            require(
                fixture.trace.verify_with_bundle(black_box(&fixture.bundle)),
                "release promote trace verifies",
            );
        });
    });
    group.bench_function("frontier_conflict_selection", |bencher| {
        bencher
            .iter(|| select_frontier(black_box(&fixture.graph_index), black_box(&fixture.lanes)));
    });
    group.bench_function("lease_grant_exclusive", |bencher| {
        bencher.iter_batched(
            || (LeaseTable::new(), fixture.lease.clone()),
            |(mut leases, lease)| {
                require(
                    leases.grant(black_box(lease), black_box(&KernelContracts)),
                    "exclusive lease grants on empty table",
                );
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("barrier_audit_append", |bencher| {
        bencher.iter_batched(
            || (InMemoryAuditLog::default(), fixture.barrier_event()),
            |(mut audit, event)| {
                require(
                    audit.append(black_box(event)),
                    "barrier event appends to empty audit log",
                )
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("replay_explain_human", |bencher| {
        bencher.iter(|| {
            fixture
                .trace
                .verify_explain(black_box(&fixture.bundle))
                .to_human()
        });
    });
    group.bench_function("runtime_guarded_audit_projection_flow", |bencher| {
        bencher.iter_batched(
            || fixture.runtime_scale.events.clone(),
            |events| black_box(fixture.runtime_scale.run(events)),
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

struct RuntimeScaleFixture {
    action: ActionId,
    barrier: ExecutionBarrier,
    execution_allow: AuthzDecisionRef,
    projection_allow: AuthzDecisionRef,
    ops: Vec<Op>,
    events: Vec<AuditEvent>,
    fields: Vec<FieldPath>,
    redaction_policy: RedactionPolicy,
    required_stages: Vec<String>,
}

impl RuntimeScaleFixture {
    fn new(plan: &PlanHash, impact_hash: &ImpactSetHash) -> Self {
        let action = ActionId("runtime.scale.release.promote".to_owned());
        let barrier = runtime_scale_barrier(&action, plan, impact_hash);
        let execution_allow = runtime_scale_decision(
            "evt-runtime-scale-authz-execution",
            RUNTIME_SCALE_STAGE,
            &action,
            plan,
        );
        let projection_allow = runtime_scale_decision(
            "evt-runtime-scale-authz-projection",
            MAY_PROJECT_STAGE,
            &action,
            plan,
        );
        let ops = (0..RUNTIME_SCALE_OPS)
            .map(runtime_scale_op)
            .collect::<Vec<_>>();
        let events = runtime_scale_events(
            &action,
            plan,
            impact_hash,
            &barrier,
            &execution_allow,
            &projection_allow,
        );

        Self {
            action,
            barrier,
            execution_allow,
            projection_allow,
            ops,
            events,
            fields: vec![
                FieldPath("release.status".to_owned()),
                FieldPath("release.window".to_owned()),
                FieldPath("release.operator_token".to_owned()),
            ],
            redaction_policy: RedactionPolicy {
                revealable: [
                    FieldPath("release.status".to_owned()),
                    FieldPath("release.window".to_owned()),
                ]
                .into_iter()
                .collect(),
            },
            required_stages: vec![RUNTIME_SCALE_STAGE.to_owned()],
        }
    }

    fn run(&self, events: Vec<AuditEvent>) -> usize {
        let guarded = GuardedExecutor::new(RuntimeScaleExecutor);
        for op in &self.ops {
            let produced_refs = require(
                guarded.call(GuardedExecutionRequest {
                    barrier: &self.barrier,
                    predicate_id: RUNTIME_SCALE_PREDICATE,
                    required_stages: &self.required_stages,
                    decisions: std::slice::from_ref(&self.execution_allow),
                    expected_policy: RUNTIME_SCALE_POLICY,
                    now: Timestamp(10),
                    op,
                }),
                "runtime scale guarded execution succeeds",
            );
            black_box(produced_refs);
        }

        let mut audit =
            TraceProjectingAuditLog::new(InMemoryAuditLog::default(), InMemoryTraceSink::default());
        require(
            AuditLogPort::append_batch(&mut audit, events),
            "runtime scale audit batch appends",
        );

        let projection = require(
            guard_projection_read(
                std::slice::from_ref(&self.projection_allow),
                &ProjectionReadRequest {
                    action: &self.action,
                    plan: &self.barrier.plan_hash,
                    predicate_id: RUNTIME_SCALE_PREDICATE,
                    actor: RUNTIME_SCALE_ACTOR,
                    policy: RUNTIME_SCALE_POLICY,
                    now: Timestamp(20),
                },
                &self.redaction_policy,
                &self.fields,
            ),
            "runtime scale projection read is authorized",
        );

        let audit_events = audit.audit_log().events().len();
        let trace_spans = audit.trace_sink().spans.len();
        let projected_fields = projection.revealed.len() + projection.redacted.len();
        let redacted_fields = projection.redacted.len();
        assert_eq!(audit_events, self.events.len());
        assert_eq!(trace_spans, audit_events);
        assert_eq!(projected_fields, self.fields.len());
        assert_eq!(redacted_fields, 1);
        self.ops.len() + audit_events + trace_spans + projected_fields + redacted_fields
    }
}

fn runtime_scale_barrier(
    action: &ActionId,
    plan: &PlanHash,
    impact_hash: &ImpactSetHash,
) -> ExecutionBarrier {
    ExecutionBarrier {
        barrier_id: AuditEventId("evt-runtime-scale-barrier".to_owned()),
        action_id: action.clone(),
        plan_hash: plan.clone(),
        op_indexes: (0..RUNTIME_SCALE_OPS).collect(),
        impact_set_hash: impact_hash.clone(),
        witnesses: Vec::new(),
        leases: vec![runtime_scale_lease(action.clone(), plan.clone())],
        authz_decision_refs: vec![
            AuditEventId("evt-runtime-scale-authz-execution".to_owned()),
            AuditEventId("evt-runtime-scale-authz-projection".to_owned()),
        ],
        constraint_snapshot_id: None,
    }
}

fn runtime_scale_events(
    action: &ActionId,
    plan: &PlanHash,
    impact_hash: &ImpactSetHash,
    barrier: &ExecutionBarrier,
    execution_allow: &AuthzDecisionRef,
    projection_allow: &AuthzDecisionRef,
) -> Vec<AuditEvent> {
    let mut events = vec![
        AuditEvent::new(
            AuditEventId("evt-runtime-scale-barrier".to_owned()),
            action.clone(),
            AuditEventKind::ExecutionBarrierLogged,
        )
        .with_plan_hash(plan.clone())
        .with_impact_set_hash(impact_hash.clone())
        .with_execution_barrier(barrier.clone()),
        runtime_scale_event(
            "evt-runtime-scale-authz-execution",
            action,
            AuditEventKind::AuthzDecisionRecorded,
            plan,
        )
        .with_authz_decision(execution_allow.clone()),
    ];
    events.extend(runtime_scale_execution_events(action, plan));
    events.extend([
        runtime_scale_event(
            "evt-runtime-scale-authz-projection",
            action,
            AuditEventKind::AuthzDecisionRecorded,
            plan,
        )
        .with_authz_decision(projection_allow.clone()),
        runtime_scale_event(
            "evt-runtime-scale-projection",
            action,
            AuditEventKind::ProjectionEmitted,
            plan,
        )
        .with_causation_id(AuditEventId(
            "evt-runtime-scale-authz-projection".to_owned(),
        )),
    ]);
    events
}

fn runtime_scale_execution_events(action: &ActionId, plan: &PlanHash) -> Vec<AuditEvent> {
    (0..RUNTIME_SCALE_OPS)
        .flat_map(|index| {
            [
                runtime_scale_event(
                    &format!("evt-runtime-scale-started-{index}"),
                    action,
                    AuditEventKind::ExecutionStarted,
                    plan,
                )
                .with_causation_id(AuditEventId("evt-runtime-scale-barrier".to_owned())),
                runtime_scale_event(
                    &format!("evt-runtime-scale-completed-{index}"),
                    action,
                    AuditEventKind::ExecutionCompleted,
                    plan,
                )
                .with_causation_id(AuditEventId(format!("evt-runtime-scale-started-{index}"))),
            ]
        })
        .collect()
}

struct RuntimeScaleExecutor;

impl ExecutorPort for RuntimeScaleExecutor {
    type Error = Infallible;

    fn execute(
        &self,
        op: &Op,
        _capability: &ExecutionCapability,
    ) -> Result<Vec<String>, Self::Error> {
        Ok(vec![format!("object://runtime/scale/{}", op.index)])
    }
}

fn runtime_scale_lease(action: ActionId, plan: PlanHash) -> LeaseRef {
    LeaseRef {
        lease_id: LeaseId("lease-runtime-scale".to_owned()),
        resource: ResourceId("resource://runtime/scale".to_owned()),
        scope: Scope("release/runtime-scale".to_owned()),
        mode: ClaimMode::ExclusiveWrite,
        amount: 1,
        holder_action_id: action,
        holder_plan_hash: plan,
        holder_op_index: None,
        epoch: ConstraintEpoch(1),
        expires_at: None,
        lease_event_id: AuditEventId("evt-runtime-scale-lease".to_owned()),
    }
}

fn runtime_scale_op(index: u32) -> Op {
    Op {
        index,
        kind: "promote".to_owned(),
        effect: EffectSignature {
            reads: Vec::new(),
            writes: vec![Scope(format!("release/runtime-scale/{index}"))],
            produces: vec![format!("object://runtime/scale/{index}")],
            requires: Vec::new(),
            invalidates: Vec::new(),
            conflict_domains: Vec::new(),
            hardness: ImpactHardness::Hard,
        },
    }
}

fn runtime_scale_decision(
    event_id: &str,
    stage: &str,
    action: &ActionId,
    plan: &PlanHash,
) -> AuthzDecisionRef {
    AuthzDecisionRef {
        decision_event_id: AuditEventId(event_id.to_owned()),
        action_id: action.clone(),
        plan_hash: plan.clone(),
        predicate_id: RUNTIME_SCALE_PREDICATE.to_owned(),
        actor: RUNTIME_SCALE_ACTOR.to_owned(),
        stage: stage.to_owned(),
        decision: AuthzDecision::Allow,
        policy_id: RUNTIME_SCALE_POLICY.id.to_owned(),
        policy_version: RUNTIME_SCALE_POLICY.version.to_owned(),
        issued_at: Timestamp(0),
        expires_at: Some(Timestamp(100)),
        attestation: None,
    }
}

fn runtime_scale_event(
    event_id: &str,
    action: &ActionId,
    kind: AuditEventKind,
    plan: &PlanHash,
) -> AuditEvent {
    AuditEvent::new(AuditEventId(event_id.to_owned()), action.clone(), kind)
        .with_plan_hash(plan.clone())
        .with_correlation_id(CorrelationId("corr-runtime-scale".to_owned()))
        .with_occurred_at(Timestamp(10))
}

fn parse_registry(yaml: &str) -> RegistryManifest {
    require(
        RegistryManifest::from_yaml_str(yaml),
        "release promote registry parses",
    )
}

fn compile_bundle(manifest: &RegistryManifest) -> CompiledDispatchBundle {
    require(
        CompiledDispatchBundle::compile(manifest),
        "release promote bundle compiles",
    )
}

fn parse_bundle_json(json: &str) -> CompiledDispatchBundle {
    require(
        CompiledDispatchBundle::from_json_str(json),
        "release promote bundle JSON parses",
    )
}

fn parse_scenario(yaml: &str) -> ReplayScenario {
    require(
        ReplayScenario::from_yaml_str(yaml),
        "release promote scenario parses",
    )
}

fn parse_trace(scenario: &ReplayScenario, bundle_hash: String) -> ReplayTrace {
    let json = require(
        scenario.to_trace_json_pretty_bound(Some(bundle_hash)),
        "release promote trace JSON serializes",
    );
    require(
        ReplayTrace::from_json_str(&json),
        "release promote trace JSON parses",
    )
}

fn ready_conflict_index() -> GraphIndex {
    let mut index = GraphIndex::new();
    for node in ready_nodes() {
        index.add_node(node);
    }
    index
}

fn ready_nodes() -> Vec<GraphNode> {
    vec![
        node("candidate-promote", 0, &["environment:staging"]),
        node("candidate-promote", 1, &["environment:staging"]),
        node("notify", 0, &["notification:release"]),
        node("audit", 0, &["audit:release"]),
        node("candidate-promote", 2, &["release_candidate:rc_123"]),
        node("metrics", 0, &["metrics:release"]),
    ]
}

fn node(action: &str, index: u32, writes: &[&str]) -> GraphNode {
    GraphNode {
        op_id: op(action, index),
        lane: LaneId(RUNTIME_LANE.to_owned()),
        requires: Vec::<FactKind>::new(),
        writes: writes
            .iter()
            .map(|scope| Scope((*scope).to_owned()))
            .collect(),
    }
}

fn op(action: &str, index: u32) -> OpId {
    OpId(ActionId(action.to_owned()), index)
}

fn runtime_lanes() -> BTreeMap<LaneId, LaneCapacity> {
    BTreeMap::from([(LaneId(RUNTIME_LANE.to_owned()), LaneCapacity::Bounded(4))])
}

fn build_lease(plan_hash: PlanHash) -> LeaseRef {
    LeaseRef {
        lease_id: LeaseId("lease-1".to_owned()),
        resource: ResourceId("environment_write".to_owned()),
        scope: Scope("environment:staging".to_owned()),
        mode: ClaimMode::ExclusiveWrite,
        amount: 1,
        holder_action_id: action_id(),
        holder_plan_hash: plan_hash,
        holder_op_index: Some(0),
        epoch: ConstraintEpoch(1),
        expires_at: None,
        lease_event_id: audit_event_id("lease-event-1"),
    }
}

fn action_id() -> ActionId {
    ActionId("act_promote_123".to_owned())
}

fn audit_event_id(value: &str) -> AuditEventId {
    AuditEventId(value.to_owned())
}

fn require<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: Debug,
{
    match result {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{context}: {error:?}");
            std::process::exit(1);
        }
    }
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = dispatch_baseline_benches
}
criterion_main!(benches);
