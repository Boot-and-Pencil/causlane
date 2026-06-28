# causlane

`causlane` is an early-stage Rust-first project skeleton for typed, auditable,
replayable, consequence-aware action dispatch.

The project is intentionally **docs-first** and **formal-model-first**. Runtime code starts as a small reference kernel, not as a production workflow engine.

## Working positioning

```text
Typed action semantics + consequence-aware dispatch for Rust systems.
```

Or, more explicitly:

```text
A portable semantic dispatch kernel for typed, auditable, replayable, consequence-aware actions.
```

## Non-goals

`causlane` is not intended to be:

- a workflow engine;
- a job queue;
- a distributed scheduler;
- a policy language;
- an event store;
- an observability platform;
- a replacement for Temporal, Restate, Dapr, Conductor, Apalis, Fang or Tokio.

It is intended to provide a small semantic/control layer that can integrate with such systems through adapters.

For external host projects, the stabilized integration seam is `causlane.host-dispatch.v1`; see [`docs/specs/host-dispatch-api-v1.md`](docs/specs/host-dispatch-api-v1.md) and `causlane_runtime::LinearHostDispatcher` for the linear reference implementation.

## Repository layout

```text
docs/                     Normative docs, ADRs, concepts, scenarios.
formal/                   Generated/bound Alloy, P, Kani and optional Verus artifacts.
contracts/                Registry and compiled bundle example shapes.
crates/causlane/          Public facade crate.
crates/causlane-core/     Pure domain/application kernel.
crates/causlane-contracts Registry/bundle contract types.
crates/causlane-runtime/  Runtime adapters and orchestration skeleton.
crates/causlane-replay/   Replay oracle: bundle-bound trace verifier.
crates/causlane-codegen/  Formal artifact generators.
crates/causlane-formal/   Pure formal-toolchain readiness logic.
crates/causlane-cli/      CLI for bundle/scenario/replay/formal commands.
examples/                 Runnable and planned end-to-end examples.
```

Maintainer-only context-pack hygiene is documented in
[`docs/context-pack-hygiene.md`](docs/context-pack-hygiene.md).

## First development principle

Do not start from adapters. Start from the contract:

```text
Docs define the contract.
Formal models attack the contract.
Replay executes the contract.
Rust kernel enforces the contract.
Adapters spend the contract.
Audit records the truth.
Projections explain the truth.
```

## Publication status

The first public crates.io release is an experimental `0.0.1` pre-alpha
bootstrap. All eight runbook crates have been published and indexed:
`causlane-core`, `causlane-formal`, `causlane-contracts`, `causlane-runtime`,
`causlane-replay`, `causlane-codegen`, `causlane` and `causlane-cli`.

Signed tag `v0.0.1` and the GitHub pre-release are public:
<https://github.com/Boot-and-Pencil/causlane/releases/tag/v0.0.1>. Release
evidence is recorded under `docs/release/`.

The library facade crate is `causlane`; the command-line binary is shipped by
the `causlane-cli` package. Install the CLI with:

```bash
cargo install causlane-cli
```

Do not expect `cargo install causlane` to install a binary: `causlane` is the
library facade crate.

## Checks

Portable Rust checks use the standard Cargo toolchain:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

Maintainers working in this repository may use checked-in wrapper scripts and
`just` recipes for the same gates.

## Project Name

The working crate name is `causlane` — a contraction of causal ordering and
execution lanes. Before uploading a crate version, run:

```bash
cargo search causlane
```

The current deterministic repository readiness report is
[`docs/release/publish-readiness.md`](docs/release/publish-readiness.md).
It does not reserve names or upload crates. Maintainers can regenerate it with
the checked-in `tools/publish-readiness` script.
