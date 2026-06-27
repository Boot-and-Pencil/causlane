# CLI Checker Rust Crate Graph Policy Override TZ

**Status:** checker upstream assignment.
**Observed in:** `cli-checker 0.1.19`, git commit `f17594c`.
**Consumer:** Causlane release hygiene, M11.5.

## Problem

`RUST-ANTI-GRAPH-001` is useful for Causlane, but the checker currently appears
to use its embedded legacy Rust crate graph policy for repo scans instead of a
repo-local `sadave.rust_crate_graph_policy.v1` policy.

In `/workspace/repo`, adding a valid policy path to `.cli-checker.toml`:

```toml
policy_file_paths = [
  ".devinfra/policy/rust-crate-graph.yaml",
]
```

and placing a Causlane policy at that path still leaves
`cli-checker check-repo --format json` reporting every workspace crate as
missing from the imported crate graph policy.

Moving the same document under `.tsqoba/sadave/policy_packs/` adds an unrelated
`REPO-HEALTH-POLICY-PACKS-001` blocker for the missing full Sadave capabilities
file and still does not make `RUST-ANTI-GRAPH-001` consume the repo policy.

## Required behavior

- `RUST-ANTI-GRAPH-001` must load the first valid
  `sadave.rust_crate_graph_policy.v1` document declared through
  `policy_file_paths`.
- The rule must fall back to the embedded legacy policy only when no repo-local
  Rust crate graph policy is configured.
- A single config-supplied Rust crate graph policy must not require adopting the
  full `.tsqoba/sadave/check_capabilities.v1.yaml` policy-pack surface.
- Invalid graph policy documents must fail closed with findings on the policy
  file, not as "workspace crate missing from imported policy" findings on every
  `Cargo.toml`.

## Acceptance fixture

Use a temporary Rust workspace with:

```text
crates/core/Cargo.toml
crates/contracts/Cargo.toml  depends on core
crates/app/Cargo.toml        depends on contracts
```

and a `.cli-checker.toml` that points `policy_file_paths` at a repo-local
`rust-crate-graph.yaml` assigning:

```text
core -> kernel
contracts -> contracts, may depend on kernel
app -> app, may depend on contracts
```

Expected results:

```text
cli-checker validate-config --config .cli-checker.toml --format json
  exits 0 and reports policy_file_path_count = 1

cli-checker check-repo --config .cli-checker.toml --format json
  exits 0 for the valid graph

cli-checker check-repo --config .cli-checker.toml --format json
  exits nonzero when app depends directly on core, with a forbidden edge finding
```

Add a negative test where the configured policy omits `contracts`; the checker
must point at the policy/configured graph problem deterministically.

## Causlane target policy

After the checker supports this, Causlane should add exactly one repo-local graph
policy with these layers:

```text
causlane-core      -> kernel
causlane-formal    -> formal
causlane-contracts -> contracts, may depend on kernel
causlane-runtime   -> runtime, may depend on kernel
causlane-replay    -> replay, may depend on kernel and contracts
causlane-codegen   -> codegen, may depend on contracts
causlane           -> facade, may depend on kernel, contracts, runtime, replay
causlane-cli       -> cli, may depend on kernel, formal, contracts, replay, codegen
```

`causlane-runtime` must be allowed to use the dependency classes required by its
declared optional adapters (`database`, `fs` under the current checker
dependency-class policy).
