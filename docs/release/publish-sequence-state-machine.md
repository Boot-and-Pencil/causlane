# Publish Sequence State Machine

This document defines the safe state transitions for first full-workspace
publication.

## Current Recorded State

As of 2026-06-28, `causlane-core 0.0.1`, `causlane-formal 0.0.1`,
`causlane-contracts 0.0.1`, `causlane-runtime 0.0.1` and
`causlane-replay 0.0.1` are `Published` and `Indexed` on crates.io. The staged
dry-run for `causlane-codegen 0.0.1` has passed. Continue only through the
staged order below; do not skip ahead to dependent crates. The next irreversible
runbook action is publishing `causlane-codegen` after explicit maintainer
confirmation.

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
  -> ...
  -> WorkspacePublished(0.0.1)
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
