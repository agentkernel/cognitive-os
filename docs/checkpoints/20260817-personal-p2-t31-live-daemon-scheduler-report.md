# P2-T31 live-daemon HTTP scheduler lease (running)

- Task: `P2-T31`
- Branch: `personal/P2-T31-live-daemon-scheduler-lease`
- Lease: `lease/personal/P2-T31/live-daemon-scheduler-lease`
- Change class: `implementation-only`
- Claim ceiling: `hypothesis` / non-claim
- Document status: D01–D03 complete; merged PR #236 at `main@da6fca2e`

Owner 2026-08-17 after `PERSONAL-PERF-EVAL-006` close. EVAL-006 B0 on
`main@103fe776` admitted `task://local/eval006-b0-C1-search-b0-0-071b35428873`
over public HTTP (200) and the live daemon scheduler left it `DRAFT` for
180 s (`lease_acquired` 0/0, no Pi child). P2-T30's focused test used
`TaskApi::handle` plus `DeterministicProductionChainProposer` and is not
this path.

## Root cause

Public `POST /task/admit` persisted Context authorization on a **second**
`SqliteAuthorityStore` connection opened per request. The periodic worker
kept the daemon-owned handle opened at startup. That is the incomplete
P9-T03 reuse: request handlers must share the single writer. Live Pi also
requires configured Unix private-candidate transport (`pi.json` + adapter +
selected-model); the focused test installs a stub adapter (not bash/edit/write).

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

| Unit | Status | Note |
|---|---|---|
| D01 live-daemon HTTP admit skip | **pass** (failure-first) | linux-002 at `ee353679`: Task stayed DRAFT, `lease_acquired` 0, empty skip lines — first tick blocked ~65 s on unused Provider completion socket |
| D02 shared store + stdout stub + first-dispatch retry | **pass** | linux-002 at `5d073398`: focused `p2_t31_live_daemon_scheduler` 1/1 in 3.20s — left DRAFT, `lease_acquired` ≥ 1 |
| D03 `cargo fmt --all -- --check` | **pass** | linux-002 at `63ab512a` (also `5d073398`) |
| D03 workspace tests | **pass** | linux-002 at `5d073398`: 0 failed |
| D03 Clippy `-D warnings` | **pass** | linux-002 at `63ab512a` |
| D03 kernel-server `--bins` | **pass** | linux-002 at `63ab512a`: 337/337 |
| D03 focused live HTTP | **pass** | linux-002 at `63ab512a`: 1/1 in 3.20s |
| Ubuntu CI | **pass** | run `32038273275` verify ubuntu-latest at `49cdfc0c` |
| Windows MSVC CI | **pass** | run `32038273275` verify windows-latest + `required-ci` at `49cdfc0c` |
| Windows GNU cargo | `not-run` | `RUST-LINK-DEV-WIN-GNU-01`; live test `cfg(unix)`; extra native GNU not-run by Linux-only route |

No Gate, release, Profile, B01, EVAL, or Agent-benefit claim.
