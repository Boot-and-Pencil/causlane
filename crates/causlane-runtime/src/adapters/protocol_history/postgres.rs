//! `PostgreSQL` causal protocol history adapter.

use causlane_core::{CausalProtocolEvent, CausalProtocolEventId, CausalProtocolHistoryPort};
use postgres::types::ToSql;
use postgres::{Client, Transaction};

use super::{
    prepared_event_ids, CausalProtocolEventEnvelope, CausalProtocolHistoryAdapterError,
    CausalProtocolHistoryAppendState, PreparedCausalProtocolEvent,
};

/// `PostgreSQL` DDL for the append-only causal protocol event envelope table.
pub const POSTGRES_CREATE_CAUSAL_PROTOCOL_EVENTS: &str = r"
CREATE TABLE IF NOT EXISTS causlane_audit_events (
    event_index BIGINT PRIMARY KEY CHECK (event_index >= 0),
    event_id TEXT NOT NULL UNIQUE,
    action_id TEXT NOT NULL,
    plan_hash TEXT,
    kind TEXT NOT NULL,
    correlation_id TEXT NOT NULL,
    causation_id TEXT,
    occurred_at BIGINT CHECK (occurred_at IS NULL OR occurred_at >= 0),
    impact_set_hash TEXT,
    drain_fence_scope TEXT
);
";

/// `PostgreSQL` insert statement for one causal protocol event envelope row.
pub const POSTGRES_INSERT_CAUSAL_PROTOCOL_EVENT: &str = r"
INSERT INTO causlane_audit_events (
    event_index,
    event_id,
    action_id,
    plan_hash,
    kind,
    correlation_id,
    causation_id,
    occurred_at,
    impact_set_hash,
    drain_fence_scope
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10);
";

const POSTGRES_SELECT_AUDIT_STATE: &str = r"
SELECT event_index, event_id
FROM causlane_audit_events
ORDER BY event_index ASC;
";

/// `PostgreSQL`-backed append-only [`CausalProtocolHistoryPort`] adapter.
pub struct PostgresCausalProtocolHistory {
    client: Client,
    state: CausalProtocolHistoryAppendState,
}

impl PostgresCausalProtocolHistory {
    /// Create an adapter around an existing `PostgreSQL` client.
    #[must_use = "creating the PostgreSQL causal protocol history can fail"]
    pub fn new(mut client: Client) -> Result<Self, CausalProtocolHistoryAdapterError> {
        Self::ensure_schema(&mut client)?;
        let state = load_state(&mut client)?;
        Ok(Self { client, state })
    }

    /// Ensure the causal protocol event envelope schema exists.
    #[must_use = "schema creation failures must be handled"]
    pub fn ensure_schema(client: &mut Client) -> Result<(), CausalProtocolHistoryAdapterError> {
        client
            .batch_execute(POSTGRES_CREATE_CAUSAL_PROTOCOL_EVENTS)
            .map_err(postgres_error)
    }

    /// Append a batch transactionally.
    #[must_use = "protocol-history append failures must be handled"]
    pub fn append_batch<I>(
        &mut self,
        events: I,
    ) -> Result<Vec<CausalProtocolEventId>, CausalProtocolHistoryAdapterError>
    where
        I: IntoIterator<Item = CausalProtocolEvent>,
    {
        let (state, prepared) = self.state.prepare_batch(events)?;
        insert_batch(&mut self.client, &prepared)?;
        let event_ids = prepared_event_ids(&prepared);
        self.state = state;
        Ok(event_ids)
    }

    /// Mutably borrow the underlying `PostgreSQL` client.
    #[must_use]
    pub fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }

    /// Return the underlying `PostgreSQL` client.
    #[must_use]
    pub fn into_client(self) -> Client {
        self.client
    }
}

impl CausalProtocolHistoryPort for PostgresCausalProtocolHistory {
    type Error = CausalProtocolHistoryAdapterError;

    fn append_batch(
        &mut self,
        events: Vec<CausalProtocolEvent>,
    ) -> Result<Vec<CausalProtocolEventId>, Self::Error> {
        PostgresCausalProtocolHistory::append_batch(self, events)
    }

    fn append(&mut self, event: CausalProtocolEvent) -> Result<CausalProtocolEventId, Self::Error> {
        let mut event_ids = <Self as CausalProtocolHistoryPort>::append_batch(self, vec![event])?;
        event_ids.pop().ok_or_else(|| {
            CausalProtocolHistoryAdapterError::storage("postgres", "append produced no event id")
        })
    }
}

fn insert_batch(
    client: &mut Client,
    prepared: &[PreparedCausalProtocolEvent],
) -> Result<(), CausalProtocolHistoryAdapterError> {
    let mut transaction = client.transaction().map_err(postgres_error)?;
    for prepared_event in prepared {
        let envelope = CausalProtocolEventEnvelope::from_event(&prepared_event.event)?;
        insert_envelope(&mut transaction, &envelope)?;
    }
    transaction.commit().map_err(postgres_error)
}

fn insert_envelope(
    transaction: &mut Transaction<'_>,
    envelope: &CausalProtocolEventEnvelope,
) -> Result<(), CausalProtocolHistoryAdapterError> {
    let event_index = postgres_i64(envelope.event_index)?;
    let occurred_at = envelope.occurred_at.map(postgres_i64).transpose()?;
    let plan_hash = envelope.plan_hash.as_deref();
    let causation_id = envelope.causation_id.as_deref();
    let impact_set_hash = envelope.impact_set_hash.as_deref();
    let drain_fence_scope = envelope.drain_fence_scope.as_deref();
    let params: &[&(dyn ToSql + Sync)] = &[
        &event_index,
        &envelope.event_id,
        &envelope.action_id,
        &plan_hash,
        &envelope.kind,
        &envelope.correlation_id,
        &causation_id,
        &occurred_at,
        &impact_set_hash,
        &drain_fence_scope,
    ];

    transaction
        .execute(POSTGRES_INSERT_CAUSAL_PROTOCOL_EVENT, params)
        .map(|_rows| ())
        .map_err(postgres_error)
}

fn load_state(
    client: &mut Client,
) -> Result<CausalProtocolHistoryAppendState, CausalProtocolHistoryAdapterError> {
    let rows = client
        .query(POSTGRES_SELECT_AUDIT_STATE, &[])
        .map_err(postgres_error)?;
    let mut state = CausalProtocolHistoryAppendState::default();

    for row in rows {
        let event_index = postgres_u64(row.get::<_, i64>(0))?;
        let event_id = CausalProtocolEventId(row.get::<_, String>(1));
        state.record_loaded(event_id, event_index)?;
    }

    Ok(state)
}

fn postgres_i64(value: u64) -> Result<i64, CausalProtocolHistoryAdapterError> {
    i64::try_from(value).map_err(|_error| {
        CausalProtocolHistoryAdapterError::storage(
            "postgres",
            format!("event_index {value} exceeds i64"),
        )
    })
}

fn postgres_u64(value: i64) -> Result<u64, CausalProtocolHistoryAdapterError> {
    u64::try_from(value).map_err(|_error| {
        CausalProtocolHistoryAdapterError::storage(
            "postgres",
            format!("negative event_index {value}"),
        )
    })
}

fn postgres_error(error: postgres::Error) -> CausalProtocolHistoryAdapterError {
    CausalProtocolHistoryAdapterError::storage("postgres", error)
}

#[cfg(test)]
mod tests {
    use super::{POSTGRES_CREATE_CAUSAL_PROTOCOL_EVENTS, POSTGRES_INSERT_CAUSAL_PROTOCOL_EVENT};
    use crate::adapters::protocol_history::{
        CausalProtocolEventEnvelope, CausalProtocolHistoryAdapterError,
    };
    use causlane_core::{
        ActionId, CausalProtocolEvent, CausalProtocolEventId, CausalProtocolEventKind,
    };

    fn event(id: &str) -> CausalProtocolEvent {
        CausalProtocolEvent::new(
            CausalProtocolEventId(id.to_owned()),
            ActionId("action-1".to_owned()),
            CausalProtocolEventKind::ExecutionStarted,
        )
        .with_event_index(0)
    }

    #[test]
    fn ddl_declares_append_only_keys_and_constraints() {
        assert!(POSTGRES_CREATE_CAUSAL_PROTOCOL_EVENTS.contains("event_index BIGINT PRIMARY KEY"));
        assert!(POSTGRES_CREATE_CAUSAL_PROTOCOL_EVENTS.contains("event_id TEXT NOT NULL UNIQUE"));
        assert!(POSTGRES_CREATE_CAUSAL_PROTOCOL_EVENTS.contains("CHECK (event_index >= 0)"));
    }

    #[test]
    fn insert_statement_uses_expected_envelope_columns() {
        for column in [
            "event_index",
            "event_id",
            "action_id",
            "plan_hash",
            "kind",
            "correlation_id",
            "causation_id",
            "occurred_at",
            "impact_set_hash",
            "drain_fence_scope",
        ] {
            assert!(POSTGRES_INSERT_CAUSAL_PROTOCOL_EVENT.contains(column));
        }
    }

    #[test]
    fn envelope_conversion_is_shared_with_other_adapters(
    ) -> Result<(), CausalProtocolHistoryAdapterError> {
        let envelope = CausalProtocolEventEnvelope::from_event(&event("event-1"))?;

        assert_eq!(envelope.event_index, 0);
        assert_eq!(envelope.event_id, "event-1");
        assert_eq!(envelope.kind, "execution.started");
        Ok(())
    }
}
