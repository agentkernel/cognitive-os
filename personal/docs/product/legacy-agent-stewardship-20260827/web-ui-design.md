# CognitiveOS Personal Web UI product design

- Status: current P7-T05 surface plus adopted Personal 2.0 target
- Current implementation: `clients/pc/web/`, daemon-served at `/ui/`
- Product boundary: owner-local, desktop-primary Personal
- Change class: `product-semantic + structural` documentation
- Related: [Product design](product-design.md),
  [Agent integration and conversations](agent-integration-and-conversations.md),
  [Account Hub](account-hub.md), and [User journeys](user-journeys.md)

[ADR-0053](../../../../docs/adr/0053-personal-web-ui-stack.md) establishes the
React + TypeScript + Vite client at `clients/pc/web/`, same-origin daemon
serving, and memory-only browser sessions. P7-T05 delivered that Control Plane;
its closure record is
[here](../../../../docs/checkpoints/20260826-personal-p7-t05-control-plane-redesign-closure.md).
Current task and evidence facts remain in
[PROGRESS.md](../../../../docs/plan/PROGRESS.md).

## 0. Reality ledger

| Boundary | Web UI truth |
|---|---|
| **Current implementation (Now)** | `/ui/` has Home, Work, Agents, Providers, Resources, Activity, and System. It provides governed Task creation/inspection, Provider and resource management, Agent dossier projections, composed Activity, and stewardship. |
| **Adopted Personal 2.0 target** | Desktop-primary Control Plane with Home, Agents, Work, Library, Activity, and Settings; adapter-backed conversations; global Agent Shell; durable Goal -> Plan revision -> Task -> Attempt supervision; Account Hub; MCP; and federated resources. |
| **Requires-backend** | Embedded conversations/history, Goal -> Plan revision -> Task -> Attempt and multi-Agent orchestration, full controls, authoritative Context/Runtime inventory, unified Activity, federated sync, new account methods, and MCP management. |
| **Requires-core (conditional)** | Existing Core Conversation/ConversationBinding is reused. P10-T02/Lane-CTR is required only for a new or changed public MCP/Goal/Plan/Run/Harness/conversation extension; Personal-private projections may not require core changes. |

## 1. Product outcome

The Control Plane lets the owner:

1. start or continue a real native Agent conversation;
2. decide when an outcome should become a governed Goal;
3. supervise Plan revisions, Tasks, attempts, Agent handoffs, Context, Effects,
   and independent verification;
4. curate federated Memory, Skills, Tools, and MCP resources;
5. manage accounts, routing, permissions, runtime engines, and system recovery;
6. understand what happened from a source-labelled timeline.

The browser remains an untrusted client. It makes daemon and adapter facts
legible and submits typed requests. It never stores Provider secrets, writes
authority data, controls a native host session through inference, or accepts
completion.

## 2. User and entry conditions

The target user is one owner, including a user new to governed Agents. The
default path is beginner-first: concrete labels, safe defaults, and one clear
next action. Full authority detail stays in inspectors. There is no
Basic/Expert mode.

Desktop is the primary layout. The Control Plane attaches to the loopback
daemon and is not a remote console, multi-user RBAC product, or public Internet
service. It remains useful when Provider, SecretStore, Agent, adapter, native
panel, MCP server, or model execution is unavailable: readiness, known facts,
staleness, and deterministic recovery remain visible.

## 3. Information architecture

The target shell uses stable desktop navigation, a persistent health/session
strip, a global Agent Shell, master/detail views, and inspectors. The six
top-level spaces are:

**Home / Agents / Work / Library / Activity / Settings**

| Target surface | Current implementation (Now) | Adopted Personal 2.0 target | Dependency |
|---|---|---|---|
| **Home** | readiness, attention, current Task composition, alerts, recent evidence | resume conversation/Goal, triage blockers and conflicts, start first chat or governed work | richer Goal/Agent composition Requires-backend |
| **Agents** | Runtime inventory/dossier and bounded dsh projection; no lifecycle buttons | roster, signed source, adapter capability matrix, embedded native conversations/history, Runtime, handoffs, permissions, health | conversations/catalog/lifecycle Require-backend |
| **Work** | Task inventory/creation/detail; Run is a composed reading | Goals, daemon Plan revisions, Tasks, each Task's attempts, Context, execution flows, multi-Agent graph, Effects, evidence | Goal/Plan/control/inventory Require-backend; only new public semantics conditionally require P10-T02 |
| **Library** | current Resources hub with Memory, Skills, Tools, and Context link to Work | Memory, Skills, Tools, and MCP; federated origin and sync state | MCP/federation Require-backend; only new public semantics conditionally require P10-T02 |
| **Activity** | provider audit plus session-known Task facts, explicitly incomplete | one merged Native/Observed/Governed/Verified timeline with coverage and object links | unified feed and durable live updates Require-backend |
| **Settings** | Providers and System are top-level peers | Account Hub, System, workspace, permissions, backup/recovery, session stewardship | regrouping uses current APIs; new account methods Require-backend |

Providers are accounts and routes inside Settings, not a resource family.
Context belongs in Work; Runtime belongs in Agents. Existing deep links should
redirect or preserve selection when navigation is regrouped.

### 3.1 Global Agent Shell

The global Agent Shell is available from every target space. It:

- answers "what is happening, why is this blocked, and what can I do next?"
  from declared daemon and adapter facts;
- explains sync conflicts and unavailable controls;
- proposes an action in the current object context;
- asks the daemon to issue a consequential preview;
- returns focus to the affected object and durable receipt.

It never owns credentials, authority, policy, Tool dispatch, write-back, or
completion. A suggestion is not a preview; only the daemon preview can be
confirmed. One confirmation covers the exact consequential action and scope,
not unrelated future actions.

Embedded/common conversation views reuse or reference existing Core
`Conversation` and `ConversationBinding` identities where applicable.
Vendor-native IDs are opaque origin bindings, and additional projection fields
remain Personal-private (ADR-0058); they are not a public Core schema.

### 3.2 Page-fact rules

These are display requirements, not new public schemas:

- every fact carries its origin, authority level, freshness, and unknown or
  not-run meaning when available;
- the first reading uses plain terms; stable IDs, versions, digests, epochs,
  raw redacted projection, and policy detail remain in an inspector;
- default labels are **execution flow** and **Agent runtime engine**;
- raw prompts, completions, headers, keys, bearer tokens, resolvable SecretRefs,
  and unbounded process output are never required display facts;
- counts, percentages, rates, and ETAs appear only with a declared denominator
  and basis.

## 4. Core journeys and interaction model

### 4.1 Five-minute first chat

The first screen offers a signed upstream Agent catalog or **Connect existing**.
The target flow has at most three steps:

1. select the Agent source;
2. review Provider, workspace, and permissions together;
3. open its embedded native conversation.

The Agent becomes **chat-ready** only when a real response arrives. Install,
process health, model discovery, or a synthetic probe is not that milestone.
The second activation milestone is the first governed and independently
verified Task.

This target is **Requires-backend**. Current `/ui/` can inspect Agents but does
not install/connect them or embed conversations.

### 4.2 Native conversation to governed work

Conversation begins as **Native**. Choosing **Manage with Personal**:

1. identifies the selected conversation and outcome;
2. asks the daemon to preview a persistent Goal;
3. presents the daemon preview for owner confirmation;
4. lets the daemon admit the Goal and first Plan revision;
5. creates bounded Tasks and preserves attempts under their owning Task;
6. shows Agent handoffs, Context, Effects, and evidence in Work.

A Goal may continue across sessions and Agents. An Agent-authored plan remains
Native until admitted. Nothing is auto-promoted because an adapter observed it.
Goal/Plan/Conversation projections and multi-Agent orchestration are
**Requires-backend**. New public machine semantics conditionally require
P10-T02/Lane-CTR; Personal-private projections may not.

Agent connection establishes the explicit observation scope for native
sessions. Automatic observation is limited to that scope; there is no
speculative/global session scan or surprise per-session enrollment.

### 4.3 Accounts and routing

Settings opens Account Hub. The target first screen offers OpenAI, Anthropic,
Google, and DeepSeek, followed by Qwen/Bailian, Kimi, Zhipu, SiliconFlow,
Volcengine-Doubao, MiniMax, OpenRouter, and a first-class custom
OpenAI-compatible endpoint.

The target methods are subscription/OAuth, API key, ADR-0055 import of an
existing credential, and custom endpoint. Custom OpenAI-compatible
account/endpoint support is already **Current implementation (Now)**; broader
OAuth/import/preset and override behavior is missing. All Personal-managed
paths terminate in daemon SecretStore custody and a daemon proxy profile.
Routing precedence is global default, Agent override, then conversation
override. A current native session changes only after explicit rebind/restart;
it never switches silently.

Current Provider accounts, API keys, models, fixed Agent bindings, usage,
budgets, alerts, and audit are implemented. The broader methods and override
hierarchy are **Requires-backend**. See [Account Hub](account-hub.md).

### 4.4 Execution flow, attempts, and controls

Work shows one Goal's Plan revisions, Tasks, each Task's attempts, Context,
Effects, artifacts, and verification without inventing a first-class Run
entity. The default label is **execution flow**; inspectors can identify the
exact daemon records that compose it.

The adopted controls are:

- interrupt the current interaction;
- request Task pause/resume;
- cancel Task;
- detach observation, which never changes work;
- retry or fork from a checkpoint into a preserved new attempt;
- restart or recover the Agent runtime engine;
- compensating undo only where a real daemon compensation exists.

These controls are **Requires-backend** today. Current `/ui/` has no Task
pause/cancel/retry and no full Agent lifecycle HTTP surface. It must continue
to show explanatory unavailable text rather than fake or disabled controls.

### 4.5 Federated resources and conflicts

Library and object inspectors show origin, native identity, Personal binding,
permission, sync freshness, and conflict state. Adapters may automatically read
and detect changes only inside the explicit observation scope established at
Agent connection. Every write-back is a daemon Intent/Effect operation. It may
run automatically inside an unchanged exact daemon grant/risk policy; new,
broader, destructive, or conflicted scope requires preview and confirmation.
A conflict fails closed; the global Agent Shell explains it and proposes a
family-specific resolution.

Bidirectional synchronization and conflict workflows are
**Requires-backend**. Current resource operations remain Personal-owned and do
not imply federation.

### 4.6 One merged timeline

Activity and per-object timelines use four source badges:

- **Native** — vendor Agent or native session;
- **Observed** — adapter/daemon observation without admission;
- **Governed** — daemon admission, authorization, mutation, and Effect reconciliation;
- **Verified** — current independent verification and daemon acceptance only.

The current Activity page is a partial composition and must keep its coverage
statement. A complete cross-domain feed is **Requires-backend**. Agent final
text, Tool result, process exit, or Provider response never receives a Verified
badge by itself.

## 5. Interaction and visual rules

The visual language is **Calm, Dense, Precise, Professional**.

- Use stable sidebar navigation, compact rows, master/detail layouts, and a
  persistent inspector. Density comes from alignment and hierarchy, not tiny
  type or hidden controls.
- Use restrained color, material, radius, shadow, and motion. This is not
  glassmorphism, Liquid Glass, a marketing site, a wall of rounded cards, a
  purple AI gradient, or an ornamental KPI dashboard.
- Home is an attention surface. It leads with readiness, blockers, conflicts,
  current Goals/Tasks, and next actions rather than hero copy or metric cards.
- Beginner language is always the default. Governance details expand in place
  or in the inspector; the layout does not switch modes.
- Status uses text plus shape/icon where useful, never color alone. Native,
  Observed, Governed, and Verified badges remain visually distinct without
  implying a percentage.
- Lists are sortable and filterable when the data supports it. Stable identity,
  source, freshness, and timestamps are copyable without exposing credentials
  or session bearers.
- Motion provides immediate feedback and spatial continuity only. It is short,
  interruptible where interactive, and replaced by non-motion feedback under
  `prefers-reduced-motion`.
- Desktop is primary. Narrow windows preserve current location, the primary
  action, forms, tables, conversation, and recovery without claiming a separate
  mobile product.
- Keyboard operation, visible focus, semantic landmarks/forms/tables, live
  status announcements, and color-independent states are required.

### 5.1 State system

Every surface defines:

| State | Required answer |
|---|---|
| Empty | Why is it empty, and what concrete action creates value? |
| Loading | What is being read or changed, what remains stable, and can the user safely leave? |
| Partial | Which source or facet is missing, and what still works? |
| Stale | How old is the fact, why is refresh needed, and what actions are unsafe meanwhile? |
| Permission | What exact scope is blocked, why is it needed, and can the user deny or choose a narrower path? |
| Error | What failed, what input/work was preserved, and what is the next safe action? |
| Success | What changed, where is the durable receipt, and what should the user do next? |
| Long-running | Which Plan/Task/attempt is active, what facts changed, and which controls genuinely exist? |

A disconnected watch never fabricates progress or completion. A disabled
control must not stand in for a capability that does not exist.

### 5.2 Consequential actions

The daemon supplies exact targets, versions, permissions, external effects,
reconciliation/compensation expectations, and idempotency identity. The user
confirms that consequential preview once. The browser does not assemble its own
authority preview and the Agent Shell cannot bypass the confirmation.

Provider keys, resolvable SecretRefs, bootstrap/session tokens, raw Provider
headers, and unbounded sensitive content are never rendered.

## 6. Backend Capability Gaps

### 6.1 Backend absent

| Gap | Current UI behavior | Adopted target |
|---|---|---|
| Embedded Agent conversations/history | absent; native dsh panel is separate | adapter-backed native conversation inside Agents |
| Goal -> Plan revision -> Task -> Attempt and multi-Agent graph | absent | durable Goal, daemon Plan revisions, Tasks with preserved attempts, and handoffs |
| Task controls | no pause/cancel/retry HTTP capability | interrupt, pause/resume request, cancel, retry/fork |
| Full Agent lifecycle | no Control Plane lifecycle API | install/connect, lifecycle, restart/recover, disconnect/uninstall |
| Context/Runtime inventory | bounded or projection-only facets | authority-backed inspectors in Work and Agents |
| Unified Activity/live updates | partial composition and bounded watch | cross-domain merged timeline with durable coverage/freshness |
| Federated resources | absent | automatic read/change detection and guarded bidirectional write-back |
| Account methods/overrides | API key and custom OpenAI-compatible accounts/endpoints plus fixed Agent binding | subscription/OAuth, credential import, broader presets, and global/Agent/conversation scopes |
| MCP family | absent | server lifecycle, health, permissions, updates, and client projection |

### 6.2 API exists, UI-dark or partially composed

- The native dsh panel can host its own interaction, but it is not a Control
  Plane conversation/history projection.
- Context authorization/revocation facts exist, but the current UI has no
  complete authority-backed Context inventory.
- Existing Provider, resource, Task-evidence, dsh runtime, readiness, and
  backup/restore APIs cover meaningful target pieces. Regrouping those pieces
  does not fill Goal, conversation, lifecycle, federation, Activity, or MCP
  gaps.

### 6.3 Contract/core gap

- MCP implementation is **Requires-backend**. Only a new/changed public MCP
  machine surface conditionally requires P10-T02/Lane-CTR.
- Existing Core Conversation/ConversationBinding is reused. Any new public
  conversation extension, Goal, Plan, Run, Harness, attempt, or cross-Agent
  handoff machine semantics conditionally require the same contract lane;
  Personal-private projections may not.
- This document deliberately names no proposed route, JSON envelope, database
  table, transition, or error code.

## 7. Fixed boundaries and non-claims

- The product origin is daemon-served `/ui/` on loopback. A Vite preview or the
  separate native dsh panel is not a substitute.
- The browser has no direct SQLite, SecretStore, filesystem, shell, host
  process, or Provider network authority.
- Remote/public access, multi-user/RBAC, organization tenancy, HA, and cloud
  authority remain outside this owner-local scope.
- Native app use is allowed, but Native or Observed activity is not silently
  governed.
- Detach is observation-only. Agent output, Tool results, process exit, or
  Provider response is not completion.
- Linux 1.0 remains six-family and Pi-qualified. The Control Plane and adopted
  Personal 2.0 target do not alter its Gate composition.
- P7-T05 implementation and rendered-review evidence do not establish a Gate,
  release, Profile, performance, containment, or Agent-benefit claim. The same
  non-claim applies to this adopted target.
