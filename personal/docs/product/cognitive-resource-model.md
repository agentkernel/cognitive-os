# Personal cognitive resource model

- Status: canonical current + adopted advanced target
- Decisions:
  [ADR-0037](../../../docs/adr/0037-personal-unified-cognitive-resource-substrate.md),
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md), and
  [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Current OPC requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v5**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v5.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](../../../clients/docs/design/opc-2.0/history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Architecture mapping:
  [Resource Manager](../architecture/resource-manager-architecture.md)

## 1. Preserved family boundary

Linux Personal 1.0 remains a six-family product:

1. Memory;
2. Skill;
3. Tool;
4. Context;
5. Task;
6. Runtime/Process.

ADR-0057 adopts MCP as a seventh Personal family, but no broad MCP family
manager exists. Personal 2.0 now includes Assistant-led, security-reviewed
MCP acquisition and per-Project/Member grants while keeping a general
marketplace/family console out of scope. Existing MCP Tool transport stays
Tool-family implementation. The current 1.0 private projection remains
six-family and rejects `mcp`; ADR-0058's fail-closed compatibility remains
unchanged. Architecture/formal-plan text that treats all MCP use as deferred is
pending reconciliation.

## 2. OPC objects are not generic Resources

The following are domain/governance concepts, not additional resource
families:

- Project, Charter, Goal, Metric, and Plan revision;
- Role Runtime Template and Project Member Runtime definition;
- Routine, Trigger, occurrence, Task Attempt, and Handoff;
- Personal-owned Conversation and episodic archive;
- Model Connection, Provider quota, usage, and source-labelled cost;
- Permission, Model, Artifact, Intent/Effect, Evidence, and Event.

They may reference cognitive resources and appear in common product views, but
they do not justify a generic public `Resource` DTO, universal SQLite table,
catch-all repository, or one lifecycle. Each family and domain object keeps its
own identity, ownership, retention, transitions, and failure semantics.

## 3. Family responsibilities

| Family | Responsibility | OPC relationship |
|---|---|---|
| Memory | admitted durable knowledge with provenance, scope, versions, conflict, expiry, correction, and forget | verified facts and explicit Owner decisions may be admitted from archive/Vault candidates |
| Skill | immutable work-method/instruction package and revision | shown as **Work method** by default; scripts remain inert without a Tool |
| Tool | registered executable action with availability, permission, budget, and dispatch policy | shown as **Executable action**; DSH/Pi native/base tools are not auto-admitted |
| Context | authorized, bounded Task input with provenance, freshness, omissions, and losses | archive/Vault/Memory fragments are filtered before ranking and selected per Attempt |
| Task | bounded admitted work, budget, Effect, evidence, verification, and acceptance | belongs to a Project/Goal/plan; retry creates preserved Attempts |
| Runtime/Process | artifact-through-execution identity and bounded host observations | DSH starts disposable Agent processes for Members; Member identity and completion do not live here |
| MCP | managed server/package/connection/capability/binding/health/quarantine | Assistant-led reviewed acquisition and scoped grants; not DSH base tools, a marketplace, or host-session control |

## 4. Candidate and authority flows

```text
Conversation/Vault/external source
  -> bounded candidate
  -> authorization + provenance + policy
  -> Context selection or Memory/Skill/Tool admission
```

DSH, Pi, an MCP server, a Skill, a connector, or a Project Member cannot
self-admit content or capability. Installing or connecting something grants no
filesystem, process, network, model, secret, Tool, Context, or host-session
authority.

External mutation remains persisted Intent/Effect work. Tool/MCP/engine success
is not Effect reconciliation or Task completion. Independent verification
remains separate from the actor.

## 5. Workspace, Vault, and archive

The Project Vault and conversation archive are source systems, not new resource
families. Their indexed fragments may become Context candidates. Semantic
Memory is created only through Memory admission and remains inspectable,
correctable, and forgettable.

Origin owns native content. Personal owns admitted policy, bindings, and
authority facts. A file resembling a plan or permission configuration creates
a candidate; it cannot mutate Project authority by being edited.

## 6. MCP acquisition and family boundary

MCP retains distinct server, package, connection, capability, binding, health,
and quarantine identities in the Personal-private envelope selected by
ADR-0058. Advertised operations remain Tool candidates, protocol resources
remain Context candidates, and prompts/instructions remain Skill candidates.

DSH's native MCP/base tools are disabled for the qualified 2.0 default path.
The Assistant may discover candidates; first installation and every permission
expansion require exact-version/permission confirmation after source, license,
hidden-instruction, prompt-injection, dependency, executable-code, network,
Secret, Tool-permission, and supply-chain review. Artifacts may be reused
globally, but grants are isolated per Project/Member, pinned, compatibility
tested on update, and rollback-capable. This behavior is **Requires-backend**
and does not enter by enabling DSH configuration.

## 7. Product language

Default surfaces use:

- Prompt -> work instruction;
- Skill -> work method;
- Tool -> executable action;
- MCP -> connected application and capability;
- Loop -> work cycle;
- Harness -> execution engine.

Exact family and contract terms remain in advanced inspectors. Language changes
do not change family ownership.

## 8. Non-claims

This model creates no new public contract, Resource family implementation,
Project aggregate, MCP manager, archive/index/retrieval, DSH tool admission,
Gate, release, Profile, support, or qualification claim.
