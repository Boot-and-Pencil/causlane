# Formal Impact Record: route I-006 lease-conflict through KernelContracts

## Change metadata

- Change ID: FIR-2026-06-19-lease-conflict-single-authority
- PR/issue: S03 reference-kernel hardening (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-19
- Impact class: F3 (kernel-invariant enforcement path) — behavior-preserving routing

## Touched protocol-critical paths

```text
crates/causlane-core/src/domain/constraint.rs
crates/causlane-core/src/contract.rs
crates/causlane-replay/src/lib.rs
```

## Summary

`KernelContracts` implements `ConflictOracle` (over `ScopeOverlap`), but the live
I-006 enforcement path — `LeaseTable::grant` — decided conflicts via a private free
fn `leases_conflict` that used **inline `a.scope == b.scope`** for the scope test,
bypassing the authority's `ScopeOverlap::overlaps` extension point. The mode rule
(`claim_modes_conflict`) was already shared by both paths; only the scope-overlap
sub-decision was duplicated. Today both resolve to exact equality, so behavior is
identical — but an extension of `overlaps` (hierarchical/prefix scopes) would have
silently not reached `LeaseTable`, a latent authority split. This routes the
decision through the single kernel authority, completing the declared S03
single-authority list (lifecycle / capability / **lease-conflict** / drain / anchor):

- `LeaseTable::grant` now takes a `&impl ConflictOracle` and calls
  `oracle.leases_conflict(active, &lease, verified_merge)`; the replay oracle (the
  only non-test caller, two sites) supplies `&KernelContracts`. The duplicate-id
  check, the `verified_merge = self.mergeable_scopes.contains(&lease.scope)`
  computation and the `LeaseTableError::Conflict { … }` path are byte-identical.
- The private free fn `leases_conflict` is deleted (the trait default replaces it).
  `claim_modes_conflict` (the shared mode rule the Kani/Verus lanes drive) is
  unchanged.
- The conflict primitive now **cannot decide a conflict without being handed the
  authority** — the single-authority property is structural, not conventional.

## Affected invariants

```text
I-006: No conflicting active leases without a verified merge — enforcement routed
       through KernelContracts (ConflictOracle); the scope-overlap test is now the
       authority's ScopeOverlap::overlaps, not a duplicated equality. Semantics
       UNCHANGED (overlaps == equality in the MVP).
new invariant ids: none
```

## Affected formal models

```text
none — no generated artifact changes. The I-006 negative control already exists and
is re-verified through the new routing; Kani drives the unchanged
`claim_modes_conflict` mode rule directly.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public API: `LeaseTable::grant` gains a `&impl ConflictOracle` parameter
  (workspace-internal; the only non-test caller is the replay oracle).
- Core semantic change: none (behavior-preserving routing).

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| `conflicting_leases_invalid` | Replay | `ConflictingLeases` (decided via the routed `KernelContracts.leases_conflict`) | existing — re-verified via new routing |
| `conflicting_leases_invalid` | Alloy | `GeneratedNoExclusiveConflicts` refuted | existing — unchanged (generated facts) |

No new negative control is required: the behavior under test is unchanged. The
`lease_table_rejects_conflicts_and_validates_claims` and
`lease_table_merge_relaxes_conflict_on_mergeable_scope` core unit tests exercise the
new DI signature (fail-closed conflict + verified-merge relaxation) directly.

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | `conflicting_leases_invalid` (unchanged) | yes | rust |
| Alloy | `GeneratedNoExclusiveConflicts` (unchanged) | yes | rust |
| P | `NoConflictingActiveLeases` (unchanged) | yes | rust |
| Kani | `lease_conflict_rule_is_fail_closed_without_verified_merge` (unchanged — drives `claim_modes_conflict`) | yes | rust |
| Verus | `lease_conflict_is_fail_closed_without_merge` (non_blocking_spec, unchanged) | yes | proof/all |
| Lean4 | `lease_conflict_fail_closed` (planned, unchanged) | yes | proof/all |

No regeneration: the generated artifacts import/model the unchanged
`claim_modes_conflict` rule, not `LeaseTable::grant`'s call shape.

## Not applicable lanes

No lane changes. The routing keeps the replay oracle's I-006 decision identical; the
Alloy/P/Kani/Verus/Lean4 conflict lanes are generated from the same rule and are
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
- Follow-up issue: extending `ScopeOverlap::overlaps` beyond exact equality
  (hierarchical scopes) is now a single-site change that propagates to `LeaseTable`
  automatically — a separate increment. TZ-007 verified-merge **runtime** enforcement
  remains blocked on the S05 frontier/dispatcher.
