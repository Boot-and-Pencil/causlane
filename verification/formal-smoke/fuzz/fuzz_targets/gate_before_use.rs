#![no_main]

use formal_smoke_model::{reference_accepts, reduce, trace_from_bytes, Event};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let (trace, len) = trace_from_bytes(data);
    let bounded = &trace[..len];
    assert_eq!(reduce(bounded).accepted, reference_accepts(bounded));

    let invalid = [Event::UseAttempted];
    assert!(!reduce(&invalid).accepted);
});
