# ADR-0022: Large refactor quality track

## Status

Accepted.

## Context

Causlane now has enough implementation surface that the primary risk has shifted
from missing primitives to accidental authority growth. The workspace contains
formal codegen, replay, runtime adapters, product-track docs, scenario gates and
derived coverage. Without a dedicated refactor track, each new feature can widen
public API, duplicate orchestration logic or let formal/runtime paths drift.

## Decision

Create a staged quality/architecture refactor track before more deep feature
expansion. The track introduces repository-shape checks, explicit module
boundaries, CLI service split, public API narrowing and module-size budgets.

The first stage, R0, is intentionally bootstrap-only: it adds architecture lint
tooling and refactor documentation without changing runtime, replay, formal or
public Rust semantics.

## Consequences

- Broken declared binaries and core/runtime boundary regressions are detected
  early.
- Public API and module-size problems become visible before they are strict
  gates.
- Formal orchestration can be moved toward reusable services without changing
  generated truth semantics.
- Import churn and module movement are expected in later stages, but each stage
  has its own gate.

## Constraints

- Formal artifacts remain generated from compiled bundle/scenario facts.
- Observed truth remains audit-authoritative.
- No hard effect crosses execution without durable barrier/capability.
- No projection is emitted without observed-truth anchor.
- No adapter may claim stronger guarantees than certification proves.
