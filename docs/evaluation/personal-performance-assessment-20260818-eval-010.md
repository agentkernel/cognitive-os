# PERSONAL-PERF-EVAL-010 assessment (running)

- Campaign: `PERSONAL-PERF-EVAL-010`
- Frozen source target: `289eebade1432fdf224cfe16661fdc102874e416` (P2-T33
  private-candidate host path; unmerged freeze)
- Lease: `lease/personal/EVAL-010/c1-c2-paired-freeze`
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: campaign **active**. Measurement-only.

This is the campaign's single running report (`TEST-REPORT-INCREMENTAL-01`).
Append each finished cell before starting the next.

Owner 2026-08-18 authorized product changes after EVAL-009, then continuing
C1/C2 真机. EVAL-009 remains **closed**. This freeze measures P2-T33 pin
`289eebad` on a **long unique root**. P2-T33 stub pass is not C1/C2
Agent-benefit.

## Cells

| Cell | Status | Note |
|---|---|---|
| EVAL-009 remains closed (coordination) | **pass** | do not reuse `48296` / `/18` / `e009` runtime |
| Evaluation lease claimed | **pass** | `PERSONAL-PERF-EVAL-010` **active** |
| Freeze (archive/binaries/root/port) | **pass** | pin `289eebad`; root `/home/hal9001/perfeval010-20260818` mode `0700`; daemon `127.0.0.1:48298` pid 287493; `log_path` mode `0600` |
| SecretStore import | **pass** | new item `/org/freedesktop/secrets/collection/login/19` via stdin; D-Bus `SearchItems` paths only; never search/lookup |
| Pi 0.81.1 pin | **pass** | `--extension` absolute; doctor ready is **not** C1/C2 |
| Exact-source `pi-agent-adapter` | **pass** | real adapter, not the P2-T33 stub; SHA-256 `70ba7f05d3b743737334186c4b8b3155047cfa5856c4b0e28c45924866095cdb` |
| B0 C1 WorkspaceSearch O-arm | `not-run` | first cell after freeze |
| B0 remaining C1/C2 families | `not-run` | after C1-search path/fairness |
| B0 P-arm / broker `48398` | `not-run` | only if O-arm is fairly measurable |
| B1/B2 C1/C2 paired | `not-run` | after B0 |

## Freeze (2026-08-18) — pass

Exact source `289eebade1432fdf224cfe16661fdc102874e416` (P2-T33 private-candidate
host path; unmerged). Guest root mode `0700`. Listeners `48181` / `48284` /
`48383` untouched. SecretStore item `/19` is new (not `/12`–`/18`). Public
doctor was ready (`first_conversation_ready: true`). That is conversation
readiness, not a C1/C2 Task. Claim ceiling `hypothesis`. No Gate, release,
Profile, B01, or Agent-benefit claim.

Host `DEV-LINUX-NATIVE-01` built release binaries from the extracted `289eebad`
archive with a dedicated `CARGO_TARGET_DIR` (`CARGO_NET_OFFLINE=true`, 1m 37s,
Rust 1.97.1). Windows GNU Rust build remains `not-run`
(`RUST-LINK-DEV-WIN-GNU-01`). Copies used `scp`.

Public start: endpoint `127.0.0.1:48298`, pid `287493`, `log_path`
`/home/hal9001/perfeval010-20260818/runtime/state/cognitiveos/daemon.log`
(mode `0600`). The campaign root is **long** so nested `completion.sock` under
the runtime tree would exceed Linux `UNIX_PATH_MAX` (108); P2-T33 binds the
socket under `$XDG_RUNTIME_DIR` / `temp_dir()` / `/tmp/cognitiveos` instead.

| Asset | Value |
|---|---|
| source archive | 14,735,360 bytes; 1544 entries; 0 `.git/` members; SHA-256 `ccf7e6a1ecba22a55e3a5fe50831f6a182bed3a21b84192d22c5ac7efaac769f` |
| `kernel-server` | 16,573,216 bytes; SHA-256 `a60e1166fa81e09b2b6b2e95892e9daccfc28fd98806f874e01d34502aedf1c5` |
| `cognitive` | 10,313,736 bytes; SHA-256 `6917dca3a0f294c34d1f177dd5ebd3e1a36fff1c71de7661094049b30741a65f` (CLI digest equal to EVAL-008/009; launcher tree unchanged) |
| `pi-agent-adapter` | 1,128,568 bytes; SHA-256 `70ba7f05d3b743737334186c4b8b3155047cfa5856c4b0e28c45924866095cdb` (differs from EVAL-008/009 `816856b4…`) |
| `o-arm-candidate.mjs` | SHA-256 `29870821488451b5728f88c4612e1616fd65681adaf23011dd898d459428e573` |
| `private_candidate_provider.mjs` | SHA-256 `2b7e52a6afe205e5997c58fe59b096fc7666dfd8733e196777e915d3a0bc245b` |
| Pi tarball | SHA-256 `420113c0282160e6181656fd16cf18742f76bf9040ee3dfb9cb67e3e6ad5641c` |

SecretStore import (stdin; D-Bus paths only): item
`/org/freedesktop/secrets/collection/login/19` (1 unlocked, 0 locked).
`login` collection `Items` contained only `/19` during the freeze.
Product report: `secret_backend=linux-secret-tool`,
`secret_material_written=true`, `secret_ref_redacted=true`,
`selected_model=deepseek-v4-flash`, `snapshot_digest=fnv1a64:c58ce6f2f7521544`.
Doctor after configure: provider `secret_ref_resolves=true`,
`secret_material_exposed=false`, Pi `package_status=ready` /
`pinned_version=0.81.1` / `observed_version=0.81.1`,
`first_conversation_ready: true`. Daemon pid 287493 still bound to
`127.0.0.1:48298`.

No Gate, release, Profile, B01, or Agent-benefit claim.
