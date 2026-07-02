# Formal Impact Record: M12.5 runtime fuzz long-run

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-runtime-fuzz-long-run
- PR/issue: M12.5 API validation loop
- Owner: repo maintainers
- Date: 2026-06-29
- Impact class: F1 (test/tooling evidence only)

## Touched protocol-critical paths

```text
docs/formal/impact/
docs/product-track/
docs/ci-dispatcher.md
```

## Summary

Records the first dispatcher long-running fuzz execution for
`runtime_guarded_audit_projection`, the M12.5 property/fuzz seed for the runtime
authz, audit and projection API surface.

The run executed on host `ci-dispatcher.lan` with:

- `cargo-fuzz 0.13.2`;
- `cargo 1.93.0-nightly (5c0343317 2025-11-18)`;
- `rustc 1.93.0-nightly (53732d5e0 2025-11-20)`.

The target completed with status 0 and produced no crash/reproducer artifact in
either the explicit `/tmp/causlane-fuzz-artifacts/runtime-guarded-20260629T131329Z/`
artifact directory or the default
`verification/fuzz/artifacts/runtime_guarded_audit_projection/` directory.

## Run results

| Target | Start UTC | End UTC | Status | Runs | Seconds | Peak RSS |
|---|---:|---:|---:|---:|---:|---:|
| `runtime_guarded_audit_projection` | 2026-06-29T13:13:29Z | 2026-06-29T13:28:31Z | 0 | 10,502,706 | 901 | 429 MB |

## Affected invariants

No invariant semantics change. This record adds execution evidence for the
existing fail-closed guarded execution, audit append, trace projection and
projection redaction fuzz target.

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
| 15-minute runtime guarded audit/projection fuzz run | cargo-fuzz | no crash/reproducer artifact | pass |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. Local libFuzzer corpus growth from the
run remains ignored; only curated seed entries or reproducer regressions should
be committed.

This evidence does not complete the M12.5 terminal classification for
`runtime_dispatch_audit_projection`; API feedback and performance-scale findings
remain pending before M12.6 freeze planning can consume this surface.

## Acceptance commands

```bash
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 fuzz run runtime_guarded_audit_projection -- -max_total_time=900 -print_final_stats=1 -artifact_prefix=/tmp/causlane-fuzz-artifacts/runtime-guarded-20260629T131329Z/
./tools/api-validation-loop-plan-check
./tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: classify runtime authz/audit/projection API feedback with
  the remaining M12.5 synthetic example and performance-scale evidence before
  terminal classification.
