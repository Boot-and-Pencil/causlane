# Formal Impact Record: Alloy multi-action sig generation (dispatcher-012 P1-002)

## Change metadata

- Change ID: FIR-2026-06-20-alloy-multi-action
- PR/issue: dispatcher-012 ТЗ follow-ups (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-20
- Impact class: F3 (formal-model coverage) — Alloy generator; no kernel/invariant
  semantics change; single-action output unchanged

## Touched protocol-critical paths

```text
crates/causlane-codegen/src/alloy.rs
crates/causlane-codegen/src/alloy_events.rs            (new)
crates/causlane-codegen/src/lib.rs
crates/causlane-codegen/tests/multi_action_alloy.rs   (new)
docs/formal/dispatcher-012-tz-status.md
```

## Summary

The Alloy scenario-facts generator collapsed every event onto a single `Action` and
single `Plan` atom (one `Action_<id>` sig, one `Plan_<hash>` sig, all events
`.action = <that one>`). A scenario spanning more than one action (e.g. a readiness
action whose observed truth a separate promotion action witnesses) could not be
modelled faithfully.

This makes the generator multi-action aware:

- one `Action` sig per distinct event `action_id` and one `Plan` sig per distinct
  plan hash, with `Action`/`PlanHash` set to the union of those atoms and the bounded
  `check` scope counting `exactly N Action`/`exactly M PlanHash`;
- each event is bound to ITS action/plan atom (per-event resolution with the
  scenario's primary as the default);
- a single-action scenario degenerates to exactly one `Action`/`Plan` atom —
  verified bit-for-bit unchanged for `release_promote_success` and
  `conflicting_leases_invalid`.

The per-event emission + multi-action sig derivation moved to a new `alloy_events`
module (the 800-line cap). Replay already keys state by `(action, plan)`, so it
needs no change.

## Non-vacuity proof (anti-theatre)

- New integration test `multi_action_alloy` builds a 2-action / 2-plan
  `AlloyScenarioFacts` and asserts the generated `.als` contains two distinct
  `Action_*` sigs, two distinct `Plan_*` sigs, `Action = a + b`, `PlanHash = p + q`,
  `exactly 2 Action, exactly 2 PlanHash`, and per-event binding (the promotion events
  name the promotion action/plan). Gate-run via `cargo test -p causlane-codegen
  --all-targets`.
- The existing single-action scenarios still generate `exactly 1 Action` and pass
  AlloyRunner; `conflicting_leases_invalid` still refutes.

## Affected invariants

```text
none — generator coverage only. No invariant semantics, replay decision, or other
lane changes.
new invariant ids: none
```

## Affected formal models

```text
Alloy: scenario facts now emit per-action Action/Plan atoms. Single-action artifacts
are byte-identical. No P/Kani/Verus/Lean4 change.
```

## Contract changes

- Bundle / Formal IR data / replay-trace / receipt / coverage fields: none.
- Core semantic change: none.

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| `multi_action_alloy` (integration test) | Alloy/codegen | 2 Action + 2 Plan atoms emitted | new — verified |
| `release_promote_success` | Alloy | single-action `exactly 1 Action`, pass | existing — re-verified bit-for-bit |
| `conflicting_leases_invalid` | Alloy | still refutes | existing — re-verified |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Alloy | per-action `Action`/`Plan` sigs (`multi_action_alloy`) | generated | rust |

## Not applicable lanes

No P/Kani/Verus/Lean4 change.

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-codegen --all-targets
just verification-full
```

## Exception request

- Exception needed? no
- Follow-up (P2-004): a full replay-valid multi-action *reference scenario* needs a
  non-strict (non-RuntimeExecution) predicate bundle — the only example predicate
  requires a barrier per action, so a minimal multi-action trace cannot replay-validate
  against it. The generator capability is proven here regardless.
