# Lean4 and Verus proof obligations

> **Repository integration status:** proposed lifecycle discipline. This
> document is design/process authority only; current proof evidence remains the
> generated chain from compiled bundle and scenario through Formal IR, generated
> artifacts, receipts, stale-check and derived coverage.

## Separation of responsibilities

Lean4 and Verus must not prove the same thing in two disconnected universes and then be counted twice. Their roles are different:

```text
Lean4 = abstract protocol semantics, contract algebra, model adequacy/refinement statements.
Verus = code-adjacent preservation proofs over Rust-like kernel rules and pure validators.
Kani  = bounded executable confirmation that Rust code paths match the intended predicates.
Replay = executable oracle over concrete bundle/trace payloads.
```

The generated proof/refinement scope
([`08-proof-refinement-scope.md`](08-proof-refinement-scope.md)) is the compact
reader-facing classification of what is proved, bounded, simulated, tested,
assumed or out of scope. This file remains the obligation/catalog detail behind
that generated summary.

A Lean theorem is evidence for Causlane only when either:

1. it is a generic theorem about the protocol calculus and generated instance facts instantiate it; or
2. it is connected to code by a Verus/Kani/replay obligation recorded in the coverage report.

A Verus proof is evidence only when run with `verus --no-cheating` under the always-blocking proof-lane contract. Future non-authoritative proof lanes must be recorded in `verification/formal-full/proof-lanes.json` and governed by `docs/formal-exceptions.json` before they can be skipped.

## Lean4 lane: operational scope

Lean4 is introduced as `FormalTarget::Lean4` with generated theorem
applications over Formal IR. It is **always run and blocking** for the obligations
currently emitted by the generator: `scripts/check-verification-full.sh` runs it on every run
(the base/rust/ci non-blocking exception was dropped 2026-06-21).

Generated layout:

```text
verification/formal-full/lean/CauslaneFormal/*.lean # generic reusable protocol calculus; not bundle authority alone
verification/formal-full/lean4/generated/*.lean     # generated bundle/scenario theorem applications
verification/formal-full/receipts/*.lean4.codegen.json
verification/formal-full/receipts/*.lean4.tool-run.json
```

Tooling:

```text
elan/lake/lean pinned in .devinfra/tool-versions.json
tools/formal-install lean4
(cd verification/formal-full/lean && ../../../tools/lean4-env lake build CauslaneFormal)
(cd verification/formal-full/lean && ../../../tools/lean4-env lake env lean ../lean4/generated/<scenario>.lean)
causlane formal generate lean4 --bundle ... --scenario ...
causlane formal stale-check-all includes Lean4 artifacts
```

## Lean4 theorem catalog

### L-001 Trace prefix order

**Statement:** if event `b` depends on event `a`, then any valid trace places `a` before `b`.

**Used by:** lifecycle, witness, anchor, authz, lease, drain.

### L-002 Barrier before execution

**Statement:** for any valid execution-bearing trace, `execution.started(op)` implies a prior `execution.barrier_logged` covering the same action, plan and op.

**Invariant:** I-001.

### L-003 Observed truth after execution

**Statement:** `observed_truth.committed` implies prior `execution.completed` for the same action/plan/op or a stricter execution-completed predicate if the protocol distinguishes start/completion.

**Invariant:** I-002.

### L-004 Projection anchor soundness

**Statement:** `projection.emitted` with anchor `a` is valid only if `a` is a prior observed-truth event and the claimed fact kind/scope equals the observed event's attestation.

**Invariant:** I-003 and I-009.

### L-005 Terminal close

**Statement:** once `lifecycle.closed` appears in a valid trace, no later event can mutate lifecycle state, spend capability, grant lease coverage, commit truth or emit projection for the closed action unless a future protocol explicitly introduces a reopen event and proves a refinement.

**Invariant:** I-008.

### L-006 Witness exact binding

**Statement:** a witness satisfying a requirement for action `A`, plan `P`, impact set `I` must carry exactly those bindings; changing any field invalidates satisfaction.

**Invariant:** I-009.

### L-007 Authz default-deny and Deny-wins

**Statement:** in an authz-required stage, absence of a fresh bound Allow or presence of a referenced Deny prevents a valid RuntimeExecution barrier.

**Invariant:** I-009 / authz slice.

### L-008 Lease conflict fail-closed

**Statement:** two overlapping claims on the same resource and scope conflict if either is exclusive and no verified merge protocol applies.

**Invariant:** I-006.

### L-009 Verified merge algebra

**Statement:** a merge protocol can be marked `Verified` only if its declared algebra satisfies the laws required by its mode: associativity, identity where declared, commutativity if concurrent ordering is intentionally irrelevant, idempotence if duplicate delivery can occur, and monotonicity with respect to observed-truth ordering.

**Invariant:** I-006 and future retry/idempotence model.

### L-010 Drain fence safety

**Statement:** a drain fence for scope `S` may be acquired only after all prior overlapping mutable leases for `S` are released or expired, and no new mutable admission for `S` can pass once the drain is pending under the drain policy.

**Invariant:** I-007.

### L-011 Overlay monotonicity

**Statement:** if overlay obligations are accepted over base obligations, then every base requirement remains required in the overlaid contract.

**Invariant:** I-004.

### L-012 Route/profile compatibility

**Statement:** a route selected for a predicate must be compatible with the predicate's consequence profile and lifecycle class. A route/profile drift is invalid.

**Invariant:** I-005.

### L-013 Constraint update future-only theorem

**Statement:** a constraint update may affect future admissions/frontier decisions but cannot change already committed observed truth or already emitted anchored projections.

**Invariant:** I-010.

### L-014 Replay soundness theorem

**Statement:** if the executable replay oracle accepts a trace under bundle `B`, and the lowering relation from concrete trace to abstract trace is sound, then the abstract trace satisfies all replay-modeled invariants.

This theorem requires a connection to Verus/Kani/replay; Lean alone cannot assert that Rust replay implements the relation.

### L-015 IR/model adequacy theorem

**Statement:** generated model facts are adequate for the source bundle/scenario: every fact used by a theorem comes from Formal IR, every Formal IR fact comes from bundle/scenario, and freshness receipts bind the generated file to those hashes.

**Purpose:** prevents formal theatre where generated model proves a property of facts not present in runtime contracts.

### L-016 Migration/refinement theorem

**Statement:** for a schema migration from version `n` to `n+1`, either the migration refines old semantics for affected invariants or old receipts are invalidated as stale.

## Verus lane: intended scope

Verus proves preservation properties for pure kernel rules and validators. It is not a substitute for replay negative controls or generated target facts.

Generated layout stays:

```text
verification/formal-full/verus/generated/*.rs
verification/formal-full/receipts/*.verus.codegen.json
verification/formal-full/receipts/*.verus.tool-run.json
```

Verus artifacts must be generated from Formal IR where scenario-bound. Generic Verus support modules may exist but cannot be counted without generated instance proof or explicit receipt.

## Verus proof catalog

### V-001 Lifecycle reducer preservation

Prove that each allowed event transition preserves the lifecycle invariant:

```text
execution_started -> barrier_logged
observed_truth -> execution_started
projection_emitted -> observed_truth
closed -> terminal
```

### V-002 Replay acceptance implies valid bounded trace

For bounded traces or abstracted reducers, prove that accepted replay states satisfy the same safety predicates as the lifecycle reducer. Full unbounded parser/IO behavior is not the first target.

### V-003 Capability derivation and validation soundness

Prove that `ExecutionCapability::derive_from_barrier` and `validate_for_barrier` imply exact barrier/action/plan/op/lease/canonical-id binding.

### V-004 Witness selector exactness

Prove that the selector resolver cannot satisfy a required witness with wrong requirement id, event kind, fact kind, scope, action, plan or impact set.

### V-005 Projection anchor exactness

Prove that projection anchor validation only accepts observed-truth events whose attested fact kind/scope matches the anchor claim.

### V-006 Authz resolver safety

Prove default-deny, Deny-wins and freshness/policy checks over an abstract timestamp/policy model. If timestamps remain replay-only, record the gap honestly.

### V-007 Lease conflict and coverage safety

Prove exact claim coverage and fail-closed conflict predicate. Non-verified merge protocol statuses must not clear conflict.

### V-008 Drain safety

Prove that a drain fence is acquirable only when overlapping active mutable leases are absent, and that pending drain blocks new mutable admission where policy requires.

### V-009 Overlay monotonicity

Prove that accepted overlay obligations preserve all base obligations.

### V-010 Route/profile compatibility

Prove that route derivation cannot return a route incompatible with consequence profile/lifecycle class.

### V-011 Constraint update preserves committed truth

Prove that `ConstraintUpdate` cannot rewrite already committed truth facts.

### V-012 Canonical serialization helper invariants

Where Rust code exposes pure canonicalization helpers, prove determinism and field-boundary injectivity. Cryptographic collision resistance remains an assumption, not a theorem.

### V-013 Receipt/coverage report non-upgrade

Prove for pure report builders that failure statuses are never upgraded to pass, and `covered` cannot be produced without required lane evidence.

## No-cheating rules

Lean4:

```text
forbid `sorry` in authoritative files
forbid new `axiom` except in explicitly whitelisted foundation files
forbid theorem statements that do not mention generated facts when claiming bundle-specific evidence
```

Verus:

```text
run with --no-cheating for proof/all profiles
forbid `assume`, `admit`, `external_body`, `unimplemented!`, or panic-based proofs in authoritative files unless exception-approved
require generated files to carry source_bundle_hash/formal_ir_hash/scenario_hash
```

## Acceptance for Lean4/Verus

A Lean4/Verus obligation is accepted only when all conditions hold:

1. theorem/proof function has stable ID;
2. ID appears in obligation manifest;
3. generated artifact contains the theorem/proof application or generated instance facts;
4. tool-run receipt records real command, version, exit code and result;
5. stale-check binds artifact to bundle/scenario/IR;
6. coverage report derives lane status from receipt;
7. docs matrix is drift-checked from coverage report;
8. no expired exception covers the lane.
