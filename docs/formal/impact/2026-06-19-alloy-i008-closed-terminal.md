# Formal Impact Record: Alloy models I-008 (closed-terminal)

## Change metadata

- Change ID: FIR-2026-06-19-alloy-i008-closed-terminal
- PR/issue: B-013 Alloy strengthening (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-19
- Impact class: F3 (formal-model coverage expansion) — no kernel behavior change

## Touched protocol-critical paths

```text
verification/formal-full/alloy/core/causlane_core.als
crates/causlane-codegen/src/obligations.rs
verification/formal-full/obligations/lifecycle_product_obligations.yaml
scripts/check-verification-full.sh
tools/coverage-matrix
docs/invariants/coverage-matrix.json
docs/invariants/coverage-matrix.md
docs/formal-readiness-status.md
```

## Summary

The generic Alloy core modeled I-001/002/003 (lifecycle guards in `Enforced`) +
I-006/I-009 (generated binding assertions), leaving I-004/005/007/008/010
`not_applicable`. This extends Alloy to model **I-008 (no event for an action may
occur after that action's `LifecycleClosed`)** — the cheapest, highest-value add,
because the generated `.als` already carries `LifecycleClosed` events with full
`hb` ordering (no codegen field change).

- `causlane_core.als`: a `ClosedIsTerminal` clause is added to the `Enforced`
  predicate (`all e | e.kind = LifecycleClosed implies no late: Event |
  late.action = e.action and e in late.hb`), plus a standalone
  `assert I_008_NoEventAfterClosed` + `check … for 6`. The
  `run ValidTraceExists` consistency command still finds a model (Enforced is not
  made contradictory; the success trace keeps `LifecycleClosed` last).
- The scenario-bound `GeneratedTraceSatisfiesCore` assertion (which asserts
  `Enforced`) now covers I-008. `obligations.rs` adds
  `("I-008", "GeneratedTraceSatisfiesCore")`; the manifest sets I-008's alloy lane
  `required` with that check_id.
- `scripts/check-verification-full.sh` adds `event_after_closed_invalid` to the Alloy
  negative-control set; coverage/docs/inventory updated (derived, not hand-asserted).

## Non-vacuity proof (anti-theatre)

The new assertion is genuinely discriminating, verified in both directions:

- `event_after_closed_invalid` run through Alloy: **status=pass BEFORE** the core
  change (Alloy did not model I-008) → **status=fail AFTER**, refuted via
  `GeneratedTraceSatisfiesCore`. That delta is the proof.
- The positive `release_promote_success` scenario still passes (status=pass).
- The abstract core check `I_008_NoEventAfterClosed` holds (UNSAT) and
  `ValidTraceExists` is SAT.

## Affected invariants

```text
I-008: No event may mutate lifecycle after terminal close — Alloy lane goes
       not_applicable → passed (a second independent lane alongside replay/P/Kani).
       Kernel SEMANTICS unchanged; this adds formal MODEL coverage only.
new invariant ids: none
```

## Affected formal models

```text
Alloy: GeneratedTraceSatisfiesCore now asserts the ClosedIsTerminal clause of
       Enforced (base model); I_008_NoEventAfterClosed abstract check added.
       Negative control: event_after_closed_invalid (refuted).
```

## Contract changes

- Bundle / Formal IR / replay-trace / receipt fields: none.
- Coverage fields: I-008 alloy cell not_applicable → passed (derived from receipts).
- Core semantic change: none (formal-model coverage only).

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| `event_after_closed_invalid` | Alloy | `GeneratedTraceSatisfiesCore` refuted (counterexample: event after `LifecycleClosed`) | new — verified pass→fail |
| `event_after_closed_invalid` | Replay | `EventAfterClosed` | existing — unchanged |
| all existing Alloy controls | Alloy | still refute their own assertions | existing — re-verified |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Alloy | `GeneratedTraceSatisfiesCore` (now also I-008) + `I_008_NoEventAfterClosed` | base model + generated | rust |
| Replay/P/Kani | unchanged (already cover I-008) | yes | rust |
| Verus/Lean4 | unchanged (non_blocking_spec for I-008) | yes | proof/all |

## Not applicable lanes

I-004/005/007/010 remain `not_applicable` in Alloy (the generated `.als` lacks the
fields, or it is a deliberate scoping choice). I-007 (drain) would need codegen to
emit drain-scope→lease bindings — a separate increment.

## Acceptance commands

```bash
just formal-ready
just verification-full
```

Additional commands:

```bash
java -cp .tools/alloy/alloy.jar:.tools/alloy/classes AlloyRunner \
  verification/formal-full/alloy/generated/event_after_closed_invalid.als   # status=fail (refuted)
tools/coverage-matrix --check
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: I-007 (drain) in Alloy (codegen drain-scope bindings); TZ-007
  runtime merge enforcement (blocked on S05); Verus/Lean4 exception renewal before
  2026-09-01.
