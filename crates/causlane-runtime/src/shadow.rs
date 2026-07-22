//! Shadow-mode comparison for in-process runtime events.
//!
//! Shadow diagnostics compare host-provided expectations with events that were
//! already emitted by the runtime. They never feed back into scheduling,
//! admission, retries, or execution.

use causlane_core::HostDispatchError;

use crate::{in_process::InProcessRuntimeEvent, partitions::PartitionKey};

/// Expected runtime outcome for one host task in shadow mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShadowExpectation {
    /// Optional partition constraint. When absent, any partition can match.
    pub partition: Option<PartitionKey>,
    /// Expected task id.
    pub task_id: String,
    /// Expected lifecycle outcome.
    pub kind: ShadowExpectationKind,
}

impl ShadowExpectation {
    /// Create an expectation scoped only by task id.
    #[must_use]
    pub fn new(task_id: impl Into<String>, kind: ShadowExpectationKind) -> Self {
        Self {
            partition: None,
            task_id: task_id.into(),
            kind,
        }
    }

    /// Create an expectation scoped by partition and task id.
    #[must_use]
    pub fn in_partition(
        partition: PartitionKey,
        task_id: impl Into<String>,
        kind: ShadowExpectationKind,
    ) -> Self {
        Self {
            partition: Some(partition),
            task_id: task_id.into(),
            kind,
        }
    }

    fn matches(&self, observation: &ShadowObservation) -> bool {
        self.scope_matches(observation) && outcome_matches(&self.kind, &observation.kind)
    }

    fn scope_matches(&self, observation: &ShadowObservation) -> bool {
        if observation.task_id.as_deref() != Some(self.task_id.as_str()) {
            return false;
        }

        match &self.partition {
            Some(partition) => observation.partition == *partition,
            None => true,
        }
    }
}

/// Runtime outcome kind used by shadow expectations and observations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShadowExpectationKind {
    /// A task was admitted.
    Accepted,
    /// A task was rejected before admission. `None` accepts any rejection error.
    Rejected {
        /// Optional exact rejection error.
        error: Option<HostDispatchError>,
    },
    /// A task could not be admitted because the queue was full.
    QueueFull,
    /// A task could not acquire route admission locks without waiting.
    RouteBusy,
    /// A task is blocked on missing dependencies.
    Blocked {
        /// Expected missing dependencies in runtime-emitted order.
        missing_dependencies: Vec<String>,
    },
    /// A task completed and produced host-visible references.
    Executed {
        /// Expected produced refs in runtime-emitted order.
        produced_refs: Vec<String>,
    },
    /// A task failed during host-effect execution. `None` accepts any failure.
    Failed {
        /// Optional exact failure error.
        error: Option<HostDispatchError>,
    },
}

/// Runtime event normalized for shadow comparison.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShadowObservation {
    /// Observed partition.
    pub partition: PartitionKey,
    /// Observed task id, when the runtime event carried one.
    pub task_id: Option<String>,
    /// Observed lifecycle outcome.
    pub kind: ShadowExpectationKind,
}

impl From<&InProcessRuntimeEvent> for ShadowObservation {
    fn from(event: &InProcessRuntimeEvent) -> Self {
        match event {
            InProcessRuntimeEvent::Accepted { partition, ticket } => Self {
                partition: partition.clone(),
                task_id: Some(ticket.task_id.clone()),
                kind: ShadowExpectationKind::Accepted,
            },
            InProcessRuntimeEvent::Rejected {
                partition,
                task_id,
                error,
            } => Self {
                partition: partition.clone(),
                task_id: task_id.clone(),
                kind: ShadowExpectationKind::Rejected {
                    error: Some(error.clone()),
                },
            },
            InProcessRuntimeEvent::QueueFull { partition, task_id } => Self {
                partition: partition.clone(),
                task_id: Some(task_id.clone()),
                kind: ShadowExpectationKind::QueueFull,
            },
            InProcessRuntimeEvent::RouteBusy { partition, task_id } => Self {
                partition: partition.clone(),
                task_id: Some(task_id.clone()),
                kind: ShadowExpectationKind::RouteBusy,
            },
            InProcessRuntimeEvent::Blocked {
                partition,
                task_id,
                missing_dependencies,
            } => Self {
                partition: partition.clone(),
                task_id: Some(task_id.clone()),
                kind: ShadowExpectationKind::Blocked {
                    missing_dependencies: missing_dependencies.clone(),
                },
            },
            InProcessRuntimeEvent::Executed {
                partition,
                task_id,
                produced_refs,
                ..
            } => Self {
                partition: partition.clone(),
                task_id: Some(task_id.clone()),
                kind: ShadowExpectationKind::Executed {
                    produced_refs: produced_refs.clone(),
                },
            },
            InProcessRuntimeEvent::Failed {
                partition,
                task_id,
                error,
            } => Self {
                partition: partition.clone(),
                task_id: Some(task_id.clone()),
                kind: ShadowExpectationKind::Failed {
                    error: Some(error.clone()),
                },
            },
        }
    }
}

/// Shadow comparison result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShadowComparison {
    /// Aggregate comparison status.
    pub status: ShadowStatus,
    /// Expectations that matched an observation.
    pub matched: Vec<ShadowExpectation>,
    /// Expectations that were missing or had a different observed outcome.
    pub mismatches: Vec<ShadowMismatch>,
    /// Observations left unmatched after all expectations were checked.
    pub unexpected: Vec<ShadowObservation>,
}

impl ShadowComparison {
    /// Return true when all expectations matched and no unexpected events remain.
    #[must_use]
    pub fn is_match(&self) -> bool {
        self.status == ShadowStatus::Match
    }
}

/// Aggregate shadow comparison status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowStatus {
    /// Expectations and observations matched exactly.
    Match,
    /// At least one expectation or observation differed.
    Mismatch,
}

/// One shadow expectation that did not match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShadowMismatch {
    /// Expected outcome.
    pub expected: ShadowExpectation,
    /// Observed event for the same keyed task, when one existed.
    pub actual: Option<ShadowObservation>,
}

/// Compare host-provided shadow expectations with in-process runtime events.
#[must_use]
pub fn compare_shadow_events<'a, I>(
    expectations: &[ShadowExpectation],
    events: I,
) -> ShadowComparison
where
    I: IntoIterator<Item = &'a InProcessRuntimeEvent>,
{
    let observations = events
        .into_iter()
        .map(ShadowObservation::from)
        .collect::<Vec<_>>();
    compare_shadow_observations(expectations, observations)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ObservationUse {
    Available,
    Consumed,
}

fn compare_shadow_observations(
    expectations: &[ShadowExpectation],
    observations: Vec<ShadowObservation>,
) -> ShadowComparison {
    let mut consumed = vec![ObservationUse::Available; observations.len()];
    let mut matched = Vec::new();
    let mut mismatches = Vec::new();

    for expectation in expectations {
        if let Some(index) = find_available(&observations, &consumed, |observation| {
            expectation.matches(observation)
        }) {
            if mark_consumed(&mut consumed, index).is_some() {
                matched.push(expectation.clone());
                continue;
            }
        }

        let actual = find_available(&observations, &consumed, |observation| {
            expectation.scope_matches(observation)
        })
        .and_then(|index| consume_observation(&observations, &mut consumed, index));
        mismatches.push(ShadowMismatch {
            expected: expectation.clone(),
            actual,
        });
    }

    let unexpected = observations
        .into_iter()
        .enumerate()
        .filter_map(|(index, observation)| match consumed.get(index) {
            Some(ObservationUse::Available) => Some(observation),
            Some(ObservationUse::Consumed) | None => None,
        })
        .collect::<Vec<_>>();
    let status = if mismatches.is_empty() && unexpected.is_empty() {
        ShadowStatus::Match
    } else {
        ShadowStatus::Mismatch
    };

    ShadowComparison {
        status,
        matched,
        mismatches,
        unexpected,
    }
}

fn find_available(
    observations: &[ShadowObservation],
    consumed: &[ObservationUse],
    matches: impl Fn(&ShadowObservation) -> bool,
) -> Option<usize> {
    observations
        .iter()
        .enumerate()
        .find_map(|(index, observation)| match consumed.get(index) {
            Some(ObservationUse::Available) if matches(observation) => Some(index),
            Some(ObservationUse::Available | ObservationUse::Consumed) | None => None,
        })
}

fn consume_observation(
    observations: &[ShadowObservation],
    consumed: &mut [ObservationUse],
    index: usize,
) -> Option<ShadowObservation> {
    if mark_consumed(consumed, index).is_some() {
        observations.get(index).cloned()
    } else {
        None
    }
}

fn mark_consumed(consumed: &mut [ObservationUse], index: usize) -> Option<()> {
    if let Some(slot) = consumed.get_mut(index) {
        *slot = ObservationUse::Consumed;
        Some(())
    } else {
        None
    }
}

fn outcome_matches(expected: &ShadowExpectationKind, actual: &ShadowExpectationKind) -> bool {
    match (expected, actual) {
        (ShadowExpectationKind::Accepted, ShadowExpectationKind::Accepted)
        | (ShadowExpectationKind::QueueFull, ShadowExpectationKind::QueueFull)
        | (ShadowExpectationKind::RouteBusy, ShadowExpectationKind::RouteBusy) => true,
        (
            ShadowExpectationKind::Rejected { error: expected },
            ShadowExpectationKind::Rejected { error: actual },
        )
        | (
            ShadowExpectationKind::Failed { error: expected },
            ShadowExpectationKind::Failed { error: actual },
        ) => match expected {
            Some(expected) => actual.as_ref() == Some(expected),
            None => true,
        },
        (
            ShadowExpectationKind::Blocked {
                missing_dependencies: expected,
            },
            ShadowExpectationKind::Blocked {
                missing_dependencies: actual,
            },
        ) => dependencies_match(expected, actual),
        (
            ShadowExpectationKind::Executed {
                produced_refs: expected,
            },
            ShadowExpectationKind::Executed {
                produced_refs: actual,
            },
        ) => produced_refs_match(expected, actual),
        _different_outcome => false,
    }
}

fn dependencies_match(expected: &[String], actual: &[String]) -> bool {
    expected == actual
}

fn produced_refs_match(expected: &[String], actual: &[String]) -> bool {
    expected == actual
}

#[cfg(test)]
mod tests {
    use causlane_core::{HostDispatchTicket, CAUSLANE_HOST_API_VERSION};

    use super::*;

    fn partition(id: &str) -> PartitionKey {
        PartitionKey(id.to_owned())
    }

    fn ticket(task_id: &str) -> HostDispatchTicket {
        HostDispatchTicket {
            ticket_id: format!("ticket://{task_id}"),
            task_id: task_id.to_owned(),
            api_version: CAUSLANE_HOST_API_VERSION.to_owned(),
        }
    }

    fn accepted(partition_id: &str, task_id: &str) -> InProcessRuntimeEvent {
        InProcessRuntimeEvent::Accepted {
            partition: partition(partition_id),
            ticket: ticket(task_id),
        }
    }

    fn executed(
        partition_id: &str,
        task_id: &str,
        produced_refs: Vec<&str>,
    ) -> InProcessRuntimeEvent {
        InProcessRuntimeEvent::Executed {
            partition: partition(partition_id),
            task_id: task_id.to_owned(),
            produced_refs: produced_refs.into_iter().map(str::to_owned).collect(),
            action_receipt_ref: Some(format!("receipt://action/{task_id}")),
            audit_ref: format!("audit://host/outcome/{task_id}"),
        }
    }

    fn rejected(
        partition_id: &str,
        task_id: Option<&str>,
        error: HostDispatchError,
    ) -> InProcessRuntimeEvent {
        InProcessRuntimeEvent::Rejected {
            partition: partition(partition_id),
            task_id: task_id.map(str::to_owned),
            error,
        }
    }

    #[test]
    fn exact_accepted_and_executed_expectations_match_order_insensitively() {
        let events = vec![
            executed("p1", "task-1", vec!["fact://task-1"]),
            accepted("p1", "task-1"),
        ];
        let expectations = vec![
            ShadowExpectation::in_partition(
                partition("p1"),
                "task-1",
                ShadowExpectationKind::Accepted,
            ),
            ShadowExpectation::in_partition(
                partition("p1"),
                "task-1",
                ShadowExpectationKind::Executed {
                    produced_refs: vec!["fact://task-1".to_owned()],
                },
            ),
        ];

        let comparison = compare_shadow_events(&expectations, &events);

        assert_eq!(comparison.status, ShadowStatus::Match);
        assert_eq!(comparison.matched, expectations);
        assert!(comparison.mismatches.is_empty());
        assert!(comparison.unexpected.is_empty());
    }

    #[test]
    fn missing_expected_event_yields_mismatch() {
        let event = accepted("p1", "task-1");
        let expected = ShadowExpectation::in_partition(
            partition("p1"),
            "task-1",
            ShadowExpectationKind::Executed {
                produced_refs: vec!["fact://task-1".to_owned()],
            },
        );
        let events = vec![event.clone()];
        let expectations = vec![expected.clone()];

        let comparison = compare_shadow_events(&expectations, &events);

        assert_eq!(comparison.status, ShadowStatus::Mismatch);
        assert_eq!(
            comparison.mismatches,
            vec![ShadowMismatch {
                expected,
                actual: Some(ShadowObservation::from(&event)),
            }]
        );
        assert!(comparison.unexpected.is_empty());
    }

    #[test]
    fn extra_actual_event_is_unexpected() {
        let extra = accepted("p1", "task-2");
        let events = vec![accepted("p1", "task-1"), extra.clone()];
        let expectations = vec![ShadowExpectation::in_partition(
            partition("p1"),
            "task-1",
            ShadowExpectationKind::Accepted,
        )];

        let comparison = compare_shadow_events(&expectations, &events);

        assert_eq!(comparison.status, ShadowStatus::Mismatch);
        assert!(comparison.mismatches.is_empty());
        assert_eq!(comparison.unexpected, vec![ShadowObservation::from(&extra)]);
    }

    #[test]
    fn produced_refs_mismatch_reports_actual_for_same_task() {
        let event = executed("p1", "task-1", vec!["fact://actual"]);
        let expected = ShadowExpectation::new(
            "task-1",
            ShadowExpectationKind::Executed {
                produced_refs: vec!["fact://expected".to_owned()],
            },
        );
        let events = vec![event.clone()];
        let expectations = vec![expected.clone()];

        let comparison = compare_shadow_events(&expectations, &events);

        assert_eq!(comparison.status, ShadowStatus::Mismatch);
        assert_eq!(
            comparison.mismatches,
            vec![ShadowMismatch {
                expected,
                actual: Some(ShadowObservation::from(&event)),
            }]
        );
        assert!(comparison.unexpected.is_empty());
    }

    #[test]
    fn optional_rejected_error_matches_any_rejection() {
        let events = vec![rejected(
            "p1",
            Some("task-1"),
            HostDispatchError::HandlerRejected {
                reason: "host refused".to_owned(),
            },
        )];
        let expectations = vec![ShadowExpectation::new(
            "task-1",
            ShadowExpectationKind::Rejected { error: None },
        )];

        let comparison = compare_shadow_events(&expectations, &events);

        assert_eq!(comparison.status, ShadowStatus::Match);
    }

    #[test]
    fn exact_failed_error_must_match_when_expected() {
        let event = InProcessRuntimeEvent::Failed {
            partition: partition("p1"),
            task_id: "task-1".to_owned(),
            error: HostDispatchError::HandlerRejected {
                reason: "actual".to_owned(),
            },
        };
        let events = vec![event.clone()];
        let expectations = vec![ShadowExpectation::new(
            "task-1",
            ShadowExpectationKind::Failed {
                error: Some(HostDispatchError::HandlerRejected {
                    reason: "expected".to_owned(),
                }),
            },
        )];

        let comparison = compare_shadow_events(&expectations, &events);

        assert_eq!(comparison.status, ShadowStatus::Mismatch);
        assert_eq!(
            comparison
                .mismatches
                .first()
                .and_then(|mismatch| mismatch.actual.clone()),
            Some(ShadowObservation::from(&event))
        );
    }

    #[test]
    fn partition_filter_disambiguates_duplicate_task_ids() {
        let unexpected = executed("left", "same", vec!["fact://left"]);
        let expected = ShadowExpectation::in_partition(
            partition("right"),
            "same",
            ShadowExpectationKind::Executed {
                produced_refs: vec!["fact://right".to_owned()],
            },
        );
        let events = vec![
            unexpected.clone(),
            executed("right", "same", vec!["fact://right"]),
        ];
        let expectations = vec![expected.clone()];

        let comparison = compare_shadow_events(&expectations, &events);

        assert_eq!(comparison.status, ShadowStatus::Mismatch);
        assert_eq!(
            comparison.unexpected,
            vec![ShadowObservation::from(&unexpected)]
        );
        assert_eq!(comparison.matched, vec![expected]);
    }

    #[test]
    fn unkeyed_rejection_cannot_satisfy_keyed_expectation() {
        let event = rejected("p1", None, HostDispatchError::MissingTaskId);
        let expected =
            ShadowExpectation::new("task-1", ShadowExpectationKind::Rejected { error: None });
        let events = vec![event.clone()];
        let expectations = vec![expected.clone()];

        let comparison = compare_shadow_events(&expectations, &events);

        assert_eq!(comparison.status, ShadowStatus::Mismatch);
        assert_eq!(
            comparison.mismatches,
            vec![ShadowMismatch {
                expected,
                actual: None,
            }]
        );
        assert_eq!(comparison.unexpected, vec![ShadowObservation::from(&event)]);
    }
}
