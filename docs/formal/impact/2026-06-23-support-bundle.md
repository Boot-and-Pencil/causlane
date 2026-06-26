# Formal Impact Record: sanitized support bundle (M07.6)

## Change metadata

- Change ID: FIR-2026-06-23-support-bundle
- PR/issue: S07 / M07.6 support bundle
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (developer/operator diagnostic surface) - sanitized artifact

## Touched protocol-critical paths

```text
crates/causlane-cli/src/cli_graph_export.rs
crates/causlane-cli/src/cli_support_bundle.rs
crates/causlane-cli/src/cli_parse.rs
crates/causlane-cli/src/main.rs
docs/specs/support-bundle-v1.md
docs/product-track/milestones/m07.6-support-bundle.md
```

## Summary

M07.6 adds `causlane support-bundle build`, a CLI command that writes a sanitized
JSON support artifact. The command composes existing surfaces: compiled-bundle
metadata, `ReplayTrace::verify_explain`, the M07.2 graph export model, the formal
doctor report, and M07.5 support-bundle redaction classes.

The support bundle does not embed raw trace documents. Subject/circumstance
binding values, raw authorization payloads, execution-capability payloads and
keyed attestations are omitted or represented by redacted summaries.

## Affected invariants

```text
I-003: unchanged - projection truth anchors are still validated by replay.
I-008: unchanged - lifecycle authority remains the audit event stream.
ADR-0014 / TD-017: unchanged - support bundles are derived diagnostics, not
       observed truth or enforcement input.
new invariant ids: none
```

## Affected formal models

```text
none - no bundle, Formal IR, replay trace, generated model or coverage schema
changes. The support bundle is a derived CLI artifact.
```

## Affected protocols

```text
PR-observability-derived: support bundle output is derived from existing
diagnostic projections. No dispatch, barrier, lease, authz, lifecycle or replay
protocol semantics change.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public CLI added: `causlane support-bundle build --bundle ... --trace ...
  --graph ... --out ... [--op ...]`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| raw trace binding values | CLI unit | subject/circumstance values absent from output | new |
| raw authz/capability payloads | CLI unit | payload objects absent; only flags/counts remain | new |
| support-bundle redaction | CLI unit | secret/restricted support-bundle fields report redacted | new |
| focused graph context | CLI smoke | command reuses graph export focus behavior | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (CLI) | support bundle tests | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Replay and generated formal lanes consume traces, bundles and formal artifacts;
M07.6 does not change those schemas. The support bundle is a derived diagnostic
artifact, so CLI tests are the applicable executable lane.

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
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m07.6-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: M07.7 cookbook docs should include a support-bundle recipe.
