# P7-T05/D21 Control Plane W9 System — rendered browser review

- 状态：running report（D21 rendered-review exit）
- 日期：2026-08-26
- 任务：`P7-T05` Non-blocking Web UI（Control Plane redesign）
- Slice：`P7-T05/D21` — Control Plane W9 System surface
- Slice 状态：`done` after this review
- Lease：`lease/personal/P7-T05/d14-rendered-review`
- Branch：`personal/P7-T05-d14-rendered-review`
- Draft PR：[cognitive-os#274](https://github.com/agentkernel/cognitive-os/pull/274)
- Reviewed tree：`32ebbbe9` (`feat(P7-T05): add the W9 System surface over status, doctor, and backup`)
- Change class：`implementation-only`
- Claim ceiling：`hypothesis`
- Non-claims：local browser observation only; not a product Gate; no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. `pnpm test` in `clients/pc/web` at the reviewed tree: **273/273 pass** (34 files).
2. `pnpm run build`: **pass** (129 modules). Bundle: CSS `index-Bbm_a9ng.css` 28.51 kB, JS `index-Cb7Zpcp6.js` 419.91 kB.
3. Bundle SHA-256 of that clean rebuild:
   - JS `sha256:5a1928f76d2f22f79acd7eb7d587ef9d73233ae0372534c7e2388e741ff0ea73`
   - CSS `sha256:2366a13a2af354e979047f19d89596b1ef9e9adf00bd1deca02683e3bfc48f5e`
4. Exclusive rendered browser review (one driver, one fixture server, CDP port 9348): headless Chrome **151.0.7922.174** over CDP against `http://127.0.0.1:8791/ui/`. Harness, fixture server, JSON report and screenshots stay in ignored `d:\tmp\cp-review-w4\` (not Git). `REVIEW_ONLY=system-`.
5. W9 System matrix: **15/15 cells** and **154/154 assertions**, with `overflow=0`, `clipped=0`, `consoleErrors=0`, and **0 contrast findings on screen and offscreen** in every cell.
6. No product finding. No UI patch after the review. One exclusive-run miss was harness copy (`system-doctor-stub` expected the code token `STUB_ROUTE`; Doctor uses the shared unavailable state and names `daemon front-door stub (R-1)`). Assertion aligned and the full matrix re-run **15/15**.

## Matrix coverage

- populated System at 1920/1440/1280/960 × light+dark: heading, standing claim ceiling, provider degraded, first conversation not ready, not “Under reconstruction”, no Gate-pass claim
- Doctor: `RESOURCE_HEALTH_NOT_PROBED` named unavailable (“not probed over HTTP”), guidance, redaction, CLI bundle only
- Doctor 200-stub: `daemon front-door stub (R-1)`, not a green doctor
- Stewardship backup class-A: archive receipt with excluded secrets 2 and sqlite copied false
- Restore 409: `HTTP 409 RESOURCE_BACKUP_TAMPERED` named; copy states live-apply and never auto-retries
- Session: expiry `unknown (BD-7)`; tab-local only
- About: gate/profile `not-claimed`; `cognitive doctor --bundle`
- keyboard/focus
- server-side allowlist includes `GET /personal/doctor` and `POST /management/resource/v1/backup`, `backup/preflight`, `restore`

## Honest not-run

Section deep links via `#/system?section=` were not the exclusive driver (cells click the subnav). Focused tests cover the sections. No Gate / release / Profile claim is made from doctor, about, or readiness copy.

## Findings

None.

## Next

`P7-T05/D21` is closed. Register and implement `P7-T05/D22` (Control Plane W10 command layer / ⌘K) on the same task branch/lease. Do not auto-claim P6 / P7-T06 / P7-T07. W11–W12 and the legacy `styles.css` / `#/tasks` retirement remain unstarted. P7-T05 task acceptance is not complete until those waves land.
