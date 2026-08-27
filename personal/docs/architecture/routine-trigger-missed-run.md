# Routine, Trigger, occurrence, and missed-run architecture

- Status: Personal 2.0 target; `Requires-backend`
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Product: [Long-running operations](../product/long-running-operations.md)

## 1. Domain separation

| Object | Owner | Invariant |
|---|---|---|
| Routine revision | daemon Project/execution domain | immutable definition; not one run |
| Trigger | daemon scheduler authority | exact manual/schedule/qualified-event source |
| Occurrence | daemon run ledger | one requested instance and disposition |
| Task/Attempt | existing/future work authority | bounded execution and preserved retry branch |
| Missed/coalesced fact | daemon run ledger | never success or invisible deletion |

Windows Task Scheduler, a connector, manager, Pi, DSH, or a workflow engine can
signal a candidate occurrence but cannot own this ledger or eligibility.

## 2. Trigger admission

Manual, schedule, and qualified platform-event triggers bind Project, Routine
revision, source/event identity, timezone/clock basis, deduplication, risk,
permission, budget, and catch-up policy. Untrusted event payloads are Context
candidates only and cannot change the Routine or expand capability.

## 3. No overlap and queue latest

For one Routine:

1. at most one active occurrence;
2. while active, at most one pending occurrence;
3. a newer pending occurrence supersedes the older pending one;
4. superseded occurrences retain `coalesced` facts and reasons;
5. eligibility is rechecked when pending work starts;
6. no queue item inherits stale policy, Provider, Context, or budget.

Temporal's overlap/catch-up vocabulary is informative only. Personal keeps its
own scheduler and ledger; no Temporal service or second recovery authority is
introduced.

## 4. Attempt-engine boundary

An optional Attempt engine port may offer start/resume/interrupt/checkpoint/
replay/subgraph mechanics. The daemon still owns Task/Attempt identity,
scheduler lease, Intent/Effect, budget, fencing, Context, evidence, and
acceptance. Engine checkpoint/pending writes are recovery inputs—not proof an
external action happened or work completed.

LangGraph may be conditionally evaluated behind this port with strict
serialization, secret-shape rejection, crash/replay, and side-effect negatives.

## 5. Offline/missed recovery

The daemon records missed causes and requested times. Resume:

1. establish fresh epoch and clock facts;
2. reconcile Effects;
3. reload current Routine/Project/permission/budget/Provider;
4. coalesce according to queue-latest;
5. classify low-risk internal versus consequential work;
6. admit automatic catch-up only inside current policy;
7. send publishing/communication/spend/delete/expansion to Inbox.

An occurrence is not silently dropped or marked complete because its trigger
time passed.

## 6. Manager revisions and reflection

The manager may propose/perform within-envelope frequency or responsibility
changes under current policy. Goal, team, budget, Provider, Tool, permission,
or external-rule changes require Owner-confirmed revisions. Key-result and
daily/weekly reflection are candidate producers only.

## 7. Observation and controls

The Control Plane reads active/latest-pending/missed/coalesced facts with
source, denominator, reason, responsible employee, Task/Attempt, budget, and
next action. Pause/stop/retry/resume exist only when typed capability is
available. Detach affects observation only.

## 8. Non-claims

No Routine/Trigger/occurrence schema, scheduler behavior, Windows background,
queue, catch-up, Attempt engine, LangGraph dependency, reflection admission, or
control is implemented here. No reliability, timing, 24/7, Gate, release, or
Profile claim follows.
