# Contract hardening plan (before formal models)

## Verdict

The final readiness gate is now the authority for starting real formal-model
work:

```bash
just formal-ready
```

That gate proves the contract surface is no longer just a docs/formal scaffold:
registry, compiled bundle, plan hash, replay, scenario emission, generated
Alloy facts, receipt v2 and stale-check all consume the same generated truth.
Formal models may now start as FM-001…FM-004, but they must keep the
generate-don't-maintain rule from [ADR-0006](adr/0006-formal-modeling-stack.md)
and [ADR-0014](adr/0014-generate-dont-maintain-pipeline.md).

## Decisions taken (ADR-0009…0014)

The previously open questions are now resolved and recorded as ADRs:

- [ADR-0009](adr/0009-plan-hash-canonicalization.md) — `plan_hash = sha256:hex(SHA-256(canonical PlanHashMaterial))`; explicit include/exclude lists; approvals bind to `action_id + plan_hash + impact_set_hash`.
- [ADR-0010](adr/0010-anchor-vs-witness.md) — a projection's truth anchor is a separate typed field, not a reused witness.
- [ADR-0011](adr/0011-authz-default-deny.md) — authorization is deny-by-default / fail-closed.
- [ADR-0012](adr/0012-merge-protocol-semantics.md) — default is *no* merge protocol; overlapping mutable writes conflict unless a `Verified` bundle-level protocol says otherwise.
- [ADR-0013](adr/0013-barrier-witness-lease-binding.md) — the execution barrier binds witnesses, leases and the impact-set hash; the executor runs from a capability derived from a valid barrier.
- [ADR-0014](adr/0014-generate-dont-maintain-pipeline.md) — bundle-specific formal artifacts are generated from the compiled bundle; hand-written models stay exploratory.

## Work breakdown (TZ-001…TZ-013)

Status legend: **done** = implemented + tested in this repo; **partial** =
started; **pending** = specified by an ADR but not yet built.

| TZ | Item | Priority | Status |
|---|---|---|---|
| TZ-001 | Bundle format v0.1 (`RegistryManifest` → `CompiledDispatchBundle` + `bundle_hash`) | P0 | **done** — `causlane-contracts::{registry,bundle}`, parses `release_promote.registry.yaml` |
| TZ-002 | Canonical serialization + validated `PlanHash` + `PlanHashMaterial` + `impact_set_hash` | P0 | **done** — `causlane-contracts::plan_hash`; fixtures carry real hashes; `sha256:TODO` rejected |
| TZ-003 | Event model v0.1 (`AuditEvent` + anchors + leases + correlation/causation + impact-set hash) | P0 | **done** — `causlane-core::domain::audit` |
| TZ-004 | Witness model + selector validation (`WitnessRef`, `RequiredWitnessSpec`, resolver) | P0 | **done** — typed `WitnessRef`/binding plus exact subject/circumstance template resolver; bundle replay checks prior evidence, kind, fact, scope and action/plan/impact binding |
| TZ-005 | Anchor model (`TruthAnchor`, projection policy) | P0 | **done** — `TruthAnchor` in core; replay uses anchors, not `witnesses.is_empty()` |
| TZ-006 | Lease + barrier mechanics (`LeaseRef`, epoch, `ExecutionBarrier`, capability) | P0 | **done for MVP** — typed `ExecutionBarrier`, `ExecutionCapability`, `LeaseTable`, lease grant/release and barrier claim coverage checks landed |
| TZ-007 | Merge protocol v0.1 (`MergeProtocolSpec`, `mergeable()`, default none) | P0 (for I-006) | **partial** — `MergeProtocolSpec`/status/algebra in bundle schema; default-none `mergeable() == false` and conflict replay enforced; verified merge protocols pending |
| TZ-008 | Authz default policy (deny-by-default, `AuthzPolicy`/`AuthzMode`) | P0 | **done** — registry/bundle carry explicit `authz_policy`; bundle replay validates typed allow/deny authz evidence post-hoc; live enforcement via core `authz_gate` + `causlane-runtime::authz::AuthzGuard`, wired into `guarded_executor::GuardedExecutor::spend_barrier` (authorizes before deriving/spending a capability; a test proves a barrier cannot be spent without authorization) |
| TZ-009 | Replay verifier hardening (per `action_id+plan_hash`; I-001/I-002/I-003/I-006/I-008; JSON loader) | P0 | **done** — structural replay plus bundle-bound lifecycle, barrier payload, witness, lease, authz-ref and claim-coverage checks |
| TZ-010 | Formal codegen path (`causlane-codegen`, bundle → Alloy/P/Kani/Verus/Lean4, receipts) | P0 (before models) | **done** — generated Alloy/P/Kani/Verus/Lean4 artifacts and receipts exist; all five lanes run (always-on, blocking) in `formal-verify-all` (the Verus/Lean4 non-blocking exceptions were dropped 2026-06-21) |
| TZ-011 | Scenario catalog v0.1 (`*.scenario.yaml`) | P1 | **done** — `contracts/scenarios/*.scenario.yaml`, JSON Schema, `scenario emit-trace`, replay tests and CLI E2E landed |
| TZ-012 | Type hardening (typed ids/newtypes over stringly fields) | P1 | **partial** — added `LifecycleClass`, `RouteId`, `FactKind`, `BundleHash`, `ContentHash`, `ImpactSetHash`, `EventHash`, `ConstraintEpoch`, `WitnessRef`, `ExecutionBarrier`, `ExecutionCapability`, `CapabilityId`; further hardening ongoing |
| TZ-013 | Docs/ADR updates | P0 | **done** — ADR-0009…0014; updated docs 05/07/10, glossary, coverage matrix, formal READMEs |

## Readiness gate (before deeper formal hardening)

Begin real formal modeling only when **all** of these hold:

```text
[x] Bundle Format v0.1 exists.
[x] release_promote.registry.yaml compiles into a bundle.
[x] bundle_hash is computed.
[x] PlanHash is computed; fixtures carry no sha256:TODO.
[x] Event model carries witnesses, anchors and leases.
[x] Replay has a JSON loader.
[x] Replay validates at least I-001/I-002/I-003/I-008 (also I-006).
[x] Authz default is fixed as deny.
[x] MergeProtocol::None and conflict-by-default are specified.
[x] Barrier binds witness refs + lease refs + impact_set_hash (TZ-004/TZ-006 MVP).
[x] Formal generated output path exists for Alloy and scenario-bound receipts; generated scenario facts open the core model in smoke.
[x] Scenario catalog emits replay traces and is regression-tested.
[x] Coverage matrix updated: generated input listed per row.
```

Items still blocking deeper model work: strengthening the generic Alloy model
beyond the MVP generated checks, runtime/provider authz enforcement, verified
merge protocols and optional Verus proof hardening. Keep bundle-specific facts
and generated P/Kani artifacts bound to Formal IR.

## Priority order for the remaining work

```text
1. Strengthen the Alloy core and generated checks beyond the MVP (B-013).
2. Authz runtime/provider deny-by-default enforcement (TZ-008).
3. Verified merge protocols beyond default `mergeable() == false` (TZ-007).
4. Broaden generated P/Kani harnesses beyond the current release-promote slice.
5. Optional Verus proof profile hardening (FM-004).
```

## What already works end-to-end

```text
release_promote.registry.yaml
  -> RegistryManifest::from_yaml_str
  -> CompiledDispatchBundle::compile  (bundle_hash)
  -> causlane bundle compile --registry ... --out ...bundle.json
  -> PlanHashMaterial::compute_plan_hash  (plan_hash, impact_set_hash)
  -> release_promote.trace.json
  -> ReplayTrace::from_json_str -> verify / verify_with_bundle
  -> release_promote_success.scenario.yaml
  -> causlane scenario emit-trace --scenario ... --out ...trace.json
  -> causlane replay verify --bundle ... --trace ...
  -> causlane formal generate alloy --bundle ... --scenario ... --out ... --receipt ...
  -> causlane formal stale-check --bundle ... --scenario ... --generated ... --receipt ...
```

Exercised by the `causlane-contracts` and `causlane-replay` test suites and by
`causlane bundle validate`, `causlane bundle compile`, `causlane scenario
emit-trace`, `causlane replay verify` and `causlane formal
generate/stale-check`.

## Formal readiness (P0) — current status

Per the formal-models TZ, the P0 readiness stage is partly landed:

| Item | Status |
|---|---|
| P0-001 `formal doctor` | **done** — `causlane formal doctor [--json] [--require ...]` (`causlane-formal` crate, pure; CLI gathers env) + `just formal-doctor`/`-json`/`-smoke` |
| P0-002 toolchain source of truth | **done** — `rust-toolchain.toml` canonical (stable + components); `tool-versions.json` records observed 1.93.1 |
| Tool install | **done** — z3 4.12.5 from the pinned Verus distribution, Alloy 6.2.0 (jar), P 3.1.0, cargo-kani 0.67.0 installed; Verus downloaded (needs Rust 1.95.0 toolchain); `tool-versions.json` `formal_tools` enabled + pinned |
| FM-001 Alloy lane | **real** — headless `AlloyRunner.java` (Alloy API + SAT4J) runs `formal/alloy/core/causlane_core.als`; I-001/I-002/I-003 hold, a negative control is correctly refuted, receipt written |
| P0-003 bundle → formal input v0.3 | **done** — `BundleBody` v3 carries route, barrier/projection/authz/truth/constraint policies, selector schemas, claim templates, scenario refs and formal obligations |
| P0-004 typed `WitnessRef` + resolver | **done** — typed refs + exact selector resolution for kind/fact/scope/action/plan/impact |
| P0-005 typed `ExecutionBarrier` + capability + trace JSON | **done** — core structs, `ExecutionCapability::derive_from_barrier`, executor port capability and normalized trace JSON |
| P0-006 `verify_with_bundle` | **done** — predicate existence, lifecycle, RuntimeExecution dispatch/barrier payload, typed witnesses, leases, typed authz decisions, capability and claim coverage |
| P0-007 `AuthzPolicy` types + deny-by-default enforcement | **done** — manifest/bundle types + replay (post-hoc) enforcement; live enforcement via the pure core `authz_gate` (ADR-0011, fail-closed) wrapped by `causlane-runtime`'s `AuthzGuard::authorize_barrier` and wired into `GuardedExecutor::spend_barrier`, which authorizes before deriving/spending the capability (missing/denied/wrong-binding/expired all refuse; a runtime test proves no op runs under an unauthorized barrier) |
| P0-008 `mergeable()` predicate | **done (replay path)** — the replay oracle resolves merge **per-protocol**: a verified `MergeProtocolSpec` applicable to an effect's `op_kind` makes that effect's conflict-domain scopes mergeable (`resolve_mergeable_scopes` → `LeaseTable::with_mergeable_scopes`), relaxing the I-006 exclusive-lease conflict; fail-closed when no verified protocol applies. Core `mergeable()` remains the global fail-closed default for callers without a bundle. Runtime/provider enforcement still pending |
| P0-009 scenario catalog (`*.scenario.yaml`) | **done** — catalog, schema, trace emitter and replay tests |
| P0-010 `causlane-codegen` (bundle → Alloy/P/Kani/Verus/Lean4) | **done** — Alloy/P/Kani/Verus/Lean4 artifacts generated from Formal IR; all five lanes run (always-on, blocking) in `formal-verify-all` (Verus/Lean4 non-blocking exceptions dropped 2026-06-21) |
| FM-002 P / FM-003 Kani / FM-004 Verus | **partial** — P and Kani executable lanes exist for the current generated slice; Verus remains optional proof hardening |
| FM-005 stale-check + receipts | **done** — `formal generate alloy --receipt` writes receipt schema v2 and `formal stale-check` validates bundle/artifact/scenario hashes |

Next, in TZ order: B-013 strengthened Alloy checks → runtime/provider authz →
verified merge protocols → broader P/Kani generated lanes → optional Verus
proof hardening.

## dispatcher-003 stage — progress

Tracking the dispatcher-003 TZ (bootstrap/toolchain reliability + contract
enrichment toward bundle-bound formal input). **done** this iteration:

| Item | Status |
|---|---|
| T-001 / B-001 bootstrap doctor (no Rust) | **done** — `tools/formal-doctor [--json] [--require ...]`; reports missing/disabled/version_mismatch/sha_mismatch/ok; works without cargo/rustc; `just formal-doctor-bootstrap` |
| T-002 tool provisioning | **done** — `tools/formal-install <alloy\|verus\|z3\|p\|kani\|all>`, SHA-verified against `tool-versions.json`, installs into `.tools/` (git-ignored) |
| B-002 tool source-of-truth | **done** — Alloy jar SHA pinned; z3 added; Verus archive URL+SHA+version pinned (`enabled:false` until its Rust 1.95.0 toolchain) |
| B-005 / T-003 bundle_id in manifest | **done** — `RegistryManifest.bundle_id`; `compile(manifest)` uses it; `compile_with_bundle_id` is tests/dev override; CLI no longer injects `causlane.cli`; pinned plan hash unchanged |
| B-014 Alloy file dedup | **done** — exploratory moved to `formal/alloy/sketches/`; `alloy/core` is a generic support model, while compiled bundle + generated facts + receipts are authoritative |
| B-015 receipts policy | **done** — live receipts git-ignored; committed sample at `formal/receipts/examples/alloy-core.sample.json` |
| B-017 canonical serialization policy | **done** — Canonical Serialization v1 is specified and used by bundle/formal IR hash material |

**done/partial** this iteration: T-004/B-004 `BundleBody` v2 enrichment;
T-005 validation for lifecycle/profile, RuntimeExecution barrier/truth/claims,
schema hash shape and local-dev authz exemption; T-006/B-006 typed `WitnessRef`
and selector/binding checks; T-007/B-007 `ExecutionBarrier` +
`ExecutionCapability`; T-008/B-009/B-010 `LeaseTable` + default
`mergeable() == false`; T-009/B-008 bundle-bound `verify_with_bundle`;
T-010/B-012 lifecycle replay; T-011/B-016 scenario catalog and trace emitter;
T-012/T-013 `causlane-codegen` Alloy facts from compiled bundle; T-014
stale-check for bundle/artifact/scenario hashes + optional receipt.

**pending** (next, in TZ priority order): B-013 strengthened Alloy core;
runtime/provider authz enforcement; verified merge protocols beyond
default-none; then broader FM P/Kani lanes and optional Verus proof hardening.

## Trait contract surface (§7)

§7 names the dispatcher's pure invariants as explicit, testable trait
contracts so the formal lanes (Alloy/P/Kani/Verus) and the executable replay
oracle verify *the same* contract the runtime enforces, rather than parallel
re-implementations. All underlying logic was already pure (no I/O below the CLI
boundary), so the work is *naming + single-authority routing + acceptance
tests*, not an I/O refactor.

The **verification-critical kernel contracts** are implemented and load-bearing:

| § | Trait(s) | Crate / module | Canonical impl | Adoption |
|---|----------|----------------|----------------|----------|
| 7.5 | `LifecycleGrammar` (`initial_stage`/`reduce`/`is_terminal`) | `causlane-core::contract` | `KernelContracts` → `reduce_lifecycle` | **replay** `validate_lifecycle` reduces through it |
| 7.7 | `ScopeOverlap` (equality-MVP), `ConflictOracle` (`claims_conflict`/`leases_conflict`), `DrainSemantics` (`can_acquire_fence`, active-interval) | `causlane-core::contract` | `KernelContracts` → `claim_modes_conflict` | shares the `claim_modes_conflict` primitive with `LeaseTable` |
| 7.7 | `MergeSemantics` (`merge_decision`) + `MergeDecision::permits_concurrency` | `causlane-contracts::bundle` | `KernelMergeSemantics` → `merge_decision` | **replay** `resolve_mergeable_scopes` resolves merge through it (feeds `ConflictOracle`'s `verified_merge`) |
| 7.8 | `CapabilityIssuer` (`derive_capability`/`validate_capability`) | `causlane-core::contract` | `KernelContracts` → `ExecutionCapability::{derive_from_barrier,validate_for_barrier}` | **replay** capability checks derive/validate through it |
| 7.9 | `TruthAnchorResolver` (`anchor_source_is_valid`/`anchor_matches`) | `causlane-core::contract` | `KernelContracts` → `projection_anchor_source_is_observed` | kernel function |

Each carries acceptance tests mirroring the §7 bullets (closed-is-terminal,
observe-without-execution forbidden, equality-only scope overlap, merge default
no, drain ignores expired leases, forged capability refused, only observed-truth
is a valid anchor source).

The **boundary & pipeline contracts** are also implemented, each in a new
`contract` module delegating to existing pure functions (single source of truth,
no logic duplication):

| § | Trait(s) | Crate / module | Canonical impl | Adoption |
|---|----------|----------------|----------------|----------|
| 7.1 | `CanonicalSerialize`, `StableDigest` + `POLICY_VERSION` | `causlane-contracts::contract` | `BoundaryContracts` → `canonical_json_bytes`/`byte_hash` | surface only (hashing is called deep inside `compile`/plan-hash; routing through it would invert) |
| 7.2 | `BundleCompiler`, `BundleValidator` | `causlane-contracts::contract` | `BoundaryContracts` → `CompiledDispatchBundle::compile` | **cli** `compile_registry` compiles bundles through it; validation = "compiles cleanly" |
| 7.3 | `TemplateResolver` | `causlane-contracts::contract` | `BoundaryContracts` → `resolve_template`/`validate_template_expression` | **replay** `resolve_scope` resolves through it (exact-only, fail-closed) |
| 7.4 | `ReplayOracle` | `causlane-replay::contract` | `ReplayContracts` → `ReplayTrace::verify_verdict` | verdict surface; the CLI replay path uses the typed-error `verify_with_bundle` |
| 7.6 | `AuthzEvidenceVerifier` + `AuthzEvidenceVerdict` | `causlane-replay::contract` | `ReplayContracts` → `validate_authz_refs` (now `pub(crate)`) | verdict surface; replay uses `validate_authz_refs` directly to keep the granular deny/missing/expired codes |
| 7.10 | `FormalIrBuilder`, `FormalGenerator` (`PGenerator`/`KaniGenerator`/`VerusGenerator`) | `causlane-codegen::contract` | `CodegenContracts` → `build_formal_ir` + per-target generators | **cli** builds the IR + P/Kani/Verus artifacts through it (Alloy excluded — bundle+scenario shape) |
| 7.11 | `StaleChecker`, `CoverageReporter` | `causlane-codegen::contract` | `CodegenContracts` → `stale_check_with_expected` + `build_report` | **cli** stale-checks + derives coverage through it (never greens `fail`→`pass`) |

Each carries acceptance tests (deterministic canonical bytes, compile-iff-valid,
exact template resolution + fail-closed, oracle accepts the example trace, authz
deny-by-default, per-target generator binding, stale-check detects bundle drift).

**Adoption (§5 follow-up — now done where layering is clean):** the consumer
layers (replay, CLI) route through the named authorities — `LifecycleGrammar`,
`CapabilityIssuer`, `MergeSemantics`, `TemplateResolver` (replay) and
`BundleCompiler`, `FormalIrBuilder`, `FormalGenerator`, `StaleChecker`,
`CoverageReporter` (CLI). The remainder are deliberately *surface only*:
`ScopeOverlap`/`ConflictOracle`/`DrainSemantics` (routing core-domain through
`core::contract` would invert layering — they instead share the
`claim_modes_conflict` primitive), `CanonicalSerialize`/`StableDigest` (called
inside the hashing internals), and the verdict-form `ReplayOracle`/
`AuthzEvidenceVerifier` (the replay path keeps the typed-error APIs so granular
stable error codes survive).

**Genuinely out of scope** (the I/O layer the §7 split pushes to runners/CLI, not
a pure contract): 7.10 `FormalToolRunner` (invokes Java/P/Kani/Verus — lives in
the shell gate `tools/formal-verify-all`) and 7.11 `ReceiptWriter` (filesystem
writes — the CLI owns them; codegen only builds receipt values).
