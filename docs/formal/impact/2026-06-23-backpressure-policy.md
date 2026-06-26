# Formal Impact Record: M09.4 backpressure policy

## Change metadata

- Change ID: FIR-2026-06-23-backpressure-policy
- PR/issue: M09.4 Backpressure policy
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (runtime adapter admission boundary)

## Touched protocol-critical paths

```text
crates/causlane-runtime/src/in_process/mod.rs
crates/causlane-runtime/src/in_process/worker.rs
crates/causlane-runtime/src/in_process/tests/backpressure.rs
crates/causlane-runtime/src/lib.rs
docs/adr/0019-in-process-backpressure-policy.md
```

## Summary

M09.4 makes in-process runtime overload behavior explicit through
`InProcessBackpressurePolicy`. Existing wait-mode APIs and fail-fast APIs are
preserved, while new per-call policy methods select the same behavior without a
second admission implementation. `QueueFull` and `RouteBusy` remain the runtime
diagnostic surface for overload decisions.

## Affected invariants

```text
ADR-0017: unchanged - partition route ordering remains the single coordinator
          ordering source.
ADR-0019: new - in-process overload policy is explicit per submit and shares
          one admission path.
I-001: unchanged - execution authority is not modified.
I-007: unchanged - drain/fence semantics are not modified.
I-008: unchanged - lifecycle/replay semantics are not modified.
new invariant ids: none
```

## Affected formal models

```text
none - no bundle, replay trace, Formal IR, generated model, scenario, receipt
or coverage-matrix schema changes.
```

## Contract changes

- Runtime Rust API changed: `InProcessBackpressureMode`,
  `InProcessBackpressurePolicy`, `submit_with_backpressure` and
  `submit_routed_with_backpressure` are added.
- Existing `submit`, `submit_routed`, `try_submit` and `try_submit_routed`
  behavior is preserved.
- Host dispatch API v2 wire/schema surface: unchanged.
- Replay trace/scenario/Formal IR schemas: unchanged.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| explicit wait policy | runtime unit | accepted event matches default submit behavior | new |
| routed wait policy | runtime unit | declared primary partition owns admission | new |
| explicit fail-fast policy | runtime unit | `QueueFull` error/event matches `try_submit` surface | new |
| route busy fail-fast | runtime unit | coordinator emits `RouteBusy` when a route permit is held | existing |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Rust | in-process backpressure tests | no | rust |
| Product docs | ADR-0019 + M09.4 track updates | no | docs |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | proof/all |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Acceptance commands

```bash
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just coverage-clean && just coverage
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m09-4-backpressure-policy-changed-files.txt
./tools/formal-exceptions-check
./tools/schema-validate-all
./tools/product-track-status-check
just formal-coverage-matrix-check
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
