# ADR-0027: M10.2 P Interleavings Bootstrap

**Status:** Accepted

**Date:** 2026-06-24

## Context

M10.1 reserved planned invariant ids `I-011..I-020` without activating coverage.
M10.2 needs deeper bounded race evidence for retry, authz revocation and
constraint epoch behavior, but the replay oracle and coverage matrix still only
credit active invariants `I-001..I-010`.

## Decision

Add M10.2 as a P-first bootstrap:

- project existing scenario payload into P: execution key, authz stage and lease
  epoch;
- add P monitors for duplicate retry execution, authz Deny-before-barrier and
  stale constraint-epoch admission;
- add P-only control scenarios that are explicitly run by `tools/formal-verify-all`;
- keep those scenarios out of replay-negative coverage by avoiding the
  `*_invalid.scenario.yaml` suffix;
- keep `I-012`, `I-014` and `I-018` planned until replay/proof/coverage
  promotion happens in a later slice.

## Consequences

P now has executable, bounded evidence for three M10.2 race families without
claiming new active invariant coverage. Cancellation/supersession and worker
partition recovery remain planned work.
