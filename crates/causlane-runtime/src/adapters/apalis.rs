//! Apalis adapter for guarded execution jobs.
//!
//! This module is deliberately a service bridge, not a durable payload schema:
//! Apalis supplies the worker/request envelope, while `ExecutorService` remains
//! the only authority boundary for guarded hard effects.

use std::{
    future::{ready, Ready},
    task::{Context, Poll},
};

use apalis::prelude::Request;
use tower::Service;

use crate::guarded_executor::{ExecutionOutcome, ExecutorService, GuardedExecutionJob};

/// Apalis request carrying one owned guarded execution job.
pub type ApalisGuardedExecutionRequest<Ctx = ()> = Request<GuardedExecutionJob, Ctx>;

/// Tower/Apalis service adapter over a guarded executor service.
#[derive(Clone, Debug)]
pub struct ApalisGuardedExecutor<S> {
    service: S,
}

impl<S> ApalisGuardedExecutor<S> {
    /// Wrap a guarded execution service for Apalis workers.
    #[must_use]
    pub fn new(service: S) -> Self {
        Self { service }
    }

    /// Borrow the wrapped service.
    #[must_use]
    pub fn service(&self) -> &S {
        &self.service
    }

    /// Mutably borrow the wrapped service.
    #[must_use]
    pub fn service_mut(&mut self) -> &mut S {
        &mut self.service
    }

    /// Split the wrapper back into the wrapped service.
    #[must_use]
    pub fn into_inner(self) -> S {
        self.service
    }
}

impl<S> ApalisGuardedExecutor<S>
where
    S: ExecutorService,
{
    /// Execute one Apalis request through the guarded execution service.
    ///
    /// # Errors
    /// Returns the wrapped service error when authorization, capability
    /// derivation/admission, or execution fails.
    pub fn call_request<Ctx>(
        &self,
        request: &ApalisGuardedExecutionRequest<Ctx>,
    ) -> Result<ExecutionOutcome, S::Error> {
        self.service.call(request.args.as_request())
    }
}

impl<S, Ctx> Service<ApalisGuardedExecutionRequest<Ctx>> for ApalisGuardedExecutor<S>
where
    S: ExecutorService,
{
    type Response = ExecutionOutcome;
    type Error = S::Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: ApalisGuardedExecutionRequest<Ctx>) -> Self::Future {
        ready(self.call_request(&request))
    }
}

#[cfg(test)]
mod tests {
    use super::{ApalisGuardedExecutionRequest, ApalisGuardedExecutor};
    use crate::{
        guarded_executor::{ExecutionOutcome, GuardedExecutor},
        test_support::MarkerExecutor,
    };
    use tower::Service;

    fn assert_apalis_service<S>(service: S) -> S
    where
        S: Service<ApalisGuardedExecutionRequest, Response = ExecutionOutcome>,
    {
        service
    }

    #[test]
    fn service_type_matches_apalis_request_shape() {
        let service = ApalisGuardedExecutor::new(GuardedExecutor::new(MarkerExecutor));
        let _service = assert_apalis_service(service);
    }
}
