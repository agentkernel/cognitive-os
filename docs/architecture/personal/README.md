# CognitiveOS Personal Architecture

- Status: active informative product architecture
- Project: `cognitiveos-personal`
- Current-status owner: [PROGRESS.md](../../plan/PROGRESS.md) `Current snapshot`
- Task/Gate owner: [PERSONAL-DEVELOPMENT-PLAN.md](../../plan/PERSONAL-DEVELOPMENT-PLAN.md)
- Normative contracts: [`specs/`](../../../specs/) and applicable
  [standards](../../standards/)

This directory explains how the CognitiveOS architecture is composed into the
Personal product. It does not create registry requirements, public DTOs, task
status, Gate evidence, release claims or Profile conformance.

## Product architecture statement

CognitiveOS Personal is a local cognitive-resource control plane with an Agent
Shell as its primary user entry. The Shell compiles natural-language intent
into governed proposals; the Rust daemon deterministically owns authorization,
state, budgets, scheduling, Agent lifecycle, Intent/Effect, recovery and final
acceptance.

Pi has two independent roles:

1. Pi hosts the first Agent Shell user experience.
2. Pi is the only managed Agent qualified for the Linux 1.0 target.

These roles may share a runtime process but never share authority identity.

## Documents

| Document | Responsibility |
|---|---|
| [System architecture](system-architecture.md) | layers, composition roots, trust boundaries and end-to-end flows |
| [Agent Shell and Agent lifecycle](agent-shell-and-agent-lifecycle.md) | Pi dual roles, identity model and lifecycle operations |
| [Authority, data and recovery](authority-data-and-recovery.md) | Task/Loop, capability, budget, Intent/Effect, evidence and recovery invariants |

## Source ownership

When these documents disagree with another source:

1. machine shape and registered state transitions come from `specs/`;
2. behavioral semantics come from applicable normative companions and
   `docs/standards/`;
3. Personal product decisions come from accepted Personal ADRs;
4. formal tasks and Gates come from the Personal development plan;
5. current facts come only from `PROGRESS.md` Current snapshot;
6. this directory is updated to match those sources.

Architecture presence is not implementation evidence. Implementation presence
is not a product Gate or release result.
