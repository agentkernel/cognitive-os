# CognitiveOS Personal 2.0 user journeys

- Status: adopted target journeys; not usability or implementation evidence
- Product intent: [Product design](product-design.md)
- Product model: [OPC product model](opc-product-model.md)
- Scope: [Personal 2.0](personal-2.0-scope.md)

Every journey separates candidate explanation from daemon authority. The
Personal Assistant, manager, employee, Pi, DSH, UI, and connector may propose,
observe, or execute admitted bounded work. Only the daemon authorizes and
accepts; independent verification, not Agent self-report, closes work.

## 1. Create and activate the first Project

1. From Today or Projects empty state, choose **Create Project**.
2. A resumable conversation captures the business situation and optional
   source-backed research.
3. The Owner and Personal Assistant refine charter, goals, metrics, team,
   first plan, permissions, budgets, and triggers.
4. The daemon produces a structured diff with sources, unknowns, risk,
   requested capability, and cost boundary.
5. The Owner edits, rejects, or confirms the exact revision.
6. Creation ends on the active Project briefing and durable receipt.

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

The returning Owner opens Today and sees a short priority narrative:

1. what is planned today;
2. what needs the Owner;
3. which Project or employee changed state;
4. which work was missed or is stale;
5. the latest verified result and actual/unknown cost;
6. one next action per item.

Project health uses goals, current plan, responsibility, state, freshness, and
evidence—not decorative metrics. A Project card opens its briefing while
preserving the Today filter and scroll position.

## 3. Read a Project briefing and revise its plan

The briefing answers goal, current phase, manager summary, today's work, team
state, Inbox items, latest artifacts/evidence, and spend basis. The Owner may
talk to the manager or inspect Goal -> Plan revision -> Routine/Task ->
Attempt -> Effect/Evidence.

A manager can autonomously reorder approved Tasks or change a member's bounded
responsibility. A primary-goal, team, budget, Provider, tool, permission, or
external-rule change creates a revision candidate and daemon preview. Rejection
keeps the draft and current plan; confirmation produces a new revision and
receipt without erasing prior plans or Attempts.

## 4. Add a role and converse with an employee

1. Team shows the base Project Manager and current employees.
2. The Owner asks the Personal Assistant for a new business role.
3. The assistant proposes a Role Blueprint and Project Assignment with
   capability, permission, budget, and collaboration expectations.
4. The daemon previews creation; the Owner confirms.
5. The employee card shows responsibility, state, next work, latest verified
   result, cost, memory scope, and DSH runtime health.
6. Opening its Conversation changes the active composer from Personal
   Assistant to employee; the assistant draft is retained.

Only one composer may submit at a time. Switching contexts preserves both
drafts and visibly names the active recipient. Conversation text does not
change Project authority until admitted.

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
6. Retrieval shows scope, source, freshness, redaction, and why a fragment was
   selected.

Conversation archives can participate, but Personal injects only relevant,
bounded, redacted, provenance-bearing, untrusted observations. Semantic Memory
requires admission and supports view, correction, and forget.

## 6. Approve or recover work from Inbox

Inbox is a priority queue for approvals, requested input, permissions,
execution blocks, unknown Effects, missed runs, and budget warnings.

An item answers: Project/employee, reason, consequence, affected targets,
reversibility, source/freshness, deadline or age, and next safe actions.
Approval shows the exact structured diff and does not rely on chat alone.
Narrow/deny/edit are first-class choices. Unknown external outcomes expose
reconcile status; retry is absent until daemon policy says it is safe.

After an error, typed input and draft work remain. A successful action ends in
a receipt linked to the affected Project object.

## 7. Schedule, miss, and resume a Routine

1. A Routine revision defines bounded work, trigger, no-overlap policy,
   budget, and risk class.
2. Manual, schedule, or qualified platform event creates a run request.
3. If one is active, only the latest pending occurrence is queued; superseded
   occurrences remain in the ledger.
4. Sleep/offline pauses dispatch and produces missed facts.
5. On return, Personal shows what was missed and why.
6. Low-risk internal work may resume under policy; publishing,
   communication, spending, permission expansion, or other consequential work
   returns to Inbox for fresh review.

Closing the window asks **Continue eligible work in background** or **Pause**.
The choice is explicit and remembered only as a policy revision. Host shutdown
never implies 24/7 work.

## 8. Inspect Installed Agents, Providers, and usage

Settings > Installed Agents shows DSH as **Preinstalled / Managed by Personal**
with source, exact artifact, version, health, capability boundary, sandbox
status, update, and rollback. Everyday employee pages show DSH only as the
execution engine. There is no native DSH UI or conversation synchronization.

Settings > Providers keeps subscription, account/authentication, API billing/
quota, model, binding, budget, and usage separate. Effective binding is
global -> Project -> employee -> Task. Pi and DSH receive Provider traffic
through the daemon proxy and never receive raw credentials.

Unknown quota or cost is not zero. An update, rotation, or binding change shows
affected Projects/employees/Tasks and whether a runtime restart is required.

## 9. Run the X/Twitter content-operation acceptance scenario

1. Owner and manager confirm account position, audience, allowed content
   forms, cadence, metrics, source-rights rules, and budget.
2. Employees research with provenance, plan, draft original content, and
   produce a publication package.
3. Only Owner-owned, licensed, open-license, or public-domain material may be
   copied. Other sources support analysis, attribution, and new creation.
4. Publishing through an independently qualified connector receives an exact
   target/content/cost preview and Owner approval.
5. Dispatch is an Intent/Effect operation. CAPTCHA, anti-abuse, account lock,
   or UI drift fails closed; no fingerprint or policy evasion occurs.
6. Receipt and later metric readback remain separate. Completion follows the
   scenario oracle and independent evidence, not the employee's claim.

The Project model remains generic; this scenario is the first important
acceptance case, not a hard-coded success path or business-result promise.

## 10. Archive, restore, export, or delete a Project

Archive stops new triggers and preserves Project history. A same-disk local
restore point can reverse eligible local changes but is labelled **not a
disaster backup**. Manual export identifies included data and excludes secrets
by default.

Permanent deletion is a separate impact-preview action after archive. It names
Project authority, files, Vault, conversations, Memory, artifacts, restore
points, bindings, pending Effects, and what cannot be recovered. A second
confirmation is required.

## 11. Surface state contract

Every Today, Projects, Team, Knowledge, Inbox, Settings, setup, Conversation,
and inspector surface defines all applicable states:

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

## 12. Evidence boundary

These journeys are specifications. A prototype walkthrough may show coverage,
but it is not human usability evidence or backend execution. Formal acceptance
requires the fixed Phase 11 denominator, supported exact revision, and honest
pass/fail/partial/not-run accounting.
