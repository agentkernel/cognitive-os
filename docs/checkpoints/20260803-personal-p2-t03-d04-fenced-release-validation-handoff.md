# P2-T03 D04 fenced release validation handoff

- Date: 2026-08-03
- Task: P2-T03 durable scheduler, lease and timer
- Slice: `P2-T03/D04`
- Revision: `b396d32`
- Change class: focused regression and supported validation
- Task status: `in-progress`
- Slice status: `done`
- Normative surface: unchanged

## Delivered evidence

The daemon-private scheduler closure boundary releases a scheduler lease only
after the exact durable Effect reaches a terminal closure state. The release
uses the leased task reference, owner, and epoch. Reconciliation-pending or
stopped attempts retain their lease and cannot imply Task acceptance.

Focused regressions additionally prove that a malformed release timestamp is
rejected before a durable write and that a closed stale dispatch cannot release
a successor lease after expired-lease takeover. The successor remains leased
under its higher epoch.

## Exact-revision validation

An archive of immutable revision `b396d32` ran in a disposable Linux worktree
on `wuz@192.168.1.2`. No Provider, secret, Pi installation, service-manager
action, B01 guest, or external operation was used.

- `cargo test -p kernel-server scheduler_authority --locked`: passed, 13/13.
- `cargo test -p cognitive-runtime --test p2_t03_scheduler_lease_timer --locked`:
  passed, 6/6.
- `cargo clippy -p kernel-server --all-targets --locked -- -D warnings`:
  passed.
- `cargo fmt --all -- --check`: passed.
- Required Ubuntu and Windows CI: passed.
- `pnpm --filter @cognitiveos/repo-tools test`: passed locally, 5/5.

## Remaining work and non-claims

`P2-T03/D05` is ready and remains the only next P2-T03 delivery slice. It must
connect the scheduler to a restart-safe BoundedHarness worker; it must not add
another helper-only path. B02, B04, B05, B12, release, GMVP-LINUX, and Profile
remain not-run or incomplete.
