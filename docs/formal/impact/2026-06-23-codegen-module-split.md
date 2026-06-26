# Formal Impact Record: codegen module split debt pass

## Change metadata

- Change ID: FIR-2026-06-23-codegen-module-split
- PR/issue: technical debt pass after codebase review
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F1 (behavior-preserving codegen refactor)

## Touched protocol-critical paths

```text
crates/causlane-codegen/src/lib.rs
crates/causlane-codegen/src/artifact.rs
crates/causlane-codegen/src/error.rs
crates/causlane-codegen/src/receipt.rs
crates/causlane-codegen/src/stale.rs
crates/causlane-codegen/src/p_monitors.rs
crates/causlane-codegen/src/targets.rs
```

## Summary

This change reduces `causlane-codegen` technical debt by splitting the crate root
into cohesive modules for artifact metadata, receipt schema, stale-checking, and
errors while preserving the existing crate-level public API through re-exports.
The long generated P monitor block moves out of `targets.rs` into a single
internal text constant plus a short emitter, removing the local
`too_many_lines` allowance without duplicating monitor text.

## Affected invariants

```text
none - no dispatch, replay, lifecycle, authz, projection, receipt or coverage
semantics change.
new invariant ids: none
```

## Affected formal models

```text
none - generated artifact text is intended to remain unchanged; no Formal IR,
scenario, trace, bundle or coverage schema changes.
```

## Affected protocols

```text
none - this is a code organization refactor inside the generator crate.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public CLI added/removed/changed: none.
- Public Rust API added/removed/changed: none; existing crate-root exports are preserved.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| stale artifact edit | unit | `stale_check` still rejects edited generated text | existing |
| stale scenario mismatch | unit | `stale_check_with_expected` still rejects wrong scenario hash | existing |
| P monitor grounding | unit | generated P monitor still carries witness/anchor/authz checks | existing |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (codegen) | stale-check and P monitor tests | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Formal model lanes are unchanged because this refactor only moves code and shared
generated text ownership inside `causlane-codegen`.

## Acceptance commands

```bash
just check
just clippy
just test
just coverage
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-codegen-debt-changed-files.txt
./tools/formal-exceptions-check
./tools/schema-validate-all
./tools/product-track-status-check
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
