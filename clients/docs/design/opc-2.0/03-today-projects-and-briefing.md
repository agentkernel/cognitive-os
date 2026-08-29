# 03 — Today, Projects, and the operating canvas

- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Status: current interaction prototype is owner-approved v9 (2026-08-30); v8 is the prior approved baseline (not overwritten); v5–v7 and archived pre-v5 / V2 are historical chrome only
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v9**](personal-20-opc-e2e-optimized-v9.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](history/2026-08-28-pre-subtraction/README.md)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## Priority rule

Every surface orders information as:

1. goal, expected result, due/openable deliverable, acceptance, and evidence;
2. operational state, exception, missed work, and Owner decision;
3. configuration, runtime, and diagnostics.

## Today: decision packet plus live-project run overview

Today (after ⑤ 验收) opens with **three default blocks**. It is not a KPI card
wall and not four exception swimlanes.

1. **Decision packet** — the one consequential Owner decision. This is the only
   primary CTA. If nothing is pending, **collapse the packet** and keep the
   run overview.
2. **Live-project run overview** — one row per live Project (status, today's
   completed-run count, current stage, duration) plus created / live / blocked
   counts and a today / week / month toggle. Click a live-project row to open
   that Project’s 运行. Blocked counts are clickable.
3. **Assistant** — questions about run data; chat cannot approve.

The decision packet states:

- consequence of acting or waiting;
- reversibility / compensation;
- alternatives;
- kernel truth (what the daemon already persisted);
- **why option A is first**.

Provenance chips are Observed / Proposed / Governed / Verified. They describe
authority relationship, not confidence.

Swimlane semantics (Needs you / Can continue / Unknown / Missed) may merge
into overview rows. They are **not** default Today blocks.

Member activity is Working / Queued / Waiting; queued is not running. It is a
table, not three identical staff cards. Generic KPI tiles, decorative charts,
welcome heroes, raw execution feeds, and false “all clear” states are rejected.
A metric appears only with source, freshness, denominator, and decision
relevance. Actual unknown is never shown as zero.

## Project cards and list

Each Project card leads with main/period goal and next expected deliverable,
then shows current manager, state reason, latest accepted result, exception or
decision, actual/estimated/unknown cost basis, and freshness.

Current chrome (v9): the list is one row per Project with a single 「打开」
control plus text links to 成员 / 运行 / 产出. Do not restore four parallel
「查看」 buttons. Selection opens 项目详情 while preserving Today filters
and scroll.

Projects supports search plus status, due result, Owner-decision, manager, and
recency filters. Empty, partial, archived, and unavailable results remain
distinct.

## Current chrome: four Project work scenes (v9)

A live Project is four submenus, not a single operating-report landing:

| Submenu | Job |
|---|---|
| 详情 | Read-only charter: name, goal, cycle, read-only process axis, destinations to members / runs / outputs |
| 成员 | Select-then-configure roster (see 05). Add uses this Project’s real members |
| 运行 | Current-stage workface. 「验收，回 Today」 only on the last ring |
| 产出 | Select-then-view artifacts. Unselected is empty |

There is no visible CEO six-step top rail. Chat cannot approve or 验收.

The numbered operating-report regions below remain the **template contract**
for what a Project must be able to answer. They are not the v9 default
landing layout (详情 / 成员 / 运行 / 产出).

## Stable Project operating report

A Project opens to a versioned operating-report canvas in the center column,
with the group conversation always in the right column. The first canvas is
the system-default routine report; the manager may version that Project's
template:

| Region | Required answer |
|---|---|
| Goals and outputs | main/period goal, expected results, due deliverables, success criteria |
| Manager summary | what changed, why, what remains uncertain, next adjustment |
| Current work | Working / Queued / Waiting Members and Tasks; queued is not running |
| Results | openable artifacts, acceptance, evidence, receipt, source, freshness; Package A is a thread preview plus acceptance, planned not published |
| Needs Owner | exact decision/input/permission, consequence, deadline, choices |
| Members | responsibility, current work, next action, latest accepted result |
| Cost | actual/estimated/unknown, source, period, warning |
| Plan and reflection | current revision, milestones, comparable gap, rollback option |

The manager may version this Project template. A template cannot hide a failed
or not-run item, Owner decision, stale source, or unknown outcome.

## Temporary ad-hoc canvases

For an unplanned question, the Assistant or manager:

1. interprets the question and identifies authoritative Project sources;
2. reads real goals, artifacts, evidence, decisions, timeline, organization,
   and cost readings;
3. composes approved typed components;
4. labels source, freshness, omissions, conflicts, and unknowns;
5. returns a temporary canvas.

The canvas is not saved by default. The Owner may pin it or save it as a
Project template. It cannot execute generated code or `eval`, perform
unapproved fetches, invent values, or become authority.

## Work and manager revision drilldown

Advanced drilldown reveals Goal -> Plan revision -> Routine/Task -> Attempt ->
Intent/Effect/Artifact/Evidence. Every Attempt is retained; retry or restart
creates a new one. Agent/manager prose, Provider success, Tool success, engine
checkpoint, and process exit remain observations.

Inside the autonomy envelope, the manager may adjust subgoals, Tasks, order,
frequency, and bounded Member responsibility. A primary-goal, team,
Provider/model, Tool/MCP, permission, global Role, or external-action-rule
change opens a daemon-issued diff with affected work, consequences,
reversibility, and edit/narrow/deny/confirm. A stale preview must be regenerated.

## Owner-approved prototype source coverage

Current chrome is `personal-20-opc-e2e-optimized-v9` (2026-08-30). The
paragraphs below retain **archived V2 / pre-subtraction** scene coverage for
provenance. They do not define current create, Today, or live-Project chrome
(no visible CEO rail; no X/Twitter P0 hero; four swimlanes are not default
Today blocks; Project work is four submenus as above).

The owner-accepted competitive-informed v2 source is retained as historical
coverage for:

- visible CEO loop: Ingest → Decide → Authorize → Execute → Verify → Report;
- Today decision packet plus four exception swimlanes (Needs you / Can
  continue / Unknown / Missed); estimated cost plus actual unknown never shown
  as zero; Working / Queued / Waiting as a table, not a staff-card mosaic;
- Project stable operating report first, then the X loop;
- Project publish preview as the full AUTONOMY packet on the canvas; no
  Confirm in chat; Package A thread preview plus acceptance; planned is not
  published;
- right-column group conversation with HITL announced and linked to the center
  preview; no chat Approve; no “Don’t ask again”;
- temporary ad-hoc canvas with sources plus pin/save-template options;
- Project empty/loading/partial/stale/archived states;
- Task/Attempt detail labelled `Requires-backend` where no backend exists;
- cost warning where unknown remains unknown and work is not auto-stopped.
