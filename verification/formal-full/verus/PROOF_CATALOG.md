# Verus proof catalog

This file is the Verus-side checklist for authoritative proof profiles.

## Required proof groups

| ID | Proof group | Invariants | Required profile |
|---|---|---|---|
| V-001 | lifecycle reducer preservation | I-001, I-002, I-003, I-008 | proof/all |
| V-002 | replay accepted bounded trace soundness | I-001, I-002, I-003, I-008, I-009 | proof/all |
| V-003 | capability derivation and validation | I-001, I-009 | proof/all |
| V-004 | witness selector exactness | I-009 | proof/all |
| V-005 | projection anchor exactness | I-003, I-009 | proof/all |
| V-006 | authz resolver safety | I-009/authz | proof/all when modeled |
| V-007 | lease conflict and coverage safety | I-006 | proof/all |
| V-008 | drain safety | I-007 | proof/all |
| V-009 | overlay monotonicity | I-004 | proof/all |
| V-010 | route/profile compatibility | I-005 | proof/all |
| V-011 | constraint update preserves committed truth | I-010 | proof/all |
| V-012 | canonical serialization helper invariants | FM-001 | proof/all when modeled |
| V-013 | receipt/coverage non-upgrade | FM-015 | proof/all when modeled |

## No-cheating policy

Authoritative Verus runs must use:

```bash
verus --no-cheating <generated-or-authoritative-file.rs>
```

Forbidden in authoritative proof files unless machine-approved exception exists:

```text
assume
admit
external_body
unimplemented!
panic-based proof shortcuts
proof over frozen flags not advanced by transition
```

## Receipt requirements

Every Verus proof run counted by coverage must have a tool-run receipt containing:

```text
target = verus
tool = verus
tool_version
command
actual_result
exit_code
source_bundle_hash
formal_ir_hash
scenario_hash if scenario-bound
generated_artifact_hash
invariant_ids
check_ids
```
