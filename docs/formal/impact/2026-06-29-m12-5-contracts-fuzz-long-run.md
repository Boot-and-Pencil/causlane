# Formal Impact Record: M12.5 contracts fuzz long-run

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-contracts-fuzz-long-run
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

Records the dispatcher long-running fuzz execution for `registry_yaml_compile`,
the M12.5 property/fuzz seed for the contracts registry, compiled bundle and
plan-hash API surface.

The run executed on host `ci-dispatcher.lan` with:

- `cargo-fuzz 0.13.2`;
- `cargo 1.93.0-nightly (5c0343317 2025-11-18)`;
- `rustc 1.93.0-nightly (53732d5e0 2025-11-20)`.

The target completed with status 0 and produced no crash/reproducer artifact in
either the explicit
`/tmp/causlane-fuzz-artifacts/contracts-registry-20260629T174121Z/` artifact
directory or the default `verification/fuzz/artifacts/registry_yaml_compile/` directory.

## Run results

The run used repository head `bb9e138989184f5e2ba07d1b72eb36372318fac5`.

| Target | Start UTC | End UTC | Status | Runs | Seconds | Average exec/sec | New units | Peak RSS |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `registry_yaml_compile` | 2026-06-29T17:41:21Z | 2026-06-29T17:56:23Z | 0 | 7,749,966 | 901 | 8,601 | 45,069 | 538 MB |

## Affected invariants

No invariant semantics change. This record adds execution evidence for the
existing fail-closed registry YAML parse and compile fuzz target.

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
| 15-minute registry YAML compile fuzz run | cargo-fuzz | no crash/reproducer artifact | pass |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. Local libFuzzer corpus growth from the
run remains ignored; only curated seed entries or reproducer regressions should
be committed.

This evidence is consumed by the M12.5 terminal classification for
`contracts_registry_bundle_plan_hash`; it is not a standalone correctness proof.

## Acceptance commands

```bash
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 fuzz run registry_yaml_compile -- -max_total_time=900 -print_final_stats=1 -artifact_prefix=/tmp/causlane-fuzz-artifacts/contracts-registry-20260629T174121Z/
./tools/api-validation-loop-plan-check
tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: continue M12.5 classification for the remaining pending
  public facade/core and replay/explain surfaces before M12.6 freeze planning.
