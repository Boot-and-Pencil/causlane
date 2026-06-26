# causlane-runtime

`causlane-runtime` contains runtime composition shells and adapter skeletons for
Causlane.

## Status

This crate is experimental and pre-alpha. It is not a production runtime,
workflow engine or queue. Adapter guarantees are intentionally narrow and may
change before `0.1`.

## Role In The Workspace

The crate hosts in-process dispatch/reference runtime pieces and optional
integration adapters. The pure protocol and kernel contracts remain in
`causlane-core`.

## Public API Entry Points

- `LinearHostDispatcher` for the linear host-dispatch reference path.
- In-process coordinator and worker types for local composition.
- Runtime adapter modules behind optional feature flags.
- Operational SLO catalog and projection guard helpers.

## In-Process Runtime Retention

The `tokio-runtime` in-process runtime is ephemeral. Each partition owns bounded
pending queue state and bounded history state. `partition_history_bound` controls
how many completed task ids, failed task ids and idempotency keys are retained
per partition.

Within that window, duplicate suppression and dependency readiness keep their
normal semantics. After eviction, an old completion no longer satisfies a new
dependency and an old idempotency key may be reused. Durable idempotency and
long-lived dependency history remain host responsibilities.

## Features

- `tokio-runtime`: enables the in-process runtime, shadow comparison and Tokio
  runtime dependency.
- `sqlite-audit`: enables the SQLite append-only audit adapter.
- `postgres-audit`: enables the Postgres append-only audit adapter.
- `otel`: enables the optional OpenTelemetry export adapter.
- `apalis`: enables the Apalis guarded execution bridge.
- `restate`: enables the Restate guarded execution bridge and its serde-backed
  payload wrapper.

Default features are empty. Enabling a runtime adapter does not create semantic
authority; hard effects still go through the guarded executor and capability
checks.
