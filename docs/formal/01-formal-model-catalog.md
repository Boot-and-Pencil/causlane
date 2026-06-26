# Complete formal model catalog

> **Repository integration status:** proposed lifecycle discipline. This
> document is design/process authority only; current proof evidence remains the
> generated chain from compiled bundle and scenario through Formal IR, generated
> artifacts, receipts, stale-check and derived coverage.

## Reading guide

A model in this catalog is authoritative only when it is backed by generated inputs and receipts. A generic model may exist as reusable support, but it must not be claimed as evidence for a bundle unless generated facts bind it to:

```text
source_bundle_hash + scenario_hash + formal_ir_hash + artifact_hash + tool_run_receipt
```

## Catalog target vs current implemented coverage

This catalog describes the **target model surface** for the product lifecycle. It
is not a statement that every listed lane is already implemented or counted for
coverage. Current implemented coverage, including Lean4 cells and theorem
`check_id`s, is the generated coverage matrix/report. Lean4 mentions on FM-* rows
are target obligations until that report contains fresh receipts for the
corresponding `check_id`s.

## Lane roles

| Lane | Role | What it is good at | What it must not claim |
|---|---|---|---|
| Replay | executable oracle over traces and bundle payloads | exact error codes, payload binding, freshness/policy checks | exhaustive interleavings |
| Alloy | relational counterexample search | structural impossibility, binding relations, small-scope counterexamples | time/freshness unless explicitly encoded |
| P | protocol/interleaving monitors | event order, concurrent interleavings, monitor firing, retry/drain behavior | deep Rust code properties |
| Kani | bounded Rust-facing checks | pure reducers, validators, bounded state transitions, panic freedom | unbounded proofs |
| Verus | code-adjacent preservation proofs | Rust-like pure kernel rules and preservation lemmas | external world/adapters without port abstraction |
| Lean4 | abstract metatheory and model adequacy | protocol calculus, refinement, compatibility, proof of proof obligations | direct claim that Rust implements it unless connected by Verus/Kani/replay |

## Model list

### FM-000 Authority-chain model

**Purpose:** prove and check that every formal claim is tied to the same source contract the runtime consumes.

**Checks:**

```text
registry content -> bundle_hash
bundle + scenario -> formal_ir_hash
generated artifact header -> source hashes
codegen receipt -> artifact hash
tool-run receipt -> real tool result and exit code
coverage report -> derived from receipts only
docs matrix -> drift-checked from coverage report
```

**Lanes:** replay/tooling, Kani for hash/stale helpers, Lean4 for abstract provenance relation.

**Acceptance:** no lane can be marked `passed` without artifact-present `check_id` and fresh receipts.

### FM-001 Bundle canonicalization and compatibility model

**Purpose:** ensure that contract identity is deterministic and stable across machines.

**Checks:** canonical serialization version, field ordering, enum representation, hash inclusion/exclusion, schema-version compatibility, migration rules.

**Lanes:** Kani for serializer helpers where feasible, Lean4 for abstract digest-material equality, replay/tooling for golden fixtures.

**Must prove/check before code:** any bundle schema change must include migration/refinement obligations and new golden hashes.

### FM-002 Formal IR lowering model

**Purpose:** prove that Formal IR is a faithful projection of compiled bundle + scenario facts and does not invent hidden facts.

**Checks:** every IR field has a source field; every modeled fact has source provenance; unknown invariant IDs fail; absent payload cannot be silently assumed.

**Lanes:** Kani for builder totality/error paths, Lean4 for lowering soundness theorem, generators for artifact-present obligations.

**Acceptance:** generated artifacts cannot refer to a fact not present in IR, and IR cannot contain a fact not present in bundle/scenario.

### FM-003 Scenario and negative-control model

**Purpose:** ensure the scenario catalog discriminates bad worlds from good worlds.

**Checks:** success scenarios pass; invalid scenarios fail with exact stable error codes; model negative controls are refuted by Alloy/P where applicable.

**Lanes:** replay primary, Alloy/P negative controls, coverage derivation.

**Acceptance:** a safety invariant is not covered unless at least one negative control can fail the gate when the corresponding check is removed or weakened.

### FM-004 Dispatch lifecycle grammar model

**Purpose:** model the event grammar for execution-bearing, projection-only and meta predicates.

**Checks:** prior barrier before execution, prior execution before observed truth, observed-truth anchor before projection, terminal close, lifecycle-class/profile permissions.

**Lanes:** replay, Alloy, P, Kani, Verus, Lean4.

**Known invariant mapping:** I-001, I-002, I-003, I-008.

### FM-005 Barrier and execution capability model

**Purpose:** make hard execution impossible without a durable barrier-derived capability.

**Checks:** barrier binds action, plan, op indexes, impact set, witnesses, leases and authz refs; capability has canonical id, derives from barrier, covers op, cannot be forged/relabelled.

**Lanes:** replay, Alloy/P structural binding, Kani/Verus for `ExecutionCapability` rules, Lean4 for abstract capability soundness.

**Acceptance:** `execution.started` without capability, with forged capability, wrong barrier, wrong plan/op or missing lease is refuted.

### FM-006 Witness and anchor attestation model

**Purpose:** prevent self-asserted witness/anchor references from manufacturing facts.

**Checks:** referenced event exists and is prior; event kind/fact kind/scope match producer attestation; witness binds exact action/plan/impact; projection anchor binds observed-truth fact/scope.

**Lanes:** replay primary, Alloy/P producer-grounding, Kani/Verus resolver checks, Lean4 exact-binding theorem.

**Known invariant mapping:** I-003, I-009.

### FM-007 Authorization evidence model

**Purpose:** enforce default deny and evidence-grounded authorization for authz-required stages.

**Checks:** Allow/Deny verdict, Deny wins, action/plan/predicate/stage binding, policy id/version, freshness, expiration, issued-before-barrier.

**Lanes:** replay for temporal/policy authority; Alloy/P for structural Allow binding; Kani/Verus for resolver/policy helper; Lean4 for abstract default-deny theorem.

**Acceptance:** missing, denied, expired, stale, wrong-policy or issued-after-barrier decisions fail closed.

### FM-008 Lease, conflict, merge and drain model

**Purpose:** prevent conflicting mutable frontier and unsafe drain.

**Checks:** exact resolved claim coverage; lease windows; exclusive/shared/token modes; conflict if same resource/scope and at least one exclusive; only verified merge protocol permits overlap; drain acquired only after prior overlap clears.

**Lanes:** replay, Alloy for interval-aware conflicts where modeled, P for interleavings/drain, Kani/Verus for pure conflict predicates, Lean4 for lease-conflict fail-closed, verified-merge algebra and drain-fence clearance.

**Known invariant mapping:** I-006, I-007.

### FM-009 Observed truth and projection model

**Purpose:** keep observed truth as the only source for derived projections.

**Checks:** observed truth requires prior execution; projection cannot commit truth; projection anchor is observed truth event; constraint updates do not rewrite committed truth.

**Lanes:** replay, Alloy/P for anchor order and grounding, Kani/Verus for truth-preservation predicates, Lean4 for anchor soundness and constraint-update future-only theorem applications.

**Known invariant mapping:** I-002, I-003, I-010.

### FM-010 Overlay, consequence profile and route model

**Purpose:** ensure overlays only strengthen obligations and routes cannot drift from consequence profiles.

**Checks:** obligation-set monotonicity, no overlay weakens barrier/witness/authz/anchor/claim requirements, route/profile compatibility, outside-kernel segregation.

**Lanes:** Kani/Verus for route/profile and overlay rules; Lean4 covers overlay monotonicity over the finite obligation-set rule model and route/profile compatibility from bundle predicate facts. Replay/Alloy/P are not applicable unless runtime events are added.

**Known invariant mapping:** I-004, I-005.

### FM-011 Constraint-update model

**Purpose:** make constraint updates affect only future frontier.

**Checks:** update epochs; committed truth immutable; future admissions see updated constraints; old observed truth remains anchored and unrevised.

**Lanes:** P, Kani, Verus, Lean4. Replay becomes applicable if trace schema gains constraint-update events.

**Known invariant mapping:** I-010.

### FM-012 Replay oracle soundness model

**Purpose:** ensure that replay acceptance implies the trace is valid under the bundle.

**Checks:** bundle hash strictness, route/profile drift, exact payload matching, stable error-code map, no event after close, negative-control discrimination.

**Lanes:** replay executable oracle, Kani for bounded reducers, Verus/Lean4 for soundness theorem over abstract trace.

**Acceptance:** `ReplayTrace::verify_with_bundle_strict` cannot accept a trace that violates any covered invariant.

### FM-013 Runtime adapter/port compliance model

**Purpose:** keep external adapters from weakening the kernel.

**Checks:** adapters can only execute through guarded executor; hard effect requires capability; audit append happens before execution; port failures cannot synthesize success truth.

**Lanes:** Kani/Verus for port-state helpers, integration tests, P for retry/duplicate execution interleavings.

**Acceptance:** adapter-specific proofs are not required, but adapter code must show a simulation to the guarded executor contract.

### FM-014 Idempotence, retry and duplicate execution model

**Purpose:** prevent retries from duplicating hard effects or bypassing barriers.

**Checks:** idempotency-domain binding, retry reuses or invalidates capability according to policy, duplicate `execution.started` for same action/plan/op is rejected unless explicitly idempotent.

**Lanes:** P primary, replay negative controls, Kani for bounded state map, Lean4/Verus for single-spend/capability invariants.

**Status:** M10.2 P bootstrap exists for duplicate hard execution; replay/Kani/Lean4/Verus coverage for planned retry invariants remains planned until those invariants become active.

### FM-015 Receipt, coverage and exception model

**Purpose:** prevent formal theatre in reporting.

**Checks:** tool-run receipt records real command, version, result, exit code; coverage is derived; exceptions have expiry and forbidden profiles; docs cannot overclaim.

**Lanes:** tooling, JSON schema, Kani for pure report builder, Lean4 for abstract evidence ordering if useful.

**Acceptance:** no `jq` patch may upgrade failure to pass; expired exception fails gate.

### FM-016 Migration, compatibility and deprecation model

**Purpose:** prevent schema/version changes from invalidating proofs silently.

**Checks:** versioned bundle/IR/receipt schemas; migration preserves or explicitly invalidates obligations; old receipts become stale when source hash changes.

**Lanes:** tooling, Kani, Lean4 refinement theorems.

**Acceptance:** deleting/renaming fields requires a migration record and a stale-check negative control.

### FM-017 Security attestation model

**Purpose:** keep capability/witness authenticity assumptions explicit.

**Checks:** canonical attestation message is injective at the field-boundary level; HMAC verification uses exact payload; secret material is outside formal model but assumptions are documented.

**Lanes:** Kani/Verus for message construction and validation paths; Lean4 for injective field encoding; cryptographic strength is an assumption, not proved.

## Status taxonomy

A model cell may be claimed only as one of:

```text
not_modelled
not_applicable
declared
ir_projected
generated
tool_passed
negative_control_refuted
proved_no_cheating
covered
```

`covered` is legal only when the coverage report derives it from concrete lane evidence. Human prose cannot assign `covered`.
