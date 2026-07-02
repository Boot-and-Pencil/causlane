# Formal Impact Record: Verus + Lean4 proof lanes are always-on and blocking

## Change metadata

- Change ID: FIR-2026-06-21-proof-lane-always-on-blocking
- PR/issue: proof-lane blocking before exception expiry (branch `verification/formal-full/proof-lane-blocking`)
- Owner: formal-core
- Date: 2026-06-21
- Impact class: F2 (formal-gate policy / discipline) — proof lanes promoted from
  time-boxed non-blocking to always-on blocking; no kernel/invariant semantics change

## Touched protocol-critical paths

```text
scripts/check-verification-full.sh                                   (RUN_PROOF=1 always; drop skip branches)
docs/formal-exceptions.json                               (exceptions -> [])
docs/formal-exceptions.md
verification/formal-full/obligations/lifecycle_product_obligations.yaml     (verus/lean4 proof_profile_required -> required)
crates/causlane-codegen/src/coverage.rs                   (verus/lean4 cell default Passed)
crates/causlane-codegen/src/obligations.rs                (comment)
tools/coverage-matrix                                     (doc text)
docs/invariants/coverage-matrix.{json,md}                 (regenerated -> passed)
docs/formal-readiness-status.md, verification/formal-full/README.md, verification/formal-full/verus/README.md,
verification/formal-full/lean/README.md, docs/formal/03-lean4-verus-proof-obligations.md,
docs/11-contract-hardening-plan.md                        (status reconciliation)
```

## Summary

The Verus and Lean4 proof lanes were real, non-vacuous artifacts but recorded
`non_blocking_skipped` under the default profile, kept "non-blocking" by the
time-boxed `LANE_REALITY_{VERUS,LEAN4}_NON_BLOCKING` exceptions expiring
**2026-09-01**. After expiry, `formal-exceptions-check --profile rust` would have
failed and broken the default gate; meanwhile the proof profile was never run by
default, so the proofs could silently rot.

This change makes both lanes **always run and blocking**:

1. **Gate (`scripts/check-verification-full.sh`):** `RUN_PROOF=1` unconditionally and the
   doctor is always `--profile all`, so `verus --no-cheating` and
   `lake build CauslaneFormal` + `lake env lean` run on **every** invocation. The
   `non_blocking_skipped` else-branches (and their `formal-exceptions-check
   --skipped-target` calls) were removed. A non-zero proof exit makes the artifact
   status `Fail`, which `overall_status` turns into a `Fail` report — i.e. blocking.
   The fast dev loop (`formal-ready`, `cargo test`, `clippy`) does not run proofs.
2. **Exceptions dropped:** `docs/formal-exceptions.json` `exceptions` is now `[]`
   (`formal-exceptions-check` passes with no expired entries); prose companion
   updated. The expiry risk is gone.
3. **Coverage honesty:** `coverage.rs` now declares Verus/Lean4 invariant cells as
   `Passed` (was `NonBlockingSpec`) where a proof obligation exists; the reconcile
   pass still downgrades any cell whose tool run did not hold, so a proof failure is
   never absorbed. Obligations flipped `proof_profile_required -> required`. The
   committed `coverage-matrix.{json,md}` were regenerated (`--write`) — no
   `non_blocking_spec` cells remain.

## Non-vacuity proof (anti-theatre)

- **Baseline confirmed by running, not assuming:** `scripts/check-verification-full.sh`
  (always-on) records `verus: actual=pass exit=0` and `lean4: actual=pass exit=0`
  in the tool-run receipts. `verus --no-cheating` accepts the generated proof; the
  Lean4 artifacts carry no `sorry`/`admit`/`axiom`.
- **Blocking is real:** `overall_status` requires every artifact to `holds()`
  (`coverage.rs`). With the lanes always run, a Verus/Lean4 `Fail` → `!holds()` →
  report `Fail` → gate fails. The matrix reconcile additionally downgrades a
  declared-`Passed` proof cell to `PendingToolRun` if its receipt did not hold.
- **No coverage overclaim:** `tools/coverage-matrix --check` passes (docs match the
  freshly derived report); `tools/formal-discipline-check --profile all` passes;
  `tools/formal-exceptions-check` passes with the empty policy.

## Affected invariants

```text
No invariant semantics change. Verus now blocking-covers I-001/2/3/4/5/6/7/8/9/10;
Lean4 blocking-covers I-001/2/3/4/5/6/7/8/9. The remaining planned Lean4 cell
(I-010) is covered by other lanes.
new invariant ids: none
```

## Affected formal models

```text
Verus + Lean4: promoted from non_blocking_skipped (default) to always-run blocking.
No generator/proof content change (artifacts already passed). Alloy/P/Kani/replay
unchanged.
```

## Contract changes

- Bundle / Formal IR / replay-trace / receipt schema: none.
- Core semantic change: none.

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| (existing corpus) | verus/lean4 | proofs pass; a proof failure fails the gate | verified pass; blocking via overall_status |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| verus | release_promote_success preservation + rule lemmas | generated | always-on (every run) |
| lean4 | I-001/2/3/4/5/8/9 native_decide theorems | generated | always-on (every run) |

## Not applicable lanes

No Alloy/P/Kani change.

## Acceptance commands

```bash
just verification-full
tools/coverage-matrix --check
tools/formal-exceptions-check --profile all
tools/formal-discipline-check --profile all --no-diff
```

## Exception request

- Exception needed? no — this change REMOVES the two standing exceptions.
- Follow-up: none for the current proof coverage. Verus I-005/I-007 and Lean4
  I-004/I-005 were later promoted to required proof/all coverage on
  2026-06-24; Lean4 lease/constraint cells remain future proof-depth work.
