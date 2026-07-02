# ТЗ: большой системный рефакторинг качества и архитектуры

Status: accepted for staged implementation.
Audience: maintainers and implementation agents.
Scope: repository-wide quality, architecture and formal/runtime alignment.

## Goal

Stabilize Causlane as a maintainable library/platform where:

- the semantic kernel stays small and checkable;
- formal artifacts are generated from compiled bundles and scenarios;
- runtime/adapters do not leak into core;
- CLI boundary code is thin and reusable services live below it;
- public API is explicit instead of accidental glob exposure;
- every gate can be reproduced from a clean checkout;
- large modules are split by authority boundary.

## Non-goals

Do not use this refactor track to introduce:

- a workflow engine;
- a new scheduler/runtime backend;
- broad scenario fixture rewrites;
- new replay or plan/bundle hash semantics;
- a crate rename;
- generated artifact regeneration unless the stage explicitly requires it.

## Invariants

Refactor work must preserve:

```text
registry/contracts -> compiled bundle -> runtime/replay/formal generated inputs
observed truth has one audit authority
execution-bearing action requires durable barrier
projection requires observed-truth anchor
overlay/constraint/authz may strengthen, never weaken kernel obligations
lanes provide capacity/capability, not semantic authority
```

## Work Packages

### WP-001: Shape/readiness gates

Add and enforce:

```text
tools/architecture-lint
just architecture-lint
just refactor-readiness
```

Checks:

- declared Cargo binaries exist;
- `causlane-core` has no runtime dependencies;
- files above the line budget are reported;
- public glob re-exports are reported;
- JSON schemas have no duplicate keys.

Initial acceptance:

```bash
python3 tools/architecture-lint --json | jq -e '.summary.errors == 0'
```

Warnings are allowed initially. `--strict` becomes blocking after R2.

### WP-002: CLI boundary split

Create a `causlane-cli` internal application layer:

```text
crates/causlane-cli/src/app/
  mod.rs
  bundle.rs
  replay.rs
  scenario.rs
  formal.rs
  contract.rs
  support.rs
  graph.rs
```

Move business logic from `main.rs` and helper binaries into reusable app
services. Keep argv parsing, environment/file adapters, exit-code mapping and
printing at the CLI boundary.

Acceptance:

- `main.rs` becomes a thin boundary;
- helper binaries share app services;
- command parsing tests still pass;
- no duplicate formal generation logic is introduced.

### WP-003: Formal orchestration service

Introduce:

```text
causlane_cli::app::formal::FormalOrchestrator
```

Responsibilities:

- build Formal IR;
- generate all targets;
- write codegen receipts;
- check staleness;
- read tool-run receipts;
- derive coverage reports;
- provide `verify-all` and `coverage` entrypoints.

Acceptance:

- `causlane-formal` calls the service directly;
- `scripts/check-verification-full.sh` keeps one stable binary boundary;
- coverage JSON remains derived from receipts and exit codes.

### WP-004: Public API narrowing

Replace broad `pub use module::*` with explicit layers:

```text
causlane_core::prelude
causlane_core::protocol
causlane_core::kernel
causlane_core::ports
causlane_core::testing
```

Migration rule:

- keep compatibility re-exports until a deliberate pre-0.1 compatibility window;
- new code imports from explicit modules.

Acceptance:

- `architecture-lint --strict` no longer reports public glob exports in core;
- docs list the stable public API surface;
- examples compile using explicit imports.

### WP-005: Module decomposition by authority

Split large modules by reason:

- DTO/schema parsing;
- pure invariant logic;
- replay lowering;
- diagnostics/explain;
- tests/fixtures.

Priority targets:

```text
causlane-codegen/src/alloy.rs
causlane-codegen/src/coverage.rs
causlane-codegen/src/ir.rs
causlane-replay/src/trace.rs
causlane-replay/src/lib.rs
causlane-runtime/src/in_process/mod.rs
causlane-runtime/src/adapters/otel.rs
causlane-cli/src/main.rs
causlane-cli/src/bin/causlane-formal.rs
```

Acceptance:

- no non-test source file exceeds 800 lines unless exception-recorded;
- each split has module-level docs explaining authority boundary;
- no semantic behavior change occurs without scenario/replay/formal evidence.

### WP-006: Contract/schema single source

Strengthen the generated truth chain:

- schemas are generated or checked from typed DTOs;
- scenario YAML validates before trace generation;
- Formal IR validates before target generation;
- receipt schema validation is included in formal gates.

Acceptance:

```bash
tools/schema-validate-all
just contract-test
```

### WP-007: Runtime/adapter certification separation

Move adapter guarantees into explicit certification matrices.

Acceptance:

- every adapter declares supported guarantees;
- certification tests use the same capability/barrier validators as replay;
- unsupported adapters cannot claim hard-effect safety.

### WP-008: Refactor-safe test gates

Before each refactor PR:

```bash
just architecture-lint
just refactor-readiness
just check
just clippy
just test
just contract-test
just formal-ready
```

Before merging authority-changing refactors:

```bash
just verification-full
just formal-coverage-matrix-check
```

## Whole-track Acceptance

- [ ] zero architecture-lint errors;
- [ ] no missing declared binaries;
- [ ] core has no runtime dependencies;
- [ ] CLI helper binaries share app services;
- [ ] generated formal artifacts remain bundle-bound;
- [ ] public API is documented and explicit;
- [ ] line-budget exceptions are explicit and temporary;
- [ ] replay/contract/formal gates still pass;
- [ ] product track is updated with actual status.
