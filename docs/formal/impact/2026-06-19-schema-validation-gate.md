# Formal Impact Record: schema sync + validation gate (dispatcher-012 P0-003/004/008)

## Change metadata

- Change ID: FIR-2026-06-19-schema-validation-gate
- PR/issue: dispatcher-012 ТЗ, DoD fix stage (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-19
- Impact class: F2 (schema/IR contract change) — adds a fail-closed schema gate; no
  kernel/invariant behavior change

## Touched protocol-critical paths

```text
contracts/schema/formal_ir.schema.json
contracts/schema/scenario.schema.json
crates/causlane-cli/src/cli_parse.rs
crates/causlane-cli/src/main.rs
tools/check-json-no-duplicate-keys   (new)
tools/validate-json-schema           (new)
tools/schema-validate-all            (new)
scripts/check-verification-full.sh
```

## Summary

The contract JSON Schemas under `contracts/schema/` were documentation only —
nothing validated fixtures or generated artifacts against them, so they had
silently drifted from the Rust DTOs (the source of truth). This lands the three
coupled dispatcher-012 schema fixes plus a fail-closed validation gate so drift
can no longer hide.

- **P0-004** — `formal_ir.schema.json` had two `$defs.witness` definitions (a
  requirement shape and a payload shape); JSON keeps only the last, so `$ref`s
  could resolve to the wrong one. Split into `witness_requirement` (predicate
  requirement) and `witness_payload` (materialized barrier/event witness); all
  `$ref`s updated. The emitted Formal IR validates against the de-duplicated schema.
- **P0-003** — `scenario.schema.json` synced to the scenario DTO + fixtures:
  `event_kind` gains `drain.fence_requested`/`drain.fence_acquired`; `claim_mode`
  `read` → `token` (the DTO is `exclusive`/`shared`/`token`); event `occurred_at`
  (integer ≥ 0) allowed (all authz fixtures use it); optional `attestation` added to
  `authz_decision`/`execution_capability` for DTO parity (`ReplayAuthzDecision`/
  `ReplayExecutionCapability` carry it).
- **P0-008** — schema validation is now a mandatory gate step (before any formal
  generation):
  - `tools/check-json-no-duplicate-keys` (Python stdlib) fails on duplicate keys at
    any depth in every `contracts/schema/*.json` (the P0-004 bug class);
  - `causlane scenario validate <scenario.yaml>` parses each scenario through the
    typed DTO (authoritative for enum/required/type errors); run over the whole
    corpus;
  - `tools/validate-json-schema` (Python stdlib `json`+`re`) validates the emitted
    Formal IR against `formal_ir.schema.json`;
  - `tools/schema-validate-all` orchestrates these and is wired into
    `scripts/check-verification-full.sh`.

The validator lives in Python/bash by design: the workspace is intentionally
dependency-lean (serde only) and the Rust discipline forbids dynamic
`serde_json::Value` / stringly logic in core, so a generic Rust schema interpreter
is the wrong vehicle. Scenarios (YAML; the toolchain has no PyYAML) are validated
through the typed Rust DTO; JSON artifacts go through the stdlib Python validator.

## Affected invariants

```text
none — this is contract-schema hygiene + a validation gate. No invariant semantics,
coverage, replay decision, or formal model changes.
new invariant ids: none
```

## Affected formal models

```text
none — `formal_ir.schema.json` is renamed internally ($defs split) with the same
structure; the emitted Formal IR is byte-unchanged and still validates. No
generated artifact (Alloy/P/Kani/Verus/Lean4) changes; no regeneration.
```

## Contract changes

- Formal IR schema: `$defs.witness` split into `witness_requirement` /
  `witness_payload` (no shape change to the emitted IR).
- Scenario schema: `event_kind`/`claim_mode` enums corrected; `occurred_at` and
  `attestation` allowed (all already accepted by the DTO).
- New CLI subcommand `causlane scenario validate` (typed parse; no new dependency).
- Bundle / Formal IR data / replay-trace / receipt / coverage fields: none.
- Core semantic change: none.

## Required negative controls

| Check | Input | Expected | Status |
|---|---|---|---|
| `check-json-no-duplicate-keys` | JSON with a duplicated key | rc=1, names the key | new — verified |
| `validate-json-schema` | Formal IR with a required field dropped + an unknown field | rc=1, reports both | new — verified |
| `scenario validate` | scenario with `kind: bogus.kind` | rc≠0, unknown-variant error | new — verified |
| `schema-validate-all` | the live tree | passes (all schemas dup-free, 24 scenarios valid, IR conforms) | new — verified |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Schema gate | `tools/schema-validate-all` (dup-keys + scenario DTO + IR-vs-schema) | n/a | base/rust |
| All formal lanes | unchanged | yes | rust/proof |

## Not applicable lanes

No formal lane changes. The schema gate runs before generation; the generated
artifacts and their receipts/coverage are unaffected.

## Acceptance commands

```bash
tools/schema-validate-all
python3 tools/check-json-no-duplicate-keys contracts/schema/*.json
just verification-full
```

## Exception request

- Exception needed? no
- Follow-up issue: scenario YAML is validated through the typed DTO rather than
  directly against `scenario.schema.json` (no PyYAML in the toolchain); the DTO is
  the source of truth. A future increment could add YAML→JSON emission to also
  validate scenarios against the JSON Schema directly.
