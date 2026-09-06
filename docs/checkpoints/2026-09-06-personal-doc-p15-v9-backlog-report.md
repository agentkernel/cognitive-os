# DOC-P15-V9-BACKLOG — running report (Phase 15 remaining v9-module plan)

- Activity: owner-directed documentation delivery `DOC-P15-V9-BACKLOG`
  (register remaining Phase 15 tasks so daemon `/ui/` can reach frozen canvas v9
  function and visual effect; planner/registrar only)
- Lease: `lease/personal/DOC-P15-V9-BACKLOG/plan-registration`
  (plan + checkpoint files on `personal/DOC-P15-V9-BACKLOG`; no second P15-T01 lease)
- Branch: `personal/DOC-P15-V9-BACKLOG` (worktree `D:\agent-kernel-wt-DOC-P15-V9-BACKLOG`; base `origin/main@dbe188f5`)
- Environment for every local unit: `DEV-WIN-GNU-01` (Windows PowerShell 5.1;
  Node tooling only; no Rust link; no guest deploy)
- Claim ceiling: `hypothesis`. Documentation/plan registration only — no Gate,
  release, Profile, implementation, T15, or EVAL campaign. `not-run` is never pass.
- Reporting rule: `TEST-REPORT-INCREMENTAL-01` — each unit appended on completion.

## 1. Owner instruction (2026-09-06)

Register a reasonable remaining-module plan so `/ui/` reaches frozen canvas v9
function and visual effect. Do not edit `clients/pc/web/**`. Do not overwrite
canvas v9. Do not rewrite AXIOMS.md. No Twitter/X P0, no fake Activate, no
Vite-as-product. Do not deploy guest `:48681`. Leave `:48181` untouched. Do not
claim a second P15-T01 lease. Do not claim T02–T06 implementation leases.
Unique next stays `P15-T01/D02` until the sibling guest walk finishes.

## 2. Split registered (canvas v9 modules actually present)

| ID | Status | Scope | Journey | Depends |
|---|---|---|---|---|
| `P15-T01` | **in-progress** #335 | shell + Today Owner copy (unchanged; not marked done until D02) | J0/J2/J10 | — |
| `P15-T02` | not-started | Write Project / ①–⑤ Owner copy + visual (过程/成员/测试/联调) | J1 | T01/D02 |
| `P15-T03` | not-started | Projects list/detail vs v9 (four-submenu visual) | J3 | T01 |
| `P15-T04` | not-started | Knowledge vs v9 files/why/import visual | J5 | T01 |
| `P15-T05` | not-started | Settings / Model Connections vs v9 | J8/J12 | T01 |
| `P15-T06` | not-started | remaining layout tokens 176px / 22px / 44px / ~1100px | J0 | T02–T05 |

HITL / AddMember / Conversation fold into T02/T03. Twitter/X is not a P0 card.
Invariants on every card: fail-closed, persist-before-dispatch, daemon-only
writer, product origin = `/ui/`, Dual Track + guest `/ui/` walk,
`JOURNEY-BROWSER-SYNC-01`, claim ceiling `hypothesis`, no Gate.

## 3. Units

| # | Unit | Instrument | Environment | Revision | Result | Notes |
|---|---|---|---|---|---|---|
| U1 | Formal plan `P15-T02`..`T06` three-column + typed deps + Delivery Slices + mermaid | edit `PERSONAL-DEVELOPMENT-PLAN.md` | `DEV-WIN-GNU-01` | worktree | **pass** | T01 remains in-progress; unique next = T01/D02 |
| U2 | Current snapshot Layer 1 181/154/1/1/9/27 + unique next T01/D02 | edit `PROGRESS.md` | `DEV-WIN-GNU-01` | worktree | **pass** | Phase 15 Remaining = 6; do not claim T02–T06 |
| U3 | `plan.md` cards + YAML `P15.acceptance_requires` + `tasks:` | edit `docs/plan/plan.md` | `DEV-WIN-GNU-01` | worktree | **pass** | `PERSONAL_2_0_0_V9_DESIGN_AUTHORITY` lists T01–T06 |
| U4 | Trace `PERS-PR-054` tasks | edit `personal-trace.yaml` | `DEV-WIN-GNU-01` | worktree | **pass** | `[P15-T01..T06]` |
| U5 | Environments §5.4 rows T02–T06 | edit `PERSONAL-TEST-ENVIRONMENTS.md` | `DEV-WIN-GNU-01` | worktree | **pass** | Dual Track + guest `/ui/` + JOURNEY; Vite/canvas/fake Activate/Twitter P0 are non-substitutes |
| U6 | Dev-prep Phase 15 mermaid (Phase 13/14 graphs untouched) | edit `personal-2.0.0-dev-prep-index.md` | `DEV-WIN-GNU-01` | worktree | **pass** | edges match formal plan T01→T02/T03/T04/T05→T06 |
| U7 | Product index fold | edit `web-ui-design.md` | `DEV-WIN-GNU-01` | worktree | **pass** | remaining modules = T02–T06; origin=`/ui/` |
| U8 | Adjacent DOC lease row (checkpoint-only writable paths) | `PARALLEL-LANES.md` | `DEV-WIN-GNU-01` | worktree | **pass** | no second P15-T01 lease; no blank line between active rows |
| U9 | `pnpm run check:consistency` | `tools/src/check-consistency.mjs` | `DEV-WIN-GNU-01` | worktree | **pass** | `check-consistency: OK` (275 req, 55 error codes, 74 schemas, 89 vectors; Phase 13 build-order verified) |
| U10 | `git diff --check` | git | `DEV-WIN-GNU-01` | worktree | **pass** | no whitespace errors |

## Unique next

`P15-T01/D02` guest `/ui/` J0/J2/J10 + `JOURNEY-BROWSER-SYNC-01` on Draft PR
[#335](https://github.com/agentkernel/cognitive-os/pull/335) after required CI
on a **pushed** exact revision. Exclusive `:48681`. Leave `:48181` untouched.
Do not claim `P15-T02`..`T06` implementation leases.
