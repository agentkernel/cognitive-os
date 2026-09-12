# P15-T04 Knowledge vs v9 — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P15-T04` / slices `P15-T04/D01` Dual Track **done** + `P15-T04/D02` guest `/ui/` J5 **walked**.
- Branch: `personal/P15-T04-knowledge-v9`
- Worktree: `D:\agent-kernel-wt-P15-T04`. Fold `origin/main@ae1dc06a` (P15-T03 #338) at `991408f8`.
- Lease: `lease/personal/P15-T04/knowledge-v9`
- Draft PR: [#339](https://github.com/agentkernel/cognitive-os/pull/339) (keep Draft until required CI on closing HEAD)
- SPA: `index-3s2lrA_c.js` + `index-C9SPnPVx.css` on guest `:48681` (PID **2740931**). `:48181` untouched (PID **166715**).
- Do not claim T05–T06.
- Change class: `implementation-only` (Owner-facing Knowledge copy vs frozen v9). Vault authority unchanged. Canvas v9 file not overwritten. AXIOMS.md not rewritten.
- Claim ceiling: `hypothesis`
- Product origin: daemon-served `/ui/`. Vite is not the product origin.
- Evaluation routing: **OFF**
- Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334).

## Failure-first (D01)

Wrote `knowledgeV9OwnerCopy.test.tsx` against unmodified Knowledge chrome, then implemented Owner copy.

| ID | Negative | Surface | Observed |
|---|---|---|---|
| N1 | Unauthenticated Knowledge must fail-closed; no ingest; no fake Activate; no Twitter/X hero | Dual Track App `#/knowledge` | **fail then pass** — first run already **pass** (SessionGate). Re-run after Owner copy **pass**. |
| N2 | Authenticated Knowledge must use v9 Owner language, not HTTP-paste / Files-Import-Why English / primary Vite honesty wall | Dual Track `#/knowledge` | **fail then pass** — first run **fail** (`h2` was `Knowledge`, expected `当前项目资料`). After Owner copy **pass**. |
| N3 | Secret ingest and file-as-authority stay refused; 0 Activate / Twitter P0 | Dual Track `#/knowledge` Import | **fail then pass** — first run **fail** (`tab not found: 导入`, chrome still `Import`). After Owner copy **pass** (secret-shaped refuse, no `vault.import`). |

## Incremental validation log

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-06 | Claim lease + worktree | pass | local | `origin/main@b3f59161` | Knowledge-only paths. PROGRESS unique-next skipped. |
| 2026-09-06 | N1 unauthenticated fail-closed | pass | Dual Track vitest | dirty worktree | SessionGate; no ingest; no Activate; no Twitter. |
| 2026-09-06 | N2 Owner language vs HTTP-paste / English | fail | Dual Track vitest | pre-change | `h2`=`Knowledge`; expected `当前项目资料`. |
| 2026-09-06 | N3 secret ingest / Import tab | fail | Dual Track vitest | pre-change | `tab not found: 导入`. |
| 2026-09-06 | Owner copy `KnowledgePage.tsx` | pass | Dual Track vitest | dirty worktree | Title `当前项目资料`; tabs 项目资料/导入/为什么用这段/记忆; no Obsidian; no fake Activate. |
| 2026-09-06 | Knowledge + opcIa Dual Track | pass | Dual Track vitest | dirty worktree | 5 files, **44/44**. opcIa empty Knowledge asserts `知识已锁定`. |
| 2026-09-06 | Full Dual Track web vitest | pass | Dual Track vitest | dirty worktree | **77 files / 549 tests**. |
| 2026-09-06 | Locked Knowledge h2 + remaining Owner table copy | pass | Dual Track vitest | dirty follow-up | Empty Knowledge title `知识已锁定`. Labels/conflicts hidden until files exist. 5 files **44/44**. |
| 2026-09-06 | Draft PR [#339](https://github.com/agentkernel/cognitive-os/pull/339) | pass | GitHub | `626681a3` | Keep Draft. Required CI in progress (ubuntu/windows). |
| 2026-09-06 | D02 guest `/ui/` J5 | not-run | `B01-Desktop-Linux-002` | — | Guest `:48681` was not this lease while T03 held it. |
| 2026-09-12 | Fold `origin/main@ae1dc06a` (P15-T03 #338) | pass | worktree | `991408f8` | T03 done; D01 Dual Track consumed. Pushed; required CI [34663102994](https://github.com/agentkernel/cognitive-os/actions/runs/34663102994) in progress. |
| 2026-09-12 | Focused Dual Track Knowledge after fold | pass | Node jsdom `clients/pc/web` | `991408f8` | knowledgeV9OwnerCopy + knowledgeIa + knowledgeIngest + knowledgeMemory + opcIa **46/46**. |
| 2026-09-12 | Exact-revision UI dist | pass | local Dual Track `clients/pc/web` at pushed `991408f8` | `991408f8` | `pnpm build` → `index-3s2lrA_c.js` + `index-C9SPnPVx.css`. No kernel-server rebuild (UI-only). |
| 2026-09-12 | Guest `:48681` UI replace | pass | `B01-Desktop-Linux-002` | `991408f8` | Copied dist into `runtime/data/cognitiveos/ui/` (served) and `ui/`. Daemon PID **2740931** unchanged. `/ui/` GET 200 serves `index-3s2lrA_c.js`. Left `:48181` untouched (PID **166715**, `cos-current`). `cognitive dsh web` `:3080` **not-run**. |
| 2026-09-12 | J0 unauthenticated fail-closed | pass | host Chrome 140 → tunnel `/ui/#/knowledge` | `991408f8` / SPA `index-3s2lrA_c.js` | Session denied / Open Session. No `[data-page=opc-knowledge]`. 0 Activate. twitter=false |
| 2026-09-12 | J5 Knowledge vs v9 Owner copy | pass | guest `/ui/#/knowledge` after session | `991408f8` | `data-page=opc-knowledge` `data-knowledge-ia=v9`. Title `当前项目资料` (unlocked; live Project). Tabs 项目资料 / 导入 / 为什么用这段 / 记忆. Import: `input[name=vault-files]`; no `textarea[name=vault-body]`. No HTTP-paste wall. No Obsidian. 0 Activate. twitter=false. Vite not used. |
| 2026-09-12 | J2 Today regression | pass | guest `/ui/#/` | `991408f8` | h2 `今日`. 0 Activate. twitter=false |
| 2026-09-12 | J3 Projects regression | pass | guest `/ui/#/projects` | `991408f8` | 项目列表 / create chrome present. 0 Activate. twitter=false |
| 2026-09-12 | J10 no X/Twitter P0 | pass | guest `/ui/#/settings` | `991408f8` | Settings exists. twitter=false. 0 Activate |
| 2026-09-12 | Windows native daemon chrome JOURNEY | not-run | walk used host Chrome against forwarded guest `/ui/` | — | not Windows-native daemon chrome |

## Unique next (T04 only)

`P15-T04/D01` Dual Track is **done**. `P15-T04/D02` guest `/ui/` J5 + `JOURNEY-BROWSER-SYNC-01` (J0/J2/J3/J10) **walked** at `991408f8` / SPA `index-3s2lrA_c.js`. Report HEAD `1cfa5f69` required CI [34663663188](https://github.com/agentkernel/cognitive-os/actions/runs/34663663188) **SUCCESS**. Unique next: ready/merge [#339](https://github.com/agentkernel/cognitive-os/pull/339) after this closure commit's required CI, then claim / continue `P15-T05`. Leave `:48181` untouched. Do not claim T06.
