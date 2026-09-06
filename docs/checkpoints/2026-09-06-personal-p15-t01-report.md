# P15-T01 v9 design authority on daemon `/ui/` — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P15-T01` / slices `P15-T01/D01` **done** + `P15-T01/D02` **done**
- Branch: `personal/P15-T01-v9-target`
- Worktree: `D:\agent-kernel-wt-P15-T01`
- Lease: `lease/personal/P15-T01/v9-design-authority-shell`
- Draft PR: [#335](https://github.com/agentkernel/cognitive-os/pull/335) (kept Draft)
- Implementation pin: Dual Track `6b524c53`; guest / required CI `4373d158` (tsc exclude Dual Track tests so `pnpm build` can emit `/ui/` dist)
- Change class: `product-semantic` (docs reframe) + `implementation-only` (shell/Today Owner chrome + `--cp-*` type scale / no-stack). No `core/specs`. No numbered migration. Canvas v9 file not shipped / not overwritten. AXIOMS.md not rewritten.
- Claim ceiling: `hypothesis`
- Product origin: daemon-served `/ui/`. Frozen canvas v9 = design authority and completion target. Vite is not the product origin.
- Evaluation routing: **OFF**
- Phase 14 journeys stay **done**. Do not claim T02–T06 from this close. Do not auto-claim P6. `P7-T07` stays blocked.

## Failure-first (D01)

| ID | Negative | Surface | Observed |
|---|---|---|---|
| N1 | Unauthenticated Today must fail-closed; no fake Activate; no Twitter/X hero; no live packets | Dual Track App `#/` | **fail** then **pass**: Session paste-bootstrap (`Session denied` / `Open Session`); no `[data-surface=today]`; no `[data-packet]` |
| N2 | Authenticated empty Today must not lead with a developer honesty wall, Vite lecture, or “not an authority writer” brand | Dual Track Today | **fail** then **pass**: `.cp-shell[data-visual=v9-target]`; no primary `#main .cp-honesty` outside `details[data-honesty=secondary]`; h2 `今日`; CTA `创建项目` |
| N3 | Type scale / no-stack tokens must match visual spec (body 14px, headline 15px, shell min-width 1100px) | `tokens.css` | **fail** (`?raw` empty) then **pass** via `readFileSync` |

## Incremental validation log

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-06 | D01 Dual Track `todayOwnerChrome.test.tsx` + `shell.test.tsx` | **pass** 10/10 (3 chrome + 7 shell) | Node jsdom `clients/pc/web` | worktree | N1–N3 after `readFileSync` tokens + Session fail-closed + secondary honesty |
| 2026-09-06 | D01 Dual Track focused pack (`opcIa` / packets / livePackets / ownerChrome / chrome / commands / App) | **pass** 54/54 | Node jsdom `clients/pc/web` | `6b524c53` | Fake Activate refused; unauthenticated fail-closed; no Twitter P0; Today h2 `今日` + CTA `创建项目` |
| 2026-09-06 | D01 Dual Track full `clients/pc/web` vitest | **pass** 543/543 (75 files) | Node jsdom `clients/pc/web` | worktree on `6b524c53` | No primary Today honesty wall; Session fail-closed; no Twitter P0 |
| 2026-09-06 | Draft PR [#335](https://github.com/agentkernel/cognitive-os/pull/335) | pass | GitHub | `6b524c53` then `4373d158` | D01 Dual Track + docs reframe pushed; tsc fix so Linux `vite build` can emit dist |
| 2026-09-06 | D01 Dual Track re-run at D02 resume | **pass** 543/543 (75 files) | Node jsdom `clients/pc/web` | `c3e5cbcb` (ancestor of `4373d158`) | Same pack; `4373d158` only excludes `*.test.tsx` from UI `tsc` |
| 2026-09-06 | Required CI | **pass** | GitHub `CI-UBUNTU-01` + `CI-WINDOWS-MSVC-01` | `4373d158` | Run [34006761447](https://github.com/agentkernel/cognitive-os/actions/runs/34006761447) **SUCCESS** (resolve, ubuntu, windows, required-ci) |
| 2026-09-06 | Exact-revision Linux UI + kernel-server | **pass** | `DEV-LINUX-NATIVE-01` `wuz@192.168.1.2` | `4373d158` | `kernel-server` ELF; UI dist `index-D06uNekN.js` + `index-C9SPnPVx.css` |
| 2026-09-06 | Guest `:48681` replace | **pass** | `B01-Desktop-Linux-002` | `4373d158` | runtime `/home/hal9001/p13-main-711a5a7c/`; PID **2740931** `kernel-server --personal --bind 127.0.0.1:48681`; `/ui/` GET 200 serves `index-D06uNekN.js`. Left `:48181` untouched (PID **166715**, `cos-current`). Unauth `GET /management/project/v1/list` **401**. `cognitive dsh web` `:3080` **not-run** (stale Path B after daemon replace; `DshAdapterError: daemon response is not a candidate-only response`). |
| 2026-09-06 | J0 gate + unauthenticated fail-closed | **pass** | Cursor browser → tunnel `/ui/` | `4373d158` | Empty Issue → `management HTTP 401; task HTTP 401. Bootstrap discarded.` Gate remained. L1 `今日` / `项目` / `知识` / `设置`. `.cp-shell[data-visual=v9-target]`. No `[data-surface=today]`, no packets, 0 Activate buttons, `twitter=false`, Vite not used. Same-origin one-shot `/ui/assets/p15-t01-once.txt` fill (file deleted immediately; secret not in Git/chat/report). Header `principal://local/owner · mgmt+task`. |
| 2026-09-06 | J2 Today Owner chrome + live packets | **pass** | guest `/ui/#/` | `4373d158` / SPA `index-D06uNekN.js` | `data-surface=today`. h2 `今日`. Lede `看清并处理要你拍板的事。` No primary `#main .cp-honesty`. No Continue create. 4 live Projects: `P14-T03 D02 titled live` / `wizard live` / `wizard titled live` / `P14-T04 D02 wizard join live`. Packet `data-collapsed=true` (“nothing pending”; overview stays). Period today `created 2 · live 4 · blocked 0`; week switch `data-period=week`. Chat cannot Approve. No KPI wall. Computed `min-width: 1100px`; body `14px`. Vite not used. `创建项目` is on Dual Track empty-home, not on this live-packet Today (T06). |
| 2026-09-06 | J1 Dual Track wizard chrome regression | **pass** | `/ui/#/projects/new` | `4373d158` | ①–⑤ chrome (`① Charter` / `② 流程初始化` / `③ 成员初始化` / `④ 分环节测试` / `⑤ 联合调试`). Full ①–⑤ click-walk not repeated (T03/T04 already walked this runtime). |
| 2026-09-06 | J10 no X/Twitter P0 hero | **pass** | Today, Projects, Knowledge, Settings | `4373d158` | CDP `twitter=false`. Bundle `index-D06uNekN.js`. |
| 2026-09-06 | J18 identity | **pass** | `#/session` | `4373d158` | principal `principal://local/owner`; header `principal://local/owner · mgmt+task`; Clear memory session present. Bootstrap copy is not a Provider key. |
| 2026-09-06 | J19 retired routes | **pass** | `#/inbox` `#/team` `#/hitl/prev-1` `#/home` `#/work` | `4373d158` | each `No such route` / Control Plane missing-address copy. |
| 2026-09-06 | Knowledge / Settings L1 regression | **pass** | `#/knowledge` `#/settings` | `4373d158` | Knowledge tabs Files / Import / Why this fragment / Memory; 0 Admit. Settings Model Connections; no fake Connect; 9×9 unmounted. |
| 2026-09-06 | Windows native chrome JOURNEY | **not-run** | walk used local Cursor browser against forwarded guest `/ui/` | — | not Windows-native daemon chrome |

## Unique next

`P15-T01/D01` + `P15-T01/D02` are **done** on Draft [#335](https://github.com/agentkernel/cognitive-os/pull/335). Unique next stays on **T01** (ready/merge/closure) — not T02. `P15-T02`..`T06` stay **not-started** on [#336](https://github.com/agentkernel/cognitive-os/pull/336). Keep #335 Draft until T01 full acceptance close. Do not auto-claim P6. `P7-T07` stays blocked. Evaluation routing OFF.
