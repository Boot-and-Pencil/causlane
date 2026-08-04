use cli_checker_verification_fixture::select_exact;

#[test]
fn integration_exact_selection_is_preserved() {
    assert_eq!(select_exact(7, &[2, 7, 9]), Some(7));
    assert_eq!(select_exact(8, &[2, 7, 9]), None);
}

#[cfg(feature = "detection-fixture")]
#[test]
fn integration_detection_control_rejects_parent_fallback() {
    assert_eq!(select_exact(7, &[2, 7, 9]), Some(2));
}

