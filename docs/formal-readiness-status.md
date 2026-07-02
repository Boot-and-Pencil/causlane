# Formal readiness status

Status date: 2026-06-24. Supersedes [`formal-readiness-final.md`](formal-readiness-final.md).

**The authority is the machine gate, not this prose.** `scripts/check-verification-full.sh`
derives `target/causlane/formal-coverage-report.json` from generated artifacts,
real tool-run receipts and real exit codes; [`docs/invariants/coverage-matrix.json`](invariants/coverage-matrix.json)
is a machine-derived projection, and `tools/coverage-matrix --check` fails if
the JSON or Markdown coverage matrix overclaims a lane the report does not back.
This file is a status summary for readers, not an independent source of proof.

## Current authority chain

```text
registry.yaml -> compiled bundle (+ bundle_hash) -> scenario/trace
  -> Formal IR v2
  -> generated Alloy/P/Kani/Verus/Lean4 artifacts
  -> codegen receipts + tool-run receipts
  -> stale-check-all
  -> derived coverage report
  -> coverage-matrix (drift-checked)
```

Every counted link is content-addressed. Emitted scenario traces are bound to
the bundle hash and verified strictly (`--require-bundle-hash`), so a trace
cannot be accepted as evidence for a bundle it is not tied to. Generated Lean4
artifacts are now part of the same source-bound contour as Alloy/P/Kani/Verus;
hand-written Lean support files are not authority by themselves.

## Current vs planned state

The repository currently has operational generation/gate support for:

```text
Replay oracle
Alloy generated facts
P generated monitors
Kani generated harnesses
Verus generated proof artifact
Lean4 generated theorem applications
receipts v2
stale-check-all
coverage derivation
formal exceptions policy
```

`tools/formal-discipline-check` is now **implemented as local/PR-diff
discipline enforcement**, backed by the `causlane-formal-discipline` CLI binary.
It now enforces strict check-id adequacy across the obligation manifest,
generated artifact text or replay negative controls, receipts, coverage and docs.
It is mandatory inside the repo strict gate: `scripts/check-verification-full.sh` runs it
after receipt-derived coverage and `tools/coverage-matrix --check`. Provider
CI can also run it in PR-diff mode with `--from-git` or `--changed-files`, but
no provider workflow is checked into this repository.

Current gate note (2026-06-21): the default `just verification-full` gate passes
locally with receipt-derived coverage `status=pass`, all generated targets fresh,
and the coverage matrix drift check clean. The Verus and Lean4 proof lanes are now
**always run and blocking** on every gate run (their time-boxed exceptions were
dropped 2026-06-21). With the formal contour, S03 reference kernel, S04
scenario/replay contract testing, S05 constraint/frontier, S06 authz/approval
and S07 observability foundations advanced in repo, the current product-track
focus is S08/S09: runtime/adapters and performance/reliability. The practical
next closure item is M09.7 chaos/recovery tests, followed by S10 formal-depth
and proof hardening. Use
[`docs/product-track/11-implementation-start-gate.md`](product-track/11-implementation-start-gate.md)
as the handoff checklist before starting a milestone branch.

## Per-invariant coverage

See [`coverage-matrix.md`](invariants/coverage-matrix.md) /
[`coverage-matrix.json`](invariants/coverage-matrix.json). An invariant is
`covered` only when at least one required lane carries a concrete named
`check_id` present in generated artifacts and fresh receipts. A lane cell may be
`not_applicable` where that lane honestly does not model the invariant. Verus
and Lean4 are always-blocking proof lanes; a missing proof obligation must be
represented as `not_applicable` or future work, not as hidden coverage.

## Proof/refinement scope

See generated [`formal/08-proof-refinement-scope.md`](formal/08-proof-refinement-scope.md)
and its schema-validated JSON source
[`formal/proof-refinement-scope.json`](formal/proof-refinement-scope.json). That
artifact classifies evidence strength (`proved`, `bounded`, `simulated`,
`tested`, `assumed`, `out_of_scope`) without granting coverage credit or
restating live invariant cells.

## Producer attestation — grounded in three runtime/model lanes

A witness ref or projection anchor may not self-assert a `fact_kind`/`scope` its
producer / observed-truth event never recorded. Enforced by:

| | replay | Alloy | P |
|---|---|---|---|
| witness fact grounding | `WitnessAttestationMismatch` | `GeneratedWitnessFactGrounded` | `WitnessFactGrounded` |
| anchor fact grounding | `AnchorAttestationMismatch` | `GeneratedAnchorFactGrounded` | `AnchorFactGrounded` |

The Formal IR v2 carries the producer attestation (`fact_kind`/`scope` on events)
and structured anchors, so generators ground the same payload the oracle checks
instead of trusting self-asserted refs.

## Authz — grounded across lanes by division of labor

| authz defect | replay | Alloy | P |
|---|:--:|:--:|:--:|
| referenced `Deny` | ✅ | ✅ | ✅ |
| no referenced decision | ✅ | ✅ | — backstopped by replay |
| wrong `policy_id`/version | ✅ | structural-pass | structural-pass |
| expired / issued-after-barrier / stale | ✅ | — | — |

Replay is the executable authority for temporal and policy facts: action/plan /
predicate binding, declared `policy_id`/`policy_version`, Deny-wins, freshness,
expiry and issued-before-barrier. Alloy and P intentionally model only the
structural requirement that a `RuntimeExecution` barrier references a bound
`Allow`; they do not claim timestamp/freshness coverage.

## Negative-control inventory

- **Replay:** 16 main `*_invalid` controls plus 6 authz controls, each refuted
  with an exact stable error code via the executable oracle.
- **Alloy:** 11 main plus 2 authz structural controls, including wrong
  witness/anchor fact and scope grounding, an event after `LifecycleClosed`
  (I-008, refuted via the `ClosedIsTerminal` clause of `Enforced`), and a drain
  fence over a scope with an active overlapping exclusive lease (I-007, refuted
  via `GeneratedDrainFenceClear`; structural — replay keeps the expiry refinement).
- **P:** 5 main plus 1 authz structural control, including wrong witness/anchor
  fact and scope grounding.
- A negative control counts only when refuted for the expected reason; accidental
  failure is not evidence.

## Verus status

Verus is an **always-on, blocking** proof lane: `scripts/check-verification-full.sh` runs
`verus --no-cheating` over the generated proof artifact on **every** run (the
`LANE_REALITY_VERUS_NON_BLOCKING` exception was dropped 2026-06-21) and records the
real exit code in the tool-run receipt; a non-zero exit fails the gate. The current
generated Verus artifact contains non-vacuous event-indexed preservation lemmas for
lifecycle predecessors and closed terminality, plus rule lemmas for overlay, lease
conflicts, route/profile compatibility, drain-fence clearance, witness binding
and constraint-update truth preservation. The prose summary does not replace the
receipt.

## Lean4 status

Lean4 is now part of the generated proof contour, not merely a future ambition.
`FormalTarget::Lean4` emits scenario-bound theorem applications from Formal IR
into `verification/formal-full/lean4/generated/*.lean`, imported by the repo-local Lake package
under `verification/formal-full/lean`.

Current implemented Lean4 coverage is intentionally not listed here by hand.
Use the generated [`coverage-matrix.md`](invariants/coverage-matrix.md),
[`coverage-matrix.json`](invariants/coverage-matrix.json) and backing coverage
report as the authority for covered invariant cells and named theorem
`check_id`s.

This lane is **always run and blocking**: `scripts/check-verification-full.sh` runs both
commands below on **every** run (the `LANE_REALITY_LEAN4_NON_BLOCKING` exception was
dropped 2026-06-21), and a non-zero exit fails the gate:

```bash
(cd verification/formal-full/lean && ../../../tools/lean4-env lake build CauslaneFormal)
(cd verification/formal-full/lean && ../../../tools/lean4-env lake env lean ../lean4/generated/release_promote_success.lean)
```

The tool-run receipt, not this paragraph, is the authority for whether the
proof/all Lean4 run passed. Catalog entries that mention Lean4 for future
models are target-state obligations, not current coverage.

## Deliberately not yet claimed

- **Provider-specific PR CI enforcement:** `tools/formal-discipline-check` is
  mandatory inside `check-verification-full`; external CI still needs a workflow that
  supplies `--from-git` or `--changed-files` for PR-diff checks. For formal
  lane execution, external CI should call the repo-local
  `scripts/check-verification-full.sh --depth <lane>` wrapper rather than duplicating Kani runner
  policy.
- **Authz provider strict-mode:** provider identity / dev-exemption expiry are
  deferred because the current decision/policy payloads do not yet carry enough
  provider data.
- **Authz temporal/policy in Alloy/P/Lean4:** replay remains authority for
  expiry/freshness. Other lanes model structural relations unless explicitly
  extended.
- **Cryptographic journal hash-chain:** capability/authz attestations exist, but
  journal-level `prev_event_hash` and per-event cryptographic content pins remain
  outside the current semantic proof contour.
- **Lean4 beyond generated coverage-matrix rows:** listed in catalogs as target
  obligations only.
- **Abstract proof lanes for mixed-predicate traces:** the replay verifier now
  supports cross-action anchor grounding (a `RuntimeExecution` producer plus a
  `ProjectionRead` reader in one trace; M04.1 projection / read-only sidecar). It
  is covered by the **replay oracle** (positive + non-vacuity controls); Alloy/P/Lean
  modeling of cross-action anchor grounding is S10 proof-hardening follow-up. The
  fixture predicate (`contracts/examples/projection_readonly.registry.yaml`) is not
  wired into the production formal contour, so existing receipts are unaffected.

## Run it

Default implementation gate:

```bash
just verification-full
just formal-coverage
just formal-coverage-matrix-check
tools/formal-discipline-check --profile rust --no-diff --json
```

Proof-capable gate:

```bash
tools/full-doctor --json --profile proof
scripts/check-verification-full.sh --profile proof
```

Lean4 quick probe when debugging the proof lane:

```bash
tools/formal-install lean4
tools/lean4-env lake --version
(cd verification/formal-full/lean && ../../../tools/lean4-env lake build CauslaneFormal)
(cd verification/formal-full/lean && ../../../tools/lean4-env lake env lean ../lean4/generated/release_promote_success.lean)
```
