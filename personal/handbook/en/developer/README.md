---
doc_id: dev.index
locale: en
kind: navigation
audience: [developer]
generated: false
---

# Developer guide

How the implementation actually works, mapped file-by-file to sources and tests.
Capability labels are honest: `partial` pages say exactly which wiring is missing.

Read every Personal 2.0 statement against the current boundary: current
Linux/API composition is six-family, Pi-only qualified, with the existing
same-origin `/ui/` SPA at `clients/pc/web/`. Personal 2.0 is a Windows-first
OPC target with Project/Role/Employee/Routine/Attempt authority,
Personal-owned Conversation/Vault/Memory, hidden Pi Assistant engine,
preinstalled managed DSH child, Provider/budget hierarchy and OPC UI. Every
missing item is `Requires-backend`/`Requires-environment`, not implementation.
MCP is advanced/deferred and native mobile/E2E relay remote is 2.1.
Canvas v9 is the frozen design prototype, not the product. Product origin is
daemon `/ui/`. Dual Track L1 is **Now / hypothesis chrome**. Authority remains
the P11 walking skeleton. `P11-T15` is independent / not-started.
One-module OPC maintenance:
[`00-maintenance-index.md`](../../../../clients/docs/design/opc-2.0/00-maintenance-index.md).

Orientation:

1. [Repository map](repository-map.md)
2. [Architecture overview](architecture-overview.md) — target design vs current composition
3. [Development environments](development-environments.md) — what runs where
4. [Contributing workflow](contributing-workflow.md) — leases, branches, CI, docs sync

The authority core:

5. [Authority kernel](authority-kernel.md) — transition gate, intent chain, budgets, recovery
6. [Store and migrations](store-and-migrations.md) — SQLite layout v1–v26
7. [Task pipeline](task-pipeline.md) — record → interpret → preview → admit → watch
8. [Execution-chain status](execution-chain-status.md) — what is wired, what is not

Domains and surfaces:

9. [Daemon and HTTP](daemon-and-http.md)
10. [Context and Artifact](context-and-artifact.md)
11. [Memory and Skill](memory-and-skill.md)
12. [Agent and Pi lifecycle](agent-and-pi-lifecycle.md)
13. [Installer and service](installer-and-service.md)
14. [Management plane](management-plane.md)
15. [TypeScript clients](typescript-clients.md)
16. [Contracts and codegen](contracts-and-codegen.md)
17. [Conformance and testing](conformance-and-testing.md)
18. [Performance surfaces](performance-surfaces.md)

Machine references (generated): [reference section](../reference/README.md).
