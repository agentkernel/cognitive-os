# CognitiveOS Personal Product Design

- Status: canonical stable product-design index
- Project: `cognitiveos-personal`
- Current-status owner: [PROGRESS.md](../../plan/PROGRESS.md) `Current snapshot`
- Task/Gate owner: [PERSONAL-DEVELOPMENT-PLAN.md](../../plan/PERSONAL-DEVELOPMENT-PLAN.md)
- Product decisions: [Personal ADRs](../../adr/)

This directory owns stable product intent, user concepts, release boundaries
and user journeys. It does not own implementation status, active leases,
campaign results, evidence or Profile claims.

## Vision

CognitiveOS Personal is an Agent Shell-led personal cognitive-resource
management system. It brings Agents, models, tools, Context, Memory, Tasks,
budgets, permissions, artifacts and execution evidence under one local control
plane so that users can install, connect, supervise, pause, upgrade and remove
Agents while execution remains recoverable, auditable and bounded.

The Linux 1.0 realization is intentionally narrower: Pi hosts the Shell and is
the only product-qualified managed Agent. The generic adapter framework is
prepared for later Agents, but their presence in a roadmap is not support or
release evidence.

## Documents

| Document | Responsibility |
|---|---|
| [Product design](product-design.md) | positioning, users, principles, entry surfaces and success criteria |
| [Cognitive resource model](cognitive-resource-model.md) | user-visible resources, actions, sources of truth and release state |
| [Linux 1.0 scope](linux-1.0-scope.md) | included, framework-ready, deferred, unsupported and non-claim boundaries |
| [User journeys](user-journeys.md) | install, first conversation, Task and Agent lifecycle, recovery and support flows |

## Non-duplication rule

- Product requirements link to `PERS-PR-*`; they do not invent REQ IDs.
- Tasks and Gate thresholds are linked, not copied as current status.
- Environment results are linked from
  [PERSONAL-TEST-ENVIRONMENTS.md](../../plan/PERSONAL-TEST-ENVIRONMENTS.md).
- Release claims use the exact scope proved by `GMVP-LINUX`; deferred features
  cannot be inferred from the product vision.
