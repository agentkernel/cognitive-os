# CognitiveOS Personal 2.0 scope

- Status: adopted product-semantic target; implementation remains capability-gated
- Date: 2026-08-27
- Current-status owner: [PROGRESS.md](../../../docs/plan/PROGRESS.md)
- Release boundary: [Linux 1.0 scope](linux-1.0-scope.md)
- Decision carriers:
  [ADR-0041](../../../docs/adr/0041-personal-axiom-system-revision.md)–[ADR-0045](../../../docs/adr/0045-personal-os-positioning.md),
  [ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md),
  [ADR-0056](../../../docs/adr/0056-personal-2-0-desktop-control-plane.md), and
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md)

This document owns the adopted Personal 2.0 product boundary. It does not own
task status, Gate results, release claims, Profile conformance, or public
machine contracts.

## 1. Reality ledger

| Boundary | Scope |
|---|---|
| **Current implementation (Now)** | Linux 1.0 is six-family and Pi-qualified. The daemon-served `/ui/` has Home, Work, Agents, Providers, Resources, Activity, and System. Activity is a labelled composition, not a unified feed. The native dsh panel is separate. |
| **Adopted Personal 2.0 target** | A desktop-primary Control Plane for native Agent conversation, durable governed work, federated resources, Account Hub, multi-Agent supervision, and seven resource families including MCP. |
| **Requires-backend** | Missing daemon projections, orchestration, typed controls, adapter integrations, synchronization, and MCP runtime management listed in §6. |
| **Requires-core (conditional)** | P10-T02/Lane-CTR is required only for new or changed public MCP/Goal/Plan/Run/Harness/conversation semantics. Personal-private projections may not require core changes. No route or schema is implied. |

## 2. Product outcome

Personal 2.0 gives one owner a coherent path:

```text
connect or install Agent
  -> receive first real native conversation response
  -> choose Manage with Personal
  -> confirm the daemon preview; daemon admits a durable Goal and Plan revision
  -> execute one or more Tasks and preserved attempts
  -> supervise Agent handoffs and federated resources
  -> independently verify the outcome
```

The Control Plane is the primary desktop entry and supervisor, not a second
authority writer. Native Agent applications remain usable. Personal observes
only inside the explicit observation scope established when the Agent is
connected. The owner requests or confirms governance; only the daemon admits
authority.

## 3. Adopted target scope

### 3.1 Desktop information architecture

The top-level spaces are:

**Home / Agents / Work / Library / Activity / Settings**

- Providers and System move under **Settings**.
- **Library** contains Memory, Skills, Tools, and MCP.
- Context belongs to **Work**.
- Runtime/Process belongs to **Agents**.
- Default user labels are **execution flow** and **Agent runtime engine**;
  precise contract terms remain in inspectors.

The experience is beginner-first with progressive disclosure. There is no
Basic/Expert mode.

### 3.2 Agent integration and conversation

- Agents contains adapter-backed native conversations and history.
- Vendor-specific adapters preserve native harness behavior, project a common
  core and capability matrix, and retain extension slots.
- Native application use remains possible.
- Agent connection establishes one explicit observation scope. Adapters may
  automatically observe only inside that scope; there is no speculative/global
  session scan or surprise per-session enrollment.
- The owner may request that an observed session be managed and confirm the
  daemon preview; only the daemon admits the resulting authority. Observation
  never auto-promotes a session, plan, or result.
- The common/native projection reuses or references existing Core
  `Conversation` and `ConversationBinding` identities where applicable.
  Vendor-native IDs remain opaque origin bindings. Additional projection state
  is Personal-private until P10-T02 decides otherwise.
- Signed catalog onboarding or **Connect existing** completes in at most three
  steps: choose source; review Provider/workspace/permissions once; receive the
  first real conversation response.
- Catalog records expose source, version, digest, signature, license, and
  adapter compatibility.
- Activation has two milestones: first chat; first governed and verified Task.
- Removal always distinguishes **Disconnect** from **Uninstall**.

### 3.3 Goal -> Plan revision -> Task -> Attempt

- Native Conversation is the interaction source.
- **Manage with Personal** requests a persistent Goal; the owner confirms the
  daemon preview and the daemon admits it.
- The daemon owns Plan revisions; Agent-authored plans remain Native until
  admitted.
- A Goal may span sessions and Agents.
- The daemon orchestrates the multi-Agent graph and handoffs.
- The hierarchy is Goal -> Plan revision -> Task -> Attempt. A Goal contains
  Tasks through its current Plan revision; each attempt belongs to one Task.
- Retry or fork from checkpoint creates a new attempt under that Task; it does
  not erase prior evidence or failure.
- A composed execution flow may remain a presentation object until a public
  Run contract is separately adopted.

### 3.4 Global Agent Shell

The global Agent Shell:

- explains current state, conflicts, missing capability, and recovery choices;
- proposes a next action in context;
- asks the daemon for the authoritative preview;
- lets the user confirm one consequential preview once;
- never holds authority, silently widens scope, dispatches ambient tools, or
  claims completion.

### 3.5 Federated resources

All vendor-native Agent resources are mapped through adapters:

- the origin side owns native content and native lifecycle;
- Personal owns admitted governance, bindings, permissions, and sync intent;
- adapters automatically read and detect changes only inside the explicit
  observation scope established at Agent connection;
- every write-back uses daemon-owned Intent/Effect. It may run automatically
  inside an unchanged exact daemon grant/risk policy; new, broader,
  destructive, or conflicted scope requires preview and confirmation;
- a conflict fails closed and invokes the Agent Shell;
- bidirectional synchronization is the adopted target, not a current claim.

### 3.6 Seven resource families

Personal 2.0 recognizes:

1. Memory;
2. Skill;
3. Tool;
4. Context;
5. Task;
6. Runtime/Process;
7. MCP.

MCP is not a Tool alias. It is the managed family for server install, health,
permissions, updates, and projection into Agent clients. Capabilities exposed
by a server still enter Tool or Context use only after separate mapping,
authorization, and policy.

P5-T03/P5-T04's delivered MCP Tool transport and bounded dynamic-Tool path
remain valid Tool-family implementation. They do not implement the seventh
family's server/package/connection/binding/health/quarantine lifecycle.

### 3.7 Account Hub

Settings contains Account Hub:

- first screen presets: OpenAI, Anthropic, Google, and DeepSeek;
- more providers: Qwen/Bailian, Kimi, Zhipu, SiliconFlow,
  Volcengine-Doubao, MiniMax, and OpenRouter;
- first-class custom OpenAI-compatible endpoint;
- subscription/OAuth, API key, ADR-0055 existing-credential import, and custom
  endpoint methods;
- daemon SecretStore and daemon proxy profile for every Personal-managed path;
- global default, Agent override, and conversation override;
- explicit current-session rebind/restart rather than silent switching;
- quota, usage, and cost shown separately with source and denominator honesty.

Custom OpenAI-compatible account/endpoint support is **Current implementation
(Now)**. The missing Account Hub work is the broader subscription/OAuth and
credential-import methods, Provider presets/adapters, and
global/Agent/conversation override hierarchy.

### 3.8 Controls and recovery

The target control model includes interrupt, pause/resume request, cancel,
detach, retry/fork from checkpoint, runtime restart/recover, and compensating
undo only. Detach never changes work. Undo is never promised when only a
forward compensation or reconciliation exists.

### 3.9 MCP integration

Personal manages MCP server installation, health, permission, update, and
client projection. Vendor-native session APIs are preferred. MCP plus
vendor rules is a cooperative fallback and cannot control a host Agent session.
After the first authorization, an admin-preauthorized configuration may be
applied automatically only inside the exact approved scope; any permission
expansion is previewed and reconfirmed.

## 4. Activity and evidence

Personal 2.0 presents one merged timeline with source badges:

| Badge | Meaning |
|---|---|
| **Native** | originated in a vendor Agent or native session |
| **Observed** | seen by an adapter or daemon but not admitted as authority |
| **Governed** | daemon admission, authorization, mutation, and Effect reconciliation |
| **Verified** | current independent verification and daemon acceptance only |

Badges are provenance/authority labels, not a linear progress percentage.
Agent final text, Tool result, Provider response, process exit, or native
harness success is not completion. Counts, rates, percentages, and ETAs require
a declared denominator and basis.

## 5. Visual and interaction boundary

The target language is **Calm, Dense, Precise, Professional**:

- stable navigation, master/detail lists, and inspectors;
- restrained color, material, radius, and motion;
- plain-language first reading with governance detail one level deeper;
- no glassmorphism, Liquid Glass treatment, marketing-card walls, ornamental
  KPI strips, fake progress, or decorative AI gradients;
- empty, loading, partial, stale, permission, error, success, and long-running
  states as first-class product states.

## 6. Backend Capability Gaps

### 6.1 Backend absent

| Capability | Current truth | Target treatment |
|---|---|---|
| Embedded conversations/history | Control Plane has none; native dsh panel is separate | adapter-backed Agent conversation surface |
| Goal -> Plan revision -> Task -> Attempt model | no Goal, Plan, Run, Harness, or common native-conversation projection APIs | persistent Goal, daemon Plan revisions, Tasks, and Task-owned preserved attempts |
| Task controls | no pause/cancel/retry HTTP surface | typed interrupt/pause/resume/cancel/retry/fork |
| Agent lifecycle | library/CLI capabilities are not a full Control Plane API | typed onboarding, lifecycle, restart/recover, disconnect/uninstall |
| Context/Runtime inventory | current projections are bounded or projection-only | authority-backed task/Agent inspectors |
| Multi-Agent orchestration | design-mainline, not current runtime | daemon graph and handoffs |
| Unified Activity/live state | current Activity is composed and watch coverage is partial | bounded cross-domain feed with declared coverage |
| Federated sync | no common bidirectional resource synchronization | change detection, guarded write-back, conflict handling |
| Account methods and override hierarchy | current API-key and custom OpenAI-compatible account/endpoint support plus fixed Agent binding | subscription/OAuth, credential import, broader presets, and three-level overrides |
| MCP family runtime | absent | install/health/permission/update/client projection |

### 6.2 API exists, UI-dark or only partially composed

- Native dsh has its own panel and session interaction, but the Control Plane
  has no common conversation/history projection.
- Context authorization and revocation facts exist, but there is no complete
  authority-backed Context inventory in Work.
- Existing Provider, resource, Task-evidence, dsh runtime, readiness, and
  backup/restore capabilities already support parts of the target. Moving them
  into Settings, Library, Agents, and Work is a frontend composition change;
  it does not prove the missing target semantics.

### 6.3 Contract/core gap

- MCP is the adopted seventh product family and its implementation is
  **Requires-backend**. Only a new or changed public machine surface requires
  P10-T02/Lane-CTR; a Personal-private projection may not.
- Any new public Goal, Plan, Run, Harness, Conversation extension, attempt, or
  cross-Agent handoff contract requires the same conditional contract-lane
  decision. Existing Core Conversation/ConversationBinding identities are
  reused rather than duplicated.
- Product documents intentionally define user concepts and boundaries only;
  they do not prescribe JSON, database tables, routes, transition names, or
  error codes.

## 7. Explicit exclusions and non-claims

- Linux 1.0 remains six-family and Pi-qualified; Personal 2.0 evidence cannot
  be back-projected into its release claim.
- A native or observed session is not governed by default.
- MCP connectivity does not qualify an Agent, Tool, server, or host session.
- Personal 2.0 remains owner-local and single-principal. Multi-user/RBAC,
  remote public administration, enterprise tenancy, HA, and cloud authority
  are not adopted by this scope.
- IoT/embodied and enterprise bridges remain architecture headroom.
- No target in this document is implementation, Gate, release, Profile,
  containment, performance, or Agent-benefit evidence.
