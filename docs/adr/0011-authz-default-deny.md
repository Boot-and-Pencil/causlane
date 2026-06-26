# ADR-0011: Authorization is deny-by-default (fail-closed)

- Status: accepted
- Date: 2026-06-05
- Supersedes: -
- Superseded by: -

## Context

The prior rule ("no admission without an authz decision, if authz is configured for that predicate") was ambiguous: a predicate with no configured authz policy fell through to an undefined state, which risks accidental open endpoints. We need a single, unambiguous, fail-closed rule.

## Decision

Authorization is DENY BY DEFAULT. Absence of an explicit allow is a denial.

```text
unknown predicate                         => Deny
known predicate WITHOUT authz policy      => Deny
provider unavailable, hard-effect action  => Deny or Wait (never Allow)
no authorizer configured                  => Deny
```

An open posture MUST be an explicit registry contract, never an implicit fallthrough:

```text
authz.mode = public                 (with rationale; typically ProjectionRead)
authz.mode = disabled_for_local_dev (with allowed_in_profiles)
```

Normative rules:

- Every predicate MUST have an explicit authz policy, or an explicit `public` / `disabled_for_local_dev` exemption, before it can run.
- The `disabled_for_local_dev` exemption MUST set `AuthzMode::DisabledForLocalDev` and MUST be gated by `allowed_in_profiles`.
- A `required` authz policy MUST name the `policy_id` and `policy_version` decisions are expected to be issued under (P0-010). A decision that authorizes the right action/plan/stage but is issued under a different policy is a `PolicyMismatch` and is denied — the gate and replay verify the decision's policy against the predicate's declared policy, not just its binding.
- "Fresh" (above) is operationalised at the barrier's evaluation time: a decision issued AFTER the barrier is forward-dated and denied; a decision older than the policy's optional `freshness_max_age` is `Stale` and denied even if it has not yet expired (so a long-lived or replayed-old grant cannot cross a barrier). Replay uses the barrier event's recorded `occurred_at` as the evaluation time; both the live gate and replay enforce the same rule.
- For RuntimeExecution hard effects: `may_call` MUST require an authz decision; `may_cross_execution_barrier` MUST require a FRESH authz decision, re-checked at the barrier to close TOCTOU (see ADR-0013); the executor MUST hold a scoped capability.

## Consequences

Easier:

- Safe by default; no accidental open endpoints.
- Authorization posture is auditable from the registry contract.

Harder:

- Every predicate needs an explicit authz policy (or an explicit dev/public exemption) before it can run.

## Enforcement

- Docs: Docs/07-security-and-authz.md normative text changed from "if configured" to deny-by-default.
- Runtime: admission denies absent an explicit allow; the barrier requires a fresh authz decision unless an explicit test/dev exemption applies (see ADR-0013).
- Tests: cover unknown-predicate-deny, no-policy-deny, and provider-unavailable-deny/wait.
- Relates to ADR-0013 (fresh authz reference at the barrier) and invariant I-009.

## Codification (M06.1)

The owned authorization-policy entity is codified in
`crates/causlane-core/src/domain/authz_policy.rs`, **on top of** the existing
deny-by-default gate (`authz.rs`: `classify_authz_decision` and the deny-wins
`authz_gate`) without duplicating or altering it. `AuthzPolicyModel { id:
AuthzPolicyId, version, stages: Vec<String>, freshness_max_age }` is the core
analogue of the `causlane-contracts` `AuthzPolicyManifest` DTO: it carries the
policy identity/version (P0-010), the lifecycle stages it authorizes (the same
string tokens the gate and `AuthzDecisionRef.stage` use — e.g.
`execution_barrier_logged`), and the freshness bound. `expected()` projects it onto
the borrowed `AuthzPolicy` view (the single source of the bound). `decide_authz_policy`
is fail-closed in two ordered steps: (1) the policy must authorize the stage
(`admits_stage` — deny-by-default membership, **precedence**: an unlisted stage
denies before any decision is classified); (2) the decisions must satisfy
`authz_gate` for that stage under `expected()` at `now` — **delegated** to the
shared kernel gate so binding, deny-wins, P0-010, and freshness/expiry are
re-derived nowhere and the model cannot drift from the live gate or replay. The
load-bearing property (`policy_model_is_fail_closed_with_gate_precedence`) asserts
over a grid that an unauthorized stage denies before classification, an authorized
stage mirrors `authz_gate` exactly, `Allowed` implies both the stage was authorized
and the gate allowed, and a decision reason is reported only for an authorized
stage — with non-vacuity over all three outcomes and temporal boundary cases
(`now` at `issued`, `issued + max_age`, and beyond). **Engine-agnostic**: it
prescribes no `RBAC`/`ABAC`/`ReBAC` and embeds no engine (`Cedar` is M06.2;
`Casbin`/`AuthZEN`/`OpenFGA` are M06.3). Pure additive `causlane-core` (no change to
`classify_authz_decision` / `authz_gate` / `AuthzDecisionRef` / `AuthzPolicy` /
`AuditEventKind` / codegen), so the I-009 receipts stand with no regeneration. The
`AuthzPolicyManifest → AuthzPolicyModel` lowering will live in `causlane-contracts`
(the single JSON source); approval-as-action (M06.4), step-up/SoD (M06.5),
capability enforcement (M06.6), and redaction (M06.7) are later S06 milestones.
