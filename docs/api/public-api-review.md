# Public API Review Before crates.io

## Goal

Make the public crate surface reusable and understandable before first
publication. Pre-alpha status does not remove API discipline: `0.0.1` still
creates docs.rs pages, package metadata, examples and downstream imports.

## Review Status

M11.3/PUB2 review is recorded for the `0.0.1` pre-alpha bootstrap. This is not
a semver freeze: APIs may still change before `0.1`, but the current public
surface has been reviewed for accidental broad facade exports, default feature
pull-ins, misleading formal/replay claims and basic docs.rs usability.

## M11.3 Checkpoint

This pass narrows the `causlane` facade to `prelude` plus curated
`core::{protocol, kernel, ports, prelude}` layers. `causlane::core::testing` is
intentionally not exposed through the facade; downstream users that need testing
helpers must depend on `causlane-core` and import `causlane_core::testing`
directly. `causlane-codegen` README entries now match the actual public Alloy
generators.

The direct `causlane-core` crate still publishes lower-level modules such as
`domain`, `contract` and `integration` for pre-alpha consumers and internal
specs. That broader low-level surface is accepted for `0.0.1`; a before-`0.1`
API narrowing pass must decide which of those paths become stable entry points.

## Intended API Layers

```text
causlane_core::protocol
causlane_core::kernel
causlane_core::ports
causlane_core::prelude
causlane_core::testing
```

The `causlane` facade should stay small:

```rust
pub mod core {
    pub use causlane_core::{kernel, ports, prelude, protocol};
}

pub mod prelude {
    pub use causlane_core::prelude::*;
}
```

Any broader public re-export needs an ADR or an explicit pre-alpha exception.
The YAML parser is not exposed as a public Rust error type; YAML remains part of
the registry/scenario document boundary through string-taking parse APIs and
schema-checked files.

## Crate-by-crate Checklist

### `causlane-core`

- [x] no async runtime, database, HTTP, workflow-engine, telemetry or policy-engine dependency;
- [x] public entry points document protocol/kernel/ports/testing boundaries;
- [x] broader low-level modules are accepted as pre-alpha surface, not facade exports;
- [x] no milestone or patch-pack vocabulary in production identifiers;
- [x] protocol-critical types have semantic names and docs;
- [x] `unsafe_code = forbid` remains effective.

### `causlane-contracts`

- [x] canonical serialization and hash contracts are documented;
- [x] JSON/YAML DTO shapes remain schema-bound document inputs;
- [x] generated truth chain remains bundle/scenario-bound;
- [x] public constructors validate protocol-critical hashes/newtypes.

### `causlane-replay`

- [x] replay diagnostics are usable as a library API, not only via CLI;
- [x] error codes are stable enough for pre-alpha consumers;
- [x] bundle-bound verification is the documented full replay path;
- [x] structural-only replay is clearly marked as weaker.

### `causlane-codegen`

- [x] generated artifact targets are explicit;
- [x] codegen library APIs return generated text/receipts and leave filesystem writes to callers;
- [x] generated artifacts carry source bundle/scenario hashes where the target is scenario-bound;
- [x] no formal target claims proof stronger than its receipts.

### `causlane-runtime`

- [x] optional adapters are behind feature flags;
- [x] default features are minimal;
- [x] adapter guarantees are narrow and documented;
- [x] in-process runtime retention is bounded and explicit in config/docs;
- [x] hard-effect safety remains delegated to barrier/capability validators.

### `causlane-formal`

- [x] API stays pure and does not download or execute external tools;
- [x] filesystem probing and process execution stay at CLI/tool boundary;
- [x] doctor reports distinguish missing optional tools from blocking tools.

### `causlane-cli`

- [x] every declared binary has a source file;
- [x] package README states pre-alpha status, and help labels structural-only replay as weaker;
- [x] CLI application services are shared rather than duplicated;
- [x] CLI does not become the semantic authority for formal/replay status.

### `causlane`

- [x] facade exports only intentional `core` and `prelude` surface;
- [x] README gives a minimal import and non-goals;
- [x] dev-only benches/tests use workspace crates and do not widen the packaged facade dependency surface.

## Documentation Acceptance

- [x] `cargo doc --workspace --no-deps --locked` succeeds in this checkout.
- [x] Crate READMEs state experimental/pre-alpha status.
- [x] Public docs do not promise production workflow engine, scheduler or formal proof completeness.
- [x] Current compiled doc examples pass; runnable example expansion remains M11.4 work.

## API Change Policy Before `0.1`

Before `0.1`, API changes are allowed, but every release note must state:

- changed public modules/types/functions;
- migration notes if a published symbol was renamed or removed;
- whether the change narrows accidental API or changes intended API.
