# Formal Impact Record: M10.4 Lean4 I-010 Constraint Update Proof

Date: 2026-06-24

## Summary

This change deepens M10.4 by adding a generated Lean4 theorem application for
active invariant I-010. The proof checks the finite committed-truth rule used by
the existing P, Kani and Verus lanes: a constraint update may affect future
constraint state, but it must not rewrite any truth category that is already
committed.

## Changed Surface

- Lean4 core vocabulary now includes finite `CommittedTruth` and
  `ConstraintUpdate` rule models.
- The single source for the Lean4 rule is `preservesCommittedTruth`, driven from
  `constraintTruthPairs` so the per-category formula is not duplicated.
- Lean4 generated proof artifacts include `constraint_update_future_only`.
- The Lean4 obligation table credits I-010 only when that theorem name is
  present in the generated artifact and the Lean4 tool-run receipt passes.
- The formal obligation manifest marks Lean4 I-010 as required.

## Coverage Effect

I-010 gains Lean4 proof-lane coverage after receipt-derived coverage is
regenerated. No new invariant is activated and no runtime semantics change.

## Validation

- Lean4 Lake package build and generated theorem check through `check-verification-full`
- stale-check for generated artifacts and receipts
- receipt-derived coverage matrix drift check
- formal discipline check
