# P15-T04 Knowledge vs v9 — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P15-T04` / unique next `P15-T04/D02` guest `/ui/` J5. `P15-T04/D01` Dual Track Owner copy already implemented.
- Branch: `personal/P15-T04-knowledge-v9`
- Worktree: `D:\agent-kernel-wt-P15-T04`. Fold `origin/main@ae1dc06a` (P15-T03 #338).
- Lease: `lease/personal/P15-T04/knowledge-v9`
- Draft PR: [#339](https://github.com/agentkernel/cognitive-os/pull/339) (keep Draft until full acceptance)
- T03 closed; guest `:48681` is now this lease for D02. Leave `:48181` untouched. Do not claim T05–T06.
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
| 2026-09-12 | Fold `origin/main@ae1dc06a` (P15-T03 #338) | pass | worktree | merge resolving | T03 done; D01 Dual Track consumed; unique next D02. |

## Unique next (T04 only)

`P15-T04/D01` Dual Track is done. `P15-T04/D02` guest `/ui/` J5 + `JOURNEY-BROWSER-SYNC-01` (J0/J2/J10) on `:48681` after this fold is pushed. Leave `:48181` untouched. Keep Draft [#339](https://github.com/agentkernel/cognitive-os/pull/339). Do not claim T05–T06.
