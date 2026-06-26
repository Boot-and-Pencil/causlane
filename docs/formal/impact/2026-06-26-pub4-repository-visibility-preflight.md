# Formal Impact Record: PUB4 repository visibility preflight

## Change metadata

- Change ID: FIR-2026-06-26-pub4-repository-visibility-preflight
- PR/issue: PUB4 GitHub baseline and history curation follow-up
- Owner: repo maintainers
- Date: 2026-06-26
- Impact class: F1 (release tooling/reporting only)

## Touched protocol-critical paths

```text
tools/publish-readiness
contracts/schema/publish_readiness.schema.json
docs/release/publish-readiness.json
docs/release/publish-readiness.md
docs/release/refactor-before-publication-gate.md
```

## Summary

Adds an explicit repository-visibility preflight to the existing
`publish-readiness` authority chain. The deterministic readiness report now
records the repository URL and the advisory command that checks public
resolution. `tools/publish-readiness --online` performs the unauthenticated
network probe together with the crates.io name probe.

The current online evidence is:

| Probe | Result | Status |
|---|---|---|
| crates.io `causlane` name | HTTP 404 | `available` |
| `https://github.com/Boot-and-Pencil/causlane` unauthenticated HTTP GET | HTTP 404 | `private_or_missing` |

This does not turn network availability into a deterministic local gate. It
records that PUB4 remains incomplete for public opening or PUB5 upload until the
curated baseline is publicly resolvable and externally scanned.

## Affected invariants

No dispatch, replay, lifecycle, constraint, authz or formal invariant semantics
change.

## Affected formal models

None. No generated Formal IR, Alloy, P, Kani, Verus, Lean, receipt or coverage
artifact changes.

## Contract changes

- Publish-readiness report schema: version `5` to `6`, adding
  `repository_visibility`.
- Bundle / replay trace / scenario / formal receipt schemas: none.
- Rust public API: none.
- Production dependencies: none.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| deterministic readiness regeneration | publish-readiness | generated JSON/Markdown match current metadata | updated |
| unauthenticated private repository probe | publish-readiness online | non-2xx GitHub response is not classified as public | observed |

## Acceptance commands

```bash
tools/publish-readiness --write
tools/publish-readiness --check
tools/publish-readiness --online
tools/schema-validate-all
python3 tools/publication-plan-doc-lint --json
python3 tools/pre-publication-review-gate --json
python3 tools/architecture-lint --json
tools/context-pack-scan
./tools/cargo-dev check --workspace --all-targets --all-features --locked
./tools/cargo-dev test --workspace --doc --locked
git diff --check
```

## Exception request

- Exception needed? no
- Follow-up issue: make the curated public baseline publicly resolvable, then
  run the required external secret scan before PUB5 upload.
