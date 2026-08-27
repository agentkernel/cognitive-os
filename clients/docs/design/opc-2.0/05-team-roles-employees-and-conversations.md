# 05 — Team, roles, employees, and conversations

## Team model

Team is a Project-scoped roster, not human account administration. Every
Project has exactly one current manager. The base Project Manager Blueprint is
the only built-in role; the Personal Assistant proposes additional roles from
the charter and plan.

`Blueprint -> Assignment -> Employee -> Runtime -> Conversation` remains
visible in the inspector and never collapses into one Agent card.

## Cards and details

Every manager/employee card answers:

- Project goal and assigned responsibility;
- current state and why;
- next planned action;
- latest verified result and evidence link;
- actual/unknown spend basis;
- current Runtime health/qualification;
- Conversation and Memory scope;
- blocked/permission/offline/missed state.

The employee detail has Work, Conversation, Memory, Runtime, and History tabs.
The Project/employee identity stays visible across tabs.

## Role creation and upgrade

1. Owner asks the Personal Assistant for a role in business language.
2. Assistant proposes purpose, responsibilities, capability needs, work
   methods, handoff expectations, risk, permission, and budget.
3. Daemon preview shows Blueprint revision plus Project Assignment and employee
   impact.
4. Owner edits/narrows/confirms.
5. Receipt links the new employee and first bounded Task.

Blueprint upgrade is versioned and opt-in per Project. Existing employees and
history remain pinned until an Assignment revision is confirmed. Project
Manager safety/governance obligations cannot be removed by specialization.

## Manager autonomy

Inside the approved envelope the manager may adjust subgoals, Task
decomposition/order/frequency, and member responsibility. Primary goal, team,
budget, Provider, Tool, permission, or external-action rule changes are
proposal-only and go to Inbox.

Manager or employee disagreement remains a source-labelled candidate.
Collaboration is expressed through daemon-owned Tasks, artifacts, and explicit
handoffs. Free group chat does not transfer authority.

## Conversation workbench

The right rail or employee detail hosts a Personal-owned Conversation:

- recipient, Project, and employee scope are always visible;
- retrieved fragments show source/scope/freshness and untrusted status;
- DSH output is candidate/observation;
- proposed Task/plan changes open the central daemon preview;
- receipts/evidence link back without exposing secrets;
- archive, correction, and forget controls follow their actual backend status.

### Single active composer

Exactly one composer can submit. Choosing Personal Assistant, manager, or
employee changes the labelled recipient. Each keeps a distinct draft.
Switching preserves drafts, returns focus predictably, and never sends or
merges text.

Inbox approval has no independent composer. It may show the originating
conversation, but confirm/edit/narrow/reject act on the structured preview.

## DSH and Pi presentation

Employee pages say **Execution engine: DSH** with status and a link to advanced
Installed Agent diagnostics. They do not expose a native DSH UI or native
conversation. DSH artifact/process/session is separate from employee identity.

Personal Assistant pages do not show Pi as an installed Agent. Advanced
diagnostics may identify the exact Pi engine and health while preserving its
candidate-only, no-secret, no-memory-ownership boundary.

## States

Team and Conversation cover empty roster, role generation, loading, partial
employee facts, stale Runtime, permission blocked, offline, missed work,
unknown outcome, conversation error with retained draft, archived employee,
runtime update/rollback, and success receipt.

## Requires-backend

Blueprint/Assignment/Employee authority, role generation, manager autonomy
policy, Personal-owned Conversation/archive, single-composer persistence,
managed DSH runtime, and Pi Assistant composition are target-only. No element
may simulate a backend mutation.
