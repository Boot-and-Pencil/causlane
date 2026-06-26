# Formal Impact Record: M10.4 Proof-Lane Blocking Contract

Date: 2026-06-24

## Summary

This change starts M10.4 by making the current Verus/Lean4 always-blocking
reality machine-readable. It prevents future exception-policy drift from
silently treating either proof lane as non-authoritative.

## Changed Surface

- `formal/proof-lanes.json` records the always-blocking proof-lane contract.
- `contracts/schema/formal_proof_lanes.schema.json` validates the contract.
- `tools/formal-exceptions-check` rejects exceptions or skipped-target requests
  for lanes that the contract marks as non-skippable.
- Current docs and code comments no longer describe Verus/Lean4 as optional or
  non-blocking in active policy.

## Coverage Effect

No active coverage changes. Verus and Lean4 continue to derive coverage only
from real tool-run receipts and generated proof artifacts.

## Validation

- proof-lane schema validation
- formal-exceptions policy checks, including negative controls for protected lanes
- `schema-validate-all`
- `formal-verify-all`
- coverage-matrix check
