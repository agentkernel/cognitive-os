# P15-T06 remaining layout tokens 176/22/44/~1100 — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P15-T06` / slice `P15-T06/D01` Dual Track **done** 562/562. `P15-T06/D02` guest `/ui/` J0 **walked** 6/6 at `6214aa3e` / SPA `index-uL34SeNM.js`.
- Branch / PR: `personal/P15-T06-layout-tokens` → Draft [#341](https://github.com/agentkernel/cognitive-os/pull/341)
- Guest pin: `6214aa3e` / SPA `index-uL34SeNM.js` + `index-w6MOHVWe.css` on `:48681` (PID **2740931**). `:48181` untouched (PID **166715**).
- Worktree: `D:\agent-kernel-wt-P15-T06`
- Lease: `lease/personal/P15-T06/layout-tokens`
- Fold: `origin/main@1e84cc49` (merged P15-T05 PR [#340](https://github.com/agentkernel/cognitive-os/pull/340))
- Change class: `implementation-only` (remaining v9 shell tokens + Owner honesty demotion). No `core/specs`. No numbered migration. Canvas v9 file not overwritten. AXIOMS.md not rewritten. T01 `--cp-shell-min-width: 1100px` not reverted.
- Claim ceiling: `hypothesis`
- Product origin: daemon-served `/ui/`. Frozen canvas v9 = design authority and completion target. Vite is not the product origin.
- Evaluation routing: **OFF**
- Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334). Do not auto-claim P6. `P7-T07` stays blocked. Leave `:48181` untouched.

## Failure-first (D01)

| ID | Negative | Surface | Observed |
|---|---|---|---|
| N1 | Unauthenticated Today must fail-closed; no fake Activate; no Twitter/X hero; Vite not product | Dual Track App `#/` | **pass** without a prior fail: Session paste-bootstrap (`Session denied` / `Open Session`); no `[data-surface=today]` |
| N2 | Tokens / shell must lock v9 176 / 22 / 44 / ~1100 and must not stack or revert T01 min-width | `tokens.css` + `app.css` | **fail** then **pass**: missing `--cp-nav-width` / `--cp-target-min`; grid still `232px` / `200px`. After change: `--cp-nav-width: 176px`; `--cp-main-min: 576px`; `--cp-rail-width: 348px`; `--cp-target-min: 44px`; `--cp-size-title1: 1.375rem`; `--cp-shell-min-width: 1100px` kept; `.cp-shell` uses the token grid; 1439px band no longer shrinks to `200px`/`240px` |
| N3 | Remaining Owner honesty on Today / Settings / Assistant rail must be secondary `details` | Dual Track `#/` + `#/settings` | **fail** then **pass**: primary rail `.cp-honesty` (`Candidate-only…`). After change: `placement="secondary"` on rail / group chat / lifecycle / member / add-member / unused authority lead |

## Incremental validation log

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-12 | D01 Dual Track `layoutTokens.test.tsx` (failure-first) | **fail** 2/3 then **pass** 3/3 | Node jsdom `clients/pc/web` | worktree on `1e84cc49`+ | N1 already green (SessionGate). N2 expected `--cp-nav-width`. N3 primary rail honesty |
| 2026-09-12 | D01 Dual Track focused T06 | **pass** 3/3 | Node jsdom `clients/pc/web` | worktree | After token + honesty demotion |
| 2026-09-12 | D01 Dual Track full `clients/pc/web` vitest | **pass** 562/562 (80 files) | Node jsdom `clients/pc/web` | `6214aa3e` | T05 559 + T06 +3. Work detail `#main details` scoped so rail secondary honesty is not a fake accordion. Unauth fail-closed; no Twitter P0; T01 min-width kept |
| 2026-09-12 | Exact-revision UI dist | **pass** | local Dual Track `clients/pc/web` at pushed `6214aa3e` | `6214aa3e` | `pnpm build` → `index-uL34SeNM.js` + `index-w6MOHVWe.css`. No kernel-server rebuild (UI-only). |
| 2026-09-12 | Guest `:48681` UI replace | **pass** | `B01-Desktop-Linux-002` | `6214aa3e` | Copied dist into `runtime/data/cognitiveos/ui/` (served) and `ui/`. Daemon PID **2740931** unchanged. `/ui/` GET 200 serves `index-uL34SeNM.js`. Left `:48181` untouched (PID **166715**, `cos-current`). `cognitive dsh web` `:3080` **not-run**. |
| 2026-09-12 | J0 unauthenticated fail-closed | **pass** | host Chrome 140 → tunnel `/ui/#/` | `6214aa3e` / SPA `index-uL34SeNM.js` | Session denied / Open Session. No `[data-surface=today]`. 0 Activate. twitter=false |
| 2026-09-12 | J0 layout vs v9 | **pass** | host Chrome 140 → tunnel `/ui/#/` | `6214aa3e` / SPA `index-uL34SeNM.js` | `.cp-shell[data-visual=v9-target]`; computed `min-width: 1100px`; `grid-template-columns: 176px 756px 348px`; title `22px`; side `176px`; rail `348px`; primaryHonesty=0; not stacked. 0 Activate. twitter=false. Vite not product |
| 2026-09-12 | JOURNEY regression J2/J3/J5/J8 | **pass** | host Chrome 140 → tunnel `/ui/` | `6214aa3e` / SPA `index-uL34SeNM.js` | J2 `今日`; J3 项目列表/创建; J5 `当前项目资料` or `知识已锁定`; J8 `设置` + `模型连接`. Walk **6/6**. Windows native chrome **not-run**. |

## Unique next

`P15-T06/D01` + `P15-T06/D02` walked. Implementation HEAD `6214aa3e` required CI [34669451814](https://github.com/agentkernel/cognitive-os/actions/runs/34669451814) **SUCCESS**. Unique next: required CI on this closure HEAD, then ready/merge [#341](https://github.com/agentkernel/cognitive-os/pull/341). Do not auto-claim P6. `P7-T07` stays blocked. Evaluation routing OFF.
