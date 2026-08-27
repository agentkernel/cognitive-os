# MCP resource family

- Status: adopted Personal 2.0 seventh-family product target
- Canonical language: English
- Decision:
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md)
- Related:
  [Cognitive resource model](cognitive-resource-model.md),
  [Resource Manager](resource-manager-design.md), and
  [Agent integration and conversations](agent-integration-and-conversations.md)
- Chinese translation: [mcp-resource-family.zh-CN.md](mcp-resource-family.zh-CN.md)

Personal 2.0 adopts MCP as a true seventh cognitive-resource family. The family
manages MCP server identity, installation, health, permissions, updates, and
projection into compatible Agent clients. It does not turn MCP into an
authority plane or a host-session controller.

## 1. Reality ledger

| Boundary | MCP truth |
|---|---|
| **Current implementation (Now)** | Linux 1.0 has six families. P5-T03/P5-T04 delivered an MCP Tool transport and bounded dynamic-Tool path inside the Tool family. They did not implement an MCP family manager, authority-backed MCP inventory, server lifecycle, permission/update workflow, or general client projection. |
| **Adopted Personal 2.0 target** | MCP is the seventh family in Library, with server install, health, permissions, update, client projection, conflict handling, and Activity. |
| **Requires-backend** | All Personal MCP runtime management, adapter/client projection, health, permission, update, synchronization, and recovery behavior. |
| **Requires-core (conditional)** | ADR-0058 kept the MCP family Personal-private. A later public MCP machine surface would need a new Lane-CTR decision. |

## 2. Why MCP is a family

MCP has a product lifecycle distinct from Tool, Context, and Agent:

- a server has source, version, trust/provenance, compatibility, health,
  permissions, update posture, and projected clients;
- one server may expose multiple kinds of candidate capability;
- one server can be healthy while a permission or one client projection is
  denied or failed;
- updating a server is different from enabling a Tool;
- connecting a server is different from authorizing a capability;
- configuring a client is different from controlling its host session.

Therefore MCP is not a Tool alias, a generic transport label, or a property of
an Agent. Tool and Context remain separate families that may consume
MCP-originated capabilities only after mapping and authorization.

## 3. Product location

**Library → MCP** provides:

- installed/connected server list;
- source, version, trust/provenance, and compatibility;
- health and last observation;
- requested versus admitted permissions;
- Agent-client projections and their freshness;
- update availability and current version;
- quarantine/requalification state, conflicts, blocked reasons, Activity, and
  receipts.

Related Agent inspectors show the MCP servers projected into that Agent. Work
shows the exact MCP-originated Tool or Context facts admitted for a Task.
Settings holds global permission/default policy, not the server inventory.

## 4. Server install and connection

**Adopted Personal 2.0 target**

1. The user selects a server source.
2. Personal shows identity, version, trust/provenance, license when available,
   adapter/client compatibility, requested permissions, affected Agent clients,
   and update behavior.
3. The daemon issues the consequential install/connect preview.
4. Confirmation authorizes only the exact server and scope.
5. The daemon performs the family-specific lifecycle and records a durable
   result.
6. Health, permission, and client projection are evaluated separately.

Install/connect grants no Tool, Context, workspace, network, model, secret, or
host-session authority. A healthy process is not a usable server unless the
required permission and client projection are also current.

The exact acquisition and trust mechanism is a backend/core decision; this
document does not invent one.

## 5. Health and compatibility

Health answers whether the managed server can provide its declared MCP service
at the observed time. It does not answer whether:

- a particular Agent client is configured;
- a permission is admitted;
- a capability is mapped to a Personal Tool or Context source;
- a Task may use it;
- the host Agent session is running or controllable;
- an outcome is verified.

The product keeps these facets separate:

- server lifecycle/health;
- protocol/client compatibility;
- permission;
- projection/configuration;
- mapped capability availability;
- Task-specific authorization.

Unknown and stale are not healthy. A process exit or handshake success is an
observation, not Task completion.

## 6. Permissions

Permission is reviewed by scope and consequence:

- server process/network access required for operation;
- data/resource categories the server may expose or consume;
- compatible Agent clients that may receive configuration;
- Personal families into which capabilities may be mapped;
- write-back/configuration targets;
- retention and update behavior where applicable.

Installing the server grants none of these implicitly. Permission expansion is
always consequential and receives a new daemon preview and user confirmation.
An MCP server, Agent, Skill, or adapter cannot grant itself more scope.

## 7. Client projection

Projection means configuring a compatible Agent client to know about the
server; it does not mean controlling the Agent's live session.

The preference order is:

1. **Vendor-native session/configuration API** when available and bounded;
2. **Managed vendor adapter path** when it provides equivalent governed
   semantics;
3. **MCP plus vendor rules** as a cooperative fallback.

The fallback may prepare or change supported configuration through the daemon's
governed write-back path. It cannot interrupt, pause, resume, restart, or
otherwise control the host Agent session. If the host requires reload/restart,
the UI says so and requests a separate supported action.

Each projected client reports success, failure, permission denial,
incompatibility, and staleness independently. Partial projection is never shown
as complete.

Agent-client observation is limited to the explicit observation scope
established when that Agent is connected. Personal performs no
speculative/global native-session scan and does not surprise-enroll newly found
sessions.

## 8. Admin-preauthorized configuration

The first configuration for a server/client/scope receives explicit
authorization. After that authorization, Personal may automatically apply
compatible configuration only when all of these remain unchanged:

- server identity/version compatibility;
- target Agent client;
- exact permission and write-back scope;
- endpoint/trust boundary;
- approved configuration class.

This is admin-preauthorized automation, not ambient authority. Any permission
expansion, new client, broader target, changed trust boundary, or incompatible
update requires a new daemon preview and confirmation.

Every configuration write-back is still a persisted Intent/Effect operation
with reconciliation and a durable receipt. The global Agent Shell may explain
or propose the action but never performs it.

## 9. Capability mapping

An MCP server may describe capabilities, but Personal treats them as
candidates:

- an operation maps to a Tool only through Tool registration, descriptor,
  availability, permission, budget, and dispatch policy;
- data or retrieval maps to Context only through authorization-before-ranking,
  provenance, freshness, loss, and Task-specific selection;
- prompts or reusable instructions enter as Skill candidates and pass Skill
  package/revision, provenance, binding, enablement, and admission rules;
- returned content does not automatically become Memory;
- server/client state does not become Runtime authority;
- MCP output, Tool result, or server success does not complete a Task.

The mapping preserves server origin and version so Activity and evidence can
trace the source.

## 10. Update, recovery, and removal

### Update

The target shows current version, available version, compatibility, permission
changes, affected clients/work, and recovery expectation before confirmation.
An update that expands permission or trust scope always requires confirmation.

### Recovery

The product distinguishes server unhealthy, quarantined/requalification
required, client projection stale, permission denied, configuration conflict,
and host reload required. Recovery never uses blind redispatch after an unknown
external outcome.

### Removal

Removal previews affected Agent clients, Tool/Context mappings, active Tasks,
pending Effects, configuration write-backs, and retained history. Removing a
server does not silently delete unrelated native configuration or governed
evidence.

All lifecycle behavior is **Requires-backend**.

## 11. Federated ownership and conflicts

- The server/origin owns native content and protocol behavior.
- Personal owns admitted governance, bindings, permission, synchronization
  intent, and authority receipts.
- Authorized read/change detection may be automatic only inside the explicit
  observation scope established at Agent connection.
- Every Personal-to-native configuration write-back is daemon-owned
  Intent/Effect. It may run automatically inside an unchanged exact daemon
  grant/risk policy; new, broader, destructive, or conflicted scope requires
  preview and confirmation.
- Concurrent or incompatible changes fail closed.
- The global Agent Shell explains the conflict and asks the daemon for a
  family-specific resolution preview.
- No last-writer-wins or model-selected resolution is assumed.

Bidirectional synchronization is the adopted target and
**Requires-backend**.

## 12. Activity and completion

MCP events use the shared timeline badges:

- **Native** — server or vendor-client fact;
- **Observed** — adapter/daemon observation;
- **Governed** — admitted permission, configuration, lifecycle, or mapping;
- **Verified** — independent current verification where defined.

Health, installation, projection, and permission counts display only with a
declared denominator. No fake percentage or ETA is inferred. MCP server success,
Agent final text, Tool result, or process exit is not Task completion.

## 13. Required states

| State | MCP behavior |
|---|---|
| Empty | explain the family and offer install/connect |
| Loading | name server, health, permission, client, or update source |
| Partial | list each successful/failed/unknown client projection |
| Permission | show exact requested scope, deny/narrow path, and affected clients |
| Error | preserve source/configuration input and offer safe recovery |
| Stale | show observation age; block unsafe update/write-back inference |
| Conflict | fail closed and require daemon-backed resolution |
| Success | show durable receipt, health/permission distinction, projected clients, and next action |

## 14. Backend Capability Gaps

### Backend absent

- MCP server inventory and lifecycle;
- health/compatibility projection;
- permission and update workflows;
- vendor-native and cooperative client projection;
- admin-preauthorized configuration;
- capability mapping and federated conflict handling.

### API/native surface exists, UI-dark or reusable

Vendor Agents may already have native configuration/session APIs, and some MCP
servers exist independently of Personal. Those surfaces are integration inputs,
not Personal governance or host-session control.

### Contract/core gap

MCP is already the adopted seventh product family and implementation is
**Requires-backend**. [ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md)
keeps the family Personal-private; a later public lifecycle, permission,
mapping, projection, error, or transition machine surface would need a new
Lane-CTR decision.

## 15. Fixed boundaries and non-claims

- Daemon-only authority and persist-before-dispatch remain unchanged.
- MCP never receives raw Provider credentials or SecretStore access.
- MCP plus rules cannot control a host Agent session.
- Connection, health, configuration, or capability discovery grants no
  permission by itself.
- MCP support does not qualify an Agent, Tool, server, or release.
- Linux 1.0 remains six-family and Pi-qualified.
- This target makes no implementation, Gate, release, Profile, performance,
  containment, or Agent-benefit claim.
