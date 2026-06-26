# Formal Impact Record: Patch-pack 015 doc readiness handoff

## Change metadata

- Change ID: FIR-2026-06-24-doc-readiness-handoff
- PR/issue: Patch pack 015 documentation readiness
- Owner: repo maintainers
- Date: 2026-06-24
- Impact class: F1/F2 (readiness docs plus formal helper boundary parity)

## Touched protocol-critical paths

```text
crates/causlane-cli/src/bin/causlane-formal.rs
crates/causlane-cli/src/bin/formal_runtime/io.rs
docs/formal-readiness-status.md
docs/product-track/*
docs/refactor/*
```

## Summary

This change applies the relevant patch-pack 015 recommendations after the R1
formal orchestrator extraction. It does not replace the existing
`causlane-formal-discipline` implementation. It adds milestone handoff docs,
fixes stale formal-readiness prose and extends the dedicated `causlane-formal`
helper with the all-target generation/stale-check commands already provided by
`FormalOrchestrator`.

The change is readiness and boundary parity work. It does not change replay
semantics, Formal IR semantics, generated artifact schemas, receipt schemas,
scenario schemas or coverage semantics.

## Affected invariants

```text
ADR-0015: unchanged - formal evidence remains generated and receipt-bound.
ADR-0022: advanced - milestone handoff now references refactor/readiness gates.
I-001: unchanged - execution authority semantics are not modified.
I-006: unchanged - conflict/merge semantics are not modified.
I-008: unchanged - lifecycle/replay semantics are not modified.
I-009: unchanged - witness/authz/anchor grounding remains generated from scenarios.
new invariant ids: none
```

## Affected formal models

```text
none - no model, generated artifact, Formal IR, receipt schema, scenario schema
or coverage matrix changes.
```

## Contract changes

- `causlane-formal generate-all` delegates to
  `FormalOrchestrator::generate_all`.
- `causlane-formal stale-check-all` delegates to
  `FormalOrchestrator::stale_check_all`.
- `causlane-formal verify-all` and `coverage` remain unchanged.
- `FormalIo::read_dir_paths` returns deterministic sorted entries.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| `execution_without_barrier_invalid` | replay/alloy | refuted by replay and generated Alloy | unchanged |
| `conflicting_leases_invalid` | replay/alloy | refuted by replay and generated Alloy | unchanged |
| missing/generated receipt | coverage | lane remains not-run/invalid, never greened | unchanged |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Rust | `causlane-formal` all-target wrapper test | no | base |
| Replay | unchanged | no | base |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | proof/all |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Formal model lanes are not changed because this is a CLI/documentation
readiness update. The executable evidence is the existing generated-artifact
stale-check/coverage contour plus the new helper-binary wrapper test.

## Acceptance commands

```bash
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just contract-test
just refactor-readiness
just formal-ready
just formal-verify-all
just formal-coverage-matrix-check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-patch-pack-015-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
