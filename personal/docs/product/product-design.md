# CognitiveOS Personal 2.0 OPC product design

- Status: canonical owner-approved product intent
- Change class: `product-semantic`
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Product-direction amendment: owner-confirmed `/grill-me` design tree,
  2026-08-28, then journey-subtraction workshop 2026-08-28/29
  ([workshop record](personal-2.0-opc-journey-subtraction-workshop-2026-08-28.md)).
  This document owns the amended product intent; architecture and
  implementation-plan reconciliation are explicitly deferred.
- Requirements baseline:
  [Personal 2.0 OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Exact scope: [Personal 2.0 scope](personal-2.0-scope.md)
- Ordered behavior: [User journeys](user-journeys.md)
- Current interaction prototype:
  [**personal-20-opc-e2e (post journey-subtraction)**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e.canvas.tsx)
- Archived historical V2 (not current chrome):
  [pre-subtraction history](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Prototype identity: current product chrome after the 2026-08-28/29
  journey-subtraction workshop. Visual tokens/components are reused from
  archived V2 so it still looks like the same product; IA and flow follow
  five-stage create, process-axis Projects, and Today without a KPI/swimlane
  wall. V2 CEO-rail / X-hero canvases live in the history folder and are not
  current chrome. Canvas-only HITL and daemon authority path remain.
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
who understands the business outcome but may not understand Agent technology
(懂业务的小白). Default UI does not show daemon / DSH / Pi / Harness / Loop;
runtime method is 执行方式, 周期, and 触发. Example work includes content
operations, ecommerce operations, AIGC production, and software development.
P0 publishes complete capabilities only; there is no default or demo Project,
and X/Twitter is not the first end-to-end acceptance journey.

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
   to create, operate, redirect, and understand everything. The Assistant has
   the highest UX privilege (it may see and initiate every flow) but writes
   only through preview → Owner confirm → receipt.
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

Project setup is a resumable **five-stage wizard**. Daily Today is not
available until all five complete and ⑤ 验收 succeeds:

```text
① project init
     empty Home -> Create Project -> Projects create page
     business description (chat if a model is bound; else guide to Settings)
     -> analysis / source-backed research
     -> item-by-item confirm list (process, per-stage outputs, 执行方式,
        auto-vs-approve, triggers, cost, rights, 总预览)
     -> inactive until 总预览; leave saves draft
② member init (roster, model required, 「确认这个班子」)
③ process init (one axis, one stage at a time)
④ per-stage test until expected sub-output is openable and verified
⑤ joint debug until expected overall outcome -> 「验收，进入 Today」
```

Empty Home shows only Create Project in the center; right chat is hidden.
Until ⑤, Today shows only continue-create; Knowledge unlocks at ③ when input
is needed (current draft only); Settings may connect models; Projects exposes
only this incomplete create. Research does not ask permission for each
ordinary web read. External text is untrusted. Secrets never appear in chat.
No silent model bind. A copy of a live Project lands as an inactive 副本
(not from ①); ④⑤ may be spot-checked or skipped after 总预览.

### P0-2 — Operate through group conversation and a process-axis canvas

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
or 验收 control and no permanent “Don’t ask again” grant. Timeboxed
「本周同一类对外不再问」 expires and is revocable in Settings. Missing
capabilities are labelled `Requires-backend` or `Requires-environment`; there
are no Connect / Install / Confirm fake buttons.

The manager speaks by default. A Member speaks proactively only when
mentioned, submitting a deliverable, handing off, blocked, or requesting a
decision. Ordinary process traces remain collapsed.

After ⑤, Today is ① one decision packet ② live-project run overview plus
counts and a today/week/month toggle ③ assistant. The primary CTA is only on
the decision packet. Chat may query run data and must not approve. Four
swimlanes are not default blocks.

A live Project is ① the business-process axis ② this stage (status, member,
auth/verify marks) ③ project group. There is no visible CEO six-step top
rail. CEO discipline remains backend: canvas HITL and independent verify.
Unknown cannot pass or accept. Close-out is openable artifact + verify state
+ 「验收，回 Today」.

### P0-3 — Create reusable Roles and Project-specific Member Runtimes

Only the base Project Manager Role is built in. The Personal Assistant performs
sufficient web research before proposing every other reusable Role Runtime
Template. Each version defines business purpose, responsibilities,
prohibitions, input/output/success and handoff contracts, instructions, Skills,
Tools, MCP needs, work cycle/reflection, model capabilities, Context/Memory,
permissions, and escalation.

A Project Member is the Project-specific long-lived Runtime definition created
from one pinned Template revision. Its Provider/model, grants, Memory, and
permissions remain Project-isolated and are explicit. Members are not shared
across Projects; only Role Runtime Templates may be reused. Executing a Task
starts a disposable Agent process/Attempt; process exit does not delete the
Member, Conversation, Memory, artifacts, or evidence.

### P0-4 — Remember and compress without losing authority

Personal owns the local Project group archive, Member work conversations,
admitted Memory, and a knowledge store that uses Obsidian as 底座; it does
not require installing the proprietary Obsidian app. Full source records
remain inspectable. Each Agent process receives a model-window-aware Context
package ordered: current Task contract -> fixed decisions -> relevant
source/artifact excerpts -> provenance-linked summaries -> older narrative.
Over-limit reduction removes older narrative first, never the Task contract
or fixed decisions.

Chat auto-admits to inspectable, correctable, and forgettable Memory
(overrides 2026-08-28 “ordinary chat is not Memory”). The assistant Memory
architecture is GitHub OpenAI Codex, in 2.0 scope as architecture — not as a
user-facing execution-engine store. Runtime output cannot own authority.
Cross-Project promotion requires confirmation. Accept/reject/edit/rate
actions become Project feedback evidence; one event cannot silently change a
global Role.

### P0-5 — Run, recover, and improve long-lived work

DSH is the hidden default Member execution engine and Pi the hidden Personal
Assistant engine. They are not shown on the 小白 default UI; engine details
appear only in recovery or advanced diagnostics. Routines support manual, schedule, accepted-artifact,
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

### P0-6 — P0 is complete capabilities, not a demo or X hero

P0 publishes complete product capabilities only. There is no default Project
and no demo Project. X/Twitter social-account operations are parked as a
later industry connector; they are not the 2.0 first-success path. First
success is ⑤ joint-debug 验收 of the Owner’s own Project. No fingerprint
evasion, CAPTCHA bypass, anti-abuse avoidance, blind retry, or unlicensed
copying is permitted.

### P0-7 — Connect models and acquire capabilities safely

Settings provides **Model Connections** only: a dropdown of mainstream
Provider templates plus advanced custom URL, compatibility mode, key, and
model. The Owner enters keys; A5 is satisfied by one-way SecretStore
handoff — the UI shows connected/failed and never echoes the raw secret in
DOM, chat, or git. Settings also revokes timeboxed 「本周不再问」 and hosts
notify/recovery. Creating every Member requires the Owner to select a
Provider/model explicitly; the Assistant may recommend but cannot bind
silently.

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

- **Today:** after ⑤, one decision packet plus live-project run overview
  (created / live / blocked counts, today/week/month toggle). Primary CTA
  only on the decision packet. Chat may query run data and cannot approve.
  Four swimlanes are not default blocks. Cost unknown is not zero. Incomplete
  create shows only continue-create. Empty Home (no Project) is Create
  Project only; chat hidden.
- **Projects:** long-lived governed workspaces. Empty create jumps here.
  Live Project: process axis + current stage + project group. Copy-project
  from a launchable Project produces an inactive 副本. No visible CEO
  six-step rail. No demo Project.
- **Knowledge:** current project files, why this fragment, import. Parse
  failure keeps the original. Chat auto-admits to inspectable Memory.
  Obsidian is 底座; the Obsidian app is not required. Knowledge stays locked
  until ③ needs input.
- **Conversation:** the Personal Assistant is global and has the highest UX
  privilege (see and initiate every flow) while writing only through preview →
  Owner confirm → receipt; inside a Project the group includes Owner, manager,
  and Members. `@` routes only into the unsent draft and does not bypass
  Task/revision authority. Chat announces HITL and links to the center
  preview; it cannot Approve or 验收.
- **Canvas:** current-stage workface and HITL preview live here. Ad-hoc
  reports are temporary until the Owner pins or saves them. Planned is not
  published. There is no Confirm in chat and no fake publish on 验收.
- **Settings:** Model Connections (dropdown + custom; Owner enters keys),
  revoke timeboxed skips, notify/recovery. No Installed Agent store,
  subscription/billing, or Inbox. Secrets never appear in chat.
- **People object chain:** Role Template → Member → Task → disposable process.
  A dead process does not delete the Member. Daily add-member: roster + chat
  + 「确认加入」; model required; 执行方式 disclosed after confirm; no silent
  grant.
- **Operations:** Working is in-progress observation, not completion.
  Backend default remains Candidate → Intent persisted → Fence → Execute →
  Independent verify → Receipt. Default UI does not say daemon / DSH / Pi /
  Harness / Loop.

Default terminology is Project, Role, Member, Goal, Deliverable, Work Plan,
Routine, and Execution Record. A Role is a reusable Runtime Template; a Member
is its Project-specific long-lived Runtime definition; an Agent process is one
disposable execution Attempt. 执行方式, 周期, and 触发 are the default
runtime labels. Prompt, Skill, Tool, MCP, and other runtime words are
progressively disclosed; Harness and Loop are not default chrome.

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
- X/Twitter as P0 hero, a default/demo Project, or a visible CEO six-step
  rail;
- a general MCP marketplace/family console: 2.0 includes only Assistant-led,
  security-reviewed, exact Project/Member capability acquisition and grants;
- disaster backup: same-disk versions are local restore points only.

## 8. Non-claims

This document changes product semantics only. Architecture, ADR, formal-plan,
and handbook reconciliation remain pending by scope. It does not implement
or qualify Windows, DSH, Pi, Project, Conversation, Vault, Trigger, UI,
Provider routing, capability acquisition, or parked X/Twitter connectivity.
It creates no Gate, release, Profile, market, usability, performance,
containment, or Agent-benefit evidence.
