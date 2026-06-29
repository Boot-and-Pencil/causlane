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

Dispatcher measurement is pending in the follow-up evidence commit after this
benchmark target has been pushed and measured on `ci-dispatcher.lan`.

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
| runtime scale benchmark compile gate | cargo bench build | benchmark target compiles | new |
| runtime scale benchmark measurement | Criterion | dispatcher result recorded before classification | pending |
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
./tools/api-validation-loop-plan-check
./tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: run `runtime_guarded_audit_projection_flow` on
  `ci-dispatcher.lan`, record the Criterion result, then classify runtime API
  feedback with the remaining M12.5 evidence.
