# Formal Impact Record: Restate adapter (M08.5)

## Change metadata

- Change ID: FIR-2026-06-23-restate-adapter
- PR/issue: S08 / M08.5 Restate adapter
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (runtime adapter surface) - Restate handler/workflow bridge only

## Touched protocol-critical paths

```text
Cargo.lock
crates/causlane-runtime/Cargo.toml
crates/causlane-runtime/src/adapters/restate.rs
crates/causlane-runtime/src/adapters/mod.rs
crates/causlane-runtime/src/adapters/apalis.rs
crates/causlane-runtime/src/lib.rs
crates/causlane-runtime/src/test_support.rs
docs/06-runtime-and-performance.md
docs/07-security-and-authz.md
docs/product-track/09-runtime-adapter-track.md
docs/product-track/milestones/m08.5-restate-adapter.md
```

## Summary

M08.5 adds an optional `causlane-runtime/restate` feature. The adapter wraps an
existing guarded execution service as a Restate handler/workflow bridge over
`Json<RestateGuardedExecutionPayload>`.

The Restate-visible payload is an opaque byte envelope. A host-owned decoder
turns that envelope into the existing `GuardedExecutionJob`; the adapter then
borrows it back into `GuardedExecutionRequest` and delegates to
`ExecutorService::call`. The authority order remains the same single
implementation:

```text
authz -> derive capability -> spend_admits -> ExecutorPort::execute
```

Restate context, state, workflow ids and handler metadata are non-authoritative.
They cannot supply authz evidence, alter policy identity, mint capabilities, or
bypass spend-time admission. Decode failures and guarded execution failures map
to Restate terminal errors by default.

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
coverage-matrix field changes. M08.5 is a runtime handler/workflow adapter over
the existing guarded execution seam.
```

## Affected protocols

```text
PR-runtime-adapter-boundary: runtime adapters spend host/kernel authority and
must not create semantic authority. M08.5 exposes a Restate bridge under that
boundary only.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Core Rust API changed: none.
- Runtime Rust API added behind `causlane-runtime/restate`:
  `RestateGuardedExecutionPayload`, `RestateExecutionOutcome`,
  `RestateGuardedExecutionInput`, `RestateGuardedExecutionOutput`,
  `RestateGuardedJobDecoder`, `RestateAdapterError`,
  `RestateErrorMapper`, `TerminalRestateErrors`, and
  `RestateGuardedExecutor`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| missing authz via Restate payload | runtime unit | `Unauthorized` and inner executor is not reached | new |
| expired capability via Restate payload | runtime unit | `CapabilityRefused::Expired` before executor entry | new |
| decoder failure | runtime unit | decode error and inner executor is not reached | new |
| Restate payload metadata authority injection | runtime unit | payload bytes do not create authz evidence | new |
| outcome conversion | runtime unit | produced refs survive Restate output wrapper conversion | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (runtime) | Restate adapter tests | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Replay and generated formal lanes already validate execution-barrier/capability
semantics from audit traces. M08.5 does not add a durable Restate payload schema
and does not make core domain types serde-serializable.

## Acceptance commands

```bash
./tools/cargo-dev check -p causlane-runtime --all-targets --features restate --locked
./tools/cargo-dev test -p causlane-runtime --all-targets --features restate --locked
./tools/cargo-dev check -p causlane-runtime --all-targets --features apalis,restate --locked
./tools/cargo-dev test -p causlane-runtime --all-targets --features apalis,restate --locked
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just coverage-clean && just coverage
./tools/schema-validate-all
./tools/product-track-status-check
./tools/formal-exceptions-check
just formal-coverage-matrix-check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m08.5-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: durable Restate payload schema, if product demand requires a
  stable Redis/HTTP/Kafka job format.
