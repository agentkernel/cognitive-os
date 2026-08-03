# P2-T02 Task preview/admit contract handoff

- Date: 2026-08-03
- Task and slice: `P2-T02/D01`
- Change class: normative-semantic
- Lease: `lease/personal/P2-T02/task-preview-admit-contracts`, closed in the
  PR #138 merge delivery
- Status at handoff: contract prerequisite implemented and validated; the
  parent `P2-T02/D01` remains `in-progress`

## Recovery tuple

| Field | Value |
|---|---|
| Branch | `lane/ctr-p2-t02-task-preview-admit-contracts` |
| Immutable validated checkpoint | `aa5007f9ce1c74d5d309101806233d9a6d3a8771` |
| Upstream | `origin/lane/ctr-p2-t02-task-preview-admit-contracts` at the validated checkpoint |
| Pull request | [#138](https://github.com/agentkernel/cognitive-os/pull/138), ready for merge after required CI passed |
| Worktree | clean at lease closure |

## Implemented contract

The narrow public `task.preview` / `task.admit` boundary is now registered.

- `TaskPreviewRequest` carries a pre-admission TaskContract draft only. It
  excludes a governed header, contract epoch, acceptance fact, and preview
  digest; the daemon must derive those authority facts.
- `TaskPreviewResult` is daemon-issued and digest-bound. It is not a contract,
  dispatch receipt, Effect result, verifier result, or Task completion signal.
- `TaskAdmitRequest` returns that preview digest with explicit interpretation
  acceptance and the expected contract epoch. The daemon must recompute the
  draft digest and verify digest, acceptance, and epoch CAS before mutation.
- `TaskAdmitResult` identifies the immutable TaskContract and epoch only. It
  makes no dispatch, Effect, verification, Task-completion, Gate, release, or
  Profile claim.

Existing `WatchSubscription`, `AkpStreamFrame`, `WATCH_CURSOR_STALE`, and
client cursor-resume/dedup semantics are intentionally unchanged.

## Validation

| Check | Result |
|---|---|
| `pnpm run check:consistency` | pass |
| `git diff --check` | pass |
| `pnpm --filter @cognitiveos/contracts-ts build` | pass |
| `pnpm --filter @cognitiveos/contracts-ts test` | pass, 39/39 |
| Linux exact revision `cargo test -p cognitive-contracts --test schema_contract` | pass, 13/13 |
| Linux exact revision `cargo test -p cognitive-conformance --test runner_execution` | pass, 13/13 |
| Linux exact revision `cargo clippy -p cognitive-contracts --all-targets -- -D warnings` | pass |
| Linux exact revision `cargo fmt --all -- --check` | pass |

Linux validation ran in the approved disposable worktree
`/tmp/cognitiveos-p2-t02-contracts-55774c9` on
`wuz@192.168.1.2`, checked out at the immutable checkpoint above. No secret,
provider, service, privilege, B01 guest, or external Provider operation was
used.

## Remaining work and next action

After PR #138 merges, open a new narrow Lane-RUN lease for the real daemon
composition: authenticated `task.preview` / `task.admit` routing through
`TaskApplicationService`, followed by the already-registered Task watch
endpoint and Rust/TypeScript integration negatives. Do not create parallel
public DTOs or alter the existing watch contract without a new Lane-CTR lease.

## Non-claims

This prerequisite does not implement a daemon route, client request path,
watch server, Task admission execution, scheduler dispatch, Effect, verifier,
Task completion, P2 Gate, release, or Profile result. `P2-T02/D01` is not
done, and B01/B02/B04/B05/B12/GMVP-LINUX statuses are unchanged.
