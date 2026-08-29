---
doc_id: user.tasks-and-execution
locale: en
kind: concept
audience: [user]
status: partial
generated: false
sources:
  - path: personal/apps/kernel-server/src/personal/task_api.rs
    symbols: ["TaskApi"]
  - path: personal/crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
  - path: core/crates/cognitive-kernel/src/intent_chain.rs
    symbols: ["record_user_intent", "mint_schedulable_task_contract"]
  - path: personal/apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
    symbols: ["run_private_scheduler_tick"]
  - path: personal/docs/product/agent-integration-and-conversations.md
  - path: personal/docs/product/agent-integration-and-conversations.zh-CN.md
  - path: personal/docs/architecture/multi-agent-orchestration.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: personal/docs/architecture/project-role-employee.md
  - path: personal/docs/architecture/routine-trigger-missed-run.md
tests:
  - personal/crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
  - personal/crates/cognitive-store/tests/m5_intent_chain.rs
fingerprint: "sha256:13aa2540b523f9b6f1aaed23cfd586d252296d32a613019414acb2012916a06c"
non_claims:
  - Admission still does not consume the worker authorization or acquire a scheduler lease on the same pass; a later tick does. No Gate, release, Profile, or EVAL promotion.
---

# Tasks and execution

## What a Task is here

A Task is not "whatever the agent said it did". It is a governed object with a
durable paper trail:

1. **Record** — your raw words are persisted before any AI interpretation
   (`POST /task/intent.record`).
2. **Interpret** — a model may propose objectives/assumptions; material ambiguity
   forces `clarification_required`. The proposal is persisted as a candidate, never
   as truth.
3. **Preview** — the daemon issues a canonical, digest-bound contract preview
   (objectives, scope, budgets, deadline, allowed tools, acceptance conditions).
4. **Admit** — you accept exactly that digest; under one fenced epoch-CAS
   transaction the daemon mints the TaskContract and publishes its named
   `START` Loop, hard Budget, and current-epoch runnable scheduler row, plus
   owner-local Context authorization for tenant `personal`. Changing
   your mind later supersedes to a new epoch and fences everything bound to the
   old one.

This admission pipeline is `implemented` and is the only human approval point on
the default path. `GET /task/watch` gives a bounded, snapshot-first event stream.
Authenticated task callers can also read bounded O2/O3/O4/O5/O13 observation
(`GET /task/observation?family=…&task_ref=…`) and Effect history
(`GET /task/effects?task_ref=…`); empty observation windows return a named
`observed_zero` rather than a silent count. O13 audit replay fails closed on a
stale cursor or digest break.

## How execution is designed to run — and what runs today

Designed chain: scheduler lease → sealed Context → Pi produces a **candidate** →
daemon admits it as Intent + Effect + a one-time Worker Iteration Authorization →
governed tool execution (persist-before-dispatch) → independent verification →
loop continuation or STOP.

Today admission durably enqueues its complete scheduler bootstrap, including
owner-local Context authorization so the first later pass can resolve Context.
Zero-Intent work now reaches candidate admission after walking Loop
`START -> DECIDE`, and leaves its new worker authorization for a later pass
that acquires the scheduler lease.
One non-reentrant periodic worker starts after the daemon is listening, so later
passes can observe Tasks admitted by the running process; pass errors do not
stop the listener, and orderly shutdown cancels and joins the worker.
**The daemon's public C1 completion implementation is native-proven**:
production dispatches parameter-free WorkspaceRead, independently verifies its
fixed reconciled Effect, then derives candidate and final acceptance only from
current CAS-backed authority facts. Exact native `22c3f502` reached `COMPLETED`.
Open-Effect, superseded-report and missing-CAS negatives are written; a stale
fixed post-state negative is still open. A RegisteredCheck-terminated software-repair
Task can continue from a closed WorkspaceWrite back to Loop `DECIDE` and complete
only after RegisteredCheckRun plus independent verification.
So admitted Tasks are durable, watchable, and runnable in authority state;
autonomous execution remains `partial`. Details for developers:
[execution-chain status](../developer/execution-chain-status.md).

## Personal 2.0 Project work (`Requires-backend`)

The current durable work object is still **Task**. The OPC target adds a
Project above it:

`Project -> Charter/Goal/Plan revision -> Routine -> Task -> Attempt`.

Project setup remains a draft until the Owner confirms the daemon-issued
charter/team/permission/budget/trigger preview. Every active Project has one
current manager. The manager may adjust approved subgoals, Tasks, order,
frequency and responsibility inside the approved envelope; primary goal, team,
budget, Provider, Tool, permission or external-rule changes require a new
Owner-confirmed revision.

Each retry/fork creates a new Attempt and preserves the prior failure/evidence.
Routine triggers may be manual, scheduled or qualified events; the same Routine
does not overlap, keeps only the latest pending occurrence, records coalesced/
missed work and asks for consequential catch-up after offline time.

Digital employees coordinate through daemon-owned Tasks, artifacts and
handoffs. DSH is the target default runtime, but process output and engine
checkpoints remain observations. These Project/Routine/Employee capabilities
are not current APIs and do not rename current Task rows.

## What can never happen, by construction

- A Provider reply, Pi `agent_end`, tool exit 0, process exit, worker self-report,
  or stale verifier report is never Task completion. Current independent
  verification, unchanged fixed state, closed Effects, retrievable evidence and
  the separate daemon acceptance authority are all required.
- An unknown external outcome is never blindly retried under a new identity —
  reconciliation reuses the original idempotency key.
- Budgets and deadlines are inclusive hard rails checked before dispatch.
