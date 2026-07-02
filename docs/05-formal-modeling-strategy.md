# Formal modeling strategy

## Status: generated from Formal IR

Formal targets consume the same compiled bundle/scenario contract through
`FormalIr` v2 (see the spec linked below). Generic hand-written support models are not authority by
themselves; generated artifacts bind `source_bundle_hash`, `scenario_hash`,
`formal_ir_hash`, target, artifact kind and invariant ids.

The acceptance gate is:

```bash
just formal-ready
just verification-full
```

The contract for the target-neutral input is documented in
[`specs/formal-ir-v2.md`](specs/formal-ir-v2.md).
The generated proof/refinement scope is documented in
[`formal/08-proof-refinement-scope.md`](formal/08-proof-refinement-scope.md);
it classifies evidence strength without replacing the coverage matrix.

## Goal

Formal modeling is used to attack the contract before production code hardens around mistakes.

The target is not to prove the whole external world. The target is the small dispatcher kernel and its protocol invariants.

## Tool roles

```text
Alloy  -> relational/counterexample lane.
P-lang -> protocol/interleaving lane.
Kani   -> bounded code-facing Rust checks.
Verus  -> metatheory / preservation proofs.
```

## Alloy

Use Alloy to ask:

```text
Can a bad world exist under these contracts?
```

Initial assertions:

- no execution without barrier;
- no observed truth without execution;
- no projection without observed-truth anchor;
- no overlay weakening;
- no route without consequence profile;
- no conflicting mutable frontier without merge protocol;
- no drain acquired while prior overlapping mutable lease remains active;
- no approval satisfying a gate without binding to action id + plan hash + impact set.

## P-lang

Use P to ask:

```text
Can a bad history/interleaving occur?
```

Initial machines:

```text
Dispatcher;
AuditLog;
LeaseManager;
ConstraintProvider;
Worker;
ProjectionBuilder;
PolicyProvider;
TestDriver.
```

Initial monitors:

- no worker execution before durable barrier;
- no observed truth before execution;
- no projection before anchor;
- drain blocks new mutable admission;
- retry does not duplicate hard execution;
- authorization revocation before barrier blocks execution;
- constraint epoch changes do not rewrite observed truth.

## Kani

Use Kani after Rust reducers exist.

Initial harnesses:

- lifecycle reducer accepts no forbidden transition;
- consequence profile obligations are total;
- bounded replay trace acceptance implies ordering invariants;
- lease acquire/release never permits conflicting active exclusive leases;
- quota/capacity counters do not overflow/underflow;
- parsers/decoders reject malformed input without panic.

## Verus

Use Verus after the abstract kernel stabilizes.

Initial theorems:

- lifecycle preservation;
- barrier-before-execution;
- projection-anchor soundness;
- overlay monotonicity;
- lease map preservation;
- replay accepts implies valid protocol trace;
- selected frontier is a conflict-free antichain.

## Coverage matrix

Every dispatcher-critical invariant should have:

```text
statement;
authority surface;
generated input;
verification lane;
code-facing confirmation;
runtime modules affected;
readiness blocker status;
stale receipt policy;
known gaps.
```

See [`invariants/coverage-matrix.md`](invariants/coverage-matrix.md).
