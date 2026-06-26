# ADR-0029: M10.4 Proof-Lane Blocking Contract

Date: 2026-06-24

## Status

Accepted

## Context

Verus and Lean4 already run as blocking proof lanes in `tools/formal-verify-all`,
but some older docs and comments still described proof facets as optional or
non-blocking. That creates a stale-policy risk: future changes could reintroduce
proof-lane skips by relying on old prose rather than the executable gate.

## Decision

Introduce `formal/proof-lanes.json` as the machine-readable source for proof
lanes that must always run in `formal-verify-all`.

`tools/formal-exceptions-check` now reads that contract and rejects:

- any exception entry targeting an always-blocking proof lane;
- any `--skipped-target` request for an always-blocking proof lane.

The current contract marks `verus` and `lean4` as blocking,
receipt-derived-coverage lanes with `exception_allowed: false`.

## Consequences

- Verus and Lean4 cannot be silently downgraded through
  `docs/formal-exceptions.json`.
- Changing that reality requires changing the proof-lane contract and its schema
  validation in the same review.
- No new theorem/proof semantics or coverage rows are introduced by this
  governance hardening.
