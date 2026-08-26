# P7-T05/D25 Control Plane legacy `styles.css` / `#/tasks` retirement — rendered browser review

- 状态：running report（D25 rendered-review exit）
- 日期：2026-08-26
- 任务：`P7-T05` Non-blocking Web UI（Control Plane redesign）
- Slice：`P7-T05/D25` — retire the legacy `styles.css` / `#/tasks` governed-task diagnostics page
- Slice 状态：`done` after this review
- Lease：`lease/personal/P7-T05/d14-rendered-review`
- Branch：`personal/P7-T05-d14-rendered-review`
- Draft PR：[cognitive-os#274](https://github.com/agentkernel/cognitive-os/pull/274)
- Reviewed tree：`872074bf` (`feat(P7-T05): retire styles.css and the #/tasks diagnostics page`)
- Change class：`implementation-only`
- Claim ceiling：`hypothesis`
- Non-claims：local browser observation only; not a product Gate; no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. `pnpm test` in `clients/pc/web` at the reviewed tree: **306/306 pass** (38 files).
2. `pnpm run build`: **pass** (135 modules). Bundle: CSS `index-BdltJIx_.css` 22.93 kB, JS `index-CEY4LiL3.js` 436.50 kB.
3. Bundle SHA-256 of that clean rebuild:
   - JS `sha256:0eafc04968c8aa8ee4522fb8e7fc89cd2f1b1a52e58a5e01aecf1c3744e48f05`
   - CSS `sha256:9a6e96d6793800b284d8d3ba61f58b9b036d7a20482c95815ba4162a09adaa31`
4. Exclusive rendered browser review (one driver, one fixture server, CDP port 9357): headless Chrome **151.0.7922.174** over CDP against `http://127.0.0.1:8797/ui/`. Harness, fixture server, JSON report and screenshots stay in ignored `d:\tmp\cp-review-w4\` (not Git). `REVIEW_ONLY=retire-`.
5. D25 retirement matrix: **15/15 cells** and **115/115 assertions**, with `overflow=0`, `clipped=0`, `consoleErrors=0`, and **0 contrast findings on screen and offscreen** in every cell.
6. No product finding. `#/tasks` redirects to Work; `styles.css` class hooks (`.panel`, `.state-note`, `.page-head`, `nav.side`, `.shell`) are absent from Control Plane markup.

## Matrix coverage

- `#/tasks` → `#/work` light and dark; old "Tasks, Effects, Evidence" / Watch poll / Simulate cursor gap copy gone
- Work honesty no longer links `#/tasks`; watch attach is named on Work detail Run
- Home 1920 light/dark; Work 1100 (top-strip band); Work 960 dark
- Session form still issues both channels; skip link remains
- Work detail Run light/dark: Attach watch present, Watch poll absent
- designed 404; `#/bindings` still folds into Providers
- reduced-motion Work; `prefers-contrast: more` Home
- `#/work/new` remains the governed creation replacement

## Honesty

`styles.css` is no longer imported. Control Plane CSS is `tokens.css` + `app.css` (`--cp-*` only). `#/tasks` is a replace redirect, not a second task surface. Watch, admit, and observation stay on `#/work` / `#/work/new` / `#/work/:taskRef`. No invented task HTTP.

## Honest not-run

Live daemon `/ui/` on linux-002 was not re-driven in this slice (fixture-server review only). Coarse-pointer 44 px targets were not emulated. No Gate / release / Profile claim.

## Findings

None carried.

## Next

`P7-T05/D25` is closed. This was the last registered Control Plane redesign slice. Next is P7-T05 task acceptance, Draft PR [#274](https://github.com/agentkernel/cognitive-os/pull/274) ready/merge, then lease/branch/main reconciliation. Do not auto-claim P6 / P7-T06 / P7-T07 until that closure finishes. P7-T07 remains blocked on owner-only B01-W prerequisites.
