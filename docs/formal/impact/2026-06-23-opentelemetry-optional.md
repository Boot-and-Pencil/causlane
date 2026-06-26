# Formal Impact Record: optional OpenTelemetry adapter (M07.4)

## Change metadata

- Change ID: FIR-2026-06-23-opentelemetry-optional
- PR/issue: S07 / M07.4 OpenTelemetry optional
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (runtime observability surface) — derived telemetry only

## Touched protocol-critical paths

```text
crates/causlane-runtime/Cargo.toml
crates/causlane-runtime/src/adapters/mod.rs
crates/causlane-runtime/src/adapters/otel.rs
docs/product-track/milestones/m07.4-opentelemetry-optional.md
```

## Summary

M07.4 adds a feature-gated OpenTelemetry sink in `causlane-runtime`. The feature
is optional (`otel`) and the default runtime build stays dependency-minimal. The
adapter consumes the M07.3 `TraceSpan` projection and emits OpenTelemetry spans,
logs and a low-cardinality counter. It does not classify `AuditEventKind`; the
single audit-event projection remains in `causlane-core`.

The sink owns local SDK providers instead of installing global OpenTelemetry
providers. OTLP construction uses HTTP/protobuf with a blocking client and no
tonic/grpc requirement. Causlane ids are preserved as OpenTelemetry attributes;
the adapter does not fabricate OpenTelemetry trace/span ids from causlane ids.

## Affected invariants

```text
I-003: unchanged — projection truth anchors are still validated from audit events,
       not from telemetry spans/logs.
I-008: unchanged — lifecycle authority remains the audit event stream.
ADR-0014 / TD-017: observability remains derived, not truth; M07.4 adds an
       external exporter over the typed projection.
new invariant ids: none
```

## Affected formal models

```text
none — no bundle, Formal IR, replay trace, generated model or coverage schema
changes. OpenTelemetry records are not consumed by replay/formal lanes.
```

## Affected protocols

```text
PR-observability-derived: telemetry is derived from `TraceSpan`, which is already
derived from committed audit events. No dispatch, barrier, lease, authz,
lifecycle or replay protocol semantics change.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public Rust API added behind `causlane-runtime/otel`: `OtlpHttpConfig`,
  `OpenTelemetryFlushError`, `OpenTelemetryTraceSink`,
  `CAUSLANE_OTEL_SCOPE`, `CAUSLANE_TRACE_SPANS_TOTAL`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| OTel adapter receives a `TraceSpan` | runtime unit | emits span/log/metric without reading `AuditEventKind` mapping | new |
| high-cardinality ids exist on a span | runtime unit | ids are span/log attributes, not metric dimensions | new |
| violation span | runtime unit | OpenTelemetry span status is error | new |
| default build | cargo check | runtime compiles without `otel` feature/deps active | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (runtime) | OpenTelemetry adapter tests | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Replay and generated formal lanes do not consume OpenTelemetry output. They
continue to consume the audit/event trace and bundle-derived artifacts.
OpenTelemetry export is a runtime telemetry adapter over the already-derived
`TraceSpan` model, so runtime unit tests are the applicable lane for this
behavior.

## Acceptance commands

```bash
./tools/cargo-dev check -p causlane-runtime --lib --locked
./tools/cargo-dev check -p causlane-runtime --all-targets --features otel
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just coverage
./tools/schema-validate-all
./tools/product-track-status-check
./tools/formal-exceptions-check
./tools/coverage-matrix --check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m07.4-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: M07.5 redaction policy for audit/log/projection/replay/support-bundle classes.
