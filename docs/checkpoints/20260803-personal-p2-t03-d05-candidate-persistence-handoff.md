# P2-T03/D05 candidate persistence checkpoint

- Date: 2026-08-03
- Task / slice: `P2-T03/D05` daemon-only candidate-input persistence
- Lease: `lease/personal/P2-T03/worker-input-contract` (active)
- Branch: `lane/ctr-p2-t03-worker-input-contract`
- Code checkpoint: `50d4fc603e55be8277fcf47392c6240c5a2bb568`
- PR: #149 (Draft)
- Change class: `implementation-only` with corrective conformance count pins
- Normative surface: unchanged

## Checkpointed implementation

Migration v4 adds `operation_candidate_proposals`, an append-only private
SQLite table. `WorkerAuthorizationStore` persists and reloads immutable
`OperationCandidateProposalRow` observations. Duplicate candidate identities
fail as conflicts and cannot replace the originally recorded observation.

This checkpoint deliberately does not admit a candidate or create an Intent,
Effect, WorkerIterationAuthorization, budget debit, scheduler dispatch,
progress fact, Task acceptance, or Task completion. Those remain daemon-only
steps for the next D05 slice.

## Validation

Local Windows non-linking checks passed:

```text
cargo fmt --check
git diff --check
pnpm run check:consistency
```

Exact immutable Linux validation used disposable Git worktrees on
`personal-linux-native-01`:

```text
d8726350906c7b9538332f4c6e7ae3f29e6f374a
cargo test -p cognitive-store --test p2_t03_worker_authorization --locked
cargo test -p cognitive-store --test p1_t01_layout_migrations --locked
```

Both commands completed successfully. The final exact revision
`50d4fc603e55be8277fcf47392c6240c5a2bb568` passed:

```text
cargo test -p cognitive-conformance --test runner_execution --locked
13 passed; 0 failed
```

The same Linux toolchain rejected its `cargo fmt` `--check` invocation as an
invalid option after the conformance test. This is `not-run` formatting
evidence for that exact final revision, not a source-format failure; local
`cargo fmt --check` passed and the earlier immutable candidate checkpoint
completed its Linux formatting check.

## Remaining work and non-claims

Implement daemon-only candidate admission that reloads the current contract,
tool/descriptor, capability/revocation, target version, Loop, and Budget before
creating the durable Intent/Effect/WIA bundle. Then bind one-time WIA
consumption atomically to Loop CAS and the exact budget debit before any
lease-fenced scheduler dispatch.

No Provider, secret, privileged action, B01 guest, or external operation was
used. D05, P2-T03, B02/B04/B05/B12, release, and Profile remain incomplete.
