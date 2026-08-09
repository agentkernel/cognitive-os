# P4-T03 Memory lifecycle closure

- Task: `P4-T03` -- Memory lifecycle, retention, and forget
- Change class: `implementation-only`; normative and public contract surfaces unchanged
- Branch: `personal/P4-T03-memory-lifecycle`
- Exact implementation revision: `8f9250dcd4cbcd8f15867e7a0f45165032e26c9d`
- PR: #174 merged at `main@e1454f3775eab5c72d9cb2b8e0a5c1e98b895f0f`; https://github.com/agentkernel/cognitive-os/pull/174
- Lease: `lease/personal/P4-T03/memory-lifecycle` closed. The remote task branch was deleted.

## Acceptance mapping

| Acceptance | Implementation and evidence |
|---|---|
| Forget/tombstone and audit | D01 appends immutable reason-coded forget facts without deleting admission history. Unknown and duplicate forget attempts conflict, and a rebuilt FTS index cannot restore forgotten Memory. |
| Retention expiry | D02 accepts only an expiry fact at or after the immutable retention deadline, rejects premature and duplicate attempts, and removes the derived FTS row atomically. |
| Version, update, and conflict | D03 migration v20 preserves immutable version lineage. A daemon-private update requires the expected version, appends a supersede fact, creates a replacement, and moves the FTS row in one transaction; stale update attempts conflict. |
| Derived-index invalidation | Search and rebuild exclude every lifecycle fact, so stale, orphaned, forgotten, expired, and superseded FTS rows do not become discoverable. |

## Validation

- PASS -- local `cargo fmt --all`, `git diff --check`, and `pnpm run check:consistency`.
- PASS -- exact `DEV-LINUX-NATIVE-01` checkout at the recorded revision:
  `cargo test -p cognitive-store --test p4_t02_memory_search --test p1_t01_layout_migrations`
  (16 passed) and `cargo clippy -p cognitive-store --tests -- -D warnings`.
- PASS -- required CI run `31324346682` at the recorded revision: Ubuntu and Windows both succeeded.
- NOT RUN -- B08, any product Gate, release, and Profile campaign; none is implied by this task closure.

## Closure state

The complete P4-T03 task acceptance is satisfied. Required CI passed at the
closure documentation revision `7473f4254535f53b8331960960f83c7b97f0c5d8`;
PR #174 is merged, the lease is closed, the remote branch is deleted, and local
`main` is at the merge result. This task creates no public Memory API, B08,
product Gate, release, or Profile claim.
