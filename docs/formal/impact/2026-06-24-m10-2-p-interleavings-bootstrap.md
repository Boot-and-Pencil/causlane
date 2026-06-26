# Formal Impact Record: M10.2 P Interleavings Bootstrap

Date: 2026-06-24

## Summary

This change deepens the generated P lane with bounded controls for retry,
authorization revocation and constraint epoch races. It is a behavior-preserving
formal evidence expansion: replay semantics and active coverage remain unchanged.

## Changed Surface

- Formal IR lease facts now carry the scenario lease epoch with default `0`.
- Generated P payloads now include `executionKey`, `authzStage` and `leaseEpoch`.
- Generated P monitors now include:
  - `NoDuplicateHardExecutionForSameIdempotencyKey`
  - `AuthzRevocationBeforeBarrierBlocksExecution`
  - `NoStaleConstraintEpochAdmission`
- `tools/formal-verify-all` runs three P-only controls under
  `contracts/scenarios/p_controls/`.

## Affected Invariants

- `I-012`: planned P evidence hook for duplicate hard execution under retry.
- `I-014`: planned P evidence hook for Deny-before-barrier revocation.
- `I-018`: planned P evidence hook for stale constraint epoch admission.

These ids remain planned. `ACTIVE_INVARIANT_IDS`, Formal IR invariant acceptance
and the coverage matrix still cover only `I-001..I-010`.

## Validation

- `./tools/cargo-dev test -p causlane-codegen --all-targets --all-features --locked targets::tests`
- `./tools/schema-validate-all`
- focused P compile/check of the three new controls, each refuted with non-zero
  `p check` exit status.
