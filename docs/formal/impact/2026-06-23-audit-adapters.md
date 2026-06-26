# Formal Impact Record: audit adapters (M08.2)

## Change metadata

- Change ID: FIR-2026-06-23-audit-adapters
- PR/issue: S08 / M08.2 Audit adapters
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (runtime adapter surface) - audit persistence boundary only

## Touched protocol-critical paths

```text
Cargo.lock
crates/causlane-core/src/domain/audit.rs
crates/causlane-runtime/Cargo.toml
crates/causlane-runtime/src/adapters/audit.rs
crates/causlane-runtime/src/adapters/audit/postgres.rs
crates/causlane-runtime/src/adapters/audit/sqlite.rs
crates/causlane-runtime/src/adapters/tracing.rs
docs/06-runtime-and-performance.md
docs/product-track/09-runtime-adapter-track.md
docs/product-track/milestones/m08.2-audit-adapters.md
```

## Summary

M08.2 replaces the placeholder in-memory audit adapter with append-only runtime
audit adapters. The public audit boundary remains
`causlane_core::AuditLogPort::append`; no duplicate audit port or async audit
API is introduced.

The default runtime build includes an in-memory append-only adapter. Optional
features add durable SQL adapters:

- `causlane-runtime/sqlite-audit` using `rusqlite`;
- `causlane-runtime/postgres-audit` using `postgres`.

All adapters share one append admission path for unique event ids, monotonic
event indexes, overflow checks and batch preparation. SQL adapters persist the
shared `AuditEnvelope` and commit batch appends transactionally.

## Affected invariants

```text
I-003: unchanged - projection truth remains anchored in committed audit events.
I-007: unchanged - drain/fence semantics remain in the kernel/formal lanes.
I-008: unchanged - lifecycle authority remains audit/replay input.
ADR-0003: strengthened at adapter boundary - audit journal append is fail-closed
          and append-only for supported runtime adapters.
ADR-0011 / ADR-0013: unchanged - authz and capability validity are not decided
                     by storage adapters.
new invariant ids: none
```

## Affected formal models

```text
none - no bundle, Formal IR, replay trace, generated model, schema, receipt or
coverage-matrix field changes. The durable schema stores a runtime audit
envelope and is not a replay trace schema.
```

## Affected protocols

```text
PR-runtime-adapter-boundary: runtime adapters spend host/kernel authority and
must not create semantic authority. M08.2 persists accepted audit envelopes
under that boundary only.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Core Rust API added:
  `AuditEventKind::stable_token`.
- Runtime Rust API added:
  `AuditAdapterError`, `AuditEnvelope`, append-only `InMemoryAuditLog`,
  `SqliteAuditLog` behind `sqlite-audit`, and `PostgresAuditLog` behind
  `postgres-audit`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| duplicate event id | runtime unit | append returns `DuplicateEventId` and state is unchanged | new |
| non-monotonic supplied event index | runtime unit | append returns `NonMonotonicEventIndex` | new |
| duplicate inside batch | runtime unit | batch append is all-or-nothing | new |
| SQLite duplicate id | runtime unit | durable append fails before state advances | new |
| SQLite batch duplicate | runtime unit | transaction rolls back and next append starts at index 0 | new |
| Postgres DDL/insert drift | runtime unit | DDL/insert constants preserve append-only envelope keys | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (runtime) | audit adapter tests | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Replay and generated formal lanes do not consume the M08.2 SQL envelope schema.
The replay trace format, bundle schema, Formal IR and generated artifacts remain
unchanged. Full replay-payload serialization for durable event-store use is a
later schema boundary.

## Acceptance commands

```bash
./tools/cargo-dev check -p causlane-runtime --lib --locked
./tools/cargo-dev check -p causlane-runtime --all-targets --features sqlite-audit --locked
./tools/cargo-dev check -p causlane-runtime --all-targets --features postgres-audit --locked
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just coverage-clean && just coverage
./tools/schema-validate-all
./tools/product-track-status-check
./tools/formal-exceptions-check
just formal-coverage-matrix-check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m08.2-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: M08.3 executor port/adapters.
