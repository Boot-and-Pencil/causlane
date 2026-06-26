# Formal Obligation Record template

```yaml
id: OBL-YYYY-NNN
model_id: FM-000
protocol_id: PR-000
invariant_id: I-001
statement: >
  Precise safety property.
authority_surface:
  - compiled_bundle
  - formal_ir
  - replay_trace
lanes:
  replay:
    status: required
    check_ids: []
  alloy:
    status: required
    check_ids: []
  p:
    status: not_applicable
    reason: "explain"
  kani:
    status: required
    check_ids: []
  verus:
    status: non_blocking_or_required
    check_ids: []
  lean4:
    status: planned
    check_ids: []
negative_controls:
  - path: contracts/scenarios/example_invalid.scenario.yaml
    expected_error_code: SomeError
proof_obligations:
  lean4: []
  verus: []
source_paths:
  - crates/causlane-core/src/domain/example.rs
acceptance:
  commands:
    - just formal-verify-all
owner: team/formal
status: declared|generated|tool_passed|covered
```
