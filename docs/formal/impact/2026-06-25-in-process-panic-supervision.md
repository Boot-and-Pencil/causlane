# Formal Impact Record: in-process panic supervision

## Change metadata

- Change ID: FIR-2026-06-25-in-process-panic-supervision
- PR/issue: H2 runtime supervision finding
- Owner: repo maintainers
- Date: 2026-06-25
- Impact class: F2 (runtime adapter behavior, no schema or model change)

## Touched protocol-critical paths

```text
crates/causlane-runtime/src/in_process/worker.rs
crates/causlane-runtime/src/in_process/tests.rs
crates/causlane-runtime/src/in_process/tests/recovery.rs
docs/refactor/code-review-finding-resolution-matrix-2026-06-25.md
docs/release/publication-blockers-dispatcher-020.md
```

## Summary

The feature-gated in-process runtime now supervises host effect handler panics.
A handler panic before future creation or while the returned future is polled is
reported as `InProcessRuntimeEvent::Failed` with
`HostDispatchError::HandlerRejected`, and the partition worker continues
processing later independent tasks.

This does not add semantic authority to the runtime. Handler failures still do
not complete tasks, failed dependencies still block dependents, and retry policy
remains host-owned.

## Affected formal models

```text
none - no bundle, Formal IR, replay trace, generated model, schema, receipt or
coverage-matrix field changes. In-process runtime events remain adapter
diagnostics, not replay/formal inputs.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Runtime public API changed: none.
- Runtime behavior changed: panicking host effect handlers are surfaced as
  failed task events instead of terminating the partition worker.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| Handler panics before future creation | runtime unit | `Failed`, later independent task `Executed` | new |
| Handler panics during future polling | runtime unit | `Failed`, later independent task `Executed` | new |

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-runtime --all-targets --features tokio-runtime --locked handler_panic
./tools/cargo-dev test -p causlane-runtime --all-targets --features tokio-runtime --locked
./tools/cargo-dev fmt --all --check
./tools/cargo-dev clippy -p causlane-runtime --all-targets --features tokio-runtime --locked -- -D warnings
tools/pre-publication-review-gate
tools/formal-discipline-check --profile all --no-diff --json
git diff --check
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: M1 participant-permit backpressure semantics and M2
  partition-state retention semantics remain separate publication-track work.
