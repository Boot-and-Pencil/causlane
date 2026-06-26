# Formal Impact Record: redaction policy classes (M07.5)

## Change metadata

- Change ID: FIR-2026-06-23-redaction-policy-classes
- PR/issue: S07 / M07.5 redaction policy
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (protocol-adjacent core surface) — redaction policy compiler

## Touched protocol-critical paths

```text
crates/causlane-core/src/domain/redaction.rs
crates/causlane-core/src/domain/redaction_policy.rs
crates/causlane-core/src/domain/mod.rs
docs/07-security-and-authz.md
docs/product-track/milestones/m07.5-redaction-policy.md
```

## Summary

M07.5 adds a typed redaction-class/profile layer in `causlane-core`. Hosts can
classify field paths as `Public`, `Operational`, `Restricted`, or `Secret` for
audit, log, projection, replay, and support-bundle surfaces. The profile compiler
reduces that typed contract to the existing M06.7 `RedactionPolicy` allowlist;
`apply_redaction` remains the only masking mechanism.

The default class policy reveals no classes. `public_only()` reveals only fields
explicitly classified as `Public`. Duplicate classifications for the same path
are fail-closed: the compiled policy reveals the path only if every declaration's
class is revealable.

## Affected invariants

```text
I-003: unchanged — projection truth anchors are still validated from audit events.
I-008: unchanged — lifecycle authority remains the audit event stream.
ADR-0011/ADR-0014: unchanged — authz remains deny-by-default and observability
       outputs are derived, not authority.
new invariant ids: none
```

## Affected formal models

```text
none — no bundle, Formal IR, replay trace, generated model or coverage schema
changes. The new class compiler is a pure Rust contract over existing field-path
redaction.
```

## Affected protocols

```text
PR-redaction-fail-closed: class profiles compile into the existing fail-closed
RedactionPolicy allowlist. No dispatch, barrier, lease, authz, lifecycle or
replay protocol semantics change.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public Rust API added: `RedactionSurface`, `RedactionClass`,
  `ClassifiedField`, `RedactionClassPolicy`, `SurfaceRedactionProfile`,
  `compile_redaction_policy`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| empty class policy | core unit | compiles to empty `RedactionPolicy` | new |
| public-only profile | core unit | only public fields reveal; operational/restricted/secret/unclassified redact | new |
| every surface | core unit | all surfaces compile through the same mechanism | new |
| duplicate weaker classification | core unit | mixed duplicate path redacts under public-only policy | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (core) | redaction policy class tests | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Replay and generated formal lanes consume traces, bundles, and formal artifacts;
M07.5 does not change those schemas. The behavior is a pure class-to-allowlist
compiler over the existing redaction mechanism, so core unit tests are the
applicable executable lane.

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
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m07.5-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: M07.6 support bundle should consume the shared class/profile
  layer instead of defining a separate sanitizer policy.
