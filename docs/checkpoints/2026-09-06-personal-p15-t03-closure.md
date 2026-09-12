# P15-T03 Projects list/detail vs v9 — closure

- Task: `P15-T03` **done** / slices `P15-T03/D01` **done** + `P15-T03/D02` **done**
- Change class: `implementation-only` (Owner-facing Projects list/detail vs frozen v9). P14 copy POST, HITL, lifecycle, and Write Attempt-without-authority stay. No `core/specs`. No numbered migration. Canvas v9 file not overwritten. AXIOMS.md not rewritten.
- Lease: `lease/personal/P15-T03/projects-v9` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P15-T03-projects-v9` → Draft PR [#338](https://github.com/agentkernel/cognitive-os/pull/338) (ready/merge in this close)
- Dual Track / guest pin: `8b47b73e73b8921a3a7d37681f68ba3fb1711b97`
- Closing docs HEAD ancestor: `c72499b17646d178f0aae1aa57b1a616bd32f886` required CI [34015268580](https://github.com/agentkernel/cognitive-os/actions/runs/34015268580) **SUCCESS**
- Guest `/ui/` pin: `8b47b73e` / SPA `index-Vx_NJPld.js` + `index-C9SPnPVx.css`
- Running report: [P15-T03 report](2026-09-06-personal-p15-t03-report.md)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` close "Projects list/detail Owner copy toward v9" only). Not Gate / release / Profile / B01. Windows native chrome JOURNEY **not-run**.
- Evaluation routing: **OFF**. P14-T05/T06 remain **done**. Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334). Do not implement T05–T06 from this close.

## 1. Acceptance mapping (formal plan P15-T03 card + D01/D02)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| Projects list/detail vs v9: titles, status, four-submenu density, Owner copy | `ProjectsPage` / `ProjectDetailPage` / `ProjectWorkNav` Owner titles `项目列表` / `未完成的创建` / `还没有项目` / `项目详情`; live 打开/成员/运行/产出; status `已上线`; creating row continues create | Dual Track N4 English inventory chrome; jargon wall | Dual Track N4 **fail** then **pass**; guest J3 list `项目列表` + 打开/成员/运行/产出 + `已上线`; detail `项目详情` / 详情/成员/运行/产出 |
| No axis / no authority stays honest empty | Empty axis copy `没有流程轴 · no PlanRevision axis`; Write Attempt `data-write-attempt=blocked` + disabled without seating | Clickable Run without authority; `#/work` as 2.0 | Dual Track N3/N2 **pass**; guest Write Attempt blocked; `#/work` has no `opc-projects` |
| Creating drafts continue create; empty list honest | Creating title `未完成的创建` → `#/projects/new`; empty `还没有项目` + `回今日` | Live chrome on creating row; fake Activate | Dual Track N5/N6 **fail** then **pass**; guest leftover `继续这份草稿`; 0 Activate |
| `JOURNEY-BROWSER-SYNC-01` J3 + regression J0/J2/J10 | exact-revision guest daemon `/ui/` on `B01-Desktop-Linux-002` | Vite origin; `:48181` replace | J3 **pass**; J0/J2/J10 **pass**. Left `:48181` untouched (PID 166715) |

Formal-plan 关闭门: 列表/详情标题与四子菜单密度对 v9 — **true**; 无权威诚实 empty — **true**.

Drift negatives: `#/work` 冒充 2.0 — Dual Track N2 + guest `#/work` have no Projects chrome. 无权威可点 Run — Write Attempt blocked+disabled. P14 功能缺口写成未做 — copy POST / HITL / lifecycle / Write Attempt stay. Vite as product origin — walk used daemon `/ui/` only. Canvas v9 not overwritten. AXIOMS.md not rewritten. P14-T03/T04/T05 functional cards not reopened.

## 2. Validation summary

| Environment | Result |
|---|---|
| Local Node Dual Track | `clients/pc/web` vitest **553/553** (77 files) at worktree ancestor of `8b47b73e`. Development evidence only. |
| `DEV-WIN-GNU-01` | Rust link **not-run** (routed). No Rust files in T03. |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | Ancestor `c72499b1` required CI [34015268580](https://github.com/agentkernel/cognitive-os/actions/runs/34015268580) **SUCCESS**. Merge-HEAD required CI is this close after push. |
| `DEV-LINUX-NATIVE-01` | Exact-revision UI dist `index-Vx_NJPld.js` at `8b47b73e`. No kernel-server rebuild (UI-only). |
| `B01-Desktop-Linux-002` | Daemon `:48681` PID **2740931** pin `8b47b73e`. Left `:48181` untouched (PID **166715**). `cognitive dsh web` `:3080` **not-run**. |
| `DEV-WINDOWS-NATIVE-OPC-01` chrome | **not-run** (walk used Cursor browser against forwarded guest `/ui/`) |

## 3. Non-claims

Not T04 Knowledge vs v9. Not T05–T06. Not Gate / release / Profile / B01. No Vite origin. No secret in Git/DOM/report. No numbered migration. Canvas v9 not overwritten. AXIOMS.md not rewritten. P14-T05/T06 stay **done**. SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334) not merged.

## 4. Unique next

Ready/merge PR [#338](https://github.com/agentkernel/cognitive-os/pull/338) after required CI on this closure HEAD (guest pin `8b47b73e` already walked). Unique next after T03 close: **claim / continue `P15-T04`** (Knowledge vs v9; Draft [#339](https://github.com/agentkernel/cognitive-os/pull/339) already exists). Do not claim T05–T06 from this close. Do not auto-claim P6. `P7-T07` stays blocked. Evaluation routing OFF.
