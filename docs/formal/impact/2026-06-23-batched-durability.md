# Formal Impact Record: M09.3 batched durability

## Change metadata

- Change ID: FIR-2026-06-23-batched-durability
- PR/issue: M09.3 Batched durability
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (runtime adapter durability boundary)

## Touched protocol-critical paths

```text
crates/causlane-core/src/application/ports.rs
crates/causlane-runtime/src/adapters/audit.rs
crates/causlane-runtime/src/adapters/audit/sqlite.rs
crates/causlane-runtime/src/adapters/audit/postgres.rs
crates/causlane-runtime/src/adapters/tracing.rs
docs/adr/0018-batched-audit-durability.md
docs/product-track/adapter-certification-matrix.json
```

## Summary

M09.3 promotes audit batch append to the existing `AuditLogPort` boundary. The
runtime keeps one audit port and one batch admission implementation:
`AuditAppendState::prepare_batch`. In-memory, SQLite and Postgres adapters
validate the full batch before mutating state; SQL adapters commit a prepared
batch in one transaction. Tracing remains derived and emits batch spans only
after the authoritative audit append succeeds.

## Affected invariants

```text
ADR-0003: strengthened at adapter boundary - batch audit append is fail-closed
          and ordered.
ADR-0008: unchanged - observability remains derived after successful audit
          append.
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

- Core Rust API changed: `AuditLogPort` now requires `append_batch`.
- Runtime Rust adapters implement `append_batch` on the port.
- Durable SQL envelope schema: unchanged.
- Replay trace/scenario/Formal IR schemas: unchanged.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| barrier + execution batched together | runtime unit | barrier row/event index precedes execution row/event index | new |
| duplicate id inside batch | runtime unit | batch fails and adapter state is unchanged | new |
| non-monotonic supplied index inside batch | runtime unit | batch fails and adapter state is unchanged | new |
| tracing wrapper batch failure | runtime unit | no spans emitted when audit append fails | new |
| SQLite batch write | runtime unit | ordered rows persist transactionally | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Rust | audit/tracing batch durability tests | no | rust |
| Product docs | ADR-0018 + M09.3 track updates | no | docs |
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
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m09-3-batched-durability-changed-files.txt
./tools/formal-exceptions-check
./tools/schema-validate-all
./tools/product-track-status-check
just formal-coverage-matrix-check
just bench-m09-1-build
just bench-m09-1
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
