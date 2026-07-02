# Formal Impact Record: M10.4 Verus I-007 Drain Proof

Date: 2026-06-24

## Summary

This change deepens M10.4 by adding a Verus proof obligation for active
invariant I-007. The proof mirrors the existing Kani `DrainFenceCheck` rule:
a drain fence is clear only when no lease slot overlaps the fence scope while
still active and not expired.

## Changed Surface

- Verus generated proof artifacts include `drain_after_overlap_clear`.
- The Verus obligation table credits I-007 only when that proof function is
  present in the generated artifact and the Verus tool-run receipt passes.
- The formal obligation manifest marks Verus I-007 as required.
- At the time of this Verus pass, Lean4 I-007 remained planned; the Lean event model did not carry
  drain/lease payloads.

Later update (2026-06-24): Lean4 I-007 was promoted to a generated
`drain_after_overlap_clear` theorem over the same finite two-slot
`DrainFenceCheck` shape used by Kani and Verus.

## Coverage Effect

I-007 gains Verus proof-lane coverage after receipt-derived coverage is
regenerated. No new invariant is activated and no runtime semantics change.

## Validation

- Verus no-cheating tool run through `check-verification-full`
- stale-check for generated artifacts and receipts
- receipt-derived coverage matrix drift check
- formal discipline check
