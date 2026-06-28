#![forbid(unsafe_code)]
#![deny(warnings)]

use std::fmt;

use causlane::core::kernel::approval_gate_stepup;
use causlane::core::protocol::{
    ActionId, ApprovalDenyReason, ApprovalOutcome, ApprovalRef, ApprovalRequirement, ApprovalVerb,
    AssuranceLevel, AuditEventId, ImpactSetHash, PlanHash, PlanHashError, Timestamp,
};
use causlane_contracts::{CompiledDispatchBundle, ContractError, RegistryManifest};
use causlane_replay::{ReplayError, ReplayScenario};

const PLAN_HASH: &str = "sha256:a241276dc389e9197710cd415072e850036429799d636873f47ae8a1bc44d47b";
const OTHER_PLAN_HASH: &str =
    "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const IMPACT_SET_HASH: &str =
    "sha256:3148f111980508df7fb1acdeab78576ae1c2852f7ec9ac91f629f3f8701faf7f";
const ACTION_ID: &str = "act_promote_123";
const INITIATOR: &str = "alice";
const REGISTRY_YAML: &str =
    include_str!("../../../contracts/examples/release_promote.registry.yaml");
const SUCCESS_SCENARIO_YAML: &str =
    include_str!("../../../contracts/scenarios/release_promote_success.scenario.yaml");
const WRONG_PLAN_SCENARIO_YAML: &str =
    include_str!("../../../contracts/scenarios/approval_wrong_plan_invalid.scenario.yaml");

/// Summary returned by the runnable example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovalGateSummary {
    /// Local gate cases checked before replay.
    pub gate_cases: usize,
    /// Bundle-bound positive replay scenarios verified.
    pub verified_scenarios: usize,
    /// Bundle-bound negative replay scenarios refuted.
    pub refuted_scenarios: usize,
}

/// Error type for the approval-gate example composition.
#[derive(Debug)]
pub enum ApprovalGateError {
    /// A static plan hash embedded in the example was malformed.
    PlanHash(PlanHashError),
    /// Registry or bundle compilation failed.
    Contract(ContractError),
    /// Bundle-bound replay rejected a positive scenario.
    Replay(ReplayError),
    /// The local approval gate produced an unexpected outcome.
    GateOutcome(&'static str),
    /// A negative scenario unexpectedly passed replay.
    NegativeReplayPassed(&'static str),
    /// A negative scenario failed with the wrong replay error.
    NegativeReplayWrongError {
        /// Scenario id used in the example.
        scenario: &'static str,
        /// Replay error returned by the oracle.
        error: ReplayError,
    },
}

impl fmt::Display for ApprovalGateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlanHash(error) => write!(f, "invalid static plan hash: {error:?}"),
            Self::Contract(error) => write!(f, "contract fixture failed: {error}"),
            Self::Replay(error) => write!(f, "bundle replay failed: {error}"),
            Self::GateOutcome(label) => write!(f, "unexpected approval gate outcome: {label}"),
            Self::NegativeReplayPassed(scenario) => {
                write!(
                    f,
                    "negative replay scenario unexpectedly passed: {scenario}"
                )
            }
            Self::NegativeReplayWrongError { scenario, error } => {
                write!(f, "negative replay scenario {scenario} failed with {error}")
            }
        }
    }
}

impl std::error::Error for ApprovalGateError {}

impl From<PlanHashError> for ApprovalGateError {
    fn from(error: PlanHashError) -> Self {
        Self::PlanHash(error)
    }
}

impl From<ContractError> for ApprovalGateError {
    fn from(error: ContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<ReplayError> for ApprovalGateError {
    fn from(error: ReplayError) -> Self {
        Self::Replay(error)
    }
}

/// Run the approval-gate example.
///
/// # Errors
/// Returns an error if local gate checks or bundle-bound replay fail.
#[must_use = "the runnable example result must be checked"]
pub fn run_approval_gate() -> Result<ApprovalGateSummary, ApprovalGateError> {
    let gate_cases = verify_local_approval_gate_cases()?;
    verify_release_promote_success()?;
    refute_wrong_plan_approval()?;
    Ok(ApprovalGateSummary {
        gate_cases,
        verified_scenarios: 1,
        refuted_scenarios: 1,
    })
}

/// Verify local approval-gate outcomes without replay fixtures.
///
/// # Errors
/// Returns an error if a local gate case diverges from the documented outcome.
#[must_use = "approval-gate checks can fail and must be inspected"]
pub fn verify_local_approval_gate_cases() -> Result<usize, ApprovalGateError> {
    let action = ActionId(ACTION_ID.to_owned());
    let plan = plan_hash()?;
    let other_plan = PlanHash::new(OTHER_PLAN_HASH.to_owned())?;
    let impact = impact_set_hash();
    let requirement = ApprovalRequirement {
        action: &action,
        plan: &plan,
        impact_set: &impact,
        initiator: INITIATOR,
        required_assurance: AssuranceLevel(80),
        max_age: 20,
        now: Timestamp(30),
    };

    require_denied(
        approval_gate_stepup(&[], &requirement),
        ApprovalDenyReason::Missing,
        "missing approval denies",
    )?;
    require_denied(
        approval_gate_stepup(
            &[approve(
                "evt_self_approval",
                &action,
                &plan,
                &impact,
                INITIATOR,
                95,
                10,
            )],
            &requirement,
        ),
        ApprovalDenyReason::SelfApproval,
        "self approval denies",
    )?;
    require_denied(
        approval_gate_stepup(
            &[approve(
                "evt_weak_approval",
                &action,
                &plan,
                &impact,
                "bob",
                20,
                10,
            )],
            &requirement,
        ),
        ApprovalDenyReason::InsufficientAssurance,
        "under-assured approval denies",
    )?;
    require_denied(
        approval_gate_stepup(
            &[approve(
                "evt_stale_approval",
                &action,
                &plan,
                &impact,
                "bob",
                95,
                0,
            )],
            &requirement,
        ),
        ApprovalDenyReason::Stale,
        "stale approval denies",
    )?;
    require_denied(
        approval_gate_stepup(
            &[approve(
                "evt_wrong_plan_approval",
                &action,
                &other_plan,
                &impact,
                "bob",
                95,
                10,
            )],
            &requirement,
        ),
        ApprovalDenyReason::WrongPlan,
        "wrong-plan approval denies",
    )?;

    let qualified = approve(
        "evt_qualified_approval",
        &action,
        &plan,
        &impact,
        "bob",
        95,
        10,
    );
    require_approved(
        approval_gate_stepup(std::slice::from_ref(&qualified), &requirement),
        "fresh distinct actor approval authorizes",
    )?;
    require_denied(
        approval_gate_stepup(
            &[
                qualified,
                deny("evt_exact_deny", &action, &plan, &impact, "carol"),
            ],
            &requirement,
        ),
        ApprovalDenyReason::ExplicitDeny,
        "exact deny wins",
    )?;

    Ok(7)
}

/// Verify the bundled release-promote success scenario.
///
/// # Errors
/// Returns an error if fixture parsing, bundle compilation or replay fails.
#[must_use = "bundle replay can fail and must be inspected"]
pub fn verify_release_promote_success() -> Result<(), ApprovalGateError> {
    let bundle = release_promote_bundle()?;
    let scenario = ReplayScenario::from_yaml_str(SUCCESS_SCENARIO_YAML)?;
    scenario.to_trace().verify_with_bundle(&bundle)?;
    Ok(())
}

/// Verify that a misbound approval witness is rejected by bundle replay.
///
/// # Errors
/// Returns an error if fixture parsing fails or replay does not reject with
/// `WitnessBindingMismatch`.
#[must_use = "negative replay can fail and must be inspected"]
pub fn refute_wrong_plan_approval() -> Result<(), ApprovalGateError> {
    let bundle = release_promote_bundle()?;
    let scenario = ReplayScenario::from_yaml_str(WRONG_PLAN_SCENARIO_YAML)?;
    let result = scenario.to_trace().verify_with_bundle(&bundle);
    match result {
        Err(ReplayError::WitnessBindingMismatch { .. }) => Ok(()),
        Err(error) => Err(ApprovalGateError::NegativeReplayWrongError {
            scenario: "approval_wrong_plan_invalid",
            error,
        }),
        Ok(()) => Err(ApprovalGateError::NegativeReplayPassed(
            "approval_wrong_plan_invalid",
        )),
    }
}

fn require_approved(
    outcome: ApprovalOutcome,
    label: &'static str,
) -> Result<(), ApprovalGateError> {
    if outcome == ApprovalOutcome::Approved {
        Ok(())
    } else {
        Err(ApprovalGateError::GateOutcome(label))
    }
}

fn require_denied(
    outcome: ApprovalOutcome,
    reason: ApprovalDenyReason,
    label: &'static str,
) -> Result<(), ApprovalGateError> {
    if outcome == ApprovalOutcome::Denied(reason) {
        Ok(())
    } else {
        Err(ApprovalGateError::GateOutcome(label))
    }
}

fn release_promote_bundle() -> Result<CompiledDispatchBundle, ContractError> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY_YAML)?;
    CompiledDispatchBundle::compile(&manifest)
}

struct ApprovalParts<'a> {
    event_id: &'a str,
    verdict: ApprovalVerb,
    action: &'a ActionId,
    plan: &'a PlanHash,
    impact: &'a ImpactSetHash,
    actor: &'a str,
    assurance: u8,
    issued_at: u64,
    expires_at: Option<Timestamp>,
}

fn approve(
    event_id: &str,
    action: &ActionId,
    plan: &PlanHash,
    impact: &ImpactSetHash,
    actor: &str,
    assurance: u8,
    issued_at: u64,
) -> ApprovalRef {
    approval(ApprovalParts {
        event_id,
        verdict: ApprovalVerb::Approve,
        action,
        plan,
        impact,
        actor,
        assurance,
        issued_at,
        expires_at: None,
    })
}

fn deny(
    event_id: &str,
    action: &ActionId,
    plan: &PlanHash,
    impact: &ImpactSetHash,
    actor: &str,
) -> ApprovalRef {
    approval(ApprovalParts {
        event_id,
        verdict: ApprovalVerb::Deny,
        action,
        plan,
        impact,
        actor,
        assurance: 95,
        issued_at: 10,
        expires_at: None,
    })
}

fn approval(parts: ApprovalParts<'_>) -> ApprovalRef {
    ApprovalRef {
        approval_event_id: AuditEventId(parts.event_id.to_owned()),
        verdict: parts.verdict,
        action_id: parts.action.clone(),
        plan_hash: parts.plan.clone(),
        impact_set_hash: parts.impact.clone(),
        actor: parts.actor.to_owned(),
        issued_at: Timestamp(parts.issued_at),
        expires_at: parts.expires_at,
        assurance: AssuranceLevel(parts.assurance),
    }
}

fn plan_hash() -> Result<PlanHash, PlanHashError> {
    PlanHash::new(PLAN_HASH.to_owned())
}

fn impact_set_hash() -> ImpactSetHash {
    ImpactSetHash(IMPACT_SET_HASH.to_owned())
}
