//! Replay contract surface (§7.4, §7.6).
//!
//! Names the replay oracle (§7.4) and the authorization-evidence verifier
//! (§7.6) as explicit trait contracts over the crate's existing pure logic.
//! [`ReplayContracts`] is the canonical authority: `verify_trace` delegates to
//! [`ReplayTrace::verify_verdict`] and `verify_authz` delegates to the crate's
//! `validate_authz_refs`, so there is a single verification authority.

use causlane_contracts::{CompiledDispatchBundle, CompiledPredicate};
use causlane_core::{AuditEvent, ExecutionBarrier};

use crate::{validate_authz_refs, ReplayTrace, ReplayVerdict};

/// §7.4 — verify a trace against a compiled bundle, returning a serializable
/// [`ReplayVerdict`] suitable for receipts and coverage provenance. The oracle
/// is pure; trace/bundle loading is the CLI's responsibility.
pub trait ReplayOracle {
    /// Verify `trace` against `bundle`.
    fn verify_trace(&self, bundle: &CompiledDispatchBundle, trace: &ReplayTrace) -> ReplayVerdict;
}

/// The outcome of verifying authorization evidence for a barrier (§7.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthzEvidenceVerdict {
    /// Whether the required authorization evidence is satisfied.
    pub allowed: bool,
    /// Stable reason code when not allowed (`None` when allowed).
    pub reason_code: Option<String>,
}

/// §7.6 — verify that a barrier carries the authorization evidence its predicate
/// requires: a non-expired `Allow` for each required stage bound to the
/// barrier's action/plan/predicate. Missing, denied, wrong-binding and expired
/// decisions all fail closed (deny-by-default, ADR-0011).
pub trait AuthzEvidenceVerifier {
    /// Verify authorization evidence for `barrier` under `predicate`, given the
    /// events prior to the barrier.
    fn verify_authz(
        &self,
        prior_events: &[AuditEvent],
        barrier: &ExecutionBarrier,
        predicate: &CompiledPredicate,
    ) -> AuthzEvidenceVerdict;
}

/// The canonical replay contract authority, delegating to the crate's pure
/// verification functions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReplayContracts;

impl ReplayOracle for ReplayContracts {
    fn verify_trace(&self, bundle: &CompiledDispatchBundle, trace: &ReplayTrace) -> ReplayVerdict {
        trace.verify_verdict(bundle)
    }
}

impl AuthzEvidenceVerifier for ReplayContracts {
    fn verify_authz(
        &self,
        prior_events: &[AuditEvent],
        barrier: &ExecutionBarrier,
        predicate: &CompiledPredicate,
    ) -> AuthzEvidenceVerdict {
        match validate_authz_refs(prior_events.iter(), barrier, predicate, None, None) {
            Ok(()) => AuthzEvidenceVerdict {
                allowed: true,
                reason_code: None,
            },
            Err(err) => AuthzEvidenceVerdict {
                allowed: false,
                reason_code: Some(err.code_token().to_owned()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthzEvidenceVerifier, ReplayContracts, ReplayOracle};
    use causlane_contracts::{
        CompiledDispatchBundle, CompiledPredicate, ContractError, RegistryManifest,
    };
    use causlane_core::{
        ActionId, AuditEventId, ExecutionBarrier, ImpactSetHash, PlanHash, PlanHashError,
    };

    use crate::{ReplayError, ReplayTrace};

    const TRACE: &str = include_str!("../fixtures/contracts/examples/release_promote.trace.json");
    const REGISTRY: &str =
        include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");

    /// Typed error union so `?` composes the crate's error types in tests.
    #[derive(Debug)]
    #[allow(dead_code)]
    enum TestError {
        Contract(ContractError),
        Replay(ReplayError),
        PlanHash(PlanHashError),
        Missing(&'static str),
    }

    impl From<ContractError> for TestError {
        fn from(err: ContractError) -> Self {
            TestError::Contract(err)
        }
    }
    impl From<ReplayError> for TestError {
        fn from(err: ReplayError) -> Self {
            TestError::Replay(err)
        }
    }
    impl From<PlanHashError> for TestError {
        fn from(err: PlanHashError) -> Self {
            TestError::PlanHash(err)
        }
    }

    type TestResult = Result<(), TestError>;

    fn demo_bundle() -> Result<CompiledDispatchBundle, ContractError> {
        let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
        CompiledDispatchBundle::compile(&manifest)
    }

    fn authz_required_bundle() -> Result<CompiledDispatchBundle, ContractError> {
        let yaml = REGISTRY.replace(
            "mode: disabled_for_local_dev\n      allowed_in_profiles: [RuntimeExecution]\n      rationale: demo fixture without real PDP",
            "mode: required\n      stages: [execution_barrier_logged]\n      policy_id: demo-policy\n      policy_version: \"1\"",
        );
        let manifest = RegistryManifest::from_yaml_str(&yaml)?;
        CompiledDispatchBundle::compile(&manifest)
    }

    fn first_predicate(bundle: &CompiledDispatchBundle) -> Result<&CompiledPredicate, TestError> {
        bundle
            .body
            .predicates
            .first()
            .ok_or(TestError::Missing("bundle has no predicate"))
    }

    fn empty_barrier() -> Result<ExecutionBarrier, PlanHashError> {
        Ok(ExecutionBarrier {
            barrier_id: AuditEventId("barrier".to_owned()),
            action_id: ActionId("act".to_owned()),
            plan_hash: PlanHash::new(
                "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            )?,
            op_indexes: Vec::new(),
            impact_set_hash: ImpactSetHash(
                "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                    .to_owned(),
            ),
            witnesses: Vec::new(),
            leases: Vec::new(),
            authz_decision_refs: Vec::new(),
            constraint_snapshot_id: None,
        })
    }

    // §7.4: the oracle accepts the bundle-bound example trace and records the
    // bundle hash + checked invariants in the verdict.
    #[test]
    fn replay_oracle_accepts_example_trace() -> TestResult {
        let bundle = demo_bundle()?;
        let trace = ReplayTrace::from_json_str(TRACE)?;
        let verdict = ReplayContracts.verify_trace(&bundle, &trace);
        assert!(verdict.accepted);
        assert_eq!(verdict.stable_error_code, None);
        assert_eq!(verdict.bundle_hash, bundle.bundle_hash.0);
        assert!(verdict.checked_invariants.contains(&"I-001".to_owned()));
        Ok(())
    }

    // §7.6: authz-not-required is allowed; authz-required with no decisions is
    // refused (deny-by-default).
    #[test]
    fn authz_verifier_is_deny_by_default() -> TestResult {
        let verifier = ReplayContracts;
        let barrier = empty_barrier()?;

        let demo = demo_bundle()?;
        let demo_pred = first_predicate(&demo)?;
        assert!(verifier.verify_authz(&[], &barrier, demo_pred).allowed);

        let required = authz_required_bundle()?;
        let required_pred = first_predicate(&required)?;
        let verdict = verifier.verify_authz(&[], &barrier, required_pred);
        assert!(!verdict.allowed);
        assert!(verdict.reason_code.is_some());
        Ok(())
    }
}
