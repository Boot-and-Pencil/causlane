# Runtime and performance

## Principle

The hot path must be small, local, precompiled and indexed.

The slow path may do explanation, replay, formal projection, heavy graph diagnostics and support bundles.

## Avoid

```text
request -> full JSON schema interpretation
        -> remote policy call
        -> full graph rebuild
        -> global lock
        -> SQL transaction per tiny step
        -> synchronous telemetry export
```

## Prefer

```text
request -> compiled predicate lookup
        -> typed normalization
        -> plan template / pure compile
        -> effect signature materialization
        -> partition enqueue
        -> indexed conflict/constraint check
        -> batched durable audit append, where required
```

## Dispatcher partitioning

Use partition-owner loops where practical:

```text
one async task owns one partition state;
messages enter through bounded channels;
state mutation is local to the partition;
conflict checks are indexed by scope/domain.
```

Possible partition keys:

```text
tenant;
conflict domain;
root subject id;
environment;
workflow/root action.
```

## M08.1 in-process runtime

`causlane-runtime/tokio-runtime` provides the first in-process implementation of
the partition-owner pattern. Hosts construct `InProcessRuntime` with a fixed
partition set and route each admitted task with an explicit `PartitionKey`.

The runtime keeps state local to each partition loop:

```text
submit/try_submit -> validate_host_task -> partition bounded queue
partition loop    -> dependency-ready task -> semaphore permit -> host handler
```

`submit` waits for bounded ingress capacity. `try_submit` returns `QueueFull`
when the partition ingress queue or admitted pending set has no room. A
runtime-wide semaphore caps concurrent handler calls across partitions; it is a
capacity control only, not semantic authority.

The runtime never executes effects directly. Effects go through the host-supplied
`InProcessEffectHandler`; authorization, hard-effect capability spending,
durable audit truth, retry, cancellation, secret handling and product
idempotency remain outside M08.1.

## M08.2 audit adapters

`causlane-runtime` keeps one audit boundary: `AuditLogPort::append`. The
in-memory adapter is available in the default build; durable SQL adapters are
feature-gated as `sqlite-audit` and `postgres-audit`.

All adapters share append admission:

```text
event -> unique event_id check -> monotonic event_index -> stable envelope -> append
```

Missing event indexes are assigned by the adapter. Supplied non-monotonic
indexes, duplicate event ids, overflow, schema/load failures and insert failures
return `AuditAdapterError`. Batch append helpers prepare the whole batch before
mutating adapter state; SQL batches commit in one transaction.

The durable schema stores the stable audit envelope (`event_index`, `event_id`,
`action_id`, `plan_hash`, `kind`, `correlation_id`, `causation_id`,
`occurred_at`, `impact_set_hash`, `drain_fence_scope`). Full replay payload
serialization remains outside M08.2.

## M08.3 executor adapters

Hard effects use the guarded executor seam. `ExecutorPort` remains the low-level
semantic adapter contract (`Op` plus `ExecutionCapability`), while
`GuardedExecutor` exposes the runtime service shape:

```text
GuardedExecutionRequest -> authz -> derive capability -> spend_admits -> execute
```

The runtime supplies `NoopExecutor` for no-op local composition and
`FunctionExecutor` for closure-backed tests/examples. These adapters do not
perform authz or capability validation themselves; the guarded seam centralizes
that order so each execution backend does not reimplement it.

## M08.4 Apalis adapter

`causlane-runtime/apalis` adds a feature-gated Apalis service bridge for
guarded execution jobs. The adapter uses Apalis's owned request envelope to carry
`GuardedExecutionJob`, then borrows it back into `GuardedExecutionRequest` and
calls the existing `ExecutorService`.

The bridge does not define a durable Causlane job schema and does not serialize
core domain objects. Durable Apalis storage payloads remain a later schema
boundary; M08.4 certifies the worker/service integration point and keeps authz,
capability derivation, spend-time admission and executor entry in one guarded
path.

## M08.5 Restate adapter

`causlane-runtime/restate` adds a feature-gated Restate handler/workflow bridge
for guarded execution jobs. Restate sees JSON handler input/output wrappers over
an opaque payload envelope and execution outcome; a host-owned decoder maps the
opaque bytes into `GuardedExecutionJob`.

The bridge journals the guarded execution result with Restate `ctx.run`, then
re-enters the same guarded path:

```text
Restate Json<opaque payload> -> host decoder -> GuardedExecutionJob
  -> GuardedExecutionRequest -> ExecutorService::call
```

The adapter does not define a canonical durable job schema, does not make
`causlane-core` serde-serializable and does not treat Restate context/state as
semantic authority. Durable payload compatibility remains a host or later schema
boundary.

## M09.1 benchmark suite

`crates/causlane` owns the first Criterion baseline suite:

```text
just bench-dispatch-baseline-build  # compile-only gate for CI/readiness
just bench-dispatch-baseline        # local measurement run
```

The suite measures the current release-promote hot-path and diagnostics surfaces:

- registry normalization from YAML;
- canonical plan hash computation;
- compiled bundle load from JSON;
- replay verification against a bundle;
- frontier selection with ready write-scope conflicts;
- exclusive lease grant on an empty lease table;
- execution barrier audit append;
- replay explain human rendering.

M09.1 records measurement coverage, not SLO enforcement. M09.6 defines the
stable operational SLO measurement contract; numeric thresholds remain a
host/release-profile gate.

## M09.2 partitioned dispatcher

Host dispatch v2 adds explicit `PartitionRoute` to every `HostTaskSpec`.
`PartitionRoute::primary` owns the task queue; `participants` name additional
partitions touched during admission. `PartitionRoute::acquisition_order()` is
the single ordering helper for `primary + participants`, so runtime code does
not duplicate cross-partition sorting.

`InProcessRuntime` coordinates admission, not durable distributed execution:

```text
submit_routed -> acquire route admission permits -> enqueue on primary -> release permits
try_submit_routed -> RouteBusy or QueueFull instead of waiting
```

Admission permits are held only until the primary partition replies to the
admission request. They are released before host effect execution, preserving
partition-local worker behavior and avoiding long-held async guards.

M09.2 does not add automatic tenant/domain parsing, durable queue storage,
distributed lease transactions, retry policy, or numeric SLO threshold
enforcement.

## M09.3 batched durability

`AuditLogPort` exposes both immediate single-event append and atomic batch
append through the same audit boundary:

```text
events -> shared batch admission -> stable envelopes -> atomic append
```

Batch admission is all-or-nothing. Duplicate ids, non-monotonic supplied
indexes, index overflow or storage failures leave adapter state unchanged.
In-memory, SQLite and Postgres adapters reuse the same `AuditAppendState`
preparation path; SQLite/Postgres persist the prepared batch in one transaction.

Write-ahead order is the caller's event order. A hard-effect execution batch
must place `ExecutionBarrierLogged` before `ExecutionStarted`; the adapter
persists that order and assigns monotonic indexes when missing. Tracing remains
derived: `TraceProjectingAuditLog` emits spans only after the authoritative
single append or batch append succeeds.

M09.3 does not introduce a new audit journal, full replay payload storage,
distributed group commit coordinator, retry policy, or numeric latency SLO
threshold enforcement.

## M09.4 backpressure policy

`InProcessRuntime` exposes a runtime-local `InProcessBackpressurePolicy` for
submit admission:

```text
Wait     -> wait for route admission and bounded ingress capacity
FailFast -> return RouteBusy or QueueFull instead of waiting
```

`submit` and `submit_routed` keep the `Wait` behavior. `try_submit` and
`try_submit_routed` are fail-fast aliases over the same internal helper, and
`submit_with_backpressure` / `submit_routed_with_backpressure` let hosts choose
the policy per call without changing `InProcessRuntimeConfig`.

Backpressure diagnostics reuse the existing runtime surface. `QueueFull` covers
both an ingress channel with no immediate capacity and a partition pending set
at its bound. `RouteBusy` covers fail-fast routed admission when any route
participant cannot be locked immediately. Partition keys remain the explicit
tenant/backpressure bucket supplied by the host.

M09.4 does not add automatic tenant parsing, priority scheduling, background
shedding workers, distributed queue storage, `RateLimit` enforcement, retry
policy, or numeric operational SLO threshold enforcement.

## M09.5 plan/template cache

`causlane-contracts` exposes a pure in-memory `PlanTemplateCache` for
memoizing canonical plan/template identity:

```text
PlanHashMaterial + compile snapshot refs -> canonical key hash
canonical key hash -> plan_hash + impact_set_hash
```

The cache key uses the same canonical JSON hasher as the rest of the contract
surface. Cache entries compute `plan_hash` through
`PlanHashMaterial::compute_plan_hash` and `impact_set_hash` through the existing
planned-impact helper, so the cache cannot mint alternate identities.

Compile snapshot refs are explicit `snapshot_id + snapshot_hash` inputs to
cache reuse. They are canonicalized by id/hash and validated as lowercase
`sha256:` tokens. These refs prevent stale cache reuse across compile-affecting
snapshots; they do not silently extend `PlanHashMaterial`. If a snapshot changes
the compiled plan identity, the planner must reflect that in the material per
ADR-0009.

M09.5 does not add a runtime LRU, TTL policy, durable cache, distributed cache,
new template resolver, generated schema or new planner semantics.

## M09.6 operational SLO catalog

`causlane-runtime` exposes the authoritative operational SLO catalog as
`OPERATIONAL_SLO_METRICS` plus
`validate_operational_slo_catalog`. The catalog is the single machine-readable
source for operational measurement shape; docs and release gates refer to these
stable metric ids instead of carrying a duplicate JSON artifact.

The required latency distributions are `p50` and `p95`, in milliseconds, for:

- `submit_latency_*`;
- `admission_latency_*`;
- `barrier_append_latency_*`;
- `replay_verify_latency_*`;
- `replay_explain_latency_*`.

The required gauges are:

- `partition_queue_depth`, counted as queued items;
- `constraint_snapshot_stale_age`, measured in milliseconds.

Every M09.6 metric has `HostDefined` threshold policy. The repo fixes the
measurement id, surface, unit, percentile shape and signal source, but it does
not set universal numeric SLO thresholds. Host deployments and release profiles
own those numbers because they depend on adapter, durability and telemetry
backend choices.

M09.6 does not add new OpenTelemetry export behavior, runtime rate limiting,
queue-depth enforcement, stale-snapshot rejection, replay semantics, generated
schemas or scenarios.

## Bounded queues

All runtime channels should be bounded:

```text
ingress;
planning;
frontier;
barrier;
execution lane;
projection;
observability.
```

## Durability classes

Not all actions should pay the same cost.

```text
StrictWriteAhead
  hard effects; durable barrier before execution.

BatchedWriteAhead
  hard effects with group commit window.

DispatchOnlyDurable
  modeled projection/meta flow, no execution barrier.

EphemeralDerived
  non-authoritative work that can be rebuilt.

Observability
  may be sampled/dropped under policy.
```

## Indexed graph updates

Do not compute all-pairs conflicts. Index by:

```text
FactSelectorKey;
ScopeKey;
ConflictDomain;
LeaseId;
ActionId;
LaneId;
ConstraintId;
TenantId.
```

A new observed fact wakes only waiters that require that fact. A released lease wakes only waiters on that scope.

## Explainability should be lazy

The hot path records compact blocker codes and references. Full explanation is built on demand.

## Optimization boundary

Allowed:

- precompile contracts;
- cache plan templates;
- intern IDs;
- use bitsets/small vectors;
- batch writes;
- shard partitions;
- async observability;
- immutable constraint snapshots.

Forbidden:

- execute hard effects without barrier;
- use logs as truth;
- skip dispatch log;
- trust worker payload as authority;
- let lane selection bypass graph/constraints;
- let dynamic overlay weaken obligations.
