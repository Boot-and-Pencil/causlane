#![no_main]

use cli_checker_verification_fixture::select_exact;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let requested = data.first().copied().unwrap_or_default();
    assert!(select_exact(requested, &[requested]).is_none());
});

