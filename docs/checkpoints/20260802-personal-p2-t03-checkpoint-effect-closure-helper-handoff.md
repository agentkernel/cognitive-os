# P2-T03 checkpoint Effect-closure helper handoff

- Date: 2026-08-02
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/checkpoint-effect-closure-helper`
- Branch: `main`
- Change class: implementation-only
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: unchanged (`tested-local` from prior slices)
- Normative surface: unchanged

## Delivered implementation slice

Linux-native validation of `6ab740d` revealed that the existing kernel
`LoopDriver::stop_for_ceiling` path called an undefined
`checkpoint_effects_are_closed` helper. This repair restores the private,
fail-closed predicate used by the existing registered
`pending_effects_closed_or_quarantined` guard.

The predicate accepts an absent or empty pending-effect list and accepts a
present list only when every Effect is `RECONCILED`, `VERIFIED`, or
`VERIFY_FAILED`. Invalid JSON, a non-array inventory, an inventory entry with
no recognized state, and every nonterminal state deny the terminal STOP. The
existing focused store regression now covers both a `RECONCILED` inventory
that allows STOP and an `EXECUTING` inventory that prevents it.

## Checks

- `cargo fmt --all -- --check`: passed after formatting.
- `git diff --check`: passed.
- `pnpm run check:consistency`: passed after this handoff was added.
- Focused local and Linux-native kernel/runtime tests, Clippy, workspace tests,
  protected CI and P2 Gates: pending the committed revision.

## Remaining work

- `blocked_paths`: none for source work; Linux-native validation requires the
  committed revision in the existing disposable exact-revision worktree.
- `blocked_task_ids`: none.
- `blocked_gate_ids`: B02, B04, B05, B12, GMVP-LINUX.
- owner: current P2-T03 Lane-KRN session.
- next action: commit and push this repair, update the disposable Linux Git
  worktree to that exact revision, run the focused kernel STOP regression and
  the runtime adapter regression, then record only the resulting evidence.

## Non-claims

This repair does not change a public contract, introduce a worker, acquire a
worker lease, dispatch or reconcile an external Effect, qualify BoundedHarness
integration, or produce Gate, release, or Profile evidence.
