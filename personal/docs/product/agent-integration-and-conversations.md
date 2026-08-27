# Agent integration and conversations

- Status: adopted Personal 2.0 product target
- Canonical language: English
- Architecture:
  [Universal Agent Adapter Contract](../architecture/agent-adapter-contract.md) and
  [Multi-Agent orchestration](../architecture/multi-agent-orchestration.md)
- Related: [Web UI product design](web-ui-design.md),
  [User journeys](user-journeys.md), and
  [Account Hub](account-hub.md)
- Chinese translation:
  [agent-integration-and-conversations.zh-CN.md](agent-integration-and-conversations.zh-CN.md)

Personal 2.0 makes the Control Plane the desktop-primary entry and supervisor
for the owner's Agents while preserving each Agent's native conversation and
harness behavior. Integration never turns an Agent, adapter, or conversation
into an authority writer.

## 1. Reality ledger

| Boundary | Agent truth |
|---|---|
| **Current implementation (Now)** | Pi is the Linux 1.0 qualified Agent/sidecar path. `/ui/` has an Agent inventory and dossier with bounded Runtime/dsh facts, no lifecycle controls, and no embedded conversations. The native `cognitive dsh web` panel is separate. |
| **Adopted Personal 2.0 target** | Agents contains signed onboarding, connected-existing Agents, a common capability view, adapter-backed native conversations/history, Runtime supervision, owner Goal requests followed by daemon admission, handoffs, and removal choices. |
| **Requires-backend** | Catalog onboarding, common conversation/history projection, scoped native-session observation and daemon admission, full Agent lifecycle HTTP, Goal -> Plan revision -> Task -> Attempt orchestration, multi-Agent graph/handoffs, and target controls. |
| **Requires-core (conditional)** | Existing Core Conversation/ConversationBinding is reused. P10-T02/Lane-CTR is required only for a new public Agent capability, conversation extension, Goal, Plan, Run, Harness, attempt, or handoff machine contract; Personal-private projections may not require core changes. |

## 2. Product model

The **Agents** space is organized around an Agent dossier:

- signed source and installation/connection identity;
- adapter compatibility and capability matrix;
- current Provider/proxy profile and workspace scope;
- native conversations and history;
- Agent runtime engine, process observations, and health;
- current Goals, Plan revisions, Tasks, each Task's attempts, and handoffs;
- permissions, federated resources, Activity, and evidence;
- supported lifecycle and recovery actions.

Default language is plain: **conversation**, **execution flow**, and
**Agent runtime engine**. Package, installation, registration, instance,
sidecar, execution, process, session, epochs, digests, and raw redacted
projections remain available in inspectors. There is no Basic/Expert mode.

## 3. Adapter projection

Every vendor adapter preserves the native harness and projects:

### Common core

- Agent display identity and exact native/managed identity facts;
- source/version/adapter compatibility;
- native conversation list and selected conversation, when observable;
- current response/activity and health, when observable;
- Provider/profile, workspace, permission, and resource-binding facts the
  adapter can truthfully expose;
- supported lifecycle, conversation, Context, Tool, and synchronization
  capabilities;
- explicit unsupported, unknown, stale, and native-only facets.

Where applicable, the common projection reuses or references existing Core
`Conversation` and `ConversationBinding` identities. Vendor-native conversation
and thread IDs remain opaque origin bindings; they do not create a second
public Conversation model. Additional native/common projection state remains
Personal-private until P10-T02 decides otherwise.

### Capability matrix

The UI shows whether a capability is:

- native and directly supported by a vendor session API;
- supported through a managed adapter path;
- cooperative through MCP plus vendor rules;
- observable only;
- unavailable.

The matrix is descriptive, not a capability grant. It never implies host
session control from process liveness or MCP connectivity.

### Vendor extension slots

Vendor-specific concepts may appear in an extension inspector when they cannot
be faithfully mapped to the common core. Extensions preserve native semantics
and source labels. They cannot override daemon authority or disguise missing
common behavior.

## 4. Native conversation as the interaction source

A conversation begins and remains **Native** unless the user requests
**Manage with Personal**, confirms the daemon preview, and the daemon admits
the governed outcome.

- The embedded view uses a vendor-native session API when available.
- The native Agent application remains usable at all times.
- The current native session and opaque vendor ID remain distinct from the
  Core Conversation/ConversationBinding-backed Personal projection, Goal,
  Task, Agent runtime engine, and process.
- A native Agent plan remains Native, even when displayed in Work.
- Adapter observation creates an **Observed** fact, not a governed Task,
  permission, Memory, or completion.

### Manage with Personal

The action **Manage with Personal**:

1. identifies the selected native conversation and desired outcome;
2. asks the daemon to preview a persistent Goal;
3. lets the owner confirm that exact consequential preview;
4. lets the daemon admit the Goal and establish the daemon-owned Plan revision;
5. lets the daemon create one or more governed Tasks, each with its own
   preserved attempts;
6. binds only the admitted Context, Agent, workspace, Provider/profile,
   permissions, budget, and acceptance criteria;
7. preserves a source link back to the native conversation without copying
   secret material or inventing native authority.

A Goal may span conversations, sessions, and Agents. The daemon owns the
multi-Agent graph and handoffs. Agents do not transfer leases, permissions, or
completion authority to one another.

This target is **Requires-backend**. New public machine semantics conditionally
require P10-T02/Lane-CTR; Personal-private projections may not.

## 5. Observed native sessions

Native Agent use remains possible outside Personal-managed execution. Agent
connection establishes an explicit observation scope. An adapter may
automatically observe supported native sessions only inside that scope.

Observation rules:

- the exact source and observation scope are visible and authorized when the
  Agent is connected;
- there is no speculative/global session scan and no surprise per-session
  enrollment;
- only capabilities the adapter can truthfully read are shown;
- an observed session is never automatically governed;
- a native plan, Tool result, process exit, or final text is never promoted to
  Task completion;
- **Manage with Personal** is the only product request from observation toward
  a new governed Goal; the owner confirms the preview and the daemon alone
  admits it;
- unsupported native sessions remain native-only rather than being controlled
  through guesses or process signals.

## 6. Agent onboarding in no more than three steps

### Step 1 — choose source

Choose either:

- a signed upstream catalog record; or
- **Connect existing**.

Every catalog record shows source, version, digest, signature, license, and
adapter compatibility. A catalog listing grants no permission and transfers no
qualification evidence.

### Step 2 — one review

Review Provider/proxy profile, Standard Workspace, and requested permissions in
one place. Optional detail stays in inspectors. The user may deny or narrow
permission and preserve a native-only path when safe.

### Step 3 — first conversation

Open the embedded native conversation. **Ready** means the first real response
arrived. Installed bytes, process health, adapter handshake, model discovery,
or a synthetic probe alone is not ready.

### Activation milestones

1. **First chat** — a real native conversation response.
2. **First governed and verified Task** — daemon-admitted work with current
   independent verification and reconciled Effects.

The product does not collapse these milestones into one readiness badge.

## 7. Runtime and lifecycle

Package, installation, registration, instance, sidecar, execution, process,
native session, Core Conversation/ConversationBinding-backed Personal
projection, Goal, Plan revision, Task, and Task-owned attempt remain distinct.
Co-location or shared bytes does not merge identity, permission, epoch, or
completion.

The adopted target controls are:

- interrupt current conversation interaction;
- request Task pause/resume;
- cancel Task;
- detach observation without changing work;
- retry/fork from checkpoint into a preserved attempt;
- restart/recover the Agent runtime engine;
- disconnect or uninstall the Agent.

These controls are **Requires-backend** today. Current `/ui/` must continue to
explain their absence rather than render false controls.

## 8. Disconnect versus uninstall

Every removal flow asks:

- **Disconnect** — remove Personal management/observation bindings while
  preserving the native installation and native data;
- **Uninstall** — remove the Personal-managed installation after a daemon
  impact preview and lifecycle procedure.

The preview distinguishes conversations, Goals, Plan revisions, Tasks,
Task-owned attempts, Agent runtime engines, pending Effects, bindings, and
retained data. Governed history remains unless a separate retention/purge
action is explicitly confirmed. A receipt states removed, retained, unknown,
and incomplete outcomes.

## 9. Multi-Agent work and handoffs

For a daemon-admitted Goal, the daemon may schedule multiple independently
supported Agents:

- each Task owns its attempts, and each attempt binds one exact
  Agent/runtime/epoch;
- handoffs are explicit events with source and target;
- downstream governed work waits when an upstream handoff fails;
- Agent disagreement is shown as Native/Observed proposals until the daemon
  admits a Plan decision;
- shared resources are reauthorized for each Task/body;
- no Agent can grant another Agent permission or acceptance.

Multi-Agent is an adopted Personal 2.0 target and **Requires-backend**. It is not
a Linux 1.0 or non-Pi qualification claim.

## 10. Timeline and completion

Conversation and work views share one timeline grammar:

| Badge | Meaning |
|---|---|
| **Native** | vendor Agent/session content or plan |
| **Observed** | adapter/daemon observation not admitted as authority |
| **Governed** | daemon admission, authorization, mutation, and Effect reconciliation |
| **Verified** | current independent verification and daemon acceptance only |

Badges are provenance/authority labels, not progress. No fake percentage or ETA
is derived from model text. Counts appear only with a declared denominator.
Agent final text, native harness result, Tool result, Provider response, or
process exit is not completion.

## 11. Required states

| State | Required treatment |
|---|---|
| Empty | signed catalog/connect-existing action and native-only explanation |
| Loading | name catalog, adapter, conversation, runtime, or governed source being loaded |
| Partial | show supported common core and list unavailable native facets |
| Permission | exact Provider/workspace/resource/native-session scope with deny/narrow path |
| Error | preserve source/review/conversation context and offer supported recovery |
| Stale | show last observation time and prevent unsafe inference/action |
| Conflict | fail closed, invoke Agent Shell explanation, require daemon preview for resolution |
| Success | distinguish first chat from first governed/verified Task |

## 12. Backend Capability Gaps

### Backend absent

- signed catalog onboarding and connect-existing workflow;
- common conversation/history projection reusing Core ConversationBinding;
- connection-scoped native-session observation and daemon admission;
- full Agent lifecycle over the Control Plane;
- Goal -> Plan revision -> Task -> Attempt and multi-Agent graph/handoff orchestration;
- interrupt/pause/resume/cancel/retry/fork/restart/recover controls;
- general federated resource synchronization.

### API/native surface exists, UI-dark or partial

- The native dsh panel is an existing separate interaction surface.
- Current Runtime and dsh projections provide bounded Agent facts but not the
  target conversation or lifecycle model.
- Existing Provider bindings and Task evidence can be linked into the dossier
  but do not fill the missing Agent semantics.

### Contract/core gap

The Personal-private projection reuses existing Core Conversation and
ConversationBinding. Only a new or changed public common capability,
conversation extension, Goal, Plan, Run, Harness, attempt, or handoff machine
surface conditionally requires P10-T02/Lane-CTR.

## 13. Fixed boundaries and non-claims

- The daemon is the sole authority writer.
- Native app use is preserved; observation and governance are explicit.
- MCP plus rules cannot control a host Agent session.
- No Agent, adapter, Shell, or native plan can self-admit, widen permission,
  commit Effects, or accept completion.
- Pi remains the Linux 1.0 qualified Agent path; other Agents need independent
  qualification.
- This target makes no implementation, Gate, release, Profile, performance,
  containment, or Agent-benefit claim.
