# P7-T05/D15 Control Plane W6 Agents dossier — rendered browser review

- 状态：running report（D15 rendered-review exit）
- 日期：2026-08-25
- 任务：`P7-T05` Non-blocking Web UI（Control Plane redesign）
- Slice：`P7-T05/D15` — Control Plane W6 Agents dossier
- Slice 状态：`done` after this review
- Lease：`lease/personal/P7-T05/d14-rendered-review`
- Branch：`personal/P7-T05-d14-rendered-review`
- Draft PR：[cognitive-os#274](https://github.com/agentkernel/cognitive-os/pull/274)
- Reviewed tree：`633215d9` (`feat(P7-T05): replace the Agents placeholder with the W6 dossier`)
- Change class：`implementation-only`
- Claim ceiling：`hypothesis`
- Non-claims：local browser observation only; not a product Gate; no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. `pnpm test` in `clients/pc/web` at the reviewed tree: **208/208 pass** (22 files; 192 pre-existing + 16 Agents tests).
2. `pnpm run build`: **pass** (117 modules). Bundle: CSS `index-Bm7b49tB.css` 27.88 kB, JS `index-CDRSjXG_.js` 363.93 kB.
3. Bundle SHA-256 of that clean rebuild:
   - JS `sha256:ff633b3debeb9aa6303bb1e201118ed3d9b4054adc908e79ba243752f2c7a0df`
   - CSS `sha256:551403f32b83fb9f651bd745de929a634e2eaa6ceec7699df646137f2e620f6a`
4. Exclusive rendered browser review (one driver, one fixture server, CDP port 9342): headless Chrome **151.0.7922.174** over CDP against `http://127.0.0.1:8791/ui/`. Harness, fixture server, JSON report and screenshots stay in ignored `d:\tmp\cp-review-w4\` (not Git). `REVIEW_ONLY=agents-`.
5. W6 matrix: **15/15 cells** and **213/213 assertions**, with `overflow=0`, `clipped=0`, `consoleErrors=0`, and **0 contrast findings on screen and offscreen** in every cell.
6. No product finding. No UI patch after the review.

## Matrix coverage

- populated inventory at 1920/1440/1280/960 × light+dark: pi and dsh rows, callable binding, projection-only runtime list, no Wave-6 placeholder
- unbound inventory (empty bindings + INACTIVE dsh): “no binding — this agent cannot call a model”; no invented `task://`
- dsh dossier: class-C header, `RESOURCE_MANAGER_NOT_FOUND` identity gap, process-liveness caption, `cognitive agent-pause`, capability annotation, dsh runtime + `candidate_only`
- pi dossier: current work named BD-2/BD-3; no dsh runtime section; `cognitive agent-stop`
- designed object-404 for `#/agents/nope` (no fabricated overview)
- bindings 401 as session denied
- dsh CRASHED + `cognitive dsh` class-C restart path
- keyboard/focus on the inventory
- no Pause/Resume/Stop/Restart/Quarantine buttons in any cell
- server-side route allowlist includes W6 reads (`/management/agent-bindings`, `/personal/dsh/runtime`, inspect, accounts, tool/exposure)

## Findings

None. The earlier exclusive W5 log (`run-9340.log`) that exited 1 was a harness-copy miss on `detail-stub` and is not W6 evidence; D14 already closed that cell after a recheck.

## Next

`P7-T05/D15` is closed. Register and implement `P7-T05/D16` (Control Plane W7 Resources family hub — four-row index, not the full Memory/Skills/Tools pages) on the same task branch/lease. Do not auto-claim P6 / P7-T06 / P7-T07. W8–W12 and the legacy `styles.css` / `#/tasks` retirement remain unstarted. P7-T05 task acceptance is not complete until those waves land.
