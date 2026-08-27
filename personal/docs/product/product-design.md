# CognitiveOS Personal product design

- Status: canonical stable product intent
- Current release boundary: [Linux 1.0 scope](linux-1.0-scope.md)
- Adopted target: [Personal 2.0 scope](personal-2.0-scope.md)
- Architecture: [Personal architecture](../architecture/README.md)
- Decisions:
  [ADR-0037](../../../docs/adr/0037-personal-unified-cognitive-resource-substrate.md),
  [ADR-0038](../../../docs/adr/0038-personal-agent-sidecar-linux-evolution-boundary.md),
  [ADR-0056](../../../docs/adr/0056-personal-2-0-desktop-control-plane.md), and
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md)

## 1. Product statement and reality boundary

CognitiveOS Personal is the owner-local operating system for cognitive
resources and the desktop supervisor for the owner's Agents. It lets a
beginner reach a real conversation quickly, then adds governance when the user
chooses to manage an outcome with Personal.

| Boundary | Product truth |
|---|---|
| **Current implementation (Now)** | Linux 1.0 is six-family and Pi-qualified. P7-T05 delivered daemon-served `/ui/` with Home, Work, Agents, Providers, Resources, Activity, and System. The native dsh panel remains separate. |
| **Adopted Personal 2.0 target** | Desktop Control Plane is the primary entry and supervisor. It embeds adapter-backed native conversations, introduces persistent Goal and daemon-owned Plan views, orchestrates admitted multi-Agent work, federates native resources, and recognizes MCP as the seventh family. |
| **Requires-backend** | Conversation/history projections, Goal -> Plan revision -> Task -> Attempt orchestration, full Task and Agent controls, unified Activity, authority-backed Context/Runtime inventory, federated sync, and MCP management are not complete daemon capabilities today. |
| **Requires-core (conditional)** | P10-T02/Lane-CTR is required only for a new or changed public MCP/Goal/Plan/Run/Harness/conversation surface. Personal-private projections may not require core changes; this document defines no schema or route. |

The Rust daemon remains the sole authority writer. Agents, adapters, the global
Agent Shell, UI, CLI, sidecars, and MCP servers may propose or observe; they do
not authorize, commit Effects, reconcile outcomes, or accept completion.

Personal is an operating layer above Linux. It is not a replacement kernel,
driver framework, distributed control plane, remote administration service, or
launcher that trusts every Agent independently.

## 2. User and jobs

The primary user is one owner who may be new to governed Agents. The product is
**beginner-first by default**: plain labels, safe defaults, and one visible
next action. Stable IDs, digests, epochs, permissions, raw projections, and
audit detail remain available in inspectors. There is no Basic/Expert mode and
no separate product personality for technical users.

The same owner moves among three working modes without changing authority:

- **conversation:** start or continue useful work in an Agent's native model;
- **supervision:** understand status, conflicts, costs, evidence, and next
  actions across Agents;
- **stewardship:** manage accounts, permissions, resources, runtime engines,
  backup, recovery, and updates.

Primary jobs:

1. Reach a real first Agent response within five minutes when prerequisites are
   available.
2. Connect an installed Agent or install one from a signed upstream catalog in
   no more than three onboarding steps.
3. Continue a native conversation, then choose **Manage with Personal** when
   the outcome needs durable Goal, Plan, Task, Context, Effect, and verification
   governance.
4. Supervise one Goal across sessions and Agents without confusing native plans
   or Agent output with daemon authority.
5. Understand what changed through one source-labelled timeline and recover
   without losing prior attempts.
6. Use native Memory, Skills, Tools, Context, Runtime resources, and MCP servers
   through vendor adapters while preserving origin ownership.
7. Configure subscriptions, API keys, imported credentials, custom endpoints,
   models, routing scope, usage, and cost without exposing secret material to
   Agents or the browser.
8. Diagnose and restore service even when a model, Provider, Agent, sidecar,
   native panel, MCP server, or watch stream is unavailable.

## 3. Resource and authority model

### Current implementation (Now)

Linux 1.0 has six user-visible families:

| Family | Product responsibility |
|---|---|
| Memory | admitted durable knowledge with scope, provenance, versions, conflicts, expiry, forget, and tombstone |
| Skill | immutable instructions/resources/scripts packages with revision and enablement policy |
| Tool | registered governed operations with explicit availability |
| Context | authorized, budgeted Task input and explicit losses/deltas |
| Task | raw intent, preview, admission, bounded execution, checkpoint, Effect, and verification |
| Runtime/Process | Agent package-through-execution identities and daemon-owned process observation |

### Adopted Personal 2.0 target

MCP becomes the true seventh family: installed servers, health, permissions,
updates, and projections into compatible Agent clients. The family layout in
the desktop product is task-oriented:

- **Library:** Memory, Skills, Tools, MCP;
- **Work:** Task and Context;
- **Agents:** Runtime/Process.

This navigation does not merge domains. Each family keeps its own identities,
storage, transitions, retention, and failure semantics. Budget, Permission,
Model, Artifact, Intent/Effect, Evidence, and Event remain cross-cutting
objects rather than additional families.

MCP is the adopted seventh product family and its implementation is
**Requires-backend**. A new or changed public MCP machine surface conditionally
requires P10-T02/Lane-CTR; a Personal-private projection may not. Linux 1.0
remains six families and does not inherit this target retroactively.

`CognitiveResourceManifest` remains an ActivityContext-filtered discovery
manifest. It is not the family catalog and grants neither read nor action.

## 4. Product principles

### Conversation first; governance by admission

A Native Conversation is the interaction source. It remains native until the
user chooses **Manage with Personal**, requests governance, and confirms the
daemon preview. The daemon then admits a persistent Goal, a Plan revision, and
one or more Tasks; each preserved attempt belongs to one Task. An Agent-authored
plan remains Native until the daemon admits it; fluent text never becomes
authority by presentation.

### Native behavior survives integration

Vendor adapters preserve each Agent's native harness and project a common core
plus a capability matrix and vendor extension slots. The native application
remains usable. Agent connection establishes an explicit observation scope;
automatic observation is limited to that scope, with no speculative/global
scan or surprise per-session enrollment. The owner may request governance and
confirm a daemon preview, but only the daemon admits authority.

The common/native conversation projection reuses or references existing Core
`Conversation` and `ConversationBinding` identities where applicable.
Vendor-native conversation IDs remain opaque origin bindings. Any additional
projection is Personal-private until P10-T02 decides otherwise.

### Preview once at the consequential boundary

The global Agent Shell may explain state, conflicts, and proposed recovery. For
a consequential action, the daemon creates the exact preview, the user confirms
that preview once, and the daemon executes. The Shell never receives authority,
and the product does not ask for repeated micro-approvals inside an unchanged
admitted scope.

### Content and connection never imply permission

Installing an Agent, enabling a Skill, selecting a model, discovering a native
resource, or connecting an MCP server grants no runtime capability. Workspace,
process, network, Memory, model, MCP, and write-back scopes remain separate and
revocable.

### Origin owns content; Personal owns governance

Vendor-native resources remain owned by their origin. Personal owns admitted
bindings, permissions, policy, synchronization intent, and authority records.
Adapters detect readable changes automatically only inside the explicit
observation scope established at Agent connection. Every write-back is a
daemon-owned Intent/Effect mutation. It may run automatically inside an
unchanged exact daemon grant/risk policy; new, broader, destructive, or
conflicted scope requires preview and confirmation. Conflicts fail closed and
invoke the Agent Shell for an explained resolution.

### Completion and progress stay honest

One merged timeline uses **Native / Observed / Governed / Verified** source
badges. These are provenance and authority labels, not confidence scores.
Effect reconciliation remains Governed; Verified is reserved for independent
verification and daemon acceptance.
Agent final text, Tool result, Provider response, native harness success, or
process exit is not completion. Percentages, counts, and ETAs appear only when
their denominator or basis is declared.

### Local and secret-safe by default

The daemon binds loopback. Provider and user secrets stay in approved Secret
Stores and daemon-mediated proxy profiles. ADR-0055 permits only user-directed,
per-source, daemon-owned, audited, non-logging credential import. Secret
material never appears in Agent configuration, ordinary config, SQLite, argv,
environment, logs, Context, Memory, evidence, or chat.

## 5. Workspace model

### Current implementation (Now)

The Standard Workspace and bounded Extended Home rules below are established
Personal boundaries.

### Standard Workspace

A Task selects a Standard Workspace as its default file boundary. Within that
boundary, policy may allow low-friction read/search and reversible write/patch
through registered Tools. Writes retain a recoverable journal and explicit
change projection.

### Bounded Extended Home

Extended Home is an explicit set of additional document/project roots,
purposes and allowed operations. It may also enable ordinary outbound network
access. It is previewed, remembered only by explicit choice and revocable.
Selecting one path does not grant its siblings or the full home directory. A
sidecar sees only the resolved paths and network policy admitted for its
current execution.

Extended Home hard-denies Secret Store contents, SSH/GPG keys, browser
credential/profile stores, CognitiveOS authority/bootstrap data, Docker and
system sockets, system directories, privilege elevation, service management
and package management. Publication, repository push, irreversible deletion
and other remote mutations remain exact typed operations with confirmation
where required.

### Adopted Personal 2.0 target

Agent onboarding presents the workspace choice together with Provider and
permission review. A Goal may span workspaces only through explicit bounded
entries. Federated native resources do not widen filesystem scope.

Broader workspace management and cross-session synchronization are
**Requires-backend**. A new public scope or permission form conditionally
requires P10-T02/Lane-CTR; a Personal-private projection may not.

## 6. Agent integration and conversations

### Current implementation (Now)

Pi is the Linux 1.0 qualified Agent/sidecar combination. Package,
installation, registration, instance, sidecar, execution, process, Shell
session, and native conversation remain distinct. The Control Plane can inspect
bounded Runtime and dsh projections, but has no full Agent lifecycle HTTP
surface and no embedded conversation/history surface. The native dsh panel is
separate.

### Adopted Personal 2.0 target

The **Agents** space is both the Agent roster and the place for adapter-backed
native conversations and history. Onboarding is at most three steps:

1. choose a signed upstream catalog record or **Connect existing**;
2. review Provider, workspace, and permissions once;
3. open the conversation; ready means the first real response arrived.

Catalog records expose source, version, digest, signature, license, and adapter
compatibility. Activation has two milestones: **first chat** and **first
governed/verified Task**. Removal always asks **Disconnect** or **Uninstall**.

The common adapter core, embedded conversations, catalog flow, native-session
observation/daemon admission, and lifecycle controls are **Requires-backend**.
The projection reuses Core Conversation/ConversationBinding; only a new public
extension conditionally requires P10-T02/Lane-CTR. See
[Agent integration and conversations](agent-integration-and-conversations.md).

## 7. Work model

### Current implementation (Now)

The current UI creates and inspects governed Tasks. Its Run is a presentation
composition from Task transitions, Effects, observations, evidence, and watch
facts; there is no first-class Run API. There are no Goal, Plan, Harness, or
common native-conversation projection APIs and no Task pause/cancel/retry HTTP
controls. Existing Core Conversation/ConversationBinding contracts are not a
Control Plane implementation.

### Adopted Personal 2.0 target

- **Native Conversation** is the interaction source.
- **Goal** is the durable outcome requested through **Manage with Personal**,
  confirmed by the owner, and admitted by the daemon. It may span sessions,
  Agents, and Tasks.
- **Plan** is daemon-owned and revisioned. Agent plans remain Native candidates
  until admission.
- **Task** is the bounded authority unit.
- **Attempt** belongs to one Task and preserves each execution or recovery
  branch. Retry or fork never erases the prior attempt.
- The daemon orchestrates a multi-Agent graph and explicit handoffs.
- The UI calls the operational reading an **execution flow** and an Agent's
  managed host an **Agent runtime engine**. Contract terms remain visible in
  inspectors.

Goal/Plan orchestration, attempt controls, multi-Agent graphs, and their public
projections are **Requires-backend**. New public machine semantics
conditionally require P10-T02/Lane-CTR; Personal-private projections may not.

## 8. Surfaces and local modes

| Surface | Current implementation (Now) | Adopted Personal 2.0 target |
|---|---|---|
| Control Plane `/ui/` | daemon-served desktop browser; current seven top-level spaces | primary desktop entry and supervisor with six target spaces |
| Global Agent Shell | no cross-Agent Control Plane Shell | persistent explainer and proposal layer; never authority |
| Native Agent app | remains independently usable | remains usable; explicit observation/admission only |
| Native dsh panel | separate `cognitive dsh web` surface | remains a native surface even when dsh conversations are adapter-projected |
| `cognitive` CLI | deterministic management and recovery | deterministic fallback to the same daemon authority |
| doctor/support | redacted facts and digests | linked recovery evidence, never secret-bearing |

Desktop, headless, and foreground modes use one daemon authority and application
services. UI attachment may differ; no mode gets a second writer.

## 9. Target information architecture

| Space | Primary job | Important contents | Dependency |
|---|---|---|---|
| **Home** | understand readiness, attention, and next action | first chat/goal entry, active Goals, blockers, stale state, recent verified outcomes | current basis exists; richer Goal/Agent composition Requires-backend |
| **Agents** | converse with and supervise Agents | roster, signed source, capability matrix, native conversations/history, Runtime, health, permissions, current handoffs | conversations and lifecycle Require-backend |
| **Work** | manage outcomes | Goal -> Plan revision -> Task -> Attempt hierarchy, Context, execution flows, Effects, evidence | hierarchy/control/inventory Require-backend; new public semantics conditionally require P10-T02 |
| **Library** | curate reusable resources for work | Memory, Skills, Tools, MCP | current resources exist; MCP implementation Requires-backend; public shape is conditional on P10-T02 |
| **Activity** | explain what happened | merged Native/Observed/Governed/Verified timeline with coverage | unified feed Requires-backend |
| **Settings** | configure Personal | Account Hub, Provider routes, System, workspace, permissions, backup/recovery | current custom OpenAI-compatible accounts/endpoints and Providers/System exist; OAuth/import/override hierarchy Require-backend |

Providers and System are nested in Settings. Context is inspected in Work;
Runtime is inspected in Agents. Neither becomes a Library family. Stable object
links and inspectors preserve cross-space navigation.

## 10. Controls and recovery

The adopted target control vocabulary is:

- interrupt current interaction;
- request Task pause/resume;
- cancel Task;
- detach observation without changing work;
- retry or fork from a checkpoint while preserving attempts;
- restart or recover the Agent runtime engine;
- offer compensating undo only when the daemon has a defined compensating
  operation. The UI never promises rollback of an irreversible action.

All controls are **Requires-backend** today. Until a typed daemon capability
exists, the UI shows an unavailable explanation rather than a disabled or fake
control. The global Agent Shell can explain and propose; only the daemon
previews and executes.

## 11. Capability-gap categories

| Category | Current gap | Product treatment |
|---|---|---|
| **Backend absent** | embedded conversation/history projection; Goal -> Plan revision -> Task -> Attempt and multi-Agent orchestration; Task controls; full Agent lifecycle HTTP; authority-backed Context/Runtime inventory; unified Activity; federated sync; MCP management | no affordance that implies availability; show the reason and dependency |
| **API or native surface exists, UI-dark/partial** | native dsh conversation surface is separate; Context authorization facts are not a complete Work inspector; several current projections cover only bounded facets | compose only declared facts, label source and coverage, and do not infer missing state |
| **Conditional contract/core gap** | only a new or changed public MCP/Goal/Plan/Run/Harness/conversation extension | use P10-T02/Lane-CTR; Personal-private projections may not require core changes and must reuse existing Core Conversation/ConversationBinding |

Existing Provider, resource, Task-evidence, backup/restore, and readiness
capabilities may be regrouped into the target IA without pretending that the
missing target capabilities already exist.

## 12. Linux 1.0 preservation and non-claims

Linux 1.0 remains a six-family, Pi-qualified product with Standard Workspace,
bounded Extended Home, one canonical local service, and the exact qualified Pi
adapter path. The current Control Plane is additive and non-blocking; the
native dsh panel remains separate. MCP, embedded conversations, non-Pi
qualification, and multi-Agent orchestration do not enter the 1.0 claim.

Personal continues to stabilize bounded Linux service, filesystem, process,
secret, package, and network ports. It does not include a kernel module, eBPF
control plane, device scheduler, custom kernel, or distributed authority.

Product outcomes should measure time to first real response, time to first
governed/verified success, recovery, conflict resolution, secret isolation,
and false-completion avoidance. A count, percentage, rate, or ETA is valid only
with a declared denominator and evidence boundary.

Formal thresholds, current task/Gate status, release evidence, and environment
qualification remain owned by the formal plan, preregistered campaigns, and
`PROGRESS.md`. See [Linux 1.0 scope](linux-1.0-scope.md).
