#![forbid(unsafe_code)]
#![deny(warnings)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use causlane::core::protocol::{
    ActionId, FactKind, FrontierBlock, FrontierSelection, GraphIndex, GraphNode, LaneCapacity,
    LaneId, OpId, Scope,
};
use causlane::prelude::select_frontier;
use causlane_contracts::{CompiledDispatchBundle, ContractError, RegistryManifest};
use causlane_replay::{ReplayError, ReplayScenario};

const REGISTRY_YAML: &str =
    include_str!("../../../contracts/examples/release_promote.registry.yaml");
const PARALLELISM_SCENARIO_YAML: &str =
    include_str!("../../../contracts/scenarios/conflict_free_parallelism_success.scenario.yaml");

/// Summary returned by the runnable example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsequenceParallelismSummary {
    /// Local frontier cases checked before replay.
    pub frontier_cases: usize,
    /// Bundle-bound positive replay scenarios verified.
    pub verified_scenarios: usize,
    /// Bundle-bound negative replay scenarios refuted.
    pub refuted_scenarios: usize,
}

/// Error type for the consequence-parallelism example composition.
#[derive(Debug)]
pub enum ConsequenceParallelismError {
    /// Registry or bundle compilation failed.
    Contract(ContractError),
    /// Bundle-bound replay rejected a positive scenario.
    Replay(ReplayError),
    /// The local frontier selector produced an unexpected outcome.
    FrontierOutcome(&'static str),
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

impl fmt::Display for ConsequenceParallelismError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contract(error) => write!(f, "contract fixture failed: {error}"),
            Self::Replay(error) => write!(f, "bundle replay failed: {error}"),
            Self::FrontierOutcome(label) => {
                write!(f, "unexpected frontier outcome: {label}")
            }
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

impl std::error::Error for ConsequenceParallelismError {}

impl From<ContractError> for ConsequenceParallelismError {
    fn from(error: ContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<ReplayError> for ConsequenceParallelismError {
    fn from(error: ReplayError) -> Self {
        Self::Replay(error)
    }
}

/// Run the consequence-parallelism example.
///
/// # Errors
/// Returns an error if local frontier checks or bundle-bound replay fail.
#[must_use = "the runnable example result must be checked"]
pub fn run_consequence_parallelism(
) -> Result<ConsequenceParallelismSummary, ConsequenceParallelismError> {
    let frontier_cases = verify_frontier_cases()?;
    verify_conflict_free_parallelism_replay()?;
    refute_overlapping_parallelism()?;
    Ok(ConsequenceParallelismSummary {
        frontier_cases,
        verified_scenarios: 1,
        refuted_scenarios: 1,
    })
}

/// Verify local frontier-selection outcomes without replay fixtures.
///
/// # Errors
/// Returns an error if a local frontier case diverges from the documented outcome.
#[must_use = "frontier checks can fail and must be inspected"]
pub fn verify_frontier_cases() -> Result<usize, ConsequenceParallelismError> {
    verify_disjoint_ready_ops_select_together()?;
    verify_same_write_scope_rejects_one_op()?;
    verify_full_lane_rejects_ready_op()?;
    Ok(3)
}

/// Verify the bundled conflict-free parallelism scenario.
///
/// # Errors
/// Returns an error if fixture parsing, bundle compilation or replay fails.
#[must_use = "bundle replay can fail and must be inspected"]
pub fn verify_conflict_free_parallelism_replay() -> Result<(), ConsequenceParallelismError> {
    let scenario = ReplayScenario::from_yaml_str(PARALLELISM_SCENARIO_YAML)?;
    scenario
        .to_trace()
        .verify_with_bundle(&release_promote_bundle()?)?;
    Ok(())
}

/// Verify that overlapping exclusive leases are rejected by bundle replay.
///
/// # Errors
/// Returns an error if fixture parsing fails or replay does not reject with
/// `ConflictingLeases`.
#[must_use = "negative replay can fail and must be inspected"]
pub fn refute_overlapping_parallelism() -> Result<(), ConsequenceParallelismError> {
    let mut scenario = ReplayScenario::from_yaml_str(PARALLELISM_SCENARIO_YAML)?;
    for event in &mut scenario.events {
        if event.event_id.as_deref() == Some("evt_b_leases_granted") {
            for lease in &mut event.leases {
                if lease.lease_id == "lease_b_env_prod" {
                    lease.scope = "environment:staging".to_owned();
                }
            }
        }
    }

    let result = scenario
        .to_trace()
        .verify_with_bundle(&release_promote_bundle()?);
    match result {
        Err(ReplayError::ConflictingLeases { .. }) => Ok(()),
        Err(error) => Err(ConsequenceParallelismError::NegativeReplayWrongError {
            scenario: "conflict_free_parallelism_overlap_mutation",
            error,
        }),
        Ok(()) => Err(ConsequenceParallelismError::NegativeReplayPassed(
            "conflict_free_parallelism_overlap_mutation",
        )),
    }
}

fn verify_disjoint_ready_ops_select_together() -> Result<(), ConsequenceParallelismError> {
    let op_staging = op("act_promote_123", 0);
    let op_prod = op("act_promote_456", 0);
    let mut index = GraphIndex::new();
    index.add_node(node(
        op_staging.clone(),
        "release",
        &[],
        &["environment:staging"],
    ));
    index.add_node(node(op_prod.clone(), "release", &[], &["environment:prod"]));

    let selected = select_frontier(&index, &BTreeMap::new());
    require_selected(
        &selected,
        [op_staging, op_prod].into_iter().collect(),
        "disjoint ready ops selected together",
    )
}

fn verify_same_write_scope_rejects_one_op() -> Result<(), ConsequenceParallelismError> {
    let first = op("act_promote_123", 0);
    let second = op("act_promote_456", 0);
    let mut index = GraphIndex::new();
    index.add_node(node(
        first.clone(),
        "release",
        &[],
        &["environment:staging"],
    ));
    index.add_node(node(
        second.clone(),
        "release",
        &[],
        &["environment:staging"],
    ));

    let selected = select_frontier(&index, &BTreeMap::new());
    require_selected(
        &selected,
        [first.clone()].into_iter().collect(),
        "same-scope case selected deterministic winner",
    )?;
    require_rejected_for_scope_conflict(
        &selected,
        &second,
        Scope("environment:staging".to_owned()),
        first,
        "same-scope case rejected conflicting op",
    )
}

fn verify_full_lane_rejects_ready_op() -> Result<(), ConsequenceParallelismError> {
    let active = op("act_promote_123", 0);
    let waiting = op("act_promote_456", 0);
    let release_lane = LaneId("release".to_owned());
    let mut index = GraphIndex::new();
    index.add_node(node(
        active.clone(),
        "release",
        &[],
        &["environment:staging"],
    ));
    index.add_node(node(waiting.clone(), "release", &[], &["environment:prod"]));
    index.mark_active(&active);
    let lanes = [(release_lane.clone(), LaneCapacity::Bounded(1))]
        .into_iter()
        .collect();

    let selected = select_frontier(&index, &lanes);
    require_selected(
        &selected,
        BTreeSet::new(),
        "full lane case selected no additional op",
    )?;
    require_rejected_for_lane(
        &selected,
        &waiting,
        release_lane,
        "full lane case rejected ready op",
    )
}

fn require_selected(
    selection: &FrontierSelection,
    expected: BTreeSet<OpId>,
    label: &'static str,
) -> Result<(), ConsequenceParallelismError> {
    if selection.selected == expected {
        Ok(())
    } else {
        Err(ConsequenceParallelismError::FrontierOutcome(label))
    }
}

fn require_rejected_for_scope_conflict(
    selection: &FrontierSelection,
    op_id: &OpId,
    scope: Scope,
    holder: OpId,
    label: &'static str,
) -> Result<(), ConsequenceParallelismError> {
    let found = selection.rejected.iter().any(|rejection| {
        rejection.op_id == *op_id
            && rejection.reason
                == FrontierBlock::WriteScopeConflict {
                    scope: scope.clone(),
                    with: holder.clone(),
                }
    });
    if found {
        Ok(())
    } else {
        Err(ConsequenceParallelismError::FrontierOutcome(label))
    }
}

fn require_rejected_for_lane(
    selection: &FrontierSelection,
    op_id: &OpId,
    lane: LaneId,
    label: &'static str,
) -> Result<(), ConsequenceParallelismError> {
    let found = selection.rejected.iter().any(|rejection| {
        rejection.op_id == *op_id
            && rejection.reason == FrontierBlock::LaneAtCapacity { lane: lane.clone() }
    });
    if found {
        Ok(())
    } else {
        Err(ConsequenceParallelismError::FrontierOutcome(label))
    }
}

fn release_promote_bundle() -> Result<CompiledDispatchBundle, ContractError> {
    let manifest = RegistryManifest::from_yaml_str(REGISTRY_YAML)?;
    CompiledDispatchBundle::compile(&manifest)
}

fn op(action: &str, index: u32) -> OpId {
    OpId(ActionId(action.to_owned()), index)
}

fn node(op_id: OpId, lane: &str, requires: &[&str], writes: &[&str]) -> GraphNode {
    GraphNode {
        op_id,
        lane: LaneId(lane.to_owned()),
        requires: requires
            .iter()
            .map(|fact| FactKind((*fact).to_owned()))
            .collect(),
        writes: writes
            .iter()
            .map(|scope| Scope((*scope).to_owned()))
            .collect(),
    }
}
