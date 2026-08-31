# P11-T02 Windows host / tray / background — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P11-T02` / slice `P11-T02/D01`
- Branch: `personal/P11-T02-windows-host`
- Lease: `lease/personal/P11-T02/windows-host`
- Change class: `implementation-only`
- Claim commit: `d3229009` (lease/plan only).
- Draft PR: [#292](https://github.com/agentkernel/cognitive-os/pull/292)
- Unique next: push implementation checkpoint; prove store/HTTP on `CI-UBUNTU-01` + `CI-WINDOWS-MSVC-01` (and Linux native if available). Native Windows tray/ACL/sleep/SecretStore E2E stays `not-run`.

| Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|
| store N1 wrong install root | **not-run** | `DEV-WIN-GNU-01` | uncommitted D01 | `p11_t02_wrong_install_root_is_rejected`; cargo link forbidden (`RUST-LINK-DEV-WIN-GNU-01`) |
| store N2 ACL escape | **not-run** | `DEV-WIN-GNU-01` | uncommitted D01 | `p11_t02_acl_escape_is_rejected`; structural/policy only |
| store N3 raw secret env/argv | **not-run** | `DEV-WIN-GNU-01` | uncommitted D01 | `p11_t02_raw_secret_env_argv_is_rejected` |
| store N4 duplicate daemon | **not-run** | `DEV-WIN-GNU-01` | uncommitted D01 | `p11_t02_duplicate_daemon_is_rejected` |
| store N5 orphan DSH | **not-run** | `DEV-WIN-GNU-01` | uncommitted D01 | `p11_t02_orphan_dsh_is_rejected` |
| store N6 fake background | **not-run** | `DEV-WIN-GNU-01` | uncommitted D01 | `p11_t02_fake_background_is_rejected` |
| store N7 restore-as-backup | **not-run** | `DEV-WIN-GNU-01` | uncommitted D01 | `p11_t02_restore_as_backup_claim_is_rejected` |
| store N8 secrets not in status | **not-run** | `DEV-WIN-GNU-01` | uncommitted D01 | `p11_t02_secrets_are_not_in_status_or_logs` |
| store green upgrade + offline + 7-step recovery | **not-run** | `DEV-WIN-GNU-01` | uncommitted D01 | `p11_t02_upgrade_offline_and_ordered_recovery`; skip-step rejected |
| HTTP negatives + task-channel 403 | **not-run** | `DEV-WIN-GNU-01` | uncommitted D01 | `p11_t02_host_negatives_and_task_channel_is_forbidden` |
| Native Windows install / tray / ACL / sleep / SecretStore E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` unqualified | — | allowed until qualified; B01-W is not a daily machine |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — | not a product fail |
| `pnpm run check:consistency` | pending | `DEV-WIN-GNU-01` | uncommitted D01 | after handbook fingerprints |
| docs-sync-gate `--staged` | pending | `DEV-WIN-GNU-01` | uncommitted D01 | after stage |

## Non-claims

Not Gate, release, Profile, B01, Windows OPC product, Agent-benefit, or tray-icon-as-proof. Not a second credential plane. Not DSH web as host shell. Same-disk restore points are not disaster backups. GNU/WSL/Linux evidence does not transfer to Windows product. T14/T15 remain registered-only until claimed after T02 closes.
