# Formal Impact Record: M12.5 replay performance scale

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-replay-performance-scale
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

Records dispatcher Criterion scale measurements for the existing replay
benchmarks selected by the M12.5 replay/explain API validation loop:

- `replay_verify_with_bundle`;
- `replay_explain_human`.

The benchmarks call existing public APIs only. They do not add a second
semantic authority and do not introduce a latency threshold.

## Run results

The benchmark build and measurements ran on `ci-dispatcher.lan` at
`caf900629dfd88379111511700775dfdb081d59b` with:

- `cargo 1.96.0 (30a34c682 2026-05-25)`;
- `rustc 1.96.0 (ac68faa20 2026-05-25)`;
- Criterion 0.7.0.

Build gate:

| Command | Start UTC | End UTC | Status |
|---|---:|---:|---:|
| `./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked --no-run` | 2026-06-29T18:46:52Z | 2026-06-29T18:46:53Z | 0 |

Measurements:

| Benchmark | Workload | Start UTC | End UTC | Status | Mean | Median |
|---|---|---:|---:|---:|---:|---:|
| `replay_verify_with_bundle` | verify bundle-bound release-promotion replay trace | 2026-06-29T18:47:01Z | 2026-06-29T18:47:03Z | 0 | 14.992 us | 14.927 us |
| `replay_explain_human` | verify and render human replay explain diagnostics | 2026-06-29T18:47:01Z | 2026-06-29T18:47:03Z | 0 | 62.047 us | 61.875 us |

Criterion terminal summaries reported:

- `replay_verify_with_bundle`: `time: [14.840 us 14.885 us 14.934 us]`;
- `replay_explain_human`: `time: [61.862 us 62.048 us 62.238 us]`.

The mean and median values above are from Criterion `estimates.json`.

## Affected invariants

No invariant semantics change. Existing replay verification and explain
diagnostic rendering semantics are measured but not modified.

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
| replay scale benchmark compile gate | cargo bench build | benchmark target compiles | pass |
| replay scale benchmark measurement | Criterion | dispatcher result recorded before classification | pass |
| latency threshold enforcement | product track | explicitly deferred to host/release-profile policy | not applicable |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. This record adds performance diagnostic
coverage only; it is not a correctness proof.

## Acceptance commands

```bash
./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked --no-run
./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked -- "replay_verify_with_bundle|replay_explain_human"
ssh ci-dispatcher.lan 'cd /workspace/repo && ./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked -- "replay_verify_with_bundle|replay_explain_human"'
./tools/api-validation-loop-plan-check
tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: continue M12.5 classification for the remaining pending
  public facade/core surface before M12.6 freeze planning.
