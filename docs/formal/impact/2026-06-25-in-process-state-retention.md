# Formal Impact Record: in-process state retention

## Change metadata

- Change ID: FIR-2026-06-25-in-process-state-retention
- PR/issue: M2 runtime state growth finding
- Owner: repo maintainers
- Date: 2026-06-25
- Impact class: F2 (runtime adapter state behavior, no schema or model change)

## Touched protocol-critical paths

```text
crates/causlane-runtime/src/in_process/mod.rs
crates/causlane-runtime/src/in_process/worker.rs
crates/causlane-runtime/src/in_process/tests.rs
crates/causlane-runtime/src/in_process/tests/retention.rs
crates/causlane-runtime/README.md
docs/specs/host-dispatch-api-v2.md
docs/refactor/code-review-finding-resolution-matrix-2026-06-25.md
docs/release/publication-blockers-dispatcher-020.md
```

## Summary

The feature-gated in-process runtime now bounds partition-local history state.
Each partition retains completed task ids, failed task ids and idempotency keys
through one FIFO bounded-set helper. `InProcessRuntimeConfig` exposes
`partition_history_bound` so the retention window is explicit instead of an
unbounded implementation detail.

Within the retention window, duplicate suppression and dependency readiness keep
their existing behavior. After eviction, old completions no longer satisfy newly
submitted dependents and old idempotency keys may be reused. Durable idempotency
and long-lived dependency history remain host responsibilities.

## Affected formal models

```text
none - no bundle, Formal IR, replay trace, generated model, schema, receipt or
coverage-matrix field changes. In-process runtime state remains adapter-local
diagnostics and scheduling state, not replay/formal input.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Runtime public API changed: `InProcessRuntimeConfig` adds
  `partition_history_bound`.
- Runtime behavior changed: completed, failed and idempotency history is bounded
  per partition.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| History bound smaller than queue bound | runtime unit | config rejects `partition_history_bound` | new |
| Idempotency key evicted from history | runtime unit | key may be reused after eviction | new |
| Completed task id evicted from history | runtime unit | new dependent remains blocked | new |

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-runtime --all-targets --features tokio-runtime --locked retention
./tools/cargo-dev test -p causlane-runtime --all-targets --features tokio-runtime --locked
./tools/cargo-dev fmt --all --check
./tools/cargo-dev clippy -p causlane-runtime --all-targets --features tokio-runtime --locked -- -D warnings
python3 tools/pre-publication-review-gate --json
python3 tools/architecture-lint --json
git diff --check
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: none for M2; durable idempotency and long-lived dependency
  history remain external host responsibilities for this pre-alpha runtime.
