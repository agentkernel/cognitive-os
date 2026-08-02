<!--
  P2-T03 operational handoff. PROGRESS.md Current snapshot remains the
  current-status source.
-->

# P2-T03 durable dispatch-closure handoff

- Date: 2026-08-03
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/durable-dispatch-closure` (closed)
- Branch: `main`
- Change class: implementation-only
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: unchanged (`tested-local` from prior Linux-host slices)
- Normative surface: unchanged

## Delivered implementation slice

The daemon-private scheduler authority boundary now permits the exact fenced
dispatch to reach a durable lease release operation only after its Effect
closure is `Closed`. It forwards the original task reference, worker owner and
lease epoch without substitution. A stopped attempt and an attempt awaiting
reconciliation retain their state and bypass release; neither becomes scheduler
or Task success.

The two focused regressions were written before the helper existed. They cover
closed-Effect forwarding once and pending-reconciliation non-release. The
release operation remains responsible for calling the existing
`SchedulerRepository::release_lease` owner-and-epoch CAS; a future worker
integration must bind this boundary to the durable task-to-Effect lookup and
must not use an external receipt as the closure signal.

## Checks

- Failure-first focused command: `cargo test -p kernel-server
  scheduler_authority::tests` was attempted after the regression was added.
  It did not reach crate compilation because Windows GNU dependency build-script
  linking failed with exit 121, before the missing helper could be reported.
- `cargo fmt --all -- --check`: passed after the focused formatting correction.
- `git diff --check`: passed.
- `pnpm run check:consistency`: passed.
- IDE diagnostics for modified files: none.
- Focused Rust test completion, Clippy, workspace tests, protected CI and
  Linux-native exact-revision validation: not-run. The local GNU linker blocks
  Rust execution; Linux-host validation requires an external SSH action and
  was not performed in this session.

## Remaining work

- `blocked_paths`: no code path; Linux-host validation is confirmation-gated.
- `blocked_task_ids`: none.
- `blocked_gate_ids`: B02, B04, B05, B12 and GMVP-LINUX.
- owner: next P2-T03 Lane-RUN session.
- next action: add the durable task-to-Effect lookup and bind the concrete
  worker closure/release call while preserving persist-before-dispatch,
  reconciliation and independent Task verification.

## Non-claims

No Provider, secret, service, B01 guest, remote host or external operation was
used. This slice adds no implementation-evidence level, P2 Gate, release or
Profile claim.
