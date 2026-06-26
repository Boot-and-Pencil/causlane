# Formal Impact Record: M10.4 Lean4 I-004 Overlay Proof

Date: 2026-06-24

## Summary

This change deepens M10.4 by adding a generated Lean4 theorem application for
active invariant I-004. The proof checks the finite obligation-set overlay rule:
if an overlay is accepted by `preservedBy`, every obligation required by the base
set remains required by the overlaid set.

## Changed Surface

- Lean4 core vocabulary now includes the five-field overlay `ObligationSet`
  model and an exhaustive finite monotonicity checker.
- Lean4 generated proof artifacts include `overlay_monotonicity`.
- The Lean4 obligation table credits I-004 only when that theorem is present in
  the generated artifact and the Lean4 tool-run receipt passes.
- The formal obligation manifest marks Lean4 I-004 as required.

## Coverage Effect

I-004 gains Lean4 proof-lane coverage after receipt-derived coverage is
regenerated. No new invariant is activated and no runtime semantics change.

## Validation

- Lean4 Lake package build and generated theorem check through `formal-verify-all`
- stale-check for generated artifacts and receipts
- receipt-derived coverage matrix drift check
- formal discipline check
