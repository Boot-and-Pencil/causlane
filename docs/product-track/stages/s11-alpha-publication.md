# S11 — Public pre-alpha/bootstrap and alpha publication

**Status:** `active_pre_alpha_prep`

**Purpose:** prepare public source/package provenance first, then mature toward
public alpha. The full-workspace `0.0.1` pre-alpha crates.io bootstrap release
is published; a later `0.1.x` release is the first public alpha.

## Current sub-track: publication bootstrap

The public GitHub baseline is open, PUB0-PUB4 are recorded complete, package
file-list review is recorded for every workspace crate, all eight runbook
crates have been published and indexed, signed tag `v0.0.1` is pushed, and the
GitHub pre-release is public. Public follow-up issues remain optional/deferred.
M11.4 Examples and M11.6 Contributor guide are closed for public alpha
preparation; the next product-roadmap action is M11.5 security/release hygiene
hardening:

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
- **Outcome:** No-publish readiness cleared deterministic local blockers and
  handed off to the staged release evidence.
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

- **Status:** `done_or_near_done`
- **Outcome:** simple-local, approval-gate, consequence-parallelism and why-not-parallel are runnable and checked.
- **Definition of done:**
  - examples build without unpublished external infrastructure;
  - examples do not imply production workflow-engine readiness.

Update 2026-06-29: `examples/simple-local` is a standalone runnable example
checked by `tools/examples-check` and CI. It covers one local runtime-execution
flow with in-memory audit, barrier/capability, observed truth, projection anchor,
replay verification and a missing-barrier negative control.

Update 2026-06-29: `examples/approval-gate` is a standalone runnable example
checked by `tools/examples-check` and CI. It covers fail-closed approval
outcomes, exact action/plan/impact binding, step-up, separation-of-duties,
deny-wins and bundle-bound replay refutation of a wrong-plan approval witness.

Update 2026-06-29: `examples/consequence-parallelism` is a standalone runnable
example checked by `tools/examples-check` and CI. It covers conflict-free
frontier selection, pending write conflicts, lane capacity and bundle-bound
replay refutation of overlapping exclusive leases.

Update 2026-06-29: `examples/why-not-parallel` is a standalone runnable example
checked by `tools/examples-check` and CI. It covers pairwise pending-write
conflicts, dependency blockers, active writer blockers and positive
parallelizable explanations.

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

- **Status:** `done_or_near_done`
- **Outcome:** public contributor guide consolidates ADR process, new predicate checklist, formal obligation template, adapter certification and AI accountability.
- **Definition of done:**
  - contributor docs explain human accountability for AI-assisted changes;
  - agent docs prohibit common architecture/documentation regressions.

Update 2026-06-29: `CONTRIBUTING.md` now routes contributors through the
existing ADR, predicate, formal impact, formal obligation, adapter
certification and milestone execution guidance. `AGENTS.md` points agents back
to that public guide while preserving the generated truth chain as authority.

### M11.7 — Release notes

- **Status:** `done_or_near_done`
- **Outcome:** clear limitations: not workflow engine, formal lanes coverage, unstable APIs.
- **Definition of done:**
  - `CHANGELOG.md` includes `0.0.1` pre-alpha notes;
  - release notes list all crates published;
  - known limitations and next roadmap are explicit.

Update 2026-06-29: `CHANGELOG.md`, `RELEASE.md` and
`docs/release/pub6-v0.0.1-post-publication.md` record the published crate list,
checksums, downstream smoke, known limitations and next PUB6 decisions.

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
