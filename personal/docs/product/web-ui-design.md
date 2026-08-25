# CognitiveOS Personal Web UI Product Design

- Status: informative post-1.0 target/design
- Formal task: `P7-T05` (non-blocking Web UI)
- Product boundary: local single-owner Personal; Linux 1.0 does not require this surface
- Change class: `product-semantic + structural` documentation
- Related: [Product design](product-design.md), [Provider Control Plane](provider-control-plane.md), [User journeys](user-journeys.md)

This document defines the operator experience for a future Web UI. It does not
claim that a Web UI, its API surface, a Gate, a release or a Profile is
implemented. Current facts remain in [PROGRESS.md](../../../docs/plan/PROGRESS.md).
[ADR-0053](../../../docs/adr/0053-personal-web-ui-stack.md) accepts React + TypeScript +
Vite in `cognitiveos-clients/pc/web/`, same-origin daemon serving, and
memory-only sessions. The SPA is implemented in the approved checkout
`D:\cognitiveos-clients\pc\web\`, not in this repository.

## 1. Product outcome

The Web UI gives the owner one local, read-first place to answer four questions:

1. Which Agents are installed, registered, healthy and currently usable?
2. Which Provider accounts and models are configured, and can the daemon reach them?
3. Which fixed Provider binding does each Agent use?
4. What is running now, what changed, what is blocked and what is actually verified?

The UI is an operator client. It makes daemon facts legible and submits typed
management or Task actions; it never becomes a second runtime or authority
writer.

## 2. Target user and entry conditions

The target user is one technically capable owner operating Personal on the same
machine as the daemon. The first delivery is a desktop browser attached to a
loopback daemon. The UI is not a remote administration console, a multi-user
RBAC surface or a public Internet service.

The entry screen must remain useful when Provider, SecretStore, Pi or an Agent is
unavailable. Readiness, doctor facts and deterministic recovery links are
rendered even when model execution is disabled.

## 3. Information architecture

The shell uses a compact left navigation and a persistent status strip. The
canonical Personal top-level spaces remain Home, Agents, Tasks, Resources and
Activity. Provider management is a dedicated operator view/shortcut reachable
from Home and Agents (and may be grouped under Resources in navigation); it is
not a sixth resource family or a replacement for the stable information
architecture. Pages are projections, not new domains:

| Surface | Primary question | Main projection |
|---|---|---|
| Home | Is Personal ready and what needs attention? | system, database, SecretStore, Provider, daemon, Agent health, current runs, alerts |
| Agents | What is installed and how is each Agent governed? | package, installation, registration, instance, sidecar, process, bindings, permissions, health |
| Provider management shortcut | Which accounts/models are available and reachable? | account metadata, endpoint trust, SecretStore resolution state, catalog, capability snapshot, last probe, usage and alerts |
| Tasks | What work was requested and what is its authority state? | raw intent, preview digest, bindings, Context, budget, Loop, checkpoint, verification |
| Activity | What is running or needs investigation? | Run, Process, Effect, Evidence and Event projections with cursor-based updates |

Memory, Skills, Tools and Context remain available through the existing
Resources area and through related links from Agent, Task and Activity views.
Provider, Model, Budget and Permission are cross-cutting facts; they do not
become additional resource families.

### 3.1 Minimum page facts

The following are display requirements, not new public schemas. Each value must
come from an approved daemon projection and retain its source, version and
unknown/not-run meaning.

| Page | Minimum facts visible to the owner |
|---|---|
| Home | readiness component, overall readiness, blocked reason, current run count, active alerts, last successful watch cursor |
| Agents | stable instance ID, package/installation/registration digests, adapter/protocol digest, lifecycle and health, binding status, current execution/Task, allowed actions |
| Provider management | account ID/name/kind, redacted endpoint, endpoint trust, SecretStore resolution state, catalog revision, selected model, capability flags, last probe result/duration/error class, usage/cost state |
| Task/Run | Task and execution IDs, raw intent reference, preview/admission digest, state, budget, Context digest/loss, checkpoint, pending/unknown Effects, verification disposition |
| Activity | event/process/effect/evidence identity, timestamp, state, bounded output reference, cursor, source and reconciliation status |

Raw prompts, completions, headers, keys, bearer tokens and unbounded process
output are never required display facts.

## 4. Core journeys

### 4.1 Inspect installed Agents

The Agents list is a dense table with stable Agent instance ID, display name,
adapter/protocol digest, installation version, lifecycle state, health, Provider
binding state, current Task count and last observation time. The UI must keep
package, installation, registration, instance, sidecar, execution and process
identities visibly distinct.

The detail page has tabs for Overview, Binding, Runs, Process and Evidence. A
healthy process is an observation, not proof of a completed Task. Drift, stale
epoch, missing binding, disabled capability and unknown outcome are explicit
badges with a next action.

### 4.2 Add or configure a Provider account

The wizard is ordered as `validate -> trust confirmation (when required) ->
persist intent -> secret input -> store/rotate secret -> connectivity/model
probe -> verify`. The browser never receives or stores the API key. Secret entry
is sent once through the daemon's approved management path and is never placed
in URL parameters, browser storage, analytics, logs or error text.

The account page exposes provider kind, redacted endpoint, endpoint trust,
SecretStore resolution state, catalog revision, selected model, last probe time,
capability flags and stable error class. It offers explicit rotate, refresh
models and remove actions. An active Agent binding prevents account deletion and
is shown before confirmation.

Connectivity testing has two levels:

- **Reachability probe**: bounded endpoint and credential validation with a
  redacted status, duration and error class.
- **Capability/model probe**: explicit model discovery and, where the Provider
  contract permits, a bounded capability check. It is never implied by a
  successful TCP/TLS connection.

Failed refresh preserves the last known catalog and existing binding. Unknown
or unavailable fields remain unknown; the UI must not display them as zero or
ready.

The account state vocabulary is rendered from the daemon's exact status and
error classes, at minimum making usable, degraded, revoked,
locked/unresolvable and unknown outcomes distinguishable when those facts are
available. These are display groupings, not new authority states. A successful
network connection alone never upgrades an account to usable.

### 4.3 Bind a Provider to an Agent

The Agent Binding panel shows the current fixed `account + provider + model`,
binding revision and status. A change preview names the exact Agent instance,
account ID, model ID, expected revision, trust state and consequences for
running work. Admission is required for the mutation.

There is no fallback, automatic routing or per-request Provider override. An
unbound, revoked or degraded binding is visibly non-callable and links to the
repair action. Agents never read the SecretStore directly.

### 4.4 Manage and inspect runs

Task and Run views distinguish request receipt, proposed, awaiting admission,
queued, running, waiting, suspended, blocked, reconciling, verifying,
completed, failed, cancelled and quarantined states. The owner can attach or
detach observation without changing execution.

Typed controls are available only when the daemon projection says they are
allowed:

- pause/suspend after the daemon's safe-checkpoint protocol;
- resume after reload, fencing, reauthorization and Context rebuild;
- cancel a Task (not equivalent to killing a process);
- stop, restart or quarantine an Agent instance through its lifecycle workflow;
- open the related Process, Effect, Evidence and Event records.

Destructive or externally mutating actions show a server-issued preview, stable
target IDs, expected versions, idempotency identity, budget impact and rollback
or reconciliation expectation before confirmation.

## 5. Interaction and visual rules

- The P7-T05/D10 operator surface uses an Apple-inspired, system-like visual
  direction within the existing Personal design language: generous whitespace,
  deliberate brand typography, quiet cool neutrals, translucent local depth,
  restrained separators and one clear visual hierarchy. It does not use purple
  "AI" gradients, cream/editorial styling, card walls, icon walls or ornamental
  dashboard strips.
- The first viewport is one composed product statement plus the most useful
  authority summary, not a grid of interchangeable metrics. Hero content is
  never placed in a card. Functional forms, tables and projections follow as
  flat sections with one title, one short explanation and a clear primary
  action.
- Lists use stable rows and details use a bounded fact hierarchy. Loading,
  authoritative-empty, denied, disconnected, unknown and not-run states retain
  explicit text and cannot be represented by color alone.
- Motion is limited to short page/context entry, control feedback and one
  ambient depth cue; `prefers-reduced-motion` removes non-essential motion.
  Narrow desktop/mobile-width layouts keep navigation, forms, tables and
  actions operable without claiming a separate mobile product.
- Read views are dense, sortable and filterable; stable IDs and timestamps are
  copyable without exposing credentials or session bearers.
- Every long-running view has loading, empty, stale, disconnected and
  permission-denied states. A disconnected watch never fabricates a final state.
- Tables link to a detail drawer/page rather than nesting unrelated cards. A
  timeline groups related Task, Process, Effect and Evidence facts without
  merging their identities.
- Provider keys, SecretRefs, session tokens, raw prompts/completions and raw
  Provider headers are never rendered.
- Confirmation copy uses exact stable names and consequences; it does not imply
  that an Agent, Provider response or process exit completed a Task.
- Desktop is the primary layout. Narrow windows remain usable for inspection,
  but the first delivery does not promise a mobile-specific workflow.
- Accessibility target is keyboard-complete operation, visible focus, semantic
  tables/forms, status announcements for watch updates and a color-independent
  state vocabulary.

## 6. Scope

### Included in P7-T05

- localhost browser shell and authenticated management/task sessions;
- Home readiness and doctor projection;
- installed Agent inventory and lifecycle detail;
- Provider account creation, rotation, removal, model refresh and explicit
  connectivity/capability probes;
- fixed Agent Provider binding and revision-aware change preview;
- Task, Run, Process, Effect, Evidence and Event observation;
- typed pause/resume/cancel/stop/restart/quarantine actions where existing
  daemon services expose them; if an operation is not exposed by an approved
  typed service, the UI renders it as unavailable rather than inventing a
  generic lifecycle route;
- redacted usage, cost state, soft-budget alerts and audit links;
- reconnectable watch with stale/unknown/not-run handling.

### Deferred or excluded

- remote or public access, multi-user/RBAC, SSO/OAuth and organization tenancy;
- direct SQLite, SecretStore or filesystem access from the browser;
- browser-side authorization, budget enforcement, completion or reconciliation;
- automatic Provider fallback, routing, load balancing or arbitrary headers;
- Agent marketplace, package acquisition policy or non-Pi qualification;
- Multi-Agent orchestration, MCP/dynamic Tool management and Web UI evidence
  promotion;
- changing Linux 1.0 release scope or any Gate definition.

## 7. Success and non-claims

P7-T05 is complete only when the supported client readiness checks, security
negatives, API compatibility, core journeys, accessibility checks and bounded
performance checks pass on the declared client route (daemon-served `/ui/` on
loopback, exact revision). A polished screen, fixture, local smoke test or
ordinary CI result is not a Gate, release, Profile or Agent-benefit claim.

Pause, resume, cancel, stop, restart and quarantine remain visible only when a
typed daemon HTTP service exists. The D01 inventory records Task cancel and
Agent lifecycle HTTP as unavailable/not-run; the UI must not invent those
routes. Detaching observation must not cancel a Task or stop an Agent.
