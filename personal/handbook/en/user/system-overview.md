---
doc_id: user.system-overview
locale: en
kind: overview
audience: [user]
status: partial
generated: false
sources:
  - path: personal/docs/product/README.md
  - path: personal/docs/product/cognitive-resource-model.md
  - path: personal/docs/product/personal-2.0-scope.md
  - path: personal/docs/product/account-hub.md
  - path: personal/docs/product/account-hub.zh-CN.md
  - path: personal/docs/product/agent-integration-and-conversations.md
  - path: personal/docs/product/agent-integration-and-conversations.zh-CN.md
  - path: personal/docs/product/mcp-resource-family.md
  - path: personal/docs/product/mcp-resource-family.zh-CN.md
  - path: personal/docs/architecture/README.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: personal/docs/product/opc-product-model.md
  - path: personal/docs/product/knowledge-memory-vault.md
  - path: personal/docs/product/long-running-operations.md
  - path: personal/apps/kernel-server/src/personal/resource_api.rs
  - path: personal/apps/kernel-server/src/personal/task_api.rs
tests:
  - personal/apps/kernel-server/tests/p2_t02_resource_projection.rs
  - personal/apps/kernel-server/tests/p2_t02_task_api_watch.rs
  - personal/apps/kernel-server/tests/p2_t28_end_to_end_journey.rs
fingerprint: "sha256:ea36d293326ec209faf2c9ba8067d056b5d06e9572ef4a5291ea03b83fa0dd8d"
non_claims:
  - This is an orientation page, not a release, Gate, Profile, or agent-benefit claim.
  - Fully autonomous scheduler-driven execution and independent verification remain partial; see Tasks and execution.
  - Linux 1.0 does not include Windows installation parity, Web UI, MCP-family, or multi-Agent orchestration in its claim composition; current `/ui/` existence does not change that boundary.
---

# System overview

CognitiveOS Personal is a local, single-owner system for governed Agent work.
The current Linux 1.0 product manages six cognitive-resource families. The
Personal 2.0 target adds a Windows-first OPC business console where the Owner
operates Projects and digital employees. The Rust daemon owns authority; every
UI, Assistant, engine, adapter and connector requests, proposes or observes.

## The model in one picture

```text
You -> cognitive CLI or Pi Shell -> local Rust daemon -> current six domain services
                                      |                 -> Provider proxy
                                      |                 -> SQLite authority store
                                      `-> Intent/Effect, budget, evidence, events
```

The daemon is the only authority writer. A client response, Provider response,
Pi `agent_end`, or process exit is not by itself task completion. Mutating work
is recorded before dispatch and must be reconciled; completion requires an
independent, current verification result.

## Six resource families

| Family | What you use it for | Current surface |
|---|---|---|
| Memory | Durable, scoped knowledge with provenance and forgetting | Daemon `remember`/`forget`/explain routes and authority-backed search |
| Skill | Versioned local packages and bindings | Import, bind, revoke, and explain routes; packages do not execute themselves |
| Tool | Bounded operations such as workspace read/search/write/patch | Static catalog, lifecycle overlay, validators, and governed invocation paths |
| Context | Authorized input assembled for a task | Daemon-side filtering, reauthorization, digest binding, and bounded views |
| Task | A durable intent, preview, contract, progress, and acceptance record | Task admission, watch, evidence, and scheduler state |
| Runtime/Process | Agent package, installation, instance, sidecar, and process attempt | Managed Pi lifecycle and runtime projections |

Budget, Permission, Model, Artifact, Intent/Effect, Evidence, and Event are
cross-cutting objects. They do not form a seventh universal resource table.

## Personal 2.0 Windows OPC target (`Requires-backend`)

The target keeps one daemon and organizes:

```text
Owner
  -> Project -> Charter / Goal / Plan / Routine -> Task -> Attempt
  -> Role Blueprint -> Assignment -> Digital Employee
       -> managed DSH runtime
       -> Personal-owned Conversation and Memory
```

The UI is Today / Projects / Knowledge, bottom Settings, and a persistent
right conversation. Team and Inbox are not first-level destinations. The
owner-approved current chrome is **CognitiveOS Personal 2.0.0**; the canvas
file may keep `personal-20-opc-e2e-optimized-v9` as a historical filename.
It is not daemon `/ui/`. Create order is
project → process → members → per-stage test → joint. One-module PM/UI
maintenance starts at
[`00-maintenance-index.md`](../../../../clients/docs/design/opc-2.0/00-maintenance-index.md).
Pi is the hidden candidate-only Assistant engine.
DSH is the hidden hosted Member execution engine (not a visible Installed
Agent), using an exact audited artifact, isolated child, stdio broker and daemon
Provider proxy. Personal owns Conversation, archive/index/retrieval, Memory,
Task, Effect and completion.

Knowledge separates Owner-shared sources, Project Markdown Vault and employee-
private Memory. Routines use manual/schedule/qualified-event triggers with no
overlap, queue-latest and visible missed work. HITL is canvas-only. Provider
binding resolves global to Project to employee to Task; actual usage stays
distinct from Provider quota; unknown cost is never shown as 0; member-level
budget is 2.1 / Deferred.

All of this remains target-only. `Requires-backend`/`Requires-environment`
means it must not be shown as working. MCP stays an advanced deferred seventh
family; native mobile/E2E relay remote is 2.1; future Agents need independent
qualification.

## How a normal interaction flows

1. You configure a Provider and a selected model; the key remains in the
   approved OS secret store.
2. Pi sends a bounded conversation request to the daemon Provider proxy. Pi
   never receives the key and cannot use native shell or file tools to bypass
   the daemon.
3. A task request is admitted only after the daemon records its intent, bounds,
   budget, and runnable state.
4. A governed tool operation creates an Intent/Effect record before external
   dispatch. The daemon reconciles the result under fencing.
5. The verifier reads durable state and evidence. Only then can the authority
   advance a task to completion.

## What to expect today

The installed and tested foundation includes the daemon, CLI, secret handling,
Provider proxy, Pi conversation path, six authority stores, task admission, and
resource/task projections. `GET /management/usage` now returns source-labelled
costs (`actual` | `estimated` | `unknown`; unknown is never `0`), a four-layer
binding explanation with missing Project/employee/Task layers unbound, and
separated account vs quota fields. Settings usage chrome remains T13. The
end-to-end autonomous execution path is still
`partial`; use the task watch and evidence commands to inspect durable facts,
and do not interpret an interactive answer as a completed task.

For the shortest practical route, continue with [Getting started](getting-started.md).
For exact commands, see [CLI basics](cli-basics.md) and the [reference](../reference/README.md).
