# CognitiveOS Personal 2.0 user journeys

- Status: adopted target journeys; not usability or implementation evidence
- Product intent: [Product design](product-design.md)
- Product model: [OPC product model](opc-product-model.md)
- Scope: [Personal 2.0](personal-2.0-scope.md)
- Requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
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

Every journey separates candidate explanation from daemon authority. The
Personal Assistant, manager, Member, Pi, DSH, UI, and connector may propose,
observe, or execute admitted bounded work. Only the daemon authorizes and
accepts; independent verification, not Agent self-report, closes work.

## 1. Create and activate the first Project

1. From Today or Projects empty state, choose **Create Project**.
2. A resumable conversation captures the business situation. The Personal
   Assistant conducts broad, high-quality source-backed research by default
   without asking for each ordinary web read. Non-secret Project context may be
   used; raw credentials and third-party data the Owner cannot disclose may
   not.
3. The Owner and Assistant refine charter, main -> phase/quarter when useful
   -> month -> week -> day/Task goals, output contracts, team Runtime
   definitions, first plan/work cycle, explicit Provider/model selection,
   capabilities, permissions/HITL, cost warnings, triggers, and
   human-intervention points.
4. Skill candidates receive source/prompt-injection review before automatic
   installation. MCP candidates receive additional executable/network/secret
   review and an exact Owner grant before activation.
5. External research text remains untrusted and cannot execute, install, or
   expand permission. Personal simulates one operating cycle and identifies any required Owner
   setup action, such as creating an account or entering a credential.
6. The daemon produces a structured launch diff with sources, unknowns, risk,
   requested capability, and cost basis.
7. The Owner edits, rejects, or confirms the exact revision.
8. Creation ends on the active Project canvas, group conversation, and durable
   receipt.

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

The Project is not active before confirmation. A research result or assistant
message cannot activate it.

## 2. Use Today without a KPI wall

The returning Owner opens Today as one decision packet plus four exception
swimlanes:

The packet states consequence, reversibility, alternatives, kernel truth, and
why option A is first. Provenance chips are Observed / Proposed / Governed /
Verified.

Then the four swimlanes:

1. **Needs you** — the consequential Owner decision (for example inspect
   Package A; planned is not published);
2. **Can continue** — work that proceeds without the Owner now;
3. **Unknown** — missing actuals, Effects, or verification; actual unknown is
   never shown as zero;
4. **Missed** — offline or coalesced Routine facts.

Member activity is a Working / Queued / Waiting table; queued is not running.
Project health uses goals, current plan, responsibility, state, freshness, and
evidence—not decorative metrics. A Project card opens its operating-report
template while preserving the Today filter and scroll position. The three
columns stay locked; a narrow canvas scrolls horizontally.

## 3. Use the Project group and flexible operating canvas

A Project opens to its stable operating-report template in the center column
and the Owner/manager/Members group conversation in the right column. The
report answers goal hierarchy, current phase, manager summary, today's work,
Member state, attention/approval items, latest openable artifacts/evidence,
spend basis, and freshness. Package inspect shows a thread preview plus
acceptance; planned is not published. Publish preview is the full AUTONOMY
packet on the canvas; there is no Confirm in chat. The Project Manager may
version this Project's template. After the report, the X loop is available when
that Project needs it. HITL is announced in chat and linked to the center
preview; chat has no Approve control and no “Don’t ask again” grant.

The Owner may `@manager` for progress or delegation and `@member` to ask or
temporarily redirect goal/path inside an approved boundary. `@` inserts only
into the unsent draft. The manager speaks
by default. A Member speaks proactively only when mentioned, submitting a
deliverable, handing off, blocked, or requesting a decision. A work-changing
message becomes a formal Task or revision before it has authority.

For a new question, the Assistant/manager interprets intent, reads real Project
results, and composes a temporary canvas from approved goal, artifact,
evidence, timeline, organization, decision, and metric components. It is not
saved unless pinned or made a template. Generated code/`eval`, invented data,
and hiding goal/acceptance state, failure/not-run work, Owner decisions, source,
or freshness are forbidden.

The Owner may drill down through Goal -> Plan revision -> Routine/Task ->
Attempt -> Effect/Evidence. Ordinary process traces remain collapsed.

A manager can autonomously reorder approved Tasks or change a Member's bounded
responsibility. A primary-goal, team, Provider/model, Tool/MCP, permission, or
external-rule change creates a revision candidate and daemon preview. Rejection
keeps the draft and current plan; confirmation produces a new revision and
receipt without erasing prior plans or Attempts.

## 4. Add a Role and Project Member

1. From the Project canvas, open the contextual Member roster.
2. The Owner asks the Personal Assistant for a new business responsibility.
3. The Assistant performs broad source-backed research and proposes a reusable,
   versioned Role Runtime Template: purpose, responsibility, prohibitions,
   input/output/success and handoff contracts, instructions, Skills, Tools,
   MCP needs, work cycle/reflection, model needs, Context/Memory, permissions,
   and escalation.
4. Project-specific creation adds responsibility/subgoal, explicit
   Provider/model choice, grants, cost policy, Memory scope, and permissions.
5. The daemon previews the exact Template and Member Runtime revisions; the
   Owner edits, narrows, rejects, or confirms.
6. The Member card shows responsibility, current work, next action, latest
   accepted deliverable, block/decision, actual/estimated/unknown cost, and
   freshness. DSH detail is hidden unless recovery requires it.
7. A first Task starts a disposable Agent process/Attempt from the exact Member
   revision. Process exit does not delete the Member, Conversation, Memory, or
   evidence.

## 5. Import knowledge and retrieve bounded context

1. Knowledge offers Owner-shared knowledge or a selected Project Vault.
2. The Owner imports files, directories, links, images, or video metadata.
3. Personal copies permitted source material into the selected archive,
   preserves source/provenance, detects duplicates and credentials, and shows
   parsing/OCR/index progress.
4. Parse failure preserves the original and permits retry, exclusion, or manual
   classification.
5. Ordinary knowledge edits reindex. A goal/role/permission/workflow-like edit
   becomes a candidate and cannot silently mutate authority.
6. Retrieval shows scope, source, freshness, redaction, and a **Why this
   fragment** table for each selected excerpt. Memory is not silent auto-ingest.

Conversation archives can participate, but Personal injects only relevant,
bounded, redacted, provenance-bearing, untrusted observations in this order:
current Task contract -> fixed decisions -> relevant source/artifact excerpts
-> provenance-linked summaries -> older narrative. Over-limit reduction removes
older narrative first. Full raw sources remain; a summary does not prove
completion or enter Memory automatically.

Ordinary chat is not Memory. Explicit instructions become revisions; “remember”
or stable verified facts produce candidates. Semantic Memory requires
admission and supports inspect, correction, promotion, and forget. Cross-Project
promotion requires Owner confirmation. Accept/reject/edit/rate actions become
Project feedback evidence; stable repeated preference may produce a versioned
Member/global Role proposal, never a silent one-event change.

## 6. Resolve contextual attention, approval, and recovery

Approvals, requested input, permissions, execution blocks, unknown Effects,
missed runs, and cost warnings appear in Today and the affected Project canvas.
They do not require a permanent first-level Inbox destination.

An item answers: Project/Member, reason, consequence, affected targets,
reversibility, source/freshness, deadline or age, and next safe actions.
Approval shows the exact structured diff on the center canvas. Chat may
announce the HITL pause and link to that preview; it cannot Approve, publish,
or install, and it offers no “Don’t ask again” grant. Narrow/deny/edit are
first-class choices. Unknown external outcomes expose reconcile status; retry
is absent until daemon policy says it is safe. Working is not completion.

After an error, typed input and draft work remain. A successful action ends in
a receipt linked to the affected Project object.

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

Settings > Model Connections offers mainstream Provider quick templates and an
advanced custom URL/compatibility-mode/key/model flow. Raw keys use a one-way
daemon handoff into an approved SecretStore and are never echoed into DOM, URL,
chat, Vault, configuration, runtime environment, or evidence.

Creating every Member requires an explicit Provider/model choice. The
Assistant may recommend but cannot bind or rebind silently. Cost appears as
source-labelled actual, estimated, or unknown; unknown is never zero. Personal
warns but does not automatically stop work at a product budget threshold.
Provider quota or unavailability may still produce an external failure.

DSH and Pi are absent from ordinary navigation. Fault recovery or advanced
diagnostics may show exact version, provenance, health, qualification,
update/rollback, affected Members/Tasks, and whether restart is required.
There is no native DSH/Pi UI, native conversation synchronization, Agent store,
or Harness selector.

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

## 10. Run the X/Twitter content-operation acceptance scenario

1. Owner and manager confirm account position, audience, allowed content
   forms, cadence, metrics, source-rights rules, cost visibility, and
   external-action policy.
2. Members research with provenance, plan, draft original content, and
   produce a publication package.
3. Only Owner-owned, licensed, open-license, or public-domain material may be
   copied. Other sources support analysis, attribution, and new creation.
4. Publishing through an independently qualified connector follows the
   approved policy. Boundary drift or high-risk exceptions receive an exact
   target/content/cost preview and Owner approval.
5. Dispatch is an Intent/Effect operation. CAPTCHA, anti-abuse, account lock,
   or UI drift fails closed; no fingerprint or policy evasion occurs.
6. Receipt and later metric readback remain separate. Completion follows the
   scenario oracle and independent evidence, not the Member's claim. Comment
   reply suggestions follow the same applicable external-action policy.
7. The manager reflects on evidence and feedback, adjusts the next plan, and
   exposes whether a later comparable cycle reduces the prior gap.

The Project model remains generic; this scenario is the first important
acceptance case, not a hard-coded success path or business-result promise.

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
[**Owner-approved interaction baseline (2026-08-28)**](../../../clients/docs/design/opc-2.0/personal-20-ai-ceo-e2e-optimized-v2.canvas.tsx)
is the accepted competitive-informed V2 overwrite (not a v3, not the
pre-overwrite overlay conversation). Source/static checks and Owner approval
are not human
usability, accessibility, backend, Gate, or acceptance evidence. Canvas
runtime/render, NVDA, host-theme contrast, and 200% real layout remain
`not-run`. Formal acceptance requires a future reconciled fixed denominator,
supported exact revision, and honest pass/fail/partial/not-run accounting. The
2026-08-27 Phase 11 task IDs and 15-scene denominator are pending formal-plan
reconciliation and do not define current product semantics.
