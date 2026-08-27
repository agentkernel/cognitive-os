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
  - path: personal/apps/kernel-server/src/personal/mod.rs
  - path: personal/apps/kernel-server/src/personal/resource_manager.rs
  - path: core/crates/cognitive-kernel/src/lib.rs
    symbols: ["KERNEL_PORTS"]
fingerprint: "sha256:0eb16b585d7bb33d1d419b6b0d7a8d11f1be0e23e390f6b2cd1ae772844c1fc0"
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

## Full Personal 2.0 composition — not current implementation

Personal 2.0 preserves the invariant above while committing to these product
boundaries:

- Windows, macOS, and Linux are independently qualified local product paths;
  no platform or Agent evidence transfers;
- the exact initial Agent set is Pi, DeepSeek Harness Developer Preview, and
  supported-platform Codex desktop. CLI, Provider, model, account, adapter, or
  bridge evidence does not qualify another product;
- Account Hub credential import is a daemon-owned source-to-SecretStore
  operation under ADR-0055. The UI supplies exact source selection and consent,
  but never reads or receives imported material;
- MCP becomes a seventh user-visible family with federated source identity,
  trust, availability, and policy. The current Resource Manager and authority
  services stay six-family on the 1.0 projection; ADR-0058 keeps MCP on a
  separate Personal-private envelope until `P10-T03` implements that envelope;
- vendor-specific conversation adapters preserve each Agent's protocol and
  identity. Pi remains the only qualified Agent; dsh implementation evidence
  and generic adapter contracts do not transfer qualification;
- embedded native conversations enter governed work only by explicit admission;
- Goal -> immutable Plan revision -> Task -> preserved Attempt composes
  governed work, while daemon-owned multi-Agent supervision assigns, fences,
  budgets, reconciles, and verifies;
- unified Activity keeps Native, Observed, Governed, and Verified provenance
  separate with declared coverage.

These missing capabilities remain `Requires-backend`. Public authority or
contract additions still need a later Lane-CTR decision; ADR-0058 already
kept MCP family and conversation projection Personal-private. The full-version
commitment and
fixed 8/8 AI-window simulated acceptance do not establish implementation,
human usability, release, or Gate evidence.

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
