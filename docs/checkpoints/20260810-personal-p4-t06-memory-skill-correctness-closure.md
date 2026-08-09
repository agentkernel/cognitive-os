# P4-T06 Memory/Skill correctness and same-task consumption closure

## Task boundary

- Task: `P4-T06`
- Slices: `D01-D03`
- Branch: `personal/P4-T06-memory-skill-correctness`
- Lease: `lease/personal/P4-T06/memory-skill-correctness`
- PR: `#177`
- Scope: daemon-private correctness and same-task consumption evidence only

## Delivered consumption boundary

- `POST /task/resource/v1/consumption` requires a Task-channel bearer.
- The daemon loads the current Task contract epoch, exact TaskContract,
  scheduler execution policy, and referenced ContextRequest before consuming
  resource facts.
- Retrieval scope and Memory purpose are daemon-derived. Client payloads cannot
  select a workspace scope or Memory purpose.
- Memory discovery uses the existing authority-filtered, non-expired,
  non-forgotten FTS retrieval port.
- Skill consumption requires an active, non-revoked binding whose workspace
  and target are compatible with the daemon-derived task scope.
- The resulting private trace binds the Task contract epoch/digest,
  ContextRequest identity/digest, selected Memory source identities/digests,
  and the exact Skill binding/revision/package/content digest.
- The response is metadata-only and has no authority write side effects.

## Failure-first coverage

- Management bearer crossing into the Task consumption channel is rejected.
- Malformed consumption bodies fail before authority reads.
- Unknown task references fail before Memory or Skill discovery.
- Missing/malformed Task policy and ContextRequest facts fail closed.
- Revoked or scope-incompatible Skill bindings are rejected.
- Forgotten, expired, stale, or source-mismatched Memory remains excluded by
  the existing Memory authority-filtered search boundary.

## Validation

- `cargo fmt --all`: passed locally.
- `git diff --check`: passed locally.
- `pnpm run check:consistency`: passed locally.
- Required Ubuntu and Windows CI run `31338813801`: passed.
- Exact-revision native Linux validation at
  `f4b4d38cd1b1c03bd918881d5d2fa0b99d5946f8` passed
  `cargo test -p kernel-server --test p4_t05_resource_api` (1/1) and
  `cargo clippy -p kernel-server --all-targets -- -D warnings`.
- Local Windows GNU Rust build/test/Clippy were not run because of the
  registered unsupported linker failure.

## Explicit non-claims

This task does not pass B08 and makes no Gate, release, Profile, public API,
public contract, embedding/vector retrieval, Skill capability grant, script
execution, or Task completion claim. The trace is a private authority-backed
consumption observation; it does not create a new generic Resource aggregate
or a public Memory/Skill schema.
