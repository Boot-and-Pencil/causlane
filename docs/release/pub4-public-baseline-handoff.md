# PUB4 Public Baseline Handoff

**Status:** complete for public GitHub opening; required evidence remains part
of the PUB5 preflight.

This is the hand-maintained handoff for the curated public GitHub baseline. It
records the release evidence that cannot be derived from repository-local
metadata alone. The generated readiness report remains
`docs/release/publish-readiness.md`; do not copy this document into generated
readiness output.

## Baseline Authority

The public baseline must be one exact commit, tag or branch tip selected by
repo maintainers after PUB0-PUB3 have passed.

Record the baseline before scanning:

```text
baseline_ref: refs/heads/main
baseline_commit: e2c376803a578dbe1db688b2db194f657f37e812
operator: Vitalii Lobanov / vitalii-lobanov
date: 2026-06-26
host: dispatcher
```

The baseline is not ready for public opening if `git status --short` is non-empty
or if the repository URL in Cargo metadata does not resolve publicly through
`tools/publish-readiness --online`.

## Required Secret Scan

The required PUB4 secret scanner is **Gitleaks**. Scan the exact public baseline
history and keep the report outside the repository:

```bash
gitleaks version
gitleaks git --redact --report-format json --report-path target/causlane/gitleaks-public-baseline.json .
```

Required evidence:

```text
scanner: gitleaks
scanner_version: version is set by build process
command: gitleaks git --log-opts=e2c376803a578dbe1db688b2db194f657f37e812 --redact --report-format json --report-path target/causlane/gitleaks-clean-main-exact.json .
exit_code: 0
report_path: target/causlane/gitleaks-clean-main-exact.json
scanned_commit: e2c376803a578dbe1db688b2db194f657f37e812
```

Pass condition: exit code `0` and no unreviewed findings. Any real secret blocks
public opening and PUB5 upload until the credential is rotated, removed from the
public baseline, and the exact baseline is rescanned.

TruffleHog may be run as defense-in-depth, but it is not a replacement for the
required Gitleaks evidence.

## Context-Pack Scan

Run the repository-local context hygiene scanner on the same baseline:

```bash
tools/context-pack-scan
```

When sharing a generated context pack or repomix output, scan that generated file
or directory explicitly with `tools/context-pack-scan <path>`.

## Repository Controls

Before inviting external contributors:

- `main` requires pull requests;
- `main` blocks force pushes and branch deletion;
- the CI workflow must pass before merge;
- publication-related changes run the publication gate or an equivalent manual
  maintainer check;
- release ownership stays with human repo maintainers; AI tools are not authors,
  reviewers, signers or release owners.

## PUB4 Completion Record

PUB4 is complete only when the release branch or release issue records:

```text
baseline_commit: e2c376803a578dbe1db688b2db194f657f37e812
gitleaks_version: version is set by build process
gitleaks_exit_code: 0
gitleaks_report_path: target/causlane/gitleaks-clean-main-exact.json
context_pack_scan_exit_code: 0
publish_readiness_online_repository_visibility: public
branch_protection_confirmed_by: vitalii-lobanov
release_owner: Vitalii Lobanov
```

Do not proceed to PUB5 while any field is missing or while
`publish_readiness_online_repository_visibility` is not `public`.
