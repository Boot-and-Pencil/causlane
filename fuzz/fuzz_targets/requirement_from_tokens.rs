//! Minimal smoke fuzz target: parse arbitrary requirement tokens.
//!
//! Purpose is to validate the cargo-fuzz pipeline end to end on the CI machine,
//! not to find bugs yet. Real replay/registry parse-boundary targets now live
//! beside this target; numeric extremes and long-run budgets remain tracked in
//! `docs/release/refactor-before-publication-gate.md` (PUB1: fuzz/property
//! adoption).

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let tokens: Vec<String> = text.split_whitespace().map(str::to_owned).collect();
        // Must never panic on arbitrary token input.
        let _requirement = causlane_formal::Requirement::from_tokens(&tokens);
    }
});
