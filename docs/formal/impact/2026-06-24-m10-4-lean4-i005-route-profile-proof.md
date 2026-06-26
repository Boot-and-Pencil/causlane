# Formal Impact Record: M10.4 Lean4 I-005 Route/Profile Proof

Date: 2026-06-24

## Summary

This change deepens M10.4 by adding a generated Lean4 theorem application for
active invariant I-005. The proof binds to Formal IR predicate facts and checks
that each route's lifecycle class matches the class derived from its consequence
profile.

## Changed Surface

- Lean4 core vocabulary now includes consequence profiles, lifecycle classes
  and predicate route facts.
- Lean4 generated proof artifacts include `generatedPredicateRoutes` and
  `route_profile_compatibility`.
- The Lean4 obligation table credits I-005 only when that theorem is present in
  the generated artifact and the Lean4 tool-run receipt passes.
- The formal obligation manifest marks Lean4 I-005 as required.

## Coverage Effect

I-005 gains Lean4 proof-lane coverage after receipt-derived coverage is
regenerated. No new invariant is activated and no runtime semantics change.

## Validation

- Lean4 Lake package build and generated theorem check through `formal-verify-all`
- stale-check for generated artifacts and receipts
- receipt-derived coverage matrix drift check
- formal discipline check
