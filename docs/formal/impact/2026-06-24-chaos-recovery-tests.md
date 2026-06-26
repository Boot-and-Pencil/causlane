# Formal Impact Record: Chaos/recovery tests (M09.7)

## Change metadata

- Change ID: FIR-2026-06-24-chaos-recovery-tests
- PR/issue: S09 / M09.7 Chaos/recovery tests
- Owner: repo maintainers
- Date: 2026-06-24
- Impact class: F2 (runtime readiness evidence, no schema or model change)

## Touched protocol-critical paths

```text
crates/causlane-runtime/src/in_process/tests.rs
crates/causlane-runtime/src/in_process/tests/recovery.rs
docs/adr/0025-chaos-recovery-semantics.md
docs/product-track/chaos-recovery-matrix.json
docs/product-track/milestones/m09.7-chaos-recovery-tests.md
docs/product-track/stages/s09-performance-reliability.md
docs/product-track/09-runtime-adapter-track.md
```

## Summary

M09.7 adds bounded executable chaos/recovery evidence for the feature-gated
in-process runtime. The tests cover slow handler overload visibility, provider
unavailability, host-owned retry/idempotency safety, routed contention under
load and ephemeral partition restart.

The change does not add new runtime authority. It records current limits:
retry policy remains host-owned, the in-process runtime has no linear drain
authority, and restart evidence is an ephemeral rejoin smoke test rather than
durable recovery.

## Affected invariants

```text
I-001: unchanged - barrier/capability semantics are not modified.
I-002: unchanged - observed truth ordering is not modified.
I-006: strengthened by executable negative controls for duplicate retry keys.
ADR-0019: preserved - fail-fast overload still reports RouteBusy/QueueFull.
ADR-0025: added - bounded chaos/recovery evidence and residual risks.
new invariant ids: none.
```

## Affected formal models

```text
FM-013 Runtime adapter/port compliance model: strengthened by runtime recovery
tests and chaos-recovery-matrix.json.
FM-014 Idempotence/retry/duplicate execution model: partially evidenced for
in-process host-owned retry; hard-effect retry interleavings remain M10.2.
No generated Alloy/P/Kani/Verus/Lean artifacts are changed.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Core Rust API changed: none.
- Runtime public API changed: none.
- Test-only runtime module added: `in_process::tests::recovery`.
- Machine-readable product artifact added:
  `docs/product-track/chaos-recovery-matrix.json`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| Provider unavailable | runtime unit | `Failed`, never `Executed` | new |
| Duplicate retry key after failure | runtime unit | `DuplicateSuppressed` | new |
| Routed contention under load | runtime unit | `RouteBusy`, no deadlock | new |
| Ephemeral partition restart | runtime unit | fresh runtime accepts new work only | new |

## Deferred controls

| Scenario | Deferred to | Reason |
|---|---|---|
| hard-effect worker retry interleavings | M10.2 | first-class hard-effect retry protocol is not implemented |
| cancellation/supersession during retry | M10.1 / M10.2 | cancellation invariants remain future semantics |
| durable drain and persisted recovery | M10.2 | in-process runtime remains ephemeral and advertises no linear drain authority |

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-runtime --all-targets --all-features --locked recovery
./tools/cargo-dev fmt --all --check
git diff --check
just check
just clippy
just test
just contract-test
just refactor-readiness
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m09-7-changed-files.txt
just formal-ready
just formal-coverage-matrix-check
./tools/product-track-status-check
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: promote hard-effect retry interleavings, durable drain and
  persisted recovery into first-class runtime/formal semantics.
