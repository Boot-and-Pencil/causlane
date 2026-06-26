# Formal Impact Record: R0 architecture lint and refactor track

## Change metadata

- Change ID: FIR-2026-06-24-architecture-lint-refactor-track
- PR/issue: Patch pack 014 R0 quality/architecture bootstrap
- Owner: repo maintainers
- Date: 2026-06-24
- Impact class: F0 (tooling/docs gate, no semantic authority change)

## Touched protocol-critical paths

```text
none - R0 adds repository-shape tooling and refactor documentation only.
```

## Summary

R0 adds `tools/architecture-lint`, a JSON report schema, justfile gate wiring and
refactor track documentation. The tool checks declared Cargo binaries, core
runtime dependency boundaries, line-budget warnings, public glob re-export
warnings and duplicate JSON schema keys.

The change does not modify Rust protocol semantics, replay semantics, runtime
admission/execution behavior, generated formal artifacts, schemas or scenarios.

## Affected invariants

```text
ADR-0008: unchanged - observability remains derived and non-authoritative.
ADR-0015: unchanged - formal evidence discipline remains generated/receipt-bound.
ADR-0022: new - staged refactor track preserves semantic authority boundaries.
ADR-0023: new - public API narrowing is staged, not performed in R0.
ADR-0024: new - architecture-lint is a bootstrap repository-shape gate.
I-001: unchanged - execution authority is not modified.
I-006: unchanged - conflict/merge semantics are not modified.
I-008: unchanged - lifecycle/replay semantics are not modified.
new invariant ids: none
```

## Affected formal models

```text
none - no bundle, replay trace, Formal IR, generated model, scenario, receipt
or coverage-matrix schema changes.
```

## Contract changes

- Tooling interface added: `python3 tools/architecture-lint [--json] [--strict]
  [--max-lines N]`.
- Devinfra schema added:
  `.devinfra/state-schema/architecture-lint.schema.json`.
- Justfile gates added: `architecture-lint`, `architecture-lint-json`,
  `architecture-lint-strict` and `refactor-readiness`.
- Existing Rust public APIs are unchanged.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| current tree architecture lint | tooling | zero errors, known warnings allowed | new |
| architecture-lint JSON report shape | tooling | validates against devinfra schema | new |
| strict mode baseline | tooling | fails while public glob warnings remain | not blocking in R0 |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Tooling | architecture-lint report schema validation | no | docs |
| Product docs | ADR-0022..0024 + refactor docs | no | docs |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | proof/all |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Acceptance commands

```bash
python3 tools/architecture-lint --json | tee target/causlane/architecture-lint.json
python3 tools/validate-json-schema --schema .devinfra/state-schema/architecture-lint.schema.json target/causlane/architecture-lint.json
jq -e '.summary.errors == 0' target/causlane/architecture-lint.json
just architecture-lint
just refactor-readiness
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just contract-test
just formal-ready
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-patch-pack-014-changed-files.txt
just formal-coverage-matrix-check
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
