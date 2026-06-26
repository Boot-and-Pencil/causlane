# Formal Impact Record: Adapter certification (M08.7)

## Change metadata

- Change ID: FIR-2026-06-23-adapter-certification
- PR/issue: S08 / M08.7 Adapter certification
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (runtime adapter certification surface)

## Touched protocol-critical paths

```text
crates/causlane-runtime/src/adapters/certification.rs
crates/causlane-runtime/src/adapters/apalis.rs
crates/causlane-runtime/src/adapters/restate.rs
crates/causlane-runtime/src/adapters/mod.rs
crates/causlane-runtime/src/test_support.rs
docs/product-track/adapter-certification-matrix.json
docs/product-track/04-readiness-gates.md
docs/product-track/09-runtime-adapter-track.md
docs/product-track/milestones/m08.7-adapter-certification.md
```

## Summary

M08.7 adds a bounded certification matrix for the runtime adapters that exist
today. The Rust certification harness reuses the shared guarded-execution test
fixtures and runs Apalis and Restate through the same behavioral checks:

```text
adapter envelope -> GuardedExecutionJob -> GuardedExecutor -> ExecutorPort
```

The certification pass does not add new adapter authority. It demonstrates that
execution-bearing adapters simulate the existing guarded executor contract:
authorization happens before executor entry, capability refusal happens before
executor entry, adapter metadata is non-authoritative, and produced refs survive
the wrapper.

## Affected invariants

```text
I-001: covered by adapter negative controls - execution-bearing adapters cannot
       execute when authorization/barrier evidence is missing.
I-002: unchanged - observed truth semantics and replay ordering are unchanged.
I-003: unchanged - projection anchor semantics are unchanged.
ADR-0011: preserved - authz remains deny-by-default before execution.
ADR-0013 / M06.6: preserved - hard effects still run only after capability
                  derivation and spend-time admission.
new invariant ids: none.
```

## Affected formal models

```text
FM-013 Runtime adapter/port compliance model: strengthened by shared runtime
tests and a machine-readable certification matrix.
FM-014 Idempotence/retry/duplicate execution model: unchanged and still future;
M08.7 records retry/idempotency as deferred, not implemented.
No generated Alloy/P/Kani/Verus/Lean artifacts are changed.
```

## Affected protocols

```text
PR-runtime-adapter-boundary: adapters spend host/kernel authority but do not
create semantic authority.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Core Rust API changed: none.
- Runtime public API changed: none.
- Test-only runtime module added: `adapters::certification`.
- Machine-readable product artifact added:
  `docs/product-track/adapter-certification-matrix.json`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| Apalis missing authz | runtime unit | `Unauthorized`, executor calls remain zero | new |
| Apalis expired capability | runtime unit | `CapabilityRefused::Expired`, executor calls remain zero | new |
| Apalis request metadata authority injection | runtime unit | metadata/extensions do not create authz | new |
| Restate missing authz | runtime unit | `Unauthorized`, executor calls remain zero | new |
| Restate expired capability | runtime unit | `CapabilityRefused::Expired`, executor calls remain zero | new |
| Restate payload metadata authority injection | runtime unit | opaque bytes do not create authz | new |

## Deferred controls

| Scenario | Deferred to | Reason |
|---|---|---|
| retry cannot double-execute hard effect | M10.2 | M09.7 covers bounded in-process host-owned retry; hard-effect worker retry remains future |
| cancellation/supersession respected | M10.1 / M10.2 | cancellation invariants are future formal/runtime semantics |
| durable observed-truth commit orchestration | M10.2 | M09.7 records that execution bridges do not own durable recovery or truth-commit orchestration |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (runtime) | Shared adapter certification tests | no | rust |
| Product artifact | adapter-certification-matrix.json | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | proof/all |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Replay and generated formal lanes already validate lifecycle, barrier,
capability and projection semantics from audit traces. M08.7 does not add new
runtime events, retry/cancellation protocols, or durable payload schemas.

## Acceptance commands

```bash
./tools/cargo-dev check -p causlane-runtime --all-targets --locked
./tools/cargo-dev test -p causlane-runtime --all-targets --locked
./tools/cargo-dev check -p causlane-runtime --all-targets --features apalis,restate --locked
./tools/cargo-dev test -p causlane-runtime --all-targets --features apalis,restate --locked
./tools/cargo-dev fmt --all --check
git diff --check
just check
just clippy
just test
just coverage-clean && just coverage
./tools/schema-validate-all
./tools/product-track-status-check
./tools/formal-exceptions-check
just formal-coverage-matrix-check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m08.7-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: promote retry/idempotency, cancellation/supersession and
  durable truth-commit orchestration into first-class runtime/formal semantics.
