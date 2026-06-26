# 11. Implementation Start Gate

This is the handoff checklist for starting a Causlane milestone branch without
re-reading historical implementation context.

## Authority

Implementation starts from machine-backed state:

```text
contracts and scenarios
  -> replay oracle
  -> generated formal artifacts
  -> receipts and stale checks
  -> coverage matrix
  -> product-track status
```

Prose is guidance, not proof. If prose and a gate disagree, fix the drift before
starting protocol-critical work.

## Minimum Start Gate

Run before opening a milestone branch:

```bash
just refactor-readiness
just product-track-check
just formal-ready
```

For protocol-critical work, also run:

```bash
just formal-verify-all
just contract-test
```

For runtime, adapter or performance work, add the relevant Rust gate:

```bash
just check
just clippy
just test
just bench-dispatch-baseline-build
```

A branch may start with a known failing heavy gate only when the failure is
unrelated to the milestone scope, recorded in the milestone note, assigned an
owner and expiry, and not used to support a release/publication claim.

## Current Focus

The current roadmap focus is the S11/PUB5 staged publication sub-track. Upload
is still not a batch action; each crate must pass its own dry-run and publish
step after internal registry dependencies are available.

```text
recorded complete:
  S11/PUB0 Repository and architecture refactor
  S11/PUB1 Readability and maintainability
  S11/PUB2 Public API review
  S11/PUB3 Human and agent documentation
  S11/PUB4 GitHub baseline and history curation

only after that:
active_next:
  S11/PUB5 staged crates.io publication
```

S08/S09/S10 remain product-roadmap workstreams, but they are not the immediate
publication-preparation action. Do not upload crates outside `PUBLISHING.md` and
`docs/release/publish-all-crates-runbook.md`; the next irreversible command,
if maintainers continue, is publishing `causlane-core`.

## Entry Checklist

- Read the stage file under `docs/product-track/stages/`.
- Read the milestone file under `docs/product-track/milestones/`.
- Confirm dependencies in `docs/product-track/05-dependency-map.md`.
- Check risks in `docs/product-track/06-risk-register.md`.
- Identify protocol-critical files before implementation.
- Add or update a Formal Impact Record for protocol-critical behavior or formal
  gate boundary changes.
- Define positive and negative controls before implementation when applicable.
- Decide whether the milestone changes runtime authority, formal claims or docs
  only.

## Exit Checklist

- Implementation, generated artifact or documented non-code deliverable exists.
- Docs and ADRs match implementation.
- Protocol-critical behavior is not prose-only.
- Relevant positive and negative controls exist.
- Replay, formal or adapter gates are updated when relevant.
- Readiness/status claims are machine-derived or explicitly marked as summary.
- Known gaps have owner and expiry.

## Status Vocabulary

Use these tokens consistently in roadmap files:

| Status | Meaning |
|---|---|
| `planned` | Not started. No implementation claim. |
| `exists` | Initial implementation or document exists, but hardening remains. |
| `exists_harden` | Working slice exists; next work is robustness or coverage. |
| `exists_expand` | Working slice exists; next work is breadth or scenarios. |
| `done_or_near_done` | Exit gate is expected to pass; any residual gap is small and named. |
| `active_next` | Stage is current work focus. |
| `future` | Deliberately out of near-term scope. |

Do not upgrade a status to `done_or_near_done` unless the corresponding gate or
explicit evidence exists.
