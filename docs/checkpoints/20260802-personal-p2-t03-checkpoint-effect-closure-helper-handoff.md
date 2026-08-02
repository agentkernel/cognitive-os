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
- Linux-native exact-revision worktree
  `d5e5024b6c130f33f465e934cf88a5a2d354385b`:
  - `cargo test -p cognitive-store --test m5_harness scheduler_ceiling_stop_fences_future_iterations_and_requires_closed_effects`:
    passed (1/1).
  - `cargo test -p cognitive-runtime scheduler_service::tests::maps_each_scheduler_ceiling_to_its_registered_kernel_stop_reason`:
    passed (1/1).
- Focused local test, Clippy, workspace tests, protected CI and P2 Gates:
  not-run.

## Remaining work

- `blocked_paths`: none.
- `blocked_task_ids`: none.
- `blocked_gate_ids`: B02, B04, B05, B12, GMVP-LINUX.
- owner: next P2-T03 Lane-RUN session.
- next action: reload durable ceiling facts at the scheduler worker boundary,
  invoke the runtime adapter before worker lease acquisition, and persist any
  resulting STOP fact before dispatch.

## Non-claims

This repair does not change a public contract, introduce a worker, acquire a
worker lease, dispatch or reconcile an external Effect, qualify BoundedHarness
integration, or produce Gate, release, or Profile evidence.
