# ADR-0023: Public API layering and prelude policy

## Status

Accepted.

## Context

The current `causlane-core` API re-exports many domain modules through broad
glob exports. This made early development quick, but it exposes internal
vocabulary as public API and makes semantic authority boundaries harder to
audit.

Patch pack 014's architecture lint confirms the current baseline: zero hard
repository-shape errors and 32 public glob re-export warnings. Those warnings
are accepted in R0 and tracked for R2.

## Decision

Move toward explicit public layers:

```text
causlane_core::protocol
causlane_core::kernel
causlane_core::ports
causlane_core::prelude
causlane_core::testing
```

Broad `pub use module::*` remains temporarily compatible during R0/R1, but new
code should prefer explicit imports. R2 will replace broad public glob exports
with explicit layers and migration docs.

## Consequences

- Downstream imports become more explicit.
- Rustdoc becomes more navigable.
- Generated formal code can target stable modules.
- Accidental public API freeze is reduced before the first compatibility window.
