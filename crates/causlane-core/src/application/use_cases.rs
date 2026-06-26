//! Application use-case skeletons.

use crate::domain::{ActionCall, ActionId, ConsequenceProfile};

/// The outcome of admitting an action call into dispatch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchAdmission {
    /// The call was accepted.
    Accepted {
        /// The accepted action.
        action_id: ActionId,
    },
    /// The call must wait before proceeding.
    Waiting {
        /// The waiting action.
        action_id: ActionId,
        /// Why the call is waiting.
        reason: String,
    },
    /// The call was rejected.
    Rejected {
        /// The rejected action.
        action_id: ActionId,
        /// Why the call was rejected.
        reason: String,
    },
}

/// Admit an action call, returning the admission outcome.
#[must_use]
pub fn admit_call(call: &ActionCall) -> DispatchAdmission {
    DispatchAdmission::Accepted {
        action_id: call.action_id.clone(),
    }
}

/// Whether a plan with the given profile requires an execution barrier.
#[must_use]
pub fn requires_execution_barrier(profile: ConsequenceProfile) -> bool {
    matches!(profile, ConsequenceProfile::RuntimeExecution)
}

/// Whether a plan with the given profile may commit observed truth.
#[must_use]
pub fn can_commit_observed_truth(profile: ConsequenceProfile) -> bool {
    matches!(profile, ConsequenceProfile::RuntimeExecution)
}
