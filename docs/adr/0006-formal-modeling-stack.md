# ADR-0006: Formal modeling stack

- Status: accepted
- Date: 2026-06-05

## Context

Different verification tools answer different questions. No single tool should be the only formal surface.

## Decision

Use:

```text
Alloy  for relational counterexamples.
P-lang for protocol/interleaving bugs.
Kani   for bounded Rust-code checks.
Verus  for abstract preservation/soundness proofs.
```

## Consequences

The project can attack both model shape and runtime event ordering. The cost is maintaining generated projections and a coverage matrix.

## Enforcement

Formal models should be generated from or tied to the compiled dispatch bundle. Manual model drift is treated as a gap.
