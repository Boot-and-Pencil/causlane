# Formal Impact Record: M12.5 contracts performance scale

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-contracts-performance-scale
- PR/issue: M12.5 API validation loop
- Owner: repo maintainers
- Date: 2026-06-29
- Impact class: F2 (developer/operator performance diagnostics)

## Touched protocol-critical paths

```text
crates/causlane/benches/dispatch_baseline_bench_suite.rs
docs/product-track/bench-suite-matrix.json
docs/product-track/api-validation-loop-plan.json
docs/product-track/
```

## Summary

Records dispatcher Criterion scale measurements for the existing contracts
benchmarks selected by the M12.5 contracts API validation loop:

- `registry_normalize_from_yaml`;
- `plan_hash_release_promote`;
- `bundle_load_from_json`.

The benchmarks call existing public APIs only. They do not add a second
semantic authority and do not introduce a latency threshold.

## Run results

The benchmark build and measurements ran on `ci-dispatcher.lan` at
`bb9e138989184f5e2ba07d1b72eb36372318fac5` with:

- `cargo 1.96.0 (30a34c682 2026-05-25)`;
- `rustc 1.96.0 (ac68faa20 2026-05-25)`;
- Criterion 0.7.0.

Build gate:

| Command | Start UTC | End UTC | Status |
|---|---:|---:|---:|
| `./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked --no-run` | 2026-06-29T17:56:45Z | 2026-06-29T17:56:46Z | 0 |

Measurements:

| Benchmark | Workload | Start UTC | End UTC | Status | Mean | Median |
|---|---|---:|---:|---:|---:|---:|
| `registry_normalize_from_yaml` | parse and normalize release-promotion registry YAML | 2026-06-29T17:56:55Z | 2026-06-29T17:56:57Z | 0 | 45.750 us | 45.631 us |
| `plan_hash_release_promote` | compute plan hash for release-promotion plan material | 2026-06-29T17:56:55Z | 2026-06-29T17:56:57Z | 0 | 10.142 us | 10.129 us |
| `bundle_load_from_json` | load compiled release-promotion bundle JSON | 2026-06-29T17:56:55Z | 2026-06-29T17:56:57Z | 0 | 29.639 us | 29.498 us |

Criterion terminal summaries reported:

- `registry_normalize_from_yaml`: `time: [45.607 us 45.653 us 45.697 us]`;
- `plan_hash_release_promote`: `time: [10.123 us 10.151 us 10.180 us]`;
- `bundle_load_from_json`: `time: [29.444 us 29.510 us 29.597 us]`.

The mean and median values above are from Criterion `estimates.json`.

## Affected invariants

No invariant semantics change. Existing registry normalization, plan hashing and
compiled bundle loading semantics are measured but not modified.

## Affected formal models

None. No generated model, receipt, coverage matrix, Formal IR, Alloy, P, Kani,
Verus or Lean artifact changes.

## Contract changes

- Bundle / replay trace / scenario / receipt schemas: none.
- Public API: none.
- Production dependencies: none.
- Dev/test dependencies: none.
- Benchmark matrix: unchanged; this record measures existing benchmark IDs.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| contracts scale benchmark compile gate | cargo bench build | benchmark target compiles | pass |
| contracts scale benchmark measurement | Criterion | dispatcher result recorded before classification | pass |
| latency threshold enforcement | product track | explicitly deferred to host/release-profile policy | not applicable |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. This record adds performance diagnostic
coverage only; it is not a correctness proof.

## Acceptance commands

```bash
./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked --no-run
./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked -- "registry_normalize_from_yaml|plan_hash_release_promote|bundle_load_from_json"
ssh ci-dispatcher.lan 'cd /workspace/repo && ./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked -- "registry_normalize_from_yaml|plan_hash_release_promote|bundle_load_from_json"'
./tools/api-validation-loop-plan-check
tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: continue M12.5 classification for the remaining pending
  public facade/core and replay/explain surfaces before M12.6 freeze planning.
