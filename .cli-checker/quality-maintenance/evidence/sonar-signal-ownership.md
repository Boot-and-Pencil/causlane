# Sonar signal ownership

The 2026-08-06 baseline contained 140 open maintainability-only findings and no
Sonar bug or vulnerability in Causlane. The inventory was limited to twelve
Python/Rust rule keys plus code smells in two repository-owned
`AlloyRunner.java` formal launchers.

The Python files are already governed by the blocking typed retirement
inventory and repository conformance checks. Refactoring those temporary
implementations would invest in the superseded implementation language rather
than complete its clean-break replacement. Rust formatting, iterator shape and
complexity remain blocking through Rustfmt, Clippy and repository tests.

The two Java files are launch adapters for the Alloy lanes, not product Java
sources. They are excluded from Sonar and remain executable, blocking inputs to
the formal verification contract.

Only the exact current maintainability rule keys are delegated. Sonar remains
required and blocking for every unlisted rule, bug and vulnerability.
Restoration is to remove each criterion when the corresponding Python surface
is retired or native ownership changes, remove the Java exclusion if those
launchers become product code, and then run the complete repository and Sonar
gates.
