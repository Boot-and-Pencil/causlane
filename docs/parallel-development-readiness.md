# Parallel Development Readiness

Repository: `causlane`
Owner role: `generic-runtime`

This repository participates in the Hopium current integration baseline parallel-development
stabilization model. The model is intentionally interface-first:
`hopium-foundation` and `hopium-contracts` define the shared vocabulary,
and downstream repositories consume those surfaces directly.

cross-version translation layers, local DTO copies, and old/new
adapters are not accepted. The repository must either depend on the
frozen surface directly or keep the concern local and isolated.

Role:

Generic dispatcher kernel/runtime crate, not a Hopium product component.

The active policy is expressed once in
`.devinfra/cli-checker/project-tooling-profile.yaml`. Run the complete
repository gate with `scripts/check-repository.sh`, or inspect its generic
ownership, dependency, clean-break, and assurance-lane decisions through the
corresponding `cli-checker project` commands.
