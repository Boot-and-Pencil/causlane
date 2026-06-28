# Reference Integration Example

Runnable M12.1 reference integration slice for the public alpha API story.

This standalone crate demonstrates one in-process service shape:

- an API surface submits host tasks through the host-dispatch v2 seam;
- a worker drains them through the deterministic linear runtime adapter;
- audit events are appended through the runtime audit adapter;
- a projection read is authorized and redacted through the guarded projection helper.

It is intentionally not a production service claim: there is no durable queue,
distributed lease coordination, HTTP server, database or retry policy here.

Run it with:

```bash
cargo run --locked --quiet
```
