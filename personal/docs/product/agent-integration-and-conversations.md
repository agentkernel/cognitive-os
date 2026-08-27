# Personal Assistant, Installed Agents, and employee conversations

- Status: adopted Personal 2.0 product target
- Canonical language: English
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Architecture:
  [Agent lifecycle](../architecture/agent-shell-and-agent-lifecycle.md) and
  [Project, Role, and Employee](../architecture/project-role-employee.md)
- Chinese mirror:
  [agent-integration-and-conversations.zh-CN.md](agent-integration-and-conversations.zh-CN.md)

## 1. Three separate product identities

| Identity | Product role | Authority boundary |
|---|---|---|
| **Personal Assistant** | global explanation, navigation, research, and proposal surface | candidate-only; daemon issues every confirmable preview |
| **Digital Employee** | long-lived Project member with responsibility, Conversation, Memory, work, and history | not an Agent process; work authority remains daemon-owned |
| **Installed Agent / Runtime** | qualified execution integration used by employees | bounded executor/observer; no Project, Memory, secret, or completion ownership |

Collapsing these identities creates false lifecycle and trust claims. Restarting
a runtime does not replace an employee. A conversation message does not update
a Project. An installed package grants no execution permission.

## 2. Personal Assistant and Pi

The Personal Assistant is the user-visible system identity. It can:

- explain a Project, employee, Inbox item, source, uncertainty, or conflict;
- navigate to the exact object;
- conduct guided Project/role research;
- draft charter, plan, role, binding, budget, or recovery candidates;
- request a daemon-issued structured preview;
- explain a receipt and remaining decision.

Pi may support this experience internally as a fixed, managed, default-deny
engine. Pi is hidden from the ordinary Installed Agents list. It owns no
authority, Provider secret, Project, Task, long-term Conversation, episodic
archive, semantic Memory, or completion. Pi output remains a candidate.

Explanations show source, scope, freshness, limitations, and uncertainty.
Personal does not expose model chain-of-thought or invented numerical
confidence. A suggestion cannot be confirmed until the daemon resolves it into
an exact preview.

## 3. Preinstalled managed DSH Installed Agent

DeepSeek Harness is supplied with Personal 2.0 as the **preinstalled managed
Installed Agent** and default runtime for Project digital employees. It remains
visible under Settings > Installed Agents so the Owner can inspect:

- exact official artifact source, version, digest, license, and admission;
- adapter/broker version and protocol compatibility;
- Windows host/sandbox qualification boundary;
- current health and bounded capabilities;
- update availability, compatibility changes, and rollback slot;
- which employees and Tasks currently use it.

It is not an in-process daemon library and not a vendored fork. Personal runs
the exact audited artifact as an isolated child process behind a bounded stdio
broker. DSH has no direct authority database, SecretStore, Provider credential,
ambient environment secret, native MCP/base-tool, HMR, or home-patch access.
Provider traffic is daemon-proxied and executable actions pass Personal
admission.

Personal does not embed DSH's native UI or synchronize native DSH
conversations. The employee's Conversation, archive, Memory, Task, Context,
and evidence belong to Personal. DSH receives a bounded Context payload and
returns candidates/observations.

The existing post-1.0 dsh Path B implementation is reusable evidence only
within its recorded scope. It does not qualify this Windows-managed artifact,
sandbox, supply chain, or product experience.

## 4. Employee conversations

Each Personal-owned Conversation is scoped to Owner, Project, and employee.
It may contain user messages, bounded retrieved Context, engine output,
tool/action proposals, receipts, and source links. The archive is local and
indexed, but retrieval injects only relevant, bounded, redacted,
provenance-bearing observations.

Conversation is not authority:

1. employee or manager output is a candidate;
2. ordinary discussion can remain conversational;
3. a Project/plan/team/budget/provider/tool/permission/external-rule change
   requests a daemon preview;
4. the Owner confirms, edits, narrows, or rejects;
5. the applied revision and receipt return to the Conversation and object page.

Agent final text, process exit, Tool result, Provider response, manager
agreement, or engine checkpoint is not Task completion.

## 5. One active composer

The right rail allows conversation with the Personal Assistant, Project
Manager, or an employee. Exactly one composer is active:

- the recipient identity is visible in the composer label and submit action;
- selecting another recipient switches contexts but preserves both drafts;
- no draft is merged, cleared, or sent on switch;
- the active composer has one keyboard focus owner;
- an Inbox approval opens a structured preview, not a second chat composer;
- offline and permission states preserve draft content.

This avoids accidental cross-Project or assistant/employee dispatch.

## 6. Runtime lifecycle

The following remain separate even when one process participates in several:

`Artifact -> Installation -> Agent definition -> Runtime instance -> Task execution -> OS process -> Conversation`

The daemon owns artifact admission, installation activation, employee/runtime
binding, execution epoch, budget, fencing, health interpretation, update,
rollback, and removal. Process liveness is only an observation.

Disconnecting an employee from a runtime preserves employee identity,
Conversation, Memory, work, and evidence. Uninstalling DSH is a managed
artifact operation with an impact preview and cannot silently delete Personal
history.

## 7. Future adapters

Personal 2.0 qualifies only DSH as an employee runtime. Hermes, Codex, Cursor,
and other products are future adapter candidates. Each needs exact artifact,
license, protocol, capability, secret, sandbox, lifecycle, platform, negative,
and independent qualification evidence. No DSH or Pi evidence transfers.

The retained generic adapter architecture may support future work, but there is
no 2.0 promise of multiple external engines, native conversation
synchronization, or a vendor-neutral runtime contract.

## 8. Required states

Installed Agent and Conversation surfaces cover empty, loading, partial,
stale, permission, error, unknown, offline, long-running, success, and archived
states. DSH-specific examples include artifact unavailable, digest mismatch,
compatibility unknown, sandbox unqualified, broker failed, Provider unavailable,
update pending, rollback available, and outcome unknown.

An unimplemented lifecycle action is `Requires-backend`, not a disabled control
that implies an existing operation.

## 9. Fixed non-claims

This target does not establish a Windows DSH package, qualification, sandbox,
native Provider support, managed child process, archive, conversation UI,
Personal Assistant, employee runtime, another adapter, support, Gate, release,
Profile, or multi-Agent benefit.
