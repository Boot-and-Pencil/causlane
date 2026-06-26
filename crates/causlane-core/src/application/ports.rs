//! Hexagonal ports.

use crate::domain::{
    ActionCall, ActionId, ActionPlan, AuditEvent, AuditEventId, ConstraintDecision,
    ExecutionCapability, Op, PlanHash, ResourceClaim,
};

/// Compiles an admitted action call into an executable plan.
pub trait PlannerPort {
    /// Error type produced by compilation.
    type Error;

    /// Compile an action call into a plan.
    fn compile(&self, call: &ActionCall) -> Result<ActionPlan, Self::Error>;
}

/// Appends events to the audit/event journal.
pub trait AuditLogPort {
    /// Error type produced by appending.
    type Error;

    /// Append a batch atomically, returning event ids in journal order.
    ///
    /// Implementations must either persist the full batch in order or leave the
    /// journal unchanged.
    fn append_batch(&mut self, events: Vec<AuditEvent>) -> Result<Vec<AuditEventId>, Self::Error>;

    /// Append an event, returning the id it was recorded under.
    fn append(&mut self, event: AuditEvent) -> Result<AuditEventId, Self::Error>;
}

/// Evaluates a plan's resource claims against the constraint plane.
pub trait ConstraintProviderPort {
    /// Error type produced by evaluation.
    type Error;

    /// Evaluate the given claims for an action and plan, returning a decision.
    fn evaluate_claims(
        &self,
        action_id: &ActionId,
        plan_hash: &PlanHash,
        claims: &[ResourceClaim],
    ) -> Result<ConstraintDecision, Self::Error>;
}

/// Executes a single plan operation under a validated execution capability.
pub trait ExecutorPort {
    /// Error type produced by execution.
    type Error;

    /// Execute an op with the given capability, returning produced fact references.
    fn execute(
        &self,
        op: &Op,
        capability: &ExecutionCapability,
    ) -> Result<Vec<String>, Self::Error>;
}

/// Builds projections from committed observed truth.
pub trait ProjectionPort {
    /// Error type produced by projection.
    type Error;

    /// Project from the anchored audit event.
    fn project(&self, anchor: &AuditEventId) -> Result<(), Self::Error>;
}
