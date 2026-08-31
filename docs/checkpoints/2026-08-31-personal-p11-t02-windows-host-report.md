# P11-T02 Windows host / tray / background — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P11-T02` / slice `P11-T02/D01`
- Branch: `personal/P11-T02-windows-host`
- Lease: `lease/personal/P11-T02/windows-host`
- Change class: `implementation-only`
- Claim commit: `d3229009` (lease/plan only).
- Implementation commit: `71c4824a` (Draft PR [#292](https://github.com/agentkernel/cognitive-os/pull/292)).
- Unique next: wait for `CI-WINDOWS-MSVC-01` + `required-ci` on run [33358224579](https://github.com/agentkernel/cognitive-os/actions/runs/33358224579). Native Windows tray/ACL/sleep/SecretStore E2E stays `not-run`.

| Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|
| store N1 wrong install root | **not-run** | `DEV-WIN-GNU-01` | `71c4824a` | `p11_t02_wrong_install_root_is_rejected`; cargo link forbidden (`RUST-LINK-DEV-WIN-GNU-01`) |
| store N2 ACL escape | **not-run** | `DEV-WIN-GNU-01` | `71c4824a` | `p11_t02_acl_escape_is_rejected`; structural/policy only |
| store N3 raw secret env/argv | **not-run** | `DEV-WIN-GNU-01` | `71c4824a` | `p11_t02_raw_secret_env_argv_is_rejected` |
| store N4 duplicate daemon | **not-run** | `DEV-WIN-GNU-01` | `71c4824a` | `p11_t02_duplicate_daemon_is_rejected` |
| store N5 orphan DSH | **not-run** | `DEV-WIN-GNU-01` | `71c4824a` | `p11_t02_orphan_dsh_is_rejected` |
| store N6 fake background | **not-run** | `DEV-WIN-GNU-01` | `71c4824a` | `p11_t02_fake_background_is_rejected` |
| store N7 restore-as-backup | **not-run** | `DEV-WIN-GNU-01` | `71c4824a` | `p11_t02_restore_as_backup_claim_is_rejected` |
| store N8 secrets not in status | **not-run** | `DEV-WIN-GNU-01` | `71c4824a` | `p11_t02_secrets_are_not_in_status_or_logs` |
| store green upgrade + offline + 7-step recovery | **not-run** | `DEV-WIN-GNU-01` | `71c4824a` | `p11_t02_upgrade_offline_and_ordered_recovery`; skip-step rejected |
| HTTP negatives + task-channel 403 | **not-run** | `DEV-WIN-GNU-01` | `71c4824a` | `p11_t02_host_negatives_and_task_channel_is_forbidden` |
| Native Windows install / tray / ACL / sleep / SecretStore E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` unqualified | — | allowed until qualified; B01-W is not a daily machine |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — | not a product fail |
| `pnpm run check:consistency` | **pass** | `DEV-WIN-GNU-01` | `71c4824a` | 275 requirements; leases verified |
| docs-sync-gate `--staged` | **pass** | `DEV-WIN-GNU-01` | `71c4824a` | handbook 58×2; generate-handbook `--check` 18 pages |
| `check:handbook` | **pass** | `DEV-WIN-GNU-01` | `71c4824a` | coverage/link/fingerprint/status/secret |
| store `p11_t02_windows_host` | **pass** 9/9 | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t02-71c4824a` | `71c4824afd2c465864443eddc2e1b71c6b2fcf59` | wrong root, ACL escape, secret env/argv, duplicate daemon, orphan DSH, fake background, restore-as-backup, secrets-not-in-status, upgrade+offline+7-step |
| kernel-server `p11_t02_host_negatives_and_task_channel_is_forbidden` | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `71c4824afd2c465864443eddc2e1b71c6b2fcf59` | task-channel 403; wrong root 422; secret env 422 without leaking `sk-http`; duplicate daemon 409 |
| store `p1_t01_layout_migrations` | **pass** 8/8 | `DEV-LINUX-NATIVE-01` | `71c4824afd2c465864443eddc2e1b71c6b2fcf59` | authority versions include v34 |
| runtime `p11_t02_host_layout_policy_is_declared_on_install_surface` | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `71c4824afd2c465864443eddc2e1b71c6b2fcf59` | inspectable `install.ps1` Personal Home `app/`/`data/` |
| independent reconfirm store `p11_t02_windows_host` | **pass** 9/9 | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t02-71c4824a` | `71c4824afd2c465864443eddc2e1b71c6b2fcf59` | STORE_EXIT=0; 0.97s; exact pushed SHA |
| independent reconfirm kernel-server `-- p11_t02` | **pass** 1/1 | `DEV-LINUX-NATIVE-01` same worktree | `71c4824afd2c465864443eddc2e1b71c6b2fcf59` | HTTP_EXIT=0; 0.20s; 385 filtered; `p11_t02_host_negatives_and_task_channel_is_forbidden` |
| `resolve validation route` | **SUCCESS** | GitHub Actions [99384354560](https://github.com/agentkernel/cognitive-os/actions/runs/33358224579/job/99384354560) | `71c4824a` | ~2s; run [33358224579](https://github.com/agentkernel/cognitive-os/actions/runs/33358224579) |
| `verify (ubuntu-latest)` | **SUCCESS** | `CI-UBUNTU-01` [99384365895](https://github.com/agentkernel/cognitive-os/actions/runs/33358224579/job/99384365895) | `71c4824a` | ~3m48s |
| `verify (windows-latest)` | in-progress | `CI-WINDOWS-MSVC-01` [99384365882](https://github.com/agentkernel/cognitive-os/actions/runs/33358224579/job/99384365882) | `71c4824a` | started 2026-08-31T04:46:04Z |

## Non-claims

Not Gate, release, Profile, B01, Windows OPC product, Agent-benefit, or tray-icon-as-proof. Not a second credential plane. Not DSH web as host shell. Same-disk restore points are not disaster backups. GNU/WSL/Linux evidence does not transfer to Windows product. T14/T15 remain registered-only until claimed after T02 closes.
