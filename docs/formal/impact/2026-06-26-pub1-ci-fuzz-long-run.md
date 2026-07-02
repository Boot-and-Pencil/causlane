# Formal Impact Record: PUB1 CI fuzz long-run

## Change metadata

- Change ID: FIR-2026-06-26-pub1-ci-fuzz-long-run
- PR/issue: PUB1 verification/fuzz/property adoption follow-up
- Owner: repo maintainers
- Date: 2026-06-26
- Impact class: F1 (test/tooling evidence only)

## Touched protocol-critical paths

```text
verification/fuzz/
docs/release/refactor-before-publication-gate.md
```

## Summary

Records the required PUB1 long-running fuzz execution for the three protocol
targets that were added earlier in the PUB1 verification/fuzz/property adoption track.

The runs executed on host `dispatcher` with:

- `cargo-fuzz 0.13.2`;
- `cargo 1.93.0-nightly (5c0343317 2025-11-18)`;
- `rustc 1.93.0-nightly (53732d5e0 2025-11-20)`.

Each target completed with status 0 and produced no crash/reproducer artifact in
its `verification/fuzz/artifacts/<target>/` directory.

## Run results

| Target | Start UTC | End UTC | Status | Runs | Seconds | Peak RSS |
|---|---:|---:|---:|---:|---:|---:|
| `replay_trace_json` | 2026-06-26T02:10:27Z | 2026-06-26T02:25:28Z | 0 | 41,735,952 | 901 | 565 MB |
| `replay_scenario_yaml` | 2026-06-26T02:25:45Z | 2026-06-26T02:40:48Z | 0 | 7,678,201 | 901 | 337 MB |
| `registry_yaml_compile` | 2026-06-26T02:41:05Z | 2026-06-26T02:56:07Z | 0 | 8,963,580 | 901 | 383 MB |

## Affected invariants

No invariant semantics change. This record closes the execution-evidence part of
the PUB1 verification/fuzz/property adoption prerequisite for the existing parse-boundary and
numeric-boundary targets.

## Affected formal models

None. No generated model, receipt, coverage matrix, Formal IR, Alloy, P, Kani,
Verus or Lean artifact changes.

## Contract changes

- Bundle / replay trace / scenario / receipt schemas: none.
- Public API: none.
- Production dependencies: none.
- Dev/test dependencies: none.
- Curated fuzz corpus: unchanged because no reproducer was produced.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| 15-minute replay trace JSON fuzz run | cargo-fuzz | no crash/reproducer artifact | pass |
| 15-minute replay scenario YAML fuzz run | cargo-fuzz | no crash/reproducer artifact | pass |
| 15-minute registry YAML compile fuzz run | cargo-fuzz | no crash/reproducer artifact | pass |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. Local libFuzzer corpus growth from the
run remains ignored; only curated seed entries or reproducer regressions should
be committed.

## Acceptance commands

```bash
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 test --manifest-path verification/fuzz/Cargo.toml --no-run --bins
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 fuzz run replay_trace_json -- -max_total_time=900 -print_final_stats=1
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 fuzz run replay_scenario_yaml -- -max_total_time=900 -print_final_stats=1
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 fuzz run registry_yaml_compile -- -max_total_time=900 -print_final_stats=1
```

## Exception request

- Exception needed? no
- Follow-up issue: none for this PUB1 long-run gate. Future crashes should be
  committed as curated `regression_*` corpus seeds plus a review-matrix row.
