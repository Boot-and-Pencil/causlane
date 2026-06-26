# Host Dispatch API v1

- API version: `causlane.host-dispatch.v1`
- Status: historical; superseded by [`host-dispatch-api-v2`](host-dispatch-api-v2.md)
- Primary code seam: `causlane_core::integration`
- Reference implementation: `causlane_runtime::LinearHostDispatcher`
- Intended consumers: host applications that need a stable dispatch boundary before binding to Causlane internals.

## Purpose

Causlane has a rich internal semantic dispatch kernel: action grammar, predicates, plans, capabilities, barriers, leases, witnesses, constraints, replay, and formal evidence. Host projects should not couple to those internals while they are still evolving.

`causlane.host-dispatch.v1` provides a deliberately small, versioned, host-facing seam. A host can submit a task, drain ready work through its own effect handler, and inspect dispatcher capabilities. The host remains responsible for policy, secrets, session context, config snapshots, idempotency, effect execution, and product safety gates.

## Stable interface

The stable v1 surface consists of:

```text
CAUSLANE_HOST_API_VERSION
HostDispatchContext
HostTaskSpec
HostEffectClass
HostRuntimeProfile
HostDispatcherCapabilities
HostDispatchTicket
HostDrainOutcome
HostEffectOutcome
HostDispatchError
HostEffectHandler
HostDispatchPort
validate_host_task
```

The reference runtime implementation is:

```text
LinearHostDispatcher
```

## Contract

A `HostDispatchPort` implementation must:

1. reject any `HostTaskSpec.host_api_version` other than `causlane.host-dispatch.v1`;
2. reject `HostEffectClass::Forbidden` at the admission boundary;
3. reject empty `task_id` values;
4. advertise capabilities before use;
5. keep `supports_parallelism = false` for the linear reference path;
6. drain at most one ready task per `drain_once` call;
7. execute only through `HostEffectHandler`; no implementation may perform host effects directly;
8. treat observability as derived evidence, not as an authority source;
9. accept only object/reference payloads, never raw secret values;
10. keep host authorization and idempotency outside the dispatcher core for v1.

## Refinery/M-CR mapping

Refinery Stage 8 should map its cross-layer foundation types as follows:

```text
Refinery CrossLayerContext -> HostDispatchContext
Refinery TaskSpec          -> HostTaskSpec
Refinery DispatcherBridge  -> HostDispatchPort
Refinery effect callback   -> HostEffectHandler
Refinery telemetry/logging -> derived events from ticket/drain outcomes
```

Refinery must still run its own RBAC, secret redaction, config snapshot, session, retry, timeout, and kill-switch checks before it submits a task. Causlane v1 advertises `requires_external_authz = true` and `requires_external_idempotency = true` so this ownership remains explicit.

## Linear reference semantics

`LinearHostDispatcher` is deterministic and in-memory:

```text
submit -> validate -> idempotency-key duplicate suppression -> enqueue
drain_once -> pick first queued task whose dependencies are completed -> handler -> mark completed
```

If the queue is empty, `drain_once` returns `Idle`. If tasks exist but none are dependency-ready, it returns `Blocked`.

## Acceptance tests

The host API is accepted when these tests exist and pass:

```text
causlane_core::integration::tests::host_api_rejects_wrong_version
causlane_core::integration::tests::host_api_rejects_forbidden_effect
causlane_core::integration::tests::linear_reference_capability_keeps_parallelism_disabled
causlane_runtime::linear_host::tests::linear_dispatcher_executes_ready_tasks_in_dependency_order
causlane_runtime::linear_host::tests::linear_dispatcher_reports_blocked_when_no_dependency_is_ready
causlane_runtime::linear_host::tests::linear_dispatcher_suppresses_duplicate_idempotency_keys
```

## Non-goals

This API does not expose production scheduling, durable queues, async runtimes, distributed leases, external persistence, or formal proof witnesses directly. Those remain internal Causlane capabilities that can later back the same host-facing port without breaking host projects.
