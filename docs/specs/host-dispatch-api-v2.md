# Host Dispatch API v2

- API version: `causlane.host-dispatch.v2`
- Primary code seam: `causlane_core::integration`
- Reference implementations: `causlane_runtime::LinearHostDispatcher` and `causlane_runtime::InProcessRuntime`
- Intended consumers: host applications that need a stable dispatch boundary with explicit partition routing.

## Purpose

`causlane.host-dispatch.v2` extends the v1 host seam with typed partition routing.
Hosts submit a `HostTaskSpec` that names a `PartitionRoute`; Causlane validates
the route and the in-process runtime coordinates admission for every partition in
the route without turning the runtime into a durable distributed scheduler.

This is a generic crates.io-style library boundary. Product projects map their
own contracts into `HostDispatchContext` and `HostTaskSpec` outside `causlane`;
`causlane` does not depend on Hopium-specific DTOs, schemas, contracts, or
business vocabulary.

## Stable interface

The v2 surface consists of:

```text
CAUSLANE_HOST_API_VERSION = causlane.host-dispatch.v2
HostDispatchContext
HostDispatchContextBuilder
HostTaskSpec
HostTaskSpecBuilder
PartitionKey
PartitionRoute
HostEffectClass
HostRuntimeProfile
HostDispatcherCapabilities
HostDispatchTicket
HostDrainOutcome
HostEffectOutcome
HostDispatchError
HostEffectHandler
HostDispatchPort
validate_host_context
validate_host_task
validate_host_submission
```

`HostTaskSpec` carries a required `partition_route`. `PartitionRoute::primary`
is the owning partition; `participants` are additional partitions touched by the
task.

## Contract

A host dispatch implementation must:

1. reject any `HostTaskSpec.host_api_version` other than `causlane.host-dispatch.v2`;
2. reject empty required context refs and empty optional context refs when supplied;
3. reject empty task ids, action ids, predicate ids, subject refs, optional task refs when supplied, forbidden effects, and empty primary or participant partition keys;
4. reject empty dependency ids, self-dependencies, and duplicate dependency ids;
5. reject `HardEffect` tasks without a non-empty host task idempotency key;
6. use `PartitionRoute::acquisition_order()` for cross-partition admission ordering;
7. dedupe and sort `primary + participants` through that single helper;
8. reject explicit submits whose supplied partition differs from `partition_route.primary`;
9. keep host authorization, durable idempotency policy, effect execution, secret handling, and product DTO translation outside the dispatcher core;
10. advertise whether multi-partition admission coordination is supported.

## In-process coordinator semantics

`InProcessRuntime` owns one bounded queue per configured partition. Routed submit
APIs use `task.partition_route.primary` as the queue owner:

```text
submit_routed(ctx, task)     -> waits for route locks and queue admission
try_submit_routed(ctx, task) -> returns RouteBusy or QueueFull instead of waiting
```

The in-process reference also exposes
`submit_routed_with_backpressure(ctx, task, policy)` for hosts that want to
choose the wait/fail-fast overload mode explicitly per call. This remains a
runtime adapter policy, not a new host-dispatch protocol requirement.

For each routed admission, the runtime acquires one admission permit per
partition in `PartitionRoute::acquisition_order()`. Permits are held only until
the owning partition returns its admission response. They are released before
host effect execution, so the coordinator orders admission and avoids deadlock
without serializing long-running effects.

Each partition also retains bounded history for completed task ids, failed task
ids and idempotency keys. `InProcessRuntimeConfig::partition_history_bound`
controls that window. Within the window, dependency readiness and duplicate
suppression keep the same behavior; after eviction, old completions no longer
satisfy newly submitted dependents and old idempotency keys may be reused. Hosts
that need durable idempotency or long-lived dependency history must provide it
outside this in-process adapter.

## Non-goals

This API does not provide distributed lease transactions, durable queue storage,
cross-process consensus, automatic tenant/domain derivation, retry policy,
priority scheduling, background shedding, `RateLimit` enforcement, or
operational SLO enforcement. The M09.6 operational SLO catalog in
`causlane-runtime` defines the required measurement surfaces separately from
the host-dispatch protocol.
