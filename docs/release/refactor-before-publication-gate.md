# Refactor-before-publication gate

**Status:** mandatory pre-publication gate; PUB0-PUB4 are recorded complete for
the current public baseline, and package file-list review is recorded for PUB5.

Causlane must not upload any crate to crates.io unless this gate remains valid
for the selected baseline. The current next repository action is:

```text
run the staged one-crate dry-run for causlane-codegen;
publish causlane-codegen only after its dry-run and maintainer confirmation;
continue the staged PUB5 sequence one crate at a time.
```

This document exists to remove ambiguity between the normal product roadmap and
the publication track. S11 is active only as **publication preparation**; it does
not authorize upload outside the staged runbook.

## Required order

```text
PUB0 Repository and architecture refactor
  -> PUB1 Readability and maintainability
  -> PUB2 Public API review
  -> PUB3 Human and agent documentation
  -> PUB4 GitHub baseline and history curation
  -> PUB5 crates.io staged publication
  -> PUB6 post-publication stabilization
```

PUB5 is unreachable until PUB0-PUB4 are recorded as complete in the release
issue or release branch notes.

## PUB0 — Repository and architecture refactor

Required outcomes:

- `tools/architecture-lint --json` passes;
- `tools/pre-publication-review-gate --json` passes;
- every declared Cargo binary has a source file;
- no source file required by a `#[path = ...]` module attribute is missing;
- crate boundaries match the intended hexagonal architecture;
- generated-truth authority remains explicit;
- duplicated invariant logic has an owner or an exception;
- production identifiers do not contain patch-pack, checkpoint, stage or
  milestone vocabulary.

## PUB1 — Readability and maintainability

Required outcomes:

- module-level docs explain authority boundaries;
- function/variable/test names describe protocol semantics, not implementation
  history;
- diagnostics are stable enough for pre-alpha users;
- AI-agent instructions document naming and authority-boundary pitfalls;
- code comments explain invariants and edge cases, not historical accidents;
- real fuzz/property coverage is adopted — the smoke scaffold is replaced or
  extended with protocol-meaningful targets (see "Fuzz & property adoption").

## Fuzz & property adoption (after PUB0 refactor, before PUB5 publication)

Before PUB1 began, a **minimal smoke scaffold** existed only to keep the
fuzz/property CI pipeline exercised on the `ci-dispatcher` machine:

- `fuzz/` — one cargo-fuzz target (`requirement_from_tokens`);
- `crates/causlane-formal/tests/proptest_smoke.rs` — one proptest property.

That scaffold alone is explicitly **not** real coverage. After the PUB0 refactor
settles the protocol-critical surfaces, and as part of PUB1, real adoption is
required and is a **hard prerequisite for PUB5**:

- replace/extend the fuzz targets with parse-boundary targets for replay DTOs
  (trace/registry/scenario) and numeric extremes;
- add property tests for constraint/lifecycle invariants and replay determinism,
  not just no-panic smoke;
- run the fuzz targets on `ci-dispatcher` for a defined time budget, commit a
  seed corpus, and record any findings as review-matrix rows;
- document the routine long-run budget in the CI machine's local notes.

Publication must not proceed on the smoke scaffold alone.

Update 2026-06-25: PUB1 now has a first real parse-boundary slice. The fuzz
crate includes dedicated targets for replay trace JSON, replay scenario YAML and
registry YAML compilation, and `causlane-replay` has proptest coverage for
parse/lowering determinism over generated text.

Update 2026-06-26: PUB1 numeric-boundary coverage now extends that slice. The
replay trace/scenario/property tests generate documents with `u64` timestamp,
lease amount/epoch/expiry and `u32` op-index boundary values, and the registry
property tests generate `u64` authz freshness and `u32` version boundary values.
The fuzz corpus includes numeric-extreme seeds for the three protocol targets.
The routine `ci-dispatcher` long-run budget is now defined as **15 minutes per
protocol target**:

```bash
cargo fuzz run replay_trace_json -- -max_total_time=900
cargo fuzz run replay_scenario_yaml -- -max_total_time=900
cargo fuzz run registry_yaml_compile -- -max_total_time=900
```

Update 2026-06-26: the 15-minute protocol fuzz runs were executed on host
`dispatcher` with `cargo-fuzz 0.13.2` and `nightly-2025-11-21`. All three
targets completed with status 0 and produced no crash/reproducer artifacts:

| Target | Runs | Seconds | Peak RSS |
|---|---:|---:|---:|
| `replay_trace_json` | 41,735,952 | 901 | 565 MB |
| `replay_scenario_yaml` | 7,678,201 | 901 | 337 MB |
| `registry_yaml_compile` | 8,963,580 | 901 | 383 MB |

See `docs/formal/impact/2026-06-26-pub1-ci-fuzz-long-run.md`. Any future
reproducer/finding from these targets must still be committed as curated corpus
plus a review-matrix row.

Update 2026-06-26: PUB1 now also has core lifecycle/constraint property coverage.
`causlane-core` has dev-only proptests that exercise `KernelContracts` against
the existing lifecycle reducer and constraint arbiter, including sampled
lifecycle triples and token-budget boundary outcomes. This does not create a
second semantic authority: the tests delegate to the public core contract
surface and the existing pure authority functions. The long-run fuzz execution
evidence is recorded separately in
`docs/formal/impact/2026-06-26-pub1-ci-fuzz-long-run.md`.

## PUB2 — Public API review

Required outcomes:

- every published crate has a crate-by-crate review entry in
  `docs/api/public-api-review.md`;
- facade exports are intentionally small;
- default features are minimal and do not pull runtime/formal toolchains
  accidentally;
- unstable surfaces are documented;
- examples compile against intended imports.

Update 2026-06-26: PUB2 public API review is recorded in
`docs/api/public-api-review.md`. The `causlane` facade is intentionally narrow,
runtime adapters remain feature-gated and non-default, crate READMEs carry
pre-alpha/non-goal language, and `cargo doc --workspace --no-deps --locked`
plus workspace doc-tests pass. The direct `causlane-core` low-level module
surface remains an accepted `0.0.1` pre-alpha caveat and must be revisited
before `0.1`; it is not re-exported through the facade.

## PUB3 — Documentation for humans and agents

Required outcomes:

- root docs exist: README, PUBLISHING, RELEASE, CONTRIBUTING, AGENTS,
  AI_USAGE, SECURITY, CHANGELOG, licenses;
- each crate README says experimental/pre-alpha;
- AI-assisted development policy is explicit;
- agent docs forbid weakening invariants, maintaining generated truth by hand,
  and using stage/milestone labels in production code.

Update 2026-06-26: PUB3 documentation readiness is recorded. Root publication,
contribution, security, release and AI/agent policy docs exist; every workspace
crate README states experimental/pre-alpha status; CLI/codegen README notes now
make the non-authority boundary explicit; and generated publish-readiness
artifacts were regenerated with `tools/publish-readiness --write`. At that
point, this did not authorize upload or public baseline opening; PUB4 remained
the next gate.

## PUB4 — GitHub baseline and history curation

Required outcomes:

- public history is curated or explicitly accepted;
- secret and context-pack scans have been run on the exact public baseline;
- repository metadata resolves publicly;
- issue/PR templates exist;
- branch protection and release ownership are decided.

Update 2026-06-26: the repository-local context-pack hygiene scan now passes on
git-visible files and is wired into the normal CI repository gates. This does
not replace the external secret scan on the curated public baseline, which
remains required before public opening or crates.io upload.

Update 2026-06-26: publish-readiness now records repository visibility as an
explicit online preflight instead of leaving the public-repository requirement as
prose only. The deterministic report remains repository-local and still does not
make network availability a blocking local gate. The advisory probe currently
reports `repository_visibility.visibility = "private_or_missing"` for
`https://github.com/Boot-and-Pencil/causlane` (unauthenticated HTTP 404), so
PUB4 is not complete for public opening or PUB5 upload until the curated public
baseline is made publicly resolvable and scanned.

Update 2026-06-26: `docs/release/pub4-public-baseline-handoff.md` now names
Gitleaks as the required PUB4 secret scanner for the exact curated public
baseline and records the branch-protection/release-owner handoff fields. The
scan report stays outside the repository; this document is release evidence, not
a generated truth artifact.

Update 2026-06-26: PUB4 GitHub opening is complete. `main` was rewritten to the
single curated public baseline commit
`e2c376803a578dbe1db688b2db194f657f37e812`, the repository now resolves
publicly, Gitleaks and context-pack scans passed on that baseline, and `main`
branch protection requires the CI jobs `repository gates`, `rust workspace` and
`declared MSRV`. Force pushes and branch deletion are disabled. This completes
the GitHub-opening part of PUB4; PUB5 upload still requires package file-list
inspection and the staged publication runbook.

## PUB5 — crates.io staged publication

Required outcomes:

- package file lists inspected for every crate;
- internal dependencies are published first;
- each crate is dry-run and uploaded one at a time;
- no upload uses `--allow-dirty`;
- downstream smoke project can depend on `causlane@0.0.1`.

Update 2026-06-26: package file lists were inspected for every workspace crate
and recorded in `docs/release/pub5-package-file-list-review.md`. No dry-run or
upload was performed as part of that review. The next PUB5 action at that point
was the staged one-crate dry-run/publish sequence, starting with
`causlane-core`.

Update 2026-06-26: the `causlane-core` dry-run passed and is recorded in
`docs/release/pub5-causlane-core-dry-run.md`. No crates.io upload was performed.
The next irreversible PUB5 action is publishing `causlane-core`, then waiting
for it to be indexed before any dependent crate dry-run.

Update 2026-06-26: `causlane-core 0.0.1` was published and indexed on
crates.io. Evidence is recorded in
`docs/release/pub5-causlane-core-publication.md`. The publication state is now
`Indexed(causlane-core)`; the next runbook step, if maintainers continue, is a
one-crate dry-run for `causlane-formal`. YAML-facing crates remain blocked by
the M11.5 dependency-hygiene decision until that debt is resolved or explicitly
accepted.

Update 2026-06-27: `causlane-formal 0.0.1` was published and indexed on
crates.io. Evidence is recorded in
`docs/release/pub5-causlane-formal-publication.md`. The publication state is now
`Indexed(causlane-formal)`. The next runbook crate is `causlane-contracts`.

Update 2026-06-27: the M11.5 YAML parser debt was resolved by migrating
YAML-facing crates from `serde_yaml 0.9.34+deprecated` / `unsafe-libyaml` to
`noyalib 0.0.8` with `compat-serde-yaml`, and by raising the declared workspace
MSRV to `1.85`. The `causlane-contracts` package-list review and dry-run passed;
evidence is recorded in
`docs/release/pub5-causlane-contracts-dry-run.md`. The current next irreversible
action is publishing `causlane-contracts` after CI and maintainer confirmation.

Update 2026-06-27: `causlane-contracts 0.0.1` was published and indexed on
crates.io. Evidence is recorded in
`docs/release/pub5-causlane-contracts-publication.md`. The publication state is
now `Indexed(causlane-contracts)`. The next runbook crate is
`causlane-runtime`.

Update 2026-06-27: the `causlane-runtime` package-list review was rechecked and
the staged dry-run passed. Evidence is recorded in
`docs/release/pub5-causlane-runtime-dry-run.md`. No crates.io upload was
performed. The publication state is now `DryRunPassed(causlane-runtime)`. The
next irreversible action is publishing `causlane-runtime` after CI and
maintainer confirmation.

Update 2026-06-28: `causlane-runtime 0.0.1` was published and indexed on
crates.io. Evidence is recorded in
`docs/release/pub5-causlane-runtime-publication.md`. The publication state is
now `Indexed(causlane-runtime)`. The next runbook crate is `causlane-replay`.

Update 2026-06-28: the staged `causlane-replay` dry-run passed. Evidence is
recorded in `docs/release/pub5-causlane-replay-dry-run.md`. The publication
state is now `DryRunPassed(causlane-replay)`. The next irreversible action is
publishing `causlane-replay` after CI and maintainer confirmation. The dry-run
evidence also records a runtime Restate dependency/MSRV follow-up before
claiming workspace-wide Rust `1.85` all-features compatibility.

Update 2026-06-28: `causlane-replay 0.0.1` was published and indexed on
crates.io. Evidence is recorded in
`docs/release/pub5-causlane-replay-publication.md`. The publication state is now
`Indexed(causlane-replay)`. The next runbook crate is `causlane-codegen`.

Update 2026-06-26: a hostile-audience publication review is recorded in
`docs/release/adversarial-audience-publication-review-2026-06-26.md`.
Immediate hygiene fixes were folded into M11.5 and the release plan:
`cargo-deny` is now an explicit hard blocker, the Restate RSA advisory regression
is documented as fixed by the `aws_lc_rs` backend, the YAML parser finding is
resolved by the 2026-06-27 `noyalib` migration, and duplicate dependency
warnings remain tracked before full workspace publication proceeds.

## Non-negotiable rule

If a refactor/readability/API/doc/history gate is still open, publication remains
blocked even when `publish-readiness` reports repo-local deterministic readiness.

## Review-finding gate

The 2026-06-25 skeptical review is folded into PUB0/PUB1/PUB2. Before PUB5,
every row in `docs/refactor/code-review-finding-resolution-matrix-2026-06-25.md`
and every confirmed finding in
`docs/release/adversarial-audience-publication-review-2026-06-26.md` must be
`fixed`, `mitigated`, explicitly `deferred` with an owner, or `not_applicable`.

Run:

```bash
python3 tools/pre-publication-review-gate --json | jq -e '.status == "pass"'
```

The automated matrix gate is fail-closed for the findings it owns: it
intentionally fails while publication-blocking review findings remain visible.
The 2026-06-26 adversarial-audience review is release evidence and release-plan
backlog until it is wired into a machine-readable gate. As of 2026-06-25 the M8
workspace-root fixture issue is fixed, and this gate reports `pass`; future
fixture drift or new publication-blocking findings should make it fail closed
again.
