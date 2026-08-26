# P7-T05/D16 Control Plane W7 Resources hub — rendered browser review

- 状态：running report（D16 rendered-review exit）
- 日期：2026-08-25
- 任务：`P7-T05` Non-blocking Web UI（Control Plane redesign）
- Slice：`P7-T05/D16` — Control Plane W7 Resources family hub
- Slice 状态：`done` after this review
- Lease：`lease/personal/P7-T05/d14-rendered-review`
- Branch：`personal/P7-T05-d14-rendered-review`
- Draft PR：[cognitive-os#274](https://github.com/agentkernel/cognitive-os/pull/274)
- Reviewed tree：`6c33c94f` (`feat(P7-T05): replace the Resources placeholder with the W7 hub`)
- Change class：`implementation-only`
- Claim ceiling：`hypothesis`
- Non-claims：local browser observation only; not a product Gate; no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. `pnpm test` in `clients/pc/web` at the reviewed tree: **221/221 pass** (24 files; 208 pre-existing + 13 Resources hub tests).
2. `pnpm run build`: **pass** (120 modules). Bundle: CSS `index-DaMO6LFF.css` 28.01 kB, JS `index-ZuA9qpSo.js` 368.55 kB.
3. Bundle SHA-256 of that clean rebuild:
   - JS `sha256:35eff7f0466d8cc4817baa2e2407b49d8089cb7714c582cd524395761f12cacf`
   - CSS `sha256:ad5669f7614ae9724b72ac779db12d63305cf8e16619dd49f5402e9865550eea`
4. Exclusive rendered browser review (one driver, one fixture server, CDP port 9343): headless Chrome **151.0.7922.174** over CDP against `http://127.0.0.1:8791/ui/`. Harness, fixture server, JSON report and screenshots stay in ignored `d:\tmp\cp-review-w4\` (not Git). `REVIEW_ONLY=resources-`.
5. W7 matrix: **12/12 cells** and **215/215 assertions**, with `overflow=0`, `clipped=0`, `consoleErrors=0`, and **0 contrast findings on screen and offscreen** in every cell.
6. No product finding. No UI patch after the review.

## Matrix coverage

- populated hub at 1920/1440/1280/960 × light+dark: four family rows, memory admitted count, no invented tombstones, skill bindings (not packages), tool quarantine, envelope limit 64, not a card wall, Context is not a standalone browser
- authoritative empty memory/skill/tool vs Context still projection-only (not `STUB_ROUTE`)
- denied / stub / empty / projection-only as four distinct statements on one page (memory 401, skill stub, tool empty, context projection-only)
- tool list at envelope bound (limit 64)
- keyboard/focus on the hub; Work is a real link
- no Remember / Import / Enable / Forget / Quarantine controls in any cell
- server-side route allowlist includes W7 `list?family=` reads

## Findings

None.

## Next

`P7-T05/D16` is closed. Register and implement `P7-T05/D17` (Control Plane W7 Memory family page) on the same task branch/lease. Do not auto-claim P6 / P7-T06 / P7-T07. Skills/Tools family pages, W8–W12, and the legacy `styles.css` / `#/tasks` retirement remain unstarted. P7-T05 task acceptance is not complete until those waves land.
