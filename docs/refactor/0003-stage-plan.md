# Stage plan for the quality/architecture refactor

## R0 — Patch-pack bootstrap

Deliverables:

- architecture lint tool;
- report schema;
- justfile readiness gates;
- refactor ADRs;
- current-state review and master TZ.

Exit:

```bash
just refactor-readiness
python3 tools/architecture-lint --json | jq -e '.summary.errors == 0'
```

## R1 — CLI and formal orchestration split

Deliverables:

- `causlane_cli::app::*` modules;
- `FormalOrchestrator` service;
- `causlane-formal` and `causlane` share service code;
- coverage derivation remains a library call over receipts.

Exit:

```bash
./tools/cargo-dev check -p causlane-cli --all-targets --all-features --locked
```

## R2 — Public API narrowing

Deliverables:

- explicit `prelude`, `protocol`, `kernel`, `ports`, `testing` modules;
- migration docs;
- no glob re-export warnings in strict lint.

Exit:

```bash
python3 tools/architecture-lint --strict --json | jq -e '.status == "pass"'
```

## R3 — Module decomposition

Deliverables:

- split large codegen/replay/runtime/CLI modules;
- move tests into focused integration or module tests where appropriate;
- module docs identify authority boundaries.

Exit:

```bash
python3 tools/architecture-lint --max-lines 800 --json | jq -e '.summary.warnings == 0'
```

or documented temporary exceptions.

## R4 — Contract/schema hardening

Deliverables:

- schema validation on all DTO outputs;
- DTO/schema sync tests where a schema exists;
- scenario compile/replay/generation stays single-path.

Exit:

```bash
tools/schema-validate-all
just contract-test
```

## R5 — Adapter certification hardening

Deliverables:

- adapter guarantee matrix;
- certification harness covers barrier/capability/audit behavior;
- unsupported hard-effect claims fail closed.

Exit:

```bash
./tools/cargo-dev test -p causlane-runtime --all-targets --all-features --locked
```

## R6 — Formal/replay release gate

Deliverables:

- full formal gate passes;
- coverage matrix derives from receipts;
- all target artifacts stale-check;
- exceptions are expired or renewed with explicit rationale.

Exit:

```bash
just verification-full
just formal-coverage-matrix-check
```
