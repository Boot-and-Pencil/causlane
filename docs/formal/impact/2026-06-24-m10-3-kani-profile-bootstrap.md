# Formal Impact Record: M10.3 Kani Profile Bootstrap

Date: 2026-06-24

## Summary

This change starts M10.3 by making Kani runner configuration explicit and
machine-validated. The generated Kani harnesses and coverage obligations are
unchanged; `check-verification-full` now consumes a checked profile for the Kani
fixture, output format and lane-specific unwind bound.

## Changed Surface

- `verification/formal-full/kani/profile.json` records the Kani fixture and lane unwind bounds.
- `contracts/schema/formal_kani_profile.schema.json` validates the profile.
- `scripts/check-verification-full.sh` reads Kani run parameters from the profile instead
  of hardcoding them.
- `tools/schema-validate-all` validates the Kani profile alongside existing
  schema gates.

## Coverage Effect

No active coverage changes. Kani still covers the same active invariant rows,
and planned invariants remain outside coverage credit.

## Validation

- profile schema validation
- `schema-validate-all`
- `check-verification-full`
- coverage-matrix check
