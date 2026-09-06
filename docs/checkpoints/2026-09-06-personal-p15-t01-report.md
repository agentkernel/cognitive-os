# P15-T01 v9 design authority on daemon `/ui/` — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P15-T01` / slices `P15-T01/D01` **in-progress** + `P15-T01/D02` **ready** (guest `/ui/` after push)
- Branch: `personal/P15-T01-v9-target`
- Worktree: `D:\agent-kernel-wt-P15-T01`
- Lease: `lease/personal/P15-T01/v9-design-authority-shell`
- Draft PR: pending push
- Change class: `product-semantic` (docs reframe) + `implementation-only` (shell/Today Owner chrome + `--cp-*` type scale / no-stack). No `core/specs`. No numbered migration. Canvas v9 file not shipped / not overwritten.
- Claim ceiling: `hypothesis`
- Product origin: daemon-served `/ui/`. Frozen canvas v9 = design authority and completion target. Vite is not the product origin.
- Evaluation routing: **OFF**
- Phase 14 journeys stay **done**. Do not claim T07/T08. Do not auto-claim P6. `P7-T07` stays blocked.

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
| 2026-09-06 | D02 guest `/ui/` J0/J2/J10 + `JOURNEY-BROWSER-SYNC-01` | **not-run** | `B01-Desktop-Linux-002` `:48681` | — | wait for pushed exact revision. Leave `:48181` untouched. |

## Unique next

Finish D01 Dual Track + docs-sync + Draft PR. Then D02 on guest `:48681` after the pushed revision.
