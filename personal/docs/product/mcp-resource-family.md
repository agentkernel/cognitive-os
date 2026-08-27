# MCP resource family

- Status: adopted advanced family; deferred from the Personal 2.0 OPC P0 path
- Canonical language: English
- Decisions:
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md),
  [ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md),
  and [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Chinese mirror: [mcp-resource-family.zh-CN.md](mcp-resource-family.zh-CN.md)

## 1. Scope

MCP remains the adopted seventh Personal product family, with distinct server,
package, connection, capability, binding, health, and quarantine identities.
It is not a Tool alias, an Agent, a Project object, a Provider route, or a host
session controller.

For the Windows OPC rebaseline, MCP is an **advanced/deferred capability**. It
does not block Project setup, the Personal Assistant, DSH-backed employees,
Conversation/Memory, Routines, Inbox, Knowledge, Provider routing, the UI, or
the first X/Twitter acceptance scenario.

## 2. Current truth

Linux Personal 1.0 remains six-family. P5 MCP Tool transport and dynamic Tool
work stay inside the Tool family. They do not implement an MCP family manager,
server lifecycle, health/permission/update projection, or general client
configuration.

ADR-0058 keeps MCP in
`cognitiveos.personal.mcp-family/0.1`, a Personal-private envelope. The existing
six-family projection continues to reject `mcp`. No Core schema, generic
`Resource`, or older-client coercion is created here.

## 3. Family behavior

An MCP family surface may eventually show:

- exact source, version, digest, license, and acquisition/admission;
- connection transport without secret material;
- capability-set digest and drift;
- health separately from permission and Task eligibility;
- admitted bindings and projected compatible clients;
- update, rollback, quarantine, requalification, and removal;
- conflicts and durable receipts.

Install/connect grants no Tool, Context, workspace, network, model, secret, or
host-session authority.

## 4. Capability admission

MCP advertisements are untrusted candidates:

- operation -> Tool candidate and Tool admission;
- resource/data -> Context candidate and authorization-before-ranking;
- prompt/instruction -> Skill candidate and immutable revision admission;
- returned content -> no automatic Memory;
- process/connection state -> no Runtime or Task authority.

DSH's native MCP and base tools are disabled in the default 2.0 managed path.
Personal must not qualify MCP indirectly by enabling DSH configuration.

## 5. Client configuration and write-back

Projection into a client configures only the exact admitted server/binding. It
does not control the Agent's live session. Configuration write-back remains a
persist-before-dispatch Intent/Effect operation with version checks,
reconciliation, and receipt.

An unchanged, previously approved low-risk configuration class may automate
inside its exact grant. A new client, permission expansion, changed endpoint,
broader filesystem/network scope, or conflict requires a fresh preview.
Conflict fails closed; timestamps and model judgment cannot resolve it alone.

## 6. Secrets and Provider traffic

Server and client credentials stay in approved Secret Stores and daemon
proxies. Raw credentials never enter MCP package metadata, ordinary config,
DSH/Pi environment, Agent messages, Context, Memory, evidence, or logs.
Connection health cannot prove Provider reachability or model availability.

## 7. Required states

The eventual product distinguishes empty, installing/connecting, partial client
projection, unhealthy, permission denied, drifted, stale, conflict,
quarantined, update available, rollback available, outcome unknown, and
removed-with-history states. Each percentage or count names its denominator.

## 8. Non-claims

All family runtime behavior is **Requires-backend** and deferred. This document
does not implement or qualify an MCP server, Tool, Agent, DSH path, client,
marketplace, support row, Gate, release, Profile, or ecosystem claim.
