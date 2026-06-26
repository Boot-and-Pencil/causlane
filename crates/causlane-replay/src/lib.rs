//! Replay verifier for causlane protocol traces.
//!
//! Two responsibilities:
//! 1. Load a JSON trace (the on-disk `*.trace.json` shape) into typed core
//!    [`AuditEvent`]s, failing closed on unknown event kinds or malformed plan
//!    hashes (invariant I-008).
//! 2. Verify the loaded events against the protocol invariants, scoped per
//!    `(action_id, plan_hash)` rather than globally:
//!    - I-001 no execution without a prior barrier,
//!    - I-002 no observed truth without a prior execution,
//!    - I-003 no projection without an anchor pointing at a prior committed
//!      observed truth (ADR-0010 — anchors, not witnesses),
//!    - I-006 no two active exclusive-write leases on the same scope (ADR-0012,
//!      merge protocol = none),
//!    - plan-hash consistency across an action's events.

#![forbid(unsafe_code)]
#![deny(warnings)]

use std::collections::{HashMap, HashSet};

use causlane_contracts::{
    BoundaryContracts, ClaimManifest, CompiledDispatchBundle, CompiledPredicate,
    ConsequenceProfileDto, TemplateBindings, TemplateResolver,
};
use causlane_core::{
    lease_covers_claim, ActionId, AuditEvent, AuditEventId, AuditEventKind, CapabilityIssuer,
    DrainSemantics, ExecutionCapability, ExecutionCapabilityError, ImpactSetHash, KernelContracts,
    LeaseTable, PlanHash, PredicateId, ResourceClaim, ResourceId, Scope, Timestamp,
    TruthAnchorResolver, WitnessAttestation, WitnessKind,
};
mod authz;
pub mod contract;
mod error;
mod explain;
mod lifecycle;
mod outcome;
mod trace;
mod trace_lowering;
pub(crate) use authz::validate_authz_refs;
pub use contract::{AuthzEvidenceVerdict, AuthzEvidenceVerifier, ReplayContracts, ReplayOracle};
pub use error::{ReplayError, ReplayErrorCode};
pub use explain::{CausalLocation, ReplayExplain};
pub(crate) use lifecycle::{resolve_action_profiles, validate_lifecycle};
pub use outcome::ReplayVerdict;
pub use trace::{
    ActionSpec, AuthzDecisionDto, EventKindDto, ExpectedReplayResult, ReplayAnchor,
    ReplayAuthzDecision, ReplayEvent, ReplayExecutionBarrier, ReplayExecutionCapability,
    ReplayLeaseRef, ReplayScenario, ReplayTrace, ReplayWitnessBinding, ReplayWitnessRef,
    ScenarioBinding, WitnessKindDto,
};

#[derive(Default)]
struct KeyState {
    barrier: bool,
    executed: bool,
}

/// What a committed `ObservedTruthCommitted` event recorded, used to ground
/// projection anchors against the producer event (I-003 / P0-004).
struct ObservedTruth {
    action_id: ActionId,
    plan_hash: Option<PlanHash>,
    attested: Option<WitnessAttestation>,
}

fn plan_token(plan: Option<&PlanHash>) -> String {
    plan.map(|hash| hash.as_str().to_owned())
        .unwrap_or_default()
}

impl ReplayTrace {
    /// Verify this trace against a compiled dispatch bundle.
    ///
    /// Structural replay still runs first; bundle mode then checks that the
    /// trace is bound to a predicate and that predicate-level obligations such
    /// as dispatch/barrier/witness requirements are present.
    ///
    /// # Errors
    /// Returns the first structural or bundle-bound replay error.
    #[must_use = "the verification result must be used"]
    pub fn verify_with_bundle(&self, bundle: &CompiledDispatchBundle) -> Result<(), ReplayError> {
        self.verify_with_bundle_options(bundle, false, None)
    }

    /// Verify against a compiled bundle in **strict** mode: in addition to every
    /// check [`Self::verify_with_bundle`] runs, the trace must itself declare the
    /// bundle hash it is bound to. An unbound trace (no `bundle_hash`) is rejected
    /// with [`ReplayError::MissingTraceBundleHash`] rather than silently accepted
    /// (P0-005: trace-to-bundle binding must be explicit in the authority chain).
    ///
    /// # Errors
    /// Returns [`ReplayError::MissingTraceBundleHash`] when the trace omits its
    /// bundle hash, plus any structural or bundle-bound replay error.
    #[must_use = "the verification result must be used"]
    pub fn verify_with_bundle_strict(
        &self,
        bundle: &CompiledDispatchBundle,
    ) -> Result<(), ReplayError> {
        self.verify_with_bundle_options(bundle, true, None)
    }

    /// Verify against a compiled bundle, additionally requiring every execution
    /// capability to carry a valid keyed attestation under `kernel_secret`
    /// (ADR-0013). With a secret configured, a capability whose attestation is
    /// missing or wrong is rejected — so an attacker who authored the trace but
    /// does not hold the secret cannot mint a spendable capability, closing the
    /// "structural id is forgeable" gap.
    ///
    /// # Errors
    /// Returns [`ReplayError::CapabilityMismatch`] for a missing/invalid
    /// attestation, plus any structural or bundle-bound replay error.
    #[must_use = "the verification result must be used"]
    pub fn verify_with_bundle_attested(
        &self,
        bundle: &CompiledDispatchBundle,
        kernel_secret: &[u8],
    ) -> Result<(), ReplayError> {
        self.verify_with_bundle_options(bundle, false, Some(kernel_secret))
    }

    fn verify_with_bundle_options(
        &self,
        bundle: &CompiledDispatchBundle,
        require_bundle_hash: bool,
        attestation_key: Option<&[u8]>,
    ) -> Result<(), ReplayError> {
        match &self.bundle_hash {
            Some(declared) => {
                if declared != &bundle.bundle_hash.0 {
                    return Err(ReplayError::BundleHashMismatch {
                        expected: bundle.bundle_hash.0.clone(),
                        actual: declared.clone(),
                    });
                }
            }
            None => {
                if require_bundle_hash {
                    return Err(ReplayError::MissingTraceBundleHash {
                        expected: bundle.bundle_hash.0.clone(),
                    });
                }
            }
        }
        let events = self.to_events()?;

        let predicate_id = self
            .predicate
            .as_ref()
            .ok_or(ReplayError::MissingTracePredicate)?;
        let predicate = bundle
            .predicate(&PredicateId(predicate_id.clone()))
            .ok_or_else(|| ReplayError::UnknownPredicate {
                predicate: predicate_id.clone(),
            })?;
        // Resolve the verified-merge conflict domains before the structural pass
        // so I-006 lease conflicts are relaxed exactly where a verified merge
        // protocol applies (per-protocol, not the global fail-closed default).
        let bindings = self.template_bindings();
        let mergeable_scopes = resolve_mergeable_scopes(bundle, predicate, &bindings)?;
        verify_events_with_mergeable(&events, &mergeable_scopes)?;

        let profile = predicate.consequence_profile.to_core();
        // A mixed-predicate trace (e.g. a RuntimeExecution producer plus a
        // ProjectionRead reader) declares each non-primary action's predicate in
        // `actions`; resolve those to profiles so the lifecycle grammar reduces
        // every action against its own profile rather than the primary's.
        let action_profiles = resolve_action_profiles(&self.actions, bundle)?;
        validate_lifecycle(&events, profile, &action_profiles)?;

        if predicate.consequence_profile == ConsequenceProfileDto::RuntimeExecution {
            let (barrier_index, barrier) = events
                .iter()
                .enumerate()
                .find(|(_index, event)| {
                    event.action_id.0 == self.action_id
                        && event.kind == AuditEventKind::ExecutionBarrierLogged
                })
                .ok_or_else(|| ReplayError::MissingRequiredBarrier {
                    action_id: self.action_id.clone(),
                })?;

            let prior_events = events.iter().take(barrier_index);
            let has_dispatch_before_barrier = prior_events.clone().any(|event| {
                event.action_id.0 == self.action_id && event.kind == AuditEventKind::DispatchLogged
            });
            if !has_dispatch_before_barrier {
                return Err(ReplayError::MissingDispatchBeforeBarrier {
                    action_id: self.action_id.clone(),
                });
            }

            let barrier_payload = barrier.execution_barrier.as_ref().ok_or_else(|| {
                ReplayError::MissingBarrierPayload {
                    event_id: barrier.event_id.0.clone(),
                }
            })?;
            if barrier_payload.impact_set_hash.0.is_empty() {
                return Err(ReplayError::MissingBarrierImpactSet {
                    event_id: barrier.event_id.0.clone(),
                });
            }
            validate_legacy_witness_consistency(barrier, barrier_payload)?;
            validate_typed_witnesses(
                prior_events.clone(),
                barrier_payload,
                predicate,
                &barrier_payload.impact_set_hash,
                &bindings,
            )?;
            validate_authz_refs(
                prior_events.clone(),
                barrier_payload,
                predicate,
                barrier.occurred_at,
                attestation_key,
            )?;
            let lease_table =
                lease_table_before(events.iter().take(barrier_index), &mergeable_scopes)?;
            // ADR-0013: leases are time-bounded authority — evaluate their
            // freshness against the barrier's own `occurred_at`, mirroring the
            // authz-ref check above. Passing `None` here previously made
            // `LeaseTableError::Expired` dead code on the replay path.
            lease_table.validate_barrier_leases(barrier_payload, barrier.occurred_at)?;
            validate_claim_manifest_coverage(barrier_payload, &predicate.claims, &bindings)?;
            validate_execution_capabilities(
                &events,
                barrier_index,
                barrier_payload,
                attestation_key,
            )?;
        }

        Ok(())
    }
}

fn validate_execution_capabilities(
    events: &[AuditEvent],
    barrier_index: usize,
    barrier: &causlane_core::ExecutionBarrier,
    attestation_key: Option<&[u8]>,
) -> Result<(), ReplayError> {
    for event in events.iter().skip(barrier_index.saturating_add(1)) {
        if event.action_id != barrier.action_id
            || event.plan_hash.as_ref() != Some(&barrier.plan_hash)
            || event.kind != AuditEventKind::ExecutionStarted
        {
            continue;
        }
        let Some(capability) = &event.execution_capability else {
            return Err(ReplayError::CapabilityMissing {
                event_id: event.event_id.0.clone(),
            });
        };
        KernelContracts
            .validate_capability(capability, barrier)
            .map_err(|err| ReplayError::CapabilityMismatch {
                event_id: event.event_id.0.clone(),
                error: format!("{err:?}"),
            })?;
        validate_expected_capability(capability, barrier, &event.event_id)?;
        if let Some(secret) = attestation_key {
            validate_capability_attestation(capability, secret, &event.event_id)?;
        }
    }
    Ok(())
}

/// ADR-0013: when a kernel secret is configured, a capability must carry a valid
/// keyed attestation over its canonical bytes. A missing or wrong tag is rejected
/// so an attacker who authored the trace but lacks the secret cannot mint a
/// spendable capability (the structural id alone is not enough).
fn validate_capability_attestation(
    capability: &ExecutionCapability,
    secret: &[u8],
    event_id: &AuditEventId,
) -> Result<(), ReplayError> {
    let Some(attestation) = &capability.attestation else {
        return Err(ReplayError::CapabilityMismatch {
            event_id: event_id.0.clone(),
            error: "missing capability attestation (kernel secret configured)".to_owned(),
        });
    };
    if !causlane_contracts::attestation::verify_attestation(
        secret,
        &capability.attestation_message(),
        &attestation.0,
    ) {
        return Err(ReplayError::CapabilityMismatch {
            event_id: event_id.0.clone(),
            error: "invalid capability attestation".to_owned(),
        });
    }
    Ok(())
}

fn validate_expected_capability(
    capability: &ExecutionCapability,
    barrier: &causlane_core::ExecutionBarrier,
    event_id: &AuditEventId,
) -> Result<(), ReplayError> {
    let expected = KernelContracts
        .derive_capability(barrier, capability.op_index)
        .map_err(|err| ReplayError::CapabilityMismatch {
            event_id: event_id.0.clone(),
            error: format!("{err:?}"),
        })?;
    if capability.lease_ids != expected.lease_ids || capability.expires_at != expected.expires_at {
        return Err(ReplayError::CapabilityMismatch {
            event_id: event_id.0.clone(),
            error: format!(
                "{:?}",
                ExecutionCapabilityError::BindingMismatch {
                    capability_id: capability.capability_id.clone()
                }
            ),
        });
    }
    Ok(())
}

fn lease_table_before<'a>(
    events: impl Iterator<Item = &'a AuditEvent>,
    mergeable_scopes: &HashSet<Scope>,
) -> Result<LeaseTable, ReplayError> {
    let mut table = LeaseTable::with_mergeable_scopes(mergeable_scopes.clone());
    for event in events {
        match event.kind {
            AuditEventKind::ConstraintLeaseGranted => {
                for lease in &event.leases {
                    table.grant(lease.clone(), &KernelContracts)?;
                }
            }
            AuditEventKind::ConstraintLeaseReleased => {
                for lease in &event.leases {
                    let _released = table.release(&lease.lease_id)?;
                }
            }
            AuditEventKind::ActionAdmitted
            | AuditEventKind::ActionPlanned
            | AuditEventKind::DispatchLogged
            | AuditEventKind::ExecutionBarrierLogged
            | AuditEventKind::ExecutionStarted
            | AuditEventKind::ExecutionCompleted
            | AuditEventKind::ObservedTruthCommitted
            | AuditEventKind::ProjectionEmitted
            | AuditEventKind::LifecycleClosed
            | AuditEventKind::GateApproved
            | AuditEventKind::GateDenied
            | AuditEventKind::AuthzDecisionRecorded
            | AuditEventKind::DrainFenceRequested
            | AuditEventKind::DrainFenceAcquired
            | AuditEventKind::ViolationDetected => {}
        }
    }
    Ok(table)
}

fn validate_typed_witnesses<'a>(
    prior_events: impl Iterator<Item = &'a AuditEvent>,
    barrier: &causlane_core::ExecutionBarrier,
    predicate: &causlane_contracts::CompiledPredicate,
    impact_set_hash: &ImpactSetHash,
    bindings: &TemplateBindings,
) -> Result<(), ReplayError> {
    let prior_by_id = prior_events
        .map(|event| (event.event_id.clone(), event))
        .collect::<HashMap<_, _>>();
    for requirement in &predicate.required_witnesses {
        let witness = barrier
            .witnesses
            .iter()
            .find(|candidate| candidate.requirement_id == requirement.id)
            .ok_or_else(|| ReplayError::RequiredWitnessMissing {
                requirement_id: requirement.id.clone(),
            })?;
        let producer =
            prior_by_id
                .get(&witness.event_id)
                .ok_or_else(|| ReplayError::WitnessNotPrior {
                    barrier_event_id: barrier.barrier_id.0.clone(),
                    witness_event_id: witness.event_id.0.clone(),
                })?;
        let expected_scope = resolve_scope(&requirement.selector.scope_expr, bindings)?;
        if witness.kind != witness_kind_for_event(producer.kind)
            || witness.fact_kind.as_ref().map(|fact| fact.0.as_str())
                != Some(requirement.selector.fact_kind.as_str())
            || witness.scope.as_ref() != Some(&expected_scope)
        {
            return Err(ReplayError::WitnessSelectorMismatch {
                requirement_id: requirement.id.clone(),
            });
        }
        // Ground the witness ref in the producer event's own attestation: the
        // barrier may not claim a fact_kind/scope the producer event did not
        // itself record (P0-004 — no self-attestation).
        let attested = producer.attested_fact.as_ref().ok_or_else(|| {
            ReplayError::WitnessAttestationMismatch {
                requirement_id: requirement.id.clone(),
            }
        })?;
        if witness.fact_kind.as_ref() != Some(&attested.fact_kind)
            || witness.scope.as_ref() != Some(&attested.scope)
        {
            return Err(ReplayError::WitnessAttestationMismatch {
                requirement_id: requirement.id.clone(),
            });
        }
        if let Some(binding) = &witness.binds_to {
            if binding.action_id != barrier.action_id
                || binding.plan_hash != barrier.plan_hash
                || binding.impact_set_hash.as_ref() != Some(impact_set_hash)
            {
                return Err(ReplayError::WitnessBindingMismatch {
                    requirement_id: requirement.id.clone(),
                });
            }
        } else if witness.kind == WitnessKind::GateApproval {
            return Err(ReplayError::WitnessBindingMismatch {
                requirement_id: requirement.id.clone(),
            });
        }
    }
    Ok(())
}

fn witness_kind_for_event(kind: AuditEventKind) -> WitnessKind {
    match kind {
        AuditEventKind::ObservedTruthCommitted => WitnessKind::ObservedFact,
        AuditEventKind::GateApproved => WitnessKind::GateApproval,
        AuditEventKind::ConstraintLeaseGranted | AuditEventKind::ConstraintLeaseReleased => {
            WitnessKind::ConstraintDecision
        }
        AuditEventKind::AuthzDecisionRecorded => WitnessKind::AuthzDecision,
        AuditEventKind::ActionAdmitted
        | AuditEventKind::ActionPlanned
        | AuditEventKind::DispatchLogged
        | AuditEventKind::ExecutionBarrierLogged
        | AuditEventKind::ExecutionStarted
        | AuditEventKind::ExecutionCompleted
        | AuditEventKind::ProjectionEmitted
        | AuditEventKind::LifecycleClosed
        | AuditEventKind::GateDenied
        | AuditEventKind::DrainFenceRequested
        | AuditEventKind::DrainFenceAcquired
        | AuditEventKind::ViolationDetected => WitnessKind::ExternalEvidence,
    }
}

fn validate_claim_manifest_coverage(
    barrier: &causlane_core::ExecutionBarrier,
    claims: &[ClaimManifest],
    bindings: &TemplateBindings,
) -> Result<(), ReplayError> {
    for claim in claims {
        let resolved_scope = resolve_scope(&claim.scope_expr, bindings)?;
        let claim_core = ResourceClaim {
            resource: ResourceId(claim.resource.clone()),
            scope: resolved_scope,
            mode: claim.mode.to_core(),
            amount: 1,
        };
        let covered = barrier
            .leases
            .iter()
            .any(|lease| lease_covers_claim(lease, &claim_core));
        if !covered {
            return Err(ReplayError::Lease(format!(
                "claim not covered: resource={} scope={}",
                claim.resource, claim.scope_expr
            )));
        }
    }
    Ok(())
}

fn resolve_scope(expression: &str, bindings: &TemplateBindings) -> Result<Scope, ReplayError> {
    // §7.3 TemplateResolver contract is the single resolution authority.
    BoundaryContracts
        .resolve_scope(expression, bindings)
        .map_err(|err| ReplayError::TemplateResolution {
            expression: expression.to_owned(),
            error: err.to_string(),
        })
}

/// Resolve the conflict-domain scopes a **verified** merge protocol permits
/// overlapping mutable writes on (I-006). For each of the predicate's effect
/// templates whose op kind resolves to a verified, applicable merge protocol
/// (via [`merge_decision`]), its conflict-domain expressions are resolved
/// against the bindings. Fail-closed: with no applicable verified protocol the
/// set is empty and every overlapping exclusive lease conflicts.
fn resolve_mergeable_scopes(
    bundle: &CompiledDispatchBundle,
    predicate: &CompiledPredicate,
    bindings: &TemplateBindings,
) -> Result<HashSet<Scope>, ReplayError> {
    // The single mergeable-scope resolver lives in `causlane-contracts` so the
    // replay oracle and the Alloy generator cannot diverge (P0-005).
    causlane_contracts::resolve_mergeable_scopes(bundle, predicate, bindings)
        .map(|scopes| scopes.into_iter().map(Scope).collect())
        .map_err(|err| ReplayError::TemplateResolution {
            expression: "merge conflict domain".to_owned(),
            error: err.to_string(),
        })
}

/// Verify a sequence of audit events against the protocol invariants.
///
/// # Errors
/// Returns the first [`ReplayError`] encountered (the trace is invalid).
#[must_use = "the verification result must be used"]
pub fn verify_events(events: &[AuditEvent]) -> Result<(), ReplayError> {
    verify_events_with_mergeable(events, &HashSet::new())
}

/// Verify protocol events, relaxing I-006 lease conflicts on `mergeable_scopes`
/// (conflict-domain scopes a verified merge protocol permits — resolved by the
/// bundle-aware caller). Fail-closed: an empty set forbids all overlaps.
#[allow(clippy::too_many_lines)]
pub(crate) fn verify_events_with_mergeable(
    events: &[AuditEvent],
    mergeable_scopes: &HashSet<Scope>,
) -> Result<(), ReplayError> {
    let mut state: HashMap<(ActionId, Option<PlanHash>), KeyState> = HashMap::new();
    let mut observed: HashMap<AuditEventId, ObservedTruth> = HashMap::new();
    let mut action_plan: HashMap<ActionId, PlanHash> = HashMap::new();
    let mut leases = LeaseTable::with_mergeable_scopes(mergeable_scopes.clone());
    let mut closed: HashSet<ActionId> = HashSet::new();

    for event in events {
        let action = event.action_id.clone();
        let plan = event.plan_hash.clone();

        // I-008: `lifecycle.closed` is terminal — no later event for the action.
        if closed.contains(&event.action_id) {
            return Err(ReplayError::EventAfterClosed {
                action_id: event.action_id.0.clone(),
                event_id: event.event_id.0.clone(),
            });
        }

        if let Some(hash) = &event.plan_hash {
            match action_plan.get(&action) {
                Some(existing) if existing != hash => {
                    return Err(ReplayError::PlanHashMismatch {
                        action_id: action.0.clone(),
                    });
                }
                Some(_existing) => {}
                None => {
                    let _previous = action_plan.insert(action.clone(), hash.clone());
                }
            }
        }

        match event.kind {
            AuditEventKind::ExecutionBarrierLogged => {
                state.entry((action, plan)).or_default().barrier = true;
            }
            AuditEventKind::ExecutionStarted => {
                let st = state.entry((action.clone(), plan.clone())).or_default();
                if !st.barrier {
                    return Err(ReplayError::ExecutionWithoutBarrier {
                        action_id: action.0,
                        plan_hash: plan_token(plan.as_ref()),
                    });
                }
                st.executed = true;
            }
            AuditEventKind::ObservedTruthCommitted => {
                let st = state.entry((action.clone(), plan.clone())).or_default();
                if !st.executed {
                    return Err(ReplayError::ObservedWithoutExecution {
                        action_id: action.0,
                        plan_hash: plan_token(plan.as_ref()),
                    });
                }
                // Only an event the kernel authority recognizes as a valid anchor
                // source (an observed-truth commit) may back a later projection
                // anchor (I-003). Route through TruthAnchorResolver so the contract
                // — not this match arm's structure alone — is the authority.
                if KernelContracts.anchor_source_is_valid(event.kind) {
                    let _previous = observed.insert(
                        event.event_id.clone(),
                        ObservedTruth {
                            action_id: action,
                            plan_hash: plan,
                            attested: event.attested_fact.clone(),
                        },
                    );
                }
            }
            AuditEventKind::ProjectionEmitted => {
                if event.anchors.is_empty() {
                    return Err(ReplayError::ProjectionWithoutAnchor {
                        event_id: event.event_id.0.clone(),
                    });
                }
                for anchor in &event.anchors {
                    let resolved = observed.get(&anchor.event_id);
                    // Route the action+plan binding through the single kernel
                    // authority (TruthAnchorResolver::anchor_matches) rather than an
                    // inline compare, so replay binds exactly what the formal lanes
                    // are generated from. A truth with no recorded plan never matches.
                    let matched = resolved.is_some_and(|truth| {
                        truth.plan_hash.as_ref().is_some_and(|truth_plan| {
                            KernelContracts.anchor_matches(anchor, &truth.action_id, truth_plan)
                        })
                    });
                    if !matched {
                        return Err(ReplayError::AnchorNotObservedTruth {
                            event_id: event.event_id.0.clone(),
                            anchor_event_id: anchor.event_id.0.clone(),
                        });
                    }
                    // Ground the anchor's claimed fact in the observed-truth
                    // event's own attestation: a projection may not self-assert
                    // a fact_kind/scope the producer event did not record (P0-004).
                    if anchor.fact_kind.is_some() || anchor.scope.is_some() {
                        let grounded = matches!(
                            resolved.and_then(|truth| truth.attested.as_ref()),
                            Some(attested)
                                if anchor.fact_kind.as_ref() == Some(&attested.fact_kind)
                                    && anchor.scope.as_ref() == Some(&attested.scope)
                        );
                        if !grounded {
                            return Err(ReplayError::AnchorAttestationMismatch {
                                event_id: event.event_id.0.clone(),
                                anchor_event_id: anchor.event_id.0.clone(),
                            });
                        }
                    }
                }
            }
            AuditEventKind::ConstraintLeaseGranted => {
                for lease in &event.leases {
                    leases.grant(lease.clone(), &KernelContracts)?;
                }
            }
            AuditEventKind::ConstraintLeaseReleased => {
                for lease in &event.leases {
                    let _released = leases.release(&lease.lease_id)?;
                }
            }
            AuditEventKind::DrainFenceAcquired => {
                // I-007 (expiry-aware): a drain fence may be acquired only when no
                // lease is still actively overlapping the fence scope AND not yet
                // expired at the fence's acquisition time. Routed through the single
                // kernel authority (`DrainSemantics::can_acquire_fence`). When the
                // event carries no timestamp we fall back to the earliest instant,
                // so no lease is treated as expired (fail-closed, existence-based).
                if let Some(fence_scope) = &event.drain_fence_scope {
                    let now = event.occurred_at.unwrap_or(Timestamp(0));
                    if !KernelContracts.can_acquire_fence(fence_scope, leases.active_leases(), now)
                    {
                        return Err(ReplayError::DrainFenceWithActiveOverlap {
                            event_id: event.event_id.0.clone(),
                            scope: fence_scope.0.clone(),
                        });
                    }
                }
            }
            AuditEventKind::ActionAdmitted
            | AuditEventKind::ActionPlanned
            | AuditEventKind::DispatchLogged
            | AuditEventKind::ExecutionCompleted
            | AuditEventKind::LifecycleClosed
            | AuditEventKind::GateApproved
            | AuditEventKind::GateDenied
            | AuditEventKind::AuthzDecisionRecorded
            | AuditEventKind::DrainFenceRequested
            | AuditEventKind::ViolationDetected => {}
        }

        if event.kind == AuditEventKind::LifecycleClosed {
            let _existing = closed.insert(event.action_id.clone());
        }
    }

    Ok(())
}

/// Reconcile the deprecated legacy `AuditEvent.witnesses` list against the typed
/// barrier payload (P0-006). The typed `ExecutionBarrier.witnesses` is the
/// authoritative evidence (validated by [`validate_typed_witnesses`], which checks
/// prior-ness, binding, scope and producer attestation). The legacy list is a
/// compatibility mirror: when present it must name exactly the producer events of
/// the typed witnesses, so a trace cannot let the two disagree, and legacy
/// witnesses are never sufficient on their own. An empty legacy list is the
/// canonical typed-only path.
fn validate_legacy_witness_consistency(
    barrier: &AuditEvent,
    barrier_payload: &causlane_core::ExecutionBarrier,
) -> Result<(), ReplayError> {
    if barrier.witnesses.is_empty() {
        return Ok(());
    }
    let typed = barrier_payload
        .witnesses
        .iter()
        .map(|witness| &witness.event_id)
        .collect::<HashSet<_>>();
    let legacy = barrier.witnesses.iter().collect::<HashSet<_>>();
    if typed == legacy {
        Ok(())
    } else {
        Err(ReplayError::LegacyWitnessMismatch {
            barrier_event_id: barrier.event_id.0.clone(),
        })
    }
}

#[cfg(test)]
mod tests;
