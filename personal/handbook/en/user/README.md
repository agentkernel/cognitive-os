---
doc_id: user.index
locale: en
kind: navigation
audience: [user]
generated: false
---

# User guide

CognitiveOS Personal is a cross-platform local, single-owner stewardship
product for Agents, accounts, cognitive resources, and governed work. One Rust
daemon governs what your Agents know (Memory), may reuse (Skills), may do
(Tools), see (Context), work on (Tasks), and run as (Runtime/Process). Those are
the six current Linux 1.0/API families. This guide labels current behavior from
code, contracts, and tests (`implemented`, `partial`, `designed`,
`unavailable`) separately from the full Personal 2.0 commitment that still
needs backend or core work (`Requires-backend`, `Requires-core`).

Personal 2.0 requires independently qualified Windows, macOS, and Linux local
paths; exact Pi, DeepSeek Harness Developer Preview, and supported-platform
Codex desktop paths; embedded Agent conversations; Account Hub; MCP as a
seventh family; Goal/Plan/Task/Attempt and multi-Agent supervision; unified
Activity; and federated resources. The current same-origin `/ui/` client is
real at `clients/pc/web/`; those target additions are not implemented.

Start here:

1. [What Personal is (and is not)](what-is-personal.md)
2. [Getting started](getting-started.md) — shortest supported Linux path
3. [Install and reach the first conversation](install-and-first-conversation.md)
4. [CLI basics](cli-basics.md) — `cognitive init | status | doctor | daemon | pi | resource | task`
5. [Provider and secrets](provider-and-secrets.md)
6. [Provider Control Plane](provider-control-plane.md) — current named accounts, keys, bindings, usage, and same-origin `/ui/`; adopted Account Hub target
7. [The Pi shell](pi-shell.md)

Understand the model:

8. [System overview](system-overview.md)
9. [Current six families and the target seventh MCP family](six-resources.md)
10. [Tasks and execution](tasks-and-execution.md)

Operate it:

11. [Operations and recovery](operations-and-recovery.md)
12. [Security boundaries](security-boundaries.md)
13. [Known limitations](known-limitations.md)
14. [Linux RC operator map](rc-and-support.md) — install/init/provider/Pi/task/recovery/update/uninstall index

Exact command, route, error, and file references live in the
[reference section](../reference/README.md). Current project status is owned by
[`docs/plan/PROGRESS.md`](../../../../docs/plan/PROGRESS.md) and is deliberately not
duplicated here.
