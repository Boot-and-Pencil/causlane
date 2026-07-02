//! Parse-boundary fuzz target for replay trace JSON.
//!
//! The target delegates to the replay crate's public parser and lowering logic;
//! it does not implement any replay rules itself.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(trace) = causlane_replay::ReplayTrace::from_json_str(text) {
            let _events = trace.to_events();
        }
    }
});
