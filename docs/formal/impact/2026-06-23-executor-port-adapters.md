# Formal Impact Record: executor port/adapters (M08.3)

## Change metadata

- Change ID: FIR-2026-06-23-executor-port-adapters
- PR/issue: S08 / M08.3 Executor port/adapters
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (runtime adapter surface) - guarded execution seam only

## Touched protocol-critical paths

```text
crates/causlane-runtime/src/adapters/executor.rs
crates/causlane-runtime/src/adapters/mod.rs
crates/causlane-runtime/src/guarded_executor.rs
docs/06-runtime-and-performance.md
docs/07-security-and-authz.md
docs/product-track/09-runtime-adapter-track.md
docs/product-track/milestones/m08.3-executor-port-adapters.md
```

## Summary

M08.3 adds a dependency-free service-shaped executor seam around the existing
runtime `GuardedExecutor`. The core `ExecutorPort` remains unchanged and still
requires a scoped `ExecutionCapability`.

The new `GuardedExecutionRequest`, `ExecutionOutcome` and `ExecutorService`
types provide a Tower-like shape without importing `tower`. `GuardedExecutor`
implements that service by sharing the same helper as `spend_barrier`, so the
authority order remains single-sourced:

```text
authz -> derive capability -> spend_admits -> ExecutorPort::execute
```

The executor adapter module now exposes `NoopExecutor` and `FunctionExecutor`
for local/test composition.

## Affected invariants

```text
I-003: unchanged - projection truth remains audit/event anchored.
I-007: unchanged - drain/fence semantics remain in the kernel/formal lanes.
I-008: unchanged - lifecycle authority remains audit/replay input.
ADR-0011: unchanged - authz remains deny-by-default before execution.
ADR-0013 / M06.6: preserved - hard effects still run only after capability
                  derivation and spend-time admission.
new invariant ids: none
```

## Affected formal models

```text
none - no bundle, Formal IR, replay trace, generated model, schema, receipt or
coverage-matrix field changes. M08.3 is a runtime adapter API refactor over the
already-covered authz/capability execution seam.
```

## Affected protocols

```text
PR-runtime-adapter-boundary: runtime adapters spend host/kernel authority and
must not create semantic authority. M08.3 exposes a service-shaped guarded
executor seam under that boundary.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Core Rust API changed: none.
- Runtime Rust API added:
  `GuardedExecutionRequest`, `ExecutionOutcome`, `ExecutorService`,
  closure-backed `FunctionExecutor`, and documented `NoopExecutor`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| missing authz via service call | runtime unit | `Unauthorized` and inner executor is not reached | new |
| expired capability via service call | runtime unit | `CapabilityRefused::Expired` before executor entry | new |
| legacy/service drift | runtime unit | `spend_barrier` and `ExecutorService::call` produce equivalent success output | new |
| function adapter routing | runtime unit | closure receives exact `Op` and `ExecutionCapability` | new |
| no-op adapter | runtime unit | derived capability executes with empty produced refs | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (runtime) | guarded executor service tests | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Replay and generated formal lanes already validate execution-barrier/capability
semantics from audit traces. M08.3 does not alter trace shape, replay inputs,
bundle schema, Formal IR or generated artifacts.

## Acceptance commands

```bash
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just coverage-clean && just coverage
./tools/schema-validate-all
./tools/product-track-status-check
./tools/formal-exceptions-check
just formal-coverage-matrix-check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m08.3-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: M08.4 Apalis adapter.
