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
  - path: personal/docs/architecture/x-twitter-connector.md
  - path: personal/docs/architecture/routine-trigger-missed-run.md
  - path: personal/docs/architecture/personal-2.0-opc-v9-implementation-mapping.md
  - path: personal/docs/architecture/personal-2.0.0-dev-prep-index.md
  - path: personal/docs/architecture/personal-2.0-opc-visual-ui-spec.md
  - path: personal/docs/architecture/personal-2.0-opc-v9-ui-comparison-checklist.md
  - path: personal/apps/kernel-server/src/personal/mod.rs
  - path: personal/apps/kernel-server/src/personal/resource_manager.rs
  - path: core/crates/cognitive-kernel/src/lib.rs
    symbols: ["KERNEL_PORTS"]
fingerprint: "sha256:d4ae526460a5d34f220e2754ab33194c216e0bfb1e70fbfa29c98fbdad7817da"
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
- DSH is the **hidden hosted** Member execution engine (not a visible
  Installed Agent / native DSH UI / engine store): exact audited artifact,
  isolated child, bounded stdio broker, daemon Provider proxy and
  update/rollback. No native DSH UI/conversation, raw secret, MCP/base tool,
  HMR or home patch.
- Personal owns scoped Conversation archive/index/retrieval, Project Markdown
  Vault integration and semantic Memory admission/correct/forget.
- Routine/Trigger uses daemon-owned no-overlap, queue-latest, missed/coalesced
  facts and risk-based resume. Engine checkpoint is not authority.
- Provider binding resolves global→Project→employee→Task; subscription,
  account, billing/quota, budget and actual usage remain separate.
- UI is Today/Projects/Knowledge with bottom Settings and one persistent
  right conversation. Team and Inbox are not first-level destinations.
  Design-frozen current chrome (2026-08-30) is **CognitiveOS Personal
  2.0.0** (os-personal 2.0.0). Canvas v9 is the frozen design prototype
  (historical filename `personal-20-opc-e2e-optimized-v9`), not the
  product. Product origin is daemon `/ui/`. Dual Track L1 is **Now /
  hypothesis chrome** after `P12-T01`–`T09` closed (merged PR [#302](https://github.com/agentkernel/cognitive-os/pull/302)).
  Architecture chapter bodies no longer present Team/Inbox as 2.0.0 L1
  (`DOC-P12-DEBT`). Authority remains the P11 walking skeleton. NVDA/200%/host-theme remain
  `not-run`. **Phase 13** (`P13-T01`–`T13`, registered in the formal plan on
  2026-09-02) carries the walking skeletons to prototype-grade product and
  design goals: P13-T02 real hosted-DSH Attempt loop (complete stdio broker),
  P13-T03 real hidden-Pi inference, P13-T04 independent verification /
  openable outputs / publication package, P13-T05 runs/routine loop + Today
  overview, P13-T06 group-chat `@manager`/`@member` routing, P13-T07
  Knowledge/Memory surfaces, P13-T08 Settings Model Connections / diagnostics /
  state-lab, P13-T09 lifecycle + restore points, P13-T10 reviewed Skill/MCP
  grants, P13-T11 reflection, P13-T12 visual spec + a11y qualification,
  P13-T13 Windows native host qualification; `P11-T15` N=15 is the exit (not
  release) and its acceptance now requires Phase 13 done + a qualified
  Windows host. One-module PM/UI work starts at
  [`00-maintenance-index.md`](../../../../clients/docs/design/opc-2.0/00-maintenance-index.md).
  That index also catalogues an informative Design-Agent / Owner-journey
  hardness assessment
  ([`13-personal-20-agent-design-difficulty-and-journey-assessment.md`](../../../../clients/docs/design/opc-2.0/13-personal-20-agent-design-difficulty-and-journey-assessment.md);
  hypothesis; not Gate or a P11 claim).
  The Scene → daemon map is
  [`personal-2.0-opc-v9-implementation-mapping.md`](../../../docs/architecture/personal-2.0-opc-v9-implementation-mapping.md)
  (historical path name contains v9; informative; Owner approval ≠ backend
  exists; Project aggregate walking skeleton is Personal-private; Markdown Vault import/index/conflict (`P11-T10`) is on `main` (files are not Project authority); scoped Memory admission/privacy/forget (`P11-T11`) is on `main`; Routine/Trigger walking skeleton (`P11-T08`) is on `main` (reuses daemon `scheduler_entries`; not Inbox L1); Dual Track `/ui/` IA (`P11-T13`) is on `main` (empty/unavailable honesty; not complete IA acceptance); Windows host/tray/background walking skeleton (`P11-T02`) is on `main` (native install/tray/sleep/SecretStore E2E `not-run`); X/Twitter connector walking skeleton (`P11-T14`) is on `main` (live X API E2E `not-run`; not P0 hero). `P12-T01`–`T09` are on `main` (merged PR [#302](https://github.com/agentkernel/cognitive-os/pull/302); Dual Track **Now / hypothesis chrome**; not pixel-replica, not 2.1, not T15).   Development-prep index
  (plan cards, test/env hard gates, window prompts):
  [`personal-2.0.0-dev-prep-index.md`](../../../docs/architecture/personal-2.0.0-dev-prep-index.md);
  its "Phase 13 build order" graph mirrors the formal plan's Phase 13 mermaid
  edge-for-edge (the formal plan is authoritative).
  The `P13-T12/D01` documentation-only visual layer is the
  [Visual UI specification](../../../docs/architecture/personal-2.0-opc-visual-ui-spec.md)
  (Apple-led rules for daemon `/ui/` expressed against the existing
  `clients/pc/web/src/tokens.css` names and the seven-category state system;
  IA unchanged; not a second design system; no CSS edit) and the
  [frozen v9 ↔ `/ui/` comparison checklist](../../../docs/architecture/personal-2.0-opc-v9-ui-comparison-checklist.md)
  (19/19 maintenance-index modules plus the nine × nine, keyboard/focus,
  200%/narrow, host-theme and NVDA cell grids). Every checklist judgement is
  `not-run` until `P13-T12/D02` records it on an exact `/ui/` revision;
  neither document is Gate, release, or T15 evidence.
- Authority object English id is **Employee**. Product chrome may still say
  **Member Runtime** until post-completion architecture/handbook wording
  reconciliation. This handbook page does not rewrite product journeys.
- HITL lives only on the project-center canvas; Today uses a deep link into
  that canvas. Do not treat `#/hitl/:approvalId` as a product first-level or
  default route.
- `state-lab` is Settings → Advanced, hidden by default, not first-level
  navigation, and not a dev-build-only switch.
- Personal Conversation archive: add a **new** Personal-private projection
  version inside `P11-T05`; do not reinterpret
  `conversation-projection/0.1`; do not open a standalone Lane-CTR first.
  Only a required **core public** conversation schema change would be
  Lane-CTR; the rest of T05 stays private. `core/specs` is unchanged here.
- `P11-T02`–`T15` plan cards were **aligned to Personal 2.0.0 chrome on
  2026-08-30** (not Team/Inbox first-level; not member-level budget stop as
  current chrome). Project aggregate walking skeleton is Personal-private;
  Markdown Vault D01 is on `main` (files are not Project authority);
  scoped Memory admission D01 is on `main`;
  Routine/Trigger walking skeleton (`P11-T08`) is on `main`;
  Dual Track `/ui/` IA (`P11-T13`) is on `main`;
  Windows host walking skeleton (`P11-T02`) is on `main` (native E2E `not-run`);
  X/Twitter connector walking skeleton (`P11-T14`) is on `main` (live X `not-run`; not P0 hero). `P12-T01`–`T09` closed (`P12-T01`–`T09` done, merged PR [#302](https://github.com/agentkernel/cognitive-os/pull/302); Dual Track Now / hypothesis chrome; not T15). Prep index:
  [`personal-2.0.0-dev-prep-index.md`](../../../docs/architecture/personal-2.0.0-dev-prep-index.md).

ADR-0058's MCP/private/fail-closed/P5-no-migration boundary remains. Only its
dsh first-conversation-slice role is superseded; `conversation-projection/0.1`
is not reinterpreted. MCP is advanced/deferred from OPC P0.

All missing capabilities remain `Requires-backend`; Windows host/DSH/connector
validation also `Requires-environment`. Native mobile/E2E relay remote is 2.1.
The future fixed denominator is N=15 and proves nothing until run on a
qualified Windows revision. That card is not the prototype-completeness mutex.

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
