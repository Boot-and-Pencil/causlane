# Formal Impact Record: PUB1 numeric-boundary fuzz/property slice

## Change metadata

- Change ID: FIR-2026-06-26-pub1-numeric-boundary-fuzz-property
- PR/issue: PUB1 fuzz/property adoption follow-up
- Owner: repo maintainers
- Date: 2026-06-26
- Impact class: F1 (test/tooling only)

## Touched protocol-critical paths

```text
fuzz/
crates/causlane-replay/tests/proptest_parse_boundaries.rs
docs/release/refactor-before-publication-gate.md
```

## Summary

Extends the existing PUB1 parse-boundary fuzz/property slice with numeric edge
coverage:

- replay trace JSON and replay scenario YAML property inputs now cover `u64`
  timestamp, lease amount, lease epoch, lease expiry and `u32` op-index
  boundaries;
- registry YAML property inputs now cover `u64` authz freshness and `u32`
  predicate/merge-protocol version boundaries;
- the cargo-fuzz corpus now has numeric-extreme seeds for replay trace JSON,
  replay scenario YAML and registry YAML compilation;
- the routine `ci-dispatcher` long-run budget is recorded as 15 minutes per
  protocol target.

The tests still delegate parsing, lowering and compilation to the existing
public replay and contract authorities. No replay, registry, lifecycle,
constraint or dispatch semantics are reimplemented.

## Affected invariants

No invariant semantics change. The slice strengthens parse-boundary totality and
determinism evidence for numeric DTO boundaries that already exist in replay and
contract inputs.

## Affected formal models

None. No generated model, receipt, coverage matrix, Formal IR, Alloy, P, Kani,
Verus or Lean artifact changes.

## Contract changes

- Bundle / replay trace / scenario / receipt schemas: none.
- Public API: none.
- Production dependencies: none.
- Dev/test dependencies: none.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| numeric-edge replay trace JSON | proptest/cargo-fuzz corpus | parser/lowering returns deterministically without panic | new |
| numeric-edge replay scenario YAML | proptest/cargo-fuzz corpus | parser/to-trace/lowering returns deterministically without panic | new |
| numeric-edge registry YAML | proptest/cargo-fuzz corpus | parser/compiler returns deterministically without panic | new |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. The 15-minute `ci-dispatcher` fuzz runs
are recorded as a PUB1/PUB5 prerequisite but are not claimed by this local test
slice until they are actually executed and any findings are committed.

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-replay --test proptest_parse_boundaries --locked
./tools/cargo-dev test -p causlane-replay --test mutation_fuzz --locked
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 test --manifest-path fuzz/Cargo.toml --no-run --bins
python3 tools/pre-publication-review-gate --json
python3 tools/architecture-lint --json
tools/product-track-status-check --json
git diff --check
```

## Exception request

- Exception needed? no
- Follow-up issue: run the three protocol fuzz targets on `ci-dispatcher` for
  15 minutes each before PUB5 and record any reproducer as corpus plus a
  review-matrix row.
