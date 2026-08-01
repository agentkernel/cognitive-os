# P2-T03 durable-authority blocker handoff

- Date: 2026-08-01
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/durable-authority` (closed)
- Branch: `lane/personal-p2-t03-durable-authority`
- Change class: implementation-only assessment; no implementation started
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: unchanged (`tested-local` from prior slices)
- Normative surface: unchanged

## Bounded blocker

The requested next path must connect deadline, retry, step, and cost ceilings
to durable authority facts and persist a stop fact before worker dispatch. The
repository currently lacks three required authority surfaces:

1. `TaskContract` and its generated schema/command have no durable task
   deadline. An AKP transport deadline is not task authority and must not be
   repurposed.
2. `scheduler_entries` is keyed by `task_ref`, progress facts by
   `loop_object_id`, and budgets by `BudgetId`; no durable task-to-loop or
   task-to-budget binding exists. The daemon must not infer those identities.
3. The registered loop transition table contains no ceiling-to-`STOP` edge.
   `cancel_requested`, checkpoints, and progress facts cannot be overloaded as
   a ceiling-stop fact without changing their contracts.

No source or test changes were made beyond this documented blocker. The lease
is closed so it does not obstruct P2-T02 or future correctly-scoped work.

## Required owner and next action

- `blocked_paths`: `specs/schemas/task-contract.schema.json`, generated
  TaskContract bindings, task contract command/model, loop transition table,
  and associated negative vectors
- `blocked_task_ids`: P2-T03 deadline/stop-fact and P2-T04 worker integration
- `blocked_gate_ids`: B02, B04, B05, B12, GMVP-LINUX
- owner: Lane-CTR / contract owner
- next action: register the authoritative Task deadline and the legal
  ceiling-stop lifecycle semantics through Lane-CTR, including generated
  bindings, transition and negative-vector coverage; decide and register the
  durable task-to-loop/budget binding rather than inferring it at runtime.

After the contract work merges, a Lane-RUN slice may add a daemon-private
adapter that reloads those authoritative facts, persists a fenced stop fact,
and only then admits scheduler/BoundedHarness work.

## Non-claims

No deadline, stop fact, dispatch, worker, Task completion transition, budget
debit, Gate result, release claim, or Profile claim was added. Rust daemon
authority boundaries remain unchanged.
