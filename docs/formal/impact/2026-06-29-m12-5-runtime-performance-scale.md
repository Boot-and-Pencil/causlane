# Formal Impact Record: M12.5 runtime performance scale

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-runtime-performance-scale
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

Adds `runtime_guarded_audit_projection_flow` to the existing Criterion dispatch
baseline suite. The benchmark exercises the runtime API surface selected for
M12.5 validation by measuring a 512-op positive path through:

- `GuardedExecutor` authorization and capability spend;
- `TraceProjectingAuditLog<InMemoryAuditLog, InMemoryTraceSink>` append plus
  span projection;
- guarded projection read and redaction via `guard_projection_read`.

The benchmark calls existing public APIs only. It does not add a second semantic
authority and does not introduce a latency threshold.

## Run results

The benchmark was measured on `ci-dispatcher.lan` at
`1b62a8c09f65ca3fc35f132cbfaac32f5fe75543` with:

- `cargo 1.96.0 (30a34c682 2026-05-25)`;
- `rustc 1.96.0 (ac68faa20 2026-05-25)`;
- Criterion 0.7.0.

| Benchmark | Workload | Start UTC | End UTC | Status | Mean | Median |
|---|---:|---:|---:|---:|---:|---:|
| `runtime_guarded_audit_projection_flow` | 512 guarded ops, 1,028 audit events, 1 projection read | 2026-06-29T13:58:13Z | 2026-06-29T13:58:34Z | 0 | 1.4399 ms | 1.4446 ms |

Criterion reported the mean 95% confidence interval as 1.4318 ms to 1.4487 ms
and the median 95% confidence interval as 1.4265 ms to 1.4466 ms. The terminal
summary printed `time: [1.4251 ms 1.4319 ms 1.4397 ms]` and one high-mild
outlier among 20 measurements.

## Affected invariants

No invariant semantics change. Existing authorization, capability, audit append,
trace projection and projection redaction semantics are measured but not
modified.

## Affected formal models

None. No generated model, receipt, coverage matrix, Formal IR, Alloy, P, Kani,
Verus or Lean artifact changes.

## Contract changes

- Bundle / replay trace / scenario / receipt schemas: none.
- Public API: none.
- Production dependencies: none.
- Dev/test dependencies: none.
- Benchmark matrix: adds `runtime_guarded_audit_projection_flow`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| runtime scale benchmark compile gate | cargo bench build | benchmark target compiles | pass |
| runtime scale benchmark measurement | Criterion | dispatcher result recorded before classification | pass |
| latency threshold enforcement | product track | explicitly deferred to host/release-profile policy | not applicable |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. This record adds performance diagnostic
coverage only; it is not a correctness proof and does not complete the M12.5
terminal classification.

## Acceptance commands

```bash
./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked --no-run
./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked -- runtime_guarded_audit_projection_flow
ssh ci-dispatcher.lan 'cd /workspace/repo && ./tools/cargo-dev bench -p causlane --bench dispatch_baseline_bench_suite --locked -- runtime_guarded_audit_projection_flow'
./tools/api-validation-loop-plan-check
./tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: classify runtime API feedback with the M12.5 synthetic
  example, fuzz and performance-scale evidence.
