# P2-T04/D01 scheduler pre-admission policy-input blocker

- Date: 2026-08-06
- Task / slice: `P2-T04/D01` private scheduler-to-Context-to-pinned-Pi worker
  composition
- Lease: `lease/personal/P2-T04/private-worker-composition` (active,
  partially blocked)
- Branch: `lane/ctr-p3-t01-context-request-binding`
- Draft PR: [#152](https://github.com/agentkernel/cognitive-os/pull/152)
- Checkpoint revision: pending this handoff commit

## Implemented checkpoint

The daemon bridge resolves authorized Context before invoking a private
proposer, creates and seals the candidate itself, and performs the existing
atomic candidate-admission operation. Candidate-admission recovery can reload
the one durable receipt for a selected candidate instead of creating a second
WIA or budget debit. A Windows-only recovery-test startup polling window was
also extended after a transient endpoint-publication timeout in required CI.

The pre-admission policy store now provides immutable, exact
`(task_ref, contract_epoch)` bindings for the Context query and daemon-owned
candidate-admission inputs. The scheduler reloads and validates that policy
before WIA lookup. Both Context-command and candidate-admission composition
independently reject malformed, empty, or row-mismatched policy facts.
Exact policy-row replay is idempotent; if TaskContract admission loses its CAS
after policy persistence, a retry for that same next epoch reloads the policy's
sealed ContextRequest and stable candidate identity instead of minting a
second immutable policy.

There is deliberately **no callable private Pi transport** at this checkpoint.
The former child-process implementation has been made fail-closed before spawn
because the pinned Pi surface does not prove its assumed request/response
protocol. It never passes Context to Pi, reads a provider configuration, or
creates WIA, Intent, Effect, budget debit, progress, evidence, verification,
acceptance, or Task completion.

## Pre-admission scheduler blocker

The real scheduler tick cannot yet safely call the bridge because it has no
supported private Pi candidate producer. It can now reconstruct the Context
resolution and daemon admission commands from immutable daemon-owned policy;
it must not substitute defaults. The remaining blocker is an evidenced,
bounded candidate protocol that preserves this separation.

## Required next slice

1. Establish an evidence-backed pinned, sessionless candidate entrypoint. The
   exact `0.81.1` `--help` output supports `--print`, `--mode`,
   `--append-system-prompt`, and explicit `--extension` loading, but does not
   establish an extension-defined candidate flag or stdin JSON protocol.
2. The entrypoint must receive exactly one bounded candidate request and emit
   exactly one bounded candidate response, with diagnostics isolated from the
   response. It must receive neither a bootstrap secret, daemon bearer, nor
   ambient provider credential.
3. Add the scheduler caller before WIA lookup only after that protocol exists,
   preserving the persisted stable candidate ID and no WIA consumption before
   candidate admission. Then validate the whole path on an exact committed
   native Linux revision.

## Status and validation

- Change class: `implementation-only` with corrective fail-closed transport
  removal and policy-binding regressions.
- Local checks: `cargo fmt --all -- --check`, `git diff --check`, and the
  Pi extension package build/test passed. Local Rust test execution remains
  blocked by the documented Windows GNU linker exit 121.
- Supported CI: Ubuntu and Windows passed the full required suite for
  `4db146247f10a2780fd438b419c2ab4e6140f04b`, which includes the Windows
  scheduler-recovery timeout correction. This handoff-only update remains
  pending its own required CI runs.
- Native Linux: not run. The remaining production invocation and policy inputs
  are incomplete, so no native runtime or P2-T04 completion claim is valid.

## Non-claims

P2-T04/D01 remains `in-progress`. This checkpoint does not add a real
scheduler pre-admission caller, a supported Pi-side entrypoint, Tool execution,
progress, evidence, verification, acceptance, Task completion, a Gate result,
release, or Profile claim.
