# P2-T03 execution-binding contract handoff

- Date: 2026-08-01
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/execution-binding-contract` (closed)
- Branch: `lane/ctr-p2-t03-execution-binding`
- Change class: normative-semantic / Lane-CTR
- Task status: `in-progress`
- Normative surface: TaskContract and loop transition table updated

## Delivered contract unblock

TaskContract now durably binds `deadline`, `loop_object_id`, and `budget_id`.
The deterministic mint path writes those facts in the immutable canonical
contract, marks new contracts `cognitiveos.task-contract/0.2`, and includes
them in the digest-bound P2-T01 admission preview. Rust and TypeScript
generated bindings were updated from the schema source.

The loop transition table is now version `0.2` and permits only `START` or
`CONTINUE` to stop for deadline, retry, step, or cost ceilings. Each edge
requires current ceiling facts, dispatch disablement, resolved effects, and
TaskContract/checkpoint/budget evidence. No arbitrary active phase receives a
new stop transition.

## Validation

- Linux host `wuz@192.168.1.2`, no-secret archive snapshot:
  `cargo test -p cognitive-store --test m5_intent_chain` -- pass, 6/6.
- Contract bindings were regenerated with
  `cargo run -p cognitive-contracts --bin contracts-codegen` on that Linux
  host before being checked in.
- Remaining required checks: schema/TS contract suites, transition exhaustive
  negatives, conformance vectors/matrix, fmt/clippy, and protected CI are
  `not-run` for this initial contract-unblock commit.

## Remaining work

- `blocked_paths`: none for a daemon-owned P2-T03 implementation slice
- `blocked_task_ids`: none
- `blocked_gate_ids`: B02, B04, B05, B12, GMVP-LINUX
- owner: next Lane-RUN P2-T03/P2-T04 session
- next action: load these bound facts from the current contract and authority
  store, establish the new stop-edge guards/evidence in one fenced transition,
  then acquire a scheduler lease only on non-stop admission and connect the
  worker to BoundedHarness.

## Non-claims

No daemon scheduler adapter, worker, external dispatch, Task completion,
budget debit, Gate result, release claim, or Profile claim is added.
