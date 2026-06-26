//! Per-action lifecycle reduction for replay traces.
//!
//! A trace may carry actions of different predicates — a *mixed-predicate* trace,
//! e.g. a `RuntimeExecution` producer plus a `ProjectionRead` reader that anchors
//! its projection on the producer's observed truth. Each action reduces against
//! its own consequence profile; the per-action roster lives on the trace's
//! `actions` field ([`crate::ActionSpec`]) and is resolved against the bundle
//! here. Actions without a roster entry fall back to the trace-level (primary)
//! profile, so single-predicate traces behave exactly as before.

use std::collections::{HashMap, HashSet};

use causlane_contracts::CompiledDispatchBundle;
use causlane_core::{
    ActionId, AuditEvent, ConsequenceProfile, KernelContracts, LifecycleGrammar, PredicateId,
};

use crate::{ActionSpec, ReplayError};

/// Resolve the consequence profile for each action declared in a mixed-predicate
/// trace's `actions` roster. A declared predicate the bundle does not define
/// fails closed with `UnknownPredicate`.
pub(crate) fn resolve_action_profiles(
    actions: &[ActionSpec],
    bundle: &CompiledDispatchBundle,
) -> Result<HashMap<ActionId, ConsequenceProfile>, ReplayError> {
    let mut profiles = HashMap::with_capacity(actions.len());
    for spec in actions {
        let predicate = bundle
            .predicate(&PredicateId(spec.predicate.clone()))
            .ok_or_else(|| ReplayError::UnknownPredicate {
                predicate: spec.predicate.clone(),
            })?;
        let _previous = profiles.insert(
            ActionId(spec.action_id.clone()),
            predicate.consequence_profile.to_core(),
        );
    }
    Ok(profiles)
}

/// Reduce each action's event substream against its own consequence profile.
///
/// Route through the shared `LifecycleGrammar` contract (§7.5) so replay reduces
/// against the same grammar the formal lanes are generated from. Each distinct
/// action owns an independent lifecycle, reduced in first-seen order for
/// deterministic error reporting. An action listed in `action_profiles` uses
/// that profile (the mixed-predicate case); otherwise `default_profile` (the
/// trace-level predicate).
pub(crate) fn validate_lifecycle(
    events: &[AuditEvent],
    default_profile: ConsequenceProfile,
    action_profiles: &HashMap<ActionId, ConsequenceProfile>,
) -> Result<(), ReplayError> {
    let grammar = KernelContracts;
    let mut seen: HashSet<&ActionId> = HashSet::new();
    let mut actions: Vec<&ActionId> = Vec::new();
    for event in events {
        if seen.insert(&event.action_id) {
            actions.push(&event.action_id);
        }
    }
    for action in actions {
        let profile = action_profiles
            .get(action)
            .copied()
            .unwrap_or(default_profile);
        let mut stage = grammar.initial_stage(profile);
        for event in events.iter().filter(|event| &event.action_id == action) {
            stage = grammar
                .reduce(stage, event.kind, profile)
                .map_err(|err| ReplayError::Lifecycle(format!("{err:?}")))?;
        }
    }
    Ok(())
}
