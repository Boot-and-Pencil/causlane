# Formal models

The formal contour attacks the dispatcher contract with five implemented lanes
(Alloy, P, Kani, Verus, Lean4). The long-term rule
(ADR-0006 / ADR-0014 / proposed ADR-0015) stands:

```text
Do not maintain formal models as a second truth.
Generate or bind them to the compiled dispatch bundle.
```

## Status (lane reality — P1-FM-013)

Lanes are rated against an honest ladder, not a binary "real/not". A higher rung
implies the lower ones:

`present` (generated from the bundle) → `compiled` (the tool accepts it) →
`checked` (the tool runs a real check) → `payload-bound` (facts discriminate
action/plan/scope/witness/authz/lease, not just event-kind order) →
`discriminating` (refutes the negative-control catalogue) →
`authoritative` (sole sufficient proof of an invariant).

| Lane  | Rung | Reality |
|-------|------|---------|
| **replay** (executable oracle) | `discriminating` | Strongest lane. Bundle-bound trace verification; payload-bound (barrier/capability/witness/authz/lease DTOs); refutes 16 main negative controls plus 6 authz controls with exact stable error codes. The coverage report leans on it. |
| Alloy | `discriminating` for I-001 + I-003 + I-006 + I-009; `checked` elsewhere | Core model + generated scenario facts run headlessly via `AlloyRunner`. **Consumes the capability/barrier/witness/anchor/lease/authz payloads**: `GeneratedCapabilityBindsToBarrier` (I-001 — a capability derives from a barrier for the same action+plan covering its op); `GeneratedApprovalBindingHolds` (I-009 — a gate-approval witness binds to the barrier's action **+ plan_hash** + impact set); `GeneratedWitnessFactGrounded` / `GeneratedAnchorFactGrounded` (producer-attested fact **+ scope**, not self-asserted refs); and `GeneratedNoExclusiveConflicts` (I-006 — interval-aware active leases). The gate runs nine main Alloy negative controls, including wrong witness/anchor fact and scope controls, plus two authz structural controls. |
| P     | `discriminating` for I-001 + producer/authz grounding; `checked` elsewhere | Events carry a typed `EventPayload` projected from the IR: action/plan/op/barrier, impact hash, producer fact/scope, witness/anchor claimed fact/scope, witness action/plan/impact binding, authz refs/stage, lease epoch and execution key. `CapabilityBindsToBarrier` checks capability-to-barrier payloads; `WitnessFactGrounded` checks producer-attested fact/scope and exact witness action/plan/impact binding; `AnchorFactGrounded` checks observed-truth fact/scope grounding; `AuthzDecisionGroundsBarrier` rejects a referenced Deny. M10.2 adds P-only planned-invariant controls for duplicate retry execution, authz revocation before barrier and stale constraint-epoch admission; they run in `check-verification-full` but do not add active coverage credit. |
| Kani  | `bounded nondet` / `checked` per generated coverage row | Generated harnesses (now in `kani_target.rs`) run under `cargo-kani` against the **real** core validators. `verification/formal-full/kani/profile.json` is the machine-readable runner profile for fixture stem, package name, output format and lane-specific unwind bounds consumed by `check-verification-full`. Current per-invariant cells and concrete harness `check_id`s are generated in [`docs/invariants/coverage-matrix.md`](../docs/invariants/coverage-matrix.md); this table describes lane role only. The bounded harnesses are mutation-sensitive over representative core rules such as capability binding, lifecycle predecessors, overlay monotonicity, route/profile compatibility, lease conflict, drain fence and constraint-update truth preservation. The remaining harnesses are fixed-example (P0-FM-006 backlog). |
| Verus | `checked` (non-vacuous) + **scenario-bound** | Generated proof is an **event-indexed transition**: `step(s, e)` computes the next state and does **not** assume the result is valid, so `step_preserves_validity` (`valid_state(s) ∧ event_allowed(s,e) ⟹ valid_state(step(s,e))`) is a real obligation, not the earlier vacuous skeleton where `transition` assumed `valid_state(s1)`. **Scenario-bound**: on top of the kernel theorems, the generator emits a concrete `generated_scenario_is_valid_trace` lemma that folds `step` over **this bundle's scenario events** (in order, from the Formal IR) and proves `valid_state` is preserved at every step — so a different scenario changes the obligation. Rule lemmas now include I-005 route/profile compatibility over the same profile-to-lifecycle-class mapping Kani drives, and I-007 drain safety over the same two-slot `DrainFenceCheck` shape. `verus --no-cheating` reports **17 verified, 0 errors** under the proof/all profile. Still abstract on the *values* (it models the protocol-critical event order and rule lemmas, not every concrete lease id/hash). Always run and blocking on every `check-verification-full` run (the rust/base/ci non-blocking exception was dropped 2026-06-21). |
| Lean4 | `compiled` + **scenario-bound** theorem applications | `FormalTarget::Lean4` emits bundle/scenario-bound theorem applications from Formal IR into `verification/formal-full/lean4/generated/*.lean`, imported through the repo-local `verification/formal-full/lean` Lake package. Current covered cells and named theorem `check_id`s are generated in [`docs/invariants/coverage-matrix.md`](../docs/invariants/coverage-matrix.md) from fresh receipts. The lane is receipt-bound, stale-check-bound, covered by a no-`sorry`/no-`axiom` check, and runs with pinned repo-local `elan`/`lean`/`lake` on every `check-verification-full` run; the base/rust/ci non-blocking exception was dropped 2026-06-21. |

So the honest one-line status is: **the formal contour is bundle-bound,
payload-bound for the replay oracle and for the generated lane checks that claim
payload-sensitive invariants, receipt-bound, stale-check-bound and
acceptance-gated; Alloy/P/Kani/Verus/Lean4 are all checked through real tool runs
on every gate run.**

The current proof/refinement classification (what is proved, bounded,
simulated, tested, assumed or out of scope) is generated from
[`docs/formal/proof-refinement-scope.json`](../docs/formal/proof-refinement-scope.json)
into
[`docs/formal/08-proof-refinement-scope.md`](../docs/formal/08-proof-refinement-scope.md).
It does not grant coverage credit; the coverage matrix remains the authority for
per-invariant cells.

`just verification-full` is the default cross-target implementation gate. It
compiles the bundle, emits Formal IR and generated Alloy/P/Kani/Verus/Lean4
artifacts, runs the required Alloy/P/Kani/Verus/Lean4 lanes recording each lane's
**real exit code**, executes replay-backed negative
controls, and **derives** the coverage report from the tool-run receipts (it is
never patched to `pass` — P0-FM-002). Use `scripts/check-verification-full.sh --profile
proof` / `--profile all` to run Verus and Lean4.

## Layout

```text
verification/formal-full/
  tools/AlloyRunner.java                 headless Alloy runner (Alloy API + SAT4J)
  alloy/
    core/causlane_core.als               generic protocol core (not authority by itself)
    checks/unconstrained_counterexample.als  negative control (must be refuted)
    generated/                           bundle-generated facts from compiled bundles
    sketches/causlane_core_sketch.als    non-authoritative exploratory sketch (never run by smoke)
  p/        core/, generated/            P machines and generated monitors
  kani/     manual/, generated/          bounded harnesses
  verus/    core/, generated/            kernel proofs
  lean/                                Lean package/core for Lean4 lane
  lean4/    generated/                 generated Lean theorem applications
  obligations/                         proposed lifecycle obligation manifest
  receipts/                              live run receipts (git-ignored)
  receipts/examples/alloy-core.sample.json   committed sample receipt (schema reference)
  smoke.sh                               compile runner, run core + negative, write receipt
```

Authority: the compiled bundle plus generated facts, receipt v2 and
`formal stale-check` form the checked contract surface. `alloy/core/*.als` is a
generic protocol core; it is useful only when opened by generated facts or run as
a smoke-control model. `alloy/sketches/*` are exploratory and are never run by
`smoke.sh`; `formal stale-check` ignores sketches and fails on stale generated
artifacts.

The lifecycle discipline is documented under `docs/formal/`.
`verification/formal-full/obligations/lifecycle_product_obligations.yaml` and
`docs/formal/formal_model_catalog.yaml` feed `tools/formal-discipline-check`,
which is run by `scripts/check-verification-full.sh` after fresh coverage and
coverage-matrix drift checking. They extend the generated evidence chain rather
than replacing it.

## Toolchain bootstrap (no Rust required)

```bash
tools/formal-doctor --profile base --lane local_smoke
tools/formal-doctor --json
tools/formal-doctor --profile proof        # explicit opt-in to Verus/z3/Lean4
tools/formal-install alloy                 # provision .tools/alloy/alloy.jar (SHA-verified)
just formal-doctor-bootstrap               # same doctor, via just
```

`tools/formal-doctor` distinguishes **missing** (binary absent), **disabled**
(pinned but intentionally turned off), **version_mismatch**, **sha_mismatch**,
and **ok**, using
`.devinfra/tool-versions.json` as the source of truth. It reports missing
cargo/rustc as a state (it never needs Rust itself).

## Running the formal gates

```bash
causlane formal doctor [--json] [--profile ...] [--lane ...] [--require ...]
just formal-doctor
just formal-smoke                                 # compile runner + run Alloy + write receipt
just formal-ready                                 # full readiness gate
just verification-full                            # default gate; runs every lane incl. Verus (no-cheating) + Lean4 (Lake/lean), always-on blocking
just verification-full fast_ci --dry-run         # provider-neutral lane entrypoint; lanes/unwinds come from verification/formal-full/kani/profile.json
```

`just formal-smoke` succeeds only if the core model holds, the generated
`release_promote_success` facts hold, and both hand-written and generated
negative controls are correctly refuted — so a vacuously-passing model fails the
smoke. It writes live receipts under `verification/formal-full/receipts/` (git-ignored), including
the core Alloy receipt and codegen receipt; see
`verification/formal-full/receipts/examples/alloy-core.sample.json` for the schema.

The Alloy jar (`.tools/alloy/alloy.jar`) is a downloaded artifact (git-ignored);
both doctors report whether it is present and SHA-correct.

## Lean4 quick path

Lean4 is generated/bound like the other targets and is **always run and blocking**
in every `check-verification-full` run (the base/rust/ci non-blocking exception was
dropped 2026-06-21).

Install and verify the repo-local toolchain:

```bash
tools/formal-install lean4
tools/lean4-env lean --version
tools/lean4-env lake --version
```

Run the package and generated scenario theorem applications directly:

```bash
(cd verification/formal-full/lean && ../../../tools/lean4-env lake build CauslaneFormal)
(cd verification/formal-full/lean && ../../../tools/lean4-env lake env lean ../lean4/generated/release_promote_success.lean)
```

Regenerate and stale-check through the CLI:

```bash
causlane formal generate lean4   --bundle target/causlane/release_promote.bundle.json   --scenario contracts/scenarios/release_promote_success.scenario.yaml   --out verification/formal-full/lean4/generated/release_promote_success.lean   --receipt verification/formal-full/receipts/release_promote_success.lean4.codegen.json

causlane formal stale-check-all   --bundle target/causlane/release_promote.bundle.json   --scenario contracts/scenarios/release_promote_success.scenario.yaml
```

The full proof profile runs the same Lake build + generated Lean check inside:

```bash
scripts/check-verification-full.sh --profile proof
```

Current Lean4 coverage is the generated Lean4 row/cell set in
[`docs/invariants/coverage-matrix.md`](../docs/invariants/coverage-matrix.md)
and the backing coverage report. Broader Lean4 entries in the catalogs are
target-state obligations, not current coverage until they have fresh receipts
and coverage rows.

## Bundle-generated facts

```bash
causlane bundle compile --registry contracts/examples/release_promote.registry.yaml --out target/causlane/release_promote.bundle.json
causlane replay verify --bundle target/causlane/release_promote.bundle.json --trace contracts/examples/release_promote.trace.json
causlane scenario emit-trace --scenario contracts/scenarios/release_promote_success.scenario.yaml --out target/causlane/release_promote.scenario.trace.json
causlane replay verify --bundle target/causlane/release_promote.bundle.json --trace target/causlane/release_promote.scenario.trace.json
causlane formal generate alloy --bundle target/causlane/release_promote.bundle.json --scenario contracts/scenarios/release_promote_success.scenario.yaml --out verification/formal-full/alloy/generated/release_promote.als --receipt verification/formal-full/receipts/release_promote.codegen.json
causlane formal stale-check --bundle target/causlane/release_promote.bundle.json --scenario contracts/scenarios/release_promote_success.scenario.yaml --generated verification/formal-full/alloy/generated/release_promote.als --receipt verification/formal-full/receipts/release_promote.codegen.json
```

Generated files carry `source_bundle_hash`, `formal_ir_hash`,
`generator_version`, `target`, `artifact_kind`, `scenario_hash` and
`invariant_ids` headers. `formal stale-check` validates the bundle hash,
generated artifact hash and, when `--scenario` is supplied, the scenario hash in
both the generated header and receipt. Codegen receipts use schema v2 and bind
source bundle, Formal IR, scenario, artifact, generator/tool versions and
invariant ids. Receipts are live artifacts and stay git-ignored.

The cross-target acceptance gate is:

```bash
just verification-full
```
