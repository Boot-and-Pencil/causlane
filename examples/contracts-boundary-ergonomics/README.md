# Contracts Boundary Ergonomics Example

This example exercises the public contracts boundary a downstream planner or CI
gate would use before replay/runtime:

```text
RegistryManifest -> BoundaryContracts -> CompiledDispatchBundle
CompiledDispatchBundle -> JSON artifact -> verified reload
PlanHashMaterial -> plan hash
CanonicalImpact list -> impact-set hash
TemplateBindings -> exact template resolution
```

It includes negative controls for stale bundle artifacts, unresolved template
bindings and mutated plan material.
