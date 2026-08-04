use cli_checker_verification_fixture::select_exact;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

fn deterministic_config() -> Config {
    Config {
        cases: 256,
        max_shrink_iters: 1024,
        failure_persistence: None,
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(0x1058),
        ..Config::default()
    }
}

proptest! {
    #![proptest_config(deterministic_config())]
    #[test]
    fn property_exact_selection_never_widens(
        requested in any::<u8>(),
        mut available in proptest::collection::vec(any::<u8>(), 0..32),
    ) {
        available.push(requested);
        prop_assert_eq!(select_exact(requested, &available), Some(requested));
    }
}

#[cfg(feature = "detection-fixture")]
proptest! {
    #![proptest_config(deterministic_config())]
    #[test]
    fn property_detection_control_shrinks_parent_fallback(requested in any::<u8>()) {
        prop_assert!(select_exact(requested, &[requested]).is_none());
    }
}

