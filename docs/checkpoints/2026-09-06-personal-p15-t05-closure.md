# P15-T05 Settings / Model Connections vs v9 — closure

- Task: `P15-T05` **done** / slices `P15-T05/D01` **done** + `P15-T05/D02` **done**
- Change class: `implementation-only` (Owner-facing Settings / Model Connections vs frozen v9). P14-T07 L1 / SecretStore handover stays. No fake Connect. Raw secret not in DOM. 9×9 hidden. No `core/specs`. No numbered migration. Canvas v9 file not overwritten. AXIOMS.md not rewritten. No `app.css` shell tokens.
- Lease: `lease/personal/P15-T05/settings-v9` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P15-T05-settings-v9` → Draft PR [#340](https://github.com/agentkernel/cognitive-os/pull/340) (ready/merge after this HEAD's required CI)
- Dual Track / guest pin: `7b93bfc5` walk
- Guest `/ui/` pin: `7b93bfc5` / SPA `index-DyIoQWPe.js` + `index-C9SPnPVx.css`
- Running report: [P15-T05 report](2026-09-06-personal-p15-t05-report.md)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` close "Settings Owner copy toward v9" only). Not Gate / release / Profile / B01. Windows native daemon chrome JOURNEY **not-run**.
- Evaluation routing: **OFF**. P14-T07 remains **done**. Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334). Do not implement T06 from this close.

## 1. Acceptance mapping (formal plan P15-T05 card + D01/D02)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| Settings / Model Connections vs v9 Owner copy | PageHeader `设置`; lede `连接模型 · 收回本周不再问 · 通知恢复`; hub `模型连接` / `供应商模板` / `密钥（一次性交接）` / `交接密钥到 SecretStore`; `通知与恢复`; `本周不再问` | Dual Track N2 English `Settings` / `Hand key to SecretStore` / Vite lecture | Dual Track N2 **fail** then **pass**; guest J8 title `设置`; hub matches; password `api_key` |
| L1 stays `role=link`; 9×9 hidden; no fake Connect; raw secret not in DOM | Settings L1 unchanged; state-lab `<details>` closed; submit is SecretStore handover; typed key never enters `textContent` | Dual Track N3 fake Connect / secret in DOM; J12 9×9 dump | N3 **fail** then **pass**; guest J8/J12 0 Connect; no `data-state-lab-cell`; no `ss://provider/` |
| Unauthenticated fail-closed; no fake Activate; no Twitter/X P0 | SessionGate; 0 Activate; twitter=false | Dual Track N1 | N1 **pass**; guest J0 Session denied / Open Session; no `opc-settings` |
| `JOURNEY-BROWSER-SYNC-01` J8/J12 + regression | exact-revision guest daemon `/ui/` on `B01-Desktop-Linux-002` | Vite origin; `:48181` replace | J8 **pass**; J12 **pass**; J0/J2/J3/J5 **pass**. Left `:48181` untouched (PID 166715) |

Formal-plan 关闭门: Settings/Model Connections 视觉对 v9 — **true**; L1 link — **true**; 9×9 隐藏 — **true**.

Drift negatives: 假 Connect — Dual Track N3 + guest 0 Connect. raw secret 进 DOM — Dual Track N3 typed `sk-live-must-never-render` absent from `textContent`. state-lab 一级 — guest J12 no cells / details closed. Vite as product origin — walk used daemon `/ui/` only. Canvas v9 not overwritten. AXIOMS.md not rewritten. P14-T07 functional card not reopened.

## 2. Validation summary

| Environment | Result |
|---|---|
| Local Node Dual Track | Focused Settings pack **24/24**. Full `clients/pc/web` **559/559** (79 files) at `7b93bfc5`. Development evidence only. |
| `DEV-WIN-GNU-01` | Rust link **not-run** (routed). No Rust files in T05. |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | This closure commit re-runs required CI before ready/merge. Implementation HEAD `7b93bfc5` + walk record `a1da9c47` superseded by this HEAD. |
| `DEV-LINUX-NATIVE-01` | Exact-revision UI dist `index-DyIoQWPe.js` at `7b93bfc5`. No kernel-server rebuild (UI-only). |
| `B01-Desktop-Linux-002` | Daemon `:48681` PID **2740931** pin `7b93bfc5`. Left `:48181` untouched (PID **166715**). `cognitive dsh web` `:3080` **not-run**. |
| `DEV-WINDOWS-NATIVE-OPC-01` chrome | **not-run** (walk used host Chrome 140 against forwarded guest `/ui/`) |

## 3. Non-claims

Not T06 layout tokens. Not Gate / release / Profile / B01. No Vite origin. No secret in Git/DOM/report. No numbered migration. Canvas v9 not overwritten. AXIOMS.md not rewritten. P14-T07 stays **done**. SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334) not merged. No fake Connect.

## 4. Unique next

Ready/merge PR [#340](https://github.com/agentkernel/cognitive-os/pull/340) after this closure is on the same required-CI HEAD. Unique next after T05 close: **claim `P15-T06`** (remaining layout tokens 176/22/44/~1100). Do not auto-claim P6. `P7-T07` stays blocked. Evaluation routing OFF.
