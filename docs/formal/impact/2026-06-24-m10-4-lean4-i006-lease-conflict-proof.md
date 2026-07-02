# Formal Impact Record: M10.4 Lean4 I-006 Lease Conflict Proof

Date: 2026-06-24

## Summary

This change deepens M10.4 by adding generated Lean4 theorem applications for
active invariant I-006. The proof checks the finite lease-conflict rule:
same-resource, same-scope claims conflict when at least one side is exclusive and
no verified merge protocol applies; a verified merge clears that conflict.

## Changed Surface

- Lean4 core vocabulary now includes the finite `ClaimMode` model and one
  `claimModesConflict` predicate.
- Lean4 generated proof artifacts include `lease_conflict_fail_closed` and
  `verified_merge_algebra`.
- The Lean4 obligation table credits I-006 only when both theorem names are
  present in the generated artifact and the Lean4 tool-run receipt passes.
- The formal obligation manifest marks Lean4 I-006 as required.

## Coverage Effect

I-006 gains Lean4 proof-lane coverage after receipt-derived coverage is
regenerated. No new invariant is activated and no runtime semantics change.

## Validation

- Lean4 Lake package build and generated theorem check through `check-verification-full`
- stale-check for generated artifacts and receipts
- receipt-derived coverage matrix drift check
- formal discipline check
