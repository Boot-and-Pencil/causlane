# 09 — Formal evidence provenance and trust boundary

**Status:** policy (mandatory before publication; closes review finding **H5/M6**).

This document states the provenance and trust boundary of the formal evidence —
the tool-run receipts and the derived coverage report — so that no reader, gate or
downstream consumer mistakes the committed/loaded JSON for a cryptographically
signed proof.

## What the evidence is

- **Generated artifacts** (`.als` / `.p` / `.lean` / Kani / Verus sources) are
  bound to `source_bundle_hash`, `scenario_hash`, `formal_ir_hash` and
  `generated_artifact_hash`. `formal stale-check` fails closed without a matching
  receipt (finding **H4**), and codegen identifiers are collision-checked
  (finding **H3/M4**).
- **Tool-run receipts** record the **real tool's** parsed result and process exit
  code. `scripts/check-verification-full.sh` runs Alloy / Z3 / P / Kani / Verus / Lean4 and
  writes `verification/formal-full/receipts/*.tool-run.json`.
- The **coverage report** is **derived** from those receipts — never authored or
  upgraded. A non-zero exit code can never become a pass (P0-FM-002), and a proof
  containing `sorry`/`admit`/`assume`/`axiom` is not counted as a pass.

## What the evidence is NOT

- Receipts are **not cryptographically signed**. A receipt on disk is a build
  artifact, not a tamper-proof attestation; tool-run receipts are git-ignored and
  ephemeral. A receipt could, in principle, be hand-edited.
- Therefore a committed or local receipt — and any coverage status derived from it
  — **must not be presented as proof on its own**. It is only as trustworthy as the
  run that produced it.

## The trust anchor: re-derivation (CI re-derivation)

The authority is **re-running `scripts/check-verification-full.sh` on a formal-capable host**
— `ci-dispatcher.lan` carries the full toolchain (Alloy/Z3/P/Kani/Verus/Lean4).
That run regenerates the artifacts, re-runs the real tools and **rewrites** the
receipts, so a hand-edited receipt is overwritten by the next real run and cannot
survive re-derivation.

- **Publication (PUB5) requires a fresh `check-verification-full` run on the
  formal-capable CI machine.** Committed receipts are evidence for review, not the
  publication authority.
- Read coverage as *"the last real tool run on a formal-capable host reported X"*,
  not *"this is proven / signed"*. Public-facing claims use the honest-ladder
  vocabulary (`present → compiled → checked → payload-bound → discriminating →
  authoritative`); see `verification/formal-full/README.md` and
  `docs/formal/04-formal-discipline-and-anti-theatre.md`.

## Enforcement

`tools/pre-publication-review-gate` checks that this provenance policy exists
(`PUB-FORMAL-PROVENANCE-POLICY`). The CLI coverage output and the coverage report
module carry a provenance note pointing here, so the report cannot be read as a
signed proof.

## Out of scope (future hardening)

- Cryptographic signing / keyed attestation of receipts (key-management story).
  Until then, **re-derivation on the formal-capable CI machine is the authority.**
