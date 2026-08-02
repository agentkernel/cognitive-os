# ADR-0035: Personal Pi Shell Host and Managed Agent Role Separation

- Status: Accepted
- Date: 2026-08-02
- Decision owners: CognitiveOS Personal product owner
- Classification: product-semantic and Personal architecture decision
- Related: ADR-0016, ADR-0022, ADR-0026, ADR-0027, P2-T02, P5-T01,
  P5-T02, P5-T05, P7-T08

## Context

Personal currently uses Pi in two ways that are compatible but must not be
collapsed into one identity or authority boundary.

1. `packages/pi-cognitiveos` runs inside Pi and provides the first
   conversational terminal surface. This is the beginning of the Personal
   Agent Shell experience.
2. Pi is also an external Agent runtime that Personal 1.0 will acquire,
   verify, register, supervise and bind to governed Task executions.

The first role is a user-interface/client role. The second is a managed
resource role. A Pi process may participate in both roles, but sharing a
process does not merge their credentials, sessions, lifecycle, permissions or
completion semantics.

## Decision

### 1. Two independent roles

**Pi-hosted Agent Shell** is a non-authority UI adapter. It translates natural
language and explicit commands into proposals, obtains deterministic previews,
submits admitted requests, and renders daemon projections. It never writes the
authority database, mints capabilities, dispatches external Effects or decides
that a Task is complete.

**Managed Pi Agent** is an installed Agent resource. Personal owns its package
record, installation transaction, registration, health, activation,
supervision, execution binding, suspension, upgrade, rollback and removal.
Installation never grants runtime permission.

### 2. Distinct identities

The following identities must remain distinct even when implementation code or
one OS process is shared:

| Identity | Meaning | Authority owner |
|---|---|---|
| `ShellSession` | client interaction and watch cursor | client-local; displayed facts come from daemon projections |
| Pi session | Pi conversation/runtime session | Pi runtime; observation only |
| `AgentPackage` | immutable acquired package and provenance facts | daemon installation service |
| `AgentInstallation` | verified installation transaction and active version | daemon installation authority |
| Agent definition/registry record | Personal policy and adapter binding | daemon management service |
| Agent instance | supervised logical running resource | daemon lifecycle authority |
| `AgentExecution` | Task-bound, epoch-fenced execution | daemon scheduler/runtime authority |
| OS process | disposable host process | process supervisor |
| `Task`/`Loop` | goal, bounds, progress and acceptance | daemon Task/Loop authority |

`AgentDefinition` and `AgentInstance` are product concepts until a Lane-CTR
review decides whether they require public machine contracts. Implementations
must not invent client-visible parallel DTOs before that decision.

### 3. Channel and credential separation

The Pi-hosted Shell uses independent channel-scoped sessions:

- the task channel carries intent, preview, admission, watch and Task control;
- the management channel carries resource inspection and explicitly authorized
  Agent lifecycle requests;
- a bearer, cache, projection or retry context from one channel is never reused
  by the other;
- ordinary conversation text is not a privileged management context.

The local bootstrap secret only mints bounded channel credentials. Provider
credentials remain in the native Secret Store and are resolved only by the
daemon-owned Provider proxy.

### 4. Natural-language compilation boundary

Natural language may produce an interpretation candidate or
`ShellActionProposal`. The daemon fixes the raw intent before interpretation,
resolves targets and policy, emits a canonical digest-bound preview, and admits
the exact preview under current epoch/CAS guards. A changed proposal, stale
preview, ambiguous target or changed permission fails closed.

Pi tool calls remain default-deny. Governed operations run through the daemon
Tool Registry, capability check and Intent/Effect protocol rather than through
Pi built-in tools.

### 5. Completion and recovery

Pi output, provider success, process exit and Pi `agent_end` are observations.
None advances a Task to completed. Completion requires closed/reconciled
Effects, criteria evidence and an independent verifier/acceptance transition.

After restart or supervision loss, the daemon reloads durable facts, fences old
execution epochs, reconciles unknown Effects, reauthorizes current operations
and only then resumes or replaces the Agent execution.

### 6. Migration of the current Pi surface

The current product-private `pi.json` and user-installed Pi path remain valid
development/legacy observations. They are not silently promoted into a
qualified Personal 1.0 installation. The P5-T01 migration must show a preview,
acquire the pinned official package through ADR-0036, create a durable
installation/registry binding, and preserve or explicitly replace the Shell
configuration. Existing conversation evidence remains historical evidence and
does not prove the new managed-Agent lifecycle.

## Consequences

- P2-T02 composes the Pi UI with the reusable task-channel Shell core and the
  daemon Task application service.
- P5-T01/P5-T02 independently make Pi a managed Agent.
- Other Agent adapters may host no Shell at all, while another Shell client may
  manage Pi; the roles remain composable rather than hard-coded.
- Adapter qualification and release claims are per adapter identity, version,
  digest and campaign. Pi evidence cannot qualify Codex, OpenClaw, Hermes,
  WorkBuddy or another adapter.

## Rejected alternatives

1. **Treat Pi as the authority because it hosts the Shell.** Rejected: UI
   placement cannot grant authorization, state-transition or completion power.
2. **Treat a Pi session as an Agent instance.** Rejected: conversations are
   ephemeral and cannot carry durable installation, health, budget or fencing
   identity.
3. **Use one management bearer for every Pi action.** Rejected: it collapses
   channel isolation and gives ordinary conversation a privileged context.
4. **Let Pi built-in tools become governed by naming convention.** Rejected:
   there is no catalog, capability or persist-before-dispatch proof inside Pi.

## Non-claims

This decision does not implement the Task API, Agent registry, managed Pi
lifecycle, B09, GMVP-LINUX, release or Profile conformance.
