# P2-T03 concrete Effect closure and lease release handoff

- Date: 2026-08-03
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/concrete-effect-closure-release` (closed)
- Branch: `main`
- Change class: implementation-only
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: unchanged (`tested-local` from prior Linux-host slices)
- Normative surface: unchanged

## Delivered implementation slice

`complete_durable_scheduler_effect_closure` is the daemon-private worker
boundary that links a leased scheduler dispatch to its exact durable
`TaskBinding`. It rejects a dispatch whose task reference differs from the
binding, obtains the closure disposition only from the durable Effect resolver,
and invokes the scheduler repository only after that disposition is `Closed`.

The repository release uses the dispatch's exact task reference, lease owner,
and lease epoch, records `Succeeded` only for the scheduler attempt, and keeps
independent Task verification separate. A ceiling STOP and pending
reconciliation retain their leases. The release timestamp is parsed before the
durable write, so malformed scheduler timing data is refused.

The focused regression was added before the helper. It uses the real SQLite
scheduler repository to prove that a closed Effect clears only the matching
owner/epoch lease and records scheduler completion without Task acceptance.

## Checks

- Failure-first focused command attempted before implementation:
  `cargo test -p kernel-server scheduler_authority::tests::closed_effect_releases_the_matching_durable_lease_without_completing_the_task`.
  It stopped before crate compilation because the Windows GNU linker returned
  exit 121 while linking dependency build scripts; therefore the failure-first
  result is `not-run`, not a behavioral pass or failure.
- Focused command retried after implementation with the same result: `not-run`
  before crate compilation due to Windows GNU linker exit 121.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- `pnpm run check:consistency`: passed (273 requirements, 55 error codes, 63
  schemas, 85 vectors, links, traceability, Personal plan/Gates, design
  sources, prompt boundary, and leases verified).
- Clippy, complete workspace tests, protected CI, and Linux-native
  exact-revision validation: not-run. Linux validation requires approved SSH
  host-key trust before any non-interactive remote worktree action.

## Remaining work

- `blocked_paths`: approved SSH host-key trust configuration for the qualified
  Linux host; it blocks Linux-native validation only, not local implementation.
- `blocked_task_ids`: none.
- `blocked_gate_ids`: B02, B04, B05, B12 and GMVP-LINUX.
- owner: next P2-T03 Lane-RUN session for BoundedHarness worker integration;
  product owner for the SSH host-key trust configuration.
- next action: integrate the bounded scheduler worker with `BoundedHarness`,
  preserving per-attempt durable contract/lease reload, Effect reconciliation,
  and independent Task verification boundaries.

## Non-claims

No Provider, secret, service, B01 guest, remote host, or external operation
was used. This slice adds no implementation-evidence level, P2 Gate, release,
or Profile claim.
