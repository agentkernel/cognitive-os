# Long-running Projects, Routines, triggers, and missed work

- Status: adopted Personal 2.0 product target
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v9**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](../../../clients/docs/design/opc-2.0/history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Prototype identity: owner-approved 2026-08-30 current chrome is
  personal-20-opc-e2e-optimized-v9. v8 is the prior approved baseline (not overwritten). Archived V2 is not current chrome. Canvas-only HITL and daemon authority path remain.
- Existing architecture input (pending reconciliation):
  [Routine, Trigger, and missed-run](../architecture/routine-trigger-missed-run.md)
- UX surface:
  [Contextual attention and recovery journey](user-journeys.md#6-resolve-contextual-attention-approval-and-recovery)

## 1. Product concepts

| Concept | Meaning | Not equivalent to |
|---|---|---|
| Routine | revisioned recurring work definition inside one Project | one execution, cron row, Loop, or Agent prompt |
| Trigger | admitted cause that requests one Routine occurrence | authority to run or proof it ran |
| Occurrence | one requested time/event/manual instance | Task completion |
| Working | in-progress observation of a current occurrence | completion, verification, or success |
| Queued | latest eligible occurrence waiting to start | a running process |
| Waiting | blocked on evidence, Owner, or handoff | a running process |
| Task / Attempt | bounded governed work and one preserved execution try | Routine definition |
| Missed-run fact | occurrence that could not start while host/dependency was unavailable | failure, retry, or success |

The daemon owns these facts. DSH, Pi, a Project Manager, Windows Task
Scheduler, or an external platform event may signal or execute bounded work but
cannot become the scheduling authority.

The default Operations working view is the daemon authority path:

**Candidate → Intent persisted → Fence → Execute → Independent verify →
Receipt.**

Working is in-progress observation, not completion.

## 2. Trigger classes

Personal 2.0 allows:

1. **Manual** — Owner or admitted manager request;
2. **Schedule** — daemon-owned time policy with timezone and clock basis;
3. **Accepted artifact** — a named artifact reaching an admitted acceptance
   state;
4. **Project state** — a testable governed Project transition;
5. **Qualified external event** — exact connector/event identity after
   independent qualification;
6. **Testable data condition** — a deterministic condition with declared
   source, freshness, and evaluation basis.

An event adapter cannot silently broaden scope or create a trigger from
untrusted content. Trigger admission identifies Project, Routine revision,
event/source, deduplication/idempotency, risk class, permission, and declared
cost/limit policy.

Product cost visibility and warnings do not create an automatic Personal 2.0
budget-threshold stop. Provider quota, unavailable credentials, or resource
unavailability may still prevent an external call.

## 3. No overlap and queue latest

The same Routine does not overlap by default:

- if no occurrence is active, the next eligible occurrence may start;
- if one is active, at most the latest pending occurrence is retained;
- an older pending occurrence superseded by a newer one is recorded as
  coalesced/skipped with reason and timestamps;
- different Routines still obey shared Project/Member/Provider availability,
  declared limits, and scheduler fencing;
- no queue policy turns a dropped occurrence into success.

The Product shows the active occurrence, latest queued occurrence, coalesced
count with declared denominator, and one next action.

Across Projects, Personal orders eligible work by Owner priority, deadline,
schedule, resource availability, and fairness. The current reason is
explainable; queue order is not inferred from Agent prose.

## 4. Offline, sleep, and missed work

Windows sleep, shutdown, daemon stop, network loss, Provider outage, connector
unavailability, or locked SecretStore can prevent dispatch. Personal records
the applicable state rather than simulating background cloud execution.

On resume, Today and the affected Project group:

- missed occurrences and their source/time;
- work safe to resume automatically;
- work needing fresh Context or Provider recovery;
- consequential work needing Owner review;
- stale/unknown external Effects that must reconcile before retry.

Low-risk internal work may resume inside the unchanged policy. Publishing,
communication, spending, deletion, permission expansion, or a changed
external target requires a new preview or renewed approval.

Expired external content is not silently backfilled. Personal shows the missed
occurrence and requires the applicable fresh research, policy, or Owner choice.

## 5. Closing the window

When eligible work is running or queued, closing the Control Plane asks:

- **Continue eligible work in background**; or
- **Pause after the current safe boundary**.

The dialog names affected Projects/Routines, work that cannot be stopped
instantly, offline limitations, and how to reopen status. It does not promise
execution after host shutdown. If a backend cannot yet honor a choice, the
prototype labels it `Requires-backend`.

A new Owner instruction creates a version and applies at a safe point by
continue, pause, or restart. It is never silently injected into a running
prompt.

## 6. Manager autonomy and reflections

Within an approved Project boundary, the manager may change a Routine's
subgoal, Task decomposition, order, frequency, or Member responsibility if the
primary goal, team, Provider/model, Tool/MCP grants, permissions, and
external-action rules remain unchanged.

A change outside that envelope creates a plan/Routine revision candidate,
structured diff, and Owner confirmation. The manager loop is observe -> plan
-> delegate -> execute -> independently verify -> summarize -> reflect ->
adjust. Reflection occurs per Task, day, cycle/week, and incident. Agent or
manager self-report does not admit a revision or mark work complete.

A one-off Task strategy adjustment may apply inside its boundary. A persistent
Member Runtime change creates a new version with replay/simulation/comparison
and rollback; the manager may activate it only inside the approved envelope.
Global Role, team, primary goal, Provider/model, Tool/MCP, permission, and
external-rule changes require Owner confirmation.

## 7. Progress and controls

A long-running surface shows:

- Project, Routine revision, Task, Attempt, and responsible Member;
- current step and latest durable fact—not hidden "thinking";
- queued/missed/coalesced occurrences;
- artifacts, Effects, evidence, Provider usage/cost basis;
- available pause/stop/retry/resume/reconcile actions;
- unsupported control reasons;
- final receipt separating completed, skipped, failed, unknown, and not-run
  work.

One Member Task process may create internal subagents only under explicit
count, time, cost, and permission limits. They return results to that Member,
receive no Project-member identity or long-term Memory, and cannot turn their
self-report into completion.

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
