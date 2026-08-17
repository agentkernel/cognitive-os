# P2-T32 public `cognitive daemon start` scheduler lease (running)

- Task: `P2-T32`
- Branch: `personal/P2-T32-public-daemon-start-scheduler`
- Lease: `lease/personal/P2-T32/public-daemon-start-scheduler` (**closed**
  2026-08-18 so EVAL-008 can own `PROGRESS.md`; task is not `done`)
- Change class: `implementation-only`
- Claim ceiling: `hypothesis` / non-claim
- Document status: D01/D02 landed. Ubuntu `verify` **pass** at `fb85cfff`
  (focused `p2_t32` ok). `DEV-LINUX-NATIVE-01` focused `p2_t32` **pass** 1/1
  in 3.33s plus `daemon_log` unit tests 2/2. Windows `verify` **fail** on
  unrelated flake `server_startup_recovers_closed_effect_before_publishing_endpoint`
  (330 pass / 1 fail); not the public-launcher test. Draft PR [#239](https://github.com/agentkernel/cognitive-os/pull/239).
  Unique next for merge: Windows required-ci green. C1/C2 真机 is EVAL-008,
  not this task merge. Stub-adapter pass ≠ real `pi-agent-adapter`.

Owner 2026-08-18 after `PERSONAL-PERF-EVAL-007` close. EVAL-007 public
`cognitive daemon start --bind 127.0.0.1:48292` admitted over HTTP (200) and the
live daemon scheduler left the Task `DRAFT` (`lease_acquired` 0, no Pi child).
P2-T31's focused test spawned `kernel-server --personal` with piped stderr and
is not this launcher.

## Discriminant

`serve_personal_loopback` already binds the 250 ms periodic tick. The public
CLI previously set kernel-server stdin/stdout/stderr to `/dev/null` and did not
create a Unix process group. Skip class was only `eprintln!("... skip row ...")`,
so EVAL-007 correctly treated it as not a public fact.

D02 retains stdout/stderr on `state/cognitiveos/daemon.log` (mode `0600`) and
applies Unix `process_group(0)` on the CLI spawn path only. Stub Workspace*
adapter is allowed in the focused test; the product CLI does not inject stub
`pi.json`. Ubuntu + linux-002 stub tests **pass**, so `pi_runtime.rs` was not
changed. Real `pi-agent-adapter` + operator `pi.json` remains an EVAL-008
measurement cell (EVAL-007 freeze assets were present and still stayed
`DRAFT`).

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

| Unit | Status | Note |
|---|---|---|
| D01 `p2_t32_public_daemon_start_scheduler` | **pass** | Ubuntu job `95438760229` at `fb85cfff`: `public_cognitive_daemon_start_admit_leaves_draft_until_scheduler_acquires_lease ... ok`. linux-002 (`wuz@192.168.1.2`) extracted archive SHA-256 `202384ee0b125c6600764042ddc7a2142bb1502da21be642b8c328440325ced3` of `fb85cfff`: focused test **1/1 in 3.33s**. Stub Workspace* adapter, not real `pi-agent-adapter`. |
| D02 `daemon_log_path` / `open_daemon_log` unit tests | **pass** | linux-002: 2/2 (`daemon_log_lives_under_layout_state_dir`, `open_daemon_log_creates_an_append_file_under_state_dir` including Unix mode 0600) |
| D02 launcher stdio + Unix `process_group(0)` | implemented | No `/dev/null` for kernel-server stdout/stderr when a layout exists; start JSON includes `log_path` |
| Local `cargo fmt --all -- --check` | **pass** | this window, Windows GNU eligible |
| Local `pnpm run check:consistency` | **pass** | 275 requirements / leases verified |
| Local `check:handbook` / `generate-handbook --check` | **pass** | 54 docs × 2 locales; 18 generated pages byte-identical |
| Local `docs-sync-gate --staged` | **pass** | `cli-product` mapped; no `DOCS_IMPACT_NONE` |
| Local `git diff --check` | **pass** | this window |
| Ubuntu required CI | **pass** | run `32047487272` `verify (ubuntu-latest)` at `fb85cfff`; Rust workspace + Clippy + handbook |
| linux-002 focused `p2_t32_*` | **pass** | exact archive of `fb85cfff`; Rust 1.97.1; 1/1 + daemon_log 2/2 |
| Windows MSVC CI | **fail** | same run `32047487272`: `p2_t32` Windows stub **ok**; fail is unrelated `server_startup_recovers_closed_effect_before_publishing_endpoint` (endpoint publish race). 330 passed / 1 failed. Not a C1/C2 public-launcher skip. |

No Gate, release, Profile, B01, EVAL, or Agent-benefit claim.
