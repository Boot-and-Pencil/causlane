# Formal Impact Record: M10.6 Proof/Refinement Scope

Date: 2026-06-24

## Summary

This change advances M10.6 by adding a schema-validated proof/refinement scope
artifact and a generated Markdown projection. The artifact classifies formal
evidence strength as proved, bounded, simulated, tested, assumed or out of
scope without creating a second coverage inventory.

## Changed Surface

- `docs/formal/proof-refinement-scope.json` records claim-strength
  classifications.
- `contracts/schema/formal_proof_refinement_scope.schema.json` validates the
  artifact in the schema gate.
- `tools/proof-refinement-scope --check` fails if the generated Markdown
  projection drifts from the JSON.
- `scripts/check-verification-full.sh` runs the proof/refinement scope drift check after
  the coverage-matrix check.
- Active formal docs now point to the generated scope projection for
  proved/bounded/simulated/tested/assumed/out-of-scope classification.

## Coverage Effect

No runtime semantics, replay rules, proof obligations or coverage cells change.
Current per-invariant coverage remains receipt-derived in
`docs/invariants/coverage-matrix.json`.

## Negative Controls

No protocol negative control is required because this pass changes docs/tooling
only. Non-vacuity comes from JSON schema validation plus generated Markdown drift
checking in the formal gate.

## Validation

- proof/refinement scope schema validation
- proof/refinement scope drift check
- coverage matrix drift check
- product-track status consistency check
- formal discipline check
- standard Rust, formal and schema gates
