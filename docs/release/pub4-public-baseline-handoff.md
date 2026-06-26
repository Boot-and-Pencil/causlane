# PUB4 Public Baseline Handoff

**Status:** required before public GitHub opening or PUB5 upload.

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
baseline_ref:
baseline_commit:
operator:
date:
host:
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
scanner_version:
command: gitleaks git --redact --report-format json --report-path target/causlane/gitleaks-public-baseline.json .
exit_code:
report_path: target/causlane/gitleaks-public-baseline.json
scanned_commit:
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
baseline_commit:
gitleaks_version:
gitleaks_exit_code:
gitleaks_report_path:
context_pack_scan_exit_code:
publish_readiness_online_repository_visibility:
branch_protection_confirmed_by:
release_owner:
```

Do not proceed to PUB5 while any field is missing or while
`publish_readiness_online_repository_visibility` is not `public`.
