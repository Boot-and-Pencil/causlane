# Formal discipline and anti-theatre policy

> **Repository integration status:** proposed lifecycle discipline. This
> document is design/process authority only; current proof evidence remains the
> generated chain from compiled bundle and scenario through Formal IR, generated
> artifacts, receipts, stale-check and derived coverage.

## Purpose

Formal theatre happens when the repository contains models or proofs, but they are not mechanically tied to what the code and runtime actually do. This policy makes such states visible and merge-blocking.

## Authority hierarchy

Authoritative evidence, from strongest to weakest:

1. generated artifact bound to `source_bundle_hash`, `scenario_hash`, `formal_ir_hash` and fresh receipt;
2. real tool-run receipt with exit code and parsed result;
3. replay negative control with exact stable error code;
4. Kani/Verus/Lean4 proof run under authoritative profile;
5. derived coverage report;
6. drift-checked documentation generated from coverage.

Not authoritative by itself:

```text
README prose
hand-written generic Alloy/P/Lean/Verus sketches
comments in Rust code
human status tables
a sample receipt
an artifact without stale-check
a proof with `sorry`, `assume`, `admit` or non-expired exception counted as pass
```

## Evidence status rules

A lane status is legal only if it was derived mechanically:

| Status | Meaning | Can count as covered? |
|---|---|---:|
| `not_modelled` | no model exists | no |
| `not_applicable` | lane intentionally does not model this invariant | no, but another lane may cover |
| `declared` | obligation exists but no generated artifact | no |
| `generated` | artifact generated but tool not run | no |
| `tool_passed` | real tool run passed | yes, if stale-check fresh |
| `negative_control_refuted` | invalid scenario rejected by expected lane | yes, for that lane/check |
| `proved_no_cheating` | proof lane passed without cheating constructs | yes |
| `non_blocking_skipped` | explicit policy skip, expiry-bound | no |
| `covered` | derived rollup from required evidence | only coverage reporter may emit |

## Forbidden practices

The following are merge blockers:

1. Marking an invariant covered in docs without a matching coverage report entry.
2. Hand-maintaining bundle-specific model facts.
3. Adding a new protocol-critical Rust behavior before adding formal obligations and negative controls.
4. Using `jq`, scripts or manual edits to upgrade a failed receipt or coverage status to pass.
5. Accepting a proof file with `sorry`, `assume`, `admit`, `external_body` or equivalent cheating in authoritative profile.
6. Claiming a lane as applicable when the generator does not emit the named `check_id`.
7. Letting exceptions expire silently.
8. Treating a context-pack omission of generated artifacts/build outputs as evidence of pass/fail. Only the live gate is authority.
9. Proving a theorem over a frozen flag that no transition can change and counting it as protocol preservation.
10. Adding a new invariant without at least one planned discriminating negative control or an explicit reason why negative control is impossible.

## Required anti-vacuity controls

Every nontrivial invariant must have at least one anti-vacuity mechanism:

```text
negative scenario that fails if the check is removed
Alloy counterexample control
P monitor-firing control
Kani mutant/fail harness or bounded branch coverage
Verus/Lean theorem dependency on generated facts
coverage check that requires artifact-present check_id
```

A proof is suspicious if it still passes when the related rule predicate is weakened. For important rules, add a mutation test or a review note explaining which conjuncts are load-bearing.

## Protocol-critical paths

Changes touching these paths require Formal Impact Record:

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
docs/invariants/**
docs/formal-exceptions.*
formal/** except generated outputs ignored by context packs
```

## Machine enforcement requirements

**Implementation status for repo 010:** `tools/formal-discipline-check` is
implemented for local and PR-diff checks and is mandatory inside the
`tools/formal-verify-all` repo gate. Provider-specific CI enforcement is a
separate workflow concern; this repository currently exposes the gate but does
not define a CI provider workflow.

`tools/formal-discipline-check`:

1. reads changed files;
2. detects protocol-critical changes;
3. requires a Formal Impact Record for such changes;
4. validates `formal/obligations/*.yaml` manifest shape and safety fields;
5. checks every `required` / `proof_profile_required` lane has `check_ids` and every `not_applicable` lane has a reason;
6. fails if a manifest-required `check_id` is missing from coverage/docs or if docs/coverage count a non-required check;
7. binds non-replay checks to generated artifact text plus codegen/tool-run receipt obligations;
8. binds replay checks to manifest negative controls and `refuted_by_replay` coverage entries;
9. rejects Lean4 authoritative files with `sorry`/non-whitelisted `axiom`;
10. rejects Verus authoritative files with cheating constructs under proof/all profile;
11. checks generated headers and receipts for known targets;
12. compares the coverage report with `docs/invariants/coverage-matrix.{json,md}`;
13. checks `docs/formal/proof-refinement-scope.json` against its generated Markdown projection;
14. checks `docs/formal-exceptions.json` expiry.

## Review discipline

A reviewer must ask:

1. What concrete behavior changed?
2. Which invariant/protocol does it affect?
3. Where is the negative control?
4. Which lane is responsible?
5. Is the model generated from the same IR/bundle as the runtime code?
6. Does the coverage report, not prose, claim success?
7. Could the proof pass if the implementation did the opposite?

If the answer is unclear, the PR is not ready.

## Provenance and trust boundary

Receipts are **not cryptographically signed**, so neither a committed receipt nor
the derived coverage report is a signed proof on its own — they are evidence of the
last real tool run. The publication authority is **re-deriving** the evidence by
re-running `tools/formal-verify-all` on a formal-capable host (CI re-derivation).
See `09-formal-evidence-provenance-and-trust-policy.md` (review finding H5/M6).
