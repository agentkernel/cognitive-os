# P2-T04/D01 pre-admission candidate boundary blocker

- Date: 2026-08-05
- Task / slice: `P2-T04/D01` private scheduler-to-Context-to-pinned-Pi worker composition
- Lease: `lease/personal/P2-T04/private-worker-composition` (active, blocked)
- Branch: `lane/run-p2-t04-private-worker-composition`
- Predecessor evidence: `P2-T03/D05` and `P2-T07/D01` closed at
  `08932f7868d46f494aaa76835f4818fd7a1f2962`
- Predecessor PR: [#149](https://github.com/agentkernel/cognitive-os/pull/149) (Draft)

## Blocker found before implementation

The current daemon tick can consume only a `WorkerIterationAuthorization` that
was issued after an immutable candidate had already been selected and admitted.
P2-T04 requires the inverse order: daemon-owned Context must be resolved, a
pinned Pi boundary may propose an untrusted candidate, and only then may the
daemon perform candidate admission to issue the WIA.

Additionally, the deterministic Context resolver consumes a caller-supplied
candidate set; the repository has no durable TaskContract-to-Context source
binding. There is also no Rust-to-Pi candidate-execution bridge. Calling Pi
after WIA consumption would fabricate the causal order and risk a partial
handoff; treating its text as already admitted, progress, evidence, Effect
closure, acceptance, or Task completion would violate the authority model.

## Bounded decision required

- `blocked_paths`: P2-T04 candidate admission ordering and durable Context
  source binding
- `blocked_task_ids`: `P2-T04/D01`
- `blocked_gate_ids`: none; B02/B04/B05/B12 remain independently `not-run`
- Owner: product/architecture authority
- Next action: choose a daemon-owned pre-admission candidate proposal and
  TaskContract-to-Context binding design. It must persist candidate admission
  before issuing WIA, use Pi only as candidate producer, and leave P2-T06 as
  the only external process/executor dispatch path.

## Non-claims

No P2-T04 implementation, Pi invocation, external executor dispatch, Tool
execution, Context authority record, candidate admission, progress, evidence,
Effect change, verification, Task acceptance, or Task completion occurred.
P2-T04 is `blocked`, not complete; D05 remains independently complete with
its exact-revision native Linux evidence.
