//! Append-only audit log adapters.

#[cfg(feature = "postgres-audit")]
mod postgres;
#[cfg(feature = "sqlite-audit")]
mod sqlite;

use std::collections::HashSet;

use causlane_core::{AuditEvent, AuditEventId, AuditLogPort};

#[cfg(feature = "postgres-audit")]
pub use self::postgres::{PostgresAuditLog, POSTGRES_CREATE_AUDIT_EVENTS};
#[cfg(feature = "sqlite-audit")]
pub use self::sqlite::{SqliteAuditLog, SQLITE_CREATE_AUDIT_EVENTS};

/// Error returned by runtime audit adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditAdapterError {
    /// The journal already contains this event id.
    DuplicateEventId {
        /// Duplicate event id.
        event_id: AuditEventId,
    },
    /// A storage projection was requested before an event had a journal index.
    MissingEventIndex {
        /// Event missing its journal position.
        event_id: AuditEventId,
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

impl AuditAdapterError {
    pub(crate) fn storage(adapter: &'static str, error: impl std::fmt::Display) -> Self {
        Self::Storage {
            adapter,
            message: error.to_string(),
        }
    }
}

impl std::fmt::Display for AuditAdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateEventId { event_id } => {
                write!(f, "duplicate audit event id {}", event_id.0)
            }
            Self::MissingEventIndex { event_id } => {
                write!(f, "audit event {} is missing event_index", event_id.0)
            }
            Self::NonMonotonicEventIndex { expected, got } => {
                write!(
                    f,
                    "non-monotonic audit event index: expected {expected}, got {got}"
                )
            }
            Self::EventIndexOverflow { last } => {
                write!(f, "audit event index overflow after {last}")
            }
            Self::Storage { adapter, message } => write!(f, "{adapter} audit storage: {message}"),
        }
    }
}

impl std::error::Error for AuditAdapterError {}

/// Stable storage projection shared by durable audit adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditEnvelope {
    /// Monotonic journal position.
    pub event_index: u64,
    /// Unique audit event id.
    pub event_id: String,
    /// Action id recorded on the event.
    pub action_id: String,
    /// Optional canonical plan hash.
    pub plan_hash: Option<String>,
    /// Stable dotted audit kind token.
    pub kind: &'static str,
    /// Correlation id for this action invocation.
    pub correlation_id: String,
    /// Directly-causing audit event id, when present.
    pub causation_id: Option<String>,
    /// Occurrence timestamp, when recorded.
    pub occurred_at: Option<u64>,
    /// Planned impact-set hash, when recorded.
    pub impact_set_hash: Option<String>,
    /// Drain fence scope, when recorded.
    pub drain_fence_scope: Option<String>,
}

impl AuditEnvelope {
    /// Project an indexed audit event into the stable storage envelope.
    #[must_use = "audit envelope projection can fail if the event is not indexed"]
    pub fn from_event(event: &AuditEvent) -> Result<Self, AuditAdapterError> {
        let event_index =
            event
                .event_index
                .ok_or_else(|| AuditAdapterError::MissingEventIndex {
                    event_id: event.event_id.clone(),
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
pub(crate) struct PreparedAuditEvent {
    pub(crate) event_id: AuditEventId,
    pub(crate) event: AuditEvent,
}

pub(crate) fn prepared_event_ids(prepared: &[PreparedAuditEvent]) -> Vec<AuditEventId> {
    prepared
        .iter()
        .map(|prepared_event| prepared_event.event_id.clone())
        .collect()
}

#[derive(Clone, Debug, Default)]
pub(crate) struct AuditAppendState {
    next_index: u64,
    seen_ids: HashSet<AuditEventId>,
}

impl AuditAppendState {
    pub(crate) fn prepare_batch<I>(
        &self,
        events: I,
    ) -> Result<(Self, Vec<PreparedAuditEvent>), AuditAdapterError>
    where
        I: IntoIterator<Item = AuditEvent>,
    {
        let mut state = self.clone();
        let mut prepared = Vec::new();
        for event in events {
            prepared.push(state.prepare_one(event)?);
        }
        Ok((state, prepared))
    }

    #[cfg(any(feature = "postgres-audit", feature = "sqlite-audit"))]
    pub(crate) fn record_loaded(
        &mut self,
        event_id: AuditEventId,
        event_index: u64,
    ) -> Result<(), AuditAdapterError> {
        self.accept(event_id, event_index)
    }

    fn prepare_one(
        &mut self,
        mut event: AuditEvent,
    ) -> Result<PreparedAuditEvent, AuditAdapterError> {
        let event_id = event.event_id.clone();
        let event_index = event.event_index.unwrap_or(self.next_index);
        self.accept(event_id.clone(), event_index)?;
        event.event_index = Some(event_index);

        Ok(PreparedAuditEvent { event_id, event })
    }

    fn accept(
        &mut self,
        event_id: AuditEventId,
        event_index: u64,
    ) -> Result<(), AuditAdapterError> {
        if self.seen_ids.contains(&event_id) {
            return Err(AuditAdapterError::DuplicateEventId { event_id });
        }
        if event_index != self.next_index {
            return Err(AuditAdapterError::NonMonotonicEventIndex {
                expected: self.next_index,
                got: event_index,
            });
        }
        let next_index =
            self.next_index
                .checked_add(1)
                .ok_or(AuditAdapterError::EventIndexOverflow {
                    last: self.next_index,
                })?;
        self.seen_ids.insert(event_id);
        self.next_index = next_index;
        Ok(())
    }
}

/// In-memory append-only [`AuditLogPort`] adapter.
#[derive(Clone, Debug, Default)]
pub struct InMemoryAuditLog {
    /// Events appended so far, in journal order.
    pub events: Vec<AuditEvent>,
    state: AuditAppendState,
}

impl InMemoryAuditLog {
    /// Append a batch atomically.
    #[must_use = "audit append failures must be handled"]
    pub fn append_batch<I>(&mut self, events: I) -> Result<Vec<AuditEventId>, AuditAdapterError>
    where
        I: IntoIterator<Item = AuditEvent>,
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
    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }
}

impl AuditLogPort for InMemoryAuditLog {
    type Error = AuditAdapterError;

    fn append_batch(&mut self, events: Vec<AuditEvent>) -> Result<Vec<AuditEventId>, Self::Error> {
        InMemoryAuditLog::append_batch(self, events)
    }

    fn append(&mut self, event: AuditEvent) -> Result<AuditEventId, Self::Error> {
        let mut event_ids = <Self as AuditLogPort>::append_batch(self, vec![event])?;
        event_ids
            .pop()
            .ok_or_else(|| AuditAdapterError::storage("memory", "append produced no event id"))
    }
}

#[cfg(test)]
mod tests {
    use super::{AuditAdapterError, AuditEnvelope, InMemoryAuditLog};
    use causlane_core::{
        ActionId, AuditEvent, AuditEventId, AuditEventKind, AuditLogPort, CorrelationId,
        ImpactSetHash, PlanHash, PlanHashError, Scope, Timestamp,
    };

    #[derive(Debug, PartialEq, Eq)]
    enum TestError {
        Audit,
        PlanHash,
    }

    impl From<AuditAdapterError> for TestError {
        fn from(_error: AuditAdapterError) -> Self {
            Self::Audit
        }
    }

    impl From<PlanHashError> for TestError {
        fn from(_error: PlanHashError) -> Self {
            Self::PlanHash
        }
    }

    fn event(id: &str) -> AuditEvent {
        event_kind(id, AuditEventKind::ExecutionStarted)
    }

    fn event_kind(id: &str, kind: AuditEventKind) -> AuditEvent {
        AuditEvent::new(
            AuditEventId(id.to_owned()),
            ActionId("action-1".to_owned()),
            kind,
        )
    }

    fn plan_hash() -> Result<PlanHash, PlanHashError> {
        PlanHash::new(format!("sha256:{}", "1".repeat(PlanHash::DIGEST_LEN)))
    }

    #[test]
    fn in_memory_assigns_missing_indexes_monotonically() {
        let mut audit = InMemoryAuditLog::default();

        assert_eq!(
            audit.append(event("event-1")),
            Ok(AuditEventId("event-1".to_owned()))
        );
        assert_eq!(
            audit.append(event("event-2")),
            Ok(AuditEventId("event-2".to_owned()))
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
        let mut audit = InMemoryAuditLog::default();
        assert_eq!(
            audit.append(event("event-1")),
            Ok(AuditEventId("event-1".to_owned()))
        );

        assert_eq!(
            audit.append(event("event-1")),
            Err(AuditAdapterError::DuplicateEventId {
                event_id: AuditEventId("event-1".to_owned())
            })
        );
        assert_eq!(audit.events().len(), 1);
    }

    #[test]
    fn in_memory_rejects_non_monotonic_supplied_index() {
        let mut audit = InMemoryAuditLog::default();

        assert_eq!(
            audit.append(event("event-1").with_event_index(7)),
            Err(AuditAdapterError::NonMonotonicEventIndex {
                expected: 0,
                got: 7
            })
        );
        assert!(audit.events().is_empty());
    }

    #[test]
    fn in_memory_batch_is_all_or_nothing() {
        let mut audit = InMemoryAuditLog::default();

        assert_eq!(
            audit.append_batch([event("event-1"), event("event-1")]),
            Err(AuditAdapterError::DuplicateEventId {
                event_id: AuditEventId("event-1".to_owned())
            })
        );
        assert!(audit.events().is_empty());

        assert_eq!(
            audit.append(event("event-2")),
            Ok(AuditEventId("event-2".to_owned()))
        );
        assert_eq!(
            audit.events().first().map(|event| event.event_index),
            Some(Some(0))
        );
    }

    #[test]
    fn audit_log_port_batch_preserves_barrier_write_ahead_order() -> Result<(), AuditAdapterError> {
        let mut audit = InMemoryAuditLog::default();

        let event_ids = AuditLogPort::append_batch(
            &mut audit,
            vec![
                event_kind("barrier", AuditEventKind::ExecutionBarrierLogged),
                event_kind("started", AuditEventKind::ExecutionStarted)
                    .with_causation_id(AuditEventId("barrier".to_owned())),
            ],
        )?;

        assert_eq!(
            event_ids,
            vec![
                AuditEventId("barrier".to_owned()),
                AuditEventId("started".to_owned())
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
                    AuditEventId("barrier".to_owned()),
                    AuditEventKind::ExecutionBarrierLogged
                ),
                (
                    Some(1),
                    AuditEventId("started".to_owned()),
                    AuditEventKind::ExecutionStarted
                )
            ]
        );
        Ok(())
    }

    #[test]
    fn audit_log_port_batch_failure_leaves_state_unchanged() {
        let mut audit = InMemoryAuditLog::default();
        assert_eq!(
            audit.append(event("event-1")),
            Ok(AuditEventId("event-1".to_owned()))
        );

        assert_eq!(
            AuditLogPort::append_batch(&mut audit, vec![event("event-2"), event("event-2")]),
            Err(AuditAdapterError::DuplicateEventId {
                event_id: AuditEventId("event-2".to_owned())
            })
        );
        assert_eq!(audit.events().len(), 1);

        assert_eq!(
            audit.append(event("event-3")),
            Ok(AuditEventId("event-3".to_owned()))
        );
        assert_eq!(
            audit.events().last().map(|event| event.event_index),
            Some(Some(1))
        );
    }

    #[test]
    fn audit_log_port_batch_rejects_non_monotonic_without_advancing() {
        let mut audit = InMemoryAuditLog::default();

        assert_eq!(
            AuditLogPort::append_batch(
                &mut audit,
                vec![event("event-1"), event("event-2").with_event_index(7)]
            ),
            Err(AuditAdapterError::NonMonotonicEventIndex {
                expected: 1,
                got: 7,
            })
        );
        assert!(audit.events().is_empty());

        assert_eq!(
            audit.append(event("event-3")),
            Ok(AuditEventId("event-3".to_owned()))
        );
        assert_eq!(
            audit.events().first().map(|event| event.event_index),
            Some(Some(0))
        );
    }

    #[test]
    fn storage_envelope_preserves_audit_boundary_fields() -> Result<(), TestError> {
        let event = event("event-1")
            .with_plan_hash(plan_hash()?)
            .with_correlation_id(CorrelationId("corr-1".to_owned()))
            .with_causation_id(AuditEventId("parent-1".to_owned()))
            .with_occurred_at(Timestamp(42))
            .with_impact_set_hash(ImpactSetHash("impact-1".to_owned()))
            .with_drain_fence_scope(Scope("scope-1".to_owned()))
            .with_event_index(3);

        let envelope = AuditEnvelope::from_event(&event)?;

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
