# Generated Alloy facts

This directory is for `.als` files produced from compiled dispatch bundles:

```bash
causlane formal generate alloy --bundle target/causlane/release_promote.bundle.json --scenario contracts/scenarios/release_promote_success.scenario.yaml --out verification/formal-full/alloy/generated/release_promote.als --receipt verification/formal-full/receipts/release_promote.codegen.json
```

Generated files must carry `source_bundle_hash`, `formal_ir_hash`,
`scenario_hash`, `target`, `artifact_kind` and `invariant_ids` headers and be
checked with `causlane formal stale-check`. Scenario-bound generated files open
`core/causlane_core` and are run by `just formal-smoke`. Do not hand-edit
generated `.als` files.
