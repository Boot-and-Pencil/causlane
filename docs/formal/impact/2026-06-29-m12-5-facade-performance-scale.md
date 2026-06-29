# Formal Impact Record: M12.5 facade/kernel performance scale

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-facade-performance-scale
- PR/issue: M12.5 API validation loop
- Owner: repo maintainers
- Date: 2026-06-29
- Impact class: F2 (developer/operator performance diagnostics)

## Touched protocol-critical paths

```text
docs/product-track/api-validation-loop-plan.json
docs/product-track/bench-suite-matrix.json
docs/product-track/
```

## Summary

Records the dispatcher Criterion scale run for the
`public_facade_and_core_kernel` surface. The run consumes the existing dispatch
baseline benchmarks selected for this surface:

- `frontier_conflict_selection`, measuring `select_frontier`;
- `lease_grant_exclusive`, measuring `LeaseTable::grant`.

Both benchmarks call existing public APIs only. This record introduces no
latency threshold and no new benchmark ID.

## Run results

The benchmark was measured on `ci-dispatcher.lan` at
`433cda118d0cdd21758a007e4a72f198ec5208e9` with:

- `cargo 1.96.0 (30a34c682 2026-05-25)`;
- `rustc 1.96.0 (ac68faa20 2026-05-25)`;
- Criterion 0.7.0.

Build gate:

| Command | Start UTC | End UTC | Status |
|---|---:|---:|---:|
| `./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked --no-run` | 2026-06-29T19:30:23Z | 2026-06-29T19:30:23Z | 0 |

Measurement run:

| Benchmark | Fixture | Start UTC | End UTC | Status | Mean | Median |
|---|---|---:|---:|---:|---:|---:|
| `frontier_conflict_selection` | ready op graph with lane capacity and write-scope conflict | 2026-06-29T19:30:23Z | 2026-06-29T19:30:25Z | 0 | 1.4565 us | 1.4685 us |
| `lease_grant_exclusive` | exclusive release-promote lease on empty lease table | 2026-06-29T19:30:23Z | 2026-06-29T19:30:25Z | 0 | 139.49 ns | 139.55 ns |

The terminal Criterion summaries printed:

- `frontier_conflict_selection`: `time: [1.4699 us 1.4943 us 1.5097 us]`;
- `lease_grant_exclusive`: `time: [140.08 ns 141.82 ns 143.01 ns]`.

The JSON estimates were:

- `frontier_conflict_selection`: mean 1456.488 ns, median 1468.453 ns;
- `lease_grant_exclusive`: mean 139.488 ns, median 139.547 ns.

## Affected invariants

No invariant semantics change. Existing frontier selection and lease-table
conflict semantics are measured but not modified.

## Affected formal models

None. No generated model, receipt, coverage matrix, Formal IR, Alloy, P, Kani,
Verus or Lean artifact changes.

## Contract changes

- Bundle / replay trace / scenario / receipt schemas: none.
- Public Rust API: none.
- Manifest versions: none.
- Production dependencies: none.
- Benchmark matrix: unchanged; this run records existing benchmark evidence.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| facade/kernel benchmark compile gate | cargo bench build | benchmark target compiles | pass |
| facade/kernel benchmark measurement | Criterion | dispatcher result recorded before classification | pass |
| latency threshold enforcement | product track | explicitly deferred to host/release-profile policy | not applicable |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. This record adds performance diagnostic
coverage only; it is not a correctness proof.

## Acceptance commands

```bash
./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked --no-run
./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked -- "frontier_conflict_selection|lease_grant_exclusive"
ssh ci-dispatcher.lan 'cd /workspace/repo && ./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked -- "frontier_conflict_selection|lease_grant_exclusive"'
./tools/api-validation-loop-plan-check
tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: consume this evidence in the terminal
  `public_facade_and_core_kernel` API feedback classification before M12.6
  freeze planning.
