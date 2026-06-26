# Formal Impact Record: R1 formal orchestrator extraction

## Change metadata

- Change ID: FIR-2026-06-24-formal-orchestrator-extraction
- PR/issue: Patch pack 014 R1 formal orchestration split
- Owner: repo maintainers
- Date: 2026-06-24
- Impact class: F1 (behavior-preserving formal CLI refactor)

## Touched protocol-critical paths

```text
crates/causlane-cli/src/bin/causlane-formal.rs
crates/causlane-cli/src/formal_generate.rs
crates/causlane-cli/src/app/formal/*
```

## Summary

R1 extracts formal artifact generation, stale checks, receipt writing and coverage
derivation into a shared `causlane_cli::app::formal::FormalOrchestrator`
service. The `causlane` and `causlane-formal` binaries now keep CLI parsing,
filesystem/clock adapters and exit-code rendering at the boundary while sharing
the same orchestration implementation.

This is a behavior-preserving refactor. It does not change replay semantics,
Formal IR semantics, generated artifact schema, receipt schema, scenario schema
or coverage matrix semantics.

## Affected invariants

```text
ADR-0015: unchanged - formal evidence remains generated and receipt-bound.
ADR-0022: advanced - R1 moves formal orchestration behind a shared app service.
I-001: unchanged - execution authority semantics are not modified.
I-006: unchanged - conflict/merge semantics are not modified.
I-008: unchanged - lifecycle/replay semantics are not modified.
I-009: unchanged - witness/authz/anchor grounding remains generated from scenarios.
new invariant ids: none
```

## Affected formal models

```text
none - no model, generated artifact, scenario, receipt schema, Formal IR schema
or coverage matrix changes.
```

## Contract changes

- New Rust library surface:
  `causlane_cli::app::formal::{FormalOrchestrator, FormalIo, *Request}`.
- Existing CLI commands and flags are unchanged:
  `causlane formal generate`, `causlane formal stale-check`,
  `causlane formal ir emit`, `causlane-formal verify-all`,
  `causlane-formal coverage`.
- Filesystem and clock access are explicit `FormalIo` boundary ports.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| `execution_without_barrier_invalid` | replay/alloy | refuted by replay and generated Alloy | unchanged |
| `conflicting_leases_invalid` | replay/alloy | refuted by replay and generated Alloy | unchanged |
| missing/generated receipt | coverage | lane remains not-run/invalid, never greened | unchanged |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Rust | shared formal service tests | no | base |
| CLI | `causlane-formal` wrapper test | no | base |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | proof/all |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Acceptance commands

```bash
./tools/cargo-dev fmt --all --check
just architecture-lint
just refactor-readiness
just check
just clippy
just test
just contract-test
just formal-ready
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-r1-formal-orchestrator-changed-files.txt
just formal-coverage-matrix-check
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
