# ADR-0026: Invariant expansion starts with a shared catalog

## Status

Accepted.

## Context

M10.1 expands the invariant surface beyond `I-001..I-010`, but the existing
tooling used several independent copies of the active invariant-id range across
Rust validators, JSON schemas and formal-discipline checks. Adding `I-011` in
one place before the range is centralized would create drift or accidental
coverage overclaim.

## Decision

`causlane-contracts` owns the stable invariant-id catalog:

- active ids: `I-001..I-010`;
- planned ids: `I-011..I-020`;
- known ids: `I-001..I-020`.

Compiled bundle formal obligations and Formal IR accept active ids only. The
formal obligation manifest may reserve planned ids with `planned` lanes, but
planned ids do not appear in the coverage matrix and do not count as evidence.

JSON schemas share `contracts/schema/common.schema.json` for invariant-id
patterns. The stdlib schema validator supports relative `$ref` between schema
files so the range is not copied across schemas.

## Consequences

- M10.1 can introduce `I-011..I-020` incrementally without adding empty coverage
  rows or pretending planned checks have passed.
- Any future promotion from planned to active must add concrete lane checks,
  update active-id validation and regenerate/validate the formal coverage
  matrix.
- No Formal IR field, replay trace field, receipt field or generated formal
  artifact changes are introduced by the bootstrap itself.
