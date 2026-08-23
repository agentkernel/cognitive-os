---
doc_id: user.index
locale: en
kind: navigation
audience: [user]
generated: false
---

# User guide

CognitiveOS Personal is a local, single-owner **operating system for cognitive
resources**: one Rust daemon governs what your AI agents know (Memory), may reuse
(Skills), may do (Tools), see (Context), work on (Tasks), and run as
(Runtime/Process). This guide documents only behavior supported by code, contracts,
and tests together; capability labels (`implemented`, `partial`, `designed`,
`unavailable`) appear on every page.

Start here:

1. [What Personal is (and is not)](./what-is-personal.md)
2. [Install and reach the first conversation](./install-and-first-conversation.md)
3. [CLI basics](./cli-basics.md) — `cognitive init | status | doctor | daemon | pi | resource | task`
4. [Provider and secrets](./provider-and-secrets.md)
5. [Provider Control Plane](./provider-control-plane.md) — named accounts, keys, bindings, usage (CLI/daemon only; no Web panel)
6. [The Pi shell](./pi-shell.md)

Understand the model:

7. [The six resource families](./six-resources.md)
8. [Tasks and execution](./tasks-and-execution.md)

Operate it:

9. [Operations and recovery](./operations-and-recovery.md)
10. [Security boundaries](./security-boundaries.md)
11. [Known limitations](./known-limitations.md)

Exact command, route, error, and file references live in the
[reference section](../reference/README.md). Current project status is owned by
[`docs/plan/PROGRESS.md`](../../../docs/plan/PROGRESS.md) and is deliberately not
duplicated here.
