# P2-T32 public `cognitive daemon start` scheduler lease (running)

- Task: `P2-T32`
- Branch: `personal/P2-T32-public-daemon-start-scheduler`
- Lease: `lease/personal/P2-T32/public-daemon-start-scheduler`
- Change class: `implementation-only`
- Claim ceiling: `hypothesis` / non-claim
- Document status: D01 test authored; D02 launcher wiring landed; D03 handbook
  in the same change set. Ubuntu CI / linux-002 focused `p2_t32_*` is the unique
  next action. Do not merge as C1/C2 proof. Do not freeze or run EVAL-008 here.

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
`pi.json`. `pi_runtime.rs` is unchanged unless CI/linux-002 shows transport
still diverges.

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

| Unit | Status | Note |
|---|---|---|
| D01 `p2_t32_public_daemon_start_scheduler` | `not-run` | Local `RUST-LINK-DEV-WIN-GNU-01`; Windows stub is `cfg(not(unix))`. Unique next proof is Ubuntu CI / `DEV-LINUX-NATIVE-01` focused `p2_t32_*` |
| D02 `daemon_log_path` / `open_daemon_log` unit tests | `not-run` | Authored; local cargo test forbidden. Expect CI to run crate tests |
| D02 launcher stdio + Unix `process_group(0)` | implemented | No `/dev/null` for kernel-server stdout/stderr when a layout exists; start JSON includes `log_path` |
| Local `cargo fmt --all -- --check` | **pass** | this window, Windows GNU eligible |
| Local `pnpm run check:consistency` | **pass** | 275 requirements / leases verified |
| Local `check:handbook` / `generate-handbook --check` | **pass** | 54 docs × 2 locales; 18 generated pages byte-identical |
| Local `docs-sync-gate --staged` | **pass** | `cli-product` mapped; no `DOCS_IMPACT_NONE` |
| Local `git diff --check` | **pass** | this window |
| Ubuntu required CI | `not-run` | after push |
| linux-002 focused `p2_t32_*` | `not-run` | next window; exact pushed revision |

No Gate, release, Profile, B01, EVAL, or Agent-benefit claim.
