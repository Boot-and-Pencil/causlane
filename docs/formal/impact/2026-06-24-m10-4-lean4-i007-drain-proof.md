# Formal Impact Record: M10.4 Lean4 I-007 Drain Proof

Date: 2026-06-24

## Summary

This change deepens M10.4 by adding a generated Lean4 theorem application for
active invariant I-007. The proof checks the finite drain-fence rule used by the
existing Kani and Verus lanes: a fence is acquirable exactly when no modeled lease
slot overlaps the fence scope while still active and not expired.

## Changed Surface

- Lean4 core vocabulary now includes the finite `DrainLeaseSlot` and
  `DrainFenceCheck` rule model.
- The single source for the Lean4 drain condition is `drainFenceClearSpec`, with
  `drainFenceAcquirable` delegating to it.
- Lean4 generated proof artifacts include `drain_after_overlap_clear`.
- The Lean4 obligation table credits I-007 only when that theorem name is present
  in the generated artifact and the Lean4 tool-run receipt passes.
- The formal obligation manifest marks Lean4 I-007 as required.

## Coverage Effect

I-007 gains Lean4 proof-lane coverage after receipt-derived coverage is
regenerated. No new invariant is activated and no runtime semantics change.

## Validation

- Lean4 Lake package build and generated theorem check through `formal-verify-all`
- stale-check for generated artifacts and receipts
- receipt-derived coverage matrix drift check
- formal discipline check
