//! Formal discipline checker binary.

#![forbid(unsafe_code)]
#![deny(warnings)]

use std::process::ExitCode;

mod formal_discipline;

fn main() -> ExitCode {
    formal_discipline::run_cli(&std::env::args().collect::<Vec<String>>())
}
