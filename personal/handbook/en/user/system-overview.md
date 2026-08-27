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
  - path: personal/apps/kernel-server/src/personal/resource_api.rs
  - path: personal/apps/kernel-server/src/personal/task_api.rs
tests:
  - personal/apps/kernel-server/tests/p2_t02_resource_projection.rs
  - personal/apps/kernel-server/tests/p2_t02_task_api_watch.rs
  - personal/apps/kernel-server/tests/p2_t28_end_to_end_journey.rs
fingerprint: "sha256:74f6954d2ff14cb31bbb32b4b637ec1b1d373e6e0034056ec0d454deb9c8c714"
non_claims:
  - This is an orientation page, not a release, Gate, Profile, or agent-benefit claim.
  - Fully autonomous scheduler-driven execution and independent verification remain partial; see Tasks and execution.
  - Linux 1.0 does not include Windows installation parity, Web UI, MCP-family, or multi-Agent orchestration in its claim composition; current `/ui/` existence does not change that boundary.
---

# System overview

CognitiveOS Personal is a cross-platform local, single-owner stewardship
product for Agents, accounts, cognitive resources, and governed work. It gives
an Agent a governed place to remember information, load skills, use tools,
assemble context, accept tasks, and run a managed process. The product is a
Rust daemon plus clients: the daemon owns authority state and clients such as
`cognitive`, Pi, and SDKs request or propose operations.

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

## Full Personal 2.0 target (`Requires-backend`)

Personal 2.0 keeps the daemon-only authority boundary and makes the following
full-version commitments:

- Windows, macOS, and Linux are independently qualified local product paths;
- the exact initial Agent set is Pi, DeepSeek Harness Developer Preview, and
  the Codex experience in the current official ChatGPT desktop app only on
  officially supported and independently qualified platforms;
- target navigation is Home, Agents, Work, Library, Activity, and Settings;
  Providers and System live under Settings, and `Work` is a navigation label
  rather than a new Task/Run authority domain;
- Account Hub manages Provider accounts and subscriptions and offers
  user-initiated, per-source credential import through the daemon;
- MCP becomes a seventh resource family for source-identified federated
  capabilities; it is not an alias for the current native Tool catalog;
- vendor-specific conversation adapters connect installed Agents to the Agent
  Shell without qualifying them by association with Pi;
- embedded native conversations enter governed work only through
  **Manage with Personal**;
- durable Goal -> Plan revision -> Task -> Attempt organizes work, preserves
  attempts, and supports daemon-owned multi-Agent handoffs;
- unified Activity separates Native, Observed, Governed, and Verified facts
  with declared coverage.

These additions remain target-only. `Requires-backend` means the product must
not show them as working; `Requires-core` additionally marks work that needs
approved contract/authority semantics. Platform, Agent, CLI, Provider, model,
account, bridge, and MCP evidence never transfers across claim boundaries.

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
resource/task projections. The end-to-end autonomous execution path is still
`partial`; use the task watch and evidence commands to inspect durable facts,
and do not interpret an interactive answer as a completed task.

For the shortest practical route, continue with [Getting started](getting-started.md).
For exact commands, see [CLI basics](cli-basics.md) and the [reference](../reference/README.md).
