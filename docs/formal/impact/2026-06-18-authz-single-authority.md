# Formal Impact Record: authz single-authority (dedup structural+temporal decision)

## Change metadata

- Change ID: FIR-2026-06-18-authz-single-authority
- PR/issue: S03 reference-kernel hardening (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-18
- Impact class: F4 (runtime/authz path) — behavior-preserving dedup

## Touched protocol-critical paths

```text
crates/causlane-core/src/domain/authz.rs
crates/causlane-core/src/contract.rs
crates/causlane-core/src/lib.rs
crates/causlane-runtime/src/authz.rs
crates/causlane-replay/src/authz.rs
```

## Summary

The deny-by-default authz decision (ADR-0011) had its structural+temporal
per-decision logic **duplicated**: once in `authz_gate` (used live by the runtime)
and again, re-implemented, in replay's `validate_authz_refs`. That duplication is
the drift risk between live enforcement and the replay oracle. This extracts ONE
shared per-decision classifier and routes everything through it, without changing
behavior:

- `classify_authz_decision(decision, stage, action, plan, predicate_id, policy,
  now: Option<Timestamp>) -> AuthzDecisionVerdict` (new, in core): the single
  structural+temporal authority — stage filter, deny-wins, action/plan/predicate
  binding, policy (P0-010), then temporal (issued-after / expiry / freshness when
  `now` is present; born-expired when `None`, matching replay's no-barrier-time
  fallback). `authz_gate` is refactored to loop over it (aggregate precedence
  unchanged).
- `KernelContracts` gains `AuthzEvaluator::evaluate_authz`, delegating to
  `authz_gate`, so the gate has a named single authority like the other contracts.
- Runtime `AuthzGuard::authorize_barrier` routes through
  `KernelContracts.evaluate_authz` (was the free `authz_gate`) — pure refactor.
- Replay `validate_authz_refs` reuses `classify_authz_decision` for the shared
  decision, keeping its OWN aggregate precedence and its replay-only layers (keyed
  PDP-MAC attestation + event-structure validation + `Option<barrier_time>`). Its
  `ReplayError` codes are unchanged.

`causlane-core` stays crypto-free: attestation and event-structure remain replay's
layer, applied after the shared classifier returns `Allow`.

The born-expired check `authz_gate` previously did in its `Some` path is omitted in
the classifier's `Some` branch as provably redundant (`now >= issued_at` makes a
born-expired decision also `expires_at <= now` → same `Expired` reason; `now <
issued_at` is already `IssuedAfter`) — confirmed against `born_expired_decision_is_refused`.

## Affected invariants

```text
I-009: Witness/authz evidence must bind exact action, plan and scope — the authz
       binding facet is now decided by one shared classifier for live + replay;
       semantics UNCHANGED.
ADR-0011 deny-by-default + freshness: structural+temporal decision deduplicated;
       outcomes and error codes preserved.
new invariant ids: none
```

## Affected formal models

```text
none — no generated artifact, bundle, IR or coverage change. The authz negative
controls already exist and are re-verified through the shared classifier.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Core semantic change: none (behavior-preserving dedup); new pure
  `classify_authz_decision` + `AuthzEvaluator` contract on `KernelContracts`.

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| `authz_denied_invalid` | Replay | `AuthzDecisionDenied` | existing — re-verified via the shared classifier |
| `authz_missing_invalid` | Replay | `AuthzDecisionMissing` | existing — re-verified |
| `authz_expired_invalid` | Replay | `AuthzDecisionExpired` | existing — re-verified |
| `authz_wrong_policy_invalid` | Replay | `AuthzPolicyMismatch` | existing — re-verified |
| `authz_issued_after_barrier_invalid` | Replay | `AuthzIssuedAfterBarrier` | existing — re-verified |
| `authz_stale_invalid` | Replay | `AuthzDecisionStale` | existing — re-verified |
| `authz_success` | Replay | accepted (fresh bound Allow) | existing — re-verified |
| `bundle_mode_attested_requires_valid_authz_attestation` | replay unit | unsigned/forged Allow refused when PDP secret configured | existing — replay-only layer unchanged |

All six authz controls re-refute with the exact same error codes through the new
classifier path (confirmed in `just verification-full`). No new control required.

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | `authz_*_invalid` + `authz_success` (unchanged) | yes | rust |
| Alloy | `AuthzDecisionGroundsBarrier` structural (unchanged) | yes | rust |
| P | `AuthzDecisionGroundsBarrier` (unchanged) | yes | rust |
| Kani | n/a | n/a | rust |
| Verus | n/a | n/a | proof/all |
| Lean4 | n/a | n/a | proof/all |

## Not applicable lanes

Alloy/P model only the structural requirement that a `RuntimeExecution` barrier
references a bound `Allow` (per formal-readiness-status.md); replay remains the
executable authority for temporal/policy/attestation facts. The dedup keeps that
division of labor — it only removes the duplicated structural+temporal core.

## Acceptance commands

```bash
just formal-ready
just verification-full
```

Additional commands:

```bash
./tools/cargo-dev test -p causlane-core -p causlane-runtime -p causlane-replay
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: provider-side authz strict-mode (provider identity /
  dev-exemption expiry) is still deferred — the decision/policy payloads do not yet
  carry enough provider data.
