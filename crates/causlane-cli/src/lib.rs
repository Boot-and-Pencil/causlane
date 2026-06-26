//! Library surface shared by the `causlane` and `causlane-formal` binaries.
//!
//! The scenario → Formal-IR-facts projection lives here so both binaries use the
//! same payload-bound mapping (P0-FM-003) instead of duplicating it.

#![forbid(unsafe_code)]
#![deny(warnings)]

pub mod app;
pub mod cli_shared;
pub mod formal_artifacts;
pub mod scenario_facts;
