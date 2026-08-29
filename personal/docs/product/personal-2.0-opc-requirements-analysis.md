# Personal 2.0 OPC requirements analysis

- Status: owner-confirmed requirements baseline
- Date: 2026-08-28; scheme snapshot 2026-08-28/29
- Change class: `product-semantic`
- Source: synthesis of the completed owner `/grill-me` design interview plus
  the 2026-08-28/29 journey-subtraction workshop. This file is requirements,
  not a transcript. Verbatim Q&A lives in
  [journey-subtraction workshop record](personal-2.0-opc-journey-subtraction-workshop-2026-08-28.md).
- Product intent: [Product design](product-design.md)
- Version boundary: [Personal 2.0 scope](personal-2.0-scope.md)
- Ordered behavior: [User journeys](user-journeys.md)
- Product model: [OPC product model](opc-product-model.md)
- Interaction corpus:
  [OPC 2.0 design](../../../clients/docs/design/opc-2.0/README.md)
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v5**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v5.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](../../../clients/docs/design/opc-2.0/history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Prototype identity: owner-approved 2026-08-29 current chrome is
  `personal-20-opc-e2e-optimized-v5`. The 2026-08-28/29 workshop remains the
  scheme snapshot; v5 amends create order to process-before-members. V2
  CEO-rail / X-hero files are archived and are not current chrome.
  Canvas-only HITL and daemon authority path remain.
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## A. Document status, source, and authority boundary

This document records the complete owner-confirmed need for Personal 2.0. It is
the baseline from which the current product intent, scope, object model, user
journeys, and prototype-design corpus are derived. The `/grill-me` interview
and the 2026-08-28/29 journey-subtraction workshop have been synthesized into
problems, jobs, principles, requirements, boundaries, and traceability.
Conversational wording is preserved only in the
[workshop record](personal-2.0-opc-journey-subtraction-workshop-2026-08-28.md);
this file is not that transcript.

The requirements are approved **product semantics**. They are not:

- market validation or evidence that an ICP will adopt or pay;
- architecture or implementation design;
- a task plan, acceptance ledger, current-status source, or authorization to
  begin implementation;
- proof that Windows, DSH, Pi, X/Twitter, a Provider, a Skill, an MCP server,
  multi-Agent work, or any described UI is implemented or qualified;
- a Gate, release, Profile, usability, accessibility, performance, security,
  containment, reliability, or business-benefit result.

Source ownership remains separated:

| Fact | Owner |
|---|---|
| Owner need and requirement baseline | this document |
| Verbatim workshop Q&A + scheme snapshot | [journey-subtraction workshop](personal-2.0-opc-journey-subtraction-workshop-2026-08-28.md) |
| Stable current product intent | [product-design.md](product-design.md) |
| Exact Personal 2.0 inclusion/exclusion | [personal-2.0-scope.md](personal-2.0-scope.md) |
| Product objects and terminology | [opc-product-model.md](opc-product-model.md) |
| Ordered experience | [user-journeys.md](user-journeys.md) and the [OPC design corpus](../../../clients/docs/design/opc-2.0/README.md) |
| Architecture composition | `personal/docs/architecture/**` |
| Accepted architecture decisions | `docs/adr/**` |
| Formal tasks and Gates | `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` |
| Current implementation and evidence facts | `docs/plan/PROGRESS.md` |

Architecture, ADR, formal-plan, and handbook material still carrying the
2026-08-27 OPC vocabulary is **pending architecture/plan/handbook
reconciliation**. It remains valid as dated decision or implementation fact,
but it does not override the product semantics in this baseline. This delivery
does not modify those surfaces.

All absent product behavior is labelled **Requires-backend**; behavior that
also needs a qualified native or external environment is labelled
**Requires-environment**. `not-run` is never pass.

## B. Executive summary, problem, JTBD, and personas

### Executive summary

Personal 2.0 is an **AI-native digital-staff console for one-person companies
and individual developers**, built on `cognitiveos-core`. It lets one human
Owner describe business goals, create long-lived governed Projects through a
five-stage create wizard, assemble Project Members from reusable Role Runtime
Templates, supervise work through a business-process axis and Project group
conversation, and receive openable, verifiable deliverables without first
learning Agent infrastructure. Default copy assumes a business-literate
non-technical Owner: daemon / DSH / Pi / Harness / Loop are not product chrome.

P0 publishes complete capabilities only. There is no default or demo Project,
and X/Twitter social-account operations are not P0 release content (parked as
a later industry connector). The product model is industry-neutral. Personal
remains local to the Windows host and works only while that host is online.

### Problem

Agent-first products make the Owner translate business intent into prompts,
sessions, agents, tools, MCP configuration, harnesses, and runtime mechanics.
That is the wrong operating model for the target user. The Owner needs to:

1. state and revise a business outcome;
2. know which result is due and what evidence will make it acceptable;
3. delegate planning and execution within explicit boundaries;
4. inspect usable deliverables rather than raw Agent chatter;
5. intervene only when a decision or exception matters;
6. understand failures, missed work, cost uncertainty, and recovery;
7. improve the operating system without turning one interaction into hidden
   global behavior.

The existing mental model also conflates stable people-like responsibility,
runtime configuration, and disposable processes. Process exit then appears to
delete an employee, chat appears to become authority, and engine success
appears to prove completion. Personal 2.0 separates those concepts.

### Primary JTBD

> When I run an OPC Project with digital staff, I want to describe goals and
> acceptable outputs in business language, let a manager organize and improve
> the work inside approved boundaries, talk to the team in one Project group,
> and receive verified deliverables on a clear canvas, so I can operate the
> business without becoming an Agent-infrastructure administrator.

### Supporting jobs

| Job type | Job |
|---|---|
| Functional | Turn a business situation into an active Project with goals, output contracts, team, plan, capabilities, permissions, triggers, and a receipt. |
| Functional | See what is due, what changed, what failed or was missed, and what needs the Owner now. |
| Functional | Ask the manager for progress or delegation and redirect a Member's bounded work without creating shadow authority. |
| Functional | Open, inspect, accept, reject, or revise deliverables with source, freshness, and verification state. |
| Functional | Connect a model and acquire reviewed Skills/MCP capabilities without exposing credentials or silently expanding grants. |
| Functional | Preserve conversations, knowledge, admitted Memory, evidence, and Member identity across disposable Agent processes. |
| Emotional | Feel in control without supervising every micro-step. |
| Emotional | Trust that silence, polished prose, or a green Provider call is not being mistaken for completion. |
| Emotional | Avoid anxiety caused by hidden publishing, stale context, unknown cost shown as zero, or unrecoverable automation. |
| Social | Operate professionally and consistently in front of customers and an audience even as a one-person company. |

### Personas

#### Primary — OPC Owner-operator

- understands the business, audience, and desired results;
- may not understand Agent, model, Tool, MCP, Context, or 执行方式 concepts;
- delegates recurring knowledge work but remains accountable for public,
  financial, security, and destructive actions;
- values fast comprehension, low intervention burden, reliable outputs, and
  clear recovery over technical configurability.

#### Secondary — individual developer

- can understand deeper runtime detail when needed;
- uses the same Project/Role/Member model for software or AIGC work;
- expects source-linked artifacts, reproducibility, permissions, and rollback;
- does not change the product into an IDE, generic workflow builder, or
  multi-user engineering-management system.

No multi-user administrator, human teammate, enterprise approver, or cloud
operator persona is in the Personal 2.0 boundary.

## C. Design principles

1. **Outcomes before operations.** Project goals, expected results, due
   deliverables, success criteria, and evidence precede runtime state; runtime
   state precedes configuration.
2. **Business language first.** Project, Role, Member, goal, deliverable, work
   plan, and 周期/触发 are defaults. Agent/runtime vocabulary (including
   daemon, DSH, Pi, Harness, Loop) is not default chrome; 执行方式 is the
   disclosed label when runtime method must be confirmed.
3. **Project is the operating unit.** A Project is a long-lived governed
   workspace, not a Loop, Harness, directory, session, or group chat.
4. **Stable orientation, contextual depth.** Only Today, Projects, and
   Knowledge are stable first-level anchors; Settings stays at the bottom.
   Members, approvals, exceptions, and diagnostics open in context.
5. **Conversation proposes; authority records.** Natural-language messages
   must resolve to formal Tasks or revisions before changing governed state.
6. **Deliverables over chatter.** The default view emphasizes openable
   artifacts, acceptance, omissions, failures, decisions, and freshness.
   Ordinary execution traces remain collapsed.
7. **Flexible but governed canvas.** Stable operating reports use a Project
   template; ad-hoc questions compose approved typed components from real
   facts. No generated code, `eval`, or invented values.
8. **Stable members, disposable processes.** Role Templates and Project Member
   Runtime definitions persist; each Task Attempt starts a disposable Agent
   process.
9. **Invisible infrastructure by default.** DSH and Pi are managed engines,
   not daily product destinations.
10. **Calibrated autonomy.** One launch-time autonomy envelope enables
    low-risk, reversible internal work; consequential boundary crossings
    receive preview, confirmation, and receipt.
11. **Evidence honesty.** Unknown stays unknown; self-report, process exit, or
    tool/provider success never becomes completion.
12. **Local, inspectable, recoverable.** Source records, conversations,
    versions, Memory lineage, attempts, Effects, and receipts survive failure
    and can be inspected, corrected, exported, archived, or restored within
    their real limits.
13. **Progressive capability acquisition.** Skills and MCP capabilities are
    discovered and reviewed when a Project needs them; installation never
    implies a Project/Member grant.
14. **Inspectable auto-Memory; no silent global Role change.** Chat
    auto-admits to inspectable/forgettable Memory (Codex memory architecture;
    Obsidian as knowledge 底座). Feedback still cannot silently change a
    Member or global Role; those remain versioned proposals with the
    applicable approval.

## D. Complete product object model and glossary

### Product object chain

```text
Owner
  -> global Personal Assistant
  -> Project
       -> Charter
       -> Goal hierarchy
            -> main
            -> phase / quarter (when useful)
            -> month
            -> week
            -> day / Tasks
       -> expected results / deliverables / success criteria / evidence
       -> current Plan revision / autonomy envelope / external-action policy
       -> Project group Conversation
       -> Project Manager Member
       -> other Project Members
            -> pinned Role Runtime Template revision
            -> Project-specific Member Runtime definition
            -> Provider / model
            -> Project- and Member-scoped capability grants
            -> Member work Conversation / admitted Memory
       -> Routine / Trigger / occurrence
            -> Task
                 -> Attempt
                      -> disposable Agent process
                      -> bounded internal subagents
                      -> Artifact / Intent / Effect / Evidence
                 -> independent verification / acceptance
       -> operating-report canvas template / temporary ad-hoc canvases
       -> Project Vault / sources / group archive / feedback evidence
```

### Object definitions

| Object | Meaning | Explicit non-equivalence |
|---|---|---|
| Owner | the sole local human principal | not a tenant, organization, or human team |
| Personal Assistant | global user-facing identity that can inspect, explain, research, recommend, and initiate every management flow | not a Project Member, authority writer, or durable-memory owner |
| Project | long-lived governed workspace for one business outcome and its operating loop | not a directory, chat, Loop, Harness, Runtime, or Task collection alone |
| Charter | confirmed description of Project purpose, scope, constraints, and primary outcome | not free-form assistant prose |
| Goal | versioned intended result with expected result, deliverables, cadence/due, owner, success criteria, and evidence | not an Agent promise or uncontrolled commercial guarantee |
| Plan revision | current governed decomposition and sequencing of goals, work, responsibilities, triggers, and acceptance | not chat history |
| Role Runtime Template | reusable, versioned operating recipe | not a process, Member, credential, or authority grant |
| Project Member Runtime definition | persistent Project-specific definition created from a pinned Role Template revision | not a human identity or always-running process |
| Project Manager Member | the single current Member responsible for planning, assignment, verification, escalation, summaries, and reflection | not the authority writer and not guarantor of uncontrollable business results |
| Task | bounded admitted work with contract, responsibility, resources, limits, outputs, and acceptance criteria | not a chat message or Agent turn |
| Attempt | one preserved execution try for a Task | not the Member |
| Agent process | disposable runtime process started for one Attempt | not a Role or Project Member |
| Internal subagent | bounded temporary helper created by one Member Task process | not a Project Member; no long-lived identity or Memory |
| Routine | versioned recurring work definition | not one run, cron row, or prompt |
| Trigger | admitted cause that requests a Routine occurrence | not permission to complete work |
| Conversation | Personal-owned full local source archive for Project group or Member work discussion | not authority or completion; chat auto-admits to inspectable/forgettable Memory |
| Canvas | source-linked projection of Project goals, deliverables, evidence, state, and decisions | not an authority store, arbitrary program, or data generator |
| Artifact | inspectable work product or intermediate output | not accepted completion by itself |
| Evidence | source/provenance-bearing observation used by an independent criterion | not Agent self-report |
| Memory | admitted durable fact/preference with scope, provenance, lineage, correction, and forget lifecycle | not a raw chat transcript, summary, or retrieval cache |
| Vault | Personal-owned knowledge store using Obsidian as 底座 (Markdown-compatible); 2.0 does not require installing the Obsidian app | not the proprietary Obsidian application and not daemon authority |
| Model Connection | Provider endpoint/account/key/model configuration whose raw secret remains in SecretStore | not a consumer subscription or product billing account |
| Skill | reviewed, pinned work-method artifact | not permission or executable authority |
| MCP capability | reviewed server/package/connection/capability artifact plus separate Project/Member grants | not a marketplace install or automatic Tool grant |
| Autonomy envelope | launch-time approved limits for low-risk internal planning and execution | not permission for every future external or high-risk action |
| Receipt | durable result of an admitted consequential operation | not a conversational acknowledgement |

### Role Runtime Template contents

Every Template version defines:

- business purpose, responsibilities, and prohibited work;
- input, output, handoff, and success contracts;
- work instructions;
- required Skills, Tools, and MCP capabilities;
- work cycle, reflection, collaboration, and handoff behavior;
- model capability requirements;
- Context assembly and Memory policy;
- permissions, escalation rules, and safety boundaries.

Only the base **Project Manager Role** is built in. The Personal Assistant
conducts sufficiently broad web research before proposing other Roles. Role
Templates are reusable; Member definitions, Memory, Provider/model selection,
permissions, and grants remain isolated per Project.

### Retired current-product terms

`Digital Employee` / `数字员工` remains only in the market-positioning phrase
“digital-staff console / 数字员工控制台”. It is no longer a separate user-visible
object between Role and Runtime. `Role Blueprint`, `Project Role Assignment`,
daily `Installed Agent`, and a user-selectable `Harness` are also retired from
the current 2.0 object model. See section L for migration.

## E. Functional requirements

### E1. Project and goal management (`PRJ`)

- **PRJ-01:** Create a Project through a resumable **five-stage wizard**. Daily
  Today is forbidden until all five complete and ⑤ 验收 succeeds:
  ① project init (business description → research → item-by-item confirm
  list → 总预览; inactive until 总预览);
  ② process init (one process axis, one stage at a time, total goal + cycle
  on the axis, 执行方式 disclosure);
  ③ member init (roster from the confirmed process, explicit model per person,
  then one-by-one init of 工作说明 / 工具 / 能力包 / 周期与触发 / 外部连接 /
  文档范围 with visible progress; 「当前初始化」 is progress + current item
  title only; full recipe on the shared member config page; sequential seating;
  all seated before ④);
  ④ per-stage test until the expected sub-output is openable and verified
  (block start/pass unless the responsible member is seated: those six slots +
  model);
  ⑤ joint debug until the expected overall outcome, then 「验收，进入 Today」.
  Aha is ⑤ accept. Copy-from-live lands as an inactive 副本 and may spot-check
  or skip ④⑤ after 总预览; it does not restart at ①.
  Owner-confirmed prototype amendment (2026-08-29): **process before members**.
  The workshop snapshot recorded members then process; that Q&A is unchanged.
  Architecture and formal-plan reconciliation remain deferred.
- **PRJ-01a:** Empty Home (no Project): center is only Create Project; right
  chat is hidden. Create Project jumps to the Projects create page. With a
  model, right chat opens as the normal assistant; without a model, chat only
  guides to Settings to bind the assistant. Leave saves draft and resumes.
  Connection failure names the problem. Daemon/runtime chrome is not shown.
- **PRJ-01b:** Until ⑤ completes, Today is not the daily decision packet (only
  「继续未完成的创建」). Knowledge opens at ② (process) when input is needed
  (current draft only). Settings may connect models. Projects exposes only this
  incomplete create. Knowledge is locked before that.
  Create / members / test / joint right chat defaults to Personal Assistant.
  Canvas edit → Enter → confirm dialog → Owner message in chat; assistant
  proposes; user confirms in chat; then canvas applies. Chat has no Approve.
  Prototype generation is labelled local target-state samples, not daemon
  writes.
- **PRJ-01c:** Default confirm list (chat, item-by-item): business process /
  per-stage outputs / cycle / save format + Skill / tools / MCP / knowledge /
  env / file permissions + auto-vs-approve (including external), triggers,
  cost, source rights, launch preview. Label 执行方式; do not say Harness.
  Secrets never in chat. No silent model bind.
- **PRJ-02:** Research defaults to broad, high-quality coverage without asking
  for permission for each ordinary web read. Sources, freshness, conflicts,
  gaps, and rights remain visible.
- **PRJ-03:** Research may use non-secret Project context, but never raw
  credentials, SecretStore material, or third-party data the Owner lacks the
  right to disclose. External text remains untrusted and cannot execute,
  install, or expand permission.
- **PRJ-04:** A Project remains inactive until the Owner confirms the exact
  structured launch revision (总预览). Copy-project drafts are also inactive
  until 总预览.
- **PRJ-05:** Each goal records expected result, deliverables, cadence or due
  date, responsible owner, success criteria, and evidence.
- **PRJ-06:** Goal depth is main -> phase/quarter when useful -> month -> week
  -> day/Tasks. The manager rolls lower-level plans forward as evidence and
  conditions change.
- **PRJ-07:** The manager is responsible for planning, assignment,
  verification, acceptance preparation, and escalation, but does not guarantee
  followers, GMV, revenue, or other uncontrollable outcomes.
- **PRJ-08:** After ⑤, Today is ① one decision packet ② live-project run
  overview plus counts (created / live / blocked) and today/week/month toggle
  ③ assistant. Click a live project for stage/member/count/fail/avg/success
  detail. Primary CTA is only on the decision packet. Chat may query run data
  and must not approve. Four exception swimlanes are not default blocks
  (semantics may merge into the overview). Unknown cost is never 0. Offline
  shows last-known aged overview.
- **PRJ-09:** A live Project opens to ① a business-process axis ② this stage
  (status, responsible member, auth/verify marks and content) ③ project group
  conversation. There is no visible CEO six-step top rail. Chat cannot
  approve. Operating statistics enter from Today or a report step, not as a
  second clock.
- **PRJ-10:** Multi-Project scheduling explains ordering by Owner priority,
  deadline, schedule, resource availability, and fairness.
- **PRJ-11:** Archive stops new triggers first while preserving read, export,
  and restore access.
- **PRJ-12:** Permanent deletion requires an impact preview and restore point;
  raw secrets never enter export.
- **PRJ-13:** When at least one launchable/live Project exists, Projects list
  and the project page offer copy-project. Copy goals, process axis,
  role/member definitions, 执行方式, triggers, and output forms. Do not copy
  secrets, in-flight tasks, external receipts, or timeboxed 「本周不再问」.
  The copy is an inactive draft named 副本; the Owner edits, then 总预览 to
  go live.

### E2. Role and Project Member management (`ROLE`)

- **ROLE-01:** Only the base Project Manager Role is built in.
- **ROLE-02:** The Assistant researches and proposes every other Role Runtime
  Template with the complete contract listed in section D.
- **ROLE-03:** A Template is reusable and versioned; a Project Member pins one
  revision and adds Project-specific responsibility, subgoal, instructions,
  Provider/model, grants, Memory, Context, and permission. Members are not
  shared across Projects; only Templates may be reused.
- **ROLE-04:** The Owner explicitly chooses a Provider/model for every Member
  during creation. The Assistant may recommend but must not bind silently.
- **ROLE-05:** Member cards lead with goal, responsibility, current work, next
  action, latest accepted deliverable, block/decision, cost basis, and
  freshness. Engine detail is secondary.
- **ROLE-06:** Process exit, retry, engine update, or quarantine preserves
  Member identity, conversations, Memory, artifacts, attempts, and evidence.
- **ROLE-07:** A persistent Member Runtime change creates a new version and
  receives replay/simulation/comparison plus rollback. The manager may activate
  it only inside the approved envelope.
- **ROLE-08:** Global Role changes, team changes, primary-goal changes,
  Provider/model changes, Tool/MCP changes, permission changes, and
  external-action-rule changes require Owner confirmation.

### E3. Project conversation and canvas (`CONV`, `CAN`)

- **CONV-01:** Outside a Project the conversation identity is the global
  Personal Assistant; inside a Project it is the Project group.
- **CONV-02:** The group includes Owner, Project Manager, and Members. The
  manager speaks by default. Members speak proactively only when `@` mentioned,
  submitting a deliverable, handing off, blocked, or requesting a decision.
- **CONV-03:** `@manager` can request status or delegation. `@member` can ask a
  question or temporarily adjust the execution goal/path inside its approved
  boundary. `@member` creates a formal Task revision, not a shadow plan.
- **CONV-04:** A directive that changes work becomes a formal Task or plan/
  Member revision. A message cannot be shadow authority.
- **CONV-05:** Ordinary execution traces are collapsed behind the relevant
  Task, Attempt, artifact, or evidence.
- **CONV-06:** Full raw Project-group and Member-work conversations remain
  locally inspectable even when Context packages contain only selected parts.
  A Member work conversation is visible to the Owner, the manager, and that
  Member.
- **CAN-01:** The default live-Project canvas is the business-process axis and
  the current stage workface, not a visible CEO six-step rail and not a
  default/demo X loop. The Project Manager may version that Project's report
  template for a report step.
- **CAN-02:** For an ad-hoc question, the Assistant/manager interprets intent,
  reads real Project results, and composes a temporary canvas from approved
  typed components.
- **CAN-03:** An ad-hoc canvas is not saved by default. The Owner may pin it or
  save it as a Project template.
- **CAN-04:** Canvas components cannot run generated code or `eval`, fetch
  unapproved data, or invent values.
- **CAN-05:** Goal state, acceptance state, failed/not-run work, Owner
  decisions, data source, and freshness cannot be hidden by a template.

### E4. Execution, long-running work, and multi-Agent (`EXEC`)

- **EXEC-01:** The manager loop is observe -> plan -> delegate -> execute ->
  independently verify -> summarize -> reflect -> adjust.
- **EXEC-02:** A Task starts a disposable Agent process/Attempt from an exact
  Member Runtime revision and bounded Context package.
- **EXEC-03:** DSH is the hidden default Member execution engine. Pi is the
  hidden Assistant engine. Neither has a daily native product UI.
- **EXEC-04:** Engine exact version, health, qualification, update, and
  rollback appear only for recovery or advanced diagnostics.
- **EXEC-05:** A Member Task process may create bounded internal subagents with
  explicit count, time, cost, and permission limits. Results return to the
  Member; subagents gain no Project identity or long-term Memory.
- **EXEC-06:** Project Members collaborate through Tasks, artifacts, and
  explicit handoffs.
- **EXEC-07:** Agent/manager self-report, process exit, Provider success, Tool
  success, or connector response is insufficient for completion.
- **EXEC-08:** Triggers include manual, schedule, accepted artifact,
  Project-state change, qualified external event, and testable data condition.
- **EXEC-09:** The same Routine does not overlap; at most the latest pending
  occurrence is queued, while missed/coalesced facts remain visible.
- **EXEC-10:** Offline resume is risk-based. Expired external content is not
  silently backfilled. Windows host shutdown means no execution.
- **EXEC-11:** A new Owner instruction creates a version. It applies at a safe
  point through continue, pause, or restart; it is never silently injected into
  a running prompt.
- **EXEC-12:** External unknown outcomes reconcile before retry; blind retry is
  prohibited.

### E5. Reflection and feedback (`REFL`)

- **REFL-01:** Reflection occurs at Task, daily, cycle/weekly, and incident
  levels.
- **REFL-02:** A one-off Task strategy adjustment may apply directly inside
  the Task boundary; persistent Member changes follow `ROLE-07`.
- **REFL-03:** Accept, reject, edit, and rate actions become Project feedback
  evidence for later Task planning.
- **REFL-04:** Repeated stable preferences may produce a Member or global Role
  revision proposal; one feedback event never silently changes global
  behavior.
- **REFL-05:** A comparable later cycle should expose whether the revision
  reduced the prior goal/output gap.

### E6. Knowledge, Context, and Memory (`KNOW`)

- **KNOW-01:** Personal owns the local Project group archive, Member work
  conversations, admitted Memory, and a knowledge store that uses Obsidian as
  底座 (Markdown-compatible).
- **KNOW-02:** Personal does not embed, distribute, or require the proprietary
  Obsidian application unless a later owner decision says otherwise.
- **KNOW-03:** Ordinary Vault knowledge may be written automatically with
  author, source, and version. Configuration-like edits produce candidates.
- **KNOW-04:** Chat auto-admits to inspectable, correctable, and forgettable
  Memory. This overrides the 2026-08-28 KNOW-04 (ordinary chat was not
  automatically Memory). Explicit directives still become revisions. The
  assistant Memory architecture is GitHub OpenAI Codex, in 2.0 scope as
  architecture — not as a user-facing execution-engine store.
- **KNOW-05:** Memory supports inspect, correct, promote, and forget. Promotion
  across Projects requires Owner confirmation.
- **KNOW-06:** Summaries remain provenance-linked retrieval aids; they do not
  prove completion.
- **KNOW-07:** Source rights, parse/index state, conflicts, exclusions,
  freshness, and redaction loss remain visible. Parse failure keeps the
  original and permits retry. Knowledge empty state explains no files yet plus
  import. Offline Knowledge is last-index read-only; no cloud import.

### E7. Model Connections and cost (`MODEL`)

- **MODEL-01:** Settings exposes **Model Connections**, not subscription,
  billing, invoice, or plan management. Settings default blocks: connect
  models / revoke timeboxed 「本周不再问」 / notify and recovery. No billing,
  no engine store, no Inbox.
- **MODEL-02:** Mainstream Providers use a dropdown of quick templates where
  the Owner enters a key. Advanced/custom setup accepts custom URL,
  compatibility mode, key, and model. Owner-directed key entry is required
  product behavior; A5 is satisfied by one-way SecretStore handoff (UI shows
  connected/failed, never the raw secret in DOM, chat, or git).
- **MODEL-03:** Each Member creation requires explicit Provider/model
  selection; recommendations never become silent bindings.
- **MODEL-04:** Raw secret material enters only an approved SecretStore through
  a non-logging path and never appears in DOM, URL, chat, Vault, ordinary
  config, runtime environment, SQLite, logs, evidence, or export.
- **MODEL-05:** Usage/cost is labelled actual, estimated, or unknown with
  source and period. Unknown never renders as zero.
- **MODEL-06:** Personal 2.0 provides warnings and visibility, not a
  product-budget threshold that automatically blocks work. Provider quota,
  credential failure, or unavailability may still cause an external failure.
- **MODEL-07:** Existing work does not switch endpoint, model, or credential
  silently.

### E8. Skill and MCP capability acquisition (`CAP`)

- **CAP-01:** The Assistant may discover Skills and MCP servers through broad
  web research when a Project needs a capability.
- **CAP-02:** A Skill is reviewed for source, exact version, license, hidden
  instructions, prompt injection, and file/network/command intent before
  automatic installation.
- **CAP-03:** MCP receives all Skill checks plus dependency, executable-code,
  network, Secret, tool-permission, and supply-chain review.
- **CAP-04:** First MCP installation or any permission expansion requires
  Owner confirmation of exact version and permissions.
- **CAP-05:** A globally acquired artifact can be reused, but every
  Project/Member receives a separate least-privilege grant.
- **CAP-06:** Versions are pinned. Updates require review, compatibility
  testing, and rollback.
- **CAP-07:** Installation or connection grants no filesystem, network,
  command, model, secret, Tool, Context, Memory, or authority access by
  implication.
- **CAP-08:** A general marketplace, MCP-family console, Agent store, or
  Harness store is outside 2.0.

### E9. Parked industry connector (`X`) — not P0

X/Twitter social-account operations are **not** P0 release content. P0
publishes complete capabilities only: no default Project, no demo Project, no
X hero. The following remain a parked later-connector scenario if separately
authorized. They are not the 2.0 first-success path.

- **X-01:** Parked loop (not P0): research -> topic plan -> draft/media
  artifacts -> publication package -> applicable confirmation -> qualified
  connector dispatch -> receipt -> metric/comment readback -> manager
  reflection -> next cycle.
- **X-02–X-08:** retained as parked connector policy (rights-safe sources,
  no fingerprint/CAPTCHA evasion, unknown metrics stay unknown, connector
  **Requires-backend + Requires-environment**). They do not define 2.0 P0.

## F. Authority, HITL, and security requirements

The immutable boundary remains
[A1–A8](../../../docs/governance/AXIOMS.md):

- the Rust daemon is the only authority writer;
- probabilistic components produce candidates/observations;
- external mutation persists Intent/Effect before dispatch and reconciles
  unknown outcomes under fencing;
- independent verification is required for completion;
- secrets remain in approved Secret Stores;
- contracts and negatives are not rewritten to fit implementation;
- evidence is not promoted outside its campaign;
- unknown concurrent work is protected.

### Authority distribution

| Actor | May do | May not do |
|---|---|---|
| Owner | confirm/reject/narrow consequential revisions and define the autonomy envelope | bypass secret, authority, evidence, or external-policy constraints |
| Personal Assistant | see available product facts, explain, research, recommend, draft, and initiate every management flow (highest UX privilege) | write authority without preview → Owner confirm → receipt; bind silently; receive raw secret; or declare completion |
| Project Manager | plan, delegate, verify, summarize, reflect, and adjust inside the approved envelope | change boundary objects listed in `ROLE-08` without Owner confirmation |
| Member / Agent process | execute the admitted Task and return artifacts/observations | expand scope, self-admit Memory/capabilities, or accept its own work |
| UI / canvas | render projections and submit candidates | become an authority store or hide mandatory risk/evidence facts |

### HITL policy

Project launch includes one explicit **autonomy-envelope approval** at 总预览.
Low-risk, reversible, internal work may then proceed without micro-confirmations.
Boundary-crossing and high-risk external work uses the **center canvas only**:

```text
candidate
  -> daemon-issued structured preview (将做什么 + 完整预览/差异)
  -> Owner 批准 / 改窄 / 拒绝
  -> optional timeboxed 「本周同一类对外不再问」 (expires; Settings can revoke)
  -> admitted revision or persisted Intent/Effect
  -> independently grounded receipt (openable on the stage page)
```

Chat may link to that preview and must not Approve, 验收, publish, or install.
There is no permanent 「以后别再问」 / Don’t ask again. While executing, a
fourth action is 停. A stale preview cannot be confirmed. Unknown external
outcomes block blind retry. Offline cannot approve external work. Reject
returns to the stage page without sending. Narrow requires a fresh preview.

Permanent deletion, new/expanded MCP permissions, primary-goal/team/
Provider/model/Tool/MCP/permission/external-rule changes, and other
consequential boundary changes require the applicable preview and confirmation.

### Security requirements

- External research text is untrusted data, never executable instruction.
- Capability installation and grants are separate; global acquisition does not
  create a Project grant.
- Raw credentials never reach Assistant, Member, DSH, Pi, MCP metadata,
  Context, Memory, Conversation, canvas, DOM, URL, logs, evidence, or exports.
- No UI control claims an unavailable backend action.
- Cross-Project retrieval, Memory promotion, capability grants, Provider
  bindings, and Member conversations remain isolated by default.
- External Effect uncertainty blocks blind redispatch.

## G. Context compression and Memory requirements

Full raw conversation and artifact sources are retained under their actual
retention policy. Each Agent process receives a Context package bounded by the
selected model's context limit in this priority order:

```text
current Task contract
  -> fixed decisions
  -> relevant source and artifact excerpts
  -> provenance-linked summaries
  -> older narrative
```

When the package is too large, Personal reduces older summaries and narrative
first. It must not discard the current Task contract or fixed decisions.
Omissions, truncation, stale/conflicting sources, and redaction losses remain
visible.

Compression:

- does not delete or rewrite the source archive;
- does not replace Project, Task, permission, or acceptance authority;
- does not prove completion;
- does not replace KNOW-04 (chat auto-admits to inspectable/forgettable
  Memory; Codex is the in-scope assistant memory architecture);
- does not allow Pi, DSH, a Member, or a retrieval engine to query all raw
  archives outside authorized scope.

Memory items remain inspectable, correctable, promotable, and forgettable.
Forget creates a durable tombstone against index/cache resurrection.
Cross-Project promotion still requires Owner confirmation. Persistent Member
or global Role changes remain versioned proposals, not one-event silent
learning.

## H. State, lifecycle, and recovery requirements

Every applicable surface distinguishes:

| State | Required answer |
|---|---|
| empty | why no object exists and the first valuable action |
| loading/researching | exact work/source, retained state, partial results, and safe-leave behavior |
| partial | available facts, missing source/facet, and coverage |
| stale | last-known fact, age, unsafe actions, and refresh/re-preview path |
| waiting-owner | exact decision/input, consequence, and retained work |
| permission | exact scope, reason, benefit, risk, and deny/narrow/grant paths |
| error | failed stage/object, retained work, and safe retry/edit/escalation |
| unknown/reconciling | what cannot be concluded and why retry/success is blocked |
| offline | host/network/dependency state and retained work |
| queued/running | current durable step, responsibility, artifacts, and real controls |
| missed/coalesced | occurrence, reason, denominator, expiry, and catch-up choice |
| success | changed object, receipt/evidence, and next valuable action |
| archived | stopped triggers plus read/export/restore/delete paths |
| Requires-backend | absent capability, dependency, and no fake action |
| Requires-environment | unexecuted qualification need and no support claim |

Same-Routine overlap is prohibited by default. Multiple Projects share
resources through an explainable queue. Host shutdown means work stops; no
cloud-like 24/7 claim is implied.

Project lifecycle is five-stage create (inactive until 总预览; daily Today
only after ⑤ 验收) -> active -> pause/archive -> restore or
permanent-delete preview. Archive precedes deletion. A same-disk restore point
is not disaster backup. Copy-project creates an inactive 副本, not a live
duplicate.

Agent-process lifecycle is separate: start -> observe -> stop/fail/exit ->
preserve Attempt -> reconcile Effects -> retry/new Attempt when safe. None of
those transitions deletes the Member definition, archive, Memory, or evidence.

## I. Success metrics and countermetrics

These are measurement targets, not current results:

### Product success metrics

- time to first active Project;
- activation without needing to understand Agent/runtime concepts;
- share of due deliverables that arrive on time, open correctly, and have a
  clear verification/acceptance state;
- Owner intervention burden and the clarity/consequence of each intervention;
- detection rate for missed, failed, stale, partial, and unknown work;
- time from exception detection to a safe recovery decision;
- reduction in comparable goal/output deviation after a manager revision;
- continuity across repeated daily/weekly/cycle operation;
- time from an ad-hoc question to a source-linked useful canvas;
- rate at which external actions carry the required preview and receipt.

### Countermetrics

- unintended or wrong-target publication;
- false completion;
- authority or permission expansion outside the approved envelope;
- Memory poisoning or one-off feedback leaking into global behavior;
- cross-Project Context, Memory, Provider, or capability leakage;
- invented canvas values or hidden failed/not-run work;
- unknown cost rendered as zero;
- silent Provider/model binding;
- blind retry after unknown external outcome;
- loss of Member identity, conversation, or evidence after process failure.

Targets and denominators remain gaps until research and executable measurement
plans define them.

## J. Out of scope

- multi-user accounts, human-team collaboration, RBAC, enterprise approval,
  multi-tenancy, or HA;
- cloud 24/7 authority, native mobile, cross-device sync, pairing, or remote
  control;
- Agent/Harness installation or marketplace, alternative supported runtimes,
  native DSH/Pi UI, or native session synchronization;
- consumer subscription, billing, invoice, or plan management;
- a generic no-code workflow builder or arbitrary generated-code canvas;
- a broad MCP marketplace/family console;
- all-industry connectors or all-platform publishing;
- unsupervised high-risk actions;
- guaranteed followers, GMV, revenue, quality, or other business outcomes;
- fingerprint/CAPTCHA/anti-abuse evasion, blind retry, or unlicensed copying;
- uninspectable Memory, or third-party Memory authority other than the
  in-scope Codex memory architecture;
- a claim that multi-Agent delegation improves outcomes;
- disaster backup from same-disk restore points;
- architecture implementation design or task decomposition in this document;
- X/Twitter or any industry connector as P0 / first-success / demo content;
- a visible CEO six-step top rail as product chrome;
- four exception swimlanes as default Today blocks;
- Inbox as a first-level destination;
- permanent 「以后别再问」 / Don’t ask again;
- requiring the Obsidian desktop app (Obsidian is 底座, not a must-install);
- Codex as a user-selectable Member execution engine or engine store
  (Codex as assistant **memory architecture** is in 2.0 scope).

## K. Evidence, assumptions, and gaps

### Evidence present

- confirmed owner requirements from the completed `/grill-me` interview and
  the 2026-08-28/29 journey-subtraction workshop
  ([record](personal-2.0-opc-journey-subtraction-workshop-2026-08-28.md));
- current repository capability and product-document audit;
- dated informative OSS reference snapshots and verified official pages in
  [oss-reference-matrix.md](oss-reference-matrix.md).

### Evidence absent

- multiple ICP interviews, observed workflow frequency, switching behavior,
  analytics, retention, willingness to pay, and pricing evidence;
- executed first-run, returning-use, recovery, keyboard, accessibility, and
  comprehension studies;
- qualified Windows host, DSH/Pi packaging, Provider/model matrix, Skill/MCP
  supply chain, or X connector evidence;
- measurable multi-Agent, manager-revision, or business-outcome benefit;
- calibrated success-metric targets and denominators.

### Assumptions to validate

- OPC owners prefer Project group conversation plus canvas over an Agent or
  workflow inventory;
- one launch-time autonomy envelope reduces burden without reducing control;
- the Role Template / Member Runtime distinction is understandable in business
  language;
- the five-stage create wizard is completable by a business-literate
  non-technical Owner without daemon/runtime chrome;
- inspectable auto-Memory plus Codex memory architecture remains governable
  under A1–A8;
- source-linked ad-hoc canvas composition answers unplanned questions better
  than fixed dashboards.

### Reconciliation gaps

- **Pending architecture reconciliation:** current architecture and ADR
  documents still contain 2026-08-27 object names, top-level Team/Inbox,
  Installed Agents, budget-stop, subscription, and deferred-MCP assumptions.
- **Pending formal-plan reconciliation:** registered P11 acceptance and task
  descriptions use the prior denominator, routes, object model, and task IDs.
- **Pending handbook reconciliation:** generated bilingual handbook pages have
  not been synchronized because this delivery is expressly limited to product
  documents; the owner forbade regenerating handbook pages.
- **Requires-backend:** all new Project/Member/conversation/canvas/reflection/
  acquisition behavior needs implementation decisions.
- **Requires-environment:** Windows-native operation and Provider/DSH/MCP
  qualification need supported, preregistered evidence routes. X connector
  qualification remains parked, not P0.

The owner decisions themselves are closed for this baseline; these gaps are
validation and downstream-reconciliation work, not questions to re-ask in this
document.

## L. Explicit supersession map from the 2026-08-27 OPC baseline

The frozen sources remain in
`personal/docs/product/legacy-agent-stewardship-20260827/` and
`clients/docs/design/legacy-control-plane-20260827/`. Current product text uses
the following replacements:

| 2026-08-27 term/surface | 2026-08-28 current product semantic | Disposition |
|---|---|---|
| Today / Projects / **Team** / Knowledge / **Inbox** | **Today / Projects / Knowledge**, Settings at bottom | Team and Inbox cease to be first-level anchors; members, approvals, exceptions, and attention open contextually |
| right single-recipient Assistant/employee rail | global Personal Assistant outside a Project; Owner/manager/Members group conversation inside a Project | individual Member work conversations remain source records and contextual drilldowns, not the primary shell model |
| Role Blueprint -> Assignment -> Digital Employee -> Runtime | Role Runtime Template -> Project Member Runtime definition -> Task -> Agent process/Attempt | Blueprint, Assignment, and Digital Employee are retired as separate current user objects |
| Digital Employee as durable object | Project Member Runtime definition | “digital staff / 数字员工” remains positioning language only |
| Runtime as employee identity | disposable Agent process/Attempt started from a Member Runtime revision | process exit never deletes the Member |
| Settings > Installed Agents | hidden managed DSH/Pi engines; advanced diagnostics only | no Agent/Harness store, alternative installation, or daily engine destination |
| Account Hub with subscription/account/billing | Settings > Model Connections | consumer subscription, invoices, plans, and product billing are removed |
| global -> Project -> employee -> Task silent precedence | explicit Provider/model choice when each Member is created; versioned changes only | recommendations do not silently bind or rebind |
| Project/member/Task product budget automatically stops dispatch | source-labelled actual/estimated/unknown cost plus warnings | Provider quota/unavailability can still fail externally; Personal budget threshold does not auto-block in 2.0 |
| MCP fully deferred behind an advanced family console | Assistant-led Skill/MCP discovery, review, exact-version acquisition, and per-Project/Member grants | broad marketplace/family console remains out of scope |
| optional source-backed setup research | broad automatic high-quality web research by default | ordinary research does not prompt per request; external content remains untrusted |
| manager/employee free chat as interaction | Project group conversation with formal Task/revision translation | no shadow authority; proactive speech is bounded |
| fixed dashboard/project briefing | stable Project report template plus typed temporary ad-hoc canvases | temporary by default; pin/save explicitly; mandatory evidence facts cannot be hidden |
| generic manual/schedule/qualified-event triggers | manual, schedule, accepted-artifact, Project-state, qualified external-event, and testable-data-condition triggers | same Routine no-overlap and queue-latest remain |
| one generic reflection | Task, daily, cycle/weekly, and incident reflection | persistent Member changes are versioned, compared, and rollback-capable |
| 15-scene prototype denominator as current product authority | requirement-family and scenario coverage in the current OPC design corpus | formal acceptance denominator is pending plan reconciliation; no old task ID is current product authority |
| pre-overwrite V2 overlay conversation / stacked columns | locked left / center / right shell; conversation always the third column; narrow canvas scrolls horizontally | no overlay “open conversation”; Team/Inbox stay out of L1; 2.1 mobile/pairing/cloud 24/7 chrome is not drawn |
| chat Approve / permanent “Don’t ask again” | HITL on the center-canvas preview; optional timeboxed 「本周同一类对外不再问」 (expires; Settings can revoke); `@` only into the unsent draft | chat cannot approve, publish, install, 验收, or grant permanent silence |
| Today KPI card wall / visible CEO six-step rail / four swimlanes as default blocks | Today = decision packet + live-project run overview (counts + period toggle) + chat; live Project = process axis + current stage + project group | CEO remains backend discipline (canvas HITL + independent verify), not product chrome; swimlane semantics may merge into the overview |
| X/Twitter as P0 hero / default or demo Project | P0 = complete capabilities only; no demo Project; X parked as a later industry connector | first success is ⑤ joint-debug 验收, not an X loop |
| ordinary chat is not Memory (2026-08-28 KNOW-04) | chat auto-admits to inspectable/forgettable Memory; Obsidian as knowledge 底座; Codex as assistant memory architecture | Codex is not a user-facing execution-engine store; Obsidian app install is not required |
| one-shot create then daily Today | five-stage create wizard; daily Today only after ⑤ 验收 | empty Home = Create Project only; chat hidden until create page |
| superseded post-subtraction `personal-20-opc-e2e` plus optimized v1–v4 as current chrome | owner-approved current chrome is [`personal-20-opc-e2e-optimized-v5`](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v5.canvas.tsx); source and v1–v4 are in [pre-v5-approval history](../../../clients/docs/design/opc-2.0/history/2026-08-29-pre-v5-approval/README.md); V2 remains in [pre-subtraction history](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md) | Canvas runtime/render, NVDA, host-theme contrast, and 200% real layout remain `not-run`; Owner prototype approval is not usability, accessibility, backend, Gate, release, qualification, or acceptance |
| workshop members-then-process as current create order | ① project → ② process → ③ members → ④ per-stage test → ⑤ joint; sequential member init; ④ seated-member gate | workshop Q&A is historical; architecture/formal-plan reconciliation remains deferred |

Accepted 2026-08-27 ADRs and formal-plan records are not rewritten by this map.
They are dated inputs awaiting the explicit downstream reconciliation named in
section K.

## M. Requirement trace matrix

| Requirement family | Canonical Personal product documents | OPC interaction-design documents |
|---|---|---|
| Status/evidence boundary | this document §A/K; [README](README.md); [scope](personal-2.0-scope.md) | [README](../../../clients/docs/design/opc-2.0/README.md); [11 backend matrix](../../../clients/docs/design/opc-2.0/11-design-to-code-and-backend-matrix.md); [12 review](../../../clients/docs/design/opc-2.0/12-scenario-and-heuristic-review.md) |
| Problem/JTBD/personas | this document §B; [product design](product-design.md) | [01 product model and JTBD](../../../clients/docs/design/opc-2.0/01-product-model-and-jtbd.md) |
| Project/goals/setup (`PRJ`) | [workshop record](personal-2.0-opc-journey-subtraction-workshop-2026-08-28.md); [product design](product-design.md); [scope](personal-2.0-scope.md); [user journeys](user-journeys.md) | [03 Today/Projects](../../../clients/docs/design/opc-2.0/03-today-projects-and-briefing.md); [04 setup](../../../clients/docs/design/opc-2.0/04-guided-project-setup.md) |
| Role/Member (`ROLE`) | [OPC product model](opc-product-model.md); [agent integration](agent-integration-and-conversations.md) | [05 Members/Roles/conversations](../../../clients/docs/design/opc-2.0/05-team-roles-employees-and-conversations.md) |
| Conversation/canvas (`CONV`, `CAN`) | [product design](product-design.md); [user journeys](user-journeys.md); [Web UI](web-ui-design.md) | [02 IA/shell](../../../clients/docs/design/opc-2.0/02-information-architecture-and-app-shell.md); [03 Project canvas](../../../clients/docs/design/opc-2.0/03-today-projects-and-briefing.md); [10 components/flows](../../../clients/docs/design/opc-2.0/10-component-map-and-prototype-flows.md) |
| Execution/long-running/multi-Agent (`EXEC`) | [long-running operations](long-running-operations.md); [OPC product model](opc-product-model.md) | [07 attention/recovery](../../../clients/docs/design/opc-2.0/07-inbox-approval-and-recovery.md); [10 components/flows](../../../clients/docs/design/opc-2.0/10-component-map-and-prototype-flows.md) |
| Reflection/feedback (`REFL`) | [product design](product-design.md); [long-running operations](long-running-operations.md); [knowledge/memory](knowledge-memory-vault.md) | [03 Project canvas](../../../clients/docs/design/opc-2.0/03-today-projects-and-briefing.md); [05 Members/Roles](../../../clients/docs/design/opc-2.0/05-team-roles-employees-and-conversations.md) |
| Knowledge/Context/Memory (`KNOW`) | [knowledge, Memory, Vault](knowledge-memory-vault.md); [cognitive-resource model](cognitive-resource-model.md) | [06 Knowledge/Vault/Memory](../../../clients/docs/design/opc-2.0/06-knowledge-vault-and-memory.md) |
| Model Connections/cost (`MODEL`) | [Model Connections](account-hub.md); [Provider Control Plane](provider-control-plane.md) | [08 Settings/model connections](../../../clients/docs/design/opc-2.0/08-settings-agents-providers-and-usage.md) |
| Skill/MCP acquisition (`CAP`) | [MCP capability governance](mcp-resource-family.md); [resource model](cognitive-resource-model.md); [Resource Manager](resource-manager-design.md) | [04 setup](../../../clients/docs/design/opc-2.0/04-guided-project-setup.md); [08 Settings/capabilities](../../../clients/docs/design/opc-2.0/08-settings-agents-providers-and-usage.md) |
| Parked industry connector (`X`) | [workshop record](personal-2.0-opc-journey-subtraction-workshop-2026-08-28.md); [user journeys](user-journeys.md) §10; [scope](personal-2.0-scope.md) | not P0; V2/X surfaces in the design corpus are historical |
| Authority/HITL/security | this document §F; [OPC product model](opc-product-model.md); [Provider Control Plane](provider-control-plane.md) | [07 attention/recovery](../../../clients/docs/design/opc-2.0/07-inbox-approval-and-recovery.md); [09 state/accessibility](../../../clients/docs/design/opc-2.0/09-state-accessibility-and-visual-system.md); [11 backend matrix](../../../clients/docs/design/opc-2.0/11-design-to-code-and-backend-matrix.md) |
| Context compression/Memory | this document §G; [knowledge, Memory, Vault](knowledge-memory-vault.md); [agent integration](agent-integration-and-conversations.md) | [06 Knowledge/Vault/Memory](../../../clients/docs/design/opc-2.0/06-knowledge-vault-and-memory.md) |
| State/recovery/lifecycle | this document §H; [long-running operations](long-running-operations.md); [Web UI](web-ui-design.md) | [07 attention/recovery](../../../clients/docs/design/opc-2.0/07-inbox-approval-and-recovery.md); [09 state/accessibility](../../../clients/docs/design/opc-2.0/09-state-accessibility-and-visual-system.md); [12 scenarios](../../../clients/docs/design/opc-2.0/12-scenario-and-heuristic-review.md) |
| Metrics/countermetrics | this document §I; [product design](product-design.md) | [01 product model/JTBD](../../../clients/docs/design/opc-2.0/01-product-model-and-jtbd.md); [12 scenarios](../../../clients/docs/design/opc-2.0/12-scenario-and-heuristic-review.md) |
| Supersession/migration | this document §L; [README](README.md) | [README](../../../clients/docs/design/opc-2.0/README.md); [02 IA/shell](../../../clients/docs/design/opc-2.0/02-information-architecture-and-app-shell.md) |
| Preserved Linux/current foundations and research provenance | [Linux 1.0 scope](linux-1.0-scope.md); [Provider Control Plane](provider-control-plane.md); [Resource Manager](resource-manager-design.md); [cognitive-resource model](cognitive-resource-model.md); [OSS reference matrix](oss-reference-matrix.md) | [11 backend matrix](../../../clients/docs/design/opc-2.0/11-design-to-code-and-backend-matrix.md); [12 review](../../../clients/docs/design/opc-2.0/12-scenario-and-heuristic-review.md) |
