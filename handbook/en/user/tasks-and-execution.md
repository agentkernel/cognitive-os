---
doc_id: user.tasks-and-execution
locale: en
kind: concept
audience: [user]
status: partial
generated: false
sources:
  - path: apps/kernel-server/src/personal/task_api.rs
    symbols: ["TaskApi"]
  - path: crates/cognitive-management/src/task_application.rs
    symbols: ["KernelTaskApplicationService"]
  - path: crates/cognitive-kernel/src/intent_chain.rs
    symbols: ["record_user_intent", "mint_schedulable_task_contract"]
  - path: apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
    symbols: ["run_private_scheduler_tick"]
tests:
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
  - crates/cognitive-store/tests/m5_intent_chain.rs
fingerprint: "sha256:d9f50d04d639be75cb74f65f07339e2ba31326830f2e474440e15dab77a2dbcc"
non_claims:
  - No claim that admitted Tasks execute autonomously today; the execution pipeline's component evidence lives in focused tests, not an end-to-end product path.
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
   `START` Loop, hard Budget, and current-epoch runnable scheduler row. Changing
   your mind later supersedes to a new epoch and fences everything bound to the
   old one.

This admission pipeline is `implemented` and is the only human approval point on
the default path. `GET /task/watch` gives a bounded, snapshot-first event stream.

## How execution is designed to run — and what runs today

Designed chain: scheduler lease → sealed Context → Pi produces a **candidate** →
daemon admits it as Intent + Effect + a one-time Worker Iteration Authorization →
governed tool execution (persist-before-dispatch) → independent verification →
loop continuation or STOP.

Today admission durably enqueues its complete scheduler bootstrap, and each later
stage exists with focused tests (lease CAS and fencing, sealed ContextViews,
candidate admission bundles, six assembled Tool executors with unknown-outcome
reconciliation, an independent verifier seam). Zero-Intent work now reaches
candidate admission and leaves its new worker authorization for a later pass.
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

## What can never happen, by construction

- A Provider reply, Pi `agent_end`, tool exit 0, process exit, worker self-report,
  or stale verifier report is never Task completion. Current independent
  verification, unchanged fixed state, closed Effects, retrievable evidence and
  the separate daemon acceptance authority are all required.
- An unknown external outcome is never blindly retried under a new identity —
  reconciliation reuses the original idempotency key.
- Budgets and deadlines are inclusive hard rails checked before dispatch.
