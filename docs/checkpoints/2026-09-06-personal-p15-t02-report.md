# P15-T02 Write Project / ①–⑤ Owner copy vs v9 — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P15-T02` / slices `P15-T02/D01` **done** + `P15-T02/D02` **done** (guest walk). Ready/merge waits required CI on merge HEAD.
- Branch: `personal/P15-T02-write-owner-copy`
- Worktree: `D:\agent-kernel-wt-P15-T02`
- Lease: `lease/personal/P15-T02/write-owner-copy`
- Draft PR: [#337](https://github.com/agentkernel/cognitive-os/pull/337)
- Dual Track / guest pin: `3528ba28df48f2a949b4189ca295736f607de96e`
- SPA: `index-V6QE_Yae.js` + `index-C9SPnPVx.css` on guest `:48681` (PID **2740931**). `:48181` untouched (PID **166715**).
- Change class: `implementation-only` (Owner-facing Write/①–⑤ copy vs frozen v9). Persist-before-dispatch Write stays. No `core/specs`. No numbered migration. Canvas v9 file not overwritten. AXIOMS.md not rewritten.
- Claim ceiling: `hypothesis`
- Product origin: daemon-served `/ui/`. Frozen canvas v9 = design authority and completion target. Vite is not the product origin.
- Evaluation routing: **OFF**
- Do not claim T03–T06. Keep P14-T05/T06 **done**. Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334).

## Failure-first (D01)

| ID | Negative | Surface | Observed |
|---|---|---|---|
| N1 | Unauthenticated Write must fail-closed; no wizard; no fake Activate; no Twitter/X hero | Dual Track App `#/projects/new` | **pass** without a prior fail: Session paste-bootstrap (`Session denied` / `Open Session`); no `[data-page=opc-create-wizard]` |
| N2 | Authenticated ① must use v9 Owner step language, not Charter / Dual Track jargon walls | Dual Track `#/projects/new` | **fail** then **pass**: steps were `① Charter` + PageHeader `Create Project` + Dual Track lede + primary Vite/Activate honesty. After change: steps `① 项目初始化`…`⑤ 联合调试`; title `创建项目 · ① 项目初始化`; h2 `① 逐项确认这件事`; labels `标题` / `这件事`; secondary honesty; no Vite lecture; `进入 ②` |
| N3 | No Activate chrome / Twitter P0 on Write | Dual Track `#/projects/new` | **fail** then **pass**: honesty text contained `Activate`; rewritten without that verb. Persist-before-dispatch `Request preview` / `Write Project` stay on ⑤ (P14-T02 pack) |

## Incremental validation log

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-06 | D01 Dual Track `createWizardOwnerCopy.test.tsx` (failure-first) | **fail** 2/3 then **pass** 3/3 | Node jsdom `clients/pc/web` | worktree | N1 already green (SessionGate). N2 expected `① 项目初始化`, received `① Charter`. N3 `\bActivate\b` in honesty wall |
| 2026-09-06 | D01 Dual Track `createWizard.test.tsx` + Owner copy | **pass** 10/10 | Node jsdom `clients/pc/web` | worktree | P14 persist-before-dispatch walk still posts draft.create → preview.request → confirm. Continue → `进入 ②`. Empty-init error matches `这件事` |
| 2026-09-06 | D01 Dual Track full `clients/pc/web` vitest | **pass** 546/546 (76 files) | Node jsdom `clients/pc/web` | worktree | +3 Owner-copy tests vs T01 543/543. Unauth fail-closed; no Twitter P0; no fake Activate; persist-before-dispatch Write stays |
| 2026-09-06 | Draft PR [#337](https://github.com/agentkernel/cognitive-os/pull/337) | pass | GitHub | `3528ba28` | D01 Dual Track pushed |
| 2026-09-06 | Exact-revision Linux UI dist | **pass** | `DEV-LINUX-NATIVE-01` worktree `/home/wuz/cognitiveos-personal-worktrees/p15-t02-3528ba28` | `3528ba28` | `pnpm build` → `index-V6QE_Yae.js` + `index-C9SPnPVx.css`. No kernel-server rebuild (UI-only) |
| 2026-09-06 | Guest `:48681` UI replace | **pass** | `B01-Desktop-Linux-002` | `3528ba28` | Copied dist into `runtime/data/cognitiveos/ui/` (served) and `ui/`. Daemon PID **2740931** unchanged. `/ui/` GET 200 serves `index-V6QE_Yae.js`. Left `:48181` untouched (PID **166715**, `cos-current`). `cognitive dsh web` `:3080` **not-run** |
| 2026-09-06 | J0 unauthenticated fail-closed | **pass** | Cursor browser → tunnel `/ui/#/projects/new` | `3528ba28` / SPA `index-V6QE_Yae.js` | Session denied / Open Session. No `[data-page=opc-create-wizard]`. 0 Activate buttons |
| 2026-09-06 | J1 Owner ①–⑤ vs v9 | **pass** | guest `/ui/#/projects/new` | `3528ba28` | ① title `创建项目 · ① 项目初始化`; h2 `① 逐项确认这件事`; labels `标题` / `这件事`; `进入 ②`. ② `创建项目 · ② 流程初始化` / `② 一条流程轴，一次只开一环`. ③ `创建岗位，再逐人就位`. ④ `测这一环，直到子产出可打开`. ⑤ `联合调试 · 第一次成功`. `Request preview` + `Write Project` present; Write disabled until preview. 0 Activate. Secondary honesty `How this page is sourced`. Vite not used |
| 2026-09-06 | J2 Today regression | **pass** | guest `/ui/#/` | `3528ba28` | h2 `今日`; lede `看清并处理要你拍板的事。`; packet collapsed; period today `created 2 · live 4 · blocked 0` |
| 2026-09-06 | J10 no X/Twitter P0 | **pass** | Today / Projects / Knowledge / Settings | `3528ba28` | `twitter=false`; 0 Activate |
| 2026-09-06 | Required CI | **in-progress** | GitHub | `3528ba28` | ubuntu **pass** 4m44s; windows pending (run [34011043523](https://github.com/agentkernel/cognitive-os/actions/runs/34011043523)) |
| 2026-09-06 | Windows native chrome JOURNEY | **not-run** | walk used local Cursor browser against forwarded guest `/ui/` | — | not Windows-native daemon chrome |

## Unique next

`P15-T02/D01` Dual Track is **done**. Guest J1 walked at `3528ba28` / SPA `index-V6QE_Yae.js`. Unique next: required CI **SUCCESS** on the exact HEAD to merge, then ready/merge [#337](https://github.com/agentkernel/cognitive-os/pull/337). Do not claim T03–T06. Evaluation routing OFF.
