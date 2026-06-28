# Publish Sequence State Machine

This document defines the safe state transitions for first full-workspace
publication.

## Current Recorded State

As of 2026-06-29, all `0.0.1` runbook crates are `Published` and `Indexed` on
crates.io, signed tag `v0.0.1` has been pushed to origin, and repository release
notes, downstream smoke evidence and the GitHub pre-release are recorded.
Evidence is recorded in `docs/release/pub5-v0.0.1-tag.md` and
`docs/release/pub6-v0.0.1-post-publication.md`. Public follow-up issues remain
optional/deferred, and the next product-roadmap action is M11.4 Examples.

## States

```text
LocalReady
  deterministic repo-local checks pass, but nothing has been uploaded.

PackageReviewed(crate)
  package file list for crate was inspected.

DryRunPassed(crate)
  cargo publish --dry-run passed for crate after its registry dependencies were available.

Published(crate)
  cargo publish succeeded for crate.

Indexed(crate)
  crate is visible through cargo search/info or downstream dependency resolution.

WorkspacePublished(version)
  all crates in the release sequence are indexed.

Tagged(version)
  the signed release tag was created and pushed.

PUB6ReleaseNotesRecorded(version)
  repository release notes, checksums and downstream smoke evidence are recorded.

GitHubReleasePublished(version)
  the GitHub Release for the signed release tag is public.
```

## Invalid Transitions

```text
LocalReady -> DryRunPassed(dependent crate)
  invalid if an internal dependency is not yet Published+Indexed.

Published(crate) -> overwrite same version
  invalid; publish a new version instead.

Yanked(crate) -> secret removed
  invalid; yanking does not delete uploaded source.
```

## Valid Sequence

```text
LocalReady
  -> PackageReviewed(all crates)
  -> DryRunPassed(causlane-core)
  -> Published(causlane-core)
  -> Indexed(causlane-core)
  -> DryRunPassed(causlane-formal)
  -> Published(causlane-formal)
  -> Indexed(causlane-formal)
  -> DryRunPassed(causlane-contracts)
  -> Published(causlane-contracts)
  -> Indexed(causlane-contracts)
  -> DryRunPassed(causlane-runtime)
  -> Published(causlane-runtime)
  -> Indexed(causlane-runtime)
  -> DryRunPassed(causlane-replay)
  -> Published(causlane-replay)
  -> Indexed(causlane-replay)
  -> DryRunPassed(causlane-codegen)
  -> Published(causlane-codegen)
  -> Indexed(causlane-codegen)
  -> DryRunPassed(causlane)
  -> Published(causlane)
  -> Indexed(causlane)
  -> DryRunPassed(causlane-cli)
  -> Published(causlane-cli)
  -> Indexed(causlane-cli)
  -> WorkspacePublished(0.0.1)
  -> Tagged(v0.0.1)
  -> PUB6ReleaseNotesRecorded(v0.0.1)
  -> GitHubReleasePublished(v0.0.1)
```

## Publication Order

```text
causlane-core
causlane-formal
causlane-contracts
causlane-runtime
causlane-replay
causlane-codegen
causlane
causlane-cli
```

## Rationale

Cargo packages with `path + version` dependencies use local paths for workspace
development, but registry-compatible versions for publication. A dependent crate
must therefore wait until its internal dependency is actually available from the
registry.
