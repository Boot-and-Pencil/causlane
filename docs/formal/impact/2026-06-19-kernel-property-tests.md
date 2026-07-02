# Formal Impact Record: property tests for the kernel decisions

## Change metadata

- Change ID: FIR-2026-06-19-kernel-property-tests
- PR/issue: S03 reference-kernel hardening (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-19
- Impact class: F1 (tests only) — plus a behavior-preserving clippy cleanup

## Touched protocol-critical paths

```text
crates/causlane-core/src/domain/lifecycle.rs
crates/causlane-core/src/domain/authz.rs
crates/causlane-replay/src/authz.rs
```

## Summary

Closes the missing **property** leg of the S03 exit gate ("all protocol-critical
functions covered by unit/property/Kani checks"). No `proptest`/RNG dependency is
added — `causlane-core` stays zero-dep; the tests are deterministic and exhaustive
over the finite/representative input spaces (the same exhaustive-over-bounded-space
approach the Kani lane uses).

- **Lifecycle reducer** (`lifecycle.rs`): a new test enumerates the ENTIRE finite
  `LifecycleStage × AuditEventKind × ConsequenceProfile` space (9×17×6 = 918
  triples) and asserts the grammar's structural invariants for every triple —
  determinism, Closed is absorbing (I-008), only `LifecycleClosed` reaches the
  terminal stage, `is_terminal ⇔ Closed`, `initial_stage == New`. A complete proof.
- **Authz classifier** (`authz.rs`): a new test compares `classify_authz_decision`
  against an INDEPENDENT reference oracle across an 8640-point grid straddling
  every binding / policy / verdict / stage / temporal boundary, pinning that the
  recent structural+temporal dedup faithfully encodes the ADR-0011 rule (the six
  example unit tests only spot-check single defects).

Also folds a behavior-preserving clippy cleanup of the authz match introduced by
the dedup (`Skip => continue` → merged `Skip | Deny(Missing) => {}`); `needless_continue`
+ `match_same_arms` were not caught by the `cargo check` gate. No logic change.

## Affected invariants

```text
I-008: No event may mutate lifecycle after terminal close — now proven exhaustively
       over the full grammar input space (in addition to Kani + replay control).
ADR-0011: authz deny-by-default structural+temporal rule — now pinned by a grid
       differential against an independent reference oracle.
new invariant ids: none
```

## Affected formal models

```text
none — test-only addition. No generated artifact, bundle, IR or coverage change.
```

## Contract changes

- Bundle / Formal IR / replay-trace / receipt / coverage fields: none.
- Core semantic change: none. The authz match cleanup is a no-op
  (`Skip`/`Missing` were already empty/`continue` arms in a trailing match).

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| `lifecycle_grammar_properties_hold_exhaustively` | core unit/property | every (stage,event,profile) triple obeys the grammar invariants | new |
| `classify_authz_decision_matches_reference_oracle` | core unit/property | classify == independent reference over the full grid | new |
| `event_after_closed_invalid` | replay | `EventAfterClosed` (I-008) | existing — unaffected |
| `authz_*_invalid` (6 controls) + `authz_success` | replay | exact authz codes | existing — re-verified (authz cleanup is a no-op) |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit/Property (core) | the two new exhaustive/grid tests | no (hand-rolled) | rust |
| Replay | unchanged | yes | rust |
| Kani | unchanged | yes | rust |
| Verus/Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

The property tests live in the core crate's `#[cfg(test)]` modules; they do not
change any generated lane. They complement (do not replace) the existing Kani
exhaustive-bounded harnesses and replay negative controls.

## Acceptance commands

```bash
just formal-ready
just verification-full
```

Additional commands:

```bash
./tools/cargo-dev test -p causlane-core lifecycle_grammar_properties_hold_exhaustively
./tools/cargo-dev test -p causlane-core classify_authz_decision_matches_reference_oracle
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: property tests for the remaining decisions (ConflictOracle,
  DrainSemantics, CapabilityIssuer, TruthAnchorResolver — bounded spaces already
  Kani-covered); and pre-existing clippy debt in
  `crates/causlane-codegen/src/lean4_target.rs` (from commit db5381b, surfaced by a
  newer clippy; `just clippy` is not part of the mandatory `check-verification-full` gate).
