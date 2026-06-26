# Formal Impact Record: in-process runtime (M08.1)

## Change metadata

- Change ID: FIR-2026-06-23-in-process-runtime
- PR/issue: S08 / M08.1 In-process runtime
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (runtime adapter surface) - ephemeral scheduling only

## Touched protocol-critical paths

```text
Cargo.lock
crates/causlane-runtime/Cargo.toml
crates/causlane-runtime/src/lib.rs
crates/causlane-runtime/src/in_process/mod.rs
crates/causlane-runtime/src/in_process/tests.rs
docs/06-runtime-and-performance.md
docs/product-track/milestones/m08.1-in-process-runtime.md
```

## Summary

M08.1 adds a feature-gated Tokio in-process runtime in `causlane-runtime`.
The feature is optional (`tokio-runtime`) and the default runtime build remains
dependency-minimal. The runtime owns fixed partition loops, bounded partition
queues, partition-local dependency state, and one runtime-wide semaphore for
host effect handler capacity.

The runtime is intentionally ephemeral. It does not persist queue state, record
audit truth, decide authorization, derive/spend hard-effect capabilities, retry
failed work, or resolve product idempotency. Work executes only through the
host-supplied `InProcessEffectHandler` after `validate_host_task` accepts the
stable host task shape.

## Affected invariants

```text
I-003: unchanged - projection truth remains audit/event anchored, not runtime
       event output.
I-007: unchanged - drain/fence semantics remain in the kernel/formal lanes.
I-008: unchanged - lifecycle authority remains audit/replay input, not in-process
       runtime scheduling events.
ADR-0011 / ADR-0013 / M06.6: unchanged - authz and capability spending remain
       outside this adapter and are not weakened by partition scheduling.
new invariant ids: none
```

## Affected formal models

```text
none - no bundle, Formal IR, replay trace, generated model, schema, receipt or
coverage-matrix field changes. In-process runtime events are adapter diagnostics,
not replay/formal inputs.
```

## Affected protocols

```text
PR-runtime-adapter-boundary: runtime adapters spend host-provided authority and
must not create semantic authority. M08.1 adds bounded in-process scheduling
under that boundary only.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public Rust API added behind `causlane-runtime/tokio-runtime`:
  `InProcessRuntime`, `InProcessRuntimeConfig`, `InProcessRuntimeError`,
  `InProcessRuntimeEvent`, `InProcessEffectHandler`, `InProcessEffectFuture`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| invalid host task | runtime unit | rejected through existing host API error metadata | new |
| duplicate idempotency key | runtime unit | suppressed partition-locally before enqueue | new |
| bounded queue full | runtime unit | `try_submit` returns `QueueFull` | new |
| semaphore capacity one | runtime unit | handler calls do not overlap | new |
| handler failure | runtime unit | task is not completed and dependents remain blocked | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (runtime) | in-process runtime tests | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Replay and generated formal lanes do not consume in-process runtime events. They
continue to consume audit/replay traces and generated formal artifacts. M08.1 is
a runtime adapter over the stable host-facing task seam, so runtime unit tests
are the applicable lane for bounded queue, partition, and capacity behavior.

## Acceptance commands

```bash
./tools/cargo-dev check -p causlane-runtime --lib --locked
./tools/cargo-dev check -p causlane-runtime --all-targets --features tokio-runtime --locked
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just coverage-clean && just coverage
./tools/schema-validate-all
./tools/product-track-status-check
./tools/formal-exceptions-check
just formal-coverage-matrix-check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m08.1-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: M08.2 audit adapters for append-only in-memory/SQLite/Postgres
  audit paths and group commit policies.
