# Formal Impact Record: Invariant catalog bootstrap (M10.1)

## Change metadata

- Change ID: FIR-2026-06-24-invariant-catalog-bootstrap
- PR/issue: S10 / M10.1 Invariant expansion
- Owner: repo maintainers
- Date: 2026-06-24
- Impact class: F2 (formal catalog/schema boundary, no new proof claim)

## Touched protocol-critical paths

```text
contracts/schema/common.schema.json
contracts/schema/formal_coverage_report.schema.json
contracts/schema/formal_exceptions.schema.json
contracts/schema/formal_ir.schema.json
contracts/schema/formal_obligation_manifest.schema.json
crates/causlane-contracts/src/invariants.rs
crates/causlane-contracts/src/bundle.rs
crates/causlane-codegen/src/ir.rs
crates/causlane-codegen/src/obligations.rs
crates/causlane-cli/src/bin/formal_discipline/manifest.rs
verification/formal-full/obligations/lifecycle_product_obligations.yaml
tools/validate-json-schema
```

## Summary

M10.1 starts with a catalog bootstrap instead of adding new proof credit. The
repo now has one Rust invariant-id catalog in `causlane-contracts`:

```text
active:  I-001..I-010
planned: I-011..I-020
known:   I-001..I-020
```

Compiled bundle formal obligations and Formal IR still accept only active ids.
The formal obligation manifest may name planned ids, but planned lanes do not
produce coverage rows and do not count as evidence.

## Affected invariants

```text
I-001..I-010: unchanged - active coverage, scenarios, receipts and generated
              artifacts keep their existing authority.
I-011..I-020: reserved as planned ids only - no active coverage claim and no
              generated proof/replay authority.
new invariant ids: I-011..I-020 reserved, not covered.
```

## Affected formal models

```text
FM-000 Authority-chain model: strengthened by a shared invariant-id catalog.
FM-015 Receipt/coverage/exception model: strengthened by schema reuse.
FM-014/FM-018/FM-019/FM-020/FM-021/FM-022: planned manifest reservations only.
No generated Alloy/P/Kani/Verus/Lean artifacts are changed.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public Rust API added:
  `ACTIVE_INVARIANT_IDS`, `PLANNED_INVARIANT_IDS`,
  `is_active_invariant_id`, `is_planned_invariant_id`,
  `is_known_invariant_id`.
- JSON schema support added:
  `contracts/schema/common.schema.json` with active/known invariant id defs.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| Bundle predicate declares `I-011` | contracts unit | compile rejects planned id as inactive | new |
| Formal IR receives `I-011` | codegen unit | builder rejects planned id as inactive | new |
| Formal discipline manifest declares `I-021` | CLI unit | manifest rejects unknown id | new |

## Deferred controls

| Scenario | Deferred to | Reason |
|---|---|---|
| I-011..I-020 proof/check implementations | M10.1 follow-up slices / M10.2 | this slice reserves ids and removes duplicate validators only |
| Coverage rows for I-011..I-020 | future proof slices | no concrete check ids exist yet |

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-contracts --all-targets --all-features --locked invariant
./tools/cargo-dev test -p causlane-codegen --all-targets --all-features --locked invariant
./tools/cargo-dev test -p causlane-cli --all-targets --all-features --locked formal_discipline
./tools/schema-validate-all
./tools/cargo-dev fmt --all --check
git diff --check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m10-1-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: add concrete proof/replay/runtime slices for selected
  I-011..I-020 invariants before any coverage claim is made.
