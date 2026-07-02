# Formal Impact Record: M12.5 facade/kernel API feedback classification

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-facade-api-feedback-classification
- PR/issue: M12.5 API validation loop
- Owner: repo maintainers
- Date: 2026-06-29
- Impact class: F2 (developer/operator API validation evidence)

## Touched protocol-critical paths

```text
docs/product-track/api-validation-loop-plan.json
docs/product-track/
examples/facade-kernel-operator-workflow/
verification/fuzz/fuzz_targets/facade_kernel_frontier.rs
tools/examples-check
```

## Summary

Classifies the M12.5 `public_facade_and_core_kernel` surface as
`accepted_for_freeze` for the current evidence slice. The classification
consumes:

- synthetic examples:
  `facade-kernel-ergonomics` and `facade-kernel-operator-workflow`;
- property/fuzz evidence:
  `facade_kernel_frontier`, existing core proptest checks and facade operator
  workflow negative controls;
- performance-scale evidence:
  `frontier_conflict_selection` and `lease_grant_exclusive` from the dispatcher
  Criterion run.

The new `facade-kernel-operator-workflow` example broadens synthetic coverage
from facade admission/frontier ergonomics to a near-real operator workflow over
admission, barrier/truth policy, frontier selection, constraint decisions,
lease validation and fail-closed controls. It depends only on `causlane`.

## API feedback classification

| Lane | Evidence | API feedback | Classification |
|---|---|---|---|
| Synthetic examples | facade-only admission, policy, frontier, constraint and lease workflows | Current `causlane::prelude` plus `causlane::core::{kernel,protocol}` are sufficient without private helper APIs. | `accepted_for_freeze` |
| Property/fuzz | core proptests, facade operator negative controls and 15-minute dispatcher fuzz long-run with no crash or reproducer | No API shape change required from current findings. | `accepted_for_freeze` |
| Performance scale | frontier selection and exclusive lease grant measured on dispatcher Criterion | No latency threshold or API shape change is introduced by this run. | `accepted_for_freeze` |

## Affected invariants

No invariant semantics change. The examples and fuzz target exercise existing
admission, barrier policy, observed-truth policy, frontier, constraint and lease
semantics without replacing their authorities.

## Affected formal models

None. No generated model, receipt, coverage matrix, Formal IR, Alloy, P, Kani,
Verus or Lean artifact changes.

## Contract changes

- Bundle / replay trace / scenario / receipt schemas: none.
- Public Rust API: none.
- Manifest versions: none.
- Production dependencies: none.
- Generated publication readiness reports: none.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| duplicate lease id | facade operator example tests | duplicate rejected | pass |
| conflicting exclusive lease | facade operator example tests | conflict rejected by `KernelContracts` authority | pass |
| expired barrier lease | facade operator example tests | barrier validation rejects expired lease | pass |
| token claim exceeds budget | facade operator example tests | constraint decision is `Deny` | pass |
| full token budget | facade operator example tests | constraint decision is `Wait` | pass |
| facade/kernel fuzz long-run | dispatcher fuzz | no crash/reproducer artifacts | pass |
| facade/kernel performance run | dispatcher Criterion | evidence recorded, no threshold claim | pass |

## Required proof/model changes

None.

## Not applicable lanes

All M12.5 selected surfaces now have terminal classifications. M12.6 freeze
planning is the next consumer of this result and must make any stabilization
claims from the classified evidence rather than prose-only assertions.

## Acceptance commands

```bash
python3 tools/examples-check
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 test --manifest-path verification/fuzz/Cargo.toml --no-run --bins --locked
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 fuzz run facade_kernel_frontier -- -max_total_time=900 -print_final_stats=1 -artifact_prefix=/tmp/causlane-fuzz-artifacts/facade-kernel-20260629T191504Z/
./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked -- "frontier_conflict_selection|lease_grant_exclusive"
./tools/api-validation-loop-plan-check
tools/product-track-bundle --check
tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: move to M12.6 semver pre-1.0 freeze planning with the full
  M12.5 classified evidence set.
