# P14-T05 Attempt/Runs/Outputs from Project chrome — closure

- Task: `P14-T05` **done** / slices `P14-T05/D01` **done** + `P14-T05/D02` **done**
- Change class: `implementation-only` (Project chrome Write Attempt posts management `dsh.hosted.attempt.run`; Runs/Outputs read the real ledger/CAS or stay honest empty; no `core/specs`; no numbered migration)
- Lease: `lease/personal/P14-T05/attempt-runs-outputs` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P14-T05-attempt-runs` → Draft PR [#332](https://github.com/agentkernel/cognitive-os/pull/332) (ready/merge in this close)
- Implementation revision: `d72c2847` (D01 Dual Track). Fold `origin/main@a6247f09` (T04 #331) then lease-ledger CI fix `cf6d09e3`. JOURNEY report `2236c18c`.
- Validated HEAD before this closure: `cf6d09e3` required CI [33991619494](https://github.com/agentkernel/cognitive-os/actions/runs/33991619494) **SUCCESS** (resolve, ubuntu, windows, required-ci)
- Running report: [P14-T05 report](2026-09-06-personal-p14-t05-report.md)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` close "Project chrome starts a 2.0 Attempt" only). Not Gate / release / Profile / B01. Windows native chrome JOURNEY **not-run**.
- Evaluation routing: **OFF**. T02/T03/T04/T07/T08 remain **done**. T06 stays in-progress on Draft PR [#333](https://github.com/agentkernel/cognitive-os/pull/333) at `3012db11` (A8). Do not claim T07/T08.

## 1. Acceptance mapping (formal plan P14-T05 card + D01/D02)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| Project chrome starts a 2.0 Attempt (not Linux 1.0 `#/work`) | `ProjectRunsPage` Write Attempt POSTs management `POST /management/project/v1/dsh.hosted.attempt.run` (`wait:false`) when Project `state=active` and roster has `state==="seated"`; URL stays `#/projects/…/runs` | `#/work` is not 2.0 chrome; Vite is not product origin | Dual Track `projectRuns` 8 + `routineRuns` 6; guest minted `dshattempt-01a0736b-d14f-7f02-9b16-d865702245da` on `#/projects/project-01a0731e-cfb2-72b2-9025-5e8372f45dc1/runs`; `#/work` → No such route |
| Runs/Outputs read the real ledger/CAS or stay honest empty | Runs: occurrence ledger + Attempt history; Outputs: CAS gallery or honesty copy | fake gallery; clickable Run without authority | Dual Track `projectOutputs` 7; guest Runs honest empty occurrence + real Attempt row; Outputs **no openable artifact yet**; Write Attempt `blocked` until seated |
| EVAL-016 J14 + `JOURNEY-BROWSER-SYNC-01` | exact-revision guest daemon `/ui/` on `B01-Desktop-Linux-002` pin `cf6d09e3` / SPA `index-BxtJu4NK.js` | unauthenticated Issue fail-closed; Vite not used | J14 Write Attempt **pass**; J14 honesty **pass**; J1 titled-live regression **pass**; J0/J10/J18/J19 **pass** |

Formal-plan 关闭门: Project chrome 启动 2.0 Attempt — **true**; Runs/Outputs 读真实 ledger/产物或诚实 empty — **true**.

Drift negatives: `#/work` 冒充 2.0 — retired hash 404; 无权威却渲染可点 Run — `data-write-attempt=blocked` until seated, task-channel `attempt.run` stays false; Vite 当产品源 — walk used daemon `/ui/` only; 把 fail-closed 写成 pass — unauthenticated list stays 401; Windows chrome **not-run**.

## 2. Validation summary

| Environment | Result |
|---|---|
| Local Node Dual Track | `clients/pc/web` `pnpm test` **33/33** (`projectRuns` 8, `projectOutputs` 7, `normalize` 12, `routineRuns` 6). Development evidence only. |
| `DEV-WIN-GNU-01` | Rust link **not-run** (routed). No Rust files in D01. |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | [33991619494](https://github.com/agentkernel/cognitive-os/actions/runs/33991619494) **SUCCESS** at `cf6d09e3`. |
| `DEV-LINUX-NATIVE-01` | Exact-revision `kernel-server` + UI dist at `cf6d09e3` (`index-BxtJu4NK.js`). Worktree `/home/wuz/cognitiveos-personal-worktrees/p14-t05-cf6d09e3`. |
| `B01-Desktop-Linux-002` | Daemon `:48681` PID 2671617 replaced T04 `fe498997`. Product origin `/ui/` GET 200. Left `:48181` untouched (PID 166715). `cognitive dsh web` restarted on 3080 (PID 2672166). |
| `DEV-WINDOWS-NATIVE-OPC-01` chrome | **not-run** (walk used Cursor browser against forwarded guest `/ui/`) |

## 3. Non-claims

Not T06 Today live packets (already claimed, Draft PR [#333](https://github.com/agentkernel/cognitive-os/pull/333) at `3012db11`; T06 D02 may use guest `:48681` after this close). T02/T03/T04/T07/T08 stay **done**. No Gate / release / Profile / B01. No Vite origin. No secret in Git/DOM/report. No numbered migration. Seating the T04 Dual Track roster (`seat.request` + `seat.confirm`) was a guest precondition for Write Attempt, not a T05 product change.

## 4. Unique next

Ready/merge PR [#332](https://github.com/agentkernel/cognitive-os/pull/332) after required CI on this closure HEAD (implementation pin `cf6d09e3` already green). Unique next: `P14-T06/D02` guest `/ui/` J2 + `JOURNEY-BROWSER-SYNC-01` on the same `:48681` with the pushed T06 HEAD (fold `origin/main` first). Do not claim T07/T08.
