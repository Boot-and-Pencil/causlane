#![no_main]

use formal_smoke_model::{reference_accepts_bounded, reduce, reduce_bounded, trace_from_bytes, Event};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let (trace, len) = trace_from_bytes(data);
    assert_eq!(reduce_bounded(&trace, len).accepted, reference_accepts_bounded(&trace, len));

    let invalid = [Event::UseAttempted];
    assert!(!reduce(&invalid).accepted);
});
