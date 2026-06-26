# ADR-0012: Merge protocol semantics (default: none)

- Status: accepted
- Date: 2026-06-05
- Supersedes: -
- Superseded by: -

## Context

Two concurrent operations that mutate overlapping write scopes can race. Without a rule, the system may admit conflicting writes whose combined effect is non-deterministic, breaking replay and observed-truth authority (ADR-0003, ADR-0008).

By default there is NO merge protocol. Overlapping mutable write scopes therefore MUST NOT run concurrently. Real parallelism over a shared scope is only safe when results can be deterministically joined under an explicit, verified contract.

## Decision

By DEFAULT no merge protocol exists, so overlapping mutable write scopes are FORBIDDEN from running concurrently. If `overlap(write_scopes(A), write_scopes(B))`, both `A` and `B` are mutable, and no explicit Verified merge protocol applies, then `A` and `B` MUST NOT both appear in the concurrent mutable frontier.

A `MergeProtocol` is an EXPLICIT, bundle-level contract — NOT a string on an `Op` — that permits concurrent overlapping mutable ops because their results join deterministically.

`MergeProtocolSpec` fields:

```text
id                 : MergeProtocolId
version            : version
status             : MergeProtocolStatus
compatible_op_kinds
compatible_resources
scope_pattern
algebra            : MergeAlgebra
preconditions      : RequiredFactSpec
observation_policy
failure_policy
formal_obligations
```

```text
MergeProtocolStatus = Experimental | Verified | Disabled
MergeAlgebra        = CommutativeIdempotentJoin
                    | AppendOnlyDisjointKeys
                    | AdditiveCounter
                    | LastWriterWinsForbidden
                    | CustomVerified { proof_id }
```

For MVP, `MergeProtocol::None`: there are NO built-in verified protocols, so all overlapping mutable writes conflict.

Formal predicate:

```text
mergeable(a, b) :=
  exists protocol p:
    p.status = Verified
    && a.merge_protocol_id = p.id
    && b.merge_protocol_id = p.id
    && p.compatible(a, b)
```

Invariant I-006 then reads: "No two mutable frontier nodes may have overlapping write scopes unless `mergeable(a, b)` is true."

## Consequences

Easier:

- Conservative-safe concurrency by default; no accidental races over shared scope.
- I-006 is expressible directly against `mergeable()`.

Harder:

- Real parallelism over a shared scope requires a Verified protocol carrying formal obligations.

## Enforcement

- Docs: the Bundle schema supports `merge_protocols`; `release_promote` MUST use none (ADR-0013).
- Formal: the Alloy-generated projection expresses `NoConflictingMutableFrontier`; models assert I-006 against `mergeable()`.
- Replay: rejects overlapping-lease traces.
- Runtime: the lease layer MUST reject overlapping active `ExclusiveWrite` leases unless `mergeable(a, b)` holds (ADR-0013).
- Tests: cover overlapping mutable writes with and without a Verified protocol, asserting the default-none path conflicts.
