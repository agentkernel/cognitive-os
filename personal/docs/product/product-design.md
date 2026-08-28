# CognitiveOS Personal 2.0 OPC product design

- Status: canonical owner-approved product intent
- Change class: `product-semantic`
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Product-direction amendment: owner-confirmed `/grill-me` design tree,
  2026-08-28. This document owns the amended product intent; architecture and
  implementation-plan reconciliation are explicitly deferred.
- Requirements baseline:
  [Personal 2.0 OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Exact scope: [Personal 2.0 scope](personal-2.0-scope.md)
- Ordered behavior: [User journeys](user-journeys.md)
- Interaction baseline:
  [**Owner-approved interaction baseline (2026-08-28)**](../../../clients/docs/design/opc-2.0/personal-20-ai-ceo-e2e-optimized-v2.canvas.tsx)
- Baseline identity: same V2 files (not a v3). Owner accepted the 2026-08-28
  competitive-informed overwrite: visible CEO loop (Ingest → Decide →
  Authorize → Execute → Verify → Report), Today decision packet plus four
  exception swimlanes, canvas-only HITL, and daemon authority path. This is
  not the pre-overwrite overlay-conversation / stacked-column V2.
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence
- Current-status owner: [PROGRESS.md](../../../docs/plan/PROGRESS.md)

## 1. Problem and evidence

The Owner wants to operate a one-person company through long-lived Projects and
digital staff without translating business intent into Agent, Prompt, Tool,
MCP, Loop, or Harness configuration. Existing Agent-first control surfaces put
packages, sessions, resources, and runtime mechanics ahead of the work the
Owner actually buys the product to accomplish: define an outcome, assemble a
capable team, receive usable deliverables, intervene when needed, and improve
the operating loop.

Personal 2.0 therefore becomes an **AI-native digital-staff console for an
OPC**, not an Agent manager with business concepts added on top. The primary
experience is a Project group conversation with a flexible evidence-backed
canvas. Settings and runtime facts exist to make Projects work; they are not
the product's organizing model.

This is an **owner-directed requirements baseline**, not a validated market
finding. Evidence consists of the Owner's stated needs, the existing product
audit, and informative OSS/product research. There are no five-or-more ICP
interviews, behavior/frequency data, observed workarounds, retention data, or
willingness-to-pay evidence. Demand, adoption, usability, and monetization
remain hypotheses.

## 2. Target user and JTBD

The 2.0 user is one local human Owner: an OPC operator or individual developer
who understands the business outcome but may not understand Agent technology.
Example
work includes content operations, ecommerce operations, AIGC production, and
software development. The first end-to-end acceptance journey is an X/Twitter
content-operation Project; other industries use the same Project/Role/Member
model without implying qualified connectors.

> When I run a long-lived business Project with digital staff, I want to
> describe the outcome in business language, let a manager organize the work,
> talk to the team in one Project group conversation, and receive verified
> deliverables on a flexible canvas, so I can operate the company without
> becoming an Agent-infrastructure administrator.

Personal 2.0 is a Windows-local product while the host is online. Native mobile
and relay-based remote control begin in 2.1.

## 3. Goals

1. Apply this fixed priority: Project main/sub/period goals and their openable,
   acceptable deliverables first; state needed to keep the work running
   second; configuration third.
2. Make the Personal Assistant and Project group conversation the default way
   to create, operate, redirect, and understand everything.
3. Let each Project Manager plan, delegate, verify, reflect, and improve the
   loop inside an Owner-approved autonomy envelope.
4. Render a flexible canvas from real Project objects and deliverables:
   standard reporting uses a stable template; ad-hoc requests compose approved
   components without generating arbitrary code or invented values.
5. Hide DSH and Pi as managed engines during normal work while preserving
   exact health, update, rollback, authority, secret, and evidence boundaries.
6. Preserve A1–A8, Intent/Effect ordering, fencing, and independent completion
   verification across every conversational and visual surface.

## 4. P0 requirements

### P0-1 — Research, design, and activate a viable Project

Project setup follows one resumable sequence:

```text
business understanding
  -> broad automatic web research
  -> charter
  -> goal hierarchy and output contracts
  -> team
  -> plan and work cycle
  -> Provider and capabilities
  -> permissions and HITL
  -> triggers
  -> simulate one cycle
  -> structured launch preview
  -> confirm
  -> receipt
```

Research optimizes for a high-quality design and does not ask permission for
each ordinary web read. It may use non-secret Project context, but never raw
credentials, SecretStore material, or third-party data the Owner lacks the
right to disclose. External text is untrusted and cannot execute, install, or
expand permission. A Project is inactive until the Owner confirms the exact
revision.

### P0-2 — Operate through group conversation and a flexible canvas

The stable anchors are **Today / Projects / Knowledge**, with Settings
secondary. Team and Inbox are not first-level destinations. The shell locks
left navigation, center canvas, and right conversation; a narrow canvas
scrolls horizontally and does not stack those columns. Conversation stays the
third column; there is no overlay “open conversation” control.

Outside a Project the conversational identity is the global Personal
Assistant; inside one it is the Owner/manager/Members Project group in the
right column. The Owner can `@manager` for a briefing or task assignment and
`@member` to ask or temporarily redirect bounded work. Every work-changing
message becomes a formal Task or revision before it has authority. HITL is
announced in chat and linked to the center-canvas preview; chat has no Approve
control and no “Don’t ask again” grant. Missing capabilities are labelled
`Requires-backend` or `Requires-environment`; there are no Connect / Install /
Confirm fake buttons.

The manager speaks by default. A Member speaks proactively only when
mentioned, submitting a deliverable, handing off, blocked, or requesting a
decision. Ordinary process traces remain collapsed.

A Project opens to a stable operating-report template first, then the X loop
when that Project needs it. The manager may version the template for that
Project. For an ad-hoc question, the Assistant/manager reads real results and
composes approved typed components. The canvas is temporary unless pinned or
saved as a template. It cannot execute generated code or `eval`, invent
values, or hide goal/acceptance state, failures/not-run work, Owner decisions,
source, or freshness. A publication package such as Package A is a thread
preview plus acceptance criteria; planned is not published.

### P0-3 — Create reusable Roles and Project-specific Member Runtimes

Only the base Project Manager Role is built in. The Personal Assistant performs
sufficient web research before proposing every other reusable Role Runtime
Template. Each version defines business purpose, responsibilities,
prohibitions, input/output/success and handoff contracts, instructions, Skills,
Tools, MCP needs, work cycle/reflection, model capabilities, Context/Memory,
permissions, and escalation.

A Project Member is the Project-specific long-lived Runtime definition created
from one pinned Template revision. Its Provider/model, grants, Memory, and
permissions remain Project-isolated and are explicit. Executing a Task starts
a disposable Agent process/Attempt; process exit does not delete the Member,
Conversation, Memory, artifacts, or evidence.

### P0-4 — Remember and compress without losing authority

Personal owns the local Project group archive, Member work conversations,
admitted Memory, and Obsidian-compatible Markdown Vault; it does not embed or
require the proprietary Obsidian app. Full source records remain inspectable.
Each Agent process receives a model-window-aware Context package ordered:
current Task contract -> fixed decisions -> relevant source/artifact excerpts
-> provenance-linked summaries -> older narrative. Over-limit reduction removes
older narrative first, never the Task contract or fixed decisions.

Ordinary chat does not automatically become Memory. Explicit instructions
become revisions; “remember” and stable verified facts produce candidates.
Runtime output can propose Memory but cannot own or admit it. The Owner can
inspect, correct, promote, and forget; cross-Project promotion requires
confirmation. Accept/reject/edit/rate actions become Project feedback evidence,
while one event cannot silently change a global Role.

### P0-5 — Run, recover, and improve long-lived work

DSH is the hidden default Member execution engine and Pi the hidden Personal
Assistant engine. Engine details appear only in recovery or advanced
diagnostics. Routines support manual, schedule, accepted-artifact,
Project-state, qualified external-event, and testable-data-condition triggers;
no overlap, queue-latest, offline/missed/coalesced visibility, expiry-aware
catch-up, and risk-based resume. Windows host shutdown means no work.

The manager loop is observe -> plan -> delegate -> execute -> independently
verify -> summarize -> reflect -> adjust. Reflection occurs per Task, day,
cycle/week, and incident. One-off Task strategy may change inside the admitted
boundary; a persistent Member Runtime change creates a new version with
replay/simulation/comparison and rollback. A new Owner instruction applies at a
safe point through continue, pause, or restart, never as a silent prompt
injection.

A Member Task process may create count/time/cost/permission-bounded internal
subagents. They are not Project Members, have no long-lived identity or Memory,
and return results to the current Member. Project Members collaborate through
Tasks, artifacts, and handoffs.

### P0-6 — Complete one controlled X/Twitter operating loop

An X/Twitter Project progresses from source-backed research through planning,
draft/media deliverables, publication-package review, qualified connector
dispatch, receipt, interaction readback, manager reflection, and the next-cycle
plan. Suggested comment replies require applicable review. Manual publication
is a degraded fallback, not the primary 2.0 acceptance route. No fingerprint
evasion, CAPTCHA bypass, anti-abuse avoidance, blind retry, or unlicensed
copying is permitted.

### P0-7 — Connect models and acquire capabilities safely

Settings provides **Model Connections** only: mainstream Provider quick
templates plus advanced custom URL, compatibility mode, key, and model.
Creating every Member requires the Owner to select a Provider/model explicitly;
the Assistant may recommend but cannot bind silently. Raw secrets go only to an
approved SecretStore through a non-logging path.

Cost is source-labelled actual, estimated, or unknown and produces warnings;
Personal 2.0 does not automatically stop work at a product budget threshold.
Provider quota or unavailability may still fail externally.

The Assistant may discover Skills and MCP capabilities online. Skills require
source, license, hidden-instruction, prompt-injection, and file/network/command
intent review. MCP adds dependency, executable-code, network, Secret,
tool-permission, and supply-chain review, with Owner confirmation of exact
version and permissions before first install or expansion. Acquired artifacts
may be reused globally, but each Project/Member grant is separate, pinned,
reviewed on update, compatibility-tested, and rollback-capable.

## 5. Product organization

The primary IA is **Today / Projects / Knowledge**. Settings is fixed at the
bottom. Team, Inbox, attention/approval queues, Role cards, Runtime details,
and execution traces open as contextual canvas regions rather than permanent
top-level destinations. Native mobile, pairing, and cloud 24/7 chrome are 2.1
and are not drawn as current product chrome.

- **Today:** one decision packet (consequence, reversibility, alternatives,
  kernel truth, why option A is first) plus four exception swimlanes—Needs you
  / Can continue / Unknown / Missed—not a KPI card wall. Cost is estimated or
  actual; actual unknown is not zero. Member activity is a Working / Queued /
  Waiting table; queued is not running.
- **Projects:** long-lived governed workspaces. A Project opens to its stable
  operating-report template first, then the X loop. The right column is the
  Project group conversation.
- **Knowledge:** Owner-shared knowledge, Project Markdown Vaults, sources,
  indexing, and admitted Memory. Context shows why each fragment was selected.
  Memory is not silent auto-ingest. Obsidian is an optional companion and is
  not an embedded app.
- **Conversation:** the Personal Assistant is global; inside a Project the
  group includes Owner, manager, and Members. `@` routes only into the unsent
  draft and does not bypass Task/revision authority. Chat announces HITL and
  links to the center preview; it cannot Approve.
- **Canvas:** standard reports update one Project template; ad-hoc reports are
  temporary until the Owner pins or saves them. Package inspect shows a thread
  preview and acceptance; planned is not published. Publish preview is the
  full AUTONOMY packet; there is no Confirm in chat.
- **Settings:** Personal Home, model connections, notifications, recovery, and
  advanced diagnostics. There is no Installed Agent store, subscription/
  billing product, or broad marketplace/family console. Secrets use
  SecretStore takeover and never appear in chat.
- **People object chain:** Role Template → Member → Task → disposable process.
  A dead process does not delete the Member.
- **Operations:** Working is in-progress observation, not completion. The
  default working view is Candidate → Intent persisted → Fence → Execute →
  Independent verify → Receipt.

Default terminology is Project, Role, Member, Goal, Deliverable, Work Plan,
Routine, and Execution Record. A Role is a reusable Runtime Template; a Member
is its Project-specific long-lived Runtime definition; an Agent process is one
disposable execution Attempt. Prompt, Skill, Tool, MCP, Loop, Harness, model
window, epoch, and digest are progressively disclosed.

## 6. Success measures and counter-measures

These are target measures, not current results or market claims:

- time from business description to the first safely activated Project;
- activation without requiring the Owner to understand Agent infrastructure;
- expected deliverables arriving on time, opening correctly, and carrying a
  clear acceptance/evidence state;
- Owner interventions being few, consequential, and clearly explained;
- missed, failed, stale, and unknown work being surfaced rather than coerced
  into success;
- time from an operational exception to a safe recovery decision;
- whether manager revisions reduce the next comparable goal/output gap;
- time from an ad-hoc question to a useful source-linked canvas;
- the number of operating cycles a Project continues to deliver value.

Counter-measures are false completion, unintended publication, permission
expansion, memory poisoning, context-scope leakage, invented canvas values,
unknown cost rendered as zero, and irreversible action without the required
preview. Human usability, retention, willingness to pay, and business benefit
still require separate research before numerical claims.

## 7. Out of scope

- native mobile, pairing, E2E relay, or remote control in 2.0;
- human-team accounts, Company/Business Space, RBAC, multi-tenancy, cloud
  authority, or offline-host execution;
- Agent/Harness marketplace, alternative supported engines, native DSH/Pi UI,
  native conversation synchronization, an in-process DSH daemon, or a vendored
  fork;
- consumer-subscription, invoice, or plan management;
- an arbitrary-code generated canvas or general no-code workflow builder;
- all-industry connectors, unsupervised high-risk action, guaranteed business
  outcomes, all-platform publishing, browser/API equivalence, or multi-Agent
  benefit;
- a general MCP marketplace/family console: 2.0 includes only Assistant-led,
  security-reviewed, exact Project/Member capability acquisition and grants;
- disaster backup: same-disk versions are local restore points only.

## 8. Non-claims

This document changes product semantics only. Architecture, ADR, formal-plan,
and handbook reconciliation remain pending by scope. It does not implement
or qualify Windows, DSH, Pi, Project, Conversation, Vault, Trigger, UI,
Provider routing, capability acquisition, or X/Twitter connectivity. It creates no
Gate, release, Profile, market, usability, performance, containment, or
Agent-benefit evidence.
