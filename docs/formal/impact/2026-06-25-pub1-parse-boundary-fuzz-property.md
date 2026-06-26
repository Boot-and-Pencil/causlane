# Formal Impact Record: PUB1 parse-boundary fuzz/property slice

## Change metadata

- Change ID: FIR-2026-06-25-pub1-parse-boundary-fuzz-property
- PR/issue: PUB1 fuzz/property adoption
- Owner: repo maintainers
- Date: 2026-06-25
- Impact class: F1 (test/tooling only)

## Touched protocol-critical paths

```text
fuzz/
crates/causlane-replay/tests/proptest_parse_boundaries.rs
docs/release/refactor-before-publication-gate.md
```

## Summary

Adds the first real PUB1 fuzz/property slice after the smoke scaffold:

- cargo-fuzz targets for replay trace JSON, replay scenario YAML and registry
  YAML compilation;
- proptest coverage for deterministic parse/lowering outcomes over generated
  text;
- small seed corpus entries for the new targets.

The targets call the existing public parsers, lowerers and bundle compiler. They
do not re-implement replay, registry validation, lifecycle, constraint or
dispatch semantics.

## Affected invariants

No invariant semantics change. The slice strengthens parse-boundary totality and
determinism evidence around the existing replay/contract authorities.

## Affected formal models

None. No generated model, receipt, coverage matrix, Formal IR, Alloy, P, Kani,
Verus or Lean artifact changes.

## Contract changes

- Bundle / replay trace / scenario / receipt schemas: none.
- Public API: none.
- Production dependencies: none.
- Dev/test-only dependency: `causlane-replay` gains `proptest` as a
  `dev-dependency`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| arbitrary replay trace JSON | cargo-fuzz/proptest | parser/lowering returns deterministically without panic | new |
| arbitrary replay scenario YAML | cargo-fuzz/proptest | parser/to-trace/lowering returns deterministically without panic | new |
| arbitrary registry YAML | cargo-fuzz/proptest | parser/compiler returns deterministically without panic | new |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. Long-running fuzz execution and numeric
extreme corpus growth remain PUB1/PUB5 follow-up work on the formal-capable CI
host.

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-replay --test proptest_parse_boundaries --locked
./tools/cargo-dev test -p causlane-replay --test mutation_fuzz --locked
cargo +nightly-2025-11-21 test --manifest-path fuzz/Cargo.toml --no-run --bins
python3 tools/pre-publication-review-gate --json
just refactor-readiness
tools/product-track-status-check --json
```

## Exception request

- Exception needed? no
- Follow-up issue: define the routine `ci-dispatcher` fuzz time budget, add
  numeric-extreme targets/corpus, and record any findings as review-matrix rows.
