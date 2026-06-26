# causlane-formal

`causlane-formal` contains pure formal-toolchain readiness logic for Causlane.

## Status

This crate is experimental and pre-alpha. It diagnoses formal-tool availability
and readiness facts; it does not download tools or mutate the host environment.

Publication status is tracked in the public repository: package file-list review
is recorded, and upload must follow the staged runbook in
<https://github.com/Boot-and-Pencil/causlane/blob/main/PUBLISHING.md>.

## Role In The Workspace

The crate takes already-gathered environment facts and requirement tokens, then
produces a formal doctor report. Filesystem probing and command execution live
at the CLI boundary.

## Public API Entry Points

- `EnvFacts`
- `Requirement`
- `DoctorReport`
- `report`
- `report_with_context`

## Features

This crate currently has no optional Cargo features.

## Non-goal

This crate does not prove dispatcher invariants. It reports formal-toolchain
readiness facts consumed by CLI/release gates. Proof and model generation and
checking live in `causlane-codegen`, `causlane-cli` and external formal tools.
