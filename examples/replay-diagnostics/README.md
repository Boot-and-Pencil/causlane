# Replay Diagnostics Example

This example exercises the public replay diagnostics path used by downstream
tooling:

```text
RegistryManifest -> CompiledDispatchBundle
ReplayScenario -> ReplayTrace -> verify_explain
ReplayExplain -> human and JSON diagnostics
```

It verifies one accepted release-promotion trace plus rejected traces that carry
different causal locations: action, witness requirement and contended lease
scope. It also checks a structural bundle-hash mismatch, which has a stable
error code but no protocol invariant location.
