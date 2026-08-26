# P7-T05/D18 Control Plane W7 Skills family page — rendered browser review

- 状态：running report（D18 rendered-review exit）
- 日期：2026-08-25
- 任务：`P7-T05` Non-blocking Web UI（Control Plane redesign）
- Slice：`P7-T05/D18` — Control Plane W7 Skills family page
- Slice 状态：`done` after this review
- Lease：`lease/personal/P7-T05/d14-rendered-review`
- Branch：`personal/P7-T05-d14-rendered-review`
- Draft PR：[cognitive-os#274](https://github.com/agentkernel/cognitive-os/pull/274)
- Reviewed tree：`27e454a3` (`feat(P7-T05): add the W7 Skills family page over bindings`)
- Change class：`implementation-only`
- Claim ceiling：`hypothesis`
- Non-claims：local browser observation only; not a product Gate; no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. `pnpm test` in `clients/pc/web` at the reviewed tree: **241/241 pass** (28 files).
2. `pnpm run build`: **pass** (124 modules). Bundle: CSS `index-DaMO6LFF.css` 28.01 kB, JS `index-JuXiK6iW.js` 390.05 kB.
3. Bundle SHA-256 of that clean rebuild:
   - JS `sha256:c1d64358137ac49a8a63c6f9600a5fa6561bf79bc92f4fd365fd817f8a983e96`
   - CSS `sha256:ad5669f7614ae9724b72ac779db12d63305cf8e16619dd49f5402e9865550eea`
4. Exclusive rendered browser review (one driver, one fixture server, CDP port 9345): headless Chrome **151.0.7922.174** over CDP against `http://127.0.0.1:8791/ui/`. Harness, fixture server, JSON report and screenshots stay in ignored `d:\tmp\cp-review-w4\` (not Git). `REVIEW_ONLY=skills-`.
5. W7 Skills matrix: **12/12 cells** and **135/135 assertions**, with `overflow=0`, `clipped=0`, `consoleErrors=0`, and **0 contrast findings on screen and offscreen** in every cell.
6. No product finding. No UI patch after the review.

## Matrix coverage

- populated Skills page at 1920/1440/1280/960 × light+dark: binding rows, list is bindings not packages, standing content≠permission caption, envelope limit 64, Preview import/bind present, Revoke not enabled until explain
- authoritative empty: how bindings arrive via Import; still not a package list
- explain after Inspect: revision/package/manifest digest, durable-revocation consequence
- explain 404 named `RESOURCE_SKILL_BINDING_NOT_FOUND`, not an empty family
- keyboard/focus
- server-side allowlist includes `skill/binding/explain`, `skill/import`, `skill/bind`, `skill/binding/revoke`
- Resources hub Skills row is **browse** → `#/resources/skill`; Import/Bind/Revoke stay off the hub

## Findings

None.

## Next

`P7-T05/D18` is closed. Register and implement `P7-T05/D19` (Control Plane W7 Tools family page) on the same task branch/lease. Do not auto-claim P6 / P7-T06 / P7-T07. W8–W12 and the legacy `styles.css` / `#/tasks` retirement remain unstarted. P7-T05 task acceptance is not complete until those waves land.
