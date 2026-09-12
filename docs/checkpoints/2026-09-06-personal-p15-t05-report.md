# P15-T05 Settings / Model Connections vs v9 — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P15-T05` / slice `P15-T05/D01` Dual Track **pass** 559/559. `P15-T05/D02` guest `/ui/` J8/J12 next.
- Branch: `personal/P15-T05-settings-v9`
- Worktree: `D:\agent-kernel-wt-P15-T05`
- Lease: `lease/personal/P15-T05/settings-v9`
- Fold: `origin/main@293e3cd4` (merged P15-T04 PR [#339](https://github.com/agentkernel/cognitive-os/pull/339))
- Change class: `implementation-only` (Owner-facing Settings / Model Connections copy vs frozen v9). SecretStore handover stays. No `core/specs`. No numbered migration. Canvas v9 file not overwritten. AXIOMS.md not rewritten. No `app.css` shell tokens.
- Claim ceiling: `hypothesis`
- Product origin: daemon-served `/ui/`. Frozen canvas v9 = design authority and completion target. Vite is not the product origin.
- Evaluation routing: **OFF**
- Do not claim T06. Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334). Do not reopen P14-T07. Leave `:48181` untouched.

## Failure-first (D01)

| ID | Negative | Surface | Observed |
|---|---|---|---|
| N1 | Unauthenticated Settings must fail-closed; no hub; no fake Connect; no Twitter/X hero | Dual Track App `#/settings` | **pass** without a prior fail: Session paste-bootstrap (`Session denied` / `Open Session`); no `[data-page=opc-settings]`; no `[data-region=opc-model-connections]` |
| N2 | Authenticated Settings must use v9 Owner language, not English jargon wall / Vite lecture as primary chrome | Dual Track `#/settings` | **fail** then **pass** (pre-fold worktree): PageHeader was `Settings` / English lede / `Model Connections` / `Hand key to SecretStore` / primary Vite honesty. After change: title `设置`; lede `连接模型 · 收回本周不再问 · 通知恢复`; hub `模型连接` / `供应商模板` / `密钥（一次性交接）` / `交接密钥到 SecretStore`; `通知与恢复`; `本周不再问`; secondary honesty; 9×9 closed; 0 Connect / Activate / Twitter P0 |
| N3 | Model Connections hub stays honest: no fake key, no Provider secret in UI, no Connect, no Activate | Dual Track `#/settings/model-connections` | **fail** then **pass** (pre-fold worktree): same English title. After change: same Owner chrome as `#/settings`; typed `sk-live-must-never-render` never enters `textContent`; password input; no `ss://provider/`; submit disabled until key; no Connect |

## Incremental validation log

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-06 | D01 Dual Track `settingsOwnerCopy.test.tsx` (failure-first) | **fail** 2/3 then **pass** 3/3 | Node jsdom `clients/pc/web` | pre-fold worktree | N1 already green (SessionGate). N2/N3 expected `设置`, received `Settings` |
| 2026-09-06 | D01 Dual Track Settings pack | **pass** 24/24 (4 files) | Node jsdom `clients/pc/web` | pre-fold worktree | Owner copy 3/3; Model Connections 5/5; connections/retract 9/9; Owner chrome IA 7/7 |
| 2026-09-06 | D01 Dual Track full `clients/pc/web` vitest | **pass** 549/549 (77 files) | Node jsdom `clients/pc/web` | pre-fold worktree | +3 Owner-copy tests vs then-current T02 546/546. Unauth fail-closed; no Twitter P0; no fake Connect/Activate; SecretStore handover stays; 9×9 hidden |
| 2026-09-12 | Fold `origin/main@293e3cd4` (T04 #339) | **pass** | Git | `293e3cd4` | Fast-forward. Adopted uncommitted Settings files (A8). Stale claim docs discarded. Guest `:48681` now this lease. `:48181` untouched. |
| 2026-09-12 | D01 Dual Track Settings pack after fold | **pass** 24/24 (4 files) | Node jsdom `clients/pc/web` | worktree on `293e3cd4`+ | Owner copy 3/3; Model Connections 5/5; connections/retract 9/9; Owner chrome IA 7/7 |
| 2026-09-12 | D01 Dual Track full `clients/pc/web` vitest after fold | **pass** 559/559 (79 files) | Node jsdom `clients/pc/web` | worktree on `293e3cd4`+ | T03 553 + T04 Knowledge Owner-copy + T05 +3. Unauth fail-closed; no Twitter P0; no fake Connect/Activate; SecretStore handover stays; 9×9 hidden |
| 2026-09-12 | D02 guest `/ui/` J8/J12 | **not-run** | `B01-Desktop-Linux-002` `:48681` | — | After D01 Dual Track + push |

## Unique next (T05 only)

Continue `P15-T05/D01` Dual Track after the T04 fold, then `P15-T05/D02` guest `/ui/` J8/J12 + `JOURNEY-BROWSER-SYNC-01`. Do not claim T06. Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334).
