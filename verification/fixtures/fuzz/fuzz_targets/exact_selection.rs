#![no_main]

use cli_checker_verification_fixture::select_exact;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let requested = data.first().copied().unwrap_or_default();
    let mut available = data.get(1..).unwrap_or_default().to_vec();
    available.push(requested);
    assert_eq!(select_exact(requested, &available), Some(requested));
});

