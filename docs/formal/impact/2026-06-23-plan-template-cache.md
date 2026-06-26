# Formal Impact Record: M09.5 plan/template cache

## Change metadata

- Change ID: FIR-2026-06-23-plan-template-cache
- PR/issue: M09.5 Plan/template caches
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (contract-layer plan identity boundary)

## Touched protocol-critical paths

```text
crates/causlane-contracts/src/plan_template_cache.rs
crates/causlane-contracts/src/lib.rs
docs/adr/0020-plan-template-cache.md
docs/06-runtime-and-performance.md
```

## Summary

M09.5 adds a pure in-memory plan/template cache in `causlane-contracts`.
`PlanTemplateCacheKey` binds canonical `PlanHashMaterial` to explicit
compile-affecting snapshot refs. Cache entries reuse existing plan-hash and
impact-set hashing helpers, so the cache cannot mint alternate identities or
duplicate template resolution.

## Affected invariants

```text
ADR-0009: unchanged - plan_hash remains a function of PlanHashMaterial.
ADR-0020: new - cache reuse is keyed by canonical material plus explicit
          compile snapshot refs.
I-001: unchanged - execution authority is not modified.
I-006: unchanged - conflict/merge semantics are not modified.
I-008: unchanged - lifecycle/replay semantics are not modified.
I-009: unchanged - approvals still bind to plan_hash + impact_set_hash.
new invariant ids: none
```

## Affected formal models

```text
none - no bundle, replay trace, Formal IR, generated model, scenario, receipt
or coverage-matrix schema changes.
```

## Contract changes

- Contract Rust API changed: `PlanTemplateCache`,
  `PlanTemplateCacheKey`, `PlanTemplateCacheKeyHash`,
  `PlanTemplateCacheEntry`, `PlanTemplateCacheLookup` and
  `PlanTemplateSnapshotRef` are added.
- Existing `PlanHashMaterial`, bundle, replay trace/scenario and Formal IR
  schemas are unchanged.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| repeated key lookup | contract unit | first lookup misses, second lookup hits | new |
| plan identity field changes | contract unit | cache key hash changes | new |
| snapshot hash changes | contract unit | cache key hash changes | new |
| snapshot order differs | contract unit | canonicalized refs produce same key | new |
| invalid snapshot ref | contract unit | empty id / non-canonical hash rejected | new |
| duplicate snapshot id with different hash | contract unit | key construction fails | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Rust | plan_template_cache unit tests | no | rust |
| Product docs | ADR-0020 + M09.5 track updates | no | docs |
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
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m09-5-plan-template-cache-changed-files.txt
./tools/formal-exceptions-check
./tools/schema-validate-all
./tools/product-track-status-check
just formal-coverage-matrix-check
just bench-m09-1-build
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
