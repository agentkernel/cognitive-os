# P2-T03 daemon ceiling-dispatch admission handoff

- Date: 2026-08-02
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/daemon-ceiling-dispatch-admission`
- Branch: `main`
- Change class: implementation-only
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: `tested-local`
- Normative surface: unchanged

## Delivered implementation slice

`admit_scheduler_dispatch` is the daemon-owned ordering boundary for a
scheduler attempt. It reloads the current TaskContract, loop, progress and
budget facts into a `SchedulerAuthoritySnapshot`, invokes the fenced runtime
ceiling STOP adapter, and only invokes `SchedulerRepository::claim_eligible`
when the result is `Proceed`. A reached ceiling returns the committed STOP
transition; no worker lease, worker process, or external Effect dispatch is
attempted.

The immutable contract supplies the loop and budget identities used by the
kernel transition. Legacy, malformed, unavailable or non-dispatchable
authority facts continue to fail closed before either STOP or lease activity.

## Checks

- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- Linux-native exact-revision worktree
  `a3cbea10eb7c6ba4a6dcf8a9399ba97effd2e38d`:
  `cargo test -p kernel-server scheduler_authority::tests` passed (2/2).
  This compilation covers the new daemon composition boundary; it retains the
  existing legacy/incomplete contract negative tests.
- Local Windows focused test: not-run to completion; the GNU linker exited
  121 while linking dependency build scripts.
- Admission-order end-to-end regression, Clippy, workspace tests, protected
  CI and P2 Gates: not-run.

## Remaining work

- `blocked_paths`: none.
- `blocked_task_ids`: none.
- `blocked_gate_ids`: B02, B04, B05, B12, GMVP-LINUX.
- owner: next P2-T03 Lane-RUN session.
- next action: add an end-to-end daemon admission regression that proves a
  reached durable ceiling commits STOP with scheduler `attempt_count` still
  unchanged, while a clear snapshot acquires exactly one fenced lease. Then
  begin the separate worker/Effect closure integration slice.

## Non-claims

This slice does not create a worker process, dispatch an external operation,
close or reconcile an Effect, qualify BoundedHarness integration, or produce
Gate, release, or Profile evidence.
