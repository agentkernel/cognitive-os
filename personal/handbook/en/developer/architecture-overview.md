---
doc_id: dev.architecture-overview
locale: en
kind: concept
audience: [developer]
status: partial
generated: false
sources:
  - path: personal/docs/architecture/system-architecture.md
  - path: personal/docs/architecture/resource-manager-architecture.md
  - path: personal/docs/product/resource-manager-design.md
  - path: personal/docs/product/personal-2.0-scope.md
  - path: personal/docs/product/account-hub.md
  - path: personal/docs/product/account-hub.zh-CN.md
  - path: personal/docs/product/agent-integration-and-conversations.md
  - path: personal/docs/product/agent-integration-and-conversations.zh-CN.md
  - path: personal/docs/product/mcp-resource-family.md
  - path: personal/docs/product/mcp-resource-family.zh-CN.md
  - path: personal/docs/architecture/web-ui-architecture.md
  - path: personal/docs/architecture/multi-agent-orchestration.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: personal/docs/architecture/project-role-employee.md
  - path: personal/docs/architecture/conversation-memory-vault.md
  - path: personal/docs/architecture/windows-host-background.md
  - path: personal/docs/architecture/routine-trigger-missed-run.md
  - path: personal/apps/kernel-server/src/personal/mod.rs
  - path: personal/apps/kernel-server/src/personal/resource_manager.rs
  - path: core/crates/cognitive-kernel/src/lib.rs
    symbols: ["KERNEL_PORTS"]
fingerprint: "sha256:66a15772056b5c20b1471cee626b3e6125906cd821896289418df9e6616a8291"
non_claims:
  - The target architecture documents intent; this page tracks which pieces exist. Neither is Gate/release evidence.
---

# Architecture overview

## The invariant everything hangs on

> A probabilistic component may produce a candidate or observation. Only the
> deterministic Rust daemon may authorize, apply CAS, advance lifecycle state,
> grant budget or capability, persist and reconcile an Effect, or accept a Task.

Concretely: every authority mutation flows through `cognitive-kernel`'s
`TransitionEngine` ten-step gate into `cognitive-store`'s single-writer SQLite WAL
adapter, checked against embedded, digest-pinned transition tables from
`cognitive-domain` and canonical digests from `cognitive-contracts`.

## Target layers vs current composition

The target design ([`system-architecture.md`](../../../docs/architecture/system-architecture.md))
draws five layers: experience clients → Task/Resource application services → six
domain services → sidecar/scheduler/executor/verifier execution layer → SQLite +
artifact + secret + Linux ports.

What exists today:

- **Experience**: `cognitive` CLI, Pi extension, TypeScript SDK/Shell library —
  all real clients over loopback HTTP with channel-bound bearers. `implemented`.
- **Application services**: `TaskApi` (record/interpret/preview/admit + watch) and
  the private six-family resource projection + Memory/Skill routes, plus the
  management Resource Manager envelope (`list`/`inspect`/`bind`/`unbind`/`enable`/
  `disable`/`revoke`) in `resource_manager.rs`. `implemented` for those operations;
  `control`/`query_intent` remain unexposed. Watch stays on `/resource/v1/watch`.
- **Domain services**: authority stores + kernel services exist for all six
  families (see the per-domain pages). `implemented` at the storage/service level.
- **Execution layer**: every primitive exists (scheduler CAS leases, sealed
  Context, candidate admission, tool executors, verifier seam, recovery), but the
  autonomous loop that connects them is not wired — `partial`; see
  [execution-chain status](execution-chain-status.md).
- **Platform ports**: SQLite WAL (two databases), filesystem artifact CAS, Linux
  Secret Service, systemd user service. `implemented`.

## Personal 2.0 Windows OPC composition — not current implementation

The target dependency direction is Windows UI/Assistant/engines/connectors ->
daemon application ports -> daemon-owned Project/execution/memory/provider
domains. Windows host, DSH, Pi, Vault and connector adapters never own
authority.

- Project owns Charter, Goal, Plan revision, manager Assignment and employee
  identity; Task/Attempt/Effect/verification remain daemon-governed.
- Pi is the hidden candidate-only Personal Assistant engine.
- DSH is the visible preinstalled managed Installed Agent and default employee
  runtime: exact audited artifact, isolated child, bounded stdio broker, daemon
  Provider proxy and update/rollback. No native DSH UI/conversation, raw
  secret, MCP/base tool, HMR or home patch.
- Personal owns scoped Conversation archive/index/retrieval, Project Markdown
  Vault integration and semantic Memory admission/correct/forget.
- Routine/Trigger uses daemon-owned no-overlap, queue-latest, missed/coalesced
  facts and risk-based resume. Engine checkpoint is not authority.
- Provider binding resolves global→Project→employee→Task; subscription,
  account, billing/quota, budget and actual usage remain separate.
- UI is Today/Projects/Team/Knowledge/Inbox with bottom Settings and one
  active Assistant/employee composer.

ADR-0058's MCP/private/fail-closed/P5-no-migration boundary remains. Only its
dsh first-conversation-slice role is superseded; `conversation-projection/0.1`
is not reinterpreted. MCP is advanced/deferred from OPC P0.

All missing capabilities remain `Requires-backend`; Windows host/DSH/connector
validation also `Requires-environment`. Native mobile/E2E relay remote is 2.1.
The future fixed denominator is N=15 and proves nothing until run on a
qualified Windows revision.

## Design decisions that explain surprises

- One canonical service + fixed loopback port 48181 (ADR-0034) — earlier UDS and
  two-unit promotion designs (ADR-0019/0032/0033) survive as text but are
  superseded for the product path.
- Pi is deliberately two roles: shell host (client) and managed agent (governed
  runtime). Identities never merge (ADR-0035).
- Current Linux 1.0/API: six families and no universal `Resource` table
  (ADR-0037). ADR-0057 adopts MCP as the seventh Personal 2.0 family; ADR-0058
  keeps it Personal-private without collapsing family authority; per-Agent
  sidecar remains the integration boundary (ADR-0038).
- MVP-first authorization: owner-local, single-principal, task-scoped; RBAC and
  approval chains are explicitly deferred.
