# Formal Impact Record: PUB1 core property coverage

## Change metadata

- Change ID: FIR-2026-06-26-pub1-core-property-coverage
- PR/issue: PUB1 verification/fuzz/property adoption follow-up
- Owner: repo maintainers
- Date: 2026-06-26
- Impact class: F1 (test/tooling only)

## Touched protocol-critical paths

```text
crates/causlane-core/tests/proptest_protocol_properties.rs
docs/release/refactor-before-publication-gate.md
```

## Summary

Adds dev-only property coverage for the core semantic contract surface:

- lifecycle samples compare `KernelContracts` with the existing
  `reduce_lifecycle` authority;
- constraint samples compare `KernelContracts` with `resolve_constraints` and
  assert documented token-budget outcomes for oversized, over-budget and
  in-budget same-batch token claims;
- `causlane-core` gains a `proptest` dev-dependency only.

No lifecycle transition table or constraint resolver is duplicated in the test
suite. The properties call the existing public contract surface and pure domain
authorities.

## Affected invariants

No invariant semantics change. The slice strengthens test evidence for lifecycle
determinism/delegation and token-budget fail-closed behavior already enforced by
the core kernel.

## Affected formal models

None. No generated model, receipt, coverage matrix, Formal IR, Alloy, P, Kani,
Verus or Lean artifact changes.

## Contract changes

- Bundle / replay trace / scenario / receipt schemas: none.
- Public API: none.
- Production dependencies: none.
- Dev/test dependencies: `proptest` for `causlane-core` tests.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| sampled lifecycle contract delegation | proptest | `KernelContracts` agrees with `reduce_lifecycle` | new |
| oversized token-budget claim | proptest | decision is `Deny` | new |
| same-batch token total exceeds budget | proptest | decision is `Wait` | new |
| same-batch token total within budget | proptest | decision is `Allow` with the submitted claims | new |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. The `ci-dispatcher` long-run fuzz
prerequisite remains tracked separately before PUB5.

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-core --test proptest_protocol_properties --locked
./tools/cargo-dev test -p causlane-core --locked
./tools/cargo-dev test -p causlane-replay --test proptest_parse_boundaries --locked
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
