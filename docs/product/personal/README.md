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

CognitiveOS Personal is a single-user, local **operating system for cognitive
resources**—a unified substrate for mainstream Agents. It gives one owner a
low-friction place to organize and govern six user-visible families: Memory,
Skill, Tool, Context, Task and Runtime/Process. A Pi-hosted Agent Shell is the
primary conversational entry for Linux 1.0, while the Rust daemon remains the
sole authority writer. Post-1.0 design (see
[personal-2.0-scope.md](personal-2.0-scope.md)) extends the same substrate to
independently qualified Agents under a Universal Adapter Contract.

Budget, Permission, Model, Artifact, Intent/Effect, Evidence and Event are
cross-cutting objects, not extra resource families. The product does not
collapse the six families into a giant `Resource` table or universal state
machine.

Linux 1.0 delivers a minimum real slice of every family. The per-Agent sidecar
is the primary Agent integration boundary, and Pi is the only qualified Agent
and sidecar combination for 1.0. Other Agents remain independently qualified
future work under Phase 8.

## Stable product shape

- **Authority:** Pi, sidecars, CLI, SDK and future UI are clients; only the
  daemon authorizes, applies CAS/epoch guards, schedules, commits Effects,
  reconciles and accepts Tasks.
- **Workspace:** a Standard Workspace provides low-friction bounded access;
  Extended Home adds explicitly selected document/project roots and ordinary
  outbound network access, while credential stores, authority data, system
  sockets/directories and privilege management stay hard-denied.
- **Local modes:** desktop, headless and foreground operation use the same
  signed artifact, daemon and application services. Desktop uses Secret
  Service; headless can use a locked encrypted vault with TTY unlock and an
  optional systemd encrypted-credential unlock path.
- **Evolution:** Linux and hardware integration stabilize bounded software
  ports. Linux 1.0 does not include a kernel module, eBPF control plane, device
  scheduler or distributed authority.

## Information architecture

The stable top-level spaces are:

1. **Home** - readiness, active work, health and blockers;
2. **Agents** - package, installation, registration, instance and sidecar;
3. **Tasks** - intent, preview, bounds, progress and acceptance;
4. **Resources** - Memory, Skills, Tools and Context;
5. **Activity** - Run, Process, Effect and Evidence.

## Documents

| Document | Responsibility |
|---|---|
| [Product design](product-design.md) | positioning, users, principles, surfaces, workspace and information architecture |
| [Cognitive resource model](cognitive-resource-model.md) | six resource families, cross-cutting objects, actions, identity and storage boundaries |
| [Linux 1.0 scope](linux-1.0-scope.md) | minimum real slices, Pi qualification, Gate composition, deferred and forbidden boundaries |
| [Personal 2.0 scope](personal-2.0-scope.md) | post-1.0 design baseline: adapter, multi-agent mainline, pillars, headroom non-claims |
| [User journeys](user-journeys.md) | install, Memory, Skill, Tool, Context, Task, Runtime, recovery and support flows |

The baseline decisions are
[ADR-0037](../../adr/0037-personal-unified-cognitive-resource-substrate.md) and
[ADR-0038](../../adr/0038-personal-agent-sidecar-linux-evolution-boundary.md).

## Non-duplication and non-claim rules

- Product requirements link to `PERS-PR-*`; they do not invent REQ IDs.
- Tasks and Gate targets are linked, not copied as current status.
- `CognitiveResourceManifest` keeps its normative ActivityContext discovery
  meaning; it is not redefined as this product taxonomy.
- Environment results are linked from
  [PERSONAL-TEST-ENVIRONMENTS.md](../../plan/PERSONAL-TEST-ENVIRONMENTS.md).
- Release claims use only the exact scope proved by `GMVP-LINUX`.
- Architecture presence, a documented Gate composition and these product
  decisions do not imply implementation, Gate, release or Profile evidence.
