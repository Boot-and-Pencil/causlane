# Publish Sequence State Machine

This document defines the safe state transitions for first full-workspace
publication.

## Current Recorded State

As of 2026-06-27, `causlane-core 0.0.1` and `causlane-formal 0.0.1` are
`Published` and `Indexed` on crates.io. Continue only through the staged order
below; do not skip ahead to dependent crates. The next runbook crate is
`causlane-contracts`. The M11.5 YAML parser dependency decision is resolved by
the `noyalib 0.0.8` migration, and the `causlane-contracts` dry-run passed on
2026-06-27. The next valid transition is `Published(causlane-contracts)` if
maintainers choose to continue after CI.

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
