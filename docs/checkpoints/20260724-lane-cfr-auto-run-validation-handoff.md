# Lane-CFR auto-run validation handoff

- Date: 2026-07-24
- Branch: `lane/cfr-auto-run-validation`
- Base: `origin/main@a5e179caacb31b4971df47429c160485f3672d3c`
- Scope: validation-tool correction and evidence report; no business crate,
  schema, registry or vector change.

## Completed

- Reproduced the L0 auto-run failure: an explicit isolated
  `CARGO_TARGET_DIR` built successfully, but the binary resolver only searched
  `<repo>/target`.
- Refreshed pins to 85 total / 60 pass / 25 not-run / 0 fail and self-check 41.
- Resolved kernel-server from default, absolute or repository-relative targets.
- Re-ran `pnpm run verify:local` with the isolated target: run
  `20260724-011357-998fe139` ended L3, `stopped=false`,
  `non_claim_preserved`.
- Added the full validation/product analysis and synchronized PROGRESS.

## Validation

Full Rust workspace test/build/strict clippy/format/diff passed; TypeScript
locked build/tests passed; runner is 85 / 60 / 25 / 0 and self-check 41/41;
consistency and matrix passed; full auto-run L3 passed.

## Remaining boundaries

Profile implemented is 0. Windows-native sandbox is unsupported, WSL2 is not
tested, durable installation is an in-process-ledger non-claim, and neither the
PERF-004 hardware campaign nor PERF-005 benefit study has run.

## Next entry

Lane-RUN should implement configured server startup with a validated privileged
management session and `SqliteAuthorityStore`, while preserving fail-closed
unconfigured management endpoints. Do not claim wire registration or Profile
compliance before the necessary machine contracts and behavior vectors exist.
