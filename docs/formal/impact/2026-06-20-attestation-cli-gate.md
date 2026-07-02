# Formal Impact Record: attestation CLI gate (dispatcher-012 P1-006)

## Change metadata

- Change ID: FIR-2026-06-20-attestation-cli-gate
- PR/issue: dispatcher-012 ТЗ follow-ups (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-20
- Impact class: F4 (evidence path) — exposes the existing attested-replay check as a
  checkable gate; no kernel/invariant semantics change

## Touched protocol-critical paths

```text
crates/causlane-replay/src/trace.rs          (mint_capability_attestations)
crates/causlane-cli/src/cli_parse.rs
crates/causlane-cli/src/main.rs
scripts/check-verification-full.sh
docs/formal/dispatcher-012-tz-status.md
```

## Summary

Replay already had `verify_with_bundle_attested(bundle, kernel_secret)` (a capability
must carry a valid keyed HMAC attestation), but the CLI had no way to invoke it and
the gate did not exercise it. This wires it into a checkable gate:

- `causlane replay verify --kernel-secret <secret>` runs **attested** verification
  (raw secret bytes → `verify_with_bundle_attested`).
- `causlane scenario emit-trace --kernel-secret <secret>` **mints** valid keyed
  attestations into the emitted trace via the new
  `ReplayTrace::mint_capability_attestations` (lower each capability DTO to the core
  `ExecutionCapability`, `attest(secret, &cap.attestation_message())`), so a positive
  attested fixture is produced without a separate minting tool.
- `scripts/check-verification-full.sh` gains an attestation block: a minted capability passes
  attested verify; a missing attestation (emitted without a secret) and a wrong one
  (minted under a different secret) both refute with `CapabilityMismatch`.

## Non-vacuity proof (anti-theatre)

The attested path is discriminating, verified in the gate every run:

- minted capability → attested verify **passes**;
- missing attestation → **refuted** (`CapabilityMismatch: missing capability
  attestation`);
- wrong attestation (different secret) → **refuted** (`CapabilityMismatch: invalid
  capability attestation`);
- the unminted trace still passes plain (no-secret) verify, so the default gate slice
  is unaffected.

## Affected invariants

```text
I-009 (evidence binding): the keyed-capability-attestation refinement is now a
checkable CLI gate. Kernel semantics unchanged (the verifier already existed).
new invariant ids: none
```

## Affected formal models

```text
none — no generated artifact change. The attestation crypto is unit-tested in
`crates/causlane-contracts/src/attestation.rs`; this adds the replay-integration gate.
```

## Contract changes

- New CLI flag `--kernel-secret` on `replay verify` and `scenario emit-trace`.
- New `ReplayTrace::mint_capability_attestations` (mints into the trace DTO).
- Bundle / Formal IR / receipt / coverage fields: none. Core semantic change: none.

## Required negative controls

| Check | Lane | Expected | Status |
|---|---|---|---|
| minted capability attested verify | Replay (gate) | pass | new — verified |
| missing attestation under `--kernel-secret` | Replay (gate) | `CapabilityMismatch` | new — verified |
| wrong attestation (different secret) | Replay (gate) | `CapabilityMismatch` | new — verified |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | attestation gate block (minted/missing/wrong) | n/a | rust |

## Not applicable lanes

Alloy/P/Kani/Verus/Lean4 unchanged; attestation is a replay/runtime evidence check.

## Acceptance commands

```bash
just verification-full
causlane scenario emit-trace --scenario … --bundle … --kernel-secret S --out t.json
causlane replay verify --bundle … --trace t.json --kernel-secret S
```

## Exception request

- Exception needed? no
- Follow-up: authz-decision attestation minting (the symmetric authz path) could be
  added the same way; out of scope here.
