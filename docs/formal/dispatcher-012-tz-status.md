# Dispatcher-012 ТЗ — Execution Status

- Source: `causlane_dispatcher_012_fix_tz_ru.zip` (review vs the `dispatcher-012`
  snapshot, dated 2026-06-19)
- Branch: `hardening/found-problems`
- Executed scope: the ТЗ's **Definition of Done §9** — the genuinely-open P0 items
  plus the two DoD-supporting P1s. The live branch was already ahead of the snapshot,
  so several items needed no work.

**Legend:** ✅ done this fix-stage · ✔ already-resolved (live branch ahead of the
snapshot) · ◑ partial · ⏸ out of scope (model-deepening / deferred)

## P0 — blocking
| Item | Title | Status | Commit / FIR |
|---|---|---|---|
| P0-001 | CLI binaries `causlane-formal[-discipline]` | ✔ already-resolved | binaries exist + gate invokes them |
| P0-002 | toolchain / provisioning / exec-bits | ✔ already-resolved | tool-versions.json, rust-toolchain.toml, scripts 100755 |
| P0-003 | scenario schema drift (drain/token/occurred_at) | ✅ done | `3a1233a` · FIR 2026-06-19-schema-validation-gate |
| P0-004 | duplicate `$defs.witness` in formal_ir.schema | ✅ done | `3a1233a` · split → witness_requirement/witness_payload |
| P0-005 | Alloy `mergeable` semantics + unrelated-merge control | ✅ done | `07caf36` (part 1: shared resolver) + `5d95eda` (part 2: scope-keyed assertion + control) · FIRs mergeable-scope-resolver, alloy-scope-keyed-mergeable |
| P0-006 | legacy `AuditEvent.witnesses` reconciliation | ✅ done | `b922f2f` · FIR legacy-witness-reconciliation |
| P0-007 | coverage/obligations honesty (I-010 not overclaimed) | ✔ already-resolved | coverage derived from receipts; I-010 not_applicable on replay/alloy |
| P0-008 | schema validation as a mandatory gate | ✅ done | `3a1233a` · `causlane scenario validate` + check-json-no-duplicate-keys + schema-validate-all wired into formal-verify-all |
| P0-009 | manual_core vs bundle_generated receipts | ✔ already-resolved | source_bundle_hash=null only for manual-core; not counted as bundle coverage |

## P1 — important before extending formal models
| Item | Title | Status | Commit / FIR |
|---|---|---|---|
| P1-001 | P model = real interleaving state machines | ✅ done | **part 1 done:** the 9 lifecycle/approval/authz/truth monitors are keyed by action (`map[string,bool]`), so they are interleaving-correct (a close/barrier/deny for one action no longer affects another); 4 new lifecycle P negative controls (execution/observed/projection/closed) prove non-vacuity. **part 2 done:** the lease/drain monitors are keyed by scope — `NoConflictingActiveLeases` by `leaseScope`, `DrainBlocks` by scope — with lease events expanded one P send per lease (new `EventPayload` lease fields). The key is load-bearing: `release_promote_success` (two exclusive leases on different scopes) still passes only because of the keying, `conflicting_leases_invalid` (same scope) refutes, and the new `lease_during_drain_invalid` refutes the keyed drain monitor. **part 3 done:** per-action `ActionDriver` machines plus a `ScenarioBootstrap` main replace the single `ScenarioDriver` and the empty role stubs, so P's scheduler interleaves the actions under the keyed monitors; the new cross-action control `multi_action_cross_action_barrier_invalid` refutes (a flat monitor would be fooled when act_x's barrier is seen first), and `multi_action_reference` passes as a two-action positive. FIR `2026-06-21-p-action-sharded-machines` |
| P1-002 | Alloy multi-action / multi-plan scenarios | ✅ done | generator emits one `Action` sig per distinct `action_id` and one `Plan` per distinct plan hash (union, scoped `exactly N`); the `multi_action_alloy` integration test proves a 2-action/2-plan scenario; single-action output is unchanged |
| P1-003 | shared claim-coverage validator | ✅ done | `6573083` · `lease_covers_claim` shared by core+replay · FIR claim-coverage-single-predicate |
| P1-004 | canonical lowercase hash validation | ✅ done | `6e9b39c` · `is_canonical_sha256_token` · FIR canonical-hash-lowercase |
| P1-005 | unify Python doctor vs Rust CLI doctor | ✅ done | Rust doctor reports pinned versions from `.devinfra/tool-versions.json` (match the Python doctor's `expected_version`); Python stays the version/SHA verification authority |
| P1-006 | attestation mode as a checkable gate | ✅ done | `--kernel-secret` on `replay verify` (attested) + `scenario emit-trace` (mints valid attestations); gate block proves a minted capability passes attested verify while missing/wrong attestations refute (`CapabilityMismatch`) |
| P1-007 | generated-artifacts policy unified | ✔ already-resolved | all 5 targets gitignored; lean4 generated not committed |

## P2 — cleanup / support quality
| Item | Title | Status | Commit / FIR |
|---|---|---|---|
| P2-001 | remove small dups / dead code | ✔ not-applicable | the cited dups are intentional: `mergeable()->false` is referenced by the Kani harness (`kani_target.rs`); `read_bundle` is per-subcommand by design; the other cited dups/legacy fields do not exist. clippy `-D warnings` green and gated |
| P2-002 | docs honest about status | ✔ already-resolved | coverage-matrix + READMEs carry honest rungs/markers |
| P2-003 | expand negative scenario corpus (15 desired) | ✅ done (replayable) | 11/15 pre-existed; added `barrier_legacy_witness_extra_invalid` (`b922f2f`), `unrelated_merge_protocol_does_not_allow_conflict_invalid` (`5d95eda`), `barrier_missing_lease_invalid` (this increment). The other 2 are **not replayable**: `constraint_update_rewrites_truth` (no constraint-update event kind in the trace schema; I-010 is P/Kani only) and `projection_anchor_wrong_event_kind` (already subsumed by `AnchorNotObservedTruth`) |
| P2-004 | multi-action reference scenario | ✅ done | a replay-valid two-action history on an `EvidenceMeta` (non-RuntimeExecution) bundle (`multi_action_reference.{registry,scenario}`); `validate_lifecycle` now reduces each action's own substream, and `causlane-replay` tests prove a broken secondary-action lifecycle is rejected. Wired into `formal-ready` + `formal-verify-all`. FIR `2026-06-21-multi-action-replay-reference` |

## Definition of Done §9 — status: closed
- [x] Cargo binary targets exist (P0-001, pre-existing)
- [x] `cargo check --workspace --all-targets --all-features` passes (gate)
- [x] Scenario schema matches Rust DTO and fixtures (P0-003)
- [x] Formal IR schema has no duplicate keys + validates generated IR (P0-004 + P0-008)
- [x] Replay does not use legacy witnesses as authoritative evidence (P0-006)
- [x] Typed barrier witnesses + leases mandatory for RuntimeExecution (pre-existing, re-verified)
- [x] Alloy conflict/mergeable assertion fixed (P0-005 part 2)
- [x] Negative control on unrelated merge protocol (P0-005 part 2)
- [x] Coverage matrix does not overclaim I-010 or others (P0-007, pre-existing)
- [x] Receipts separate manual_core and bundle_generated (P0-009, pre-existing)
- [x] Toolchain doctor/provisioning reproducible (P0-002, pre-existing)
- [x] `formal/smoke.sh` passes in clean checkout (gate)
- [x] `tools/formal-verify-all --profile base` passes (gate)
- [x] Docs reflect actual status (P2-002, pre-existing)

## Commits (this fix-stage)
| Commit | Items | FIR |
|---|---|---|
| `3a1233a` | P0-003, P0-004, P0-008 | 2026-06-19-schema-validation-gate |
| `b922f2f` | P0-006 | 2026-06-19-legacy-witness-reconciliation |
| `6e9b39c` | P1-004 | 2026-06-19-canonical-hash-lowercase |
| `6573083` | P1-003 | 2026-06-19-claim-coverage-single-predicate |
| `07caf36` | P0-005 (part 1) | 2026-06-20-mergeable-scope-resolver |
| `5d95eda` | P0-005 (part 2) | 2026-06-20-alloy-scope-keyed-mergeable |

## Out-of-scope follow-ups (genuinely deferred)
- The 2 non-replayable P2-003 negatives: `constraint_update_rewrites_truth` (no
  constraint-update event kind in the trace schema — I-010 stays P/Kani only) and
  `projection_anchor_wrong_event_kind` (subsumed by `AnchorNotObservedTruth`).

P1-001 (parts 1–3) and P2-004 are now closed — see the P1/P2 tables above and FIRs
`2026-06-21-p-action-sharded-machines` and `2026-06-21-multi-action-replay-reference`.
All other dispatcher-012 items and the Definition of Done §9 were already closed.
