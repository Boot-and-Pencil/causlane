# ADR-0034: Public API Review Before crates.io Publication

## Status

Accepted.

## Context

The current facade and core crates expose broad re-exports that were useful
during rapid development but may accidentally freeze internal vocabulary as
public API.

## Decision

Run a public API review before publication.

Prefer explicit API layers:

```text
causlane_core::protocol
causlane_core::kernel
causlane_core::ports
causlane_core::prelude
causlane_core::testing
```

Avoid broad public re-exports in public-facing crates unless explicitly accepted
for a pre-alpha compatibility window.

## Consequences

- docs.rs surface becomes more navigable.
- Downstream users get clearer imports.
- Internal milestone vocabulary is less likely to leak.

