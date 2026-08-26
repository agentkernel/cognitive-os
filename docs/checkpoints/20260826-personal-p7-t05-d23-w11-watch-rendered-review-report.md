# P7-T05/D23 Control Plane W11 watch streaming — rendered browser review

- 状态：running report（D23 rendered-review exit）
- 日期：2026-08-26
- 任务：`P7-T05` Non-blocking Web UI（Control Plane redesign）
- Slice：`P7-T05/D23` — Control Plane W11 watch streaming + refresh policy
- Slice 状态：`done` after this review
- Lease：`lease/personal/P7-T05/d14-rendered-review`
- Branch：`personal/P7-T05-d14-rendered-review`
- Draft PR：[cognitive-os#274](https://github.com/agentkernel/cognitive-os/pull/274)
- Reviewed tree：`db599bfd` (`feat(P7-T05): wire Work detail Run to GET /task/watch`)
- Change class：`implementation-only`
- Claim ceiling：`hypothesis`
- Non-claims：local browser observation only; not a product Gate; no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. `pnpm test` in `clients/pc/web` at the reviewed tree: **295/295 pass** (37 files).
2. `pnpm run build`: **pass** (134 modules). Bundle: CSS `index-jcZzzPtK.css` 30.47 kB, JS `index-CsMgHj27.js` 439.65 kB.
3. Bundle SHA-256 of that clean rebuild:
   - JS `sha256:9786e0ba70561e3c2cfd64de615a74ad269ce656d80c054c56a7b3902e5caa9d`
   - CSS `sha256:d927acff5d0b00d4adb73b852c4afb74a1222c2087aa2e93eae5aa71193e3047`
4. Exclusive rendered browser review (one driver, one fixture server, CDP port 9351): headless Chrome **151.0.7922.174** over CDP against `http://127.0.0.1:8793/ui/`. Harness, fixture server, JSON report and screenshots stay in ignored `d:\tmp\cp-review-w4\` (not Git). `REVIEW_ONLY=watch-`.
5. W11 watch matrix: **15/15 cells** and **188/188 assertions**, with `overflow=0`, `clipped=0`, `consoleErrors=0`, and **0 contrast findings on screen and offscreen** in every cell.
6. No product finding. Two exclusive-run misses were harness copy (authority fixture uses `event_type: ADMITTED`, not `task.transition`; inventory watch copy lives in the inspector, while the inventory page already states the snapshot is unused). Assertions were aligned and the matrix re-run **15/15**.

## Matrix coverage

- unattached Work detail Run at 1920/1440/1280/960 × light+dark: unknown/not attached, not live; EventSource-bearer honesty; process-local 128-event ring; no `GET /task/watch` until attach
- attach: live + 15 s bounded poll; watch delta labeled `obs` on the observation lane only; authority lane unchanged; bearer not on the watch URL
- detach: disconnected; never cancelled a Task or stopped an Agent
- 409 `TASK_WATCH_RESUME_STALE`: stale gap, reconnect offered, completion stays unknown
- keyboard: visible stops have a focus ring; Attach watch is a real button
- Work inventory and Home stay manual: no watch GET; inventory names that the watch snapshot is not used

## Transport honesty

Native `EventSource` cannot set `Authorization`, and the task bearer must not enter the URL. Attach opens `GET /task/watch` as an authenticated fetch of the same SSE framing (`daemonFetch` + `parseSse` + kept `createWatchController`). The daemon's watch is snapshot-first, process-local, empty `tasks:[]`, and `Connection: close` after the snapshot, so an attached watch follows with a named 15 s bounded poll (OQ-2, Work detail only). Home and Work inventory stay manual-refresh.

## Honest not-run

Long-lived EventSource (a socket that never closes) is not available on this daemon: each watch GET ends after the snapshot. Bounded polling while attached is the live-within-reality path, not a fabricated continuous stream. Palette still does not attach/detach. No Gate / release / Profile claim.

## Findings

None.

## Next

`P7-T05/D23` is closed. Register and implement `P7-T05/D24` (Control Plane W12 accessibility / QA gate) on the same task branch/lease. Do not auto-claim P6 / P7-T06 / P7-T07. Legacy `styles.css` / `#/tasks` retirement remains unstarted after W12. P7-T05 task acceptance is not complete until those land. P7-T07 remains blocked on owner-only B01-W prerequisites.
