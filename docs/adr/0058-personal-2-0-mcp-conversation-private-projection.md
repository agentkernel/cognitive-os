# ADR-0058: Personal 2.0 MCP family and conversation projection stay Personal-private

- Status: Accepted (Lane-CTR compatibility decision, 2026-08-27)
- Date: 2026-08-27
- Decision owner: CognitiveOS Personal product owner (executed under standing
  continuous-delivery authorization for `P10-T02`)
- Change class: **normative-semantic Lane-CTR compatibility** (freezes the
  public/private boundary **without** adding or changing a public machine
  contract)
- Task anchor: `P10-T02`
- Executed under: `lease/personal/P10-T02/lane-ctr-compatibility`
- Completes the compatibility deferral in
  [ADR-0057](0057-personal-2-0-mcp-resource-family.md) §6
- Related: ADR-0006, ADR-0037, ADR-0043, ADR-0050, ADR-0056, ADR-0057,
  P5-T03, P5-T04, P8-T12, P10-T03, P10-T05

## Context

[ADR-0056](0056-personal-2-0-desktop-control-plane.md) and
[ADR-0057](0057-personal-2-0-mcp-resource-family.md) adopted Personal 2.0
product semantics: a desktop Control Plane, vendor-specific conversation
adapters behind a common internal capability matrix, and MCP as a seventh
product family. Both ADRs deferred the **public-contract and compatibility
surface** to Lane-CTR in `P10-T02`.

The existing machine surfaces that this decision must not disturb are:

- Core `ConversationBinding` (`conversation-binding.schema.json`), which is
  already a public identity binding and must not grow vendor transcript fields;
- the Personal-private six-family projection
  `GET /resource/v1/projection?family=&version=1`, whose family vocabulary is
  exactly `memory|skill|tool|context|task|runtime`;
- the P5-T03 MCP Tool adapter and P5-T04/B10 dynamic Tool path, which remain
  Tool-transport observations and are not seventh-family identities;
- the Lane-CTR standing rule that unified Personal projection stays private +
  versioned until a second real adapter/client justifies a minimal public
  `ResourceSummary`, and that no giant `Resource` schema or `Process` domain is
  added for this work.

`P10-T03` and `P10-T05` need an explicit boundary before they persist MCP
family rows or conversation/history projections. Freezing a public Core
schema now would either invent a generic `Resource` bucket or freeze vendor
conversation shapes into Core — both rejected by ADR-0056/0057.

## Decision

### 1. No public machine-contract change in this batch

`P10-T02` **does not** add or modify any Core public schema, transition,
registered error, generated Rust/TS binding, or conformance vector.

MCP family identities and the common conversation/history projection are
**Personal-private, versioned envelopes**. They are not Core contracts. A later
public surface requires a **new** Lane-CTR decision and must ship schema,
generated bindings, registered errors, transitions, and focused negatives in
the same batch (ADR-0006). This ADR is not that public surface.

The existing Core `Conversation` / `ConversationBinding` identities remain
the only public conversation identity. Vendor-native conversation and thread
IDs stay opaque origin bindings. No second public Conversation model is
created.

No generic public `Resource` DTO, table, lifecycle, or catch-all API is
created. `CognitiveResourceManifest` remains the existing
ActivityContext-filtered discovery manifest.

### 2. Envelope identifiers

Implementations of `P10-T03` and `P10-T05` MUST name private envelopes with
these exact identifiers and MUST fail closed on any other value:

| Envelope | Identifier | Owner |
|---|---|---|
| MCP family projection | `cognitiveos.personal.mcp-family/0.1` | Personal daemon |
| Common conversation/history projection | `cognitiveos.personal.conversation-projection/0.1` | Personal daemon |

These identifiers are Personal-private. They MUST NOT appear as Core schema
`$id` values or as a seventh `family=` query value on the 1.0 six-family
projection. Envelope version `0.1` is not a public compatibility promise; a
later private revision uses a new identifier and MUST NOT silently coerce an
older client.

### 3. MCP family identities stay distinct and Personal-private

The seven MCP identities from ADR-0057 remain distinct even when one package
starts one server over one connection:

1. **server** — logical MCP endpoint and declared protocol identity;
2. **package** — installed or acquired bytes, version, digest, origin, and
   adapter/transport association when a package exists;
3. **connection** — one configured transport endpoint and current bounded
   connection facts (no secret material);
4. **capability** — the server-advertised set plus the exact observed
   revision/digest from which candidates were derived;
5. **binding** — explicit Agent, Task, workspace, or owner scope that may
   discover or request admitted capabilities;
6. **health** — bounded readiness, drift, timeout, and last-observation facts;
7. **quarantine** — reasoned isolation and requalification state.

Each identity has a daemon-issued Personal-private id. Binding does **not**
grant Tool, Context, or Skill authority. Health does **not** mean an advertised
tool is enabled. Quarantine isolates the MCP family object; it does not
silently revoke unrelated admitted Tool/Context/Skill objects (those families
keep their own admission and revocation).

Exact SQLite tables, HTTP routes, and transitions are `P10-T03` work. This ADR
only freezes that they are Personal-private and must not be stuffed into the
1.0 six-family projection or a Core schema.

### 4. Capability digest

A capability identity carries a **capability digest**: SHA-256 over the
RFC 8785 canonical JSON of the observed advertised capability set, using the
existing `cognitiveos.canonical-json/0.1` profile
([canonical-encoding-and-digest.md](../standards/canonical-encoding-and-digest.md)).

The digested object is Personal-private and MUST contain at least: origin
server identity, observed protocol/revision, and each advertised item's
kind (`tool` | `resource` | `prompt`) plus origin-stable name. Secret material,
raw credential values, and unbounded transcript bytes MUST NOT enter the
digest preimage.

A digest mismatch with a later observation is **drift**. Drift makes health
not current and is a quarantine *candidate*; it MUST NOT auto-enable a Tool or
auto-admit a Context/Skill source.

Advertised tools remain Tool candidates. MCP protocol `resources` remain
Context candidates. MCP prompts remain Skill candidates. Admission stays in
those families.

### 5. Common conversation projection

The common internal conversation/history projection:

- MAY reference Core `Conversation` / `ConversationBinding` when a Personal
  Conversation exists;
- MUST carry vendor conversation/thread identifiers only as opaque origin
  bindings;
- MUST redact secret-shaped content on the daemon side before persistence or
  HTTP;
- MUST isolate management-channel and task-channel reads;
- MUST treat history as observation, never as Task, Effect, verification, or
  acceptance authority.

This projection is the `cognitiveos.personal.conversation-projection/0.1`
envelope. `P10-T05` implements the first vertical slice (dsh Path B
adapter-backed transcription). This ADR does not freeze Goal, Plan revision,
attempt, or handoff public contracts; those remain `Requires-backend` for
their own tasks (`P10-T06`, `P10-T13`).

### 6. 1.0 six-family projection stays six-family (older-client fail-closed)

The current private projection family vocabulary is frozen for the 1.0
surface:

```text
memory | skill | tool | context | task | runtime
```

`GET /resource/v1/projection` and `GET /task/resource/v1/projection` MUST
continue to reject any other `family` query, including `mcp`, with
`RESOURCE_PROJECTION_FAMILY_INVALID`. An unsupported `version` query continues
to fail with `RESOURCE_PROJECTION_VERSION_UNSUPPORTED`.

Older 1.0 clients therefore never receive an MCP row disguised as Tool, never
see a fabricated seventh family on the six-family route, and never parse
unknown Core `ConversationBinding` fields (`additionalProperties: false`
remains).

`P10-T03` MUST expose MCP family facts on a **new** Personal-private route or
envelope, not by extending the 1.0 `family=` allowlist. `P10-T05` MUST expose
conversation/history facts on a **new** Personal-private route or envelope, not
by adding transcript fields to Core `ConversationBinding`.

Clients that do not understand envelope identifier
`cognitiveos.personal.mcp-family/0.1` or
`cognitiveos.personal.conversation-projection/0.1` MUST be refused that
envelope. Implementations MUST NOT down-convert those envelopes into a fake
six-family row or a Core ConversationBinding extension.

### 7. P5-era records do not auto-migrate

P5-T03 MCP Tool adapter sessions and P5-T04/B10 dynamic Tool records remain
**Tool-transport observations**. They do **not** automatically become MCP
family server, package, connection, capability, binding, health, or quarantine
identities.

`P10-T03` MAY observe those records as origin evidence when constructing a
new family identity through explicit governed admission. It MUST NOT rewrite
historical P5 rows in place, MUST NOT treat a successful MCP initialize as
family registration, and MUST NOT enable advertised tools from P5 transport
success.

Older P5 clients keep the Tool-transport path. If they send seventh-family
fields on a 1.0 six-family or Core ConversationBinding surface, the daemon
MUST fail closed.

### 8. Secret, authority, and claim boundaries

MCP clients, servers, packages, adapters, and conversation adapters remain
candidate or observation producers only. The Rust daemon remains the sole
authority writer. Connection credentials enter and remain in an approved
Secret Store. Raw secret material never reaches the Control Plane, Agent,
sidecar, package metadata, ordinary configuration, SQLite, logs, Context,
evidence, or chat.

This decision creates no Gate, support, release, Profile, B01, marketplace,
Provider-quality, or Agent-benefit claim. Linux/Personal 1.0 remains
finalized six-family. Existing B10 evidence stays bounded to its recorded MCP
Tool/dynamic ecosystem MVP.

## Consequences

- `P10-T03` may implement daemon-owned MCP family lifecycle against the
  private envelope in §2–§4 without waiting for a Core schema.
- `P10-T05` may implement Personal-private conversation/history projection
  against §5 without freezing a public conversation-history schema.
- The 1.0 six-family projection and Core `ConversationBinding` stay
  byte-compatible for older clients.
- A future public `ResourceSummary` or public MCP schema is still allowed, but
  only after a second real adapter/client need and a new Lane-CTR batch that
  includes schema, generated bindings, errors, transitions, and negatives.

## Rejected alternatives

1. **Add Core public MCP family schemas now.** Rejected because there is no
   second public consumer, ADR-0057 forbids a generic `Resource`, and a public
   freeze would block `P10-T03` behind unnecessary Core churn.
2. **Add `family=mcp` to the 1.0 projection allowlist.** Rejected because older
   1.0 clients would observe an unknown seventh family on a six-family route,
   violating fail-closed compatibility.
3. **Extend Core `ConversationBinding` with vendor transcript fields.**
   Rejected because vendor IDs must stay opaque origin bindings and Core
   `additionalProperties: false` is the older-client fail-closed mechanism.
4. **Auto-migrate P5-T03/P5-T04 rows into MCP family identities.** Rejected
   because those records are Tool-transport observations, not family
   lifecycle facts.
5. **Keep the compatibility question open.** Rejected because `P10-T03` and
   `P10-T05` cannot persist honest private rows while the public shape is
   still unmarked.

## Non-goals and non-claims

This ADR implements no MCP family store, conversation store, HTTP route, UI,
Core schema, transition, registered error, generated binding, migration job,
or negative vector beyond the Lane-CTR focused absence/allowlist checks in
`P10-T02`. It does not decide Goal, Plan revision, attempt, or handoff
contracts. It creates no Gate and makes no support, release, Profile,
benchmark, B01, or Agent-benefit claim.
