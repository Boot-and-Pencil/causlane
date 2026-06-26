# Historical / partially completed ТЗ: integration of the formal lifecycle patch-pack

> **Repository integration status:** historical / partially completed integration
> TЗ for the 009→010 line. This file is no longer a fresh active backlog by
> itself. Use the status table below to distinguish `done`, `partial` and `open`.
> Current proof evidence remains the generated chain from compiled bundle and
> scenario through Formal IR, generated artifacts, receipts, stale-check and
> derived coverage.

## 1. Purpose of the original TЗ

The original goal was to integrate a full formal lifecycle discipline into
Causlane: a protocol-critical feature or fix should not be able to change
runtime behavior without a named formal obligation, scenario/negative control,
generated model artifact, tool-run receipt, stale-check and derived coverage.

That goal is still valid, but the document is now a status ledger rather than an
unqualified future-looking plan.

## 2. Baseline 010 summary

The current line already contains:

```text
Formal IR v2
Alloy/P/Kani/Verus/Lean4 generation surface
formal-verify-all gate
receipts v2
stale-check-all
derived coverage report
coverage matrix drift check
formal exceptions policy
full doctor/bootstrap scripts
Lean4 repo-local Lake package
Verus proof catalog
formal model/protocol catalogs
```

The main open governance gap is no longer the executable itself:
`tools/formal-discipline-check` exists and can run local/PR-diff discipline
checks, including strict check-id adequacy across obligations, generated
artifacts, receipts, coverage and docs. It is now wired into the mandatory
`tools/formal-verify-all` repo gate after fresh coverage derivation and
coverage-matrix plus proof/refinement-scope drift checking. Provider-specific
PR CI wiring is outside this repository because no workflow config is present
here.

## 3. Status table

| ID | Original item | Status in repo 010 | Notes / next action |
|---|---|---|---|
| INT-FM-001 | accept lifecycle docs and ADR | done | ADR-0015 and `docs/formal/00..07` exist. |
| INT-FM-002 | obligation manifest + schema | done | `formal/obligations/lifecycle_product_obligations.yaml` and schema exist. |
| INT-FM-003 | implement `tools/formal-discipline-check` | done | Executable wrapper and `causlane-formal-discipline` binary exist with focused tests. |
| INT-FM-004 | integrate discipline check into gates | done | `tools/formal-verify-all` runs discipline after derived coverage, coverage-matrix checking and proof/refinement-scope checking. |
| INT-FM-005 | Lean4 toolchain profile | done | `elan`/`lean`/`lake` are in proof/all profile; install/doctor scripts know Lean4. |
| INT-FM-006 | Lean4 codegen target | done | `formal generate lean4` and `FormalTarget::Lean4` exist. |
| INT-FM-007 | Lean4 generic core + generated theorem applications | done/partial | Core package and generated scenario-bound theorem applications exist; current covered cells and theorem `check_id`s are the generated coverage matrix/report. |
| INT-FM-008 | Verus proof catalog and no-cheating proof profile | done/partial | Verus proof catalog exists; proof/all profile is authoritative. Keep checking no-cheating receipts. |
| INT-FM-009 | model adequacy checks | done | `formal-discipline-check` cross-checks every manifest-required `check_id` across obligation → artifact/negative control → receipt → coverage → docs. |
| INT-FM-010 | feature/fix PR workflow | partial/open | Policy docs/templates and executable PR-diff checker exist; provider-specific CI adoption remains external to this repo. |

## 4. Current enforced gates

Currently enforceable from repo 010:

```bash
just rust-full-check
just formal-ready
just formal-verify-all
tools/formal-verify-all --profile proof
tools/formal-exceptions-check --profile rust
tools/coverage-matrix --check
tools/formal-discipline-check --profile rust --no-diff --json
tools/full-doctor --json --profile proof
```

Available for provider-specific PR CI when a workflow supplies the baseline:

```bash
tools/formal-discipline-check --profile rust --from-git origin/main...HEAD
```

The repository has no checked-in provider workflow. External CI must not use
`--no-diff` for PR enforcement; it must supply a changed-file source with
`--from-git` or `--changed-files`.

## 5. Remaining active work packages

### DONE-FM-001 — implement `tools/formal-discipline-check`

Implemented as `tools/formal-discipline-check` plus the
`causlane-formal-discipline` CLI binary.

Minimum acceptance:

```bash
test -x tools/formal-discipline-check
tools/formal-discipline-check --profile rust --no-diff --manifest formal/obligations/lifecycle_product_obligations.yaml --json
```

Synthetic negative acceptance:

- changed file `crates/causlane-replay/src/lib.rs` with no Formal Impact Record fails;
- expired formal exception fails;
- docs coverage overclaim fails;
- Lean4 `sorry` or unapproved `axiom` in authoritative paths fails;
- Verus cheating constructs fail under proof/all.

### DONE-FM-002 — wire discipline check into the mandatory repo gate

`tools/formal-verify-all` runs:

```bash
tools/formal-discipline-check --profile "$PROFILE" --no-diff --json
```

after receipt-derived coverage, `tools/coverage-matrix --check` and
`tools/proof-refinement-scope --check`.

Provider-specific PR CI should run:

```bash
tools/formal-discipline-check --profile rust --from-git "$BASE_REF...HEAD"
```

PR enforcement must not use `--no-diff`.

### DONE-FM-003 — check-id adequacy across all lanes

`tools/formal-discipline-check` checks that every manifest-required `check_id`
appears in all required places:

```text
obligation manifest
generated artifact body/header
codegen receipt
tool-run receipt
coverage report
docs projection
```

The check is strict: counted checks that are not manifest-required also fail.

### DONE-FM-004 — document current Lean4 scope everywhere

All status documents must distinguish generated coverage-matrix evidence from
target-state catalog obligations. Current Lean4 coverage must be read from the
generated matrix/report, not restated as a hand-maintained invariant list.

### OPEN-FM-005 — release governance adoption

Integrate Formal Impact Record, proof exception and release formal signoff into
PR/release process after the docs-only pack is accepted.

## 6. Non-goals

- Do not turn Causlane into workflow engine, job queue, scheduler or production PDP.
- Do not treat generic hand-written Lean/Alloy/P/Verus sketches as authority.
- Do not count `non_blocking_skipped` proof facets as coverage.
- Do not claim provider-specific PR CI enforcement unless a workflow supplies
  the changed-file baseline and runs `formal-discipline-check`.

## 7. Definition of done for retiring this historical TЗ

This file can be retired or replaced by a current `formal-governance-status.md`
when:

1. `tools/formal-discipline-check` exists and has tests.
2. The command is wired into `tools/formal-verify-all` or an equivalent gate.
3. All docs describing it distinguish mandatory repo-gate integration from
   provider-specific PR CI adoption.
4. The status table above is reflected in a machine-readable governance report.
