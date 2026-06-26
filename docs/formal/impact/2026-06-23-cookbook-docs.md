# Formal Impact Record: cookbook docs closeout (M07.7)

## Change metadata

- Change ID: FIR-2026-06-23-cookbook-docs
- PR/issue: M07.7 cookbook docs
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (developer/operator documentation and DX surface)

## Touched protocol-critical paths

```text
docs/scenarios/cookbook.md
docs/README.md
docs/product-track/00-executive-roadmap.md
docs/product-track/01-product-track-map.md
docs/product-track/02-milestone-catalog.md
docs/product-track/roadmap.json
docs/product-track/roadmap.yaml
docs/product-track/stages/s07-observability-explainability-dx.md
docs/product-track/stages/s08-runtime-adapters.md
docs/product-track/milestones/m07.7-cookbook-docs.md
```

## Summary

M07.7 adds a cookbook page that points users to existing executable fixtures and
CLI surfaces for action authoring, approval/witness debugging, conflict and
drain diagnosis, replay explain, contract tests, authz scenarios, projection
checks, support bundles and adapter-boundary checks.

The change also closes M07.7 in the product track, marks S07 advanced in-repo,
and moves the active next stage to S08 runtime/adapters.

## Affected invariants

```text
I-001: unchanged - cookbook points to existing replay/barrier/capability checks.
I-002: unchanged - observed-truth ordering semantics are not changed.
I-003: unchanged - projection anchor examples use existing scenarios.
I-004: unchanged - no new invariant semantics.
I-005: unchanged - no new invariant semantics.
I-006: unchanged - conflict examples use existing lease/frontier checks.
I-007: unchanged - drain examples use existing active-overlap checks.
I-008: unchanged - lifecycle semantics are not changed.
I-009: unchanged - witness/authz recipes use existing scenarios.
I-010: unchanged - no new invariant semantics.
new invariant ids: none
```

## Affected formal models

```text
none - no Formal IR schema, generated model artifact, scenario, replay trace or
coverage schema changes.
```

## Affected protocols

```text
PR-docs-cookbook: derived developer documentation over existing commands and
fixtures. No dispatch, replay, authz, lease, graph, support-bundle or adapter
protocol behavior changes.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public CLI added/changed/removed: none.

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| cookbook replay explain examples | CLI smoke | existing scenarios emit and replay through documented commands | verified |
| cookbook graph/support-bundle examples | CLI smoke | graph export and support-bundle build consume documented scratch artifacts | verified |
| product-track status move | product-track gate | roadmap projections agree on M07.7/S07/S08 status | verified |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | unchanged | no | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | proof/all |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Formal model lanes are not regenerated because this is documentation over
existing executable surfaces. The applicable checks are CLI smoke commands for
the documented recipes plus product-track consistency validation.

## Acceptance commands

```bash
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m07.7-changed-files.txt
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
