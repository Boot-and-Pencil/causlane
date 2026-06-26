# ADR-0033: Publish All Crates As A Pre-Alpha Package Family

## Status

Accepted.

## Context

Initial publication is needed partly for internal deployment convenience. The
workspace already contains multiple crates with internal dependencies.

## Decision

Publish the full crate family as `0.0.1`, in dependency order:

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

All README files and release notes must state experimental/pre-alpha status.

## Consequences

- Internal deployment can use crates.io dependencies.
- Public API surface becomes visible early.
- Publication order and package hygiene become strict gates.

