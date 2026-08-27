# CognitiveOS Personal product design

- Status: canonical stable product-design index
- Project: `cognitiveos-personal`
- Current-status owner: [PROGRESS.md](../../../../docs/plan/PROGRESS.md) `Current snapshot`
- Task/Gate owner: [PERSONAL-DEVELOPMENT-PLAN.md](../../../../docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)
- Product decisions: [Personal ADRs](../../../../docs/adr)

This directory owns stable product intent, user concepts, release boundaries,
information architecture, and user journeys. It does not own implementation
status, leases, campaign evidence, Gate results, or Profile claims.

## How to read product status

Every product document uses the same four labels:

| Label | Meaning |
|---|---|
| **Current implementation (Now)** | A shipped or otherwise repository-established capability. Exact current status still comes from `PROGRESS.md`. |
| **Adopted Personal 2.0 target** | An owner-approved product direction. It is not an implementation claim. |
| **Requires-backend** | The target needs a daemon projection, workflow, adapter, or typed API that is absent or insufficient today. The UI must not fake it. |
| **Requires-core (conditional)** | P10-T02/Lane-CTR is required only if the target adds or changes a public machine surface. A Personal-private projection may not require core changes. Product prose does not invent that surface. |

English product documents are canonical. The new `*.zh-CN.md` documents are
faithful translations and explicitly link back to their English source.

## Product model at a glance

### Current implementation (Now)

- Linux 1.0 is a six-family, Pi-qualified product:
  Memory, Skill, Tool, Context, Task, and Runtime/Process.
- P7-T05 delivered the daemon-served local Control Plane at `/ui/` with
  **Home / Work / Agents / Providers / Resources / Activity / System**.
- The current UI has governed Task creation and inspection, Provider and
  resource operations, Agent dossier projections, composed Activity, and
  system stewardship. It does not have task pause/cancel/retry, full Agent
  lifecycle, embedded native conversations, Goal/Plan/Run/Harness APIs, a
  common native-conversation projection, authority-backed Context/Runtime
  inventory, or a unified Activity feed.
- The native `cognitive dsh web` panel is a separate product surface.

### Adopted Personal 2.0 target

CognitiveOS Personal is the desktop-primary entry and supervisor for one
owner's Agents. Its top-level information architecture is:

**Home / Agents / Work / Library / Activity / Settings**

- **Agents** embeds adapter-backed native conversations and history while
  preserving vendor-native harness behavior and native-app use.
  Agent connection establishes the explicit observation scope; there is no
  speculative/global scan or surprise per-session enrollment.
- **Work** holds persistent Goals, daemon-owned Plan revisions, Tasks,
  each Task's preserved attempts, Context, execution flows, evidence, and
  multi-Agent handoffs.
- **Library** is task-oriented: Memory, Skills, Tools, and MCP. Context belongs
  in Work; Runtime belongs in Agents.
- **Activity** becomes one source-labelled timeline: Native, Observed,
  Governed, and Verified.
- **Settings** contains Account Hub and System. Providers are accounts and
  routes inside Account Hub, not a resource family or top-level peer.
- A global Agent Shell explains state and conflicts and proposes actions. The
  daemon issues previews, the user confirms consequential actions once, and
  the daemon executes. The Shell never has authority.
- Personal 2.0 adopts MCP as a true seventh resource family. Its machine
  implementation remains **Requires-backend**. Only a new or changed public
  machine surface requires P10-T02/Lane-CTR; a Personal-private projection may
  not require core changes.

The common/native conversation experience reuses or references existing Core
[`Conversation`](../../../../core/specs/schemas/conversation.schema.json) and
[`ConversationBinding`](../../../../core/specs/schemas/conversation-binding.schema.json)
identities where applicable. Vendor-native conversation IDs remain opaque
origin bindings; any additional projection is Personal-private (ADR-0058) and
is not a public Core schema.

The target is beginner-first by default. Governance detail remains available
in inspectors; there is no Basic/Expert mode.

## Documents

### Product and release boundaries

| Document | Responsibility |
|---|---|
| [Product design](product-design.md) | product model, authority, users, adopted IA, interaction principles, and capability boundaries |
| [Personal 2.0 scope](personal-2.0-scope.md) | adopted 2.0 scope, delivery boundaries, and categorized capability gaps |
| [Linux 1.0 scope](../linux-1.0-scope.md) | preserved six-family, Pi-qualified 1.0 boundary and non-claims |
| [User journeys](user-journeys.md) | first chat, governed success, daily work, recovery, conflict, account import, MCP, and state handling |

### Desktop Control Plane and Agents

| Document | Responsibility |
|---|---|
| [Web UI product design](web-ui-design.md) | current `/ui/`, target surfaces, global Shell, visual language, controls, state system, and backend gaps |
| [Agent integration and conversations](agent-integration-and-conversations.md) ([中文](agent-integration-and-conversations.zh-CN.md)) | signed onboarding, adapter capability projection, native conversations, owner confirmation followed by daemon Goal admission, Task attempts, and removal |
| [Account Hub](account-hub.md) ([中文](account-hub.zh-CN.md)) | provider presets, subscriptions/API keys/import, proxy profiles, override hierarchy, and honest metering |
| [Provider Control Plane](provider-control-plane.md) | current Provider authority and its evolution into Account Hub |

### Cognitive resources

| Document | Responsibility |
|---|---|
| [Cognitive resource model](cognitive-resource-model.md) | current six families, adopted seventh MCP family, cross-cutting objects, federation, and ownership |
| [Resource Manager](resource-manager-design.md) | common projection envelope, family-native actions, federated bindings, and conflict behavior |
| [MCP resource family](mcp-resource-family.md) ([中文](mcp-resource-family.zh-CN.md)) | server lifecycle, health, permissions, updates, client projection, and cooperative fallback |

The product decisions build on
[ADR-0037](../../../../docs/adr/0037-personal-unified-cognitive-resource-substrate.md),
[ADR-0038](../../../../docs/adr/0038-personal-agent-sidecar-linux-evolution-boundary.md),
[ADR-0043](../../../../docs/adr/0043-personal-universal-agent-adapter.md),
[ADR-0044](../../../../docs/adr/0044-personal-multi-agent-mainline.md),
[ADR-0055](../../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md),
[ADR-0056](../../../../docs/adr/0056-personal-2-0-desktop-control-plane.md), and
[ADR-0057](../../../../docs/adr/0057-personal-2-0-mcp-resource-family.md).

## Authority and non-claim rules

- Only the Rust daemon authorizes, applies CAS/epoch guards, schedules,
  persists Intent/Effect, reconciles, and accepts Tasks. UI, Shell, Agents,
  adapters, MCP servers, sidecars, CLI, and SDK remain clients or candidate
  producers.
- Provider and user secrets stay in approved Secret Stores and daemon-mediated
  proxy paths. The user-directed import boundary is defined by ADR-0055.
- Agent final text, Tool result, Provider response, process exit, or native
  harness success is not Task completion. Current independent verification is.
- `CognitiveResourceManifest` keeps its normative ActivityContext discovery
  meaning; this product taxonomy does not redefine it.
- Unknown, unavailable, stale, and not-run remain explicit. A percentage,
  count, rate, or ETA appears only when its denominator and basis are declared.
- Architecture or product adoption does not imply implementation, Gate,
  release, Profile, containment, performance, or Agent-benefit evidence.
