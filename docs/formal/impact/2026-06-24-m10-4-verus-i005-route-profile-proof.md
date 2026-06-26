# Formal Impact Record: M10.4 Verus I-005 Route/Profile Proof

Date: 2026-06-24

## Summary

This change deepens M10.4 by adding a Verus proof obligation for active
invariant I-005. The proof mirrors the existing Kani route/profile rule:
a route lifecycle class is compatible with a consequence profile only when it is
the exact class derived from that profile.

## Changed Surface

- Verus generated proof artifacts include `route_profile_compatibility`.
- The Verus obligation table credits I-005 only when that proof function is
  present in the generated artifact and the Verus tool-run receipt passes.
- The formal obligation manifest marks Verus I-005 as required.
- Later update: Lean4 I-005 was promoted to a generated route/profile theorem
  model over Formal IR predicate facts on 2026-06-24.

## Coverage Effect

I-005 gains Verus proof-lane coverage after receipt-derived coverage is
regenerated. No new invariant is activated and no runtime semantics change.

## Validation

- Verus no-cheating tool run through `formal-verify-all`
- stale-check for generated artifacts and receipts
- receipt-derived coverage matrix drift check
- formal discipline check
