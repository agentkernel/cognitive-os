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
TaskContract has no durable Context source-reference field, and `ContextView`
has no durable daemon admission/query path. The required real Context input
therefore cannot be supplied by implicit daemon configuration or Pi. The
remaining design is fixed as two ordered prerequisites:

1. Lane-CTR/KRN must add a TaskContract v0.4 strong `ContextView` binding and
   a daemon-owned durable ContextView admission/query path. A reference is not
   valid until the daemon can reload and verify its exact identity, version,
   digest, and current contract binding.
2. Lane-RUN must provide a daemon-created private pinned-Pi sidecar candidate
   transport. The existing bootstrap-session loopback client is read-only and
   cannot be widened into a worker authority route. The transport returns one
   bounded structured candidate only; it never grants Pi a tool permit.

## Remaining bounded blocker

- `blocked_paths`: P2-T04 candidate admission ordering and durable Context
  source binding
- `blocked_task_ids`: `P2-T04/D01`
- `blocked_gate_ids`: none; B02/B04/B05/B12 remain independently `not-run`
- Owner: product/architecture authority
- Next action: complete the ordered durable ContextView binding and private
  sidecar-transport prerequisites, then resume P2-T04 with the selected
  structured candidate shape. Candidate admission must persist before WIA
  issuance; Pi stays a candidate producer; P2-T06 remains the sole external
  process/executor dispatch path.

## Non-claims

No P2-T04 implementation, Pi invocation, external executor dispatch, Tool
execution, Context authority record, candidate admission, progress, evidence,
Effect change, verification, Task acceptance, or Task completion occurred.
P2-T04 is `blocked`, not complete; D05 remains independently complete with
its exact-revision native Linux evidence.
