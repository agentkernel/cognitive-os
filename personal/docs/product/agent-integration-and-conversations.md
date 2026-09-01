# Personal Assistant, Project Members, and governed conversations

- Status: adopted Personal 2.0 product target
- Canonical language: English
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v9**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](../../../clients/docs/design/opc-2.0/history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Prototype identity: owner-approved 2026-08-30 current chrome is
  personal-20-opc-e2e-optimized-v9. v8 is the prior approved baseline (not overwritten). Archived V2 is not current chrome. Canvas-only HITL and daemon authority path remain.
- Existing architecture inputs:
  [Agent lifecycle](../architecture/agent-shell-and-agent-lifecycle.md) and
  [Project, Role, and Employee](../architecture/project-role-employee.md)
- Chinese mirror:
  [agent-integration-and-conversations.zh-CN.md](agent-integration-and-conversations.zh-CN.md)

## 1. Three separate product identities

| Identity | Product role | Authority boundary |
|---|---|---|
| **Personal Assistant** | global explanation, navigation, research, and proposal surface | candidate-only; daemon issues every confirmable preview |
| **Project Member Runtime definition** | long-lived Project-specific responsibility, Conversation, Memory, work, grants, and history | not an Agent process; work authority remains daemon-owned |
| **Agent process / Attempt** | disposable execution started from an exact Member revision for one Task | bounded executor/observer; no Project identity, long-term Memory, secret, or completion ownership |

Collapsing these identities creates false lifecycle and trust claims. Restarting
a process does not replace a Member. A conversation message does not update a
Project. An acquired package grants no execution permission.

## 2. Personal Assistant and Pi

The Personal Assistant is the user-visible system identity. It has the highest
UX privilege: it may see available product facts and initiate every management
flow. It still writes only through a daemon-issued preview, Owner confirm, and
receipt. It can:

- inspect available product facts and explain a Project, Member, attention
  item, source, uncertainty, or conflict;
- navigate to the exact object;
- conduct guided Project/role research;
- draft charter, plan, Role, Model Connection, capability, or recovery
  candidates;
- initiate every management flow without becoming its writer;
- request a daemon-issued structured preview;
- explain a receipt and remaining decision.

Pi may support this experience internally as a fixed, managed, default-deny
engine. Pi is hidden from ordinary navigation. It owns no
authority, Provider secret, Project, Task, long-term Conversation, episodic
archive, semantic Memory, or completion. Pi output remains a candidate.

Explanations show source, scope, freshness, limitations, and uncertainty.
Personal does not expose model chain-of-thought or invented numerical
confidence. A suggestion cannot be confirmed until the daemon resolves it into
an exact preview.

## 3. Hidden managed DSH execution engine

DeepSeek Harness is the hidden default engine for Project Member Task
processes. It is not an Installed Agent product object, user-selectable
Harness, or everyday destination. Only fault recovery and advanced diagnostics
may expose:

- exact official artifact source, version, digest, license, and admission;
- adapter/broker version and protocol compatibility;
- Windows host/sandbox qualification boundary;
- current health and bounded capabilities;
- update availability, compatibility changes, and rollback slot;
- which Members and Tasks currently use it.

It is not an in-process daemon library and not a vendored fork. Personal runs
the exact audited artifact as an isolated child process behind a bounded stdio
broker. DSH has no direct authority database, SecretStore, Provider credential,
ambient environment secret, native MCP/base-tool, HMR, or home-patch access.
Provider traffic is daemon-proxied and executable actions pass Personal
admission.

Personal does not embed DSH's native UI or synchronize native DSH
conversations. The Member's Conversation, archive, Memory, Task, Context,
and evidence belong to Personal. DSH receives a bounded Context payload and
returns candidates/observations.

The existing post-1.0 dsh Path B implementation is reusable evidence only
within its recorded scope. It does not qualify this Windows-managed artifact,
sandbox, supply chain, or product experience.

## 4. Project group and Member work conversations

Outside a Project the conversational identity is the global Personal
Assistant. Inside a Project, the primary surface is the group containing
Owner, manager, and Members. The manager speaks by default. A Member speaks
proactively only when mentioned, submitting a deliverable, handing off,
blocked, or requesting a decision. `@manager` can request progress/delegation;
`@member` can ask or temporarily redirect bounded goal/path. `@member` creates
a formal Task revision, not a shadow plan.

Personal also retains scoped Member work conversations as inspectable source
records. A Member work conversation is visible to the Owner, the manager, and
that Member. Conversations may contain user messages, bounded retrieved Context,
engine output, action proposals, receipts, and source links. The full local
archive remains inspectable, while an Agent process receives only relevant,
bounded, redacted, provenance-bearing observations.

Conversation is not authority:

1. Member or manager output is a candidate;
2. ordinary discussion can remain conversational;
3. a work-changing message becomes a formal Task or revision; a
   Project/plan/team/Provider/model/Tool/MCP/permission/external-rule change
   requests a daemon preview;
4. the Owner confirms, edits, narrows, or rejects;
5. the applied revision and receipt return to the Conversation and object page.

Agent final text, process exit, Tool result, Provider response, manager
agreement, or engine checkpoint is not Task completion.

## 5. Composer and authority handoff

The visible composer posts to the currently named context: global Personal
Assistant outside a Project or the selected Project group inside one.

- Project/Assistant context switching preserves independent unsent drafts;
- switching cannot merge, clear, or send text;
- `@` routing inserts only into the unsent draft and never bypasses the Project
  scope or approved envelope;
- contextual approval opens a structured daemon preview on the center canvas,
  not a second chat authority; chat has no Approve control and no “Don’t ask
  again” grant;
- offline and permission states preserve draft content;
- ordinary execution traces remain folded behind Tasks/Attempts.

This avoids accidental cross-Project or Assistant/group dispatch while
preserving one understandable conversation model.

## 6. Member and process lifecycle

The following remain separate:

`Role Runtime Template -> Project Member Runtime definition -> Task -> Attempt -> Agent process`

The daemon owns artifact admission, Member revision activation, process
identity, execution epoch, fencing, health interpretation, update, rollback,
and removal. Process liveness is only an observation.

Stopping or losing a process preserves Member identity, group and work
Conversations, Memory, work, Attempts, and evidence. Process death does not
delete the Member. Updating or rolling back
DSH is an advanced managed-artifact operation with impact preview and cannot
silently delete Personal history.

A Member Task process may create bounded internal subagents with explicit
count, time, cost, and permission limits. They are disposable helpers, not
Project Members, and retain no long-lived identity or Memory.

## 7. Alternative engines are outside 2.0

Personal 2.0 targets only DSH as a Member execution engine. Hermes, Codex,
Cursor,
and other products are future adapter candidates. Each needs exact artifact,
license, protocol, capability, secret, sandbox, lifecycle, platform, negative,
and independent qualification evidence. No DSH or Pi evidence transfers.

The retained generic adapter architecture may support future work, but there is
no 2.0 promise of multiple external engines, native conversation
synchronization, or a vendor-neutral runtime contract.

## 8. Required states

Assistant, Project-group, Member-work, and advanced-diagnostics surfaces cover
empty, loading, partial, stale, permission, error, unknown, offline,
long-running, success, and archived states. DSH-specific diagnostic examples
include artifact unavailable, digest mismatch, compatibility unknown, sandbox
unqualified, broker failed, Provider unavailable, update pending, rollback
available, and outcome unknown.

An unimplemented lifecycle action is `Requires-backend`, not a disabled control
that implies an existing operation.

## 9. Fixed non-claims

This target does not establish a Windows DSH package, qualification, sandbox,
native Provider support, managed child process, archive, conversation UI,
Personal Assistant, Member Runtime, another adapter, support, Gate, release,
Profile, or multi-Agent benefit.
