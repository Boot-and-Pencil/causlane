# Formal Impact Record: M12.5 facade/kernel fuzz long-run

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-facade-fuzz-long-run
- PR/issue: M12.5 API validation loop
- Owner: repo maintainers
- Date: 2026-06-29
- Impact class: F1 (test/tooling evidence only)

## Touched protocol-critical paths

```text
verification/fuzz/fuzz_targets/facade_kernel_frontier.rs
verification/fuzz/Cargo.toml
docs/product-track/
docs/ci-dispatcher.md
```

## Summary

Records the dispatcher long-running fuzz execution for
`facade_kernel_frontier`, the M12.5 property/fuzz target for the public facade
and curated core/kernel API surface.

The run executed on host `ci-dispatcher.lan` at
`433cda118d0cdd21758a007e4a72f198ec5208e9` with:

- `cargo-fuzz 0.13.2`;
- `cargo 1.93.0-nightly (5c0343317 2025-11-18)`;
- `rustc 1.93.0-nightly (53732d5e0 2025-11-20)`;
- stable `cargo 1.96.0 (30a34c682 2026-05-25)`;
- stable `rustc 1.96.0 (ac68faa20 2026-05-25)`.

The target completed with status 0 and produced no crash/reproducer artifact in
either the explicit
`/tmp/causlane-fuzz-artifacts/facade-kernel-20260629T191504Z/` artifact
directory or the default `verification/fuzz/artifacts/facade_kernel_frontier/` directory.

## Run results

| Target | Start UTC | End UTC | Status | Runs | Seconds | Avg exec/s | New units | Peak RSS |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `facade_kernel_frontier` | 2026-06-29T19:15:04Z | 2026-06-29T19:30:08Z | 0 | 5,105,061 | 901 | 5,665 | 328 | 496 MB |

## Affected invariants

No invariant semantics change. This record adds execution evidence for the
existing public facade and curated kernel APIs:

- action admission preserves action identity;
- runtime execution remains the only profile in this surface that requires an
  execution barrier and can commit observed truth;
- `KernelContracts.resolve` delegates to `kernel::resolve_constraints`;
- selected frontier outputs remain structurally ready, conflict-free by write
  scope and within bounded lane capacity.

## Affected formal models

None. No generated model, receipt, coverage matrix, Formal IR, Alloy, P, Kani,
Verus or Lean artifact changes.

## Contract changes

- Bundle / replay trace / scenario / receipt schemas: none.
- Public Rust API: none.
- Manifest versions: none.
- Production dependencies: none.
- Curated fuzz corpus: unchanged because no reproducer was produced.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| 15-minute facade/kernel fuzz run | cargo-fuzz | no crash/reproducer artifact | pass |
| facade admission identity | fuzz invariant | accepted action id matches submitted action id | pass |
| constraint delegation | fuzz invariant | public contract delegation equals kernel resolver | pass |
| frontier output consistency | fuzz invariant | selected set is ready, conflict-free and within lane capacity | pass |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. Local libFuzzer corpus growth from the
run remains ignored; only curated seed entries or reproducer regressions should
be committed.

## Acceptance commands

```bash
tools/ci-dispatcher-preflight
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 fuzz run facade_kernel_frontier -- -max_total_time=900 -print_final_stats=1 -artifact_prefix=/tmp/causlane-fuzz-artifacts/facade-kernel-20260629T191504Z/
./tools/api-validation-loop-plan-check
tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: consume this evidence in the terminal
  `public_facade_and_core_kernel` API feedback classification before M12.6
  freeze planning.
