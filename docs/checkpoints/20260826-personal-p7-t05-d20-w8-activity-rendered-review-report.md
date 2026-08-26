# P7-T05/D20 Control Plane W8 Activity — rendered browser review

- 状态：running report（D20 rendered-review exit）
- 日期：2026-08-26
- 任务：`P7-T05` Non-blocking Web UI（Control Plane redesign）
- Slice：`P7-T05/D20` — Control Plane W8 Activity evidence stream
- Slice 状态：`done` after this review
- Lease：`lease/personal/P7-T05/d14-rendered-review`
- Branch：`personal/P7-T05-d14-rendered-review`
- Draft PR：[cognitive-os#274](https://github.com/agentkernel/cognitive-os/pull/274)
- Reviewed tree：`acc5814e` (`feat(P7-T05): add the W8 Activity evidence stream over real sources`)
- Change class：`implementation-only`
- Claim ceiling：`hypothesis`
- Non-claims：local browser observation only; not a product Gate; no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. `pnpm test` in `clients/pc/web` at the reviewed tree: **267/267 pass** (32 files).
2. `pnpm run build`: **pass** (128 modules). Bundle: CSS `index-Bbm_a9ng.css` 28.51 kB, JS `index-s3CDxXWQ.js` 408.53 kB.
3. Bundle SHA-256 of that clean rebuild:
   - JS `sha256:c7cb6909355a872b57f4386921c4a3acc69718b683e2907e7739111662e3e1dc`
   - CSS `sha256:2366a13a2af354e979047f19d89596b1ef9e9adf00bd1deca02683e3bfc48f5e`
4. Exclusive rendered browser review (one driver, one fixture server, CDP port 9347): headless Chrome **151.0.7922.174** over CDP against `http://127.0.0.1:8791/ui/`. Harness, fixture server, JSON report and screenshots stay in ignored `d:\tmp\cp-review-w4\` (not Git). `REVIEW_ONLY=activity-`.
5. W8 Activity matrix: **15/15 cells** and **170/170 assertions**, with `overflow=0`, `clipped=0`, `consoleErrors=0`, and **0 contrast findings on screen and offscreen** in every cell.
6. No product finding. No UI patch after the review.

## Matrix coverage

- populated Activity at 1920/1440/1280/960 × light+dark: seven-kind labels as text, BD-5 coverage banner, unacked alert, `key.rotate` as Change, `account.create` as Event, audit age unknown
- authoritative empty: "Nothing recorded in this view yet" with the banner still present
- partial: named `AUDIT_UNAVAILABLE`; alerts still render; not coerced to empty
- daemon 200-stub: `STUB_ROUTE`, not an empty stream
- alert ack inline: persistent receipt; ack becomes Intervention
- inspect: source named, no chat transcript
- bounded window: showing 50 of 54
- keyboard/focus
- server-side allowlist includes `GET /management/audit` and `POST /management/alerts/acknowledge`

## Honest not-run

Session-observed task evidence/effects in the browser: focused tests cover admission rows and Task-channel probes; the exclusive review did not inject `noteObservedTask` into the page, so that path is `not-run` here, not claimed as a browser pass.

## Findings

None.

## Next

`P7-T05/D20` is closed. Register and implement `P7-T05/D21` (Control Plane W9 System) on the same task branch/lease. Do not auto-claim P6 / P7-T06 / P7-T07. W10–W12 and the legacy `styles.css` / `#/tasks` retirement remain unstarted. P7-T05 task acceptance is not complete until those waves land.
