# Publish readiness

This file is generated from
[`publish-readiness.json`](publish-readiness.json) by
`tools/publish-readiness --write`. Do not hand-edit this Markdown;
edit the readiness logic or package metadata and regenerate it.

This artifact is a deterministic publication-prep report. It does not publish crates, reserve names, verify cargo tokens or turn network availability into a blocking repo gate.

## Summary

- Strategy: `publication_prep_no_upload`
- Facade crate: `causlane`
- Readiness status: `pass`
- Version: `0.0.1`
- Manifest: `crates/causlane/Cargo.toml`
- Package root: `crates/causlane`

## Feature Surface

- Status: `pass`
- Policy: `m11_2_default_minimal_optional_integrations`
- Blockers: none

### Feature Packages

#### causlane

- Manifest: `crates/causlane/Cargo.toml`
- Default features: none
- Declared features: none
- Optional dependencies: none
- docs.rs all-features: `false`

#### causlane-core

- Manifest: `crates/causlane-core/Cargo.toml`
- Default features: none
- Declared features: none
- Optional dependencies: none
- docs.rs all-features: `false`

#### causlane-contracts

- Manifest: `crates/causlane-contracts/Cargo.toml`
- Default features: none
- Declared features: none
- Optional dependencies: none
- docs.rs all-features: `false`

#### causlane-replay

- Manifest: `crates/causlane-replay/Cargo.toml`
- Default features: none
- Declared features: none
- Optional dependencies: none
- docs.rs all-features: `false`

#### causlane-runtime

- Manifest: `crates/causlane-runtime/Cargo.toml`
- Default features: none
- Declared features: `apalis`, `default`, `otel`, `postgres-audit`, `restate`, `sqlite-audit`, `tokio-runtime`
- Optional dependencies: `apalis via apalis`, `opentelemetry via otel`, `opentelemetry-otlp via otel`, `opentelemetry_sdk via otel`, `postgres via postgres-audit`, `restate-sdk via restate`, `rusqlite via sqlite-audit`, `serde via restate`, `tokio via tokio-runtime`, `tower via apalis`
- docs.rs all-features: `true`

#### causlane-codegen

- Manifest: `crates/causlane-codegen/Cargo.toml`
- Default features: none
- Declared features: none
- Optional dependencies: none
- docs.rs all-features: `false`

#### causlane-cli

- Manifest: `crates/causlane-cli/Cargo.toml`
- Default features: none
- Declared features: none
- Optional dependencies: none
- docs.rs all-features: `false`

## Crates.io Name Availability

- Deterministic gate: `not_run`
- Advisory command: `tools/publish-readiness --online`
- Note: Crates.io availability is intentionally advisory because it is network-dependent and can change at any time.

## Repository Visibility

- Repository: `https://github.com/Boot-and-Pencil/causlane`
- Deterministic gate: `not_run`
- Advisory command: `tools/publish-readiness --online`
- Note: Public repository resolution is intentionally advisory because it is network-dependent and can change at any time.

## Workspace Publication Order

- Status: `pass`
- Facade dependency closure: `causlane-core`
- Facade publish sequence: `causlane-core`, `causlane`
- Full publish sequence: `causlane-core`, `causlane-contracts`, `causlane-runtime`, `causlane-replay`, `causlane-codegen`, `causlane`, `causlane-cli`
- Cycle packages: none

### Dependency Tiers

- Tier 0: `causlane-core`
- Tier 1: `causlane`, `causlane-contracts`, `causlane-runtime`
- Tier 2: `causlane-replay`, `causlane-codegen`
- Tier 3: `causlane-cli`

### Workspace Packages

#### causlane

- Manifest: `crates/causlane/Cargo.toml`
- Package root: `crates/causlane`
- Normal workspace dependencies: `causlane-core ^0.0.1 via crates/causlane-core`
- Package files: 10

#### causlane-core

- Manifest: `crates/causlane-core/Cargo.toml`
- Package root: `crates/causlane-core`
- Normal workspace dependencies: none
- Package files: 46

#### causlane-contracts

- Manifest: `crates/causlane-contracts/Cargo.toml`
- Package root: `crates/causlane-contracts`
- Normal workspace dependencies: `causlane-core ^0.0.1 via crates/causlane-core`
- Package files: 21

#### causlane-replay

- Manifest: `crates/causlane-replay/Cargo.toml`
- Package root: `crates/causlane-replay`
- Normal workspace dependencies: `causlane-contracts ^0.0.1 via crates/causlane-contracts`, `causlane-core ^0.0.1 via crates/causlane-core`
- Package files: 43

#### causlane-runtime

- Manifest: `crates/causlane-runtime/Cargo.toml`
- Package root: `crates/causlane-runtime`
- Normal workspace dependencies: `causlane-core ^0.0.1 via crates/causlane-core`
- Package files: 37

#### causlane-codegen

- Manifest: `crates/causlane-codegen/Cargo.toml`
- Package root: `crates/causlane-codegen`
- Normal workspace dependencies: `causlane-contracts ^0.0.1 via crates/causlane-contracts`
- Package files: 30

#### causlane-cli

- Manifest: `crates/causlane-cli/Cargo.toml`
- Package root: `crates/causlane-cli`
- Normal workspace dependencies: `causlane-codegen ^0.0.1 via crates/causlane-codegen`, `causlane-contracts ^0.0.1 via crates/causlane-contracts`, `causlane-core ^0.0.1 via crates/causlane-core`, `causlane-replay ^0.0.1 via crates/causlane-replay`
- Package files: 42

## Publication Execution

- Status: `deferred`
- Reason: Actual upload is deferred until the explicit release runbook is invoked.
- Deferred dependencies: `causlane-core ^0.0.1 via crates/causlane-core`
- Publish sequence: `causlane-core`, `causlane-contracts`, `causlane-runtime`, `causlane-replay`, `causlane-codegen`, `causlane`, `causlane-cli`
- Next command: `cargo package -p causlane-core --list`
- Dry-run command: `cargo publish -p causlane-core --dry-run --locked`
- Actual publish command: `cargo publish -p causlane-core --locked`
- Note: This report is a deterministic repository gate. It does not publish crates, reserve names, verify tokens or claim that registry upload has been executed.

## Checks

### workspace-version

- Status: `pass`
- Summary: Workspace packages use the first public publication version.
- Evidence: workspace package versions: 0.0.1
- Remediation: Set workspace package version and lockfile packages to 0.0.1.

### facade-metadata

- Status: `pass`
- Summary: Facade crate metadata is complete enough for a readiness report.
- Evidence: description/license/authors/repository/homepage/documentation/keywords/categories/rust-version are present
- Remediation: Fill required package metadata in Cargo.toml before publication.

### license-files

- Status: `pass`
- Summary: Dual-license files match the workspace license expression.
- Evidence: present: LICENSE-MIT, LICENSE-APACHE
- Remediation: Add both license files or update the workspace license expression and release docs.

### root-publication-docs

- Status: `pass`
- Summary: Root publication, contribution, security and AI-policy docs are present.
- Evidence: present root docs: README.md, LICENSE-MIT, LICENSE-APACHE, AI_USAGE.md, AGENTS.md, CONTRIBUTING.md, SECURITY.md, CHANGELOG.md, PUBLISHING.md, RELEASE.md
- Remediation: Add the missing root publication documents before publication.

### crate-readme

- Status: `pass`
- Summary: The facade package has a crate-local README for crates.io.
- Evidence: crate README candidates present: crates/causlane/README.md
- Remediation: Add a crate-local README or package.readme before publication.

### workspace-crate-readmes

- Status: `pass`
- Summary: Every workspace crate has a crate-local README for crates.io.
- Evidence: crate READMEs: causlane: crates/causlane/README.md, causlane-core: crates/causlane-core/README.md, causlane-contracts: crates/causlane-contracts/README.md, causlane-replay: crates/causlane-replay/README.md, causlane-runtime: crates/causlane-runtime/README.md, causlane-codegen: crates/causlane-codegen/README.md, causlane-cli: crates/causlane-cli/README.md
- Remediation: Add crate-local README files or explicit package.readme entries.

### workspace-package-file-lists

- Status: `pass`
- Summary: Workspace package file lists include manifests, READMEs and Rust entrypoints.
- Evidence: package file lists captured for 7 workspace crates
- Remediation: Adjust package include/exclude metadata if required files are absent.

### workspace-publication-order

- Status: `pass`
- Summary: Workspace crate publication order is machine-derived from normal path dependencies.
- Evidence: dependency tiers: tier 0: causlane-core, tier 1: causlane, causlane-contracts, causlane-runtime, tier 2: causlane-replay, causlane-codegen, tier 3: causlane-cli, facade publish sequence: causlane-core, causlane
- Remediation: Break workspace dependency cycles before publishing internal crates.

### full-workspace-publish-sequence

- Status: `pass`
- Summary: Full workspace publication sequence is explicit and release-runbook aligned.
- Evidence: sequence: causlane-core, causlane-contracts, causlane-runtime, causlane-replay, causlane-codegen, causlane, causlane-cli
- Remediation: Update the publish sequence or workspace package list before publication.

### internal-dependency-versions

- Status: `pass`
- Summary: Internal workspace path dependencies carry registry-compatible versions.
- Evidence: all internal path dependencies match their package versions
- Remediation: Update internal path+version dependency requirements before publication.

### feature-surface

- Status: `pass`
- Summary: Workspace feature flags keep the facade minimal and optional integrations explicit.
- Evidence: policy: m11_2_default_minimal_optional_integrations, feature-bearing packages: causlane-runtime features=apalis, default, otel, postgres-audit, restate, sqlite-audit, tokio-runtime, all workspace crates have empty default features and explicit optional integration flags
- Remediation: Move optional integrations behind non-default features and keep docs.rs metadata explicit.

### publication-dependencies

- Status: `warn`
- Summary: Facade normal dependencies are deferred until internal crates are published.
- Evidence: deferred unpublished path dependencies: causlane-core ^0.0.1 via crates/causlane-core
- Remediation: Publish internal crates first before attempting a real facade upload.

### facade-dry-run

- Status: `warn`
- Summary: `cargo publish --dry-run -p causlane` remains an execution probe, not a repository readiness blocker.
- Evidence: actual facade dry-run is deferred until internal dependencies are available from crates.io
- Remediation: Run `./tools/cargo-dev publish -p causlane --dry-run` only after dependency blockers clear.

## Package Files

`.cargo_vcs_info.json`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, `README.md`, `benches/dispatch_baseline_bench_suite.rs`, `fixtures/contracts/examples/release_promote.registry.yaml`, `fixtures/contracts/scenarios/release_promote_success.scenario.yaml`, `src/lib.rs`, `tests/public_api.rs`
