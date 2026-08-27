# Resource Manager product design

- Status: current six-family envelope plus adopted OPC composition
- Architecture pair:
  [resource-manager-architecture.md](../architecture/resource-manager-architecture.md)
- Current task evidence: formal plan `P8-T12`
- Current-status owner: [PROGRESS.md](../../../docs/plan/PROGRESS.md)

## 1. Current implementation (Now)

The current private six-family projection and management envelope support
bounded list, inspect, watch, bind, unbind, enable, disable, and revoke where
the selected family defines the operation. Context and Runtime may be
projection-only or empty. Generic create/install/execute/complete are refused.

The envelope is a read/command projection, not a universal Resource aggregate.
Every mutation resolves to a typed family workflow under daemon authority.
Unknown, unavailable, stale, not-backed, and empty remain different states.

## 2. OPC product composition

Personal 2.0 business surfaces compose resource facts around Projects:

- Project briefing links Tasks, Context, artifacts, evidence, and cost;
- employee detail links Runtime, effective Tool/Skill/Memory bindings, and
  current Context;
- Knowledge links source archive/Vault to derived index and admitted Memory;
- Settings Advanced links MCP, Tool/Skill details, and family governance.

Project, Role, Employee, Routine, Attempt, Conversation, Vault, Provider, and
Budget are not added to the Resource Manager family allowlist. The Resource
Manager does not become the Project repository.

## 3. Common operation vocabulary

| Operation | Shared meaning | Explicit exclusion |
|---|---|---|
| list | bounded page for one declared family/version | unbounded dump or cross-family search |
| inspect | stable identity and available projected facts | generic edit |
| watch | resume the family cursor with honest coverage | fabricated unified live feed |
| bind/unbind | typed guarded relationship | create/delete domain history |
| enable/disable | change eligibility under family policy | execute/install/uninstall |
| revoke | invalidate a grant/binding/revision | Memory forget or source deletion |

Acquisition, import, Memory admission, Tool execution, MCP lifecycle,
Intent/Effect reconciliation, Project activation, and permanent deletion stay
family/domain-specific.

## 4. Knowledge and retrieval boundary

Archive/Vault indexing is derived. A Resource projection may show provenance,
index freshness, conflict, and the admitted Memory/Skill/Tool object reached
from a source, but it cannot treat an index row as authority.

Retrieval applies scope and policy before ranking. A source edit that appears
to change Project configuration becomes a candidate. Missing index or parse
failure does not authorize an Agent cache as fallback truth.

## 5. Origin and conflict behavior

Origin-owned native content and Personal-owned admission remain separate:

1. read/change detection is limited to an explicit observation scope;
2. Personal records source/version/freshness;
3. write-back is a daemon Intent/Effect operation;
4. unchanged low-risk policy may allow automatic admitted work;
5. new, broader, destructive, or conflicted scope requires preview;
6. conflicts fail closed—no timestamp/model last-writer-wins.

The Personal Assistant can explain the conflict but cannot resolve or write it.

## 6. Advanced MCP boundary

MCP is a separately managed advanced family under ADR-0057/0058, not a Tool
alias or DSH native/base-tool grant. Server/package/connection/capability/
binding/health/quarantine stay distinct. Capabilities pass Tool, Context, or
Skill admission before use.

The MCP family manager and client projection are **Requires-backend** and
deferred from the 2.0 OPC success path.

## 7. Channels and non-claims

Management mutations use the management channel; Task use stays Task-scoped.
The deterministic CLI, UI, Personal Assistant, DSH, Pi, and adapters are
clients. Partial watch coverage is never shown as complete.

This design changes no private/public envelope, Core schema, family allowlist,
route, store, or transition. It makes no Project, MCP, synchronization, support,
Gate, release, Profile, performance, or Agent-benefit claim.
