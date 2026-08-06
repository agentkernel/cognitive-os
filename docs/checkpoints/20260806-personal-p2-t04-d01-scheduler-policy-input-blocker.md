# P2-T04/D01 scheduler pre-admission policy-input blocker

- Date: 2026-08-06
- Task / slice: `P2-T04/D01` private scheduler-to-Context-to-pinned-Pi worker
  composition
- Lease: `lease/personal/P2-T04/private-worker-composition` (active,
  partially blocked)
- Branch: `lane/ctr-p3-t01-context-request-binding`
- Draft PR: [#152](https://github.com/agentkernel/cognitive-os/pull/152)
- Checkpoint revision: `1690260ab76e57ad6c620d300ed82bbb5e0c43cc`

## Implemented checkpoint

The daemon now has a bounded, private Pi candidate transport and scheduler
adapter. The transport clears the child environment, rechecks the exact Pi
pin per invocation, bounds request/response and captured stderr, applies a
timeout, and accepts only a strict candidate response. It cannot carry WIA,
Effect, progress, evidence, receipt, or Task-state fields.

The daemon bridge resolves authorized Context before invoking the private
proposer, creates and seals the candidate itself, and performs the existing
atomic candidate-admission operation. Candidate-admission recovery can reload
the one durable receipt for a selected candidate instead of creating a second
WIA or budget debit. A Windows-only recovery-test startup polling window was
also extended after a transient endpoint-publication timeout in required CI.

## Pre-admission scheduler blocker

The real scheduler tick cannot yet safely call the bridge. A review of the
current durable TaskContract, ContextRequest, scheduler row, and authority
stores establishes only task/request identity, the Context principal, the
current contract epoch, loop/budget CAS inputs, and the candidate admission
checks performed after persistence. It does not establish all inputs required
to construct a Context resolution command and daemon admission command without
inventing product semantics.

`ContextResolutionCommand` lacks durable derivations for the resource-scope
prefix and optional conversation reference. A governed ResourceScope strong
reference is not the source-scope string used by Context discovery, and the
current intent reference may be a conversation or scope rather than a known
conversation binding. Tenant-scoped headers are also optional and require an
explicit matching policy.

`DaemonCandidateAdmissionCommand` lacks a durable policy source for the exact
operation authorization subject/purpose, per-admission `BudgetCharge`,
daemon-created governance provenance/identity, and a stable correlation URI.
Using test literals, a Context request purpose, a TaskContract creator, a
budget ceiling, or an arbitrary header purpose would silently invent those
policies and make a daemon-created admission claim false provenance.

## Required next slice

1. Define and persist the task-to-Context scope/conversation binding and its
   tenant consistency rule.
2. Define a daemon-owned candidate-admission policy that resolves operation
   subject/purpose, charge schedule, governance identity/provenance, and
   stable correlation identity from durable facts.
3. Add the scheduler caller before WIA lookup, with a stable candidate identity
   and no WIA consumption before candidate admission.
4. Replace the currently unproven private Pi invocation mode with an
   evidence-backed pinned Pi entrypoint, then validate the whole path on an
   exact committed native-Linux revision.

## Status and validation

- Change class: `implementation-only` with corrective test stability.
- Local checks: `cargo fmt` and `git diff --check` passed.
- Supported CI: Ubuntu run `31059951647` passed for `fc45274`; Windows run
  `31059955073` reached the full workspace test stage but failed only the
  existing scheduler recovery endpoint-publication timeout. The timeout
  correction is in `1690260`; its required Ubuntu/Windows runs are active at
  this checkpoint.
- Native Linux: not run. The remaining production invocation and policy inputs
  are incomplete, so no native runtime or P2-T04 completion claim is valid.

## Non-claims

P2-T04/D01 remains `in-progress`. This checkpoint does not add a real
scheduler pre-admission caller, a supported Pi-side entrypoint, Tool execution,
progress, evidence, verification, acceptance, Task completion, a Gate result,
release, or Profile claim.
