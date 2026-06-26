# ADR-0017: Host dispatch API v2 partition coordinator

- Status: accepted
- Date: 2026-06-23
- Supersedes: ADR-0016 for the current host API version
- Related: ADR-0005, ADR-0008, ADR-0013, ADR-0016

## Context

M08 introduced an in-process runtime with partition-owned queues, while the
stable host API remained v1 and had no typed route. M09.2 needs explicit
partition routing and deterministic cross-partition admission ordering without
freezing Causlane as a workflow engine or distributed scheduler.

## Decision

Introduce `causlane.host-dispatch.v2` with `PartitionKey` and `PartitionRoute`
in `causlane-core`. Every `HostTaskSpec` carries a route. The route's
`acquisition_order()` is the only helper runtime code may use to order
cross-partition admission.

`InProcessRuntime` adds `submit_routed` and `try_submit_routed`. It coordinates
admission by using one permit per route partition in deterministic order.
Wait-mode submit validates the route, reserves primary ingress capacity, then
acquires route permits and sends through the reserved ingress slot. Fail-fast
submit keeps the immediate route-permit acquisition and `try_send` path so
overload diagnostics remain non-blocking. In both modes, route permits are
released after the partition returns its admission response. Effect execution
remains partition-local and host-owned.

## Consequences

Benefits:

```text
host API versioning honestly reflects the task shape change;
route ordering has one typed source of truth;
reversed multi-partition routes cannot deadlock admission;
wait-mode does not hold participant permits while waiting for primary ingress;
existing partition workers, queues, dependency readiness and effect handling stay reused.
```

Costs:

```text
all host task builders must provide a partition route;
v1 consumers need an adapter or migration to v2;
the coordinator is admission-only and does not provide durable distributed isolation.
```

## Compatibility

`causlane.host-dispatch.v1` remains documented as a historical linear seam.
Current code validates `causlane.host-dispatch.v2`; any v1 compatibility adapter
must be explicit and preserve v1 behavior.
