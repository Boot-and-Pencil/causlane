# Formal map for the full Causlane product lifecycle

> **Repository integration status:** proposed lifecycle discipline. This
> document is design/process authority only; current proof evidence remains the
> generated chain from compiled bundle and scenario through Formal IR, generated
> artifacts, receipts, stale-check and derived coverage.

## Status

This document is normative for process and design. It is not proof evidence by itself. The evidence authority remains the generated chain:

```text
registry -> compiled bundle -> scenario/trace -> Formal IR -> generated artifacts
  -> tool-run receipts -> stale-check -> derived coverage report
```

## Two lifecycles that must not be confused

Causlane has two relevant lifecycles:

1. **Product delivery lifecycle**: how a feature/fix/spec change moves from idea to release and later operation.
2. **Protocol lifecycle**: how a dispatchable action moves through audit events and kernel states.

Formal discipline covers both. Product lifecycle gates prevent humans from coding a new kernel behavior before the model/proof obligation exists. Protocol lifecycle models prove that the kernel behavior is safe once it exists.

## Product delivery lifecycle

| Stage | Output | Required formal work before code moves forward | Blocking gate |
|---|---|---|---|
| L0 Idea / issue intake | Change intent | Classify whether the change touches protocol-critical surface. If yes, create a Formal Impact Record. | `formal-discipline-check` |
| L1 Contract design | ADR/spec/registry delta | Name affected invariant IDs, protocol IDs and model IDs. Add or update obligation records before implementation. | schema validation + obligation manifest check |
| L2 Scenario design | success + negative scenarios | For every new safety condition, add at least one negative control that should fail before the implementation is fixed. | replay negative-control check |
| L3 Bundle/IR design | bundle fields / IR fields | If a model needs payload not present in Formal IR, extend bundle/IR first. A lane must never infer hidden facts. | `formal stale-check-all` + IR hash drift |
| L4 Generator/model design | generated Alloy/P/Kani/Verus/Lean4 artifacts | Add generated artifact obligations with `check_id`s. Generic hand-written support models are not authority. | codegen receipt + artifact hash |
| L5 Tool execution | tool-run receipts | Run real tools. A lane cannot be marked passed by docs or `jq` patching. | tool exit code + parsed result |
| L6 Implementation | Rust code | Code must implement the already-modeled contract. New behavior cannot appear first in `crates/*`. | PR path trigger + impact record |
| L7 Runtime integration | guarded executor / adapters | Adapters must satisfy port contracts; they do not weaken kernel proofs. | adapter simulation/port tests |
| L8 Release | coverage report | Release requires derived coverage to match docs and no expired exception. | `just verification-full` |
| L9 Operations | receipts, incidents, drift reports | Operational fixes must add failing scenario/control first when they affect protocol behavior. | incident formal impact gate |
| L10 Migration/deprecation | migration plan | Compatibility/refinement proof obligations must exist before removing or renaming protocol fields. | migration model check |

## Causlane protocol lifecycle

The minimal execution-bearing protocol sequence is:

```text
action.admitted
  -> action.planned
  -> dispatch.logged
  -> evidence/authz/lease/drain events*
  -> execution.barrier_logged
  -> execution.started
  -> execution.completed
  -> observed_truth.committed
  -> projection.emitted
  -> lifecycle.closed
```

The sequence is not a workflow engine. It is an audit/protocol contract for when hard effects may happen and how their truth may be observed and projected.

## Protocol event classes

| Class | Examples | Formal obligation |
|---|---|---|
| lifecycle advancement | admitted, planned, barrier, started, completed, closed | monotone state transition; no event after close |
| evidence | gate approved/denied, witness refs, fact attestation | producer-grounded fact kind/scope; exact action/plan/impact binding |
| authz | authz.decision_recorded | default deny, Deny wins, Allow must be fresh and stage-bound |
| lease/constraint | lease granted/released, drain fence | exact claim coverage, no conflict without verified merge, drain after overlap clear |
| capability | execution capability minted/spent | capability derives from a concrete barrier and canonical id |
| truth/projection | observed_truth.committed, projection.emitted | truth requires execution, projection requires observed-truth anchor |
| codegen/provenance | generated artifact, receipt, coverage | content-addressed freshness and no overclaim |

## What must be modeled before new code

A change is protocol-critical if it affects any of the following:

```text
compiled bundle schema
Formal IR schema
ReplayTrace / ReplayScenario / replay oracle
AuditEvent payloads
ExecutionBarrier / ExecutionCapability
WitnessRef / projection anchors / attested facts
Authz policy or decision binding
LeaseTable / conflict / merge / drain behavior
Consequence profiles, routes, lifecycle classes
constraint update semantics
formal generators / receipts / coverage report
runtime guarded executor
```

For such a change, implementation may start only after the PR contains:

1. a Formal Impact Record;
2. affected invariant IDs;
3. affected protocol IDs;
4. model/lane decision: Alloy/P/Kani/Verus/Lean4/replay/not-applicable;
5. new or updated negative controls;
6. codegen/IR changes if needed;
7. an explicit acceptance command.

## Known lifecycle statuses from the 009 baseline

The 009 baseline already contains a strong authority chain around Alloy/P/Kani/Verus, Formal IR v2, receipts v2, stale-check and derived coverage. This patch-pack extends the lifecycle policy in three directions:

1. adds an explicit **product delivery** gate, not only formal artifact generation;
2. uses **Lean4** as a proof-profile lane for abstract protocol semantics and model adequacy;
3. makes formal discipline enforceable before new protocol-critical features/fixes are implemented.
