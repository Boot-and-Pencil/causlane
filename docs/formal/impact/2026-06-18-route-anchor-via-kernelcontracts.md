# Formal Impact Record: route projection-anchor decisions through KernelContracts

## Change metadata

- Change ID: FIR-2026-06-18-route-anchor-via-kernelcontracts
- PR/issue: S03 reference-kernel hardening (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-18
- Impact class: F3 (kernel-invariant enforcement path) — behavior-preserving routing

## Touched protocol-critical paths

```text
crates/causlane-replay/src/lib.rs
```

## Summary

`KernelContracts` implements `TruthAnchorResolver` (`anchor_source_is_valid`,
`anchor_matches`) but replay enforced I-003 (projection requires an observed-truth
anchor) with inline equivalents instead of calling the contract. This routes both
decisions through the single kernel authority so replay binds exactly what the
formal lanes are generated from — completing the declared S03 single-authority
list (lifecycle / capability / lease / drain / **anchor**):

- `lib.rs` ProjectionEmitted handler: the inline action+plan match becomes
  `KernelContracts.anchor_matches(anchor, &truth.action_id, truth_plan)` (the
  observed truth's `Option<PlanHash>` is threaded so a truth with no recorded plan
  never matches — identical to before).
- `lib.rs` ObservedTruthCommitted handler: the `observed`-map insert is gated on
  `KernelContracts.anchor_source_is_valid(event.kind)` so the contract, not the
  match-arm structure alone, is the authority for what may be an anchor source
  (trivially true in this arm, so no behavior change).

The anchor fact-grounding check (`AnchorAttestationMismatch`, P0-004) is unchanged.

## Terminal / I-008 note (no separate routing needed)

I-008 "no event after terminal close" is already single-authority: replay reduces
lifecycle through `KernelContracts.reduce` (`validate_lifecycle`), which rejects
any transition out of `Closed` (`ForbiddenTransition`), and the main verification
loop additionally enforces event-ordering via a positional `closed` set. The
`is_terminal` predicate has no live decision site to route (no `stage == Closed`
comparison exists in replay; the closed-set marking is event-positional, reached
before `validate_lifecycle`), so forcing it in would be dead code or change error
precedence. Terminal is therefore left as-is, behavior-preserving.

## Affected invariants

```text
I-003: Projection requires an observed-truth anchor — enforcement routed through
       KernelContracts (TruthAnchorResolver); semantics UNCHANGED.
I-002: Observed truth requires prior execution — unchanged (enforced by the
       ObservedWithoutExecution check; the anchor source it feeds is now
       contract-gated).
I-008: unchanged; already routed via KernelContracts.reduce (see note above).
new invariant ids: none
```

## Affected formal models

```text
none — no generated artifact changes. The I-003 negative controls already exist
and are re-verified through the new routing.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Core semantic change: none (behavior-preserving routing).

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| `projection_without_anchor_invalid` | Replay | `ProjectionWithoutAnchor` | existing — re-verified via new routing |
| `projection_anchor_wrong_plan_invalid` | Replay | `AnchorNotObservedTruth` (anchor_matches refuses the wrong plan) | existing — re-verified |
| `projection_anchor_wrong_fact_invalid` | Replay | `AnchorAttestationMismatch` | existing |
| `projection_anchor_wrong_scope_invalid` | Replay | `AnchorAttestationMismatch` | existing |
| `observed_without_execution_invalid` | Replay | `ObservedWithoutExecution` | existing |

No new negative control is required: the behavior under test is unchanged, and the
existing I-003 controls exercise both the routed `anchor_matches` (wrong plan) and
the anchor-source path.

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | `projection_*_invalid` (unchanged) | yes | rust |
| Alloy | `GeneratedAnchorFactGrounded` (unchanged) | yes | rust |
| P | `AnchorFactGrounded` (unchanged) | yes | rust |
| Kani | n/a | n/a | rust |
| Verus | non_blocking_spec (unchanged) | n/a | proof/all |
| Lean4 | `projection_anchor_soundness` (unchanged) | yes | proof/all |

## Not applicable lanes

No lane changes. The routing keeps the replay oracle's I-003 assertion identical;
the Alloy/P/Lean4 anchor-grounding lanes are generated from the same IR and are
unaffected.

## Acceptance commands

```bash
just formal-ready
just formal-verify-all
```

Additional commands:

```bash
./tools/cargo-dev test -p causlane-core -p causlane-replay
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: authz single-authority — replay `validate_authz_refs` and
  runtime `authz_gate` are parallel implementations; `KernelContracts` has no authz
  trait. Needs a new `AuthzEvaluator` contract + a semantic parity audit (its own
  FIR-gated increment).
