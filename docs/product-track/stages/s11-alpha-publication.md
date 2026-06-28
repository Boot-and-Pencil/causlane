# S11 — Public pre-alpha/bootstrap and alpha publication

**Status:** `active_pre_alpha_prep`

**Purpose:** prepare public source/package provenance first, then mature toward
public alpha. The current target is a full-workspace `0.0.1` pre-alpha crates.io
bootstrap release. A later `0.1.x` release is the first public alpha.

## Current sub-track: publication bootstrap

The public GitHub baseline is open, PUB0-PUB4 are recorded complete, package
file-list review is recorded for every workspace crate, and `causlane-core`,
`causlane-formal`, `causlane-contracts`, `causlane-runtime` and
`causlane-replay` have been published and indexed. The immediate next action is
the staged package-list and dry-run gate for `causlane-codegen`:

```text
PUB0 refactor first
PUB1 readability/maintainability
PUB2 public API review
PUB3 human/agent docs
PUB4 GitHub history/repository preparation
PUB5 staged crates.io publication
PUB6 post-publication stabilization
```

## Milestones

### M11.1 — Crate naming/publish check

- **Status:** `exists_expand`
- **Outcome:** No-publish readiness clears deterministic local blockers while
  publication execution remains deferred.
- **Definition of done:**
  - generated readiness report is current;
  - report clearly separates repo-local readiness from actual upload readiness;
  - public crate names are checked as advisory probes, not treated as reserved;
  - publication order is machine-derived and points to the staged runbook.

### M11.2 — Feature flags

- **Status:** `done_or_near_done`
- **Outcome:** default minimal; existing optional runtime integrations are
  explicit, non-default and captured in generated readiness evidence.
- **Definition of done:**
  - default features do not pull runtime adapters into core/facade accidentally;
  - `docs/release/publish-readiness.json` records the Cargo-derived feature
    surface;
  - crate READMEs describe feature flags honestly;
  - docs.rs build does not require external formal tools.

### M11.3 — Public API review

- **Status:** `done_or_near_done`
- **Outcome:** Rust API guidelines, builders, newtypes, no raw Strings on critical fields.
- **Definition of done:**
  - `docs/api/public-api-review.md` is completed crate-by-crate;
  - public exports are intentional;
  - current doc examples compile against intended imports;
  - unstable surfaces are marked as such.

### M11.4 — Examples

- **Status:** `planned`
- **Outcome:** simple-local, approval-gate, consequence-parallelism, why-not-parallel.
- **Definition of done:**
  - examples build without unpublished external infrastructure;
  - examples do not imply production workflow-engine readiness.

### M11.5 — Security/release hygiene

- **Status:** `exists_harden`
- **Outcome:** licenses, dependency audit, context-pack scan, secret rules, vulnerability policy.
- **Definition of done:**
  - root security and AI/provenance docs are present;
  - `cargo-deny` advisory/license/source policy passes without unreviewed
    suppressions;
  - package file-list review is recorded for the selected baseline;
  - secret/context scan is recorded before publication;
  - deprecated parser boundaries and duplicate dependency warnings are either
    resolved or explicitly tracked before full workspace publication.

### M11.6 — Contributor guide

- **Status:** `planned`
- **Outcome:** ADR process, new predicate checklist, formal obligation template, adapter certification.
- **Definition of done:**
  - contributor docs explain human accountability for AI-assisted changes;
  - agent docs prohibit common architecture/documentation regressions.

### M11.7 — Release notes

- **Status:** `planned`
- **Outcome:** clear limitations: not workflow engine, formal lanes coverage, unstable APIs.
- **Definition of done:**
  - `CHANGELOG.md` includes `0.0.1` pre-alpha notes;
  - release notes list all crates published;
  - known limitations and next roadmap are explicit.

## Exit gate for `0.0.1`

- PUB0-PUB4 are complete and recorded;
- refactor-before-publication gate records PUB0-PUB4 complete;
- public API review is recorded;
- GitHub public baseline is curated and scanned;
- package file lists are reviewed;
- all crates are dry-run/published in dependency order;
- downstream smoke project can depend on `causlane@0.0.1`.

## Exit gate for public alpha `0.1.x`

- examples are runnable;
- cookbook/docs are usable by external users;
- formal/replay status is honest and receipt-backed;
- feature flags and facade surface are intentionally shaped;
- at least one reference integration validates the API story.
