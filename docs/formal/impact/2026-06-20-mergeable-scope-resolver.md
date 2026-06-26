# Formal Impact Record: single mergeable-scope resolver (dispatcher-012 P0-005, part 1)

## Change metadata

- Change ID: FIR-2026-06-20-mergeable-scope-resolver
- PR/issue: dispatcher-012 ТЗ, DoD fix stage (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-20
- Impact class: F3 (enforcement-path consolidation) — behavior-preserving; no
  invariant semantics change

## Touched protocol-critical paths

```text
crates/causlane-contracts/src/bundle.rs
crates/causlane-contracts/src/lib.rs
crates/causlane-replay/src/lib.rs
```

## Summary

The replay oracle resolved the I-006 verified-merge conflict-domain scopes with a
private `resolve_mergeable_scopes`, while the Alloy generator modelled `mergeable`
separately (and pinned it empty). To let both lanes consume one merge truth (the
P0-005 goal), the resolver moves to `causlane-contracts` as the single authority:

- New `causlane_contracts::resolve_mergeable_scopes(bundle, predicate, bindings) ->
  Vec<String>` (sorted scope tokens): per effect template, when
  `merge_decision(...).permits_concurrency()` holds, resolve the template's
  conflict-domain expressions against the bindings. Fail-closed when no verified
  applicable protocol matches.
- Replay's `resolve_mergeable_scopes` is now a thin wrapper over the contracts
  resolver (mapping `Vec<String>` → `HashSet<Scope>`), so it cannot diverge from
  what the Alloy generator will consume in part 2.

Behavior is identical: the existing bundles declare no applicable verified merge
protocol, so the resolved set stays empty and every overlapping exclusive lease
still conflicts (the `conflicting_leases_invalid` control still refutes).

## Affected invariants

```text
I-006 (lease conflict / verified merge): the mergeable-scope resolution is now
single-sourced in contracts. Semantics unchanged.
new invariant ids: none
```

## Affected formal models

```text
none yet — this part only consolidates the resolver. Part 2 makes the Alloy
`GeneratedNoExclusiveConflicts` assertion consume it per-scope and adds the
unrelated-merge negative control.
```

## Contract changes

- New public `resolve_mergeable_scopes`.
- Bundle / Formal IR / trace / receipt / coverage fields: none.
- Core semantic change: none (behavior-preserving).

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| `conflicting_leases_invalid` | Replay | `ConflictingLeases` | existing — re-verified through the consolidated resolver |
| `release_promote_success` | Replay | pass | existing — re-verified |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Contracts/Replay | shared `resolve_mergeable_scopes` | n/a | rust |
| Alloy | unchanged (part 2) | yes | rust |

## Not applicable lanes

No generated-model change in this part.

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-replay
just formal-verify-all
```

## Exception request

- Exception needed? no
- Follow-up: P0-005 part 2 — make the Alloy conflict assertion scope-keyed (relax
  only on mergeable scopes) and add the `unrelated_merge_protocol_does_not_allow_conflict_invalid`
  negative control on an applicable-merge bundle.
