# Formal Impact Record: M09.1 bench suite

## Change metadata

- Change ID: FIR-2026-06-23-bench-suite
- PR/issue: M09.1 Bench suite
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (developer/operator performance diagnostics)

## Touched protocol-critical paths

```text
crates/causlane/Cargo.toml
crates/causlane/benches/m09_1_bench_suite.rs
justfile
docs/06-runtime-and-performance.md
docs/product-track/bench-suite-matrix.json
docs/product-track/milestones/m09.1-bench-suite.md
docs/product-track/stages/s09-performance-reliability.md
```

## Summary

This change adds a Criterion benchmark harness for the M09.1 baseline surfaces:
registry normalization, plan hash computation, bundle load, replay verification,
frontier conflict selection, lease grant, barrier audit append, and replay
explain rendering.

The harness reuses existing release-promote fixtures and domain APIs. It does
not alter runtime, replay, formal, or contract semantics.

## Affected invariants

```text
I-001: unchanged - replay barrier/execution checks are only measured.
I-002: unchanged - observed truth ordering checks are only measured.
I-003: unchanged - projection anchor checks are only measured.
I-006: unchanged - lease conflict primitives are only measured.
I-007: unchanged - drain semantics are not changed.
I-008: unchanged - lifecycle/replay checks are only measured.
I-009: unchanged - plan/impact/witness/authz binding semantics are unchanged.
new invariant ids: none
```

## Affected formal models

```text
none - no formal contour, Formal IR schema, generated model artifact, scenario,
or replay trace schema changes.
```

## Affected protocols

```text
PR-bench-suite: adds measurement harnesses only. The bench binary calls existing
public APIs and records no new authority, replay, audit, or dispatch rule.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public API added/changed/removed: none.
- Dev-only dependencies added: Criterion 0.7 for the `causlane` bench target.

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| bench target compile gate | cargo bench build | `just bench-m09-1-build` compiles the harness | new |
| benchmark matrix consistency | product track | benchmark IDs and threshold policy are documented | new |
| latency threshold enforcement | product track | explicitly deferred to M09.6 | not applicable |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Rust bench build | `m09_1_bench_suite` | no | rust |
| Product track | `bench-suite-matrix.json` | no | docs |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | proof/all |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Formal model and generated-artifact lanes are not regenerated because this
change does not modify protocol semantics, schemas, Formal IR, scenarios, or
generated monitors. M09.1 is a baseline measurement pass, not a correctness
claim or SLO gate.

## Acceptance commands

```bash
./tools/cargo-dev fmt --all --check
just bench-m09-1-build
just check
just clippy
just test
just coverage-clean && just coverage
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m09-1-bench-suite-changed-files.txt
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
