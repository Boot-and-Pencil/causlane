# Formal Impact Record: M10.2 Obligation Alignment

Date: 2026-06-24

## Summary

This change corrects the formal obligation manifest entries for the M10.2 P-only
planned evidence hooks. The generated P monitors and the M10.2 impact record
identify these hooks as evidence for `I-012`, `I-014` and `I-018`; the manifest
now matches that mapping.

## Changed Surface

- `NoDuplicateHardExecutionForSameIdempotencyKey` is recorded under `I-012`.
- `AuthzRevocationBeforeBarrierBlocksExecution` is recorded under `I-014`.
- `NoStaleConstraintEpochAdmission` is recorded under `I-018`.
- The formal-discipline manifest validator now rejects these known M10.2 hooks
  when they are attached to a different invariant or lack their paired P control.

## Coverage Effect

No active coverage changes. `I-011..I-020` remain planned invariant ids, and the
coverage matrix still reports active coverage only for `I-001..I-010`.

## Validation

- `causlane-formal-discipline` manifest unit tests
- `formal-discipline-check`
- standard schema, product-track and formal gates
