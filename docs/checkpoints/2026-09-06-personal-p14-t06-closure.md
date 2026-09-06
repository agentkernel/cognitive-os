# P14-T06 Today live packets — closure

- Task: `P14-T06` **done** / slices `P14-T06/D01` **done** + `P14-T06/D02` **done**
- Change class: `implementation-only` (Today after activation is decision packets + one row per live Project, not continue-create; no `core/specs`; no numbered migration)
- Lease: `lease/personal/P14-T06/today-packets` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P14-T06-today-packets` → Draft PR [#333](https://github.com/agentkernel/cognitive-os/pull/333) (ready/merge in this close)
- Implementation revision: D01 Dual Track then fold T04 `3012db11` (required CI [33991080971](https://github.com/agentkernel/cognitive-os/actions/runs/33991080971) **SUCCESS**). Fold `origin/main@d9cee39d` (T05 #332) then D02 heartbeat `01004a49`.
- Guest `/ui/` pin: `01004a49` / SPA `index-05YCvTci.js`
- Running report: [P14-T06 report](2026-09-06-personal-p14-t06-report.md)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` close "Today after activation is live packets + per-Project overview" only). Not Gate / release / Profile / B01. Windows native chrome JOURNEY **not-run**.
- Evaluation routing: **OFF**. T01–T05 / T07 / T08 remain **done**. Do not claim T07/T08.

## 1. Acceptance mapping (formal plan P14-T06 card + D01/D02)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| After activation, Today = decision packets + one row per live Project (not continue-create) | `TodayPage` + `loadTodayLiveSurface`: per-live `GET /management/project/v1/pending-previews?subject_ref=`; leftover `creating` drafts stay `opc-today-leftover-drafts`; overview one row per live id | leftover drafts keep Continue create; packets only for the first live id | Dual Track `todayLivePackets` fail-then-pass; guest `#/` `data-surface=today`, 4 live titled rows, leftover-drafts honesty, packet collapsed, week period switch |
| EVAL-016 J2 + `JOURNEY-BROWSER-SYNC-01` | exact-revision guest daemon `/ui/` on `B01-Desktop-Linux-002` pin `01004a49` / SPA `index-05YCvTci.js` | KPI wall; unactivated still painting live packets; T13 empty chrome as packet acceptance | J2 **pass**; J1 titled-live **pass**; J0/J10/J18/J19 **pass**; Dual Track N1/N2/N3 |

Formal-plan 关闭门: 激活后 Today = 决策包 + 每 live Project 一行 — **true**.

Drift negatives: KPI 墙 — Dual Track `data-kpi-wall=refused`; live guest has no KPI chrome. 未激活仍画 live 包 — creating-only stays continue-create. T13 empty chrome as packet acceptance — empty home only-create. Vite 当产品源 — walk used daemon `/ui/` only. Windows chrome **not-run**.

## 2. Validation summary

| Environment | Result |
|---|---|
| Local Node Dual Track | `clients/pc/web` focused Today tests **pass** (D01). Development evidence only. |
| `DEV-WIN-GNU-01` | Rust link **not-run** (routed). No Rust files in T06. |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | [33991080971](https://github.com/agentkernel/cognitive-os/actions/runs/33991080971) **SUCCESS** at `3012db11`. Closure HEAD required CI recorded in this close after push. |
| `DEV-LINUX-NATIVE-01` | Exact-revision `kernel-server` + UI dist at `01004a49` (`index-05YCvTci.js`). |
| `B01-Desktop-Linux-002` | Daemon `:48681` PID 2684337 replaced T05 `cf6d09e3`. Product origin `/ui/` GET 200. Left `:48181` untouched (PID 166715). `cognitive dsh web` restarted on 3080. |
| `DEV-WINDOWS-NATIVE-OPC-01` chrome | **not-run** (walk used Cursor browser against forwarded guest `/ui/`) |

## 3. Non-claims

Not Gate / release / Profile / B01. No Vite origin. No secret in Git/DOM/report. No numbered migration. Pending ApprovalPreview on this runtime was honestly empty (collapsed packet); that is not a KPI and is not packet-acceptance of T13 empty chrome. T07/T08 stay **done** and were not claimed from this close.

## 4. Unique next

Ready/merge PR [#333](https://github.com/agentkernel/cognitive-os/pull/333) after required CI on this closure HEAD (implementation pin `01004a49` / D01 `3012db11` already green). Unique next after T06 close: Phase 14 Remaining = 0. No remaining ready `P*-T*` implementation task. Do not auto-claim P6. `P7-T07` stays blocked.
