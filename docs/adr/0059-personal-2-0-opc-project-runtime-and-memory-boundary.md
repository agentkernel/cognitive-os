# ADR-0059: Personal 2.0 Windows-first OPC product, project, runtime, and memory boundary

- Status: Accepted (owner-directed, 2026-08-27)
- Date: 2026-08-27
- Decision owner: CognitiveOS Personal product owner
- Change class: **product-semantic** with architecture and planning
  follow-through; no CognitiveOS public machine-contract change
- Delivery: `DOC-PERSONAL-2.0-OPC` / documentation-only `P11-T01/D01`
- Related:
  [ADR-0035](0035-personal-pi-shell-and-managed-agent-role-separation.md),
  [ADR-0043](0043-personal-universal-agent-adapter.md),
  [ADR-0044](0044-personal-multi-agent-mainline.md),
  [ADR-0053](0053-personal-web-ui-stack.md),
  [ADR-0055](0055-personal-credential-import-boundary-and-a5-revision.md),
  [ADR-0056](0056-personal-2-0-desktop-control-plane.md),
  [ADR-0057](0057-personal-2-0-mcp-resource-family.md), and
  [ADR-0058](0058-personal-2-0-mcp-conversation-private-projection.md)

## Context

ADR-0056 adopted a desktop Control Plane organized around installed Agents,
generic Work, Library, and Activity. ADR-0058 then kept MCP and common
conversation projections Personal-private and chose a dsh Path B transcript as
the first conversation slice. Those decisions remain auditable, but the owner
has now fixed a narrower product job: Personal 2.0 is the Windows-local
operating console for a one-person company or individual developer. The user
organizes governed projects and long-lived digital employees in business
language; they do not supervise a gallery of unrelated native Agent products.

Two owner-approved external Canvas artifacts informed the decision:

| Artifact | Filesystem observation (UTC) | SHA-256 |
|---|---|---|
| `personal-2-opc-requirements-baseline.canvas.tsx` | 2026-08-27 11:18:24.5228820 | `e5e8a93a20389c27939fba6e9b094f474b3023ae655c2cea1415ab6ae5652054` |
| `personal-2-opc-research-migration-plan.canvas.tsx` | 2026-08-27 12:31:50.8803276 | `4892cb827d98f4ab850826e6317d5f481221809fb8393135d3885ee58e5ae292` |

They are informative provenance, not canonical sources. Their confirmation
gates and proposed deletion of older Canvas files were superseded by the
owner's approval and amendments in this session. In particular, the final
decision keeps the old Canvas files and preserves **Installed Agent** semantics
for DeepSeek Harness (DSH).

## Decision

### 1. Version, principal, and product boundary

Personal 2.0 is **Windows-first** and its formal product boundary is a
Windows-local project loop while the host is online. Personal 2.1, not 2.0,
owns native mobile clients, device pairing, and an end-to-end encrypted relay.
Linux Personal 1.0 remains a finalized, separate historical product boundary;
its support, six-family model, Pi qualification, and Gate evidence are not
reinterpreted.

There is one local human **Owner**. Projects, roles, and digital employees
belong directly to that Owner. Optional business or brand information is a
profile input, not a current `Company`, `Organization`, or `Business Space`
aggregate. Human teams, multi-tenant RBAC, and cloud authority are out of
scope.

The primary information architecture is:

**Today / Projects / Team / Knowledge / Inbox**, with **Settings** fixed at the
bottom and a global right-side **Personal Assistant**. Project pages open to a
business briefing. Advanced implementation terms are progressively disclosed:
Prompt = work instruction, Skill = work method, Tool = executable action,
MCP = connected application/capability, Loop = work cycle, and Harness =
execution engine.

### 2. Project and digital-employee model

A `Project` is a governed long-term workspace, not a folder, chat, Agent,
workflow, or generic cognitive resource. It owns a confirmed charter, goals,
metrics, revisioned plan, approved team, permissions, budgets, triggers,
Tasks/Attempts, artifacts, handoffs, Effects, evidence, and reflection
candidates.

The people/execution chain is:

`Role Blueprint -> Project Role Assignment -> Digital Employee Instance -> Agent Runtime -> Personal-owned Conversation`

Only a **Project Manager base blueprint** is built in. Project initialization
specializes it without removing its governance obligations. The Personal
Assistant proposes all other role blueprints from project needs. Each project
has exactly one current manager. Managers coordinate through daemon-owned
Tasks, artifacts, and handoffs; free-form Agent agreement never transfers
authority or proves completion.

Managers may adjust approved subgoals, Tasks, order, frequency, and member
responsibility inside the currently approved boundary. Changes to the primary
goal, team, budget, Provider, tools, permissions, or external-action rules are
revision candidates that require Owner confirmation.

### 3. Personal Assistant, Pi, and Installed Agents

The Personal Assistant is the global natural-language explanation, navigation,
research, and proposal surface. Pi may support it internally as a hidden
library/engine, but Pi is not shown as an ordinary Installed Agent and owns no
authority, secrets, long-term memory, or completion decision. It produces
candidates only.

**Installed Agent remains a valid advanced product concept.** DSH is the
preinstalled, Personal-managed Agent supplied with Personal 2.0 and is visible
under **Settings > Installed Agents** with exact version, source, health,
qualification, update, and rollback facts. It is the default runtime for
project digital employees.

Product-managed DSH means an exact audited official artifact executed as a
Personal-managed isolated child process behind a bounded stdio broker. It is
not linked in-process with the Rust daemon and is not a vendored fork. Personal
does not embed or synchronize DSH's native UI or native conversations.
Personal owns Conversation, Memory, Task, archive, and employee identity; DSH
receives bounded Context and returns candidates/observations.

Personal 2.0 qualifies only DSH for this runtime role. Hermes, Codex, Cursor,
and other adapters are future independently qualified candidates, not promised
support. ADR-0043's generic adapter separation and ADR-0044's daemon arbitration
remain architecture constraints, not current multi-engine support claims.

DSH and Pi Provider traffic goes through the daemon proxy. Raw Provider
credentials never reach either process. Environment/plaintext credentials,
native MCP/base tools, HMR, and home-directory patching are denied by default.
Every executable capability still passes Personal admission.

### 4. Project activation and governed change

Project initialization is a resumable guided conversation:

`research -> charter -> goals/metrics -> team -> plan -> permissions/budgets -> triggers -> structured diff preview -> confirm -> receipt`

Research sources retain provenance and are treated as untrusted observations.
A project does not become active until the Owner confirms the exact charter
revision. Personal Assistant or manager changes use the same
candidate -> daemon preview -> Owner confirmation -> receipt boundary whenever
they cross the approved autonomy envelope. A stale revision forces
re-preview; rejection preserves the draft.

### 5. Routines, missed work, and completion

Routines may be triggered manually, by schedule, or by an independently
qualified platform event. The same routine does not overlap; at most the latest
pending occurrence is queued, and dropped/coalesced occurrences remain visible.
Sleep, shutdown, network loss, and Provider failure produce explicit offline
or missed-run facts. On resume, low-risk internal work may continue under
policy, while publishing, communication, spending, permission expansion, and
other consequential work require renewed review.

Closing the Control Plane asks whether eligible work should continue in the
background or pause. The product never promises work while the host is off.

Agent self-report, manager agreement, Provider success, Tool success, process
exit, or an engine checkpoint is not completion. Completion requires current
criteria evidence, reconciled Effects, and independent verification under
daemon acceptance. Key-result and daily/weekly reflections are candidates that
may propose a new revision; they never mutate the approved plan directly.

### 6. Knowledge, conversations, and memory

The Owner selects a Personal Home with separate `app/` and `data/` roots.
Projects receive managed data directories, but a directory is not Project
authority and its disappearance cannot silently delete a Project.

Knowledge has three product scopes: Owner-shared knowledge, a per-project
Markdown Vault, and employee-private memory. Vault content remains
Obsidian-compatible; Obsidian may be an optional companion but its proprietary
application is neither embedded nor required. Ordinary knowledge edits trigger
reindexing. Configuration-like edits to goals, roles, permissions, budgets, or
workflows become candidates and require the applicable daemon admission path.

All conversations form a scoped, indexed, episodic archive that can participate
in active retrieval. They are never injected wholesale. Retrieval is bounded,
redacted, scope-authorized, provenance-bearing, and marked as untrusted
observation. Semantic Memory requires admission from verified facts or an
explicit Owner decision. The Owner can inspect, correct, and forget admitted
memory. DSH receives only bounded Context and may submit Memory candidates.

### 7. Accounts, Providers, budgets, and external work

Consumer subscription, Provider account/authentication, API billing/quota,
model availability, binding, budget, and measured usage are separate facts.
Effective binding resolves:

`global -> project -> employee -> task`

Projects, members, and Tasks have budget boundaries; Provider quota and actual
usage/cost remain source-labelled and `unknown` is never rendered as zero.
Secrets stay in approved Secret Stores under ADR-0055 and are consumed only
through daemon-mediated proxies.

The first important acceptance scenario is an X/Twitter content operation, but
the Project model does not hard-code one success path. Browser/API connectors
are qualified one platform at a time, remain stoppable and auditable, and fail
closed on platform drift. Fingerprint evasion, CAPTCHA bypass, and anti-abuse
avoidance are forbidden. Only Owner-owned, licensed, open-license, or public
domain material may be copied; other sources are for analysis, attribution,
and new creation.

### 8. Local data, recovery, and 2.1 remote headroom

Product and business data remain local. Diagnostics leave the machine only
after explicit opt-in. Automatic same-disk versions are called **local restore
points**; they do not protect against disk failure and are not disaster
backups. Manual export remains available, excludes secrets by default, and
archive-first precedes permanent deletion.

Personal 2.1 remote operation remains host-online only. The Owner decided
against per-action biometric reauthentication after pairing and accepts that
future risk. Device-bound keys, revocation, short sessions, preview, receipts,
audit, and no secret downlink remain mandatory compensating controls.

## Supersession

### ADR-0056

This ADR **preserves** ADR-0056's desktop-primary entry, candidate-only
assistant, daemon preview/authority boundary, ADR-0055 credential-import rules,
capability honesty, same-origin client constraints, and non-claims.

It **supersedes** ADR-0056 only for:

1. the target IA (`Home/Agents/Work/Library/Activity/Settings` becomes
   `Today/Projects/Team/Knowledge/Inbox`, with Settings at the bottom);
2. external/native Agent conversation aggregation and native-app coexistence
   as the Personal 2.0 default (Personal now owns employee conversations);
3. the old P10-T04 delivery anchor and any cross-platform reading of 2.0
   (Phase 11 and Windows-first scope now apply).

### ADR-0058

This ADR **preserves** ADR-0058's MCP private envelope, no-Core-change
decision, older-client fail-closed behavior, P5 no-auto-migration rule,
capability digest, and secret/authority constraints.

It **supersedes** only the choice of dsh Path B vendor transcript as the first
canonical Personal 2.0 conversation slice. The existing
`cognitiveos.personal.conversation-projection/0.1` identifier is not silently
reinterpreted. A Personal-owned archive/projection shape must receive a new
private version or a future Lane-CTR decision before implementation.

## Consequences

- Canonical product, interaction, architecture, formal-plan, trace, support,
  environment, and handbook documents move to the OPC model in one delivery.
- P10-T01/T02 remain completed historical facts. P10-T03..T18 receive explicit
  dispositions; successor work is registered under stable Phase 11 IDs.
- Project, Role, Employee, Routine, Trigger, Attempt, Conversation, Vault,
  account, and budget concepts do not become a generic Core `Resource` family.
- The delivered Linux UI, dsh Path B, Pi qualification, existing private
  envelopes, and ordinary CI evidence do not establish the Windows OPC target.
- Paperclip, CrewAI, OpenAI Agents SDK, Temporal, LangGraph, Letta, Mem0,
  OpenHands, LobeHub, assistant-ui, n8n, Codex, Obsidian, DSH, and Pi research
  remains subject to the exact informative reference matrix. No external code
  is adopted by this ADR.

## Alternatives considered

1. **Keep the Agent-supervision hub.** Rejected because the Owner's primary
   job is governing business Projects and digital employees, not managing
   unrelated native Agent sessions.
2. **Hide DSH entirely as an internal library.** Rejected by the final Owner
   amendment. DSH is a preinstalled managed Installed Agent with advanced
   supply-chain visibility, while its native UI and conversation ownership are
   excluded.
3. **Embed or fork DSH in-process.** Rejected because it expands the daemon TCB
   and couples release, secret, and failure boundaries.
4. **Let a workflow engine own execution authority.** Rejected. LangGraph may
   receive a bounded Attempt adapter spike; Temporal remains behavior reference
   only. Neither becomes a second scheduler or authority writer.
5. **Treat the entire conversation archive as prompt memory.** Rejected for
   privacy, prompt-injection, cost, and provenance reasons.
6. **Ship native mobile remote control in 2.0.** Rejected to keep the formal
   boundary to a qualified Windows-local loop.

## Non-goals and non-claims

This decision implements no Windows host, Project aggregate, digital employee,
Conversation archive, Vault, retrieval, Personal Assistant, DSH package,
sandbox, scheduler, trigger, connector, UI route, Provider proxy, budget
enforcement, contract, schema, transition, error, vector, or test. It does not
qualify DSH, Windows, X/Twitter, Obsidian, Pi, or any future adapter. It creates
no support, Gate, release, Profile, B01, market-validation, usability,
performance, 24/7-operation, business-outcome, or multi-Agent-benefit claim.
