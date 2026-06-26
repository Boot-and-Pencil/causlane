# Formal Impact Record: clippy-clean the workspace + gate clippy

## Change metadata

- Change ID: FIR-2026-06-19-clippy-clean-and-gate
- PR/issue: hardening (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-19
- Impact class: F5 (formal tooling / gate hardening) — behavior-preserving

## Touched protocol-critical paths

```text
crates/causlane-codegen/src/lean4_target.rs
crates/causlane-cli/src/bin/causlane-formal-discipline.rs
crates/causlane-cli/src/bin/formal_discipline/adequacy.rs
crates/causlane-cli/src/bin/formal_discipline/args.rs
crates/causlane-cli/src/bin/formal_discipline/manifest.rs
crates/causlane-cli/src/bin/formal_discipline/mod.rs
crates/causlane-cli/src/bin/formal_discipline/paths.rs
tools/formal-verify-all
tools/formal-ready
```

## Summary

The workspace configures clippy `pedantic` lints + per-crate `#![deny(warnings)]`,
but `just clippy` was RED and clippy was NOT part of the mandatory gate
(`formal-verify-all`/`formal-ready` ran `fmt --check` + `check` + `test`, not
clippy) — so pedantic clippy debt accumulated unnoticed. This clears ALL of it and
wires clippy into the gate so it cannot regress.

Behavior-preserving fixes (all pre-existing debt, surfaced by clippy 1.96.0):

- **`lean4_target.rs`** (Lean4 codegen): `push_theorem(proposition: &str)` and
  `join_conjunction(clauses: &[String])` (by-ref instead of by-value); `push_events`
  split into a `push_event_def` helper with local `anchor`/`barrier`/`witness`
  bindings (`too_many_lines`). The **generated Lean4 artifact is byte-identical**
  (sha256 `441e982a…` before and after).
- **`formal_discipline` CLI**: `single_error(&AdequacyError)` + `map_err`
  closures; `run_cli(argv: &[String])` + main caller; rename `args`→`parsed`
  (`similar_names`); `clone_from` (`assigning_clones`); `("01"..="12")/("01"..="31").contains`
  (`manual_range_contains`); two `r#"…"#`→`r"…"` (`needless_raw_string_hashes`);
  `assert_contains` rewritten via `result.err().unwrap_or_default()` (drops
  `assert!(false,…)` without using `panic!`/`expect`, per the repo hooks); a
  justified `#[allow(clippy::case_sensitive_file_extension_comparisons)]` on the two
  path matchers (source extensions are intentionally lower-case / case-sensitive).
- **Gate wiring**: `tools/formal-verify-all` and `tools/formal-ready` now run
  `clippy --workspace --all-targets --all-features --locked -- -D warnings` after
  `fmt --check`; `formal-ready`'s report `gates` array gains `cargo-clippy`.

## Affected invariants

```text
none — no kernel semantics, generated artifact, or invariant coverage change.
new invariant ids: none
```

## Affected formal models

```text
none — the Lean4 generator's output is byte-identical; no other generator touched.
```

## Contract changes

- Bundle / Formal IR / replay-trace / receipt / coverage fields: none.
- Core semantic change: none. Lint-only cleanup + an added gate step.

## Required negative controls

| Check | Lane | What it guards | Status |
|---|---|---|---|
| Lean4 artifact byte-identity | codegen | `formal generate lean4` output unchanged (sha256 441e982a…) | verified |
| `causlane-codegen` + `causlane-cli` test suites | unit | the codegen + discipline-CLI behavior is unchanged | existing — green |
| `formal-discipline-check --profile rust --no-diff` | tool | the edited gating tool still reports `status=pass` | verified |
| `just clippy` | gate | whole workspace clippy-clean (exit 0) | new regression guard |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| All | unchanged | yes | rust |

## Not applicable lanes

No generated lane changes. This is lint/tooling hygiene plus a gate step; the
Lean4 generator's emitted bytes are unchanged, so every lane's inputs are identical.

## Acceptance commands

```bash
just clippy
just formal-ready
just formal-verify-all
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: TZ-007 verified-merge runtime/provider enforcement (I-006);
  Verus/Lean4 exception renewal before 2026-09-01; S04 scenario/contract-testing.
