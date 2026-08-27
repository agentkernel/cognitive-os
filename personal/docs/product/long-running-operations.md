# Long-running Projects, Routines, triggers, and missed work

- Status: adopted Personal 2.0 product target
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Architecture:
  [Routine, Trigger, and missed-run](../architecture/routine-trigger-missed-run.md)
- UX surface: [Inbox and recovery journey](user-journeys.md#6-approve-or-recover-work-from-inbox)

## 1. Product concepts

| Concept | Meaning | Not equivalent to |
|---|---|---|
| Routine | revisioned recurring work definition inside one Project | one execution, cron row, Loop, or Agent prompt |
| Trigger | admitted cause that requests one Routine occurrence | authority to run or proof it ran |
| Occurrence | one requested time/event/manual instance | Task completion |
| Task / Attempt | bounded governed work and one preserved execution try | Routine definition |
| Missed-run fact | occurrence that could not start while host/dependency was unavailable | failure, retry, or success |

The daemon owns these facts. DSH, Pi, a Project Manager, Windows Task
Scheduler, or an external platform event may signal or execute bounded work but
cannot become the scheduling authority.

## 2. Trigger classes

Personal 2.0 allows:

1. **Manual** — Owner or admitted manager request;
2. **Schedule** — daemon-owned time policy with timezone and clock basis;
3. **Qualified platform event** — exact connector/event identity after
   independent qualification.

An event adapter cannot silently broaden scope or create a trigger from
untrusted content. Trigger admission identifies Project, Routine revision,
event/source, deduplication/idempotency, risk class, permission, and budget.

## 3. No overlap and queue latest

The same Routine does not overlap by default:

- if no occurrence is active, the next eligible occurrence may start;
- if one is active, at most the latest pending occurrence is retained;
- an older pending occurrence superseded by a newer one is recorded as
  coalesced/skipped with reason and timestamps;
- different Routines still obey shared Project/member/Provider budgets and
  scheduler fencing;
- no queue policy turns a dropped occurrence into success.

The Product shows the active occurrence, latest queued occurrence, coalesced
count with declared denominator, and one next action.

## 4. Offline, sleep, and missed work

Windows sleep, shutdown, daemon stop, network loss, Provider outage, connector
unavailability, or locked SecretStore can prevent dispatch. Personal records
the applicable state rather than simulating background cloud execution.

On resume, Inbox groups:

- missed occurrences and their source/time;
- work safe to resume automatically;
- work needing fresh Context or Provider recovery;
- consequential work needing Owner review;
- stale/unknown external Effects that must reconcile before retry.

Low-risk internal work may resume inside the unchanged policy. Publishing,
communication, spending, deletion, permission expansion, or a changed
external target requires a new preview or renewed approval.

## 5. Closing the window

When eligible work is running or queued, closing the Control Plane asks:

- **Continue eligible work in background**; or
- **Pause after the current safe boundary**.

The dialog names affected Projects/Routines, work that cannot be stopped
instantly, offline limitations, and how to reopen status. It does not promise
execution after host shutdown. If a backend cannot yet honor a choice, the
prototype labels it `Requires-backend`.

## 6. Manager autonomy and reflections

Within an approved Project boundary, the manager may change a Routine's
subgoal, Task decomposition, order, frequency, or member responsibility if the
total goal, team, budget, Provider, tools, permissions, and external-action
rules remain unchanged.

A change outside that envelope creates a plan/Routine revision candidate,
structured diff, and Owner confirmation. Key-result and daily/weekly
reflections may propose such revisions. Agent or manager self-report does not
admit a revision or mark work complete.

## 7. Progress and controls

A long-running surface shows:

- Project, Routine revision, Task, Attempt, and responsible employee;
- current step and latest durable fact—not hidden "thinking";
- queued/missed/coalesced occurrences;
- artifacts, Effects, evidence, Provider usage/cost basis;
- available pause/stop/retry/resume/reconcile actions;
- unsupported control reasons;
- final receipt separating completed, skipped, failed, unknown, and not-run
  work.

Every control is capability-backed. Detach affects observation only. Process
exit is not cancel, recovery, or completion. Engine checkpoints are recovery
inputs, not authority or external-effect receipts.

## 8. Failure and recovery

| State | Required recovery |
|---|---|
| queued | source, position/basis, no-overlap reason |
| running | current durable step and real controls |
| waiting-owner | exact decision, consequence, preserved work |
| paused-offline | unavailable dependency and reconnect behavior |
| missed | occurrence/time/reason and risk-based catch-up choice |
| failed-recoverable | retained Attempt, safe retry/resume basis |
| outcome-unknown | Effect reconciliation; no blind redispatch |
| failed-terminal | evidence, export/support, new revision path |
| complete | independent verification, receipt, next occurrence |

## 9. Non-claims

This document does not implement a scheduler, Windows background service,
Routine/Trigger authority, queue, missed-run ledger, reflection admission,
controls, or connector. It makes no 24/7, reliability, timing, Gate, release,
Profile, or business-outcome claim.
