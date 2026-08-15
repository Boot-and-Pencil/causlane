//! Append-only causal protocol history adapters.

#[cfg(feature = "postgres-protocol-history")]
mod postgres;
#[cfg(feature = "sqlite-protocol-history")]
mod sqlite;

use std::collections::HashSet;

use causlane_core::{CausalProtocolEvent, CausalProtocolEventId, CausalProtocolHistoryPort};

#[cfg(feature = "postgres-protocol-history")]
pub use self::postgres::{PostgresCausalProtocolHistory, POSTGRES_CREATE_CAUSAL_PROTOCOL_EVENTS};
#[cfg(feature = "sqlite-protocol-history")]
pub use self::sqlite::{SqliteCausalProtocolHistory, SQLITE_CREATE_CAUSAL_PROTOCOL_EVENTS};

/// Error returned by runtime causal protocol history adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CausalProtocolHistoryAdapterError {
    /// The journal already contains this event id.
    DuplicateEventId {
        /// Duplicate event id.
        event_id: CausalProtocolEventId,
    },
    /// A storage projection was requested before an event had a journal index.
    MissingEventIndex {
        /// Event missing its journal position.
        event_id: CausalProtocolEventId,
    },
    /// The supplied event index does not match the next append position.
    NonMonotonicEventIndex {
        /// Next index the adapter would accept.
        expected: u64,
        /// Index supplied by the event or loaded row.
        got: u64,
    },
    /// The append index exceeded the adapter's supported range.
    EventIndexOverflow {
        /// Last accepted index.
        last: u64,
    },
    /// A storage-specific operation failed.
    Storage {
        /// Adapter that produced the failure.
        adapter: &'static str,
        /// Storage error message.
        message: String,
    },
}

impl CausalProtocolHistoryAdapterError {
    pub(crate) fn storage(adapter: &'static str, error: impl std::fmt::Display) -> Self {
        Self::Storage {
            adapter,
            message: error.to_string(),
        }
    }
}

impl std::fmt::Display for CausalProtocolHistoryAdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateEventId { event_id } => {
                write!(f, "duplicate causal protocol event id {}", event_id.0)
            }
            Self::MissingEventIndex { event_id } => {
                write!(
                    f,
                    "causal protocol event {} is missing event_index",
                    event_id.0
                )
            }
            Self::NonMonotonicEventIndex { expected, got } => {
                write!(
                    f,
                    "non-monotonic causal protocol event index: expected {expected}, got {got}"
                )
            }
            Self::EventIndexOverflow { last } => {
                write!(f, "causal protocol event index overflow after {last}")
            }
            Self::Storage { adapter, message } => {
                write!(f, "{adapter} causal protocol history storage: {message}")
            }
        }
    }
}

impl std::error::Error for CausalProtocolHistoryAdapterError {}

/// Stable storage projection shared by durable causal protocol history adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CausalProtocolEventEnvelope {
    /// Monotonic journal position.
    pub event_index: u64,
    /// Unique causal protocol event id.
    pub event_id: String,
    /// Action id recorded on the event.
    pub action_id: String,
    /// Optional canonical plan hash.
    pub plan_hash: Option<String>,
    /// Stable dotted causal protocol event-kind token.
    pub kind: &'static str,
    /// Correlation id for this action invocation.
    pub correlation_id: String,
    /// Directly-causing causal protocol event id, when present.
    pub causation_id: Option<String>,
    /// Occurrence timestamp, when recorded.
    pub occurred_at: Option<u64>,
    /// Planned impact-set hash, when recorded.
    pub impact_set_hash: Option<String>,
    /// Drain fence scope, when recorded.
    pub drain_fence_scope: Option<String>,
}

impl CausalProtocolEventEnvelope {
    /// Project an indexed causal protocol event into the stable storage envelope.
    #[must_use = "causal protocol event envelope projection can fail if the event is not indexed"]
    pub fn from_event(
        event: &CausalProtocolEvent,
    ) -> Result<Self, CausalProtocolHistoryAdapterError> {
        let event_index = event.event_index.ok_or_else(|| {
            CausalProtocolHistoryAdapterError::MissingEventIndex {
                event_id: event.event_id.clone(),
            }
        })?;

        Ok(Self {
            event_index,
            event_id: event.event_id.0.clone(),
            action_id: event.action_id.0.clone(),
            plan_hash: event.plan_hash.as_ref().map(ToString::to_string),
            kind: event.kind.stable_token(),
            correlation_id: event.correlation_id.0.clone(),
            causation_id: event
                .causation_id
                .as_ref()
                .map(|event_id| event_id.0.clone()),
            occurred_at: event.occurred_at.map(|timestamp| timestamp.0),
            impact_set_hash: event.impact_set_hash.as_ref().map(ToString::to_string),
            drain_fence_scope: event
                .drain_fence_scope
                .as_ref()
                .map(|scope| scope.0.clone()),
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PreparedCausalProtocolEvent {
    pub(crate) event_id: CausalProtocolEventId,
    pub(crate) event: CausalProtocolEvent,
}

pub(crate) fn prepared_event_ids(
    prepared: &[PreparedCausalProtocolEvent],
) -> Vec<CausalProtocolEventId> {
    prepared
        .iter()
        .map(|prepared_event| prepared_event.event_id.clone())
        .collect()
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CausalProtocolHistoryAppendState {
    next_index: u64,
    seen_ids: HashSet<CausalProtocolEventId>,
}

impl CausalProtocolHistoryAppendState {
    pub(crate) fn prepare_batch<I>(
        &self,
        events: I,
    ) -> Result<(Self, Vec<PreparedCausalProtocolEvent>), CausalProtocolHistoryAdapterError>
    where
        I: IntoIterator<Item = CausalProtocolEvent>,
    {
        let mut state = self.clone();
        let mut prepared = Vec::new();
        for event in events {
            prepared.push(state.prepare_one(event)?);
        }
        Ok((state, prepared))
    }

    #[cfg(any(
        feature = "postgres-protocol-history",
        feature = "sqlite-protocol-history"
    ))]
    pub(crate) fn record_loaded(
        &mut self,
        event_id: CausalProtocolEventId,
        event_index: u64,
    ) -> Result<(), CausalProtocolHistoryAdapterError> {
        self.accept(event_id, event_index)
    }

    fn prepare_one(
        &mut self,
        mut event: CausalProtocolEvent,
    ) -> Result<PreparedCausalProtocolEvent, CausalProtocolHistoryAdapterError> {
        let event_id = event.event_id.clone();
        let event_index = event.event_index.unwrap_or(self.next_index);
        self.accept(event_id.clone(), event_index)?;
        event.event_index = Some(event_index);

        Ok(PreparedCausalProtocolEvent { event_id, event })
    }

    fn accept(
        &mut self,
        event_id: CausalProtocolEventId,
        event_index: u64,
    ) -> Result<(), CausalProtocolHistoryAdapterError> {
        if self.seen_ids.contains(&event_id) {
            return Err(CausalProtocolHistoryAdapterError::DuplicateEventId { event_id });
        }
        if event_index != self.next_index {
            return Err(CausalProtocolHistoryAdapterError::NonMonotonicEventIndex {
                expected: self.next_index,
                got: event_index,
            });
        }
        let next_index = self.next_index.checked_add(1).ok_or(
            CausalProtocolHistoryAdapterError::EventIndexOverflow {
                last: self.next_index,
            },
        )?;
        self.seen_ids.insert(event_id);
        self.next_index = next_index;
        Ok(())
    }
}

/// In-memory append-only [`CausalProtocolHistoryPort`] adapter.
#[derive(Clone, Debug, Default)]
pub struct InMemoryCausalProtocolHistory {
    /// Events appended so far, in journal order.
    pub events: Vec<CausalProtocolEvent>,
    state: CausalProtocolHistoryAppendState,
}

impl InMemoryCausalProtocolHistory {
    /// Append a batch atomically.
    #[must_use = "protocol-history append failures must be handled"]
    pub fn append_batch<I>(
        &mut self,
        events: I,
    ) -> Result<Vec<CausalProtocolEventId>, CausalProtocolHistoryAdapterError>
    where
        I: IntoIterator<Item = CausalProtocolEvent>,
    {
        let (state, prepared) = self.state.prepare_batch(events)?;
        let event_ids = prepared_event_ids(&prepared);
        self.events.extend(
            prepared
                .into_iter()
                .map(|prepared_event| prepared_event.event),
        );
        self.state = state;
        Ok(event_ids)
    }

    /// Borrow appended events in journal order.
    #[must_use]
    pub fn events(&self) -> &[CausalProtocolEvent] {
        &self.events
    }
}

impl CausalProtocolHistoryPort for InMemoryCausalProtocolHistory {
    type Error = CausalProtocolHistoryAdapterError;

    fn append_batch(
        &mut self,
        events: Vec<CausalProtocolEvent>,
    ) -> Result<Vec<CausalProtocolEventId>, Self::Error> {
        InMemoryCausalProtocolHistory::append_batch(self, events)
    }

    fn append(&mut self, event: CausalProtocolEvent) -> Result<CausalProtocolEventId, Self::Error> {
        let mut event_ids = <Self as CausalProtocolHistoryPort>::append_batch(self, vec![event])?;
        event_ids.pop().ok_or_else(|| {
            CausalProtocolHistoryAdapterError::storage("memory", "append produced no event id")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CausalProtocolEventEnvelope, CausalProtocolHistoryAdapterError,
        InMemoryCausalProtocolHistory,
    };
    use causlane_core::{
        ActionId, CausalProtocolEvent, CausalProtocolEventId, CausalProtocolEventKind,
        CausalProtocolHistoryPort, CorrelationId, ImpactSetHash, PlanHash, PlanHashError, Scope,
        Timestamp,
    };

    #[derive(Debug, PartialEq, Eq)]
    enum TestError {
        Audit,
        PlanHash,
    }

    impl From<CausalProtocolHistoryAdapterError> for TestError {
        fn from(_error: CausalProtocolHistoryAdapterError) -> Self {
            Self::Audit
        }
    }

    impl From<PlanHashError> for TestError {
        fn from(_error: PlanHashError) -> Self {
            Self::PlanHash
        }
    }

    fn event(id: &str) -> CausalProtocolEvent {
        event_kind(id, CausalProtocolEventKind::ExecutionStarted)
    }

    fn event_kind(id: &str, kind: CausalProtocolEventKind) -> CausalProtocolEvent {
        CausalProtocolEvent::new(
            CausalProtocolEventId(id.to_owned()),
            ActionId("action-1".to_owned()),
            kind,
        )
    }

    fn plan_hash() -> Result<PlanHash, PlanHashError> {
        PlanHash::new(format!("sha256:{}", "1".repeat(PlanHash::DIGEST_LEN)))
    }

    #[test]
    fn in_memory_assigns_missing_indexes_monotonically() {
        let mut audit = InMemoryCausalProtocolHistory::default();

        assert_eq!(
            audit.append(event("event-1")),
            Ok(CausalProtocolEventId("event-1".to_owned()))
        );
        assert_eq!(
            audit.append(event("event-2")),
            Ok(CausalProtocolEventId("event-2".to_owned()))
        );

        let indexes: Vec<_> = audit
            .events()
            .iter()
            .map(|event| event.event_index)
            .collect();
        assert_eq!(indexes, vec![Some(0), Some(1)]);
    }

    #[test]
    fn in_memory_rejects_duplicate_event_ids() {
        let mut audit = InMemoryCausalProtocolHistory::default();
        assert_eq!(
            audit.append(event("event-1")),
            Ok(CausalProtocolEventId("event-1".to_owned()))
        );

        assert_eq!(
            audit.append(event("event-1")),
            Err(CausalProtocolHistoryAdapterError::DuplicateEventId {
                event_id: CausalProtocolEventId("event-1".to_owned())
            })
        );
        assert_eq!(audit.events().len(), 1);
    }

    #[test]
    fn in_memory_rejects_non_monotonic_supplied_index() {
        let mut audit = InMemoryCausalProtocolHistory::default();

        assert_eq!(
            audit.append(event("event-1").with_event_index(7)),
            Err(CausalProtocolHistoryAdapterError::NonMonotonicEventIndex {
                expected: 0,
                got: 7
            })
        );
        assert!(audit.events().is_empty());
    }

    #[test]
    fn in_memory_batch_is_all_or_nothing() {
        let mut audit = InMemoryCausalProtocolHistory::default();

        assert_eq!(
            audit.append_batch([event("event-1"), event("event-1")]),
            Err(CausalProtocolHistoryAdapterError::DuplicateEventId {
                event_id: CausalProtocolEventId("event-1".to_owned())
            })
        );
        assert!(audit.events().is_empty());

        assert_eq!(
            audit.append(event("event-2")),
            Ok(CausalProtocolEventId("event-2".to_owned()))
        );
        assert_eq!(
            audit.events().first().map(|event| event.event_index),
            Some(Some(0))
        );
    }

    #[test]
    fn causal_protocol_history_port_batch_preserves_barrier_write_ahead_order(
    ) -> Result<(), CausalProtocolHistoryAdapterError> {
        let mut audit = InMemoryCausalProtocolHistory::default();

        let event_ids = CausalProtocolHistoryPort::append_batch(
            &mut audit,
            vec![
                event_kind("barrier", CausalProtocolEventKind::ExecutionBarrierLogged),
                event_kind("started", CausalProtocolEventKind::ExecutionStarted)
                    .with_causation_id(CausalProtocolEventId("barrier".to_owned())),
            ],
        )?;

        assert_eq!(
            event_ids,
            vec![
                CausalProtocolEventId("barrier".to_owned()),
                CausalProtocolEventId("started".to_owned())
            ]
        );
        let recorded = audit
            .events()
            .iter()
            .map(|event| (event.event_index, event.event_id.clone(), event.kind))
            .collect::<Vec<_>>();
        assert_eq!(
            recorded,
            vec![
                (
                    Some(0),
                    CausalProtocolEventId("barrier".to_owned()),
                    CausalProtocolEventKind::ExecutionBarrierLogged
                ),
                (
                    Some(1),
                    CausalProtocolEventId("started".to_owned()),
                    CausalProtocolEventKind::ExecutionStarted
                )
            ]
        );
        Ok(())
    }

    #[test]
    fn causal_protocol_history_port_batch_failure_leaves_state_unchanged() {
        let mut audit = InMemoryCausalProtocolHistory::default();
        assert_eq!(
            audit.append(event("event-1")),
            Ok(CausalProtocolEventId("event-1".to_owned()))
        );

        assert_eq!(
            CausalProtocolHistoryPort::append_batch(
                &mut audit,
                vec![event("event-2"), event("event-2")]
            ),
            Err(CausalProtocolHistoryAdapterError::DuplicateEventId {
                event_id: CausalProtocolEventId("event-2".to_owned())
            })
        );
        assert_eq!(audit.events().len(), 1);

        assert_eq!(
            audit.append(event("event-3")),
            Ok(CausalProtocolEventId("event-3".to_owned()))
        );
        assert_eq!(
            audit.events().last().map(|event| event.event_index),
            Some(Some(1))
        );
    }

    #[test]
    fn causal_protocol_history_port_batch_rejects_non_monotonic_without_advancing() {
        let mut audit = InMemoryCausalProtocolHistory::default();

        assert_eq!(
            CausalProtocolHistoryPort::append_batch(
                &mut audit,
                vec![event("event-1"), event("event-2").with_event_index(7)]
            ),
            Err(CausalProtocolHistoryAdapterError::NonMonotonicEventIndex {
                expected: 1,
                got: 7,
            })
        );
        assert!(audit.events().is_empty());

        assert_eq!(
            audit.append(event("event-3")),
            Ok(CausalProtocolEventId("event-3".to_owned()))
        );
        assert_eq!(
            audit.events().first().map(|event| event.event_index),
            Some(Some(0))
        );
    }

    #[test]
    fn storage_envelope_preserves_protocol_history_boundary_fields() -> Result<(), TestError> {
        let event = event("event-1")
            .with_plan_hash(plan_hash()?)
            .with_correlation_id(CorrelationId("corr-1".to_owned()))
            .with_causation_id(CausalProtocolEventId("parent-1".to_owned()))
            .with_occurred_at(Timestamp(42))
            .with_impact_set_hash(ImpactSetHash("impact-1".to_owned()))
            .with_drain_fence_scope(Scope("scope-1".to_owned()))
            .with_event_index(3);

        let envelope = CausalProtocolEventEnvelope::from_event(&event)?;

        assert_eq!(envelope.event_index, 3);
        assert_eq!(envelope.event_id, "event-1");
        assert_eq!(envelope.action_id, "action-1");
        assert_eq!(
            envelope.plan_hash,
            Some(format!("sha256:{}", "1".repeat(PlanHash::DIGEST_LEN)))
        );
        assert_eq!(envelope.kind, "execution.started");
        assert_eq!(envelope.correlation_id, "corr-1");
        assert_eq!(envelope.causation_id, Some("parent-1".to_owned()));
        assert_eq!(envelope.occurred_at, Some(42));
        assert_eq!(envelope.impact_set_hash, Some("impact-1".to_owned()));
        assert_eq!(envelope.drain_fence_scope, Some("scope-1".to_owned()));
        Ok(())
    }
}
