# ADR-0057: Personal 2.0 MCP Resource Family

- Status: Accepted (owner-directed, 2026-08-27)
- Date: 2026-08-27
- Decision owner: CognitiveOS Personal product owner
- Change class: **product-semantic + structural documentation** (adds a
  Personal 2.0 product family without changing a public machine contract)
- Task anchor: `P10-T01`
- Executed under: `lease/personal/P10-T01/desktop-mcp-semantics`
- Partially supersedes:
  [ADR-0037](0037-personal-unified-cognitive-resource-substrate.md) for the
  Personal 2.0 family count only
- Related: ADR-0038, ADR-0043, ADR-0050,
  [ADR-0056](0056-personal-2-0-desktop-control-plane.md), P5-T03, P5-T04,
  P8-T12, P10-T02, P10-T03, P10-T04

## Context

ADR-0037 defines exactly six user-visible families for the finalized
Linux/Personal 1.0 product. Its model deliberately keeps family-specific
identity, lifecycle, storage, and authority semantics separate and rejects a
universal `Resource` table or state machine.

Post-1.0 work delivered an MCP Tool transport adapter (`P5-T03`) and a bounded
dynamic Tool ecosystem/B10 MVP (`P5-T04`). Those deliveries treat MCP as an
interop source for Tool candidates. They do not give MCP server/package,
connection, capability, binding, health, or quarantine facts a first-class
Personal product identity and lifecycle.

For Personal 2.0, MCP must be governable as a product object in its own right,
not hidden inside a Tool descriptor or presented as a generic bucket for every
object called a "resource" by the MCP protocol.

## Decision

### 1. MCP is the seventh Personal 2.0 product family

Personal 2.0 has seven user-visible product resource families:

1. Memory;
2. Skill;
3. Tool;
4. Context;
5. Task;
6. Runtime/Process;
7. **MCP**.

This family-count change applies only to Personal 2.0. Linux/Personal 1.0
remains finalized with the six-family boundary, manifest, support policy, and
Gate evidence defined by ADR-0037 and its 1.0 decision set.

### 2. MCP owns integration identities and lifecycle

The MCP family owns the product identity and lifecycle of:

- **server identity** — the logical MCP endpoint and its declared protocol
  identity;
- **package identity** — installed or acquired bytes, version, digest, origin,
  and adapter/transport association when a package exists;
- **connection identity** — one configured transport endpoint and its current
  authenticated, bounded connection facts;
- **capability identity** — the server-advertised capability set and the exact
  observed revision/digest from which candidates were derived;
- **binding identity** — explicit Agent, Task, workspace, or owner scope that
  may discover or request admitted capabilities;
- **health identity** — bounded readiness, drift, timeout, and last-observation
  facts without secret material;
- **quarantine identity** — reasoned isolation and requalification state after
  drift, policy failure, unsafe behavior, or unresolved outcome.

These identities remain distinct even when one package starts one server over
one connection. The target lifecycle covers explicit acquire/import,
registration, inspection, connection, capability refresh, binding,
enable/disable, quarantine, requalification, reconciliation, and removal.
Exact transitions, errors, compatibility rules, and persisted representations
are not defined by this ADR; they require the Lane-CTR decision in `P10-T02`.

### 3. Advertised tools remain Tool candidates

An MCP server advertising a tool does not create an enabled Personal Tool and
does not grant dispatch authority. Each advertisement is an untrusted,
version-bound **Tool candidate** that must pass the existing Tool descriptor,
policy, scope, enablement, availability, budget, and drift checks before it can
be exposed through the Tool family.

Dispatch remains daemon-owned. External or irreversible mutations still use
persist-before-dispatch Intent/Effect, idempotency, fencing, and
reconciliation. A successful MCP protocol response is an observation, not an
Effect commit, independent verification, or Task completion.

### 4. MCP resources and prompts enter existing admission paths

The MCP protocol's named `resources` do not become instances of a generic
Personal `Resource` domain. Their content and references enter as Context
candidates and must pass Context source authorization, provenance, freshness,
budget, loss, and admission rules before use.

MCP prompts or reusable instruction packages enter as Skill candidates and
must pass Skill package/revision, provenance, binding, enablement, and
admission rules. A prompt name, resource URI, server advertisement, or
successful fetch grants no permission by itself.

Where one advertised object contains both contextual content and reusable
instruction semantics, the daemon routes each aspect through the applicable
Context and Skill admission paths rather than creating an MCP-owned authority
shortcut.

### 5. No generic Resource schema

This decision does not create a universal public `Resource` DTO, database
table, lifecycle, or catch-all API. It does not redefine
`CognitiveResourceManifest`, which remains the ActivityContext-filtered
discovery manifest defined by existing contracts.

MCP family projections compose MCP-specific identities with references to
admitted Tool, Context, and Skill objects. Those references do not merge the
families or transfer lifecycle authority between them.

### 6. Authority, secret, and compatibility boundaries

MCP clients, servers, packages, adapters, and remote peers are candidate or
observation producers only. The Rust daemon remains the sole authority writer.
Connection credentials enter and remain in an approved Secret Store; raw
secret material never reaches the Control Plane, Agent, sidecar, package
metadata, ordinary configuration, SQLite, logs, Context, evidence, or chat.

`P10-T02` decided the public-contract and compatibility surface under
Lane-CTR in [ADR-0058](0058-personal-2-0-mcp-conversation-private-projection.md):
MCP family identities and the common conversation projection stay
Personal-private versioned envelopes; no Core public schema is added; the 1.0
six-family projection and Core `ConversationBinding` remain fail-closed for
older clients; P5-era Tool-transport records do not auto-migrate.
`P10-T03` owns daemon authority and product integration against that private
envelope. `P10-T04` may expose the family in the desktop Control Plane only
after its required backend surface exists.

## Supersession and migration

ADR-0037 remains unchanged and authoritative for the finalized six-family
Linux/Personal 1.0 product. This ADR partially supersedes only its exact family
count for Personal 2.0 and preserves its family-separation and no-generic-
Resource rules.

The existing P5-T03 MCP Tool adapter and P5-T04/B10 dynamic Tool implementation
remain valid historical implementation evidence for their accepted scope.
They do **not** retroactively constitute the Personal 2.0 MCP family and do not
prove server/package/connection/binding/health/quarantine lifecycle support.
Migration or compatibility treatment of those records is decided by
[ADR-0058](0058-personal-2-0-mcp-conversation-private-projection.md)
(no automatic promotion into seventh-family identities). Current
implementation of the seventh-family model remains absent until `P10-T03`.

## Consequences

- Personal 2.0 navigation, planning, support policy, and future release
  manifests may identify MCP as the seventh family.
- MCP integration health and quarantine can be supervised without pretending
  every advertised item is already an enabled Tool.
- Tool, Context, and Skill retain their own authority and lifecycle semantics.
- Existing B10 evidence remains bounded to its recorded MCP Tool/dynamic
  ecosystem MVP and does not become seventh-family support evidence.
- A structural public contract cannot be inferred from this documentation
  decision; [ADR-0058](0058-personal-2-0-mcp-conversation-private-projection.md)
  is the Lane-CTR compatibility boundary and keeps the family Personal-private.

## Rejected alternatives

1. **Keep MCP only as a Tool transport forever.** Rejected because connection,
   package, capability, binding, health, and quarantine are product lifecycle
   facts, not Tool descriptor fields.
2. **Treat every MCP-advertised object as one generic Resource.** Rejected
   because tools, contextual content, and reusable instructions require
   different admission and authority semantics.
3. **Auto-enable advertised tools after a successful connection.** Rejected
   because discovery and protocol completion grant no Tool authority.
4. **Count P5-T03/P5-T04 as the seventh-family implementation.** Rejected
   because those tasks did not implement the family identities and lifecycle
   accepted here.

## Non-goals and non-claims

This ADR implements no MCP family store, route, UI, contract, schema,
transition, registered error, migration, compatibility adapter, or negative
vector. It creates no Gate and makes no support, release, Profile, benchmark,
B01, marketplace, Provider-quality, or Agent-benefit claim.
