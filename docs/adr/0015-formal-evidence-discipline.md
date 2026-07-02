# ADR-0015: Formal evidence discipline and anti-theatre gates

- Status: proposed
- Date: 2026-06-17
- Supersedes: none
- Related: ADR-0006, ADR-0014

## Context

Causlane uses formal artifacts to protect a small dispatch kernel: lifecycle, barrier, capability, witnesses, authz, leases, truth anchors, replay and generated evidence. The major risk is not absence of models; it is formal theatre: models proving facts that are not mechanically connected to the runtime contract.

## Decision

A formal claim is accepted only if it is derived from the authority chain:

```text
compiled bundle + scenario -> Formal IR -> generated artifact -> receipt -> stale-check -> coverage
```

Protocol-critical changes require a Formal Impact Record before implementation. Coverage is machine-derived. Exceptions are executable, profile-bound and expiry-bound. Hand-written generic models are allowed only as reusable cores; bundle-specific facts must be generated.

## Consequences

Benefits:

```text
fewer silent drifts between code and proof
clear owner/lane/check_id per invariant
negative controls discriminate vacuous models
docs cannot overclaim coverage
```

Costs:

```text
new PR ceremony for protocol-critical work
obligation manifest and discipline checker to maintain
Lean4/Verus proofs must be connected to generated facts or code-facing contracts
```

## Enforcement

Current repo 010 enforcement:

1. `scripts/check-verification-full.sh` remains the main evidence gate.
2. `tools/coverage-matrix --check` prevents coverage docs drift.
3. `tools/formal-exceptions-check` prevents expired waivers.
4. Proof profiles forbid cheating constructs through the active proof gate.
5. New invariants require negative controls or explicit not-applicable reasoning.
6. `tools/formal-discipline-check` is mandatory inside `scripts/check-verification-full.sh`
   after fresh coverage derivation and coverage-matrix checking.

Provider-specific CI may additionally run `tools/formal-discipline-check` with
`--from-git` or `--changed-files` for PR-diff enforcement. This repository does
not define a provider workflow; the mandatory checked-in integration is the
repo gate.

## Non-goals

This ADR does not require proving external workflow engines, job queues, schedulers, cloud APIs or cryptographic hardness. Those are outside the semantic dispatch kernel and are modeled only through port contracts or explicit assumptions.
