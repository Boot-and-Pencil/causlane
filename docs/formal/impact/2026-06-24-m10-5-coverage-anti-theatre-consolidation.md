# Formal Impact Record: M10.5 Coverage Anti-Theatre Consolidation

Date: 2026-06-24

## Summary

This change advances M10.5 by making the human coverage matrix a generated,
fully drift-checked projection of the receipt-derived formal coverage report.
It also removes hand-maintained live coverage inventories from active status
docs, so current invariant cells and proof `check_id`s have one source of truth.

## Changed Surface

- `tools/coverage-matrix --check` now compares the full generated Markdown
  coverage matrix, not only the JSON and table cells.
- `docs/invariants/coverage-matrix.md` includes a generated lane summary derived
  from the same report as the table.
- Formal README/status/catalog docs now point to the generated matrix/report for
  current coverage inventory instead of restating Lean4/Kani invariant lists.
- M10.5 product-track status moves to `exists_expand` for this docs/tooling
  hardening pass.

## Coverage Effect

No runtime semantics, replay rules, active invariants or proof obligations
change. This pass reduces reporting drift risk: prose coverage summaries cannot
claim a lane unless the fresh coverage report regenerates the same Markdown.

## Negative Controls

No new protocol negative control is required because this change does not alter
protocol behavior. The non-vacuity check is the formal gate itself:
`tools/coverage-matrix --check` fails when the committed Markdown differs from
the generated body.

## Validation

- coverage matrix regeneration
- coverage matrix drift check
- product-track status consistency check
- formal discipline check
- standard Rust, formal and schema gates
