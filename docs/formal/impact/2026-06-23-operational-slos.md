# Formal Impact Record: M09.6 operational SLO catalog

## Change metadata

- Change ID: FIR-2026-06-23-operational-slos
- PR/issue: M09.6 Operational SLOs
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F1 (runtime diagnostics/readiness contract)

## Touched protocol-critical paths

```text
crates/causlane-runtime/src/operational_slo.rs
crates/causlane-runtime/src/lib.rs
docs/adr/0021-operational-slo-catalog.md
docs/06-runtime-and-performance.md
docs/specs/host-dispatch-api-v2.md
```

## Summary

M09.6 adds a typed operational SLO measurement catalog in
`causlane-runtime`. The catalog fixes stable metric ids, measurement shape,
units, percentile requirements and signal-source boundaries for submit,
admission, barrier append, replay verify, replay explain, queue depth and
constraint snapshot stale-age diagnostics.

The change is structural only. It does not set numeric thresholds, alter runtime
admission semantics, extend OpenTelemetry export behavior, reject stale
snapshots or change replay verification/explain behavior.

## Affected invariants

```text
ADR-0008: unchanged - observability remains derived and non-authoritative.
ADR-0017: unchanged - host dispatch v2 routing/admission semantics are unchanged.
ADR-0019: unchanged - backpressure policy semantics are unchanged.
ADR-0021: new - operational SLO measurement shape is a typed runtime catalog.
I-001: unchanged - execution authority is not modified.
I-006: unchanged - conflict/merge semantics are not modified.
I-008: unchanged - lifecycle/replay semantics are not modified.
new invariant ids: none
```

## Affected formal models

```text
none - no bundle, replay trace, Formal IR, generated model, scenario, receipt
or coverage-matrix schema changes.
```

## Contract changes

- Runtime Rust API changed: `OperationalSloMetricId`,
  `OperationalSloSurface`, `OperationalSloMeasure`, `OperationalSloUnit`,
  `OperationalSloPercentile`, `OperationalSloSignalSource`,
  `OperationalSloThresholdPolicy`, `OperationalSloMetric`,
  `OperationalSloMetricField`, `OperationalSloCatalogError`,
  `M09_6_OPERATIONAL_SLO_METRICS`, `operational_slo_metric` and
  `validate_operational_slo_catalog` are added.
- Existing host-dispatch, replay trace/scenario and Formal IR schemas are
  unchanged.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| duplicate metric id | runtime unit | catalog validation rejects duplicate id | new |
| missing metric id | runtime unit | catalog validation rejects missing required id | new |
| latency percentile removed | runtime unit | catalog validation rejects percentile drift | new |
| gauge percentile added | runtime unit | catalog validation rejects gauge percentile | new |
| stale snapshot age unit changed | runtime unit | catalog validation rejects unit drift | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Rust | operational_slo unit tests | no | rust |
| Product docs | ADR-0021 + M09.6 track updates | no | docs |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | proof/all |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Acceptance commands

```bash
./tools/cargo-dev fmt --all --check
./tools/cargo-dev test -p causlane-runtime operational_slo --locked
just check
just clippy
just test
just coverage-clean && just coverage
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m09-6-operational-slos-changed-files.txt
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
