# Formal Impact Record: M12.5 replay fuzz long-run

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-replay-fuzz-long-run
- PR/issue: M12.5 API validation loop
- Owner: repo maintainers
- Date: 2026-06-29
- Impact class: F1 (test/tooling evidence only)

## Touched protocol-critical paths

```text
docs/formal/impact/
docs/product-track/
```

## Summary

Records dispatcher long-running fuzz executions for the M12.5 replay/explain
parse-boundary fuzz targets:

- `replay_trace_json`;
- `replay_scenario_yaml`.

The runs executed on host `ci-dispatcher.lan` with:

- `cargo-fuzz 0.13.2`;
- `cargo 1.93.0-nightly (5c0343317 2025-11-18)`;
- `rustc 1.93.0-nightly (53732d5e0 2025-11-20)`.

Both targets completed with status 0 and produced no crash/reproducer artifact
in either their explicit `/tmp/causlane-fuzz-artifacts/` artifact directories or
the default `fuzz/artifacts/<target>/` directories.

## Run results

The runs used repository head `caf900629dfd88379111511700775dfdb081d59b`.

| Target | Start UTC | End UTC | Status | Runs | Seconds | Average exec/sec | New units | Peak RSS |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `replay_trace_json` | 2026-06-29T18:16:34Z | 2026-06-29T18:31:37Z | 0 | 63,502,840 | 901 | 70,480 | 84,363 | 678 MB |
| `replay_scenario_yaml` | 2026-06-29T18:31:37Z | 2026-06-29T18:46:39Z | 0 | 7,182,729 | 901 | 7,971 | 48,464 | 528 MB |

## Affected invariants

No invariant semantics change. This record adds execution evidence for the
existing fail-closed replay trace JSON and replay scenario YAML parse-boundary
fuzz targets.

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

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. Local libFuzzer corpus growth from the
runs remains ignored; only curated seed entries or reproducer regressions should
be committed.

This evidence is consumed by the M12.5 terminal classification for
`replay_scenario_explain`; it is not a standalone correctness proof.

## Acceptance commands

```bash
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 fuzz run replay_trace_json -- -max_total_time=900 -print_final_stats=1 -artifact_prefix=/tmp/causlane-fuzz-artifacts/replay_trace_json-20260629T181634Z/
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 fuzz run replay_scenario_yaml -- -max_total_time=900 -print_final_stats=1 -artifact_prefix=/tmp/causlane-fuzz-artifacts/replay_scenario_yaml-20260629T183137Z/
./tools/api-validation-loop-plan-check
tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: continue M12.5 classification for the remaining pending
  public facade/core surface before M12.6 freeze planning.
