# P1-T09 Windows Provider fixture timeout repair handoff

## Record metadata

- record_type: historical-handoff
- project_id: cognitiveos-personal
- task_id: P1-T09
- lease_id: `lease/personal/P1-T09/windows-provider-fixture-timeout` (closed)
- status_at_handoff: in-progress
- gate_status_at_handoff: B01 running
- claim_scope_at_handoff: non-claim
- current_status_source: `docs/plan/PROGRESS.md` Current snapshot
- Date: 2026-08-02
- Classification: implementation-only; normative surface unchanged
- Implementation commit: `035e5c9`
- Branch: `lane/personal-p1-t09-windows-fixture-timeout`
- Pull request: #132

## Completed slice

The Windows job in CI run `30753436524` previously failed when the deterministic
Provider fixture's ready-path discovery used a 500 ms exchange timeout. The
failure was observed before this repair as
`ProviderTransportError::Timeout` in
`binary_fixture_drives_real_rustls_discovery_without_leaking_provider_material`.

Commit `035e5c9` makes only the ready-path integration test more tolerant of
Windows CI scheduling: its bounded total discovery budget is 10 seconds and
each exchange deadline is 1.5 seconds. The independently focused timeout test
continues to use its explicit 50 ms deadline, so timeout behavior remains
covered. Oversize, redirect, secret-isolation, fixture loopback-only, and
authority non-side-effect assertions remain unchanged.

## Verification

- `git diff --check` - passed.
- `cargo fmt --all -- --check` - passed.
- `pnpm run check:consistency` - passed.
- Local focused fixture command - not-run to completion: the Windows GNU
  linker exited 121 while linking dependencies before the test compiled.
- GitHub Actions branch workflow `30754142573` - Ubuntu passed in 1m44s and
  Windows passed in 5m29s.
- GitHub Actions PR workflow `30754184257` - Ubuntu passed in 1m51s and
  Windows passed in 5m15s.

## Status and non-claims

- P1-T09 remains `in-progress` with existing `tested-supported-ci` fixture
  evidence; this repair does not advance its evidence level.
- B01 remains `running` at attempt 1 of fixed N=20. This repair neither starts
  an attempt nor changes campaign accounting, thresholds, or verifier work.
- GMVP-LINUX remains `not-run`; Profile conformance remains `implemented: 0`.
- No provider, user secret, Pi load, remote host, B01 guest, service change,
  Task, Effect, Verification, capability, or authority writer was used.

## Next safe action

Continue P1-T09 through preregistered B01 attempt 2 only under its established
campaign procedure. Do not edit, delete, renumber, replace, or rerun attempt 1.
