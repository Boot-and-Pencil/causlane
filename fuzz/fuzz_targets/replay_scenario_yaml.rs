//! Parse-boundary fuzz target for replay scenario YAML.
//!
//! The target uses the scenario parser and canonical scenario-to-trace lowering
//! path; it does not mirror the replay oracle.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(scenario) = causlane_replay::ReplayScenario::from_yaml_str(text) {
            let trace = scenario.to_trace();
            let _events = trace.to_events();
        }
    }
});
