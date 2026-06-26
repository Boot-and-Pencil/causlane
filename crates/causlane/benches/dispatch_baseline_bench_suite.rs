//! Criterion baseline suite for dispatch, replay and audit hot paths.

#![allow(missing_docs)]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hint::black_box;
use std::time::Duration;

use causlane::prelude::{
    select_frontier, ActionId, AuditEvent, AuditEventId, AuditEventKind, AuditLogPort, ClaimMode,
    ConstraintEpoch, ExecutionBarrier, FactKind, GraphIndex, GraphNode, ImpactSetHash,
    KernelContracts, LaneCapacity, LaneId, LeaseId, LeaseRef, LeaseTable, OpId, PlanHash,
    ResourceId, Scope,
};
use causlane_contracts::{
    examples::release_promote_impacts, examples::release_promote_plan_material, impact_set_hash,
    CompiledDispatchBundle, PlanHashMaterial, RegistryManifest,
};
use causlane_replay::{ReplayScenario, ReplayTrace};
use causlane_runtime::adapters::audit::InMemoryAuditLog;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

const REGISTRY_YAML: &str =
    include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");
const SCENARIO_YAML: &str =
    include_str!("../fixtures/contracts/scenarios/release_promote_success.scenario.yaml");
const RUNTIME_LANE: &str = "runtime-execution";

struct BenchFixture {
    bundle: CompiledDispatchBundle,
    bundle_json: String,
    graph_index: GraphIndex,
    impact_hash: ImpactSetHash,
    lanes: BTreeMap<LaneId, LaneCapacity>,
    lease: LeaseRef,
    plan_material: PlanHashMaterial,
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

        Self {
            bundle,
            bundle_json,
            graph_index: ready_conflict_index(),
            impact_hash,
            lanes: runtime_lanes(),
            lease,
            plan_material,
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

    group.finish();
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
