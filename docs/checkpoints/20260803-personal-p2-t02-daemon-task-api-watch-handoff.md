# P2-T02 D01 daemon Task API/watch closure handoff

- Date: 2026-08-03
- Task and slice: `P2-T02/D01`
- Change class: implementation-only
- Lease: `lease/personal/P2-T02/daemon-task-api-watch`, closed in the PR #141
  closure delivery
- Status: slice `done`; parent `P2-T02` remains `in-progress`

## Recovery tuple

| Field | Value |
|---|---|
| Branch | `lane/run-p2-t02-daemon-task-api-watch` |
| Immutable validated checkpoint | `734cbce19f5dba474ea2bd6ffd1db7d6a30cb951` |
| Upstream | `origin/lane/run-p2-t02-daemon-task-api-watch` at the validated checkpoint |
| Pull request | [#141](https://github.com/agentkernel/cognitive-os/pull/141), Draft at documentation closure |
| Worktree | closure documentation changes pending commit |

## Delivered slice

The loopback Personal daemon now routes generated Task bindings through the
existing authoritative `TaskApplicationService`:

- authenticated `intent.record`, `intent.interpret`, `preview`, and `admit`;
- daemon-created, durable, principal-bound governance root; the client does
  not supply governance anchors or writer authority facts;
- server-side `WriterLease` for every mutating operation;
- admission verifies `accepted_by` against the authenticated principal and
  delegates preview-digest recomputation and epoch CAS to the application
  service;
- snapshot-first `/task/watch` with bounded cursor resume/dedup and an
  explicit stale-cursor failure.

Watch state is process-lifetime only. A daemon restart intentionally has no
durable replay claim. Preview is non-persistent, and admit does not claim
dispatch, Effect execution, verification, or Task completion.

## Validation

| Check | Result |
|---|---|
| `cargo fmt --all` | pass locally; non-linking Windows allowlist command |
| `git diff --check` before implementation checkpoint | pass |
| focused daemon process test on exact Linux revision | pass, 1/1: `cargo test -p kernel-server --test p2_t02_task_api_watch` |
| required CI Ubuntu | pass |
| required CI Windows | pass |
| local Windows GNU Rust build/test/Clippy | not-run; prohibited by `RUST-LINK-DEV-WIN-GNU-01` |
| B02/B04/B05/B12 | not-run |

Linux validation used a disposable Git clone at
`/tmp/cognitiveos-p2-t02-734cbce-shallow/repository` on
`personal-linux-native-01`, checked out from the remote task branch and
verified at `734cbce19f5dba474ea2bd6ffd1db7d6a30cb951`. No secret, Provider,
privilege, service-manager, B01 guest, or release operation was used.

## Remaining work and next action

`P2-T02` still requires its private versioned projections and deterministic
CLI/Shell-through-sidecar parity. Claim a new exact-path lease before starting
that bounded slice. Do not re-open D01 merely to claim cross-restart watch
replay, dispatch, Effect, verification, Task completion, a P2 Gate, release,
or Profile conformance.

