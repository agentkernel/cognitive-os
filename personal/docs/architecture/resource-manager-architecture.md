# Personal Resource Manager architecture

- Status: current six-family projection plus advanced MCP boundary
- Product pair: [Resource Manager design](../product/resource-manager-design.md)
- Decisions: ADR-0037, ADR-0057, ADR-0058, and ADR-0059

## 1. Current implementation

P8-T12 delivered a daemon-owned common management projection across Memory,
Skill, Tool, Context, Task, and Runtime/Process. It supports bounded
list/inspect/watch/bind/unbind/enable/disable/revoke where the owning domain
defines the operation. Generic create/install/execute/complete remains refused.

Family-specific identity, version, admission, execution, retention, removal,
channel, and failure rules remain authoritative. A projection cannot be written
back as one universal Resource.

## 2. OPC domain separation

Project, Charter, Goal, Plan, Role Blueprint, Assignment, Employee, Routine,
Trigger, Attempt, Handoff, Conversation, Vault, Provider account, and Budget
are not added as Resource families. They reference resources through typed
application ports.

Knowledge archive/Vault and indexes are source/derived layers. Retrieved
fragments become Context candidates; admitted semantic knowledge remains the
Memory family.

## 3. Advanced MCP family

MCP retains distinct server/package/connection/capability/binding/health/
quarantine identities in `cognitiveos.personal.mcp-family/0.1`. It is
Personal-private, advanced/deferred, and not the OPC P0 path.

Advertisements remain candidates into Tool, Context, and Skill. DSH native
MCP/base tools are disabled by default and cannot be used as an implicit MCP
family implementation. Connection grants no Tool, Context, workspace, model,
secret, or host-session authority.

## 4. Origin and conflict

Origin-owned content and Personal-owned admission/binding remain separate.
Observation is scoped. Write-back uses current expected version, persisted
Intent/Effect, fencing, reconciliation, and independent verification. Conflict
fails closed; no timestamp/model last-writer-wins.

## 5. Common projection

When available, projections carry identity/family, source/provenance,
version/digest, scope, freshness, availability/health, bindings, blocked
reason, allowed actions, and coverage. Missing facts are explicit and no common
UI label changes domain ownership.

## 6. Contract and non-claims

Current six-family implementation remains **Now**. OPC Project composition,
archive/index links, federated conflict, and MCP family runtime are
**Requires-backend**. Any public shape requires Lane-CTR. This chapter changes
no Linux 1.0 family count and creates no support, Gate, release, Profile, or
Agent-benefit claim.
