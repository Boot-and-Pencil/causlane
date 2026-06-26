# Formal Impact Record template

## Change metadata

- Change ID:
- PR/issue:
- Owner:
- Date:
- Impact class: F0/F1/F2/F3/F4/F5

## Touched protocol-critical paths

```text
<list files or glob classes>
```

## Affected invariants

```text
I-001:
I-002:
I-003:
I-004:
I-005:
I-006:
I-007:
I-008:
I-009:
I-010:
new invariant ids:
```

## Affected formal models

```text
FM-000:
FM-001:
...
```

## Affected protocols

```text
PR-000:
PR-001:
...
```

## Contract changes

- Bundle fields added/changed/removed:
- Formal IR fields added/changed/removed:
- Replay trace/scenario fields added/changed/removed:
- Receipt/coverage fields added/changed/removed:

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| | | | |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | | | |
| Alloy | | | |
| P | | | |
| Kani | | | |
| Verus | | | |
| Lean4 | | | |

## Not applicable lanes

For every not-applicable lane, explain why the lane does not model this behavior and which lane does.

## Acceptance commands

```bash
just formal-ready
just formal-verify-all
```

Additional commands:

```bash
<commands>
```

## Exception request

- Exception needed? yes/no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
