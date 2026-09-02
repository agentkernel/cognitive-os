# P13-T12/D01 Visual UI spec + v9 comparison checklist — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. Documentation-only. A7: local/CI is not Gate.

- Task: `P13-T12` / slice `P13-T12/D01` (`P13-T12/D02` is not claimed by this slice)
- Branch: `personal/P13-T12-D01-visual-spec` (worktree `D:/agent-kernel-wt-p13-t12` from `origin/main@a0465653`)
- Lease: `lease/personal/P13-T12/visual-spec`
- Change class: `implementation-only` documentation (a visual specification and a comparison checklist for an unchanged product/IA contract; no product code, contract, test, CSS, or canvas change)
- Deliverables: [`personal-2.0-opc-visual-ui-spec.md`](../../personal/docs/architecture/personal-2.0-opc-visual-ui-spec.md), [`personal-2.0-opc-v9-ui-comparison-checklist.md`](../../personal/docs/architecture/personal-2.0-opc-v9-ui-comparison-checklist.md)
- Unique next: **in-progress** — see the last unit row.

This report is documentation evidence only. It cannot establish rendered accessibility, host-theme contrast, 200% layout, NVDA behaviour, Windows support, Gate, release, Profile, T15 N=15 acceptance, or Agent-benefit. Every checklist judgement stays `not-run` until `P13-T12/D02` records it on an exact daemon `/ui/` revision.

## Inputs read (read-only)

| Source | Role in this slice |
|---|---|
| `AGENTS.md`, `PROJECT-IDENTITY.md`, `AXIOMS.md`, Operating Model §2.1–§2.4/§3.1/§4/§7 | governance; stop conditions; report discipline |
| `PROGRESS.md` Current snapshot; `PARALLEL-LANES.md` §2/§3; `PERSONAL-DEVELOPMENT-PLAN.md` Phase 13; `plan.md` `### P13-T12`; `PERSONAL-TEST-ENVIRONMENTS.md` §1.1/§3/§5.2 | claim rules, card acceptance, validation routing |
| `clients/docs/design/opc-2.0/README.md`, `00-maintenance-index.md` (table A 19 modules, table B sources), `09-state-accessibility-and-visual-system.md`, `10-component-map-and-prototype-flows.md`, `11-design-to-code-and-backend-matrix.md` | design truth |
| `clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx` (all 19 scenes, `StateKey`/`SurfaceKey`, `SURFACE_CONTEXT`, `StateBanner`, dialogs, `Conversation`, shell CSS incl. `prefers-reduced-motion` / `prefers-contrast`) | frozen prototype; read only, never modified; v8 untouched |
| `personal/docs/product/web-ui-design.md`, `personal-2.0-scope.md` §3.1–§3.6, `user-journeys.md`, `opc-product-model.md`, `product-design.md` | product truth (REFRAME-owned; not written) |
| `personal/docs/architecture/personal-2.0-opc-v9-implementation-mapping.md` (§6.0 hashes, §6.1 scene rows, §6.4 nine states), `web-ui-architecture.md`, `personal-2.0.0-dev-prep-index.md` | `/ui/` mapping and convention model (single mixed zh/en file) |
| `clients/pc/web/src/tokens.css`, `app.css`, `router.tsx`, `shell/*`, `views/opc/*`, `components/states.tsx`, `state/stateMap.ts` | existing design system and component vocabulary the spec builds on |
| `clients/docs/design/legacy-control-plane-20260827/09-…apple-design-principles.md`, `11-…design-system.md`, `22-…state-system.md`, `24-…visual-direction.md` | earlier Apple-led principles (tracked paths; the untracked `clients/docs/design/09|11|24-*.md` copies named in the task brief do not exist on `main`) |
| `personal/handbook/_meta/source-map.json`, `source-coverage.json`, `sync-policy.md` | `personal/docs/architecture/README.md` is a mapped source (`personal-2-baseline`); new `personal/docs/**` files are classified `legacy-docs` |

## Units

| Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|
| `git fetch origin`; new worktree `D:/agent-kernel-wt-p13-t12` on `personal/P13-T12-D01-visual-spec`; main checkout untouched (only known owner-local untracked files present) | **pass** | `DEV-WIN-GNU-01` | `origin/main@a0465653` | Sibling worktrees `-p13-t02`, `-p13-t03`, `-doc-p13-drift` observed, not touched. `pnpm install --frozen-lockfile --offline` OK. `.githooks` registered. |
| Location check: `plan.md` `### P13-T12` and the formal plan do not mandate a REFRAME-owned path; deliverables placed under `personal/docs/architecture/` | **pass** | `DEV-WIN-GNU-01` | worktree | No lease overlap with `lease/personal/DOC-PERSONAL-2.0-OPC-REFRAME/product-prototype-docs`; no REFRAME-owned file is written or linked into. |
| Claim `lease/personal/P13-T12/visual-spec` (PARALLEL-LANES §3 row; PROGRESS Active task lease row; `P13-T12` → `in-progress`; `P13-T12/D01` → `in-progress`; Layer 1 167/132/1/1/17/35; plan status line, Phase 13 summary 13/1/1/0/11, totals, roadmap, 三栏 row) | **pass** | `DEV-WIN-GNU-01` | worktree | `P13-T12/D02` left `ready`. Ledger edited row-locally; unrelated rows preserved. |
| Write `personal/docs/architecture/personal-2.0-opc-visual-ui-spec.md` (§2 stance, §3 typography, §4 spacing/density, §5 light/dark/high-contrast roles + contrast targets + nine-state tone mapping, §6 locked three columns / narrow horizontal scroll / 200%, §7 focus ring + keyboard order + widget patterns + NVDA key paths, §8 motion, §9 nine states × nine surfaces with host selectors, §10 component states, §12 proposed tokens, §13 drift observed 12 items) | **pass** | `DEV-WIN-GNU-01` | worktree | Expressed against existing `--cp-*` names and the seven `StateCategory`s; no CSS/TSX edit; no IA change; states explicitly it is not a second design system. Drift a (current `app.css` stacks columns ≤ 1279 px vs product no-stack rule) has no owning P13 card yet — see the final summary. |
| Write `personal/docs/architecture/personal-2.0-opc-v9-ui-comparison-checklist.md` (§1 19/19 table-A modules × table-B authority × v9 expectation × `/ui/` route/selector × daemon fact / honest state × owning card × judgement; Grid A 81 nine × nine cells; Grid B 57 keyboard/focus cells; Grid C 36 200%/narrow cells; Grid D 40 theme cells incl. `forced-colors`; Grid E 10 NVDA paths; D02 pin block; module-level drift; counters) | **pass** | `DEV-WIN-GNU-01` | worktree | Every judgement `not-run` (0 pass / 0 fail). Honest-state rule for missing backend written into §0. Canvas never modified; v8 untouched. |
| Link from `personal/docs/architecture/README.md` (two chapter rows); bilingual `dev.architecture-overview` pointer + two new `sources` entries; `plan.md` `### P13-T12` D01 progress note; `personal-trace.yaml` PERS-PR-052 comment (evidence_status stays `not-run`); `node tools/src/fill-handbook-fingerprints.mjs` | **pass** | `DEV-WIN-GNU-01` | worktree | 4 pages re-fingerprinted (`dev.architecture-overview` ×2 with content; `user.system-overview` ×2 fingerprint-only because the architecture README is one of their sources). Dev-prep index **not** edited (avoids `DOC-P13-DRIFT-FIX` overlap). No REFRAME-owned file touched. Lease row extended with the two `system-overview` pages. |
| `pnpm run check:consistency` | **pass** | `DEV-WIN-GNU-01` | worktree | 275 requirements; leases verified (new row + PROGRESS reference + `P13-T12/D01` in-progress; Layer 1 167/132/1/1/17/35 matches plan). |
| `pnpm run check:handbook` (first run, before the new docs were staged) | **fail** (expected) | `DEV-WIN-GNU-01` | worktree | HB005/HB006 ×8: the new architecture docs were untracked, so the tracked-only link/source check rejected them. Not a content defect; recorded because a negative result is a result. |
| `git add <two new docs> <report>` then `pnpm run check:handbook` | **pass** | `DEV-WIN-GNU-01` | worktree | 58 × 2 documents OK; coverage/link/fingerprint/status/secret checks verified. Confirms the handbook checker is already tracked-only (`git ls-files` incl. index). |
| `node tools/src/generate-handbook.mjs --check` | **pass** | `DEV-WIN-GNU-01` | worktree | 18 generated pages byte-identical. |
| `pnpm run check:rules` | **pass** | `DEV-WIN-GNU-01` | worktree | 4 rules, 88 path references, 5 known local-only warnings, 0 failures. |
| Manual tracked-only link check (every relative link in the two new docs, the architecture README and this report resolved against `git ls-files` + index) | **pass** | `DEV-WIN-GNU-01` | worktree | 128 relative links checked; 127 resolve to tracked files; the one remaining hit is the pre-existing README directory link `../../../core/specs` (a tracked directory, not a file; not written by this slice). No link targets any untracked local file (`clients/docs/design/opc-2.0/14–18, 21–26, window-c-*.md`, `docs/plan/p11-plan-review-and-optimization.md`). `P0-T09` will make this mechanical. |
| `git add -u`; `git diff --cached --check`; `node tools/src/docs-sync-gate.mjs --staged` | **pass** | `DEV-WIN-GNU-01` | worktree | Whitespace clean. Gate routes `personal-2-baseline`, `personal-2-opc-rebaseline`, `handbook-itself`; handbook check set green; no `DOCS_IMPACT_NONE` needed. |
| `pnpm --filter @cognitiveos/repo-tools test` | **pass** | `DEV-WIN-GNU-01` | worktree | 115/115. |
| product code / contracts / tests / CSS / canvas | **not-run** | documentation-only | — | allowed by the D01 exit; nothing edited under `clients/pc/web/src` or `clients/docs/design/opc-2.0/` |
| NVDA / 200% / host-theme / nine × nine rendered cells | **not-run** | Requires-environment | — | owned by `P13-T12/D02`; every checklist judgement is `not-run` |
| `DEV-WIN-GNU-01` cargo build / test / Clippy / link | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — | not a product fail; no Rust surface changed |
