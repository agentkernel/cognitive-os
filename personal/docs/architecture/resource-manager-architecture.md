# Personal Resource Manager Architecture

- Status: informative current/target alignment
- Change class: `product-semantic + structural` documentation
- Product pair:
  [Resource Manager design](../product/resource-manager-design.md)
- Current decision:
  [ADR-0037](../../../docs/adr/0037-personal-unified-cognitive-resource-substrate.md)
- Personal 2.0 decision:
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md)
- Related:
  [System architecture](system-architecture.md) and
  [MCP product family](../product/mcp-resource-family.md)

This chapter describes domain composition and federated governance. It does not
define a public resource DTO, API route, error, or shared lifecycle.

## 1. Current Resource Manager

### Now

P8-T12 delivered a daemon-owned common management projection over the six
Personal families. It supports a bounded common vocabulary for listing,
inspecting, watching, binding, unbinding, enabling, disabling, and revoking
where the owning domain supports that meaning.

The projection remains deliberately narrow:

- each item comes from the owning domain's authority facts;
- object/version guards retain domain-specific meaning;
- family-specific acquisition, admission, execution, retention, and removal
  stay in typed workflows;
- generic create/install/execute/complete behavior remains refused;
- Context and Runtime may honestly expose projection-only or bounded depth;
- Task callers do not inherit management authority; and
- a projected item cannot be written back as one universal resource object.

The delivered implementation is no longer design-only. It remains
implementation evidence, not a new Gate, release, or Profile claim.

P5-T03/P5-T04 also delivered an MCP Tool transport and bounded dynamic-Tool
ecosystem. Those capabilities remain Tool-family integration; they do not
implement the Personal 2.0 MCP family identities or lifecycle.

## 2. Current and target families

| Family | Linux 1.0 / Now | Personal 2.0 responsibility |
|---|---|---|
| Memory | admitted durable knowledge | retain provenance to origin observations; admission remains daemon-owned |
| Skill | immutable package/revision and binding | native/MCP prompts or instructions remain candidates until admitted |
| Tool | registered descriptors and governed operation | MCP-advertised tools remain candidates; installation grants no Tool authority |
| Context | authorized resolved Task input | federate origin and MCP content with source, version, policy, and explicit loss |
| Task | governed unit of work | participate in admitted Goal/Plan graphs without losing Task authority |
| Runtime/Process | Agent package-to-process identities and observation | add native runtime/conversation attachment links without merging identities |
| MCP | not a Linux 1.0 family | seventh family for server/package/connection/capability/binding/health/quarantine governance; config projection remains an external mutation |

The 2.0 MCP family implementation is **Requires-backend**. A new or changed
public MCP machine surface conditionally requires P10-T02/Lane-CTR; a
Personal-private projection may not. Until the backend exists, current family
counts and Control Plane labels remain six-family.

## 3. Why MCP is a separate family

An MCP integration has lifecycle and identity that cannot be represented
honestly as a Tool:

- an MCP server or package can be known without being installed;
- an installation can exist without a running connection;
- a connection can advertise a changing capability catalog;
- Personal can bind only a subset of advertisements;
- the bound subset can feed different existing domains;
- external Agent configuration can be projected and later drift; and
- disabling or removing the MCP integration must preserve provenance and
  affected bindings.

Treating the server as one Tool would collapse package, connection,
advertisement, policy, and writeback. Treating each advertised item as
automatically registered would turn content into authority. The seventh family
owns the integration relationship; existing domains continue to own admitted
Tool, Context, and Skill semantics.

## 4. MCP identity model

### 2.0 target

The family keeps these identities distinct:

| Identity | Responsibility | Not equivalent to |
|---|---|---|
| Server | logical endpoint and declared protocol identity | package, connection, or trusted capability |
| Package | acquired/installed bytes, origin, version, digest, and transport/adapter association when applicable | running server or authority |
| Connection | one configured transport endpoint and its current bounded/authenticated observation | durable registration or capability grant |
| Capability | exact observed advertisement revision/digest from which candidates were derived | admitted Tool, Context, or Skill |
| Binding | explicit owner, Agent, Task, or workspace scope allowed to discover/request admitted capabilities | external config already projected or Tool authority |
| Health | bounded readiness, drift, timeout, and last-observation facts | verification, qualification, or completion |
| Quarantine | reasoned isolation and requalification requirement after drift, policy failure, unsafe behavior, or unresolved outcome | removal or successful reconciliation |

These are conceptual identities. Their concrete machine form is deferred to
the normative contract process. The target lifecycle covers explicit
acquire/import, registration, inspection, connection, capability refresh,
binding, enable/disable, quarantine, requalification, reconciliation, and
removal without defining a shared transition machine here.

## 5. Advertisement admission

MCP advertisements are untrusted candidates:

| MCP advertisement | Candidate destination | Required treatment |
|---|---|---|
| tool | Tool | descriptor normalization, policy, risk/capability review, exact binding, and ordinary Tool execution guards |
| protocol resource | Context | source authorization, provenance, freshness, budget, explicit loss, and no generic-Resource shortcut |
| prompt or reusable instruction package | Skill | package/revision provenance, binding, enablement, admission, and no implied Tool capability |

An MCP result is likewise a candidate or observation. It does not commit an
Effect, verify an outcome, update Memory, or complete a Task by itself.

Package/acquisition, connection, advertisement, binding, and admission are
separate. Where one advertisement contains contextual content and reusable
instruction semantics, each aspect enters its applicable Context or Skill path.
The UI and Shell must never summarize these relationships as one "enabled"
fact.

## 6. Federated resource model

### 2.0 target

Personal does not copy every origin record into authority. It maintains a
federated relationship:

| Concern | Origin-owned | Personal-owned |
|---|---|---|
| content | native conversation/history, MCP advertisement, source file or remote record | explicitly admitted Memory/Skill/Context/Task artifacts |
| lifecycle | origin create/update/delete/close and native availability | Personal policy, binding, admission, revocation, and reconciliation |
| identity/version | origin identity and observed revision/cursor | stable binding identity, policy revision, provenance, expected origin revision |
| observation | source event and current source state | observation receipt, freshness, coverage, and links to governed work |
| writeback | resulting origin state | authorization, Intent/Effect, expected version, preimage, dispatch, verification, rollback/reconcile |

Agent connection establishes the explicit observation scope. Automatic
observation is allowed only inside that scope; there is no speculative/global
scan or surprise per-session enrollment. Existing Core Conversation and
ConversationBinding identities are reused/referenced where applicable, while
vendor-native IDs remain opaque origin bindings. Additional projection state
stays Personal-private until P10-T02 decides otherwise. Observation grants no
mutation. Personal may cache bounded projections for availability, but the
cache never replaces either origin truth or daemon authority.

## 7. Conflict and reconciliation

Federated mutation fails closed on ambiguity:

- an observed source revision changed after preview;
- Personal and origin both changed a governed relationship;
- a write succeeded but acknowledgement is missing;
- the origin cannot report a version or stable identity;
- rollback cannot be proved;
- an advertisement disappeared while still bound; or
- multiple sources claim the same semantic identity without an accepted merge
  rule.

There is no last-write-wins policy. Wall-clock recency alone cannot choose an
authority outcome. The daemon preserves both observations, blocks unsafe
writeback, and requires deterministic reconciliation or owner confirmation.

Read-only observation may continue while mutation is blocked, provided stale
and conflict state remains visible.

## 8. External configuration projection

Projecting MCP or Provider configuration into an Agent-owned file/service is an
external mutation, not local presentation:

1. resolve the exact Agent, target, binding, policy, and current observed
   revision;
2. capture a bounded non-secret preimage or recoverable reference;
3. persist Intent/Effect and stable operation identity before dispatch;
4. compare the expected origin revision and fail closed on drift;
5. write only the admitted non-secret projection through the approved adapter;
6. re-read and independently verify the post-state;
7. close, compensate, or quarantine the Effect; and
8. retain redacted provenance and rollback/reconciliation evidence.

Secret material is never projected into Agent configuration. The Agent receives
only a daemon proxy profile/binding or other opaque non-secret reference.

An administrator may preauthorize automatic reconciliation when all of these
remain unchanged:

- source and target identity;
- granted capability and operation class;
- filesystem/network/Provider scope;
- Personal binding purpose;
- secret boundary;
- rollback class; and
- budget/retention bounds.

Any expansion requires a new confirmation. "The server advertised one more
tool" is an expansion, not routine drift.

## 9. Common projection in Personal 2.0

The common Resource Manager remains a read/action composition. For all seven
families it may expose conceptual facts such as:

- stable Personal and origin identities;
- family;
- Personal and origin revisions;
- source/provenance and freshness;
- health and capability condition;
- typed bindings and admitted scope;
- conflict or blocked reason;
- currently allowed actions;
- usage/budget facts when applicable; and
- observation and reconciliation coverage.

Not every family supplies every fact. Missing is explicit. The manager does not
normalize domain lifecycle labels or synthesize a common writable aggregate.

## 10. Provider and Agent relationships

MCP binding, Provider binding, native conversation attachment, and Task
resource binding are distinct:

- MCP binding chooses which integration advertisements may become candidates;
- Provider binding chooses daemon-mediated model egress;
- native attachment identifies the origin runtime/conversation being observed;
- Task/assignment binding chooses exact admitted resources for governed work.

One relationship never implies another. Provider switch or MCP config change
does not silently alter current governed work; current work requires explicit
rebind under current versions.

## 11. Product placement

The Personal 2.0 **Library** space contains Memory, Skills, Tools, and MCP.
Context and Task remain in **Work**; Runtime/Process remains in **Agents**.
This navigation composition changes no family ownership and does not turn
Library into a generic resource domain.

## 12. Current/target boundary

| Capability | Status |
|---|---|
| Six-family common Resource Manager | **Now** |
| Family-specific Memory/Skill/Tool authority paths | **Now** |
| MCP seventh-family identities and projection | **Requires-backend**; P10-T02/Lane-CTR only for new public semantics |
| MCP advertisement observation and admission mapping | **Requires-backend**; public shapes conditionally require Lane-CTR |
| Federated origin version/conflict model | **Requires-backend**; shared public semantics conditionally require Lane-CTR |
| Governed external config projection and automatic within-grant reconcile | **Requires-backend** |
| Last-write-wins reconciliation | **Forbidden target** |

This chapter authorizes no concrete contract or implementation and changes no
Linux 1.0 family count.
