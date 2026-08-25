---
doc_id: user.what-is-personal
locale: en
kind: overview
audience: [user]
status: partial
generated: false
sources:
  - path: personal/docs/product/product-design.md
  - path: personal/apps/kernel-server/src/personal/server.rs
    symbols: ["serve_personal_loopback"]
  - path: personal/docs/product/linux-1.0-scope.md
tests:
  - personal/apps/kernel-server/tests/p1_t04_personal_daemon.rs
fingerprint: "sha256:e6a88761a70a397baaf92ed549ec43667edfb79429763cd795e72c22dd8cff3a"
non_claims:
  - No Gate, release, Profile, Windows-parity, or agent-benefit claim; the Linux 1.0 target composition is owned by the formal plan.
---

# What Personal is (and is not)

## What it is

A local daemon plus deterministic clients that make agent work **auditable,
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
- Not Windows-installable today: the product target for 1.0 is Linux x86_64 only.
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
