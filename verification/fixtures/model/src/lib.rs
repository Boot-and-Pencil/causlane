#![forbid(unsafe_code)]
#![deny(warnings)]

pub fn select_exact(requested: u8, available: &[u8]) -> Option<u8> {
    available.iter().copied().find(|candidate| *candidate == requested)
}

pub fn unobserved_parent_fallback(requested: u8) -> bool {
    requested == 0
}

#[cfg(test)]
mod tests {
    use super::select_exact;

    #[test]
    fn unit_exact_selection() {
        assert_eq!(select_exact(3, &[1, 3, 5]), Some(3));
        assert_eq!(select_exact(4, &[1, 3, 5]), None);
    }

    #[cfg(feature = "detection-fixture")]
    #[test]
    fn unit_detection_control_rejects_parent_fallback() {
        assert_eq!(select_exact(3, &[1, 3, 5]), None);
    }
}

#[cfg(kani)]
#[kani::proof]
fn exact_selection_kani() {
    let requested: u8 = kani::any();
    let other: u8 = kani::any();
    kani::assume(other != requested);
    assert_eq!(select_exact(requested, &[other, requested]), Some(requested));
}

#[cfg(kani)]
#[kani::proof]
fn exact_selection_kani_detection() {
    let requested: u8 = kani::any();
    assert!(select_exact(requested, &[requested]).is_none());
}

