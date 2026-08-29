# MCP capability acquisition and governance

- Status: adopted Personal 2.0 Project capability path; broad family console
  remains out of scope
- Canonical language: English
- Decisions:
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md),
  [ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md),
  and [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v9**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](../../../clients/docs/design/opc-2.0/history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Prototype identity: owner-approved 2026-08-30 current chrome is
  personal-20-opc-e2e-optimized-v9. v8 is the prior approved baseline (not overwritten). Archived V2 is not current chrome. Canvas-only HITL and daemon authority path remain.
- Chinese mirror: [mcp-resource-family.zh-CN.md](mcp-resource-family.zh-CN.md)

## 1. Scope

The Personal Assistant may discover an MCP capability when Project setup or
operation identifies a need. Security-reviewed acquisition, exact-version
pinning, and separate Project/Member grants are in the Personal 2.0 target;
they are not fully deferred. Skills use a different path: they may auto-install
only after the same class of source/prompt-injection review. MCP is stricter:
first installation or any permission expansion still needs Owner confirmation
of exact version and permissions.

The retained underlying model keeps server, package, connection, capability,
binding, health, and quarantine identities distinct. MCP is not a Tool alias,
Agent, Project object, Provider route, or host-session controller. A general
marketplace or family-management console remains outside 2.0.

## 2. Current truth

Linux Personal 1.0 remains six-family. P5 MCP Tool transport and dynamic Tool
work stay inside the Tool family. They do not implement an MCP family manager,
server lifecycle, health/permission/update projection, or general client
configuration.

ADR-0058 keeps MCP in
`cognitiveos.personal.mcp-family/0.1`, a Personal-private envelope. The existing
six-family projection continues to reject `mcp`. No Core schema, generic
`Resource`, or older-client coercion is created here.

The 2026-08-27 architecture/formal-plan text that describes MCP as fully
deferred is **pending architecture/plan reconciliation**. This product document
does not modify the accepted private-envelope compatibility decision.

## 3. Discovery and security review

The Assistant may perform broad web discovery without asking for every
ordinary read. Every candidate remains untrusted. Before acquisition, the
review records:

- source, exact version, digest, license, maintainer/provenance, and update
  channel;
- hidden instructions and prompt-injection content;
- dependencies and executable-code/supply-chain intent;
- requested filesystem, network, command, Secret, model, and Tool permissions;
- connection transport and external destinations;
- compatibility evidence, removal, update, rollback, and quarantine behavior.

External text cannot execute, install, or expand permission. Raw credentials
and third-party data the Owner cannot disclose are excluded from research.

## 4. Acquisition, grant, and admission

First installation and every permission expansion require Owner confirmation
of the exact version and permission set. Acquisition creates a globally
reusable pinned artifact only. Every Project/Member receives a separate,
least-privilege grant; revoking one grant does not silently remove unrelated
uses.

Install/connect grants no Tool, Context, workspace, network, command, model,
secret, Memory, or host-session authority by implication.

MCP advertisements remain untrusted candidates:

- operation -> Tool candidate and Tool admission;
- resource/data -> Context candidate and authorization-before-ranking;
- prompt/instruction -> Skill candidate and immutable revision admission;
- returned content -> no automatic Memory;
- process/connection state -> no Runtime or Task authority.

DSH's native MCP and base tools are disabled in the default 2.0 managed path.
Personal must not qualify MCP indirectly by enabling DSH configuration.

## 5. Version, update, and client projection

Projection into a client configures only the exact admitted server/binding. It
does not control the Agent's live session. Configuration write-back remains a
persist-before-dispatch Intent/Effect operation with version checks,
reconciliation, and receipt.

Versions are pinned. Updates repeat source/security review, run compatibility
tests against affected grants, show exact changed permissions and destinations,
and retain a rollback path. An unchanged, previously approved low-risk
configuration class may automate inside its exact grant. A new client,
permission expansion, changed endpoint, broader filesystem/network scope, or
conflict requires a fresh preview. Conflict fails closed; timestamps and model
judgment cannot resolve it alone.

## 6. Secrets and Provider traffic

Server and client credentials stay in approved Secret Stores and daemon
proxies. Raw credentials never enter MCP package metadata, ordinary config,
DSH/Pi environment, Agent messages, Context, Memory, evidence, or logs.
Connection health cannot prove Provider reachability or model availability.

## 7. Required states

The target distinguishes discovered/reviewing, confirmation-required,
installing/connecting, grant-required, partial client projection, unhealthy,
permission denied, drifted, stale, conflict, quarantined, update available,
compatibility failed, rollback available, outcome unknown, and
removed-with-history states. Each percentage or count names its denominator.

## 8. Non-claims

Discovery, review, acquisition, grants, family runtime behavior, and client
projection are **Requires-backend**; external execution also requires the
applicable environment qualification. This document does not implement or
qualify an MCP server, Tool, Agent, DSH path, client, marketplace, support row,
Gate, release, Profile, or ecosystem claim.
