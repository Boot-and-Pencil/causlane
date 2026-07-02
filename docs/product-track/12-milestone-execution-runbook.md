# 12. Milestone Execution Runbook

This is the operational recipe for implementing Causlane milestones while
preserving the small-core, generated-truth and formal-discipline principles.

## Start With The Contract Surface

Before code, write or update the relevant evidence surface:

- registry or schema contract;
- scenario YAML and replay expectation;
- formal obligation manifest;
- adapter certification matrix;
- benchmark or SLO matrix;
- ADR when authority or public API changes.

## Classify The Change

| Class | Examples | Required evidence |
|---|---|---|
| F0 docs-only | typo, wording, links | docs/status check |
| F1 tooling/docs | runbook, non-authoritative script, CLI wrapper | product-track/status check |
| F2 contract-critical | bundle, replay, lifecycle, authz, barrier, leases | replay + FIR + negative control |
| F3 runtime authority | executor, audit, adapters, capabilities | F2 + adapter/runtime tests |
| F4 public API/release | crate API, semver, publication docs | F3 + compatibility/release docs |

## Pick The Verification Lane

| Change | Primary lane |
|---|---|
| impossible graph or contract shape | Alloy |
| interleaving or race protocol | P |
| Rust reducer, bounded trace or decoder | Kani or property tests |
| abstract preservation or refinement | Verus / Lean4 |
| production trace validity | replay oracle |
| adapter behavior | certification matrix + runtime tests |
| docs status claim | coverage/status gate |

## Add Negative Controls First

For every new invariant or enforcement rule, add at least one invalid scenario,
runtime test or formal control that would pass if enforcement were missing.

Existing examples:

```text
execution_without_barrier_invalid
projection_without_anchor_invalid
missing_witness_invalid
conflicting_leases_invalid
authz_missing_invalid
lease_during_drain_invalid
```

## Keep Adapters Outside The Semantic Core

Adapters may execute through scoped capability, persist audit events, export
observability and integrate backend runtimes.

Adapters must not decide semantic admissibility, create observed truth outside
audit, bypass execution barriers, weaken authz/lease/witness obligations or
become hidden workflow engines.

## Update Docs After Evidence Exists

When implementation is complete, update:

- stage and milestone status;
- readiness gate notes;
- ADR/FIR links;
- coverage matrix through the machine path when possible;
- risk register when residual risk remains.

## Close With Reproducible Commands

Every milestone change should end with the exact command list used, for example:

```bash
just refactor-readiness
just check
just clippy
just test
just contract-test
just verification-full
```

If a command cannot run locally, record the reason and the required environment
or tooling instead of silently omitting it.
