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
    symbols: ["record_user_intent", "mint_task_contract"]
  - path: apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
    symbols: ["run_private_scheduler_tick"]
tests:
  - crates/cognitive-runtime/tests/p2_t01_task_application_service.rs
  - crates/cognitive-store/tests/m5_intent_chain.rs
fingerprint: "sha256:676c5492e30ef088f59b6e8f9ec12ff38715895eaf31a84817e2ceb67d580800"
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
4. **Admit** — you accept exactly that digest; the daemon mints the TaskContract
   under an epoch CAS. Changing your mind later supersedes to a new epoch and
   fences everything bound to the old one.

This admission pipeline is `implemented` and is the only human approval point on
the default path. `GET /task/watch` gives a bounded, snapshot-first event stream.

## How execution is designed to run — and what runs today

Designed chain: scheduler lease → sealed Context → Pi produces a **candidate** →
daemon admits it as Intent + Effect + a one-time Worker Iteration Authorization →
governed tool execution (persist-before-dispatch) → independent verification →
loop continuation or STOP.

Today each stage exists with focused tests (lease CAS and fencing, sealed
ContextViews, candidate admission bundles, WorkspaceRead/ProcessCheck executors with
unknown-outcome reconciliation, an independent verifier seam), **but the daemon does
not yet drive the chain autonomously**: admission does not enqueue scheduler work,
the daemon runs only one scheduler pass at startup, and production code does not yet
call the tool executors or verifier. So: admitted Tasks are durable and watchable;
autonomous execution is `partial`. Details for developers:
[execution-chain status](../developer/execution-chain-status.md).

## What can never happen, by construction

- A Provider reply, Pi `agent_end`, tool exit 0, or process exit is never Task
  completion (independent verification is required).
- An unknown external outcome is never blindly retried under a new identity —
  reconciliation reuses the original idempotency key.
- Budgets and deadlines are inclusive hard rails checked before dispatch.
