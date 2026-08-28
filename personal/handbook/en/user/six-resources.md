---
doc_id: user.resources-model
locale: en
kind: concept
audience: [user]
status: partial
generated: false
sources:
  - path: personal/docs/product/cognitive-resource-model.md
  - path: personal/docs/product/mcp-resource-family.md
  - path: personal/docs/product/mcp-resource-family.zh-CN.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
  - path: personal/crates/cognitive-store/src/memory_store.rs
  - path: personal/crates/cognitive-store/src/skill_store.rs
  - path: core/crates/cognitive-kernel/src/tool_registry.rs
    symbols: ["BUILTIN_TOOL_CATALOG"]
  - path: personal/crates/cognitive-store/src/context_store.rs
tests:
  - personal/crates/cognitive-store/tests/p4_t01_memory_store.rs
  - personal/crates/cognitive-store/tests/p4_t04_skill_store.rs
  - personal/crates/cognitive-store/tests/m5_context_store.rs
fingerprint: "sha256:fe8f131e44bd33b0403e80210b060e3e5c865ba4f5c33a3fa2c24e01cf0d8d8a"
non_claims:
  - Family presence in authority storage does not claim complete user-facing workflows; per-family gaps are listed below and in known-limitations.
---

# The current six resource families

Linux 1.0 and the current APIs govern six separate families. They intentionally
do **not** share one table, one lifecycle, or one state machine. Today every
family has a real authority store and daemon services; user-facing reach varies,
so the honest label is `partial`.

| Family | What it is | Today's user-facing reach |
|---|---|---|
| **Memory** | admitted durable knowledge with scope, purpose, provenance, versions, expiry, forget/tombstone | `remember`/`forget`/explain via daemon routes; full-text search is a rebuildable FTS5 index behind authority filters; no automatic conversation harvesting |
| **Skill** | immutable locally imported package/revision with bindings | import/bind/revoke/explain via daemon routes; scripts never execute by themselves |
| **Tool** | seven static native operations (workspace read/search/write/patch, process check, HTTP fetch, registered check) | catalog, overlay lifecycle, and validators implemented; the projection reports registration, overlay state, and execution readiness separately (assembled families report `execution_ready` when enabled); Agent exposure follows overlay plus readiness; HTTP fetch stays fail-closed until a campaign pins an HTTPS origin; execution requires the governed Effect path (see [Tasks and execution](tasks-and-execution.md)) |
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

Deferred from Linux 1.0: embeddings/vector/graph Memory, skill marketplaces,
the Personal MCP family (the delivered post-1.0 MCP Tool transport/dynamic-Tool
MVP is not that family), and general federation.

## Advanced Personal seventh family (`Requires-backend`, deferred)

ADR-0057 retains **MCP** as an advanced seventh Personal family. ADR-0059
defers it from the Windows OPC P0 path; it does not block Project, employee,
Conversation, Routine, Provider, UI, or X acceptance. This does not change the
current six-family Resource Manager API or turn MCP content into native Tools.
ADR-0058 keeps it Personal-private; older clients see six families.

The target family owns distinct server, package, connection, advertised
capability, binding, health, update, and quarantine identities. A **federated
resource** projection keeps source identity, provenance, revision/freshness,
trust, availability, and allowed actions without copying external authority
into Personal. Origin owns native content; Personal owns admitted bindings,
permissions, synchronization intent, and authority records. Conflicts fail
closed and require an exact daemon preview before write-back.

Advertised MCP items still enter the existing families through their own
admission paths: tools are version-bound Tool candidates, protocol resources
are Context candidates, and prompts/reusable instructions are Skill
candidates. Discovery alone grants no read or dispatch permission. Daemon
policy must still authorize use; mutations still require
persist-before-dispatch Intent/Effect, fencing, budget, and independent
verification.

Unified Activity keeps these labels distinct:

- `Native`: originated in a native Agent application or session;
- `Observed`: seen by an adapter or daemon but not admitted as authority;
- `Governed`: daemon admission, authorization, mutation, and Effect reconciliation;
- `Verified`: current independent verification and daemon acceptance only.

They are not an automatic maturity ladder and `Verified` is not a release or
qualification claim. MCP/federation remains `Requires-backend` for a future
advanced successor. DSH native MCP/base tools stay disabled; connection alone
grants no Tool, Context, workspace, model, or host-session authority.
