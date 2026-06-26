# Current-state review: dispatcher-014 patch pack

Status: accepted review note for the quality/architecture refactor track.

## Summary

Patch pack 014 correctly identifies the next class of risk: Causlane now has
enough implementation mass that further feature work can widen public API,
duplicate orchestration logic, or let formal/runtime paths drift. The relevant
next step is a staged refactor track, starting with repository-shape gates that
run before expensive Rust or formal tooling.

The archive was produced against an older snapshot. Current `main` already has
tracked `causlane-formal` and `causlane-formal-discipline` binaries, and those
implement more than the archive scaffold. Those scaffold files are therefore not
applied.

## Observed strengths

- `causlane-core` remains runtime-free.
- `causlane-contracts` owns registry, bundle, canonical hashing and plan material.
- `causlane-codegen` emits target-specific Alloy, P, Kani, Verus and Lean4 artifacts.
- `causlane-replay` is the executable oracle for bundle-bound traces.
- `causlane-runtime` owns adapters, audit storage, execution guards and observability.
- Formal receipts, coverage and stale checks are present and are derived from generated artifacts.

## Current quality risks

### QR-001: repository shape needs a cheap first gate

Declared Cargo binaries, core dependency boundaries and duplicate schema keys
should fail before Rust/formal gates run. The R0 `architecture-lint` gate covers
these as hard errors.

### QR-002: public API is too broad through glob re-exports

The current baseline has 32 `ARCH-004` warnings for broad public glob
re-exports. They are accepted as R0 warnings and become the R2 public API
narrowing backlog.

### QR-003: modules are nearing or exceeding refactor pressure

Several codegen, replay, runtime and CLI files are large. R0 reports the budget;
R3 will split modules by authority boundary, not by arbitrary line count.

### QR-004: CLI orchestration still needs an application layer

The formal helper binary already performs real generation and coverage
derivation, but `causlane-cli/src/main.rs` remains a mixed CLI boundary. R1
should move command services into `causlane_cli::app::*` without changing
semantics.

### QR-005: generated truth chain must remain first-class

Formal artifacts must keep consuming compiled bundle and scenario facts. No
refactor stage may introduce manually maintained parallel formal facts.

## Immediate R0 objective

R0 adds:

```text
tools/architecture-lint
.devinfra/state-schema/architecture-lint.schema.json
just architecture-lint*
just refactor-readiness
docs/refactor/*
ADR-0022..0024
```

R0 does not change runtime, replay, formal or public Rust semantics.
