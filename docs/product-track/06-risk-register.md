# 06. Risk register

| Risk | Why it matters | Mitigation |
|---|---|---|
| Formal model as second truth | Proofs may verify a hand-maintained fantasy, not runtime contracts | Generate artifacts from bundle/formal IR; stale-check; receipt-bound coverage |
| Overbuilding workflow engine | Causlane competes with mature runtimes and loses focus | Keep adapters outside core; document non-goals; adapter certification |
| Hot path overhead | Users reject system if every request pays full formal/control cost | Compiled bundle, partitions, indexes, bounded queues, batching, lazy explain |
| Authz bypass | Endpoint/job middleware may bypass semantic action policy | Authz stages in dispatch; default deny; scoped capabilities |
| Projection as truth | UI/status diverges from observed reality | Typed TruthAnchor; replay/formal checks |
| Merge protocol ambiguity | Parallel writes corrupt state | Default no merge; verified explicit MergeProtocolSpec only |
| Toolchain friction | Formal stack too hard to reproduce | formal-doctor/install, pinned versions, receipts, profiles |
| Proof theatre | Docs overclaim formal coverage | Coverage generated from receipts; exceptions policy; anti-overclaim checks |
| Stringly protocol fields | Invalid IDs/scopes/hash slip through | Validated newtypes, canonical serialization, schema validation |
| Adapter bypass | Adapter executes without barrier/capability | Adapter certification; guarded executor APIs |
| Documentation drift | Docs become stale as codegen changes | Machine-derived status docs; docs gate |
| Scope creep | Causlane tries to solve policy/db/observability/jobs | Non-goals, modular feature flags, external adapters |
