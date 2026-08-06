# P2-T04/D01 pre-admission candidate boundary blocker

- Date: 2026-08-05
- Task / slice: `P2-T04/D01` private scheduler-to-Context-to-pinned-Pi worker composition
- Lease: `lease/personal/P2-T04/private-worker-composition` (active)
- Branch: `lane/ctr-p3-t01-context-request-binding`
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

## Candidate proposal decision recorded

The product/architecture authority selected a **structured operation proposal**
for the pre-admission Pi boundary. Pi may propose only the existing
`OperationCandidateProposal` candidate fields: an allowed Tool reference,
action, target, immutable operation-descriptor reference, and bounded
schema-validated parameter digest. The proposal remains non-authoritative.
The daemon must reload and validate TaskContract, descriptor, authorization,
budget, lease, and fencing facts, persist the immutable candidate, and only
then atomically admit it and issue a WIA.

This decision does not create a Context source binding. The current public
TaskContract has no durable Context source-reference field, and Context has no
durable daemon admission/query path. The required real Context input therefore
cannot be supplied by implicit daemon configuration or Pi. The remaining design
is fixed as two ordered prerequisites:

1. Lane-CTR/KRN must add a TaskContract v0.4 strong `ContextRequest` binding
   and a daemon-owned durable ContextRequest/ContextView admission/query path.
   The request is the immutable contract input; each resolved ContextView is a
   request-linked per-resolution artifact, and each LoopCheckpoint retains the
   exact view used for that iteration. A reference is not valid until the
   daemon can reload and verify its exact identity, version, digest, task
   perspective, and current contract binding.
2. Lane-RUN must provide a daemon-created private pinned-Pi sidecar candidate
   transport. The existing bootstrap-session loopback client is read-only and
   cannot be widened into a worker authority route. The transport returns one
   bounded structured candidate only; it never grants Pi a tool permit.

## 2026-08-06 durable policy checkpoint

Checkpoint `331a584` adds the private, append-only
`SchedulerExecutionPolicyRow` store boundary and authority schema migration
v15. A policy is keyed by the exact `(task_ref, contract_epoch)` binding and
records the bound `ContextRequest` identity plus a daemon-issued canonical
policy document. Duplicate policy rows conflict, and policy identity cannot be
updated or deleted. The migration and immutable/epoch-bound lookup regression
are included in the checkpoint.

Checkpoint `fe5eb33` makes scheduler tick reload that policy before WIA lookup
for every Context-bound v0.4 TaskContract. A missing policy, weak/unversioned
ContextRequest reference, or policy/request mismatch fails closed before any
Pi invocation or WIA consumption. This is a pre-admission fence only: it does
not yet construct the full Context/admission commands or make Pi callable.

The checkpoint does **not** yet write an execution policy at Task admission.
The existing TaskContract and ContextRequest durable facts do not safely
determine the query tenant/scope, candidate admission subject/purpose/charge,
daemon-created governance provenance, or correlation identity. The next
vertical implementation must persist those values as daemon-owned policy
inputs atomically with, or fail-closed alongside, Task admission; it must not
invent defaults in the scheduler.

## Remaining bounded blocker

- `blocked_paths`: Task-admission policy creation, supported sessionless
  secret-free pinned Pi candidate entrypoint, and scheduler bridge invocation
- `blocked_task_ids`: `P2-T04/D01`
- `blocked_gate_ids`: none; B02/B04/B05/B12 remain independently `not-run`
- Owner: daemon runtime implementation
- Next action: persist a complete daemon-owned scheduler execution policy at
  Task admission, then construct Context/admission commands and invoke a
  documented sessionless, secret-free pinned Pi candidate entrypoint before
  WIA lookup. Candidate admission must persist before WIA issuance; Pi stays
  a candidate producer; P2-T06 remains the sole external process/executor
  dispatch path.

## Non-claims

The daemon bridge and candidate-only transport boundary exist, but no real
supported Pi candidate request is callable from scheduler tick yet. No external
executor dispatch, Tool execution, progress, evidence, Effect change,
verification, Task acceptance, or Task completion is claimed. P2-T04 remains
`in-progress`, and D05 remains independently complete with its exact-revision
native Linux evidence.
