# Formal Impact Record: multi-action replay-valid reference scenario (dispatcher-012 P2-004)

## Change metadata

- Change ID: FIR-2026-06-21-multi-action-replay-reference
- PR/issue: dispatcher-012 ТЗ P2-004 (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-21
- Impact class: F3 (formal-model coverage / correctness) — replay oracle
  generalization + new reference fixture; no invariant added

## Touched protocol-critical paths

```text
crates/causlane-replay/src/lib.rs                                   (validate_lifecycle)
crates/causlane-replay/tests/multi_action_reference.rs              (new)
contracts/examples/multi_action_reference.registry.yaml            (new)
contracts/scenarios/multi_action_reference.scenario.yaml           (new)
tools/formal-ready
tools/formal-verify-all
docs/formal/dispatcher-012-tz-status.md
```

## Summary

P2-004 asked for a replay-valid **multi-action** reference scenario. The only
example predicate is `RuntimeExecution`, which requires a full barrier ceremony per
action, so a *minimal* multi-action trace could not replay-validate against it.

Two changes close it:

1. **Per-action lifecycle reduction** (`validate_lifecycle`, `lib.rs`): the reducer
   previously filtered events to the trace's single `action_id` and reduced only
   that stream — a second action's lifecycle was silently ignored. It now collects
   the distinct `action_id`s (first-seen order) and reduces **each** action's own
   substream through the shared `LifecycleGrammar`. For a single-action trace this is
   identical to before. (`ProjectionRead` is unsuitable: `verify_events` requires
   every `projection.emitted` to anchor an in-trace `observed_truth.committed`, which
   the `ProjectionRead` grammar forbids — so the non-RuntimeExecution choice is a
   `EvidenceMeta` predicate whose lifecycle is `admitted → planned → dispatch_logged
   → closed`, with no barrier/projection obligation.)
2. **The reference fixture**: `multi_action_reference.registry.yaml` declares one
   `EvidenceMeta` predicate; `multi_action_reference.scenario.yaml` carries one trace
   with two independent actions (`act_ref_a`, `act_ref_b`) whose lifecycles are
   interleaved. It compiles, emits a bundle-bound trace, and replay-verifies
   `[strict]`.

The same two-action fixture is the input that exercises the P interleaving lane
(P1-001 part 3) — see `FIR-2026-06-21-p-action-sharded-machines`.

## Non-vacuity proof (anti-theatre)

- **Positive:** `multi_action_reference` replay-verifies with `--require-bundle-hash`
  (8 events, two actions). Added to `tools/formal-ready` and `tools/formal-verify-all`.
- **Generalization is load-bearing** (`tests/multi_action_reference.rs`): dropping
  `act_ref_b`'s `admit`+`plan` leaves it starting at `dispatch.logged` — a forbidden
  `New → DispatchLogged` transition. The new per-action reducer **rejects** it with
  `ReplayError::Lifecycle`; under the old single-action filter the secondary action's
  events were ignored and it wrongly passed.
- Full pre-existing replay corpus still passes (39 `causlane-replay` unit tests +
  the catalogued negative controls): the change is identical for single-action traces.

## Affected invariants

```text
I-008 (closed-terminal, keyed by action) is the invariant the EvidenceMeta reference
exercises. The lifecycle grammar itself is unchanged — only replay now reduces it
per action rather than for one action. new invariant ids: none
```

## Affected formal models

```text
Replay oracle: validate_lifecycle reduces every distinct action's substream. No
Alloy/P/Kani/Verus/Lean4 generator change (the same fixture feeds the P interleaving
lane via the unchanged P codegen path).
```

## Contract changes

- New registry/scenario fixtures only; no change to bundle/Formal-IR/receipt schema.
- Core semantic change: none (grammar unchanged; reduction is now per-action).

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| `multi_action_reference` | replay | pass (two-action history, strict) | new — verified |
| secondary-action broken lifecycle (in-test mutation) | replay | `Lifecycle` | new unit test — verified |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| replay | per-action `validate_lifecycle` + multi-action reference | n/a (oracle) | rust |

## Not applicable lanes

No Alloy/Kani/Verus/Lean4 change.

## Acceptance commands

```bash
just formal-ready
just formal-verify-all
```

## Exception request

- Exception needed? no
- Follow-up: none — P2-004 is complete.
