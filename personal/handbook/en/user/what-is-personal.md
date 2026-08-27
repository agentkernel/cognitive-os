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
tests:
  - personal/apps/kernel-server/tests/p1_t04_personal_daemon.rs
fingerprint: "sha256:539cbd20fa414530f63e5eff6852b751a6f2f90f379ee77307fb8853619a89e0"
non_claims:
  - No Gate, release, Profile, Windows-parity, or agent-benefit claim; the Linux 1.0 target composition is owned by the formal plan.
---

# What Personal is (and is not)

## What it is

A cross-platform local stewardship product for one owner's Agents, accounts,
resources, and governed work. Its current release is Linux 1.0; its full
Personal 2.0 target independently qualifies Windows, macOS, and Linux paths.
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

## Current product and full Personal 2.0 commitment

Keep these two baselines separate:

- **Current Linux 1.0/current API:** six resource families; Pi is the only
  qualified Agent; the daemon, CLI, Pi paths, Provider Control Plane, and the
  same-origin `/ui/` SPA at `clients/pc/web/` are current implementation. The
  Web UI is not part of the Linux 1.0 release claim.
- **Full Personal 2.0 target — `Requires-backend`:** independently qualified
  Windows, macOS, and Linux local product paths; exact Pi, DeepSeek Harness
  Developer Preview, and supported-platform Codex desktop paths; Account Hub;
  MCP as a seventh family; embedded conversations; durable Goal -> Plan
  revision -> Task -> Attempt work; multi-Agent supervision; unified Activity;
  controls; and federated resources.

The complete version commitment makes every item a release blocker, but remains
separate from implementation evidence. The fixed AI-window denominator stays
eight scenarios; the Codex desktop scenario is platform-conditional (owner
decision 2026-08-27) and is recorded `not-run (platform-conditional)` while no
supported Codex desktop platform is in the active execution scope, so
Linux-mainline acceptance closes at seven platform-eligible passes plus that
disposition. Even a full 8/8 pass is simulated product acceptance only and
proves no human desirability, usability, adoption, willingness to pay, or
release/Gate technical readiness.
