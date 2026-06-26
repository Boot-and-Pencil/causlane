# Formal verification exceptions

This file is the **prose** companion to the **executable** policy in
[`docs/formal-exceptions.json`](formal-exceptions.json) (schema:
`contracts/schema/formal_exceptions.schema.json`). The JSON is the machine
source of truth; this file explains it. They must stay in sync with:

- `docs/formal-exceptions.json` (executable policy — enforced by `tools/formal-exceptions-check`)
- `crates/causlane-cli/src/bin/causlane-formal.rs` (declared `LANE_REALITY` matrix)
- `docs/invariants/coverage-matrix.json`
- `target/causlane/formal-coverage-report.json` (derived, never patched — P0-FM-002)

## Executable enforcement (P1-FM-012)

`tools/formal-exceptions-check --profile <profile> [--skipped-target <lane>]`
enforces the policy with `jq` (no extra runtime deps):

- any exception past its `allowed_until` date fails the gate;
- a lane may be treated as non-authoritative (`non_blocking_skipped`) only when a
  non-expired exception lists the active profile in `allowed_profiles` and not in
  `forbidden_profiles`.

`tools/formal-verify-all` calls it up front to catch any expired exception. The
Verus and Lean4 lanes are now **always run and blocking**, so there is no
`non_blocking_skipped` path to justify. The machine-readable contract for that
reality is `formal/proof-lanes.json`; `tools/formal-exceptions-check` rejects
any exception or skipped-target request for a proof lane marked
`exception_allowed: false`.

## Open exceptions

**None.** The `LANE_REALITY_{VERUS,LEAN4}_NON_BLOCKING` exceptions were dropped on
2026-06-21: both proof lanes are now **always run and blocking** in
`tools/formal-verify-all` (confirmed passing `verus --no-cheating` and
`lake build CauslaneFormal` + `lake env lean`).

So every `just formal-verify-all` run executes the Alloy, P, Kani, **Verus and
Lean4** lanes, derives coverage from real tool-run receipts + exit codes, runs the
replay-backed negative controls, and requires the derived coverage report to be
`pass`. The fast dev loop (`just formal-ready`, `cargo test`, `clippy`) does not run
the proof lanes. Any not-yet-real lane, if introduced, goes in
`formal-exceptions.json` with an `allowed_until` date.

## Scope Notes

Some cells are `not_applicable` rather than waived:

| Invariant | `not_applicable` lanes | Reason |
|---|---|---|
| I-004 Overlay monotonicity | replay, Alloy, P | No overlay reducer/event in the runtime trace schema, and neither Alloy nor P models overlay. **Kani** exercises `ObligationSet::preserved_by` against the real predicate; **Verus** carries the overlay lemma; **Lean4** carries a finite obligation-set theorem application (blocking, always-on). |
| I-010 Constraint updates do not rewrite truth | replay, Alloy | No constraint-update reducer/event in the runtime trace schema, and Alloy does not model it. **P** and **Kani** cover truth immutability; **Verus** carries the constraint-update lemma; **Lean4** carries a finite constraint-update future-only theorem application (blocking, always-on). |

These are not open waivers. If the corresponding runtime events/reducers land,
the `not_applicable` cells must become required passing cells in the same PR.
