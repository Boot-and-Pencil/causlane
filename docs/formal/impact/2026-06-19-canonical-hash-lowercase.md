# Formal Impact Record: canonical lowercase hash validation (dispatcher-012 P1-004)

## Change metadata

- Change ID: FIR-2026-06-19-canonical-hash-lowercase
- PR/issue: dispatcher-012 ТЗ, DoD fix stage (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-19
- Impact class: F2 (contract input validation) — tightens hash-token acceptance; no
  invariant/model change

## Touched protocol-critical paths

```text
crates/causlane-contracts/src/plan_hash.rs   (new shared validator + tests)
crates/causlane-contracts/src/lib.rs
crates/causlane-contracts/src/bundle.rs
crates/causlane-codegen/src/alloy.rs
crates/causlane-replay/src/trace_lowering.rs
contracts/schema/scenario.schema.json
```

## Summary

Three independent hash-token validators (`bundle.rs::require_sha`,
`alloy.rs::validate_hash`, `trace_lowering.rs::validate_hash_token`) each accepted
`sha256:` + 64 hex via `is_ascii_hexdigit()`, which permits **uppercase** hex. The
hasher always mints lowercase (`{byte:02x}`), so accepting uppercase would let two
distinct strings denote the same digest — a canonicalization hole.

Unify on a single validator and enforce canonical lowercase
`sha256:[0-9a-f]{64}`:

- New `causlane_contracts::is_canonical_sha256_token` (the one shared predicate),
  reused by `require_sha`, `validate_hash` and `validate_hash_token`.
- `scenario.schema.json` `sha256` pattern tightened `[0-9a-fA-F]` → `[0-9a-f]`.

All minted/committed hashes are already lowercase (verified: no uppercase
`sha256:` token exists in `contracts/`), so this rejects only malformed input.

## Affected invariants

```text
none — content-hash token validation hygiene. No invariant semantics, coverage,
replay decision or formal model change.
new invariant ids: none
```

## Affected formal models

```text
none — no generated artifact change; no regeneration. The Alloy hash-field
validation now shares the canonical predicate but emits identical facts.
```

## Contract changes

- New public helper `is_canonical_sha256_token`.
- Hash-token acceptance: uppercase hex now rejected (lowercase canonical only).
- Bundle / Formal IR / trace / receipt / coverage fields: none.
- Core semantic change: none for canonical (lowercase) input.

## Required negative controls

| Check | Input | Expected | Status |
|---|---|---|---|
| `is_canonical_sha256_token` unit test | `SHA256:…`, `sha256:ABC…`, `sha256:TODO`, 63-char, 65-char, no prefix | all rejected; lowercase 64-hex accepted | new — verified |
| `just verification-full` | live tree (all lowercase) | green | re-verified |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Contracts/codegen/replay | shared `is_canonical_sha256_token` | n/a | base/rust |
| All formal lanes | unchanged | yes | rust |

## Not applicable lanes

No formal lane change; the validators feed the same generated artifacts unchanged.

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-contracts
just verification-full
```

## Exception request

- Exception needed? no
- Follow-up issue: the HMAC attestation tags (`attestation.rs`/`hmac.rs`) are a
  separate hex domain (not `sha256:` content hashes) and are out of scope here.
