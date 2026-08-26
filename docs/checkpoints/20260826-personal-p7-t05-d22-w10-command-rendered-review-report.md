# P7-T05/D22 Control Plane W10 command layer — rendered browser review

- 状态：running report（D22 rendered-review exit）
- 日期：2026-08-26
- 任务：`P7-T05` Non-blocking Web UI（Control Plane redesign）
- Slice：`P7-T05/D22` — Control Plane W10 command layer (⌘K)
- Slice 状态：`done` after this review
- Lease：`lease/personal/P7-T05/d14-rendered-review`
- Branch：`personal/P7-T05-d14-rendered-review`
- Draft PR：[cognitive-os#274](https://github.com/agentkernel/cognitive-os/pull/274)
- Reviewed tree：`5f4185fd` (`feat(P7-T05): add the W10 command palette over loaded projections`)
- Change class：`implementation-only`
- Claim ceiling：`hypothesis`
- Non-claims：local browser observation only; not a product Gate; no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. `pnpm test` in `clients/pc/web` at the reviewed tree: **285/285 pass** (36 files).
2. `pnpm run build`: **pass** (131 modules). Bundle: CSS `index-BLKDlklC.css` 30.25 kB, JS `index-C9wixoMr.js` 431.80 kB.
3. Bundle SHA-256 of that clean rebuild:
   - JS `sha256:dff714733e1f7fca0cb2ddecf0fb89d6c12471a8950861b15d44d9706dc67b6b`
   - CSS `sha256:9ec28d81ea457545b26f442385e669c18ee9cc90da56c16b966196bb42049f81`
4. Exclusive rendered browser review (one driver, one fixture server, CDP port 9350): headless Chrome **151.0.7922.174** over CDP against `http://127.0.0.1:8792/ui/` (8791 was already occupied by an earlier fixture). Harness, fixture server, JSON report and screenshots stay in ignored `d:\tmp\cp-review-w4\` (not Git). `REVIEW_ONLY=palette-`.
5. W10 command-layer matrix: **15/15 cells** and **156/156 assertions**, with `overflow=0`, `clipped=0`, `consoleErrors=0`, and **0 contrast findings on screen and offscreen** in every cell.
6. No product finding. One exclusive-run miss was harness copy (`Actions`/`Destinations` vs CSS `text-transform:uppercase` innerText `ACTIONS`/`DESTINATIONS`). Group labels were aligned to the design anatomy and the matrix re-run **15/15**.

## Matrix coverage

- populated palette at 1920/1440/1280/960 × light+dark: dialog present, BD-6 honesty, BD-3 partial inventory, class-C absent copy, ACTIONS/DESTINATIONS groups, no “Cancel task”
- known-object search over the Home-loaded task list (no server search)
- no-results names the BD-3 partial inventory
- destination navigate to Work closes the palette
- class-B alert acknowledge keeps the receipt and leaves the palette open
- Work-detail context: Copy task ref, Open evidence, Open Run; watch is not a palette mutation
- keyboard: query takes focus with a visible ring
- Control+K opens the palette
- server-side allowlist: palette ack reuses `POST /management/alerts/acknowledge`; no new route

## Honest not-run

Clipboard write was not clicked in the exclusive browser (permission surface); focused tests cover class-B copy. Watch attach/detach is not executed in the palette — W11 owns `EventSource`. No Gate / release / Profile claim.

## Findings

None.

## Next

`P7-T05/D22` is closed. Register and implement `P7-T05/D23` (Control Plane W11 watch streaming + refresh policy) on the same task branch/lease. Do not auto-claim P6 / P7-T06 / P7-T07. W12 and the legacy `styles.css` / `#/tasks` retirement remain unstarted. P7-T05 task acceptance is not complete until those waves land.
