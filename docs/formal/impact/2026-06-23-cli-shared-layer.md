# Formal Impact Record: CLI shared layer debt pass

## Change metadata

- Change ID: FIR-2026-06-23-cli-shared-layer
- PR/issue: technical debt pass after branch consolidation
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F1 (behavior-preserving CLI refactor)

## Touched protocol-critical paths

```text
crates/causlane-cli/src/lib.rs
crates/causlane-cli/src/cli_shared.rs
crates/causlane-cli/src/formal_artifacts.rs
crates/causlane-cli/src/main.rs
crates/causlane-cli/src/cli_parse.rs
crates/causlane-cli/src/formal_generate.rs
crates/causlane-cli/src/bin/causlane-formal.rs
```

## Summary

This change reduces CLI technical debt after the S07 branch consolidation. It
extracts pure shared helpers and formal artifact planning into the
`causlane-cli` library so the `causlane` and `causlane-formal` binaries use one
target list, one path layout and one scenario-stem rule.

Direct filesystem and clock access remains in the binaries as platform-boundary
adapters. The shared library exposes testable ports and pure planning logic; it
does not add new command behavior.

## Affected invariants

```text
none - no dispatch, replay, lifecycle, authz, projection, receipt or coverage
semantics change.
new invariant ids: none
```

## Affected formal models

```text
none - no Formal IR, generated model, scenario, trace, bundle or coverage schema
changes.
```

## Affected protocols

```text
none - this is a CLI/module-structure refactor. Formal artifacts and receipts
continue to be generated from the same CodegenContracts and generator APIs.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public CLI added/removed/changed: none.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| shared formal path layout | CLI unit | artifact and receipt paths match existing layout | new |
| shared argv helpers | CLI unit | flag parsing remains position-independent | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (CLI) | shared helper and path-layout tests | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Formal model lanes are unchanged because this refactor only removes duplicated
CLI glue and path planning. Existing coverage and stale-check gates remain the
applicable executable controls.

## Acceptance commands

```bash
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just coverage
./tools/schema-validate-all
./tools/product-track-status-check
./tools/formal-exceptions-check
./tools/coverage-matrix --check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-cli-debt-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
