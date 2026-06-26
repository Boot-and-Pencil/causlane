# Formal Impact Record: derived tracing connector (M07.3)

## Change metadata

- Change ID: FIR-2026-06-23-derived-tracing-connector
- PR/issue: S07 / M07.3 tracing connector
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (protocol-adjacent core surface) — derived observability only

## Touched protocol-critical paths

```text
crates/causlane-core/src/domain/audit.rs
crates/causlane-core/src/domain/mod.rs
crates/causlane-core/src/domain/tracing.rs
```

## Summary

M07.3 adds a typed `AuditEvent -> TraceSpan` projection in `causlane-core` and a
runtime adapter that emits one span after a successful `AuditLogPort::append`.
The audit/event journal remains the only authority for observed truth. Spans are
diagnostic projections over already-recorded events and are not replay inputs.

The change deliberately keeps the mapping in one core function,
`trace_span_from_audit_event`, so runtime sinks and future exporters do not
reclassify event semantics. The runtime wrapper is fail-open for telemetry sink
errors and fail-closed for audit append errors: if the audit append fails, no
span is recorded.

## Affected invariants

```text
I-003: unchanged — projection truth anchors are still validated from audit events,
       not from trace spans.
I-008: unchanged — lifecycle authority remains the audit event stream.
ADR-0014 / TD-017: observability is derived, not truth; now backed by a typed
       projection and negative runtime controls.
new invariant ids: none
```

## Affected formal models

```text
none — no bundle, Formal IR, replay trace, generated model or coverage schema
changes. Tracing spans are not consumed by replay/formal lanes.
```

## Affected protocols

```text
PR-observability-derived: tracing/logging projections are derived from audit events.
No dispatch, barrier, lease, authz, lifecycle or replay protocol semantics change.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public Rust API added: `TraceSpan`, `TraceSpanKind`, `TraceAttribute`,
  `TraceSpanId`, `trace_span_from_audit_event`,
  `trace_span_kind_from_audit_event_kind`, `ALL_AUDIT_EVENT_KINDS`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| every audit event kind maps through the single projector | core unit | span kind is total over `ALL_AUDIT_EVENT_KINDS` | new |
| optional audit fields become typed attributes only when present | core unit | no empty-field attributes are synthesized | new |
| audit append failure | runtime unit | no span is recorded | new |
| trace sink failure | runtime unit | audit append still succeeds | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (core/runtime) | tracing projector + adapter tests | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Replay and generated formal lanes do not consume spans. They continue to consume
the audit/event trace and bundle-derived artifacts. This change adds a derived
observability view over that authority, so unit tests are the applicable lane
for the new projection and runtime sink behavior.

## Acceptance commands

```bash
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just coverage
./tools/schema-validate-all
./tools/product-track-status-check
./tools/formal-exceptions-check
./tools/coverage-matrix --check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m07.3-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: M07.4 external OpenTelemetry/OTLP exporter using the same
  typed span projection.
