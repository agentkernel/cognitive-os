# 03 — Today, Projects, and Project briefing

## Today: attention, not analytics

Today answers:

1. what is planned today;
2. what needs the Owner;
3. which Project/employee changed state;
4. what was missed, blocked, stale, or unknown;
5. which result was independently verified;
6. what it cost or why cost is unavailable;
7. what action is next.

It uses a priority narrative and compact rows. Generic KPI cards, decorative
charts, welcome heroes, and undifferentiated activity are rejected. A metric
appears only when it changes a decision and declares source/denominator.

## Project cards

Each card shows:

- Project goal and current phase;
- current manager;
- health reason in plain language;
- next planned work;
- Inbox count/reason;
- latest verified result;
- actual/unknown spend basis;
- freshness/offline/missed indicator.

Card selection opens the briefing while preserving Today context.

## Projects master/detail

The list supports search, status, next-decision, manager, and recency filters.
The detail defaults to the Project briefing:

| Briefing section | Required answer |
|---|---|
| Goal | what outcome and metric were confirmed |
| Manager brief | what changed, why, and what is uncertain |
| Today | planned/active/queued/missed work |
| Team | responsibility, state, next action |
| Needs Owner | approvals, input, permissions, budget |
| Results | artifacts, external receipts, verified evidence |
| Cost | Project/member/Task basis and unknowns |
| Timeline | current plan revision and key milestones |

Advanced drilldown reveals Goal/Plan/Routine/Task/Attempt/Effect/Evidence. A
Task row preserves every Attempt; retry/fork never replaces the failed one.
Agent self-report and process exit are labelled Observed, not Verified.

## Manager revision

Changing approved subgoal, Task order/frequency, or bounded responsibility may
run under manager policy. Primary goal, team, budget, Provider, Tool,
permission, or external rule opens a structured diff:

- current vs proposed revision;
- source/reason and uncertainty;
- affected employees/Tasks/Routines;
- permission/budget/external consequences;
- reversibility and rollback/compensation truth;
- confirm, edit, narrow, or reject.

A stale revision disables confirmation and offers re-preview with edits
preserved.

## States

Today and Projects cover:

- empty: create/import sample Project, with no fake activity;
- loading: stable shell and labelled sources;
- partial/stale: last-known safe facts and missing coverage;
- permission: exact Project/folder/Provider scope;
- error/unknown: preserved list/context and recovery;
- offline/missed: host/dependency state and catch-up decisions;
- success: receipt/evidence and next action;
- archived: read/export/restore/delete paths.

## Prototype scenes

The Canvas prototype must switch among:

1. Today with missed work and a budget approval;
2. active Project briefing with one verified result and one unknown Effect;
3. Project list empty/loading/partial;
4. archived Project;
5. Task/Attempt detail labelled `Requires-backend`.
