# Formal Smoke

This directory contains the canonical non-fake formal/property/fuzz smoke model.

Invariant: `UseAttempted` is accepted only after `GateOpened` has been observed
in the same bounded trace.

The smoke lanes intentionally stay domain-neutral. They prove that the repo has
live toolchains, executable models, positive controls, negative controls, and
machine-readable evidence. Stronger repo-specific formal lanes remain mandatory
where they already exist.
