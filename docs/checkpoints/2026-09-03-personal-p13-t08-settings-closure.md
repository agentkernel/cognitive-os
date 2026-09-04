# P13-T08 Settings completeness — closure

- Task: `P13-T08` **done** / slices `P13-T08/D01` **done** + `P13-T08/D02` **done**
- Change class: `implementation-only` (Settings Model Connections write path + notification/diagnostics/state-lab chrome; no `core/specs`, no axiom/contract weaken)
- Lease: `lease/personal/P13-T08/settings-connections` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P13-T08-settings` (worktree `D:\agent-kernel-wt-P13-T08`) → PR [#317](https://github.com/agentkernel/cognitive-os/pull/317)
- Fold HEAD: `b6bc5ea8` (parents `10c0ae04` + `origin/main@2217722d` after T06 #316 + T10 #318)
- Required CI: [33756778043](https://github.com/agentkernel/cognitive-os/actions/runs/33756778043) **SUCCESS** at `b6bc5ea8` (resolve 2s, ubuntu 4m18s, windows 14m50s, required-ci 3s)
- Live SecretStore: **pass 12/12** at pushed `8b71fb8d` on `DEV-LINUX-NATIVE-01` (`settings_connections.rs` unchanged by the fold)
- Linux focused cargo at fold SHA: **8/8** on `DEV-LINUX-NATIVE-01` worktree `~/cognitiveos-personal-worktrees/p13-t08-b6bc5ea8` (`HEAD=b6bc5ea8`, dirty=0)
- Running report: [P13-T08 report](2026-09-03-personal-p13-t08-settings-report.md)
- Claim ceiling: `hypothesis`. Linux SecretStore + ordinary CI close "the implementation exists" only. Windows SecretStore host E2E stays **not-run** until `P13-T13`. Not Gate / release / Profile / B01 / Windows support.

## 1. Acceptance mapping

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| OPC Settings Model Connections: template + custom URL/compat/key/model | `settings_connections.rs` `POST /management/settings/v1/connection.connect`; web `SettingsPage.tsx` | keyless / blank key 400; custom without URL 400; task alias 403 | Dual Track TS; Linux HTTP L3–L5; cargo 8/8 |
| SecretStore takeover; connected/failed; no raw secret | store-receipt after PCP (`8b71fb8d`); envelope `secret=present/absent` only | connect never echoes key; live L10 0 key hits | live 12/12 at `8b71fb8d`; failure-first receipt test |
| No `/providers` detour | Settings form posts connect; no `#/providers` link | Dual Track forbids the detour and "Open Providers" | Dual Track TS |
| usage actual / estimated / unknown≠0 | `connectionUsage.ts` | unknown never serializes as 0 | Dual Track `connectionUsage` + Settings table |
| Notification groups missed / offline / resume | `GET /management/settings/v1/notifications`; empty groups when no home | honest-empty without home | cargo + Dual Track + live L9 |
| Advanced diagnostics default-collapsed; honest empty if P13-T02 facts absent | `GET /management/settings/v1/diagnostics`; `<details>` closed | honest-empty DSH/Pi | cargo + Dual Track + live L8 |
| state-lab 9×9 real components; Settings advanced hidden; not L1 | `StateLabPage.tsx` 81 cells; no `#/state-lab` nav | hidden until Advanced opened | Dual Track 81 cells |

## 2. Non-claims

Windows SecretStore / proxy host E2E = `P13-T13` / `not-run`. No fake Connect. state-lab is not L1. A1 daemon-only writer. A5 Secret Store only. Not T07/T09/T11/T12/T13/T15. T09 owns additive lifecycle handbook rows; this delivery kept T08 Settings + T06 chat rows and did not add `project_lifecycle.rs`.

## 3. Unique next

Ready/merge PR #317 after required CI on this closure HEAD. Do not claim T09 / T11 / T12 / T13 / T15. Do not merge at the same instant as T07 #319 / T09 #321 / T11 #320.
