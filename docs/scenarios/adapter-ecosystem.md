# Adapter Ecosystem Guide

M12.4 documents how external adapters fit around Causlane without becoming a
second semantic authority. The existing M08.7 adapter certification matrix is
the bounded evidence ledger for adapters that exist in this repository today;
this guide explains the interface and certification expectations for future
ecosystem adapters.

## Adapter classes

External adapters should identify exactly one primary adapter class:

- **execution bridge:** maps a backend job, service request or workflow payload
  into a guarded Causlane execution job;
- **audit adapter:** persists audit events while preserving append-only,
  monotonic observed-truth ordering;
- **authz adapter:** maps Causlane authorization coordinates into an external
  policy engine without making the policy engine a lifecycle authority;
- **observability adapter:** emits logs, spans or metrics derived from Causlane
  events without creating observed truth;
- **registry or bundle adapter:** loads contract artifacts without changing the
  canonical bundle or plan hash semantics.

Adapters may implement more than one class only when each boundary is certified
independently.

## Semantic boundary

Adapters spend or carry the contract; they do not define it. An adapter must not:

- decide semantic admissibility outside the kernel;
- create observed truth outside audit append;
- bypass hard-effect barriers, leases, witnesses or capabilities;
- treat backend metadata as authorization evidence;
- weaken replay, redaction or projection invariants;
- become a hidden workflow engine, job queue or scheduler.

If a backend provides retry, cancellation, deduplication or workflow state, that
behavior remains host/backend behavior until Causlane has first-class protocol
semantics and evidence for it.

## Certification path

External adapter maintainers should add certification evidence before claiming
compatibility:

1. Identify the implemented port or adapter class.
2. Document failure semantics and idempotency behavior.
3. Define capability, lease, witness and authz handling.
4. Define audit ordering and observed-truth boundaries.
5. Define observability and redaction behavior.
6. Document unsupported consequence profiles.
7. Add positive and negative controls.
8. Link tests to a machine-readable certification artifact.

Execution bridges should demonstrate the existing guarded-executor contract:

- no execution before barrier;
- executor requires scoped capability;
- adapter metadata is not authority;
- authorized execution reaches the executor exactly once;
- produced refs survive the adapter wrapper.

Audit and observability adapters should demonstrate their own boundaries:

- audit append rejects duplicate or non-monotonic truth writes;
- audit batches are all-or-nothing where batching is supported;
- telemetry/logging happens after the authority event;
- telemetry failure does not affect correctness.

## Current bounded evidence

`docs/product-track/adapter-certification-matrix.json` records current bounded
evidence for:

- `runtime.apalis`;
- `runtime.restate`;
- `runtime.audit`;
- `runtime.tracing`.

That matrix is the source of truth for in-repository certification status.
Prose may summarize it, but cannot upgrade an adapter from deferred or bounded
status to production-certified status.

## Unsupported claims

M12.4 does not certify production readiness for external providers. The
following remain deferred:

- durable retry and cancellation semantics for hard effects;
- durable observed-truth commit orchestration;
- provider-specific deployment or CI integration;
- stable semver compatibility for post-1.0 adapter APIs;
- certification for adapters that have no tests or machine-readable evidence.
