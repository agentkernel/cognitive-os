---
doc_id: user.system-overview
locale: en
kind: overview
audience: [user]
status: partial
generated: false
sources:
  - path: docs/product/personal/README.md
  - path: docs/product/personal/cognitive-resource-model.md
  - path: docs/architecture/personal/README.md
  - path: apps/kernel-server/src/personal/resource_api.rs
  - path: apps/kernel-server/src/personal/task_api.rs
tests:
  - apps/kernel-server/tests/p2_t02_resource_projection.rs
  - apps/kernel-server/tests/p2_t02_task_api_watch.rs
  - apps/kernel-server/tests/p2_t28_end_to_end_journey.rs
fingerprint: "sha256:4f72ff9d8badb8656fbe390643e814b996065842cf38f3daaa880f122a35e9f2"
non_claims:
  - This is an orientation page, not a release, Gate, Profile, or agent-benefit claim.
  - Fully autonomous scheduler-driven execution and independent verification remain partial; see Tasks and execution.
  - Linux 1.0 does not claim Windows installation parity, Web UI, MCP/dynamic tools, or multi-agent orchestration.
---

# System overview

CognitiveOS Personal is a local, single-owner operating system for cognitive
resources. It gives an agent a governed place to remember information, load
skills, use tools, assemble context, accept tasks, and run a managed process.
The product is a Rust daemon plus clients: the daemon owns authority state and
clients such as `cognitive`, Pi, and SDKs request or propose operations.

## The model in one picture

```text
You -> cognitive CLI or Pi Shell -> local Rust daemon -> six domain services
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

For the shortest practical route, continue with [Getting started](./getting-started.md).
For exact commands, see [CLI basics](./cli-basics.md) and the [reference](../reference/README.md).
