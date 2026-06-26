# ADR-0025: Chaos/recovery evidence is bounded to runtime-owned semantics

## Status

Accepted.

## Context

M09.7 needs executable evidence for slow handlers, provider unavailability,
retry behavior, drain under load and partition restart. The in-process runtime
is intentionally ephemeral: it owns bounded queues, partition admission and
host-effect scheduling, but it does not own durable truth, retry policy,
distributed leases or recovery orchestration.

## Decision

M09.7 treats chaos/recovery readiness as bounded executable evidence over the
runtime surfaces that exist today:

- slow host handlers must leave overload visible through stable fail-fast
  diagnostics;
- provider failure must fail closed as `InProcessRuntimeEvent::Failed` and must
  not be reported as execution success;
- retry is host-owned: duplicate idempotency keys are suppressed, and a new
  host-owned attempt must use distinct task/idempotency material;
- drain under load does not create linear drain authority for the in-process
  runtime, which continues to advertise `supports_linear_drain = false`;
- partition restart is an ephemeral rejoin smoke test, not durable recovery of
  in-flight or persisted work.

`docs/product-track/chaos-recovery-matrix.json` is the machine-readable M09.7
evidence ledger. It maps each bounded scenario to runtime unit tests and records
the residual risks that remain future work.

## Consequences

- No host-dispatch schema, replay trace schema, Formal IR schema, generated
  model or scenario catalog changes are introduced.
- No public runtime API changes are introduced.
- M09.7 closes the prose-only readiness gap for the covered in-process runtime
  scenarios.
- Hard-effect retry interleavings, cancellation/supersession, durable drain
  orchestration and persisted recovery remain future formal/runtime-depth work,
  primarily M10.2.
