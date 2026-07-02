# Publication blockers from dispatcher-020 review

**Status:** current publication blocker record.

This document records the concrete state after the 2026-06-25 deep skeptical
review, folded into the live repository (which had already moved past the
`dispatcher-020` snapshot the review was written against).

## Immediate verdict

The repository is not ready for GitHub publication or crates.io upload.

The publication order remains:

```text
finish PUB0-PUB4 first;
then publish all crates in dependency order.
```

## State at fold-in

The review was written against the `dispatcher-020` snapshot. Re-checked against
the live tree, several of its observations were already resolved, and the rest
are tracked here:

- formal CLI binary sources (`causlane-formal`, `causlane-formal-discipline`)
  **already exist** in the live tree with a richer module layout; the review's
  "missing binary sources" no longer applies — `tools/architecture-lint` passes.
- `causlane-runtime` had **no** docs.rs metadata — now added (M7).
- `causlane-formal` README listed **stale** API names — now corrected (L12).
- root README did not document `cargo install causlane-cli` — now added (L18).
- there was **no** `.github/workflows/` CI — now added.
- workspace-root `include_str!` paths under published crates (M8) — **now
  vendored** into per-crate `fixtures/` and drift-guarded; the gate passes (see
  the dated update below).
- `#![deny(warnings)]` in published crate roots — **kept by decision** (M5):
  the project enforces warnings-as-errors as policy *and* in CI; the review's
  removal recommendation is declined.

## Fixed by this fold-in

- docs.rs metadata for `causlane-runtime` (M7);
- `causlane-formal` README API names and crate non-goal (L12);
- root README install documentation (L18);
- a fail-closed `tools/pre-publication-review-gate`;
- GitHub CI scaffolding (`ci.yml`) and a manual `publication-gate.yml`;
- the review-finding resolution matrix;
- this blocker record.

## Still tracked after dated updates

After the dated updates below, no M-series publication correctness blocker
remains open. The remaining release work is still governed by PUB0-PUB4 and the
staged publication runbook; this document alone does not authorize crates.io
upload.

- L17 YAML parser migration/deprecation decision;
- non-blocking L-series cleanup where it remains relevant to public docs.

(M8 is no longer open — see the dated update below.)

## Required gate before PUB5

Run:

```bash
python3 tools/pre-publication-review-gate --json | jq -e '.status == "pass"'
python3 tools/architecture-lint --json | jq -e '.status == "pass"'
python3 tools/publication-plan-doc-lint --json | jq -e '.status == "pass"'
python3 tools/semantic-naming-scan --json
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

No crates.io upload may proceed while this document has unresolved P0 rows.

## Update — 2026-06-25: M8 closed

Workspace-root test fixtures are now **vendored** into per-crate `fixtures/`
directories across `causlane-replay`, `-codegen`, `-contracts`, `-cli` and
`causlane` (benches); every `include_str!`/`include_bytes!` now resolves inside
its own crate. `tools/pre-publication-review-gate` was upgraded to (a) flag only
includes that *escape the crate* and (b) enforce byte-for-byte sync between each
vendored copy and its canonical `contracts/`/`.devinfra/` source
(`PUB-VENDORED-FIXTURE-DRIFT`/`-ORPHAN`). The gate now reports **status: pass**.

Canonical `contracts/` is unchanged and remains the single source of truth for
the runtime/formal pipeline (`schema-validate-all`, formal negative-control
collection, etc.). At that point, M2 remained a correctness finding the
lightweight gate did not check; see the M2 closeout update below.

## Update — 2026-06-25: H1 closed

`resolve_constraints` no longer over-allocates a token budget when one batch
carries several claims on the same resource: each claim is now checked against the
held leases **plus** the token amounts already claimed earlier in the same batch
(`batch_token_amount`), so an over-subscribed batch yields `Wait` instead of
`Allow`. Single oversized claims still `Deny`; different-resource claims keep
independent budgets. Regression test:
`same_resource_token_claims_in_a_batch_cannot_over_allocate` in
`crates/causlane-core/src/domain/constraint_spec.rs`.

## Update — 2026-06-25: H3/M4 closed

Generated Alloy/P identifiers are now collision-checked. `alloy_ident` (shared by
both backends) maps every non-alphanumeric character to `_` and is not injective,
so distinct domain names like `evt-1`/`evt_1` would silently merge into one sig
and make the proof unsound. `build_formal_ir` — the single IR choke point every
backend passes through — now calls
`identifier_check::check_identifier_injectivity`, which fails closed
(`CodegenError::Collision`) when two distinct names of one kind (predicate, event,
action, resource, lease-scope, fact-scope, plan) sanitize to the same identifier.
This is detection-only: non-colliding output is byte-identical, so the committed
golden `.als`/`.p` artifacts are unchanged (no formal re-verification needed).
At that point, M2 remained open.

## Update — 2026-06-25: H4 closed

The formal-artifact stale-check now **fails closed when no receipt is supplied**.
Previously, with no receipt, `stale_check_with_expected` validated only editable
header comment fields and never compared the artifact body hash, so a hand-edited
body with an intact header passed. The receipt-present path already compares
`generated_artifact_hash` against the artifact (and the header is part of the
hashed text), so requiring a receipt closes the hole. All real callers already
supply receipts (`tools/formal-ready` passes `--receipt`; `scripts/check-verification-full.sh`
generates receipts before `stale-check-all`). Regression tests cover: no-receipt
rejected, body-edit-without-receipt rejected, and a matching receipt accepted.
At that point, M2 remained open.

## Update — 2026-06-25: M3 closed (strict must-fix set complete)

Bundle-less `replay verify <trace.json>` no longer prints a generic "verified".
It runs structural invariants only (it cannot check predicate/barrier/witness/
capability/authz obligations without the compiled bundle), so it now reports
"passed structural checks" and labels the coverage structural-only, pointing at
`--bundle` for full replay. Execution-bearing traces (detected from their own
events via `EventKindDto::ExecutionBarrierLogged`/`ExecutionStarted`) get an
explicit caveat that the bundle-bound obligations were not checked. The
bundle-bound path was already honestly qualified ("verified with bundle … [mode]").
With M3 closed, the **entire strict non-negotiable must-fix set (H1, H3/M4, H4,
M3, M8) was complete**. At that point, M2 remained open.

## Update — 2026-06-25: H5/M6 mitigated (no non-negotiable findings remain)

Two parts:

1. **Provenance/trust policy + disclosure (committed, enforced).** A new policy
   `docs/formal/09-formal-evidence-provenance-and-trust-policy.md` states that
   receipts are **unsigned evidence — not signed proof** — and that the authority
   is re-derivation, not the JSON on disk. The coverage module
   (`crates/causlane-codegen/src/coverage.rs`) and the anti-theatre doc carry the
   disclaimer, and `tools/pre-publication-review-gate` enforces the policy exists
   (`PUB-FORMAL-PROVENANCE-POLICY`). This is the credible mitigation per the matrix
   ("public-doc downgrade of verified claims" + provenance policy).

2. **Formal toolchain installed on `ci-dispatcher.lan`** (Alloy 6.2.0 / Z3 / P
   3.1.0 / Kani 0.67.0 / Verus 0.2026.05.31 under Rust 1.95 / Lean4 4.31.0);
   `tools/formal-doctor` passes. This **enables CI re-derivation** — re-running
   `scripts/check-verification-full.sh` regenerates artifacts and rewrites receipts from real
   tool runs, so a hand-edited receipt cannot survive a fresh run.

   **Follow-up (tracked, not blocking the formal mitigation):** the z3/P
   environment caveat is resolved for the formal gate. `.devinfra/tool-versions.json`
   now pins z3 to the Verus-bundled `.tools/verus_dist/verus-x86-linux/z3`
   (4.12.5), and `tools/formal-doctor`/`scripts/check-verification-full.sh` resolve that
   configured binary instead of whichever system z3 appears first on `PATH`.
   Fresh local and `ci-dispatcher.lan` runs of
   `scripts/check-verification-full.sh --profile all --lane local_smoke` pass with derived
   coverage `status=pass`. The remaining out-of-band `cli-checker` artifact
   (empty `archive_url` in `.devinfra/tool-versions.json`) is still a devinfra
   bootstrap P0, not a formal re-derivation blocker.

Cryptographic receipt signing is deferred as future hardening. **No non-negotiable
review findings remained open.** At that point, remaining publication-track work
was M2 state retention and L-series cleanup.

## Update — 2026-06-25: H2 closed

The feature-gated in-process runtime now supervises host effect handler panics.
Both panic phases are covered: a panic before the handler returns its future and
a panic while the returned future is polled. In either case the owning partition
emits `InProcessRuntimeEvent::Failed` with `HostDispatchError::HandlerRejected`
and continues processing later independent tasks.

The implementation uses one worker-local supervision helper and reuses the
existing `Failed` event surface; no replay schema, bundle schema, generated
formal artifact or public runtime API changed. Regression tests:
`handler_panic_before_future_fails_task_and_worker_continues` and
`handler_panic_during_poll_fails_task_and_worker_continues` in
`crates/causlane-runtime/src/in_process/tests/recovery.rs`.

At that point, M2 remained open.

## Update — 2026-06-25: M1 closed

Wait-mode routed submit no longer holds route participant permits while waiting
for bounded primary ingress capacity. The runtime validates the route first,
reserves a primary ingress slot, and only then acquires route permits before
sending through the reservation. Fail-fast submit keeps its immediate
route-busy/queue-full behavior.

The implementation reuses one coordinator route lookup helper for validation,
blocking acquisition and fail-fast acquisition; it does not add a second
admission authority or a new public runtime API. Regression test:
`routed_wait_does_not_hold_participant_while_primary_ingress_is_full` in
`crates/causlane-runtime/src/in_process/tests/recovery.rs`.

At that point, M2 remained open.

## Update — 2026-06-25: M2 closed

The feature-gated in-process runtime now bounds partition-local history state.
`InProcessRuntimeConfig` exposes `partition_history_bound`, and each partition
uses one bounded FIFO set helper for completed task ids, failed task ids and
idempotency keys.

Within the retention window, duplicate suppression and dependency readiness keep
their existing semantics. After eviction, old completions no longer satisfy new
dependencies and old idempotency keys may be reused; that is explicit ephemeral
adapter behavior, not durable host policy. Regression tests cover config
validation, idempotency eviction and completed-dependency eviction in
`crates/causlane-runtime/src/in_process/tests/retention.rs`.

No bundle schema, replay schema, Formal IR, generated artifact or receipt changed.
This closes the M-series publication correctness blocker, but crates.io upload
and public GitHub baseline opening remain blocked until PUB0-PUB4 complete.
