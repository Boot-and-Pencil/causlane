//! Restate adapter for guarded execution jobs.
//!
//! This module is a handler/workflow bridge, not a canonical durable payload
//! schema. Restate supplies the durable handler envelope and journals the
//! handler result; a host-owned decoder turns opaque bytes into the existing
//! `GuardedExecutionJob`, and `ExecutorService` remains the only authority
//! boundary for guarded hard effects.

use std::fmt;

use restate_sdk::{
    context::{ContextSideEffects, RunFuture},
    errors::{HandlerResult, TerminalError},
    serde::Json,
};

use crate::guarded_executor::{ExecutionOutcome, ExecutorService, GuardedExecutionJob};

/// Restate-serializable envelope for host-owned guarded job payload bytes.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestateGuardedExecutionPayload {
    /// Opaque host payload; Causlane does not define its durable wire schema here.
    pub bytes: Vec<u8>,
}

impl RestateGuardedExecutionPayload {
    /// Create an opaque Restate payload envelope.
    #[must_use]
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }
}

/// Restate-serializable guarded execution outcome.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestateExecutionOutcome {
    /// Fact/object references produced by the executed op.
    pub produced_refs: Vec<String>,
}

impl From<ExecutionOutcome> for RestateExecutionOutcome {
    fn from(outcome: ExecutionOutcome) -> Self {
        Self {
            produced_refs: outcome.produced_refs,
        }
    }
}

/// Restate JSON input wrapper for guarded execution handlers.
pub type RestateGuardedExecutionInput = Json<RestateGuardedExecutionPayload>;

/// Restate JSON output wrapper for guarded execution handlers.
pub type RestateGuardedExecutionOutput = Json<RestateExecutionOutcome>;

/// Host-owned decoder from opaque Restate payload bytes to a guarded job.
pub trait RestateGuardedJobDecoder {
    /// Error returned when the opaque payload cannot be decoded.
    type Error;

    /// Decode opaque bytes into the existing owned guarded execution job.
    fn decode(
        &self,
        payload: &RestateGuardedExecutionPayload,
    ) -> Result<GuardedExecutionJob, Self::Error>;
}

/// Error returned by the Restate guarded execution bridge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestateAdapterError<DecodeError, ServiceError> {
    /// The host-owned payload decoder rejected the request.
    Decode(DecodeError),
    /// The guarded executor service rejected or failed the decoded job.
    Execute(ServiceError),
}

/// Maps adapter failures to Restate terminal errors.
pub trait RestateErrorMapper<ServiceError, DecodeError> {
    /// Convert a guarded execution bridge failure into a Restate terminal error.
    fn terminal_error(
        &self,
        error: &RestateAdapterError<DecodeError, ServiceError>,
    ) -> TerminalError;
}

/// Default terminal mapper: all adapter failures fail closed.
#[derive(Clone, Copy, Debug, Default)]
pub struct TerminalRestateErrors;

impl<ServiceError, DecodeError> RestateErrorMapper<ServiceError, DecodeError>
    for TerminalRestateErrors
where
    ServiceError: fmt::Debug,
    DecodeError: fmt::Debug,
{
    fn terminal_error(
        &self,
        error: &RestateAdapterError<DecodeError, ServiceError>,
    ) -> TerminalError {
        match error {
            RestateAdapterError::Decode(inner) => TerminalError::new_with_code(
                400,
                format!("causlane.restate.decode_failed: {inner:?}"),
            ),
            RestateAdapterError::Execute(inner) => TerminalError::new_with_code(
                500,
                format!("causlane.restate.execution_failed: {inner:?}"),
            ),
        }
    }
}

/// Restate handler bridge over a guarded executor service.
#[derive(Clone, Debug)]
pub struct RestateGuardedExecutor<S, D, M = TerminalRestateErrors> {
    service: S,
    decoder: D,
    error_mapper: M,
}

impl<S, D> RestateGuardedExecutor<S, D, TerminalRestateErrors> {
    /// Wrap a guarded execution service and host payload decoder.
    #[must_use]
    pub fn new(service: S, decoder: D) -> Self {
        Self::with_error_mapper(service, decoder, TerminalRestateErrors)
    }
}

impl<S, D, M> RestateGuardedExecutor<S, D, M> {
    /// Wrap a guarded execution service, host decoder and error mapper.
    #[must_use]
    pub fn with_error_mapper(service: S, decoder: D, error_mapper: M) -> Self {
        Self {
            service,
            decoder,
            error_mapper,
        }
    }

    /// Borrow the wrapped service.
    #[must_use]
    pub fn service(&self) -> &S {
        &self.service
    }

    /// Borrow the host-owned payload decoder.
    #[must_use]
    pub fn decoder(&self) -> &D {
        &self.decoder
    }

    /// Borrow the Restate error mapper.
    #[must_use]
    pub fn error_mapper(&self) -> &M {
        &self.error_mapper
    }

    /// Split the wrapper back into its parts.
    #[must_use]
    pub fn into_inner(self) -> (S, D, M) {
        (self.service, self.decoder, self.error_mapper)
    }
}

impl<S, D, M> RestateGuardedExecutor<S, D, M>
where
    S: ExecutorService,
    D: RestateGuardedJobDecoder,
{
    /// Execute one decoded Restate payload through the guarded execution service.
    ///
    /// # Errors
    /// Returns decode failures from the host-owned decoder or service failures
    /// from authorization, capability derivation/admission, or execution.
    #[must_use = "adapter failures are fail-closed and must be handled"]
    pub fn call_payload(
        &self,
        payload: &RestateGuardedExecutionPayload,
    ) -> Result<RestateExecutionOutcome, RestateAdapterError<D::Error, S::Error>> {
        let job = self
            .decoder
            .decode(payload)
            .map_err(RestateAdapterError::Decode)?;
        self.service
            .call(job.as_request())
            .map(Into::into)
            .map_err(RestateAdapterError::Execute)
    }
}

impl<S, D, M> RestateGuardedExecutor<S, D, M> {
    /// Run one payload inside a Restate journaled action.
    ///
    /// # Errors
    /// Returns a Restate handler error when the journaled action completes with a
    /// terminal adapter failure.
    pub async fn handle_payload<'ctx, Ctx>(
        &'ctx self,
        ctx: Ctx,
        payload: RestateGuardedExecutionInput,
    ) -> HandlerResult<RestateGuardedExecutionOutput>
    where
        Ctx: ContextSideEffects<'ctx> + 'ctx,
        S: ExecutorService + Sync + 'ctx,
        D: RestateGuardedJobDecoder + Sync + 'ctx,
        M: RestateErrorMapper<S::Error, D::Error> + Sync + 'ctx,
    {
        let payload = payload.into_inner();
        ctx.run(move || async move {
            self.call_payload(&payload)
                .map(Json::from)
                .map_err(|error| self.error_mapper.terminal_error(&error).into())
        })
        .name("causlane.guarded_executor")
        .await
        .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RestateAdapterError, RestateErrorMapper, RestateExecutionOutcome,
        RestateGuardedExecutionPayload, RestateGuardedExecutor, RestateGuardedJobDecoder,
        TerminalRestateErrors,
    };
    use crate::{
        guarded_executor::{GuardedExecutionJob, GuardedExecutor, SpendError},
        test_support::CountingExecutor,
    };

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum FixtureDecodeError {
        UnknownPayload,
    }

    #[derive(Clone, Copy, Debug)]
    struct RejectingDecoder;

    impl RestateGuardedJobDecoder for RejectingDecoder {
        type Error = FixtureDecodeError;

        fn decode(
            &self,
            _payload: &RestateGuardedExecutionPayload,
        ) -> Result<GuardedExecutionJob, Self::Error> {
            Err(FixtureDecodeError::UnknownPayload)
        }
    }

    #[test]
    fn decoder_error_never_reaches_executor() {
        let executor = CountingExecutor::default();
        let service =
            RestateGuardedExecutor::new(GuardedExecutor::new(executor.clone()), RejectingDecoder);
        let payload = RestateGuardedExecutionPayload::new(b"unknown".to_vec());

        let result = service.call_payload(&payload);

        assert_eq!(
            result,
            Err(RestateAdapterError::Decode(
                FixtureDecodeError::UnknownPayload
            ))
        );
        assert_eq!(executor.calls(), 0);
    }

    #[test]
    fn outcome_conversion_preserves_produced_refs() {
        let outcome = RestateExecutionOutcome::from(crate::guarded_executor::ExecutionOutcome {
            produced_refs: vec!["ref-a".to_owned(), "ref-b".to_owned()],
        });

        assert_eq!(
            outcome,
            RestateExecutionOutcome {
                produced_refs: vec!["ref-a".to_owned(), "ref-b".to_owned()]
            }
        );
    }

    #[test]
    fn default_error_mapper_marks_decode_failures_terminal() {
        let error =
            RestateAdapterError::<FixtureDecodeError, SpendError<core::convert::Infallible>>::Decode(
                FixtureDecodeError::UnknownPayload,
            );

        let terminal = TerminalRestateErrors.terminal_error(&error);

        assert_eq!(terminal.code(), 400);
        assert!(terminal
            .message()
            .contains("causlane.restate.decode_failed"));
    }
}
