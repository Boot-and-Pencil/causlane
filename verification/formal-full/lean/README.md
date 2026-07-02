# Lean4 lane

This directory is the Lean4 package for the generated proof lane. Lean4 is an
**always-on, blocking** proof tool: `FormalTarget::Lean4` generates
scenario-bound theorem applications from Formal IR, records codegen/tool-run
receipts, participates in stale-check and coverage, and is checked with
`lake build CauslaneFormal` + `lake env lean` on **every** `scripts/check-verification-full.sh`
run; a non-zero exit fails the gate.

The `LANE_REALITY_LEAN4_NON_BLOCKING` exception was dropped 2026-06-21. The fast
dev loop (`just formal-ready`, `cargo test`, `clippy`) does not run Lean4.

## Intended layout

```text
verification/formal-full/lean/CauslaneFormal/Core.lean  generic event/payload vocabulary
verification/formal-full/lean/lakefile.lean             Lean package manifest
verification/formal-full/lean4/generated/               generated theorem applications (git-ignored)
```

## Proof scope

The current Lean4 lane proves the scenario-bound theorem applications credited
by the generated coverage matrix. Treat
[`docs/invariants/coverage-matrix.md`](../../docs/invariants/coverage-matrix.md)
and its backing coverage report as the inventory for current invariant cells and
named theorem `check_id`s; this README describes the lane role only.

Lean4 does not by itself prove that Rust code implements the protocol. That
connection still comes from replay, Kani and Verus evidence.

See `docs/formal/03-lean4-verus-proof-obligations.md`.
