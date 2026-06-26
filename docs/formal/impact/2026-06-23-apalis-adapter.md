# Formal Impact Record: Apalis adapter (M08.4)

## Change metadata

- Change ID: FIR-2026-06-23-apalis-adapter
- PR/issue: S08 / M08.4 Apalis adapter
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (runtime adapter surface) - Apalis worker service bridge only

## Touched protocol-critical paths

```text
Cargo.lock
crates/causlane-runtime/Cargo.toml
crates/causlane-runtime/src/adapters/apalis.rs
crates/causlane-runtime/src/adapters/mod.rs
crates/causlane-runtime/src/guarded_executor.rs
docs/06-runtime-and-performance.md
docs/07-security-and-authz.md
docs/product-track/09-runtime-adapter-track.md
docs/product-track/milestones/m08.4-apalis-adapter.md
```

## Summary

M08.4 adds an optional `causlane-runtime/apalis` feature. The adapter wraps an
existing guarded execution service as an Apalis/Tower service over an owned
`GuardedExecutionJob`.

The service bridge borrows that owned job back into `GuardedExecutionRequest`
and delegates to `ExecutorService::call`, so the authority order remains the
same single implementation:

```text
authz -> derive capability -> spend_admits -> ExecutorPort::execute
```

Apalis request context, extensions and worker metadata are non-authoritative.
They cannot supply authz evidence, alter policy identity, mint capabilities, or
bypass spend-time admission.

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
coverage-matrix field changes. M08.4 is a runtime service adapter over the
existing guarded execution seam.
```

## Affected protocols

```text
PR-runtime-adapter-boundary: runtime adapters spend host/kernel authority and
must not create semantic authority. M08.4 exposes an Apalis service bridge under
that boundary only.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Core Rust API changed: none.
- Runtime Rust API added:
  `ExpectedAuthzPolicy`, `GuardedExecutionJob`,
  `ApalisGuardedExecutionRequest`, and `ApalisGuardedExecutor` behind
  `causlane-runtime/apalis`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| missing authz via Apalis request | runtime unit | `Unauthorized` and inner executor is not reached | new |
| expired capability via Apalis request | runtime unit | `CapabilityRefused::Expired` before executor entry | new |
| Apalis metadata authority injection | runtime unit | request extensions do not create authz evidence | new |
| Apalis service shape | runtime unit | adapter implements `tower::Service<Request<GuardedExecutionJob, ()>>` | new |
| owned job borrow shape | runtime unit | `GuardedExecutionJob::as_request` matches original borrowed request | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (runtime) | Apalis adapter tests | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Replay and generated formal lanes already validate execution-barrier/capability
semantics from audit traces. M08.4 does not add a durable Apalis payload schema
and does not make core domain types serde-serializable.

## Acceptance commands

```bash
./tools/cargo-dev check -p causlane-runtime --lib --locked
./tools/cargo-dev check -p causlane-runtime --all-targets --features apalis --locked
./tools/cargo-dev test -p causlane-runtime --all-targets --features apalis --locked
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just coverage-clean && just coverage
./tools/schema-validate-all
./tools/product-track-status-check
./tools/formal-exceptions-check
just formal-coverage-matrix-check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m08.4-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: durable Apalis storage payload schema, if product demand
  requires Redis/Postgres/SQLite job persistence.
