# Resource Manager — product design

- Status: informative product design for CognitiveOS Personal
- Change class: `implementation-only` companion (no new public contract)
- Project: `cognitiveos-personal`
- Architecture pair: [resource-manager-architecture.md](../../architecture/personal/resource-manager-architecture.md)
- Current-status owner: [PROGRESS.md](../../plan/PROGRESS.md) `Current snapshot`
- Task owner: [PERSONAL-DEVELOPMENT-PLAN.md](../../plan/PERSONAL-DEVELOPMENT-PLAN.md) `P8-T12`

This document describes the owner-facing Resource Manager: one versioned way
for the Shell and the deterministic CLI to list, inspect, watch, bind, unbind,
enable, disable, and revoke cognitive resources. It does not invent a seventh
resource family, a public generic DTO, or a writable universal aggregate.

## Why this surface exists

Personal already has six independent domain lifecycles (Memory, Skill, Tool,
Context, Task, Runtime) and a private six-family projection. Family-specific
mutations exist (`skill/bind`, `tool/enable`, `memory/remember`). Operators and
clients still need one common envelope so they do not learn six HTTP dialects
for the shared operations named in
[system architecture §3.1](../../architecture/personal/system-architecture.md).

The Resource Manager is that envelope. Each command is resolved to a typed
domain sink. The Rust daemon remains the only authority writer.

## Common operations

| Operation | Operator meaning | Not this operation |
|---|---|---|
| `list` | bounded family page at a declared projection version | full-table dump, search, ranking |
| `inspect` | one stable ID and current object version | generic write-back of the envelope |
| `watch` | resume the existing family watch cursor | a second SSE surface |
| `bind` | typed relationship under expected-version guards | generic create |
| `unbind` | remove a typed relationship under guards | delete/purge of domain history |
| `enable` | admit to a domain-defined usable state | execute |
| `disable` | stop new use without fabricating completion | uninstall |
| `revoke` | invalidate a grant, binding, or usable revision | Memory forget (different domain verb) |

Generic `create`, `install`, `execute`, and `complete` are refused. Acquisition,
admission, execution, reconciliation, retention, and purge stay typed domain or
Task workflows.

## Envelope (read projection)

List and inspect items expose a stable envelope assembled from authority facts:

- stable ID and family
- revision digest, or an explicit reason the object has none
- owner and scope
- health
- typed bindings
- blocked reason
- currently allowed common actions
- object version and projection version

The envelope cannot be written back as a generic resource object. Unknown or
unavailable data stays explicit. Context and Runtime may honestly return an
empty, projection-only page rather than fabricating AgentInstance or ContextView
rows.

## Channels and callers

- Management channel only. Task-channel Resource Manager routes fail closed.
- `cognitive resource list|inspect|bind|unbind|enable|disable|revoke` is the
  deterministic CLI caller.
- `cognitive resource get|watch` remains the private six-family projection and
  existing watch cursor; watch is not duplicated.

## Non-claims

This design does not create a public contract, a `Resource` SQLite table, Gate
evidence, a release, a Profile, or an Agent-benefit claim. Implementation
evidence lives on `P8-T12` and does not promote those conclusions.
