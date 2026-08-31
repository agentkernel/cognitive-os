# P11-T02 Windows host / tray / background — closure

- Task: `P11-T02` / slice `P11-T02/D01` (full Phase 11 T02 acceptance)
- Change class: `implementation-only` (v34 Personal Home `app/`/`data/`, typed lifecycle, close honesty, missed segments, ordered seven-step recovery, restore-point-not-backup; no second credential plane; not DSH web as host shell; not chrome)
- Branch: `personal/P11-T02-windows-host`
- Linux native focused HEAD: `71c4824afd2c465864443eddc2e1b71c6b2fcf59`
- Required-CI / PR head: `19300b92`
- Merge revision: `main@cb66c7fb5d2fa8f2821f373bdb2c9ae91b40a438`
- Pull request: [#292](https://github.com/agentkernel/cognitive-os/pull/292) (merged 2026-08-31)
- Lease: `lease/personal/P11-T02/windows-host` (closed into PARALLEL-LANES §3.1 by this ledger)
- Required CI on PR head `19300b92`: **SUCCESS** — run [33358661063](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063): `resolve validation route` **SUCCESS** [99385648722](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063/job/99385648722) ~4s, `verify (ubuntu-latest)` **SUCCESS** [99385666462](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063/job/99385666462) ~3m45s, `verify (windows-latest)` **SUCCESS** [99385666475](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063/job/99385666475) ~15m23s, `required-ci` **SUCCESS** [99388098195](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063/job/99388098195). Incremental log: [report](2026-08-31-personal-p11-t02-windows-host-report.md)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## Acceptance mapping

D01 covers the T02 close gate: inspectable Personal Home `app/`/`data/`, daemon/tray typed lifecycle (observe/request only), close background-or-pause honesty, explicit offline/missed segments, ordered seven-step wake/restart with resume-only-eligible, restore-point-not-backup, and fail-closed install/ACL/secret/process negatives. Native Windows install/tray/ACL/sleep/SecretStore E2E, B01-W, and `DEV-WIN-GNU-01` cargo remain honest **not-run**. Linux store **9/9** and HTTP **1/1** at `71c4824a` are **pass**. Workspace `required-ci` on `19300b92` is **SUCCESS**.

| Acceptance item | Evidence |
|---|---|
| Wrong install root rejected | store N1 `p11_t02_wrong_install_root_is_rejected`; HTTP 422 |
| ACL escape rejected (structural/policy) | store N2 `p11_t02_acl_escape_is_rejected`; native ACL E2E **not-run** |
| Raw secret env/argv rejected | store N3 `p11_t02_raw_secret_env_argv_is_rejected`; HTTP 422 without leaking `sk-http` |
| Duplicate daemon rejected | store N4 `p11_t02_duplicate_daemon_is_rejected`; HTTP 409; task XML `IgnoreNew` |
| Orphan DSH rejected | store N5 `p11_t02_orphan_dsh_is_rejected` |
| Fake background rejected | store N6 `p11_t02_fake_background_is_rejected` |
| Restore-as-backup rejected | store N7 `p11_t02_restore_as_backup_claim_is_rejected` |
| Secrets not in status/logs | store N8 `p11_t02_secrets_are_not_in_status_or_logs` |
| Upgrade preserves data; 7-step recovery; skip-step rejected | store `p11_t02_upgrade_offline_and_ordered_recovery` |
| Task-channel mutation fail-closed | HTTP `WINDOWS_HOST_CHANNEL_FORBIDDEN` 403 |
| Linux store T02 focused negatives + green path | **pass** **9/9** at `71c4824a` (`DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t02-71c4824a`); independent reconfirm STORE_EXIT=0 |
| Linux HTTP negatives + task-channel | **pass** **1/1** at `71c4824a`; independent reconfirm HTTP_EXIT=0 |
| Native Windows install / tray / ACL / sleep / SecretStore E2E | **not-run** (`DEV-WINDOWS-NATIVE-OPC-01` unqualified; card allows) |
| B01-W campaign guest | **not-run** (evaluation routing OFF; B01-W is not a daily machine) |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |
| Workspace `required-ci` on `19300b92` | **SUCCESS** run [33358661063](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063) |

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| store N1–N8 + upgrade/offline/7-step | **pass** 9/9 | `DEV-LINUX-NATIVE-01` | `71c4824afd2c465864443eddc2e1b71c6b2fcf59` |
| kernel-server `p11_t02` HTTP negatives + task-channel 403 | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `71c4824afd2c465864443eddc2e1b71c6b2fcf59` |
| layout migrations v34 | **pass** 8/8 | `DEV-LINUX-NATIVE-01` | `71c4824afd2c465864443eddc2e1b71c6b2fcf59` |
| inspectable install-surface policy | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `71c4824afd2c465864443eddc2e1b71c6b2fcf59` |
| Native Windows install/tray/sleep/SecretStore E2E | **not-run** | unqualified | `71c4824a` |
| B01-W guest | **not-run** | evaluation routing OFF | `71c4824a` |
| Rust link on `DEV-WIN-GNU-01` | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | `19300b92` |
| `verify (ubuntu-latest)` on `19300b92` | **SUCCESS** [99385666462](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063/job/99385666462) | `CI-UBUNTU-01` | `19300b92` |
| `verify (windows-latest)` on `19300b92` | **SUCCESS** [99385666475](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063/job/99385666475) | `CI-WINDOWS-MSVC-01` | `19300b92` |
| `required-ci` on PR head `19300b92` | **SUCCESS** [99388098195](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063/job/99388098195) | GitHub Actions run [33358661063](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063) | `19300b92` |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC product qualification, Agent-benefit, or tray-icon-as-proof (A7: local/CI is hypothesis only). Not a second credential plane. Not DSH web as host shell. Same-disk restore points are not disaster backups. GNU/WSL/Linux evidence does not transfer to Windows product. Native install/tray/ACL/sleep/SecretStore E2E **not-run**. Evaluation routing OFF.

## Deterministic closure

1. Linux native focused **pass** at `71c4824a` (store 9/9, HTTP 1/1, layout 8/8, install-surface 1/1);
2. required CI [33358661063](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063) **SUCCESS** on `19300b92`;
3. PR [#292](https://github.com/agentkernel/cognitive-os/pull/292) merged as `main@cb66c7fb` on 2026-08-31;
4. lease `lease/personal/P11-T02/windows-host` moved to §3.1;
5. remote `personal/P11-T02-windows-host` deleted by GitHub after merge.

Unique next: claim `P11-T14/D01`. This file does **not** claim `lease/personal/P11-T14/*`.
