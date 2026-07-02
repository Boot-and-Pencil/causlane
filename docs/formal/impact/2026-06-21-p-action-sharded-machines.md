# Formal Impact Record: action-sharded concurrent P machines (dispatcher-012 P1-001 part 3)

## Change metadata

- Change ID: FIR-2026-06-21-p-action-sharded-machines
- PR/issue: dispatcher-012 ТЗ P1-001 (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-21
- Impact class: F3 (formal-model coverage / correctness) — P generator; no
  kernel/invariant semantics change

## Touched protocol-critical paths

```text
crates/causlane-codegen/src/targets.rs
scripts/check-verification-full.sh
contracts/scenarios/multi_action_cross_action_barrier_invalid.scenario.yaml   (new)
docs/formal/dispatcher-012-tz-status.md
```

## Summary

The final P1-001 increment replaces the **single sequential `ScenarioDriver` plus
the empty role stub machines** (`Dispatcher`/`AuditLog`/`LeaseManager`/… each
`start state Init { entry { } }`) with **one driver machine per distinct
`action_id`**, all created by a `ScenarioBootstrap` `main` machine. P's scheduler
now interleaves the actions' event streams, so the action/scope-keyed monitors
landed in parts 1 and 2 are finally exercised against real interleaving rather than
one fixed send order.

Changes in `push_p_machines` / new `push_p_test` (`targets.rs`):

1. `distinct_actions(ir)` collects the action ids in first-seen order; each gets a
   machine `ActionDriver_<alloy_ident(action)>` that, in its `entry`, sends only
   *its* events in order (lease events still expand one send per lease, unchanged).
2. `ScenarioBootstrap` (the new `main`) does `new ActionDriver_<a>();` for every
   action, so the drivers run concurrently.
3. The `test` declaration is emitted by `push_p_test` with the dynamic machine set
   (`in { ScenarioBootstrap, ActionDriver_… }`). The test **name is unchanged**
   (`release_promote_generated`) — the gate still selects it with
   `--testcase release_promote_generated` — only `main` and the module set change.

The 15 monitors (`push_p_monitors`) are untouched. A single-action scenario yields
exactly one driver and no cross-action interleaving, so its P output is behaviourally
unchanged (the success P-check still passes, rc=0).

## Non-vacuity proof (anti-theatre)

- **Positive, multi-action:** `multi_action_reference` (the P2-004 EvidenceMeta
  fixture, two actions `act_ref_a`/`act_ref_b`) generates two `ActionDriver`
  machines + `ScenarioBootstrap` and **passes** the P-check (rc=0) across all
  schedules — the action-keyed monitors hold under interleaving.
- **Negative, cross-action (the keying is load-bearing):**
  `multi_action_cross_action_barrier_invalid` — `act_x` runs a clean approved
  barrier; `act_y` has **no barrier of its own** and its execution rides `act_x`'s
  barrier. In the schedule where `act_x`'s barrier is observed first, a *flat*
  monitor ("was SOME barrier seen?") would be fooled; the **action-keyed**
  `NoExecutionBeforeBarrier` and the **barrier-keyed** `CapabilityBindsToBarrier`
  refute it. Verified `p check` rc=1.
- **Regression:** `release_promote_success` (single action) re-verified rc=0 with the
  new one-driver output.

## Affected invariants

```text
I-001 (and the lifecycle/lease monitors generally): now verified under real P
interleaving across actions, not a single fixed send order. Single-action semantics
unchanged. new invariant ids: none
```

## Affected formal models

```text
P: machine topology only — per-action driver machines + ScenarioBootstrap replace
the single ScenarioDriver and the empty role stubs; the test main/module set is
generated dynamically. Monitors, EventPayload and the 18 events are unchanged. No
Alloy/Kani/Verus/Lean4 change.
```

## Contract changes

- Bundle / Formal IR / replay-trace / receipt / coverage fields: none.
- Core semantic change: none.

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| `multi_action_cross_action_barrier_invalid` | P | refute (cross-action barrier riding) | new — verified rc=1 |
| `multi_action_reference` | P | pass (two action drivers interleave) | new positive — verified rc=0 |
| `release_promote_success` | P | pass (single driver, no regression) | re-verified rc=0 |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| P | action-sharded drivers + `ScenarioBootstrap` + cross-action control | generated | rust |

## Not applicable lanes

No Alloy/Kani/Verus/Lean4 change.

## Acceptance commands

```bash
just verification-full
```

## Exception request

- Exception needed? no
- Follow-up: none — P1-001 is complete (parts 1, 2 and 3 landed).
