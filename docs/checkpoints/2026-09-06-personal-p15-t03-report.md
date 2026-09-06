# P15-T03 Projects list/detail vs v9 — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P15-T03` / unique Slice `P15-T03/D01` Dual Track (this delivery); `P15-T03/D02` guest `/ui/` J3 after pushed SHA.
- Branch: `personal/P15-T03-projects-v9`
- Worktree: `D:\agent-kernel-wt-P15-T03`
- Lease: `lease/personal/P15-T03/projects-v9`
- Draft PR: pending this D01 push
- Base: `origin/main` `b3f59161` (P15-T02 #337). Do not redo T01/T02.
- Change class: `implementation-only` (Owner-facing Projects list/detail vs frozen v9). P14 Write Attempt / copy POST / HITL / lifecycle stay. No `core/specs`. No numbered migration. Canvas v9 file not overwritten. AXIOMS.md not rewritten.
- Claim ceiling: `hypothesis`
- Product origin: daemon-served `/ui/`. Frozen canvas v9 = design authority and completion target. Vite is not the product origin.
- Evaluation routing: **OFF**
- Do not claim T04–T06. Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334). Leave `:48181` untouched.

## Failure-first (D01)

| ID | Negative | Surface | Observed |
|---|---|---|---|
| N1 | Unauthenticated Projects must fail-closed; no list; no fake Activate; no Twitter/X hero | Dual Track App `#/projects` | **pass** without a prior fail: Session paste-bootstrap (`Session denied` / `Open Session`); no `[data-page=opc-projects]` |
| N2 | `#/work` must not masquerade as 2.0 Projects list/detail | Dual Track `#/work` | **pass** without a prior fail: no `opc-projects` / `opc-project-detail` / Project sections nav |
| N3 | Clickable Run without live Project authority must fail-closed | Dual Track `#/projects/proj-1/runs` with empty roster | **pass** without a prior fail: Write Attempt `data-write-attempt=blocked` and disabled; no enabled Run control |
| N4 | Live list Owner copy vs v9 (title/status/four-submenu density), not a jargon wall | Dual Track `#/projects` | **fail** then **pass**: title was English inventory chrome. After change: `项目列表`; 打开/成员/运行/产出; status `已上线`; no Vite lecture in `#main` |
| N5 | Creating-only list must continue create; must not open detail/members/runs/outputs | Dual Track `#/projects` creating row | **fail** then **pass**: title `未完成的创建`; `继续这份草稿` → `#/projects/new`; no live submenu links |
| N6 | Empty list honest: `还没有项目` + 回今日; no demo / Activate | Dual Track `#/projects` empty | **fail** then **pass**: emptyAction `回今日`; still `no Project`; no Activate |
| N7 | Detail Owner copy + four submenus + honest empty axis | Dual Track `#/projects/proj-1` | **fail** then **pass**: title `项目详情`; lede 只读章程与流程轴; nav 详情/成员/运行/产出; empty axis `没有流程轴 · no PlanRevision axis` |

## Incremental validation log

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-06 | D01 Dual Track `projectsListDetail.test.tsx` (failure-first) | **fail** 4/7 then **pass** 7/7 | Node jsdom `clients/pc/web` | worktree | N1–N3 already green (SessionGate / `#/work` / Write Attempt). N4–N7 expected v9 Owner copy |
| 2026-09-06 | D01 Dual Track `projectSubmenus.test.tsx` | **pass** 9/9 | Node jsdom `clients/pc/web` | worktree | Nav labels 详情/成员/运行/产出; `打开` not `Open` |
| 2026-09-06 | D01 Dual Track `projectLifecycle.test.tsx` | **pass** 3/3 | Node jsdom `clients/pc/web` | worktree | Copy button still matches `副本`; P14 copy POST stays |
| 2026-09-06 | D01 Dual Track full `clients/pc/web` vitest | **pass** 553/553 (77 files) | Node jsdom `clients/pc/web` | worktree | +7 Owner-copy tests vs T02 546/546. Unauth fail-closed; no Twitter P0; no fake Activate; Write Attempt without authority stays blocked |
| 2026-09-06 | Guest `/ui/` J3 | **not-run** | `B01-Desktop-Linux-002` `:48681` | after pushed SHA | D02 only after this D01 push; skip if T04/T05 are deploying `:48681`. `:48181` untouched |

## Unique next

`P15-T03/D01` Dual Track is **done** locally (553/553). Unique next after this push: **`P15-T03/D02` guest `/ui/` J3** + `JOURNEY-BROWSER-SYNC-01` on the pushed SHA. Do not claim T04–T06. Evaluation routing OFF.
