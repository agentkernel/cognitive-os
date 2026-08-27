# CognitiveOS Personal Architecture

- Status: informative current/target alignment
- Change class: owner-approved `product-semantic + structural` documentation
- Project: `cognitiveos-personal`
- Current-status owner: [PROGRESS.md](../../../docs/plan/PROGRESS.md) `Current snapshot`
- Task/Gate owner: [PERSONAL-DEVELOPMENT-PLAN.md](../../../docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)
- Normative contracts: [`core/specs/`](../../../core/specs) and applicable
  [standards](../../../docs/standards)
- Product direction:
  [Personal 2.0 scope](../product/personal-2.0-scope.md),
  [Agent integration and conversations](../product/agent-integration-and-conversations.md),
  [Account Hub](../product/account-hub.md), and
  [MCP resource family](../product/mcp-resource-family.md)
- Adopted decisions:
  [ADR-0037](../../../docs/adr/0037-personal-unified-cognitive-resource-substrate.md),
  [ADR-0043](../../../docs/adr/0043-personal-universal-agent-adapter.md),
  [ADR-0044](../../../docs/adr/0044-personal-multi-agent-mainline.md),
  [ADR-0053](../../../docs/adr/0053-personal-web-ui-stack.md),
  [ADR-0054](../../../docs/adr/0054-repository-subproject-structure-and-1.0.0-finalization.md),
  [ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md),
  [ADR-0056](../../../docs/adr/0056-personal-2-0-desktop-control-plane.md), and
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md)

This directory explains how Personal composes accepted contracts and product
decisions. It does not create public DTOs, routes, errors, state machines,
current task status, Gate evidence, release claims, or Profile conformance.

## Status vocabulary

Architecture chapters use these labels:

| Label | Meaning |
|---|---|
| **Now** | Implemented current behavior confirmed by the canonical current-status source; not automatically a release or Gate claim |
| **2.0 target** | Adopted Personal product direction that must not be presented as current capability |
| **Requires-backend** | Needs a future daemon/client implementation task before a product surface may offer it |
| **Requires-core (conditional)** | P10-T02/Lane-CTR is needed only for a new or changed public machine surface. Personal-private projections may not require core changes. |

`Requires-backend` and conditional `Requires-core` can both apply when a target
also changes a public machine surface. Unknown capability is never upgraded to
either supported or unavailable by architecture prose alone.

## Product architecture statement

### Now

Linux Personal 1.0 remains a six-family cognitive-resource system:

1. Memory;
2. Skill;
3. Tool;
4. Context;
5. Task;
6. Runtime/Process.

The daemon is the sole authority writer. The delivered Resource Manager,
Provider Control Plane, learning admission path, adapter registration boundary,
and daemon-served Control Plane all remain clients or deterministic daemon
services under that rule. The desktop client implementation lives at
[`clients/pc/web/`](../../../clients/pc/web) and is served same-origin by the
daemon under `/ui/`. Native `cognitive dsh web` is a separate Agent surface.

Pi is still the only Agent qualified by the Linux 1.0 claim. The delivered
general adapter registration/lifecycle machinery and fixture-based non-Pi work
do not transfer Pi qualification or create a current multi-Agent runtime.

### Personal 2.0 target

Personal 2.0 adds **MCP as a seventh user-visible family** without changing the
Linux 1.0 six-family claim. The delivered P5 MCP Tool transport/dynamic-Tool
work remains valid for its bounded scope but is not the seventh-family
implementation. MCP server, package, connection, capability, binding, health,
and quarantine identities remain distinct. Advertised tools, protocol
resources, and prompts are untrusted candidates into the existing Tool,
Context, and Skill domains respectively; connection or installation alone
grants no authority.

The Desktop Control Plane remains a same-origin daemon client and adopts six
product spaces: **Home, Agents, Work, Library, Activity, and Settings**.
Library contains Memory, Skills, Tools, and MCP; Work owns Context and Task;
Agents owns Runtime/Process. Providers, Account Hub, System stewardship, and
sessions are Settings sections. Activity is the provenance-preserving merged
timeline, not another authority or resource family. The global Agent Shell is
likewise a candidate-producing assistant, never an authority or a replacement
for typed Control Plane state.

Vendor-specific Agent adapters use the strongest safe native protocol available
for conversation/session control. They project only a minimal common capability
and conversation model, with adapter-specific render slots for native detail.
ACP conformance is not required. MCP plus rules can be a cooperative fallback
for candidate/tool exchange, but cannot impersonate native session control.

Native conversations remain origin-owned observations. An explicit admission
request and owner confirmation let the daemon create the Goal, revisioned Plan,
and governed Tasks. The daemon alone admits and owns the multi-Agent graph,
assignment and handoff state, budgets, Intent/Effect, reconciliation,
verification, and acceptance.

## Stable composition rules

- Experience surfaces, Shells, native Agents, adapters, and MCP servers are
  clients or candidate producers; none is authority.
- A common projection may unify reading and navigation, but never collapses
  family-specific identity, lifecycle, retention, or policy.
- Origin-owned content and Personal-owned policy/binding remain separate.
  Agent connection establishes an explicit observation scope; observation may
  be automatic only inside it, with no speculative/global scan or surprise
  per-session enrollment. Every writeback uses daemon-owned Intent/Effect and
  may run automatically only inside an unchanged exact grant/risk policy.
  New, broader, destructive, or conflicted scope requires preview and
  confirmation; conflict resolution fails closed without last-write-wins.
- Provider and user secrets enter through approved non-logging daemon paths,
  stay in an approved `SecretStore`, and are consumed through daemon proxy
  profiles. Raw material never crosses an Agent/adapter conversation wire.
- External configuration projection is an external mutation: capture the
  preimage, persist Intent/Effect before dispatch, compare the expected
  revision, verify the result, and retain rollback/reconciliation evidence.
- Native output, process state, an adapter event, or an MCP result is never
  Task completion. Independent verification remains required.
- Public contract changes use the normative contract process. Architecture
  does not pre-empt that process with parallel machine shapes.

## Documents

| Document | Current/target responsibility |
|---|---|
| [System architecture](system-architecture.md) | **Now:** six-family daemon composition. **2.0:** Desktop Control Plane, native adapter fabric, Goal/Plan admission, seventh MCP family |
| [Web UI architecture](web-ui-architecture.md) | **Now:** delivered same-origin client. **2.0:** six-space Desktop Control Plane, conceptual state projections, capability-honest gaps |
| [Agent Shell and Agent lifecycle](agent-shell-and-agent-lifecycle.md) | Shell role, native-session observation, explicit admission, strict identities, and recovery verb distinctions |
| [Agent adapter architecture](agent-adapter-contract.md) | delivered P8 registration boundary plus the vendor-specific 2.0 capability/conversation projection |
| [Multi-agent orchestration](multi-agent-orchestration.md) | daemon-owned Goal -> Plan revision -> Task -> Attempt graph, assignments, handoffs, budgets, verification, and default-off current boundary |
| [Resource Manager](resource-manager-architecture.md) | delivered six-family common projection plus federated resource policy and target MCP family |
| [Provider Control Plane](provider-control-plane.md) | delivered provider governance plus target Account Hub import, proxy profiles, and scoped switching |
| [Authority, data and recovery](authority-data-and-recovery.md) | authority placement, progress provenance, external writeback, restart/reconcile ordering, and compensating undo |
| [Learning loop](learning-loop.md) | delivered candidate/admission loop plus native-session and federated-origin learning inputs |
| [Context evolution](context-evolution.md) | compaction and adaptive budgets |
| [Async event evolution](async-event-evolution.md) | measured async migration decision gate |
| [Performance architecture](performance-architecture.md) | floors, stage timing, and structure-debt candidates |
| [Headroom: IoT and multi-tenancy](headroom-iot-and-multitenancy.md) | reserved bridges; not current Personal implementation scope |
| [Web UI route inventory](web-ui-route-inventory.json) | frozen P7-T05/D01 input with stale pre-ADR-0054 checkout metadata; not a current exhaustive route inventory and not a 2.0 contract |

## Contract decisions intentionally unresolved

Architecture does not pre-decide the Lane-CTR work assigned to `P10-T02`.
That work must decide:

1. how the common Agent conversation projection reuses or references existing
   Core `Conversation` and `ConversationBinding`, while vendor-native IDs remain
   opaque origin bindings; any additional projection stays Personal-private
   unless P10-T02 selects a public extension, including identity/version
   compatibility and capability digesting;
2. the MCP public/private boundary for server, package, connection,
   capability, binding, health, and quarantine;
3. compatibility and migration from the delivered P5 MCP Tool/adapter-era
   records into any seventh-family representation;
4. fail-closed behavior for older clients that do not understand the new
   family or conversation capabilities; and
5. which target Goal -> Plan revision -> Task -> Attempt, execution-flow,
   Harness, Conversation extension, and cross-Agent handoff concepts require
   separate public core contracts rather than Personal-private projections.

Provider-profile override and federated synchronization shapes likewise remain
private/product concepts until a later contract decision selects a public
surface.

## Source ownership and non-claims

When these documents disagree with another source:

1. machine shape and registered transitions come from `core/specs/`;
2. behavioral semantics come from applicable normative companions and
   `docs/standards/`;
3. Personal product decisions come from accepted Personal ADRs;
4. formal tasks and Gates come from the Personal development plan;
5. current facts come only from `PROGRESS.md` Current snapshot;
6. these architecture chapters are corrected to match those sources.

Architecture presence is not implementation evidence. Implementation presence
is not by itself a Gate, release, Profile, or Agent-benefit result.
