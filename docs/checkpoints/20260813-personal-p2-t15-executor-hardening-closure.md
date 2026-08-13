# P2-T15 closure — independent-review executor hardening

- Task: `P2-T15`
- Slices: `P2-T15/D01-D03`
- Branch: `personal/executor-hardening-review-fixes`
- Draft/closure PR: [#208](https://github.com/agentkernel/cognitive-os/pull/208)
- Lease: `lease/personal/P2-T15/executor-hardening-review-repair`
- Accepted implementation/native revision:
  `580c0a06d39ee3d6fb460e23be9c7ac0939a4b63`
- Change class: `implementation-only`; normative surface unchanged

## Acceptance mapping

1. Mutation target, parent and staging operations use component-wise,
   handle-relative no-follow opens; opened-handle metadata rejects links and
   Windows reparse points. Linux active parent swaps and Windows reparse paths
   are covered.
2. A stable per-target OS lock serializes CognitiveOS writers, the preimage is
   checked through the held parent immediately before handle-relative rename,
   and a deterministic final-window competitor cannot be overwritten.
3. Mutation reconciliation requires a durable receipt bound to the original
   idempotency key. Matching bytes from a competitor are indeterminate; a
   later reversion does not erase a completed receipt.
4. HTTP attempted/completed state is durable. Timeout/network and missing-state
   restart paths reconcile indeterminate; completed original-key receipts
   survive restart without a second request.
5. Search opens each file/directory no-follow relative to an already-open
   parent and verifies metadata after open. Active file/directory swaps and
   Windows reparse paths never expose outside content.
6. Validation and every sink require exact equality with the immutable
   built-in descriptor. Every immutable field is drifted across every family.
7. `maximum_visited_entries` is consumed during enumeration, so a large
   directory is never collected before the ceiling applies.
8. Whole-file write preimages hash as a stream. Patch preimages have an
   explicit 4 MiB ceiling; sparse/over-limit negatives retain no unbounded
   allocation.
9. Staging cleanup failures produce Unknown/Indeterminate. Restart removes a
   regular orphan and refuses hostile/unremovable residue without hiding it.
10. Unified diff `\ No newline at end of file` semantics apply independently
    to old and new sides, including adding/removing a final newline.
11. Provider readiness resolves the secret reference from the already-loaded
    config snapshot; a deterministic fake-SecretStore swap cannot mix config
    versions.

Post-CI defect-first reviews also closed state-root link following, receipt
placement inside a mutable workspace, and state-loss restaging. Stable lock
files are an independent seen-key witness, so a missing record cannot be
recreated as fresh `NotExecuted`.

## Validation

- Local Windows non-linking: `cargo fmt --all -- --check`, Node tools 58/58,
  consistency, handbook, generator and docs-sync checks pass.
- Required GitHub Ubuntu/Windows matrix at exact `580c0a0`: run
  `31666003044`, Ubuntu job `94340591397`, Windows job `94340591336`, both
  pass.
- Exact `DEV-LINUX-NATIVE-01` at `580c0a0`:
  - focused Tool executor: 76/76 pass;
  - focused readiness: 11/11 pass;
  - full kernel-server package: 179/179 unit tests plus all integration suites
    pass;
  - full Rust workspace: all executed unit/integration/doc tests pass (one
    pre-existing runtime test remains explicitly ignored);
  - `cargo clippy -p kernel-server --all-targets --locked -- -D warnings`
    pass;
  - `cargo fmt --all -- --check` pass;
  - disposable native worktree removed cleanly.
- Final defect-first review: **no findings**.
- Running detail:
  [20260813-personal-executor-hardening-validation.md](20260813-personal-executor-hardening-validation.md).

## Residual limits and non-claims

- These test-called sinks still have no production worker caller; P2-T12 and
  P2-T13 own that separate wiring.
- The target lock serializes CognitiveOS writers. Arbitrary third-party writers
  cannot be forced to honor a portable advisory lock; the final preimage
  recheck detects the deterministic uncooperative race.
- Production composition must provide one shared daemon-private durable state
  root outside every approved workspace.
- No Gate, release, Profile, B01, benchmark-performance or Agent-benefit claim
  is created.

## Closure disposition

All task acceptance and supported/native validation are complete. PR #208 may
leave Draft, merge normally, archive the lease, remove the task branch, and
reconcile local `main`; no force push or history rewrite is permitted.
