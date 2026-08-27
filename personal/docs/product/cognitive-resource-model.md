# Personal Cognitive Resource Model

- Status: canonical current + Personal 2.0 product concept model
- Architecture mapping: [System architecture](../architecture/system-architecture.md)
- Decisions:
  [ADR-0037](../../../docs/adr/0037-personal-unified-cognitive-resource-substrate.md) and
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md)

## 1. Definition and boundaries

A cognitive resource is a user-visible family of durable knowledge, reusable
capability description, governed operation, resolved input, goal-directed work
or managed integration/runtime activity.

| Boundary | Family model |
|---|---|
| **Current implementation (Now)** | Linux 1.0 and current authority projections have six families: Memory, Skill, Tool, Context, Task, Runtime/Process. |
| **Adopted Personal 2.0 target** | MCP becomes a true seventh family beside the existing six. |
| **Requires-backend** | MCP server lifecycle, health, permissions, updates, client projection, and federated synchronization are not implemented as a Personal family today. |
| **Requires-core (conditional)** | P10-T02/Lane-CTR is required only if MCP adds or changes a public machine surface. A Personal-private projection may not require core changes. |

The seven-family target is:

1. Memory;
2. Skill;
3. Tool;
4. Context;
5. Task;
6. Runtime/Process;
7. MCP.

The word *family* is a product taxonomy. It does not authorize a public generic
`Resource` DTO, a giant SQLite `Resource` table, a universal repository or one
state machine. Each family retains its own stable identities, transitions,
storage, ownership, retention, and failure semantics. Product views compose
family projections without erasing those boundaries.

The global Agent Shell may provide names and explain state, but a daemon-issued
preview resolves exact identity, version, permission, and consequence before a
mutation. The Shell, Agents, adapters, sidecars, and MCP servers remain
non-authority clients or candidate producers.

## 2. Cross-cutting objects

The following objects bind across resource families rather than becoming new
families:

| Object | Cross-cutting role |
|---|---|
| Budget | deadline, retry, step, token, cost, retrieval and output ceilings |
| Permission | principal, capability, scope, purpose, expiry and revocation |
| Model | selected inference capability and daemon-owned Provider route |
| Artifact | content-addressed input or output referenced by one or more families |
| Intent/Effect | persist-before-dispatch record for external or irreversible mutation |
| Evidence | immutable fact evaluated by an independent verifier |
| Event | ordered authority change and watch projection input |

These objects are displayed where they explain Memory, Skill, Tool, Context,
Task, Runtime, or MCP activity. An Agent remains a navigation and actor concept
composed from Runtime identities; it is not a resource family. MCP is the
seventh target family because Personal manages the integration's own lifecycle,
health, permissions, updates, and client projections rather than treating an
Agent as stored resource content.

## 3. Discovery is not the family catalog

`CognitiveResourceManifest` keeps its existing normative scope. It is a logical
discovery manifest filtered for an `ActivityContext`, subject, purpose, scope,
policy/revocation versions, budget and expiry. It may expose domains,
expandable references and query capabilities.

The manifest does not enumerate every product resource, grant read access,
grant Tool invocation or replace family-specific projections. Discovery,
reading and action remain separate. This product model creates no alternative
manifest schema.

## 4. Memory family

### 4.1 Authority flow

Memory follows:

`MemoryCandidate -> MemoryAdmissionDecision -> MemoryObject`

An explicit user `remember` and an Agent Memory proposal both create a
candidate. The source is retained. Explicit user intent may receive higher
admission priority, but it does not bypass permission, purpose, conflict,
provenance or retention policy. An Agent cannot write `MemoryObject` directly.

### 4.2 Admitted object semantics

An admitted Memory object exposes, as applicable:

- stable object identity and kind;
- owner and scope, including Task, workspace or owner-level scope;
- admitted purpose and use restrictions;
- source provenance and candidate/admission references;
- immutable content digest and monotonic version;
- conflict relationship and deterministic disposition;
- created, updated and expiry facts;
- forget request and tombstone state;
- authority events that explain each transition.

Updates create a new version rather than silently replacing provenance. A
conflict remains visible until a deterministic admission decision resolves,
keeps separate or rejects it. Expiry makes content unavailable under policy.
Forget creates a durable tombstone that prevents stale index, cache or sidecar
copies from resurrecting the object.

### 4.3 Storage and retrieval

Daemon-owned SQLite is the Memory source of truth. FTS5 and metadata filters
are derived, rebuildable indexes. Retrieval applies authorization, scope,
purpose, tombstone, expiry and metadata filters before FTS ranking. A missing
index is a degraded retrieval condition, not permission to use an untracked
Agent cache as truth.

Linux 1.0 does not require embeddings, a vector store, a knowledge graph or
automatic extraction from every conversation. Those features require separate
privacy, utility and migration decisions.

## 5. Skill family

### 5.1 Package and revision

A Skill is a first-class immutable package/revision. Local import accepts a
bounded package compatible with `SKILL.md` and optional `resources/` and
`scripts/`. The daemon records source path/provenance, normalized manifest,
content digest, revision identity, compatibility result and lifecycle state.

Mutable edits create a new imported revision. Pin selects an exact revision;
it does not mutate package bytes in place.

### 5.2 Linux 1.0 actions

Linux 1.0 supports:

- `install` from a local path after preview and validation;
- `list` installed packages and revisions;
- `inspect` manifest, provenance, digest, files and requirements;
- `pin` an exact revision for use;
- `enable` or `disable` eligibility without changing package bytes;
- `remove` a revision after impact and retention checks.

### 5.3 Content, execution and permission

Authorized Skill instructions and resources may enter a Task's Context. A
script is inert package content until a registered Tool is selected,
authorized and dispatched under its own descriptor and policy. Skill import,
pin or enablement grants no Tool, filesystem, process, network, model, secret
or budget capability. A Skill cannot authorize itself and cannot dispatch
directly.

Marketplace discovery, automatic download, Skill chaining and autonomous
dependency resolution are deferred beyond Linux 1.0.

## 6. Tool family

### 6.1 Linux 1.0 Tool groups

The minimum Tool catalog contains:

1. Standard Workspace read and search;
2. Standard Workspace write and patch;
3. bounded process and check execution;
4. read-only HTTP fetch.

Workspace operations resolve canonical paths under the admitted Standard
Workspace or explicit Extended Home entries. Process/check Tools use an exact
registered command/check descriptor, bounded arguments, working directory,
environment, time, output and exit observations. HTTP fetch is read-only and
bounded by registered origin/URL policy, redirects, size, time and content
handling; it carries no ambient cookies or Provider secrets.

### 6.2 Registry and availability

The daemon owns a static Tool registry. Each Tool has a stable identity,
immutable descriptor digest, operation class, input/output contract, risk,
required capability and availability reason. Runtime discovery does not add a
Tool to the registry.

An unknown, descriptor-drifted, disabled or quarantined Tool has dispatch
count zero. Pi, a Skill or a sidecar cannot turn a similarly named operation
into a registered Tool.

### 6.3 Mutation protocol

Within an admitted Standard Workspace, low-risk reversible writes use a
recoverable journal that records intended paths, before/after identity and
rollback status. This preserves low friction without accepting untracked Agent
file mutation.

External or irreversible mutations require persisted Intent/Effect before
dispatch, stable idempotency identity, epoch fencing, result recording and
reconciliation. Tool exit zero is not authoritative Effect or Task completion.

## 7. Context family

### 7.1 Per-Task request and view

Every admitted Task has a real `ContextRequest` and `ContextView`. The request
states required and optional inputs, purpose, audience, scope, freshness,
budget and loss policy. The view records exact selected source identities,
versions/digests, ordering, transformations, losses, policy versions and
expiry.

The resolver may combine:

- Task intent, contract/bindings and current authority state;
- admitted Memory;
- enabled and pinned Skill instructions/resources;
- registered Tool summaries and current availability;
- artifacts and evidence;
- Standard Workspace and bounded Extended Home inputs;
- explicit Task inputs supplied by the user or client.

### 7.2 Authorization before ranking

The daemon authenticates, applies permission/scope/purpose policy and filters
revoked, expired, hidden or forbidden candidates before ranking. Denied
content cannot affect ranking or leak through counts and names. Required input
that cannot be authorized, resolved or validated fails closed. Optional loss
is explicit and machine-visible, including omission, truncation, conflict,
staleness, unavailable source and budget exhaustion.

### 7.3 Deterministic Linux 1.0 selection

Linux 1.0 selection uses deterministic source priority, metadata filters and
FTS retrieval with stable tie-breaking. A `ContextView` has a canonical digest.
Unchanged leading segments form a stable prefix across refreshes. A delta binds
the base digest and explicitly lists additions, removals/invalidations,
replacements, conflicts and cumulative loss/cost.

Learned reranking, embedding retrieval, graph expansion and other complex
ranking are deferred. The Agent may propose a request or information gap but
cannot widen admitted Context.

## 8. Task family

Task retains these authority semantics:

- raw user intent is durably fixed before probabilistic interpretation;
- the server issues a canonical, digest-bound preview;
- admission validates exact preview digest, object versions and CAS/epoch;
- budgets and scheduler eligibility are deterministic authority facts;
- watch projections expose ordered state without client fabrication;
- checkpoints are compatibility- and epoch-bound;
- external mutations close through Effect reconciliation;
- an independent verifier decides whether acceptance criteria are satisfied.

Provider response, Pi `agent_end`, sidecar success, process exit and Tool output
are observations, not Task completion.

### Future TaskContract direction

A future `TaskContract` revision is intended to fix exact resource references
and constraints, Agent/sidecar adapter identity and Context policy. This is a
future contract direction, not a statement that those fields or bindings are
implemented. Any public contract change requires Lane-CTR synchronization of
schema, registry, generated bindings, transitions and negative vectors.

Linux 1.0 resource selections may be bound through existing daemon authority
records without claiming that future `TaskContract` shape.

## 9. Runtime/Process family

Runtime identity is deliberately decomposed:

| Identity | Meaning | Must not be confused with |
|---|---|---|
| Package | immutable Agent distribution and provenance | installed or trusted runtime |
| Installation | verified private bytes and acquisition lock | permission or active registration |
| Registration | Personal policy plus installation/sidecar binding | running instance |
| Instance | supervised logical Agent runtime | conversation or Task |
| Sidecar | versioned per-Agent protocol adapter | authority service |
| Execution | Task/Loop/instance/epoch binding | process or final acceptance |
| Process | PID/handle and bounded host observations | a new domain or execution success |

The per-Agent sidecar is the primary integration boundary. It translates
Agent protocol and observations into daemon services and is always a client.
Unknown or drifted sidecar identity fails closed before execution dispatch.

Process is daemon-owned observation and supervision data. It may show spawn,
alive, exit, signal, resource sample and restart facts. It does not create a
separate Process authority domain, advance Task state or prove completion.

Pi is the only Linux 1.0 qualified Agent/sidecar combination. Other adapters do
not inherit Pi evidence.

## 10. MCP family

### Current implementation (Now)

MCP is outside the Linux 1.0 six-family model. Personal has no MCP family
authority, server manager, health/permission/update projection, or general
client-configuration workflow today. P5-T03/P5-T04 delivered an MCP Tool
transport and bounded dynamic-Tool path inside the Tool family; that evidence
does not implement the seventh family. Existing Tool, Context, and Skill
semantics must not be relabelled MCP.

### Adopted Personal 2.0 target

The MCP family represents a managed server integration and its projection into
compatible Agent clients. Its product responsibilities are:

- server source, installation/registration, version, and update posture;
- health and compatibility, separate from permission;
- requested and admitted permissions;
- projection into one or more Agent clients;
- configuration/write-back state, conflicts, and durable receipts;
- source-labelled capabilities that may later map into Tool or Context use.

MCP protocol prompts or reusable instruction packages may map into Skill only
as candidates through Skill admission; they do not become MCP-owned Skill
authority.

Connecting a server grants no Tool, Context, workspace, network, model, secret,
or host-session authority. Each capability remains subject to family-specific
mapping and daemon authorization. Vendor-native session APIs are preferred.
MCP plus vendor rules is only a cooperative fallback and cannot control the
host Agent session.

After first explicit authorization, admin-preauthorized configuration may be
applied automatically only within the exact approved scope. Permission
expansion always requires a fresh daemon preview and confirmation.

### Dependency boundary

MCP implementation is **Requires-backend**. A new or changed public MCP machine
surface conditionally requires P10-T02/Lane-CTR; a Personal-private projection
may not. This model defines no public DTO, database row, endpoint, transition,
or error. See [MCP resource family](mcp-resource-family.md).

## 11. Workspace, federation, and relationship rules

1. Standard Workspace is the low-friction default boundary for registered
   read/search and reversible write/patch Tools.
2. Extended Home is an explicit bounded set of document/project roots,
   operations and optional ordinary outbound network access, never ambient
   home-directory access. Secret/credential stores, CognitiveOS authority
   data, Docker/system sockets, system directories and privilege management
   remain hard-denied.
3. An installed Agent or Skill has no runtime permission by default.
4. A selected Model grants no Tool, filesystem or network capability.
5. Context is Task input; Memory is admitted durable knowledge. Conversation
   history becomes neither automatically.
6. Skill instructions/resources may enter Context; scripts execute only
   through registered Tools.
7. One artifact may serve as output and as verification evidence only under
   explicit references and policy.
8. An Effect is not complete because a process or Tool exited; reconciliation
   establishes the authoritative outcome.
9. Verification remains independent from the Agent/executor that produced the
   result.
10. Vendor-native resource content remains owned by its origin. Personal owns
    admitted governance, bindings, permissions, and synchronization intent.
11. Existing Core Conversation and ConversationBinding identities are reused or
    referenced where applicable. Vendor-native conversation IDs remain opaque
    origin bindings; additional projection state stays Personal-private until
    P10-T02 decides otherwise.
12. Agent connection establishes an explicit observation scope. Adapters may
    automatically read and detect native changes only inside it; there is no
    speculative/global scan or surprise per-session enrollment.
13. Every native write-back uses daemon-owned Intent/Effect and execution. It
    may run automatically inside an unchanged exact daemon grant/risk policy;
    new, broader, destructive, or conflicted scope requires preview and
    confirmation. Conflicts fail closed and invoke the global Agent Shell for
    an explanation and a daemon-backed resolution path.
14. Bidirectional synchronization is an adopted Personal 2.0 target and
    **Requires-backend**; it is not inferred from current one-way imports or
    bindings.

## 12. Information architecture and Shell examples

### Current implementation (Now)

The daemon-served Control Plane has Home, Work, Agents, Providers, Resources,
Activity, and System. The Linux 1.0 conceptual projection remains six-family.
Current Context and Runtime resource inventories are bounded or projection-only,
and current Activity is composed rather than unified.

### Adopted Personal 2.0 target

- **Library:** Memory, Skills, Tools, MCP;
- **Work:** Task and Context;
- **Agents:** Runtime/Process;
- **Activity:** source-labelled Native, Observed, Governed, and Verified facts.

The top-level IA is Home, Agents, Work, Library, Activity, and Settings.
Providers and System live in Settings. Navigation placement does not change
family ownership.

Examples of intended requests:

- "Remember that this workspace uses the checked-in formatter, scoped to this
  workspace, and expire it in 90 days."
- "Show the candidate, admission decision and current version for that Memory."
- "Import this local Skill, pin its digest and keep its scripts disabled until
  a registered Tool is selected."
- "Explain which Context inputs were required, omitted or truncated for this
  Task."
- "Patch these files in the Standard Workspace and show the recovery journal."
- "Why is this Pi sidecar quarantined, and which execution and process did it
  last serve?"
- "Which Effect is unresolved, and what evidence still blocks acceptance?"
- "Show the native Skill change that conflicts with Personal's binding, but do
  not write back until the daemon previews the resolution."
- "Which MCP servers are healthy, which permissions are admitted, and which
  Agent clients have a current projection?"

The global Agent Shell explains and proposes. It never answers a state-changing
request by invoking ambient Agent tools, running Skill scripts, changing MCP
configuration, or mutating native resources directly. Only the daemon issues
and executes the confirmed preview.

## 13. Linux 1.0 and target boundary

Linux 1.0 targets a minimum real slice of its six families: Memory, Skill,
Tool, Context, Task, and Runtime/Process. MCP is not retroactively included.
Embeddings/vector/graph Memory, automatic
full-conversation extraction, Skill marketplaces/chaining/auto-download,
complex Context ranking, broad dynamic Tool ecosystems, MCP, non-Pi
qualification, federated synchronization, and Multi-Agent orchestration remain
outside the Linux 1.0 claim.

Personal 2.0 adopts MCP and federated resource behavior as target semantics.
MCP implementation remains **Requires-backend**. Public machine semantics
conditionally require P10-T02/Lane-CTR; Personal-private projections may not.
Other target features remain capability-gated as identified in
[Personal 2.0 scope](personal-2.0-scope.md).

Documented scope is not implementation evidence. Current task, Gate, release
and Profile facts remain in [PROGRESS.md](../../../docs/plan/PROGRESS.md).
