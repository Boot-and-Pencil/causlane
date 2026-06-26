//! Partition runtime compatibility exports.

pub use causlane_core::PartitionKey;

/// A message routed to a partition's owning task. Placeholder shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartitionMessage {
    /// Submit a new action into the partition.
    SubmitAction(String),
    /// Notify the partition that an audit event was observed.
    AuditEventObserved(String),
    /// Notify the partition that the constraint snapshot changed.
    ConstraintSnapshotChanged(String),
}
