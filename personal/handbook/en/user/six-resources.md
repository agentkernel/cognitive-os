---
doc_id: user.resources-model
locale: en
kind: concept
audience: [user]
status: partial
generated: false
sources:
  - path: docs/product/personal/cognitive-resource-model.md
  - path: crates/cognitive-store/src/memory_store.rs
  - path: crates/cognitive-store/src/skill_store.rs
  - path: crates/cognitive-kernel/src/tool_registry.rs
    symbols: ["BUILTIN_TOOL_CATALOG"]
  - path: crates/cognitive-store/src/context_store.rs
tests:
  - crates/cognitive-store/tests/p4_t01_memory_store.rs
  - crates/cognitive-store/tests/p4_t04_skill_store.rs
  - crates/cognitive-store/tests/m5_context_store.rs
fingerprint: "sha256:bdc0cf93154976d7a59aaaab07dab4a7ef75256e3c8b97bf436c8583d97d5e5c"
non_claims:
  - Family presence in authority storage does not claim complete user-facing workflows; per-family gaps are listed below and in known-limitations.
---

# The six resource families

Personal governs six separate families. They intentionally do **not** share one
table, one lifecycle, or one state machine. Today every family has a real authority
store and daemon services; user-facing reach varies, so the honest label is
`partial`.

| Family | What it is | Today's user-facing reach |
|---|---|---|
| **Memory** | admitted durable knowledge with scope, purpose, provenance, versions, expiry, forget/tombstone | `remember`/`forget`/explain via daemon routes; full-text search is a rebuildable FTS5 index behind authority filters; no automatic conversation harvesting |
| **Skill** | immutable locally imported package/revision with bindings | import/bind/revoke/explain via daemon routes; scripts never execute by themselves |
| **Tool** | seven static native operations (workspace read/search/write/patch, process check, HTTP fetch, registered check) | catalog, overlay lifecycle, and validators implemented; the projection reports registration, overlay state, and execution readiness separately (assembled families report `execution_ready` when enabled); Agent exposure follows overlay plus readiness; HTTP fetch stays fail-closed until a campaign pins an HTTPS origin; execution requires the governed Effect path (see [Tasks and execution](./tasks-and-execution.md)) |
| **Context** | per-Task authorized input request + resolved view with explicit losses | fully daemon-side: metadata-first filtering, per-body reauthorization, sealed views, digest-bound caches |
| **Task** | raw intent → interpretation → preview → admitted contract | the four admission operations work over HTTP; watch is bounded and snapshot-first |
| **Runtime/Process** | agent package, installation, registration, instance, sidecar session, process attempt | full Pi lifecycle via `admin-cli`; identities never merge |

Cross-cutting objects (budgets, permissions, Model, Artifact, Intent/Effect,
Evidence, Events) appear inside these families rather than as extra ones.

Two rules explain most behavior you will see:

1. **Content never implies permission.** An imported Skill, installed agent,
   discovered Tool, or admitted Memory grants no runtime capability by itself.
2. **Filter before ranking.** Memory and Context candidates pass authorization,
   scope, tombstone, and freshness filters before any ranking sees them; denied
   content cannot even influence ordering.

Deferred by design (Linux 1.0 scope): embeddings/vector/graph Memory, skill
marketplaces, dynamic tool ecosystems (an MCP adapter exists as a post-1.0 fixture
qualification), multi-agent orchestration, and Web UI.
