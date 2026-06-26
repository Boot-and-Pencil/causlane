//! `SQLite` audit adapter.

use causlane_core::{AuditEvent, AuditEventId, AuditLogPort};
use rusqlite::{params, Connection, Transaction};

use super::{
    prepared_event_ids, AuditAdapterError, AuditAppendState, AuditEnvelope, PreparedAuditEvent,
};

/// `SQLite` DDL for the append-only audit envelope table.
pub const SQLITE_CREATE_AUDIT_EVENTS: &str = r"
CREATE TABLE IF NOT EXISTS causlane_audit_events (
    event_index INTEGER NOT NULL PRIMARY KEY,
    event_id TEXT NOT NULL UNIQUE,
    action_id TEXT NOT NULL,
    plan_hash TEXT,
    kind TEXT NOT NULL,
    correlation_id TEXT NOT NULL,
    causation_id TEXT,
    occurred_at INTEGER,
    impact_set_hash TEXT,
    drain_fence_scope TEXT
);
";

const SQLITE_INSERT_AUDIT_EVENT: &str = r"
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
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);
";

const SQLITE_SELECT_AUDIT_STATE: &str = r"
SELECT event_index, event_id
FROM causlane_audit_events
ORDER BY event_index ASC;
";

/// SQLite-backed append-only [`AuditLogPort`] adapter.
pub struct SqliteAuditLog {
    connection: Connection,
    state: AuditAppendState,
}

impl SqliteAuditLog {
    /// Open an in-memory `SQLite` audit log.
    #[must_use = "opening the SQLite audit log can fail"]
    pub fn open_in_memory() -> Result<Self, AuditAdapterError> {
        let connection = Connection::open_in_memory().map_err(sqlite_error)?;
        Self::new(connection)
    }

    /// Create an adapter around an existing `SQLite` connection.
    #[must_use = "creating the SQLite audit log can fail"]
    pub fn new(connection: Connection) -> Result<Self, AuditAdapterError> {
        Self::ensure_schema(&connection)?;
        let state = load_state(&connection)?;
        Ok(Self { connection, state })
    }

    /// Ensure the audit envelope schema exists.
    #[must_use = "schema creation failures must be handled"]
    pub fn ensure_schema(connection: &Connection) -> Result<(), AuditAdapterError> {
        connection
            .execute_batch(SQLITE_CREATE_AUDIT_EVENTS)
            .map_err(sqlite_error)
    }

    /// Append a batch transactionally.
    #[must_use = "audit append failures must be handled"]
    pub fn append_batch<I>(&mut self, events: I) -> Result<Vec<AuditEventId>, AuditAdapterError>
    where
        I: IntoIterator<Item = AuditEvent>,
    {
        let (state, prepared) = self.state.prepare_batch(events)?;
        insert_batch(&mut self.connection, &prepared)?;
        let event_ids = prepared_event_ids(&prepared);
        self.state = state;
        Ok(event_ids)
    }

    /// Borrow the underlying `SQLite` connection.
    #[must_use]
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Mutably borrow the underlying `SQLite` connection.
    #[must_use]
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }

    /// Return the underlying `SQLite` connection.
    #[must_use]
    pub fn into_connection(self) -> Connection {
        self.connection
    }
}

impl AuditLogPort for SqliteAuditLog {
    type Error = AuditAdapterError;

    fn append_batch(&mut self, events: Vec<AuditEvent>) -> Result<Vec<AuditEventId>, Self::Error> {
        SqliteAuditLog::append_batch(self, events)
    }

    fn append(&mut self, event: AuditEvent) -> Result<AuditEventId, Self::Error> {
        let mut event_ids = <Self as AuditLogPort>::append_batch(self, vec![event])?;
        event_ids
            .pop()
            .ok_or_else(|| AuditAdapterError::storage("sqlite", "append produced no event id"))
    }
}

fn insert_batch(
    connection: &mut Connection,
    prepared: &[PreparedAuditEvent],
) -> Result<(), AuditAdapterError> {
    let transaction = connection.transaction().map_err(sqlite_error)?;
    for prepared_event in prepared {
        let envelope = AuditEnvelope::from_event(&prepared_event.event)?;
        insert_envelope(&transaction, &envelope)?;
    }
    transaction.commit().map_err(sqlite_error)
}

fn insert_envelope(
    transaction: &Transaction<'_>,
    envelope: &AuditEnvelope,
) -> Result<(), AuditAdapterError> {
    let event_index = sqlite_i64(envelope.event_index)?;
    let occurred_at = envelope.occurred_at.map(sqlite_i64).transpose()?;
    transaction
        .execute(
            SQLITE_INSERT_AUDIT_EVENT,
            params![
                event_index,
                envelope.event_id,
                envelope.action_id,
                envelope.plan_hash.as_deref(),
                envelope.kind,
                envelope.correlation_id,
                envelope.causation_id.as_deref(),
                occurred_at,
                envelope.impact_set_hash.as_deref(),
                envelope.drain_fence_scope.as_deref(),
            ],
        )
        .map(|_rows| ())
        .map_err(sqlite_error)
}

fn load_state(connection: &Connection) -> Result<AuditAppendState, AuditAdapterError> {
    let mut statement = connection
        .prepare(SQLITE_SELECT_AUDIT_STATE)
        .map_err(sqlite_error)?;
    let mut rows = statement.query([]).map_err(sqlite_error)?;
    let mut state = AuditAppendState::default();

    while let Some(row) = rows.next().map_err(sqlite_error)? {
        let event_index = sqlite_u64(row.get::<_, i64>(0).map_err(sqlite_error)?)?;
        let event_id = AuditEventId(row.get::<_, String>(1).map_err(sqlite_error)?);
        state.record_loaded(event_id, event_index)?;
    }

    Ok(state)
}

fn sqlite_i64(value: u64) -> Result<i64, AuditAdapterError> {
    i64::try_from(value).map_err(|_error| {
        AuditAdapterError::storage("sqlite", format!("event_index {value} exceeds i64"))
    })
}

fn sqlite_u64(value: i64) -> Result<u64, AuditAdapterError> {
    u64::try_from(value).map_err(|_error| {
        AuditAdapterError::storage("sqlite", format!("negative event_index {value}"))
    })
}

fn sqlite_error(error: rusqlite::Error) -> AuditAdapterError {
    AuditAdapterError::storage("sqlite", error)
}

#[cfg(test)]
mod tests {
    use super::{SqliteAuditLog, SQLITE_CREATE_AUDIT_EVENTS};
    use crate::adapters::audit::AuditAdapterError;
    use causlane_core::{ActionId, AuditEvent, AuditEventId, AuditEventKind, AuditLogPort};

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

    #[test]
    fn creates_schema_for_in_memory_sqlite() -> Result<(), AuditAdapterError> {
        let audit = SqliteAuditLog::open_in_memory()?;
        let table_count: i64 = audit
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'causlane_audit_events'",
                [],
                |row| row.get(0),
            )
            .map_err(super::sqlite_error)?;

        assert_eq!(table_count, 1);
        assert!(SQLITE_CREATE_AUDIT_EVENTS.contains("event_id TEXT NOT NULL UNIQUE"));
        Ok(())
    }

    #[test]
    fn append_persists_envelope_row() -> Result<(), AuditAdapterError> {
        let mut audit = SqliteAuditLog::open_in_memory()?;

        assert_eq!(
            audit.append(event("event-1"))?,
            AuditEventId("event-1".to_owned())
        );

        let row: (i64, String, String, String) = audit
            .connection()
            .query_row(
                "SELECT event_index, event_id, action_id, kind FROM causlane_audit_events",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(super::sqlite_error)?;

        assert_eq!(
            row,
            (
                0,
                "event-1".to_owned(),
                "action-1".to_owned(),
                "execution.started".to_owned()
            )
        );
        Ok(())
    }

    #[test]
    fn duplicate_event_id_fails() -> Result<(), AuditAdapterError> {
        let mut audit = SqliteAuditLog::open_in_memory()?;
        assert_eq!(
            audit.append(event("event-1"))?,
            AuditEventId("event-1".to_owned())
        );

        assert_eq!(
            audit.append(event("event-1")),
            Err(AuditAdapterError::DuplicateEventId {
                event_id: AuditEventId("event-1".to_owned())
            })
        );
        Ok(())
    }

    #[test]
    fn batch_rolls_back_on_duplicate() -> Result<(), AuditAdapterError> {
        let mut audit = SqliteAuditLog::open_in_memory()?;

        assert_eq!(
            audit.append_batch([event("event-1"), event("event-1")]),
            Err(AuditAdapterError::DuplicateEventId {
                event_id: AuditEventId("event-1".to_owned())
            })
        );

        let count: i64 = audit
            .connection()
            .query_row("SELECT COUNT(*) FROM causlane_audit_events", [], |row| {
                row.get(0)
            })
            .map_err(super::sqlite_error)?;
        assert_eq!(count, 0);

        assert_eq!(
            audit.append(event("event-2"))?,
            AuditEventId("event-2".to_owned())
        );
        Ok(())
    }

    #[test]
    fn batch_persists_ordered_rows_transactionally() -> Result<(), AuditAdapterError> {
        let mut audit = SqliteAuditLog::open_in_memory()?;

        let event_ids = AuditLogPort::append_batch(
            &mut audit,
            vec![
                event_kind("barrier", AuditEventKind::ExecutionBarrierLogged),
                event_kind("started", AuditEventKind::ExecutionStarted),
            ],
        )?;
        assert_eq!(
            event_ids,
            vec![
                AuditEventId("barrier".to_owned()),
                AuditEventId("started".to_owned())
            ]
        );

        let mut statement = audit
            .connection()
            .prepare(
                "SELECT event_index, event_id, kind FROM causlane_audit_events ORDER BY event_index",
            )
            .map_err(super::sqlite_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(super::sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(super::sqlite_error)?;

        assert_eq!(
            rows,
            vec![
                (
                    0,
                    "barrier".to_owned(),
                    "execution.barrier_logged".to_owned()
                ),
                (1, "started".to_owned(), "execution.started".to_owned())
            ]
        );
        Ok(())
    }
}
