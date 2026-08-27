---
doc_id: index
locale: en
kind: navigation
audience: [user, developer, ai]
generated: false
---

# CognitiveOS Personal Handbook (English)

A cross-platform local, single-owner stewardship product for Agents, accounts,
cognitive resources, and governed work: one Rust daemon governs what Agents
know, may reuse, may do, see, work on, and run as. This handbook separates
current implementation truth from the full Personal 2.0 product-version
commitment; target-only behavior is always marked `Requires-backend` or
`Requires-core`.

**Status boundary:** Linux 1.0 and the current APIs remain six-family and Pi is
the only qualified Agent. The current same-origin `/ui/` SPA exists at
`clients/pc/web/`. Personal 2.0 requires independently qualified Windows,
macOS, and Linux local product paths; exact Pi, DeepSeek Harness Developer
Preview, and supported-platform Codex desktop paths; embedded conversations;
Goal/Plan/Task/Attempt and multi-Agent supervision; Account Hub; a seventh MCP
family; unified Activity; and federated resources. These are full-version
release blockers and remain `Requires-backend`, not current implementation.

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
