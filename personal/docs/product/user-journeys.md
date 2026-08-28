# CognitiveOS Personal 2.0 user journeys

- Status: adopted target journeys; not usability or implementation evidence
- Product intent: [Product design](product-design.md)
- Product model: [OPC product model](opc-product-model.md)
- Scope: [Personal 2.0](personal-2.0-scope.md)
- Requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Workshop record (verbatim Q&A + scheme snapshot):
  [journey-subtraction workshop 2026-08-28](personal-2.0-opc-journey-subtraction-workshop-2026-08-28.md)
- Current interaction prototype:
  [**personal-20-opc-e2e (post journey-subtraction)**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e.canvas.tsx)
- Archived historical V2 (not current chrome):
  [pre-subtraction history](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Prototype identity: current chrome is `personal-20-opc-e2e` after the
  workshop. Archived V2 is historical, not current chrome. Canvas-only HITL
  and daemon authority path remain.
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

Every journey separates candidate explanation from daemon authority. The
Personal Assistant, manager, Member, Pi, DSH, UI, and connector may propose,
observe, or execute admitted bounded work. Only the daemon authorizes and
accepts; independent verification, not Agent self-report, closes work. Default
UI does not display daemon / DSH / Pi / Harness / Loop to the 小白 Owner.

## 1. Create and activate the first Project

Empty Home (no Project): center is only **Create Project**; right chat is
hidden. Knowledge is locked. Settings may be opened only to connect a model.
Today is not the daily decision packet.

1. Choose **Create Project**. The product jumps to the Projects create page.
2. If no Provider is bound, the opened chat only guides the Owner to Settings
   to connect a model and bind the assistant. If a model is bound, the right
   chat is a normal assistant: describe the business situation and intended
   outputs. Leave saves a draft and resumes.
3. The Assistant analyzes and, when online, researches (GitHub, skill hubs,
   and similar). The Owner confirms item-by-item: process, per-stage outputs,
   cycle, save format, Skill/tools/MCP/knowledge/env/file permissions,
   auto-vs-approve (including external), triggers, cost, source rights, and
   总预览. Runtime method is labelled **执行方式** (the word Harness does not
   appear). Secrets never enter chat. No silent model bind. Connection
   failure names the problem. The Project stays inactive until 总预览.
4. ② Member init: roster (duty / what to hand over) + chat + 「确认这个班子」.
   Model is required per person; Skill/MCP wait for ③ 执行方式. Refuse = not
   joined. Missing model = pending, not silent bind.
5. ③ Process init: one process axis, one stage at a time, total goal + cycle
   on the axis, right chat. 「确认这一环」 then the next; after the last
   stage, confirm total goal + project trigger. Unknown gaps stay on the axis
   and are not marked ready.
6. ④ Per-stage test: which stage is under test, openable result + pass/fail,
   「通过，下一环」. Fail returns to ③ for that stage. Unknown cannot pass.
   Offline cannot start a test. No process/engine chrome.
7. ⑤ Joint debug: where the full flow is, openable overall artifact + verify
   state, 「验收，进入 Today」. Fail names the stage and returns to ④/③.
   Unknown cannot 验收. Offline cannot joint-debug. No fake publish. Aha is
   this 验收.
8. Copy-project (when a launchable Project exists): from the Projects list or
   project page, copy goals, process axis, role/member definitions, 执行方式,
   triggers, and output forms. Do not copy secrets, in-flight tasks, external
   receipts, or timeboxed skips. Lands as an inactive 副本; edit, then 总预览.
   ④⑤ may be spot-checked or skipped. This path does not restart at ①.

Daily Today is forbidden until all five stages complete. Until then Today
shows only 「继续未完成的创建」; Projects exposes only this incomplete
create; Knowledge opens at ③ when input is needed (current draft only).

Required setup states:

| State | Treatment |
|---|---|
| `local-draft` | browser/client draft is preserved but not authority |
| `daemon-draft` | draft has a daemon identity and version; still inactive |
| `researching` | sources, coverage, cancellation, and partial findings are visible |
| `waiting` | names the exact Owner input or permission needed |
| `re-preview` | an earlier preview is stale; diff and user edits are preserved |
| `creating` | durable operation and safe-leave behavior are visible |
| `failed` | names the failed stage, retained draft, and retry/edit path |
| `active-receipt` | charter revision, Project id, first plan, team, and next action |

The Project is not active before 总预览. A research result or assistant
message cannot activate it. ⑤ 验收 is first success; card-in-the-middle after
① is not.

## 2. Use Today without a KPI wall

After ⑤, the returning Owner opens Today as three default blocks:

1. **Decision packet** — the consequential Owner decision (consequence,
   reversibility, alternatives, kernel truth, why option A is first). This is
   the only primary CTA (去处理这一件拍板).
2. **Live-project run overview** — current state, today's completed-run
   count, current stage, elapsed time on current work; plus created / live /
   blocked project counts; today / week / month toggle. Click a live project
   for stage name, responsible member, today's complete/fail counts, average
   duration, and success rate. Four exception swimlanes are not default
   blocks; their semantics may merge into this overview.
3. **Assistant** — natural-language questions about run data and analysis.
   Chat cannot approve. Mis-tapping statistics cannot publish. Decline / later
   leaves the item on Today.

Empty: no pending decision and no live project (incomplete create still shows
only continue-create). Loading: overview refreshes; the packet stays
clickable. Blocked: blocked count is visible; open to handle. Unknown: the
row says 说不清; unknown cost is never 0. Offline: last-known overview marked
stale; not current success.

## 3. Use the Project group and process-axis canvas

A live Project opens to ① the business-process axis ② this stage (status,
responsible member, auth/verify marks and content) ③ the Owner/manager/Members
group conversation. There is no visible CEO six-step top rail. Click the axis
to change stage. When the Owner must act, the primary button is in ②. Chat
cannot approve.

Empty stage: not started, what is missing. In progress: who, for how long.
Blocked: needs auth or input; primary button handles it. Unknown: cannot mark
complete. Offline: last status marked stale. Verify fail: stay on this stage;
do not jump ahead.

HITL is announced in chat and linked to the center preview; chat has no
Approve control and no permanent “Don’t ask again” grant. Close-out is
openable artifact + verify state + 「验收，回 Today」. Fail stays on the
stage. Chat cannot 验收. There is no Inbox first-level entry and no fake
publish.

The Owner may `@manager` for progress or delegation and `@member` to ask or
temporarily redirect goal/path inside an approved boundary. `@member` creates
a formal Task revision, not a shadow plan. `@` inserts only into the unsent
draft. The manager speaks by default. A Member speaks proactively only when
mentioned, submitting a deliverable, handing off, blocked, or requesting a
decision. A work-changing message becomes a formal Task or revision before it
has authority. A Member work conversation is visible to the Owner, the
manager, and that Member.

For a new question, the Assistant/manager interprets intent, reads real Project
results, and composes a temporary canvas from approved goal, artifact,
evidence, timeline, organization, decision, and metric components. It is not
saved unless pinned or made a template. Generated code/`eval`, invented data,
and hiding goal/acceptance state, failure/not-run work, Owner decisions, source,
or freshness are forbidden.

The Owner may drill down through Goal -> Plan revision -> Routine/Task ->
Attempt -> Effect/Evidence. Ordinary process traces remain collapsed. Engine
and daemon chrome stay hidden.

A manager can autonomously reorder approved Tasks or change a Member's bounded
responsibility. A primary-goal, team, Provider/model, Tool/MCP, permission, or
external-rule change creates a revision candidate and daemon preview. Rejection
keeps the draft and current plan; confirmation produces a new revision and
receipt without erasing prior plans or Attempts.

## 4. Add a Role and Project Member

Daily add-member (five-stage create already complete):

1. From the live Project, open the roster. Unique task: add one post (what to
   do, what to hand over), confirm, then disclose 执行方式. Not “install MCP
   first” and not “open the engine”.
2. Default blocks: existing roster + chat suggesting the new post + 「确认加入」.
   Model is required. Skill/MCP/file permissions wait for 执行方式.
3. Refuse = not joined. No model = pending; go to Settings. After join,
   process/permission changes need another approval; no silent grant. Leave
   saves draft. Empty = this post does not exist yet. In progress = suggesting
   a post. Offline = duties can be edited; online search for a post scheme
   cannot.

A first Task starts a disposable Agent process/Attempt from the exact Member
revision. Process exit does not delete the Member, Conversation, Memory, or
evidence. Members are not shared across Projects.

## 5. Import knowledge and retrieve bounded context

1. Knowledge offers current project files, **Why this fragment**, and import.
   Without a Project, Knowledge stays locked. During create, it opens at ③
   only for the current draft.
2. The Owner imports files, directories, links, images, or video metadata.
3. Personal copies permitted source material into the selected archive,
   preserves source/provenance, detects duplicates and credentials, and shows
   parsing/OCR/index progress.
4. Parse failure preserves the original and permits retry, exclusion, or manual
   classification. Empty = no files yet + import. Offline = last index
   read-only; no cloud import.
5. Ordinary knowledge edits reindex. A goal/role/permission/workflow-like edit
   becomes a candidate and cannot silently mutate authority.
6. Retrieval shows scope, source, freshness, redaction, and a **Why this
   fragment** table for each selected excerpt.

The knowledge store uses Obsidian as 底座; 2.0 does not require installing the
Obsidian app. Chat auto-admits to inspectable, correctable, and forgettable
Memory (overrides 2026-08-28 KNOW-04). The assistant Memory architecture is
GitHub OpenAI Codex, in 2.0 scope as architecture.

Conversation archives can participate, but Personal injects only relevant,
bounded, redacted, provenance-bearing, untrusted observations in this order:
current Task contract -> fixed decisions -> relevant source/artifact excerpts
-> provenance-linked summaries -> older narrative. Over-limit reduction removes
older narrative first. Full raw sources remain; a summary does not prove
completion.

Cross-Project promotion requires Owner confirmation. Accept/reject/edit/rate
actions become Project feedback evidence; stable repeated preference may
produce a versioned Member/global Role proposal, never a silent one-event
change.

## 6. Resolve contextual attention, approval, and recovery

Approvals, requested input, permissions, execution blocks, unknown Effects,
missed runs, and cost warnings appear in Today and on the affected stage.
They do not require a permanent first-level Inbox destination.

An item answers: Project/Member, reason, consequence, affected targets,
reversibility, source/freshness, deadline or age, and next safe actions.
The canvas shows ① what will be done ② full preview/diff ③ 批准 / 改窄 /
拒绝, plus optional timeboxed 「本周同一类对外不再问」 (expires; Settings can
revoke). Chat may link to that preview; it cannot Approve. While executing,
a fourth action is 停. Narrow requires a fresh preview. A stale preview
cannot be confirmed. Reject does not send and returns to the stage.
Unknown external outcomes expose reconcile status; blind retry is forbidden.
Offline cannot approve external work. Empty = nothing pending. Success
receipt is openable on the stage.

Working is not completion. After an error, typed input and draft work remain.
A successful action ends in a receipt linked to the affected Project object.

## 7. Schedule, miss, and resume a Routine

1. A Routine revision defines bounded work, trigger, no-overlap policy,
   cost visibility, and risk class.
2. Manual, schedule, accepted-artifact, Project-state, qualified external
   event, or testable data condition creates a run request.
3. If one is active, only the latest pending occurrence is queued; superseded
   occurrences remain in the ledger.
4. Sleep/offline pauses dispatch and produces missed facts.
5. On return, Personal shows what was missed and why.
6. Low-risk internal work may resume under policy; publishing,
   communication, spending, permission expansion, or other consequential work
   returns to contextual attention for fresh review.
7. Expired external content is not silently backfilled.
8. Across Projects, Personal explains queue order using Owner priority,
   deadline, schedule, resource availability, and fairness.

Closing the window asks **Continue eligible work in background** or **Pause**.
The choice is explicit and remembered only as a policy revision. Host shutdown
never implies 24/7 work.

A new Owner instruction becomes a version applied at a safe point through
continue, pause, or restart; it is never injected silently into a running
prompt.

## 8. Connect a model and inspect advanced engine diagnostics

Settings > Model Connections offers a dropdown of mainstream Provider
templates and an advanced custom URL/compatibility-mode/key/model flow. The
Owner enters keys. Raw keys use a one-way daemon handoff into an approved
SecretStore and are never echoed into DOM, URL, chat, Vault, configuration,
runtime environment, or evidence. Empty Settings = not connected yet; go
connect. Settings also revokes timeboxed 「本周不再问」 and hosts
notify/recovery. No billing, no engine store, no Inbox.

Creating every Member requires an explicit Provider/model choice. The
Assistant may recommend but cannot bind or rebind silently. Cost appears as
source-labelled actual, estimated, or unknown; unknown is never zero. Personal
warns but does not automatically stop work at a product budget threshold.
Provider quota or unavailability may still produce an external failure.

DSH and Pi are absent from ordinary navigation and from the 小白 default UI.
Fault recovery or advanced diagnostics may show exact version, provenance,
health, qualification, update/rollback, affected Members/Tasks, and whether
restart is required. There is no native DSH/Pi UI, native conversation
synchronization, Agent store, or Harness selector.

## 9. Acquire a Skill or MCP capability

1. The Assistant discovers candidates online from a Project need.
2. A Skill review covers source, exact version, license, hidden instructions,
   prompt injection, and file/network/command intent.
3. MCP adds dependency, executable-code, network, Secret, tool-permission, and
   supply-chain review.
4. First MCP installation or permission expansion pauses for Owner confirmation
   of exact version and permissions.
5. The acquired artifact may be reused globally, but each Project/Member
   receives a separate least-privilege grant.
6. Versions stay pinned; update requires review, compatibility testing, and
   rollback.

Installation or connection grants nothing by implication. A broad marketplace
or MCP-family console is outside 2.0.

## 10. Parked X/Twitter scenario (not P0)

X/Twitter social-account operations are **not** P0 release content. P0 is
complete capabilities only; there is no default or demo Project. This section
is retained as a parked later-connector scenario. It is not the first-success
path (that is §1 ⑤ 验收).

If separately authorized later: Owner and manager would confirm account
position, audience, allowed content forms, cadence, metrics, source-rights
rules, cost visibility, and external-action policy; publishing would still
require canvas HITL, rights-safe sources, no fingerprint/CAPTCHA evasion, and
unknown metrics staying unknown. Manual publication would remain a degraded
fallback. The Project model stays generic; this is not a hard-coded success
path or business-result promise.

## 11. Archive, restore, export, or delete a Project

Archive stops new triggers and preserves Project history. A same-disk local
restore point can reverse eligible local changes but is labelled **not a
disaster backup**. Manual export identifies included data and excludes secrets
by default.

Permanent deletion is a separate impact-preview action after archive. It names
Project authority, files, Vault, conversations, Memory, artifacts, restore
points, bindings, pending Effects, and what cannot be recovered. A second
confirmation is required.

## 12. Surface state contract

Every Today, Projects, Knowledge, Settings, setup, Project canvas/group,
contextual Member/attention, and inspector surface defines all applicable
states:

| State | Required answer |
|---|---|
| empty | why no object exists and the first value action |
| loading | exact source/work, stable content, and safe-leave behavior |
| partial | missing source/facet and what still works |
| stale | last-known fact, age, unsafe actions, and refresh path |
| permission | exact scope, reason, deny/narrow/grant paths |
| error | failed stage, preserved work, safe retry/edit/escalation |
| unknown | what cannot be concluded; never rendered as healthy/zero/success |
| offline | host/network dependency, retained work, reconnect behavior |
| missed | missed/coalesced work, risk class, resume/approval choice |
| long-running | plan, current step, artifacts, real controls, blocked reason |
| success | changed object, receipt/evidence, next valuable action |
| archived | read/export/restore/delete options and trigger state |

An unavailable backend is labelled **Requires-backend** and rendered as
explanation/navigation, not an active-looking button.

## 13. Evidence boundary

These journeys are specifications. The
[**current interaction prototype**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e.canvas.tsx)
is the post journey-subtraction canvas. Archived V2 CEO-rail / X-hero files are
in the
[pre-subtraction history folder](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
and are not current chrome. The 2026-08-28/29
[workshop record](personal-2.0-opc-journey-subtraction-workshop-2026-08-28.md)
is the scheme snapshot that canvas implements. Source/static checks and Owner approval
are not human
usability, accessibility, backend, Gate, or acceptance evidence. Canvas
runtime/render, NVDA, host-theme contrast, and 200% real layout remain
`not-run`. Formal acceptance requires a future reconciled fixed denominator,
supported exact revision, and honest pass/fail/partial/not-run accounting. The
2026-08-27 Phase 11 task IDs and 15-scene denominator are pending formal-plan
reconciliation and do not define current product semantics.
