# Code review finding resolution matrix — 2026-06-25

**Status:** mandatory input to the nearest publication refactor.

This matrix turns the external read-only code review dated 2026-06-25 into an
explicit refactor backlog. It is intentionally publication-oriented: a finding can
be acceptable for private experimentation and still block the first public
pre-alpha release.

## Integration status (2026-06-25 patch-021 fold-in)

Resolved while folding the review into this repository:

- **M7** — `causlane-runtime` now ships `[package.metadata.docs.rs]`.
- **L12** — `causlane-formal/README.md` lists the real API
  (`Requirement` / `report` / `report_with_context`) and a crate non-goal.
- **L18** — root README documents `cargo install causlane-cli` and the
  facade-vs-binary distinction.
- **Tooling/CI** — `tools/pre-publication-review-gate` and GitHub CI added so the
  finding classes above are guarded going forward.
- **M8** — workspace-root test fixtures are now **vendored** into per-crate
  `fixtures/` dirs (`include_str!` paths stay inside each crate), and the gate
  enforces byte-for-byte sync with canonical `contracts/` via
  `PUB-VENDORED-FIXTURE-DRIFT`/`-ORPHAN`. The pre-publication review gate now
  passes.

Decided (not "fixed"):

- **M5** — the project keeps crate-level `#![deny(warnings)]` as deliberate
  policy and *also* enforces warnings-as-errors in CI and the local checker. See
  the M5 row below.

No publication-blocking correctness finding remains open after M2 closeout.
L-series cleanup remains tracked separately, and PUB5 still requires the full
PUB0-PUB4 release gate rather than this matrix alone.

## Policy

Before crates.io publication, every finding below must be in one of these states:

- `fixed`: implemented and covered by tests/checks;
- `mitigated`: intentionally narrowed, documented and guarded;
- `deferred`: explicitly accepted as non-blocking for `0.0.1` with owner and
  follow-up milestone;
- `not_applicable`: refuted against current code.

Untriaged findings block PUB5.

## Publication-blocking set

| ID | Area | Required state before PUB5 | Required action |
|---|---|---|---|
| H1 | core constraints | `fixed` | **Done** — `resolve_constraints` now folds prior same-batch token claims into the held amount (`batch_token_amount`), so several claims on one resource cannot collectively over-allocate the budget (over-subscription → `Wait`). Regression test `same_resource_token_claims_in_a_batch_cannot_over_allocate` added. |
| H2 | runtime supervision | `fixed` | **Done** — in-process host effect execution is supervised through one worker helper. Handler panics before future creation or while polling become `Failed` events with `HandlerRejected`, and the partition worker continues processing later independent tasks. Regression tests cover both panic phases. |
| H3/M4 | codegen identifiers | `fixed` | **Done** — `build_formal_ir` now runs `identifier_check::check_identifier_injectivity`, failing closed (`CodegenError::Collision`) when two distinct domain names of one kind sanitize to the same Alloy/P identifier. Per-namespace collision tests added (predicate/event/action/resource/lease-scope/fact-scope) plus a lease-vs-fact-scope no-false-collision test. Detection-only: non-colliding output is byte-identical, so committed goldens are unchanged. |
| H4 | stale-check | `fixed` | **Done** — `stale_check_with_expected` now fails closed when no receipt is supplied; the receipt path already compares `generated_artifact_hash` against the artifact (and the header is part of the hashed text), so a hand-edited body/header is caught. Regression tests: no-receipt rejected, body-edit-without-receipt rejected, matching-receipt accepted. |
| H5/M6 | formal evidence | `mitigated` | **Mitigated.** (1) Provenance/trust-boundary policy `docs/formal/09-…`: receipts are **unsigned evidence, not signed proof**, and re-derivation is the authority; the coverage module + anti-theatre doc carry the disclaimer; `pre-publication-review-gate` enforces the policy (`PUB-FORMAL-PROVENANCE-POLICY`). (2) The full formal toolchain (Alloy/Z3/P/Kani/Verus/Lean4 + Rust 1.95) is **installed on `ci-dispatcher.lan`** and `tools/formal-doctor` passes, enabling **CI re-derivation** as the trust anchor. The z3/P environment caveat is closed for the formal gate: z3 resolves to the configured Verus-bundled 4.12.5 binary, and fresh local + `ci-dispatcher.lan` `tools/formal-verify-all --profile all --lane local_smoke` runs pass with derived coverage `status=pass`. The empty-URL `cli-checker` artifact remains a separate devinfra bootstrap P0; cryptographic receipt signing is deferred. |
| M1 | runtime partitions | `fixed` | **Done** — wait-mode routed submit validates the route, reserves primary ingress capacity, and only then acquires route permits before sending through the reserved slot. Participant permits are not held while waiting for primary ingress capacity; regression test covers a saturated primary with an independent participant admission. |
| M2 | runtime state growth | `fixed` | **Done** — `InProcessRuntimeConfig` now exposes `partition_history_bound`, and each partition retains completed task ids, failed task ids and idempotency keys through one bounded FIFO set helper. Within the retention window, duplicate suppression and dependency readiness keep existing semantics; after eviction, old completions no longer satisfy new dependencies and old idempotency keys may be reused. Regression tests cover config validation, idempotency eviction and completed-dependency eviction. |
| M3 | replay CLI | `fixed` | **Done** — bundle-less `replay verify` no longer prints a generic “verified”; it reports “passed structural checks” and labels the coverage structural-only, pointing at `--bundle` for full replay (execution-bearing traces, detected via `EventKindDto`, get an explicit caveat that predicate/barrier/witness/capability/authz were NOT checked). Smoke test asserts the structural-only label. |
| M5 | packaging | `mitigated` (decided) | **Decision: keep** crate-level `#![deny(warnings)]` as deliberate policy and enforce warnings-as-errors in CI (`clippy -- -D warnings`) and the local `.devinfra` checker. The review's *removal* recommendation is declined; the downstream new-compiler-warning risk is accepted for pre-alpha and revisited if it bites. The `pre-publication-review-gate` therefore does **not** flag `#![deny(warnings)]`. |
| M7 | docs.rs | `fixed` | Add docs.rs metadata for feature-gated public surfaces, at least `causlane-runtime`. **Done.** |
| M8 | package contents | `fixed` | No published crate may `include_str!`/`include_bytes!` workspace-root files absent from its `.crate` tarball. **Done** — fixtures vendored into per-crate `fixtures/`; `tools/pre-publication-review-gate` now checks escape-the-crate includes and enforces vendored↔canonical byte-sync. |
| L12 | docs/API | `fixed` | Fix `causlane-formal` README to list real API entry points. **Done.** |
| L16 | public API | `fixed` | **Done** — the `causlane` facade now exposes a curated `core::{protocol, kernel, ports, prelude}` wrapper instead of a full `causlane-core` alias. `causlane::core::testing` is intentionally hidden and covered by a compile-fail doc test; direct testing helpers remain available through `causlane_core::testing`. |
| L18 | install docs | `fixed` | Document `cargo install causlane-cli` and clarify that `causlane` is the library facade. **Done.** |

## Refactor backlog by stage

### PUB0 — Repository and architecture refactor

- H1: constraint arbitration fix — **done** (token-budget batch over-allocation).
- H2: runtime panic supervision — **done**. M1: route ingress reservation —
  **done**. M2: runtime state-retention boundaries — **done**.
- H3/M4: generated-target identifier soundness — **done** (collision-detecting
  in `build_formal_ir`). L8/L10: remaining generated-target soundness boundary.
- L1/L2/L3/L4/N1/N2/N3: core cleanup while touching constraint/lifecycle code.
- Missing Cargo binary sources must remain fixed; `tools/architecture-lint` must pass.
- M8: **done** — workspace-root test fixtures vendored into per-crate `fixtures/`
  across `causlane-replay`/`-codegen`/`-contracts`/`-cli`/`causlane`; the
  `pre-publication-review-gate` passes and enforces vendored↔canonical sync.

### PUB1 — Readability and maintainability

- Rename tests/functions that describe historical stages rather than protocol semantics.
- Add comments where fail-closed behavior is intentional, especially for L1/L2/L5/L7/N5.
- Fuzz/property: a first PUB1 parse-boundary slice exists (`fuzz/` replay
  trace/scenario/registry targets plus
  `crates/causlane-replay/tests/proptest_parse_boundaries.rs`). Numeric-boundary
  property coverage and corpus seeds now cover replay DTO timestamps/leases/op
  indexes and registry freshness/version fields. Core property coverage now
  samples lifecycle reducer inputs and token-budget constraint outcomes through
  `KernelContracts` without duplicating semantic authority. The routine long-run
  budget is defined as 15 minutes per protocol fuzz target; the 2026-06-26
  execution on host `dispatcher` completed all three protocol targets with
  status 0 and produced no crash/reproducer artifacts. Future fuzz findings
  remain subject to curated corpus plus review-matrix rows. See
  `docs/release/refactor-before-publication-gate.md` ("Fuzz & property
  adoption") and
  `docs/formal/impact/2026-06-26-pub1-ci-fuzz-long-run.md`.
- Replace broad claims with check-specific language in docs.

### PUB2 — Public API review

- Facade exports and `causlane::core::testing` exposure — **done** for L16.
- Check each crate README against actual public API; `causlane-codegen` Alloy
  generator names are corrected.
- Add install section for `causlane-cli`.
- Resolved 2026-06-27: the YAML parser boundary moved to internal `noyalib`
  compat use; parser error types remain outside the public Rust API.

### PUB3 — Human and agent documentation

- Document that generated formal evidence must be generated/re-derived, not hand-maintained.
- Add review-finding classes to `AGENTS.md`: do not add non-injective sanitizers, do not
  trust editable headers, do not add workspace-root `include_str!` in published crates.
  (The project deliberately *keeps* `#![deny(warnings)]`; do not remove it.)

### PUB4 — GitHub baseline and history

- CI is in place (`.github/workflows/ci.yml`); run secret scan and package-list review on
  the exact public baseline.
- Mark review findings in the release issue.

### PUB5 — crates.io publication

- All publication-blocking rows must be `fixed`/`mitigated`/`deferred` with owner.
- Package file lists must be inspected after the fixture/include cleanup.
- Staged publication order remains dependency-first.

## Deferred candidates for 0.0.1

These may be deferred only with explicit release-note disclosure:

- L5/L6: audit retention/migration story, if SQL audit adapters are documented as experimental.
- L10/L14: Verus/Lean fidelity nuance, if formal claims are narrowed to “abstract lane” in public docs.
- L15: MSRV CI gate, if README says MSRV is declared but not yet CI-enforced.
- N1-N8: nits, except where they are cheap and adjacent to a touched module.

## Non-negotiable publication blockers

The following must not be deferred for a public pre-alpha that advertises formal/replay discipline:

- **All non-negotiable findings are resolved**: **H1** (batch over-allocation), **H3/M4** (codegen identifier collisions), **H4** (stale-check fail-closed), **M3** (replay-verify overclaim), **M8** (fixtures vendored + drift-guarded), and **H5/M6 — mitigated** (CI re-derivation on the formal-capable ci-dispatcher + provenance/trust policy + coverage disclosure).
- H5/M6 mitigation: **provenance/trust policy + coverage disclosure + gate guard** (a committed, enforced statement that receipts are unsigned evidence, not signed proof), backed by an installed formal toolchain on `ci-dispatcher.lan` and fresh local + CI `formal-verify-all` re-derivation with coverage `status=pass`. The z3/P caveat is closed for the formal gate; the out-of-band `cli-checker` artifact remains a separate devinfra bootstrap P0, and cryptographic receipt signing remains future hardening.
- Remaining publication-track work is non-blocking-by-default L-series cleanup;
  M2 state retention is fixed.
