//! Shared adapter certification tests.

use core::convert::Infallible;

use causlane_core::{CapabilitySpendRefusal, PlanHashError, Timestamp};

use crate::{
    guarded_executor::{ExecutionOutcome, GuardedExecutionJob, SpendError},
    test_support::{
        allow, barrier, barrier_lease_expiring, job, plan, CountingExecutor, RecordingExecutor,
    },
};

#[derive(Debug, PartialEq, Eq)]
enum CertifiedAdapterError {
    Unauthorized,
    Capability(String),
    CapabilityRefused(CapabilitySpendRefusal),
}

type CertifiedResult = Result<ExecutionOutcome, CertifiedAdapterError>;

fn authorized_job() -> Result<GuardedExecutionJob, PlanHashError> {
    let plan = plan()?;
    Ok(job(barrier(plan.clone()), vec![allow(plan)], Timestamp(10)))
}

fn missing_authz_job() -> Result<GuardedExecutionJob, PlanHashError> {
    Ok(job(barrier(plan()?), Vec::new(), Timestamp(10)))
}

fn expired_capability_job() -> Result<GuardedExecutionJob, PlanHashError> {
    let plan = plan()?;
    Ok(job(
        barrier_lease_expiring(plan.clone(), Timestamp(10)),
        vec![allow(plan)],
        Timestamp(10),
    ))
}

fn normalize_spend_error(error: SpendError<Infallible>) -> CertifiedAdapterError {
    match error {
        SpendError::Unauthorized(_error) => CertifiedAdapterError::Unauthorized,
        SpendError::Capability(error) => CertifiedAdapterError::Capability(format!("{error:?}")),
        SpendError::CapabilityRefused(error) => CertifiedAdapterError::CapabilityRefused(error),
        SpendError::Execute(error) => match error {},
    }
}

fn assert_authorized_reaches_executor_once(executor: &RecordingExecutor, result: &CertifiedResult) {
    assert_eq!(
        result,
        &Ok(ExecutionOutcome {
            produced_refs: vec!["executed:0:0".to_owned()]
        })
    );
    assert_eq!(executor.calls(), 1);
}

fn assert_missing_authz_refused_before_executor_entry(
    executor: &CountingExecutor,
    result: &CertifiedResult,
) {
    assert_eq!(result, &Err(CertifiedAdapterError::Unauthorized));
    assert_eq!(executor.calls(), 0);
}

fn assert_expired_capability_refused_before_executor_entry(
    executor: &CountingExecutor,
    result: &CertifiedResult,
) {
    assert_eq!(
        result,
        &Err(CertifiedAdapterError::CapabilityRefused(
            CapabilitySpendRefusal::Expired {
                expires_at: Timestamp(10),
                now: Timestamp(10),
            }
        ))
    );
    assert_eq!(executor.calls(), 0);
}

fn assert_metadata_not_authority(executor: &CountingExecutor, result: &CertifiedResult) {
    assert_eq!(result, &Err(CertifiedAdapterError::Unauthorized));
    assert_eq!(executor.calls(), 0);
}

#[cfg(feature = "apalis")]
mod apalis {
    use super::{
        assert_authorized_reaches_executor_once,
        assert_expired_capability_refused_before_executor_entry, assert_metadata_not_authority,
        assert_missing_authz_refused_before_executor_entry, authorized_job, expired_capability_job,
        missing_authz_job, normalize_spend_error, CertifiedResult,
    };
    use crate::{
        adapters::apalis::{ApalisGuardedExecutionRequest, ApalisGuardedExecutor},
        guarded_executor::GuardedExecutor,
        test_support::{allow, plan, CountingExecutor, RecordingExecutor},
    };
    use causlane_core::PlanHashError;

    fn execute_with_recording<Ctx>(
        executor: RecordingExecutor,
        request: &ApalisGuardedExecutionRequest<Ctx>,
    ) -> CertifiedResult {
        let service = ApalisGuardedExecutor::new(GuardedExecutor::new(executor));
        service.call_request(request).map_err(normalize_spend_error)
    }

    fn execute_with_counting<Ctx>(
        executor: CountingExecutor,
        request: &ApalisGuardedExecutionRequest<Ctx>,
    ) -> CertifiedResult {
        let service = ApalisGuardedExecutor::new(GuardedExecutor::new(executor));
        service.call_request(request).map_err(normalize_spend_error)
    }

    #[test]
    fn apalis_certified_authorized_job_reaches_executor_once() -> Result<(), PlanHashError> {
        let executor = RecordingExecutor::default();
        let request: ApalisGuardedExecutionRequest =
            ApalisGuardedExecutionRequest::new(authorized_job()?);

        let result = execute_with_recording(executor.clone(), &request);

        assert_authorized_reaches_executor_once(&executor, &result);
        Ok(())
    }

    #[test]
    fn apalis_certified_missing_authz_never_reaches_executor() -> Result<(), PlanHashError> {
        let executor = CountingExecutor::default();
        let request: ApalisGuardedExecutionRequest =
            ApalisGuardedExecutionRequest::new(missing_authz_job()?);

        let result = execute_with_counting(executor.clone(), &request);

        assert_missing_authz_refused_before_executor_entry(&executor, &result);
        Ok(())
    }

    #[test]
    fn apalis_certified_expired_capability_never_reaches_executor() -> Result<(), PlanHashError> {
        let executor = CountingExecutor::default();
        let request: ApalisGuardedExecutionRequest =
            ApalisGuardedExecutionRequest::new(expired_capability_job()?);

        let result = execute_with_counting(executor.clone(), &request);

        assert_expired_capability_refused_before_executor_entry(&executor, &result);
        Ok(())
    }

    #[test]
    fn apalis_certified_request_metadata_is_not_authority() -> Result<(), PlanHashError> {
        let executor = CountingExecutor::default();
        let mut request = ApalisGuardedExecutionRequest::new_with_ctx(
            missing_authz_job()?,
            "worker-metadata-authz-allow",
        );
        request.insert(vec![allow(plan()?)]); // ignored: semantic evidence lives in the job

        let result = execute_with_counting(executor.clone(), &request);

        assert_metadata_not_authority(&executor, &result);
        Ok(())
    }
}

#[cfg(feature = "restate")]
mod restate {
    use super::{
        assert_authorized_reaches_executor_once,
        assert_expired_capability_refused_before_executor_entry, assert_metadata_not_authority,
        assert_missing_authz_refused_before_executor_entry, authorized_job, expired_capability_job,
        missing_authz_job, normalize_spend_error, CertifiedAdapterError, CertifiedResult,
    };
    use crate::{
        adapters::restate::{
            RestateAdapterError, RestateGuardedExecutionPayload, RestateGuardedExecutor,
            RestateGuardedJobDecoder,
        },
        guarded_executor::{ExecutionOutcome, GuardedExecutionJob, GuardedExecutor, SpendError},
        test_support::{CountingExecutor, RecordingExecutor},
    };
    use causlane_core::PlanHashError;

    #[derive(Clone)]
    struct FixedDecoder {
        job: GuardedExecutionJob,
    }

    impl RestateGuardedJobDecoder for FixedDecoder {
        type Error = core::convert::Infallible;

        fn decode(
            &self,
            _payload: &RestateGuardedExecutionPayload,
        ) -> Result<GuardedExecutionJob, Self::Error> {
            Ok(self.job.clone())
        }
    }

    fn normalize_restate_error(
        error: RestateAdapterError<
            core::convert::Infallible,
            SpendError<core::convert::Infallible>,
        >,
    ) -> CertifiedAdapterError {
        match error {
            RestateAdapterError::Decode(error) => match error {},
            RestateAdapterError::Execute(error) => normalize_spend_error(error),
        }
    }

    fn execute_with_recording(
        executor: RecordingExecutor,
        job: GuardedExecutionJob,
        payload: &RestateGuardedExecutionPayload,
    ) -> CertifiedResult {
        let service =
            RestateGuardedExecutor::new(GuardedExecutor::new(executor), FixedDecoder { job });
        service
            .call_payload(payload)
            .map(|outcome| ExecutionOutcome {
                produced_refs: outcome.produced_refs,
            })
            .map_err(normalize_restate_error)
    }

    fn execute_with_counting(
        executor: CountingExecutor,
        job: GuardedExecutionJob,
        payload: &RestateGuardedExecutionPayload,
    ) -> CertifiedResult {
        let service =
            RestateGuardedExecutor::new(GuardedExecutor::new(executor), FixedDecoder { job });
        service
            .call_payload(payload)
            .map(|outcome| ExecutionOutcome {
                produced_refs: outcome.produced_refs,
            })
            .map_err(normalize_restate_error)
    }

    #[test]
    fn restate_certified_authorized_job_reaches_executor_once() -> Result<(), PlanHashError> {
        let executor = RecordingExecutor::default();
        let payload = RestateGuardedExecutionPayload::new(b"authorized".to_vec());

        let result = execute_with_recording(executor.clone(), authorized_job()?, &payload);

        assert_authorized_reaches_executor_once(&executor, &result);
        Ok(())
    }

    #[test]
    fn restate_certified_missing_authz_never_reaches_executor() -> Result<(), PlanHashError> {
        let executor = CountingExecutor::default();
        let payload = RestateGuardedExecutionPayload::new(b"missing-authz".to_vec());

        let result = execute_with_counting(executor.clone(), missing_authz_job()?, &payload);

        assert_missing_authz_refused_before_executor_entry(&executor, &result);
        Ok(())
    }

    #[test]
    fn restate_certified_expired_capability_never_reaches_executor() -> Result<(), PlanHashError> {
        let executor = CountingExecutor::default();
        let payload = RestateGuardedExecutionPayload::new(b"expired".to_vec());

        let result = execute_with_counting(executor.clone(), expired_capability_job()?, &payload);

        assert_expired_capability_refused_before_executor_entry(&executor, &result);
        Ok(())
    }

    #[test]
    fn restate_certified_payload_bytes_are_not_authority() -> Result<(), PlanHashError> {
        let executor = CountingExecutor::default();
        let payload = RestateGuardedExecutionPayload::new(b"metadata:authz=allow".to_vec());

        let result = execute_with_counting(executor.clone(), missing_authz_job()?, &payload);

        assert_metadata_not_authority(&executor, &result);
        Ok(())
    }
}
