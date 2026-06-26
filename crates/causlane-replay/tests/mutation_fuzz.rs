//! M04.6: mutation + fuzz harness — the replay oracle is fail-closed.
//!
//! Two properties, through the public API only, with NO new dependency and NO
//! new fixture:
//!   (A) TARGETED mutations of a known-valid trace each cause `verify_with_bundle`
//!       to reject with one exact `ReplayErrorCode` — programmatic negative
//!       controls for I-001 (barrier/capability), I-003 (anchor), I-006 (lease),
//!       I-007 (drain), I-008 (closed) and I-009 (witness binding), plus a
//!       structural code. (The authz I-009 codes and `ObservedWithoutExecution`
//!       are covered by the `*_invalid` scenario suite, the `src/tests.rs`
//!       `verify_events` slice tests and the M04.5 contract harness — not
//!       re-derived here, since this bundle runs authz disabled.)
//!   (B) TOTALITY: over a broad, deterministic corpus of malformed traces (the
//!       valid trace mutated every way, plus truncated "small worlds"),
//!       `verify_with_bundle` always RETURNS `Ok|Err` — it never panics, hangs or
//!       aborts. The oracle is a total function over arbitrary `ReplayTrace`
//!       input (the source is panic-free by construction: `#![forbid(unsafe_code)]`,
//!       no unwrap/expect/indexing on the verify path).
//!
//! Integration test (outside `src/tests.rs`, which is at the file-length limit).

use causlane_contracts::{CompiledDispatchBundle, RegistryManifest};
use causlane_replay::{EventKindDto, ReplayErrorCode, ReplayEvent, ReplayScenario, ReplayTrace};
use EventKindDto as K;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const REGISTRY: &str = include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");
const SCENARIO: &str =
    include_str!("../fixtures/contracts/scenarios/release_promote_success.scenario.yaml");

/// All 17 `EventKindDto` variants, for retyping fuzz mutations. Kept honest by
/// `event_kind_table_is_complete` below — a new kernel event kind breaks that
/// test's exhaustive match until this table is updated.
const KINDS: [EventKindDto; 17] = [
    K::ActionAdmitted,
    K::ActionPlanned,
    K::DispatchLogged,
    K::ExecutionBarrierLogged,
    K::ExecutionStarted,
    K::ExecutionCompleted,
    K::ObservedTruthCommitted,
    K::ProjectionEmitted,
    K::LifecycleClosed,
    K::GateApproved,
    K::GateDenied,
    K::ConstraintLeaseGranted,
    K::ConstraintLeaseReleased,
    K::ViolationDetected,
    K::AuthzDecisionRecorded,
    K::DrainFenceRequested,
    K::DrainFenceAcquired,
];

fn demo_bundle() -> Result<CompiledDispatchBundle, Box<dyn std::error::Error>> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
    Ok(CompiledDispatchBundle::compile(&manifest)?)
}

fn base_trace() -> Result<ReplayTrace, Box<dyn std::error::Error>> {
    Ok(ReplayScenario::from_yaml_str(SCENARIO)?.to_trace())
}

fn find_mut(trace: &mut ReplayTrace, kind: EventKindDto) -> Option<&mut ReplayEvent> {
    trace.events.iter_mut().find(|event| event.kind == kind)
}

/// Clone the valid base and apply one mutation.
fn mutated(base: &ReplayTrace, mutate: impl FnOnce(&mut ReplayTrace)) -> ReplayTrace {
    let mut trace = base.clone();
    mutate(&mut trace);
    trace
}

/// Assert a mutated trace is rejected with exactly `expected`.
fn assert_rejects(
    trace: &ReplayTrace,
    bundle: &CompiledDispatchBundle,
    expected: ReplayErrorCode,
    label: &str,
) -> TestResult {
    match trace.verify_with_bundle(bundle) {
        Err(err) if err.code() == expected => Ok(()),
        other => Err(format!("{label}: expected {expected:?}, got {other:?}").into()),
    }
}

/// Sanity / non-vacuity: the unmodified fixture verifies (so every rejection
/// below is caused by the mutation, not a broken base).
#[test]
fn base_fixture_verifies() -> TestResult {
    base_trace()?.verify_with_bundle(&demo_bundle()?)?;
    Ok(())
}

/// The fuzz retype table covers every event kind (exhaustive match — a new
/// kernel kind fails compilation here until `KINDS` is updated).
#[test]
fn event_kind_table_is_complete() {
    fn covered(kind: EventKindDto) -> bool {
        match kind {
            K::ActionAdmitted
            | K::ActionPlanned
            | K::DispatchLogged
            | K::ExecutionBarrierLogged
            | K::ExecutionStarted
            | K::ExecutionCompleted
            | K::ObservedTruthCommitted
            | K::ProjectionEmitted
            | K::LifecycleClosed
            | K::GateApproved
            | K::GateDenied
            | K::ConstraintLeaseGranted
            | K::ConstraintLeaseReleased
            | K::ViolationDetected
            | K::AuthzDecisionRecorded
            | K::DrainFenceRequested
            | K::DrainFenceAcquired => true,
        }
    }
    assert_eq!(KINDS.len(), 17);
    assert!(KINDS.iter().copied().all(covered));
}

/// I-001 — execution requires a write-ahead barrier, its payload, and a
/// barrier-derived capability.
#[test]
fn targeted_barrier_and_capability_mutations() -> TestResult {
    let bundle = demo_bundle()?;
    let base = base_trace()?;

    // Drop the barrier but keep execution.started: the pass-1 invariant pass
    // (I-001) trips before the pass-3 MissingRequiredBarrier check.
    assert_rejects(
        &mutated(&base, |t| {
            t.events
                .retain(|event| event.kind != K::ExecutionBarrierLogged);
        }),
        &bundle,
        ReplayErrorCode::ExecutionWithoutBarrier,
        "drop barrier",
    )?;
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::ExecutionBarrierLogged) {
                event.execution_barrier = None;
            }
        }),
        &bundle,
        ReplayErrorCode::MissingBarrierPayload,
        "null barrier payload",
    )?;
    // A blank hash fails the format check during lowering, before the pass-3
    // MissingBarrierImpactSet guard (defense in depth).
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::ExecutionBarrierLogged) {
                if let Some(barrier) = &mut event.execution_barrier {
                    barrier.impact_set_hash = String::new();
                }
            }
        }),
        &bundle,
        ReplayErrorCode::BadImpactSetHash,
        "blank barrier impact set",
    )?;
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::ExecutionStarted) {
                event.execution_capability = None;
            }
        }),
        &bundle,
        ReplayErrorCode::CapabilityMissing,
        "strip capability",
    )?;
    // Tamper the capability's lease set so it no longer matches the barrier.
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::ExecutionStarted) {
                if let Some(capability) = &mut event.execution_capability {
                    capability.lease_ids.pop();
                }
            }
        }),
        &bundle,
        ReplayErrorCode::CapabilityMismatch,
        "tamper capability lease_ids",
    )?;
    Ok(())
}

/// I-003 (anchor), I-006 (lease), I-007 (drain), I-008 (closed) + a structural
/// plan-hash control.
#[test]
fn targeted_truth_lease_and_lifecycle_mutations() -> TestResult {
    let bundle = demo_bundle()?;
    let base = base_trace()?;

    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::ProjectionEmitted) {
                event.anchors.clear();
            }
        }),
        &bundle,
        ReplayErrorCode::ProjectionWithoutAnchor,
        "clear anchors",
    )?;
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::ProjectionEmitted) {
                if let Some(anchor) = event.anchors.first_mut() {
                    anchor.event_id = "evt_dispatch".to_owned();
                }
            }
        }),
        &bundle,
        ReplayErrorCode::AnchorNotObservedTruth,
        "repoint anchor at non-truth event",
    )?;
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::ProjectionEmitted) {
                if let Some(anchor) = event.anchors.first_mut() {
                    anchor.fact_kind = Some("fabricated_fact".to_owned());
                }
            }
        }),
        &bundle,
        ReplayErrorCode::AnchorAttestationMismatch,
        "anchor claims unattested fact",
    )?;
    // A second exclusive lease on the same scope conflicts (I-006).
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::ConstraintLeaseGranted) {
                if let Some(mut dup) = event.leases.first().cloned() {
                    dup.lease_id = format!("{}_dup", dup.lease_id);
                    event.leases.push(dup);
                }
            }
        }),
        &bundle,
        ReplayErrorCode::ConflictingLeases,
        "duplicate exclusive lease on same scope",
    )?;
    // A drain fence over a still-active lease scope is refused (I-007).
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(mut drain) = t
                .events
                .iter()
                .find(|event| event.kind == K::ExecutionStarted)
                .cloned()
            {
                drain.kind = K::DrainFenceAcquired;
                drain.event_id = Some("evt_drain".to_owned());
                drain.scope = Some("environment:staging".to_owned());
                drain.execution_capability = None;
                let pos = t
                    .events
                    .iter()
                    .position(|event| event.kind == K::ConstraintLeaseReleased)
                    .unwrap_or(t.events.len());
                t.events.insert(pos, drain);
            }
        }),
        &bundle,
        ReplayErrorCode::DrainFenceWithActiveOverlap,
        "drain fence over active lease",
    )?;
    // No event after lifecycle.closed (I-008).
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(closed) = t
                .events
                .iter()
                .find(|event| event.kind == K::LifecycleClosed)
                .cloned()
            {
                t.events.push(closed);
            }
        }),
        &bundle,
        ReplayErrorCode::EventAfterClosed,
        "event after closed",
    )?;
    // Inconsistent plan hash within an action (structural).
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::DispatchLogged) {
                event.plan_hash = Some(format!("sha256:{}", "2".repeat(64)));
            }
        }),
        &bundle,
        ReplayErrorCode::PlanHashMismatch,
        "plan hash mismatch",
    )?;
    Ok(())
}

/// I-009 — witness evidence must be present and bound to the right scope/action.
#[test]
fn targeted_witness_mutations() -> TestResult {
    let bundle = demo_bundle()?;
    let base = base_trace()?;

    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::ExecutionBarrierLogged) {
                event.witnesses.clear();
                if let Some(barrier) = &mut event.execution_barrier {
                    barrier.witnesses.clear();
                }
            }
        }),
        &bundle,
        ReplayErrorCode::RequiredWitnessMissing,
        "clear required witnesses",
    )?;
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::ExecutionBarrierLogged) {
                if let Some(barrier) = &mut event.execution_barrier {
                    if let Some(witness) = barrier.witnesses.first_mut() {
                        witness.scope = Some("release_candidate:other".to_owned());
                    }
                }
            }
        }),
        &bundle,
        ReplayErrorCode::WitnessSelectorMismatch,
        "witness wrong scope",
    )?;
    assert_rejects(
        &mutated(&base, |t| {
            if let Some(event) = find_mut(t, K::ExecutionBarrierLogged) {
                if let Some(barrier) = &mut event.execution_barrier {
                    if let Some(witness) = barrier.witnesses.first_mut() {
                        if let Some(binding) = &mut witness.binds_to {
                            binding.action_id = "other_action".to_owned();
                        }
                    }
                }
            }
        }),
        &bundle,
        ReplayErrorCode::WitnessBindingMismatch,
        "witness wrong binding",
    )?;
    Ok(())
}

/// Build a broad, deterministic corpus of malformed traces from the valid base:
/// truncations ("small worlds"), and per-event drop / duplicate / retype-to-each
/// kind / null-payloads / corrupt-hash, plus predicate/binding mutations.
fn malformed_corpus(base: &ReplayTrace) -> Vec<ReplayTrace> {
    let mut corpus = Vec::new();
    let len = base.events.len();

    // Truncated prefixes — short "small worlds".
    for keep in 0..=len {
        let mut trace = base.clone();
        trace.events.truncate(keep);
        corpus.push(trace);
    }

    for index in 0..len {
        // Drop event i.
        let mut dropped = base.clone();
        if index < dropped.events.len() {
            dropped.events.remove(index);
        }
        corpus.push(dropped);

        // Duplicate event i.
        if let Some(event) = base.events.get(index).cloned() {
            let mut dup = base.clone();
            dup.events.insert(index, event);
            corpus.push(dup);
        }

        // Retype event i to every kind.
        for &kind in &KINDS {
            let mut retyped = base.clone();
            if let Some(event) = retyped.events.get_mut(index) {
                event.kind = kind;
            }
            corpus.push(retyped);
        }

        // Null every payload + give a foreign action id.
        let mut nulled = base.clone();
        if let Some(event) = nulled.events.get_mut(index) {
            event.execution_barrier = None;
            event.execution_capability = None;
            event.authz_decision = None;
            event.anchors.clear();
            event.leases.clear();
            event.witnesses.clear();
            event.witness_refs.clear();
            "zzz".clone_into(&mut event.action_id);
        }
        corpus.push(nulled);

        // Corrupt the plan hash (malformed digest).
        let mut corrupt = base.clone();
        if let Some(event) = corrupt.events.get_mut(index) {
            event.plan_hash = Some("sha256:not-a-valid-digest".to_owned());
        }
        corpus.push(corrupt);
    }

    // Trace-level binding mutations.
    let mut no_predicate = base.clone();
    no_predicate.predicate = None;
    corpus.push(no_predicate);

    let mut bogus_predicate = base.clone();
    bogus_predicate.predicate = Some("bogus.predicate".to_owned());
    corpus.push(bogus_predicate);

    let mut no_bundle_hash = base.clone();
    no_bundle_hash.bundle_hash = None;
    corpus.push(no_bundle_hash);

    corpus
}

/// TOTALITY: over the whole malformed corpus, `verify_with_bundle` returns
/// `Ok|Err` and never panics/hangs. Reaching the assertions proves the oracle is
/// a total function over arbitrary input; the rejection count proves the corpus
/// is non-vacuous (the fuzzer drives the oracle to reject in most cases, while a
/// minority of benign mutations still verify — so the oracle is discriminating,
/// not trivially rejecting everything).
#[test]
fn totality_no_panic_over_malformed_traces() -> TestResult {
    let bundle = demo_bundle()?;
    let base = base_trace()?;
    let corpus = malformed_corpus(&base);
    let total = corpus.len();

    let rejects = corpus
        .iter()
        .filter(|trace| trace.verify_with_bundle(&bundle).is_err())
        .count();

    assert!(total > 100, "corpus too small to be meaningful: {total}");
    // Most malformed mutations of a tightly-constrained valid trace must reject;
    // a weak `rejects > 0` would let the totality pass near-vacuously.
    assert!(
        rejects > total / 2,
        "only {rejects}/{total} malformed cases rejected — fuzz too weak or oracle too lax"
    );
    // ...but not ALL: some benign mutations (e.g. clearing an already-empty field)
    // legitimately still verify, proving the oracle discriminates.
    assert!(
        rejects < total,
        "every case rejected — the corpus may be trivially malformed"
    );
    Ok(())
}
