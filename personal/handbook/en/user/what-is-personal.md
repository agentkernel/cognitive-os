---
doc_id: user.what-is-personal
locale: en
kind: overview
audience: [user]
status: partial
generated: false
sources:
  - path: personal/docs/product/product-design.md
  - path: personal/docs/product/personal-2.0-scope.md
  - path: personal/apps/kernel-server/src/personal/server.rs
    symbols: ["serve_personal_loopback"]
  - path: personal/docs/product/linux-1.0-scope.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: personal/docs/product/opc-product-model.md
tests:
  - personal/apps/kernel-server/tests/p1_t04_personal_daemon.rs
fingerprint: "sha256:922d6c9e48d9d94a7231181e5fc75b21e850d8122a430d072a878712bc7108f9"
non_claims:
  - No Gate, release, Profile, Windows-parity, or agent-benefit claim; the Linux 1.0 target composition is owned by the formal plan.
---

# What Personal is (and is not)

## What it is

A local, single-owner system for auditable, budgeted, recoverable Agent work.
Its current finalized release boundary is Linux 1.0. Its adopted Personal 2.0
target is a **Windows-first operating console for one-person companies and
individual developers**: the Owner runs governed Projects and long-lived
digital employees in business language while the host is online.
A local daemon plus deterministic clients make Agent work **auditable,
budgeted, recoverable, and never falsely completed**:

- One Rust daemon (`kernel-server --personal`) binds loopback only and is the sole
  writer of authority state (SQLite WAL databases under your XDG directories).
- Everything else — the `cognitive` CLI, the Pi conversation shell, SDKs, sidecars —
  is a client. Clients propose; the daemon authorizes, persists, schedules,
  reconciles, and accepts.
- Six user-visible resource families are governed separately: Memory, Skill, Tool,
  Context, Task, and Runtime/Process. Budgets, permissions, artifacts,
  Intent/Effect, evidence, and events cut across them.
- Your Provider API key lives only in an approved secret store (Linux Secret
  Service). It never appears in configuration files, the database, process
  arguments, logs, or the Pi process.

## What it is not

- Not a cloud service, account system, or multi-tenant control plane — everything
  is local and single-owner.
- Not a general agent marketplace: Linux 1.0 targets exactly one qualified agent
  (the pinned Pi package) with its sidecar; other agents require independent
  qualification.
- Not a Linux kernel replacement, driver framework, or eBPF control plane.
- Not Windows- or macOS-qualified today: the current product target for 1.0 is
  Linux x86_64 only.
  A Windows install surface (Credential Manager secret backend, inspectable
  installer and scheduled-task templates) exists in the tree and passes CI, but
  its end-to-end install campaign (B01-W) has not been executed, so Windows
  installation is not offered or claimed.

## Current shape (honest summary)

`partial` overall: installation, daemon, CLI, secrets, Provider proxy, Pi
conversation, Task admission, and the six authority stores are implemented and
tested; fully autonomous Task **execution** (scheduler-driven tool runs and
independent verification wired end-to-end) is not yet connected — see
[Tasks and execution](tasks-and-execution.md). The stable product intent is owned
by [`personal/docs/product/`](../../../docs/product/README.md); this page
tracks what the code does today.

## Current product and Personal 2.0 OPC target

Keep these two baselines separate:

- **Current Linux 1.0/current API:** six resource families; Pi is the only
  qualified Agent; the daemon, CLI, Pi paths, Provider Control Plane, and the
  same-origin `/ui/` SPA at `clients/pc/web/` are current implementation. The
  Web UI is not part of the Linux 1.0 release claim.
- **Personal 2.0 OPC target — `Requires-backend` /
  `Requires-environment`:** Today / Projects / Knowledge, bottom Settings
  (Team and Inbox are not first-level), and a persistent right conversation; Project/Charter/Goal/Plan/
  Routine/Task/Attempt; Role Blueprint/Assignment/Digital Employee; Personal-
  owned Conversations, archive, Vault and admitted Memory; global→Project→
  employee→Task Provider/budget control; and one fixed Windows acceptance path.
- **Agent boundary:** Pi is the hidden candidate-only Personal Assistant
  engine. DSH is the preinstalled managed Installed Agent and default employee
  runtime through an exact audited artifact, isolated child, bounded stdio
  broker and daemon Provider proxy. Personal owns Conversation, Memory, Task
  and completion. Hermes, Codex, Cursor and others are future qualification
  candidates.
- **Deferred:** MCP remains an advanced seventh-family target but is not an OPC
  P0 dependency. Native mobile, device pairing and E2E relay remote begin in
  Personal 2.1.
- **Current interaction prototype (not shipped):** owner-approved 2026-08-30
  chrome is `personal-20-opc-e2e-optimized-v9` under
  `clients/docs/design/opc-2.0/`. It is a Canvas specification, not daemon
  `/ui/`. Create order is ① project → ② process → ③ members → ④ test → ⑤ joint.
  Maintain one module or flow from
  [`clients/docs/design/opc-2.0/00-maintenance-index.md`](../../../../clients/docs/design/opc-2.0/00-maintenance-index.md);
  do not treat that catalog as shipped UI. Owner prototype approval is not
  usability, Gate, or release evidence.

No OPC backend or Windows/DSH qualification is claimed. Phase 11's future fixed
denominator is 15 scenarios; Canvas and ordinary CI do not execute or promote
it. There is no human desirability, usability, adoption, willingness-to-pay,
support, release, Gate, Profile or Agent-benefit evidence.
