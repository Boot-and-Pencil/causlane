//! Public API regression tests for the facade crate.

use causlane::core::{kernel, ports, prelude as core_prelude, protocol};
use causlane::prelude::{
    claim_modes_conflict, mergeable, ActionCall, AuditLogPort, ClaimMode, KernelContracts,
};

fn assert_admit_type(_: fn(&protocol::ActionCall) -> kernel::DispatchAdmission) {}

#[test]
fn facade_exposes_curated_core_layers() {
    let _ = std::any::type_name::<protocol::ActionCall>();
    let _ = std::any::type_name::<core_prelude::ActionCall>();
    let _ = std::any::type_name::<dyn ports::AuditLogPort<Error = ()>>();

    assert_admit_type(kernel::admit_call);
    let conflict: fn(protocol::ClaimMode, protocol::ClaimMode, bool, bool, bool) -> bool =
        kernel::claim_modes_conflict;

    assert!(conflict(
        protocol::ClaimMode::ExclusiveWrite,
        protocol::ClaimMode::SharedRead,
        true,
        true,
        false
    ));
}

#[test]
fn facade_prelude_keeps_common_imports() {
    let conflict: fn(ClaimMode, ClaimMode, bool, bool, bool) -> bool = claim_modes_conflict;
    let mergeable: fn() -> bool = mergeable;

    let _ = std::any::type_name::<ActionCall>();
    let _ = std::any::type_name::<dyn AuditLogPort<Error = ()>>();
    let _ = std::any::type_name::<KernelContracts>();

    assert!(!mergeable());
    assert!(conflict(
        ClaimMode::ExclusiveWrite,
        ClaimMode::SharedRead,
        true,
        true,
        false
    ));
}
