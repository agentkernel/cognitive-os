# CognitiveOS Personal Architecture

- Status: informative target/design baseline
- Change class: owner-approved `product-semantic + structural` documentation
- Project: `cognitiveos-personal`
- Current-status owner: [PROGRESS.md](../../plan/PROGRESS.md) `Current snapshot`
- Task/Gate owner: [PERSONAL-DEVELOPMENT-PLAN.md](../../plan/PERSONAL-DEVELOPMENT-PLAN.md)
- Normative contracts: [`specs/`](../../../specs) and applicable
  [standards](../../standards)

This directory explains the intended Personal composition. It does not create
registry requirements, public DTOs, a second authority, current task status,
Gate evidence, release claims or Profile conformance.

## Product architecture statement

CognitiveOS Personal is a local **operating system for cognitive resources**: a
unified control plane above the host OS. Experience surfaces call two
application-service families: the governed Task service and a narrow
`ResourceApplicationService` projection over six independent domain services:

1. Memory;
2. Skill;
3. Tool;
4. Context;
5. Task;
6. Runtime/Process.

Agent is a user-facing projection over Runtime package, installation,
registration, instance, sidecar, execution and process facts. Model, Budget,
Permission, Artifact, Intent/Effect, Evidence and Event are cross-cutting
objects rather than additional resource families.

The common resource service gives the Shell and deterministic clients one
versioned way to list, inspect, watch, bind, unbind, enable, disable and revoke
resources. It does not collapse the six domains into one schema, one lifecycle
or one giant resource state machine. Each command is resolved to a typed domain
operation and the Rust daemon remains the only authority writer.

Each active `AgentInstance` has one daemon-supervised logical sidecar session.
The sidecar is the private adapter, candidate and observation boundary for the
Agent; scheduler, authorization, budgets, Intent/Effect, reconciliation and
acceptance remain deterministic daemon responsibilities. Linux 1.0 may realize
one logical sidecar as one separate OS process connected through daemon-created
private framed AKP over stdio or a socketpair. It does not add a public listener,
TLS PKI or a service mesh.

Pi has two independent roles:

1. Pi hosts the first Agent Shell experience.
2. Pi is the only Agent adapter qualified for the Linux 1.0 target.

The Shell reaches sidecar/application-service routes as a bounded client. It
does not receive daemon bootstrap or management authority. Shell session, Pi
session, Agent identity, sidecar session, process, execution and Task identity
remain distinct even when an implementation co-locates some of them.

## Documents

| Document | Responsibility |
|---|---|
| [System architecture](./system-architecture.md) | layered control plane, six domains, common projection, execution boundary and future Linux/hardware ports |
| [Resource Manager](./resource-manager-architecture.md) | common ResourceApplicationService HTTP envelope, authority sources, and fail-closed generics (P8-T12) |
| [Agent Shell and Agent lifecycle](./agent-shell-and-agent-lifecycle.md) | Pi dual roles, strict runtime identity, sidecar supervision, channels and lifecycle operations |
| [Authority, data and recovery](./authority-data-and-recovery.md) | authority data, control/data-plane separation, mutation permits, evidence and restart/reconciliation order |
| [Agent adapter contract](./agent-adapter-contract.md) | universal AKP adapter capabilities, lifecycle, negatives (P8 design) |
| [Multi-agent orchestration](./multi-agent-orchestration.md) | mainline multi-agent design with fail-closed default |
| [Context evolution](./context-evolution.md) | compaction and adaptive budgets (P8-T05 design) |
| [Learning loop](./learning-loop.md) | cross-episode Skill/Memory candidate admission (P8-T06 design) |
| [Async event evolution](./async-event-evolution.md) | measured async migration decision gate (P9-T01) |
| [Performance architecture](./performance-architecture.md) | floors, stage timing, structure-debt candidates |
| [Headroom: IoT and multi-tenancy](./headroom-iot-and-multitenancy.md) | reserved bridges; not formal plan tasks |
| [UCR-01 workload](../../evaluation/personal-unified-cognitive-resource-workload.md) | target six-resource trace, fault/reuse assertions and bounded benefit-evaluation design |

## Stable composition rules

- Experience components are clients; their local cache or fluent response is
  never authority state.
- The common resource envelope is a versioned read/action projection, not a
  persisted universal resource aggregate.
- Domain services retain independent schemas, lifecycle guards, event types
  and compatibility rules.
- A sidecar can translate protocols and report bounded observations; it cannot
  authorize, commit an Effect or complete a Task.
- Task and management channels keep separate credentials, retry state, caches
  and operation sets.
- Public contract changes still require the normative contract process. This
  design must not pre-empt that process with parallel DTOs.

## Source ownership and non-claims

When these documents disagree with another source:

1. machine shape and registered state transitions come from `specs/`;
2. behavioral semantics come from applicable normative companions and
   `docs/standards/`;
3. Personal product decisions come from accepted Personal ADRs;
4. formal tasks and Gates come from the Personal development plan;
5. current facts come only from `PROGRESS.md` Current snapshot;
6. this directory is updated to match those sources.

Everything here is target/design unless a canonical evidence source says
otherwise. Architecture presence is not implementation evidence, and neither
architecture nor implementation presence is a Gate, release or Profile result.
