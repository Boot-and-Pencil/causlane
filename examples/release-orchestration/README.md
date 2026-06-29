# Release Orchestration Example

Runnable M12.2 reference integration slice for CI/CD release orchestration.

This standalone crate models one bounded release graph:

- CI gates run before package review;
- package-list review runs before publish dry-run planning;
- downstream smoke planning waits for dry-run planning;
- forbidden upload tasks are rejected rather than executed.

It intentionally does not publish crates, sign tags, read tokens, mutate
crates.io, or operate as a durable CI worker.

Run it with:

```bash
cargo run --locked --quiet
```
