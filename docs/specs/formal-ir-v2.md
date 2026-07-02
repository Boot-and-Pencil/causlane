# Formal IR v2

`FormalIr` is the target-neutral contract input for generated Alloy, P, Kani
and Verus artifacts.

Authority remains the compiled dispatch bundle plus scenario catalog. Formal IR
is a deterministic projection of that authority:

```text
compiled bundle + optional scenario -> FormalIr v2 -> target artifacts -> receipts
```

The IR carries:

- `source_bundle_hash`
- `scenario_hash`, when scenario-bound
- predicate route/profile/barrier/projection/authz/lease/witness obligations
- `merge_protocols` — bundle merge-protocol fail-closed `status` (v2)
- scenario event facts, each with producer attestation `fact_kind`/`scope` (v2)
- lease facts with constraint `epoch` for P interleaving checks (M10.2)
- active invariant ids `I-001..I-010`
- `formal_ir_hash`

`formal_ir_hash` is computed with canonical serialization v1 over the IR digest
material excluding the `formal_ir_hash` field itself. Generated artifacts must
carry the same hash in their header, and receipts must bind it through
`formal_ir_hash`.

Schema: `contracts/schema/formal_ir.schema.json`.

M10.1 reserves planned invariant ids `I-011..I-020` in the formal obligation
manifest. They are not accepted in Formal IR until a later slice promotes them
to active ids with concrete checks and coverage evidence.

## v1 → v2 migration

v2 is a strict superset of v1; `schema_version` is `2`. It adds **faithful
projections of facts the replay oracle already enforces**, so the IR no longer
drops them on the way to the generators:

- `FormalIr.merge_protocols: [{ protocol_id, version, status, permits_concurrency }]`
  — only a `verified` protocol permits overlapping mutable writes; every other
  status (`absent` / `declared_but_unverified` / `disabled`) fails closed (I-006).
- `FormalEvent.fact_kind` / `FormalEvent.scope` — the producer attestation a
  `gate.approved` / `observed_truth.committed` event records about itself, which
  the replay oracle grounds witness/anchor claims against (P0-004). A witness ref
  or projection anchor may not self-assert a fact its producer event never
  recorded.
- `FormalEvent.anchors` — structured (`{ event_id, fact_kind, scope }`) rather
  than bare event ids, so a generator can ground a projection's claimed
  `fact_kind`/`scope` against the observed-truth event's attestation.
- `FormalLeaseFact.epoch` — the constraint epoch already present in scenario
  lease refs, projected into P so stale-epoch admission controls can be checked
  without inventing a second constraint model.

These fields are projections of enforced truth, not a new specification. The
generator-side consumers (Alloy/P witness-fact, merge assertions and M10.2 P
interleaving controls) bind to this payload.

The acceptance gate is:

```bash
just formal-ready
just verification-full
```

`check-verification-full` writes:

- `verification/formal-full/ir/generated/*.formal_ir.json`
- `verification/formal-full/alloy/generated/*.als`
- `verification/formal-full/p/generated/*.p`
- `verification/formal-full/kani/generated/*.rs`
- `verification/formal-full/verus/generated/*.rs`
- target codegen/tool-run receipts under `verification/formal-full/receipts/`
- `target/causlane/formal-coverage-report.json`
