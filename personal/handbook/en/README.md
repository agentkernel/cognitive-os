---
doc_id: index
locale: en
kind: navigation
audience: [user, developer, ai]
generated: false
---

# CognitiveOS Personal Handbook (English)

A local, single-owner operating system for cognitive resources: one Rust daemon
governs what AI agents know, may reuse, may do, see, work on, and run as. This
handbook separates current implementation truth from the adopted Personal 2.0
target; target-only behavior is always marked `Requires-backend` or
`Requires-core`.

**Status boundary:** Linux 1.0 and the current APIs remain six-family and Pi is
the only qualified Agent. The current same-origin `/ui/` SPA exists at
`clients/pc/web/`. Personal 2.0 adopts a desktop-first redesign, a seventh MCP
family, Account Hub credential import, vendor-specific Agent conversation
adapters, Goal/Plan and multi-Agent supervision, and federated resources. Those
target additions — including the redesign of the existing UI — are not
implemented.

- **[User guide](user/README.md)** — install, first conversation, CLI, secrets,
  Provider Control Plane, the Pi shell, the resource model, operations, security,
  limitations.
- **[Developer guide](developer/README.md)** — repository map, authority kernel,
  storage, HTTP surface, execution-chain status, domains, testing, workflow.
- **[Reference](reference/README.md)** — generated CLI/HTTP/error/config/env/
  transition/schema/tool references plus capability and compatibility matrices.
- **[AI entry](ai/README.md)** — source-of-truth order, code map, safe editing,
  validation commands, docs impact.

中文版入口：[`personal/handbook/zh-CN/`](../zh-CN/README.md)。Machine metadata:
[`personal/handbook/_meta/manifest.json`](../_meta/manifest.json). Dynamic project status
is owned by [`docs/plan/PROGRESS.md`](../../../docs/plan/PROGRESS.md) and is never
copied here.
