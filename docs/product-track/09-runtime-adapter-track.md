# 09. Runtime and adapter track

## Principle

Runtime adapters spend the contract; they do not create semantics.

## Core runtime shell

- Tokio partition loops by primary conflict domain / tenant / subject.
- Bounded queues for ingress/planning/barrier/execution/projection/observability.
- Lane semaphores for capacity; durable leases for correctness.
- Group commit for audit/barrier where allowed.
- Guarded executor API only; no raw execute for hard effects.

## Persistence adapters

- In-memory append-only audit adapter for tests and examples.
- SQLite local/dev audit adapter behind `causlane-runtime/sqlite-audit`.
- Postgres production/server audit adapter behind `causlane-runtime/postgres-audit`.
- Audit adapters store the stable audit envelope first; full event-store/CQRS
  payload serialization remains optional later.
- Group commit is exposed through the same audit port: `AuditLogPort::append`
  for immediate single-event writes and `AuditLogPort::append_batch` for
  all-or-nothing ordered batch writes.

## Execution adapters

- Guarded executor service seam first; hard effects go through
  `GuardedExecutor` rather than raw adapter calls.
- `NoopExecutor` and `FunctionExecutor` cover local/test composition without
  creating execution authority.
- Apalis service bridge behind `causlane-runtime/apalis`; durable storage
  payload schema remains separate.
- Restate handler/workflow bridge behind `causlane-runtime/restate`; durable
  payload schema remains separate.
- Temporal/Dapr/Conductor experimental/community unless demand is clear.

## Policy adapters

- Cedar first for embedded fine-grained AuthZ.
- Casbin for simple RBAC/domain model.
- AuthZEN for external PDP interoperability.
- OpenFGA/SpiceDB for ReBAC as optional adapters.

## Observability adapters

- tracing connector first.
- JSON logs for local/dev.
- OpenTelemetry optional.
- Logging/metrics never become observed truth.

## Adapter certification suite

M08.7 codifies a bounded certification matrix for adapters that exist today.
Execution-bearing adapters pass by showing simulation to `GuardedExecutor`:

- hard effect cannot start before barrier;
- executor validates capability;
- adapter envelope/metadata cannot create semantic authority;
- authorized execution reaches the executor exactly once;
- produced refs survive the adapter wrapper.

Audit and observability adapters certify their own boundaries:

- append-only audit state rejects duplicate/non-monotonic truth writes;
- observability spans emit only after successful audit append;
- observability failure does not affect correctness.

`docs/product-track/adapter-certification-matrix.json` is the machine-readable
evidence ledger. M09.7 adds bounded in-process retry/idempotency evidence, while
hard-effect retry interleavings, cancellation/supersession and durable
truth-commit orchestration remain deferred to formal-depth milestones that make
those semantics first-class.

## Chaos/recovery readiness

M09.7 adds bounded chaos/recovery evidence for the feature-gated in-process
runtime. `docs/product-track/chaos-recovery-matrix.json` records the executable
scenarios:

- slow host handlers keep fail-fast overload visible;
- provider unavailability fails closed as `Failed`, never `Executed`;
- retry is host-owned and duplicate idempotency keys are suppressed;
- routed contention under load reports `RouteBusy` without deadlock;
- partition restart is an ephemeral rejoin smoke test.

This does not turn execution adapters into durable recovery authorities. Hard
effect retry interleavings, cancellation/supersession and persisted drain or
recovery orchestration remain future formal-depth work.

## Shadow mode diagnostics

M08.8 adds a bounded shadow comparer for the feature-gated in-process runtime.
Host integrations subscribe to `InProcessRuntimeEvent`, supply
`ShadowExpectation` values, and call `compare_shadow_events` to receive a
`ShadowComparison`.

The comparer is diagnostic-only:

- it never admits, schedules, retries, cancels, blocks, or executes work;
- accepted events match by task lifecycle outcome, not ticket sequence;
- partition-scoped expectations can disambiguate duplicate task ids;
- rejected/failed expectations can match any error or an exact
  `HostDispatchError`;
- missing, mismatched, and unexpected observations are returned as data.

Full migration playbooks and reference integration rollout guidance remain in
the later M12.3 migration/shadow docs milestone.
