# 0004 - Documentation Readiness Review For Dispatcher 015

**Date:** 2026-06-24
**Snapshot:** `causlane_patch_pack_015_doc_readiness.zip`
**Status:** `partially_superseded` -> `ready_with_guardrails`

## Summary

Patch-pack 015 correctly identified a readiness problem in the dispatcher-015
snapshot: documentation and shell gates referenced formal helper binaries that
were missing from the packed source tree, and milestone handoff guidance was too
diffuse.

The binary-source part of the patch is now superseded by the R1 formal
orchestrator extraction. `causlane-formal`, `causlane-formal-discipline`,
`FormalOrchestrator` and the shared filesystem `FormalIo` adapter already exist
on `main` with stronger behavior than the patch scaffold.

The relevant remaining work is documentation readiness and helper CLI parity:

- add a single milestone start gate;
- add a milestone execution runbook;
- fix stale S05 "next track" prose in formal readiness status;
- expose `generate-all` and `stale-check-all` on the dedicated
  `causlane-formal` helper so the binary surface matches the documented helper
  boundary;
- keep all formal orchestration in `causlane_cli::app::formal`.

## Evidence Checked

Non-mutating checks during review:

```bash
git status --short --branch
unzip -l /tmp/causlane_patch_pack_015_doc_readiness.zip
python3 tools/architecture-lint --json | jq '.summary'
python3 tools/product-track-status-check --json
```

Observed current state before implementation:

```text
git tree: clean
architecture-lint errors: 0
architecture-lint warnings: 32 public-glob/line-budget warnings
product-track-status-check: OK
formal helper binaries: present
causlane-formal-discipline: implemented as local/PR-diff discipline enforcement
```

## Decision

Apply the documentation handoff and the helper CLI parity changes. Do not apply
the patch-pack's `causlane-formal-discipline` scaffold because it would replace
the current stricter implementation with a weaker presence-only checker.

Do not hand-edit `docs/product-track/causlane_product_track_full_ru.md`; the
README identifies atomic product-track files as primary sources, and
`tools/product-track-bundle` is the supported generator/checker for the
concatenated reference bundle.

## Acceptance

The implementation is ready when these commands pass:

```bash
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just contract-test
just refactor-readiness
just formal-ready
just formal-verify-all
just formal-coverage-matrix-check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-patch-pack-015-changed-files.txt
```

## Next Implementation Focus

The roadmap marks S08 and S09 as `active_next`. The practical next closure item
remains M09.7 chaos/recovery tests. S10 formal-depth/proof-hardening should
follow after S08/S09 gates are green or explicitly scoped with a release-profile
proof policy.
