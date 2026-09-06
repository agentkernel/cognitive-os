# P15-T02 Write Project / ①–⑤ Owner copy vs v9 — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P15-T02` / slice `P15-T02/D01` **in-progress**; `P15-T02/D02` ready after a **pushed** SHA
- Branch: `personal/P15-T02-write-owner-copy`
- Worktree: `D:\agent-kernel-wt-P15-T02`
- Lease: `lease/personal/P15-T02/write-owner-copy`
- Draft PR: pending first push
- Change class: `implementation-only` (Owner-facing Write/①–⑤ copy vs frozen v9). Persist-before-dispatch Write stays. No `core/specs`. No numbered migration. Canvas v9 file not overwritten. AXIOMS.md not rewritten.
- Claim ceiling: `hypothesis`
- Product origin: daemon-served `/ui/`. Frozen canvas v9 = design authority and completion target. Vite is not the product origin.
- Evaluation routing: **OFF**
- Do not claim T03–T06. Keep P14-T05/T06 **done**. Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334). Leave guest `:48181` untouched. Guest `:48681` only after a **pushed** T02 SHA.

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
| 2026-09-06 | D02 guest `/ui/` J1 | **not-run** | `B01-Desktop-Linux-002` `:48681` | — | wait for a **pushed** T02 SHA; leave `:48181` untouched |

## Unique next

`P15-T02/D01` Dual Track Owner copy. After commit/push/Draft PR: `P15-T02/D02` guest `/ui/` J1 + `JOURNEY-BROWSER-SYNC-01` on that pushed SHA. Do not claim T03–T06. Evaluation routing OFF.
