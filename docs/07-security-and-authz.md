# Security and authorization

## Core idea

Authorization is not endpoint middleware. Authorization must evaluate whether an actor may move a specific typed action through a specific lifecycle stage.

```text
Actor + Predicate + Subject + Circumstance + Stage + PlannedImpacts
  -> AuthorizationDecision
  -> DispatchDecision / Gate / Barrier / Capability
  -> AuditEvidence
```

## Default posture: deny-by-default

Authorization is **deny-by-default / fail-closed** (see [ADR-0011](adr/0011-authz-default-deny.md)). Absence of an explicit allow is a denial:

```text
unknown predicate                         -> Deny
known predicate WITHOUT authz policy      -> Deny
provider unavailable, hard-effect action  -> Deny or Wait (never Allow)
no authorizer configured                  -> Deny
```

An open posture MUST be an explicit registry contract, never an implicit fallthrough:

```text
authz.mode = public                 (with rationale; typically ProjectionRead)
authz.mode = disabled_for_local_dev (with allowed_in_profiles)
```

## Stages to authorize

```text
may_call;
may_plan;
may_dispatch;
may_attach_overlay;
may_cross_execution_barrier;
may_execute_op;
may_commit_observed_truth;
may_project;
may_view_audit;
may_administer_policy.
```

## Approval as action

Approval is not a boolean flag. It is a modeled action:

```text
gate.approve(subject, circumstance)
gate.deny(subject, circumstance)
gate.revoke(subject, circumstance)
```

For hard effects, approval must bind to:

```text
action_id + plan_hash + planned impact set.
```

If the plan changes, approval becomes stale unless policy explicitly says otherwise.

## Execution capability

A worker should not execute from raw job payload alone. It should receive a scoped execution capability:

```text
action_id;
plan_hash;
op_index;
lease ids;
allowed impacts;
write scopes;
policy decision id;
expiry;
executor identity.
```

### Spend-time admission (M06.6)

Holding a capability is not sufficient to execute. Before the worker spends one, it
runs a fail-closed admission check (`ExecutionCapability::spend_admits`) that the
guarded executor enforces between deriving the capability and calling the executor —
the seam, not each executor adapter, is the enforcement point. The check is
deny-wins, in precedence order:

```text
NotBoundToBarrier  -> the capability fails structural validation against the
                      barrier (`validate_for_barrier`: barrier/action/plan/op/lease
                      binding and canonical id). Highest precedence.
OpMismatch         -> the capability is valid for the barrier but scoped to a
                      different op than the one about to run (no op-substitution).
Expired            -> the capability's lease-derived expiry is at or before the
                      spend instant `now` (parity `<=` with the authz gate).
```

`Ok` means: structurally bound, op-exact, and live. The `Expired` check enforces the
**lease-derived** expiry, which the authorization gate does not look at — authz judges
the *decision's* freshness, this judges the *capability's*. Under normal in-process
flow the capability is freshly derived, so only `Expired` can arise; the binding and
op checks become load-bearing the moment a capability is carried across a trust
boundary (cached, serialized, or minted by a separate component). Governing decision:
ADR-0013 (barrier/witness/lease binding).

### Guarded executor service seam (M08.3)

`GuardedExecutor` is also the runtime's dependency-free executor service seam:
`GuardedExecutionRequest` carries the barrier, authz evidence, policy, spend
time and op; `ExecutorService::call` returns an `ExecutionOutcome`. The legacy
`spend_barrier` API and the service API share the same helper, so the security
order above has one implementation. Individual executor adapters implement only
`ExecutorPort`; they do not re-check authz or capability validity.

### Apalis guarded execution bridge (M08.4)

The optional Apalis adapter carries `GuardedExecutionJob` inside an Apalis
request and immediately re-enters `ExecutorService::call`. Apalis request
context, extensions and worker metadata are non-authoritative: they cannot add
authz evidence, change the expected policy, bypass capability derivation, or
override spend-time admission.

### Restate guarded execution bridge (M08.5)

The optional Restate adapter accepts a JSON wrapper over opaque host payload
bytes, decodes those bytes through a host-owned decoder, and immediately
re-enters `ExecutorService::call`. Restate handler context, state, workflow id
and metadata are non-authoritative: they cannot add authz evidence, change the
expected policy, bypass capability derivation, or override spend-time admission.
Adapter decode failures and guarded execution failures are mapped to Restate
terminal errors by default, so failed authorization/capability checks fail
closed rather than retrying into execution.

## Projection read authorization and redaction (M06.7)

Reads are part of the dispatch protocol, not middleware. A projection read is
authorized at the `may_project` stage by the **same** deny-by-default gate the
execution path uses (`read_authz_gate` delegates to `authz_gate`), so the read and
execution paths cannot drift. A read differs in one way: it is authorized **for a
specific reader** — `read_authz_gate` restricts the decisions to those issued for
the requesting `actor` before applying the gate, so a `may_project` `Allow` minted
for another reader does not authorize this one (it is invisible to the gate, which
then denies `Missing`). A read is admissible only if it is both **grounded** (the
projection is anchored to a prior `observed_truth.committed`, enforced by replay)
and **authorized** here.

Field redaction has two layers. The M06.7 kernel **mechanism** fixes the masking
rule, fail-closed as an **allowlist**: a projection field is revealed iff the
host's `RedactionPolicy` lists it as `revealable`; every other field — sensitive,
unknown, or merely unlisted — is redacted. So a field the host forgot to list is
masked, never leaked. `FieldPath` is an exact-match token (the kernel normalizes
nothing, so canonicalizing payload paths is a host obligation). The runtime
composes read authz with this mechanism: `guard_projection_read` authorizes the
read and only then emits the `RedactionView`, so no field reaches an unauthorized
reader.

M07.5 adds the shared **classification/profile** layer for audit, log,
projection, replay and support-bundle surfaces. A host assigns each known field a
`RedactionClass` (`Public`, `Operational`, `Restricted`, `Secret`) inside a
`SurfaceRedactionProfile`; `compile_redaction_policy` reduces that typed profile
to the same `RedactionPolicy` allowlist. The class policy is itself an allowlist:
`Default` reveals no classes, while `public_only()` reveals only fields classified
`Public`. Duplicate declarations for the same path are fail-closed: a path is
revealable only if every declaration's class is revealable. This layer defines
classification and policy composition only; value-byte masking / JSON shaping
remains a host or adapter responsibility.

## TOCTOU boundary

Check authorization at admission and again at execution barrier for hard effects. Long-running actions should use authorization leases/capabilities rather than permanent allow decisions.

## Trust boundary and tamper-evidence

Replay verifies an audit **trace**, and the trace is authored by whoever produced
it. Everything replay checks — event ordering (positional), barrier/witness/lease
bindings, plan hashes — is over data the trace author controls. Replay therefore
establishes **internal consistency against the invariants**, and, where a secret
is configured, **authenticity of the attested artifacts**; it does not by itself
make an arbitrary trace tamper-evident. State this boundary explicitly rather than
letting "replayable/auditable" be read as "tamper-proof".

What is cryptographically enforced (when a secret is configured):

```text
Execution capability  -> keyed attestation (HMAC) over its canonical bytes,
                         minted by the kernel; verified by
                         `ReplayTrace::verify_with_bundle_attested`. A trace
                         author who lacks the secret cannot mint a spendable
                         capability, even though the structural id matches.
Authz decision        -> keyed attestation (HMAC) over its canonical bytes,
                         minted by the PDP; an unsigned/forged Allow is refused.
```

What remains trust-on-input (the journal is the trusted root):

```text
- Causal ordering is positional in the events array (the author orders it).
- Semantic producer-attestation for anchors/witnesses is enforced by replay and
  generated formal negative controls, but there is no journal-level event
  hash-chain (`prev_event_hash`) yet and no cryptographic per-event `event_hash`
  content-pin for anchors/witnesses.
- Plan/bundle/impact hashes are compared, not (yet) recomputed from material in
  replay (see ADR-0009).
```

Operational requirement: the audit journal MUST be made tamper-evident **out of
band** — an append-only, signed/hash-chained log — and the kernel/PDP secrets
held outside the trace. The attestations above raise the bar from "structural
binding within a trusted trace" to "cryptographic binding under a held secret";
they do not remove the need to protect the journal itself. The journal-level hash-chain/content-pin work is a separate backlog item from the already-enforced semantic witness/anchor grounding controls.

## Policy adapters

Initial adapter candidates:

```text
Cedar       embedded fine-grained authz;
Casbin      simple RBAC/ABAC/domain model;
AuthZEN     external PDP protocol adapter;
OpenFGA     relationship-based authorization;
SpiceDB     Zanzibar-style permissions;
OPA/Rego    platform-policy ecosystem adapter.
```

## Invariants

```text
No action admission without an explicit authz allow (deny-by-default; ADR-0011).
No hard-effect execution without fresh barrier authz.
No executor may execute op without scoped capability.
No approval may satisfy gate unless bound to action_id + plan_hash + impact set.
No self-approval unless policy explicitly allows it.
No projection of sensitive truth without projection authz.
No stale authorization decision may cross execution barrier.
```
