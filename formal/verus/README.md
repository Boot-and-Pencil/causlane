# Verus lane (FM-004) — always-on proof lane

The Verus lane proves abstract preservation: *"if an abstract Causlane kernel
state is valid, an admissible transition preserves validity and the key safety
laws."*

## Current status: always-on, blocking

`just formal-verify-all` runs `verus --no-cheating` over the generated Verus
artifact on **every** run and records the real tool-run receipt; a non-zero exit
fails the gate. The `LANE_REALITY_VERUS_NON_BLOCKING` exception was dropped
2026-06-21. Every run requires the pinned Verus binary and fails if it is
unavailable. The fast dev loop (`just formal-ready`, `cargo test`, `clippy`) does
not run Verus.

## Proof catalog

[`PROOF_CATALOG.md`](PROOF_CATALOG.md) lists the proof groups required before
the Verus lane can be treated as authoritative under proof/all profiles. The
catalog is a checklist and does not replace `verus --no-cheating`, generated
receipts, stale-check or coverage reconciliation.

## Planned layout (FM-004 deliverables)

```
formal/verus/core/kernel_state.rs       KernelState abstract state
formal/verus/core/transitions.rs        transition relation
formal/verus/core/invariants.rs         valid_state predicate (I-001..I-010)
formal/verus/core/preservation.rs       transition_preserves_validity
formal/verus/core/replay_soundness.rs   replay_accepts_implies_valid_protocol_trace
```

`formal/verus/generated/release_promote_success.rs` is generated from Formal IR
and is verified on every `formal-verify-all` run (a required, blocking lane).

## Unblocking

```bash
rustup toolchain install 1.95.0-x86_64-unknown-linux-gnu
verus --version   # should succeed
```

The pinned toolchain is installed via `just formal-install verus` / `just
doctor-full`; `tools/formal-verify-all` then verifies Verus on every run.
