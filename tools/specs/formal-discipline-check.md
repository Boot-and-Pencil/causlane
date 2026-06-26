# Specification: tools/formal-discipline-check

## Implementation status

This command is **implemented in repo 010** as `tools/formal-discipline-check`,
backed by the `causlane-formal-discipline` CLI binary. It is usable as a local
or PR-diff discipline check, and is wired into the mandatory repo strict gate:
`tools/formal-verify-all` runs it after receipt-derived coverage and
`tools/coverage-matrix --check` plus `tools/proof-refinement-scope --check`.

Provider-specific CI may run the same command with `--from-git` or
`--changed-files` for PR-diff enforcement. This repository does not define a
provider workflow; the mandatory in-repo integration point is
`tools/formal-verify-all`.

## Purpose

`formal-discipline-check` is the machine gate that turns the anti-theatre policy into enforcement.

## CLI

```bash
tools/formal-discipline-check \
  --profile rust|base|ci|proof|all \
  [--changed-files path.txt | --from-git base...head | --no-diff] \
  [--manifest formal/obligations/lifecycle_product_obligations.yaml] \
  [--json]
```

`--no-diff` is allowed for local manifest-only checks but forbidden in CI. A
local `--no-diff` run may report stale cached coverage as `warn`; `--changed-files`
and `--from-git` treat the same coverage drift as `fail`.

## Inputs

```text
changed file list
formal obligation manifests
coverage report, when present
docs/invariants/coverage-matrix.json
formal-exceptions.json
generated artifacts and receipts, when present
```

## Required checks

1. Validate obligation manifest shape and safety fields.
2. Detect protocol-critical path changes.
3. Require Formal Impact Record for protocol-critical F2+ changes.
4. Check that every required lane has nonempty `check_ids`.
5. Check every `not_applicable` lane has a reason.
6. Reject expired exceptions.
7. Compare the coverage report with `docs/invariants/coverage-matrix.json` when the report exists.
8. Enforce strict check-id adequacy: every `required` / `proof_profile_required`
   manifest `check_id` must be counted in docs and coverage, and every counted
   check must be manifest-required.
9. Bind every non-replay counted check to generated artifact text, codegen receipt
   obligations and tool-run receipt obligations.
10. Bind every replay counted check to an obligation negative control and a
   `refuted_by_replay` coverage entry.
11. Reject authoritative Lean files containing `sorry` or non-whitelisted `axiom`.
12. Reject authoritative Verus files containing cheating constructs under proof/all profile.
13. Verify generated artifacts carry source hashes in headers.
14. Verify tool-run receipts record `actual_result` and `exit_code`.
15. Refuse to count `non_blocking_skipped` as pass. It is allowed only as a
    warning with `exit_code: 0` in non-proof profiles for lanes not forbidden by
    `formal/proof-lanes.json`; it fails in proof/all.

## Protocol-critical path globs

```text
contracts/examples/**
contracts/scenarios/**
contracts/schema/**
crates/causlane-contracts/src/**
crates/causlane-core/src/domain/**
crates/causlane-replay/src/**
crates/causlane-codegen/src/**
crates/causlane-runtime/src/guarded_executor.rs
crates/causlane-runtime/src/authz.rs
crates/causlane-cli/src/formal_*.rs
crates/causlane-cli/src/bin/causlane-formal.rs
crates/causlane-cli/src/bin/causlane-formal-discipline.rs
crates/causlane-cli/src/bin/formal_discipline/**
tools/formal-*
tools/coverage-matrix
tools/proof-refinement-scope
tools/doc_projection.py
docs/invariants/**
docs/formal/proof-refinement-scope.json
docs/formal/08-proof-refinement-scope.md
docs/formal-exceptions.*
formal/**
```

Generated ignored paths may be excluded from diff classification but not from stale-check:

```text
formal/*/generated/**
formal/receipts/*.json
```

## Output JSON

```json
{
  "schema_version": 1,
  "status": "pass|fail",
  "profile": "rust",
  "protocol_critical_change": true,
  "impact_record_found": true,
  "manifest_status": "pass",
  "exceptions_status": "pass",
  "coverage_drift_status": "pass|warn|skipped|fail",
  "adequacy_status": "pass|skipped|fail",
  "proof_cheating_status": "pass",
  "artifact_status": "pass",
  "receipt_status": "pass|warn|fail",
  "errors": []
}
```

## Acceptance tests

1. Touching `crates/causlane-replay/src/lib.rs` without an impact record fails.
2. Removing a required `check_id` from an obligation fails.
3. `not_applicable` without reason fails.
4. Expired exception fails.
5. Lean file with `sorry` in authoritative path fails.
6. Verus file with `assume` fails in `--profile proof`.
7. Coverage matrix overclaim fails.
8. Generated artifact without source hash header fails stale discipline check.
9. A manifest-required `check_id` missing from docs, coverage, receipts or
   generated artifact text fails adequacy.
10. A counted `check_id` not declared as manifest-required fails adequacy.

## Current documentation rule

Docs may reference this command as mandatory inside `tools/formal-verify-all`
and available for local/PR-diff enforcement. Do not claim provider-specific CI
enforcement unless an external workflow explicitly runs it with a changed-file
source such as `--from-git` or `--changed-files`.
