# P7-T05/D14 Control Plane W5 Work detail — rendered browser review

- 状态：running report（D14 rendered-review exit）
- 日期：2026-08-25
- 任务：`P7-T05` Non-blocking Web UI（Control Plane redesign）
- Slice：`P7-T05/D14` — Control Plane W5 Work detail + composed Run timeline
- Slice 状态：`done` after this review
- Lease：`lease/personal/P7-T05/d14-rendered-review`
- Branch：`personal/P7-T05-d14-rendered-review`
- Reviewed tree：`main@b77a02435037ff91b664f6c360bd82e853fdcd7c` (`clients/pc/web/` carries W5 at clients `d7f68164`)
- Change class：`implementation-only` for the review itself (no product patch required)
- Claim ceiling：`hypothesis`
- Non-claims：local browser observation only; not a product Gate; no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. `pnpm test` in `clients/pc/web` at the reviewed tree: **192/192 pass** (20 files).
2. `pnpm run build`: **pass** (110 modules). Bundle: CSS `index-hzxVOqHJ.css` 27.42 kB, JS `index-CgxwoJi4.js` 343.11 kB.
3. Bundle SHA-256 of that clean rebuild:
   - JS `sha256:fb22f7efed154f0267f10c99f93501d6d1c9778bac542c4cfc867f89203d26c4`
   - CSS `sha256:68be223616b4d33a2b7ebe926f85381ebf18c302876ba74fccf0aebe094c6e0a`
4. Exclusive rendered browser review (one driver, one fixture server, CDP port 9340): headless Chrome **151.0.7922.174** over CDP against `http://127.0.0.1:8791/ui/`. Harness, fixture server, JSON report and screenshots stay in ignored `d:\tmp\cp-review-w4\` (not Git).
5. Full matrix: **62/63 cells** and **668/669 assertions** on the first exclusive pass, with `overflow=0`, `clipped=0`, `consoleErrors=0`, and **0 contrast findings on screen and offscreen** in every cell.
6. The single miss was harness copy, not a product defect: `detail-stub` expected the W4 inventory sentence “not implemented over HTTP”, while Work detail names the daemon 200-stub as `STUB_ROUTE` and “not an observed zero” (matching `workDetail.test.tsx`). Assertion aligned to that copy; `REVIEW_ONLY=detail-stub` recheck **1/1 pass** (9/9 assertions).
7. Combined disposition: **63/63 cells pass**. No product finding was carried; no UI patch commit was required.

## Matrix coverage

W4 inventory/creation cells were re-run against the W5 tree so the new `#/work/:taskRef` links and the updated legacy `#/tasks` seam (“until they are migrated”) stay honest.

W5 cells covered:

- populated detail at 1920/1440/1280/960 × light+dark: two independent lanes, solid authority markers, hollow observation samples, six continuous sections, watch not attached, no streaming claim, bounded transition row, `no recorded facts` version gap, Loop/DECIDE named unavailable
- effects attention order (`OUTCOME_UNKNOWN` before `OUTCOME_APPLIED`) and truncation; empty effects as absence of recorded mutation
- evidence 404; verification passed ≠ completed; current acceptance may say completed; non-current verification is not acceptance
- ephemeral preview without a session chain, and with the session chain after a same-document admit
- designed object-404 for an unknown `task_ref` (no Overview/Run shell)
- section deep link; return to Work restores filter and selection
- consumption refusals: `RESOURCE_CONSUMPTION_NOT_FOUND`, `RESOURCE_TASK_CONTEXT_MISSING`, `RESOURCE_TASK_CONTEXT_MISMATCH`, `RESOURCE_CONSUMPTION_NOT_ELIGIBLE`, `RESOURCE_CONSUMPTION_UNAVAILABLE`
- denied evidence (`UNAUTHORIZED`), task-session gate, disconnect (`DISCONNECTED`), 200-stub (`STUB_ROUTE`), error (`TASK_EVIDENCE_UNAVAILABLE` / `TASK_EFFECTS_UNAVAILABLE`)
- server-side route whitelist on every cell; extra observation families only `o4`/`o5`
- keyboard/focus on the detail page

## Findings

| Finding | Disposition |
|---|---|
| `detail-stub` harness expected “not implemented over HTTP” | **harness-only**; product already names `STUB_ROUTE` / “not an observed zero”. Assertion corrected; cell rechecked pass. |
| Earlier colliding dual-driver runs | discarded; not evidence. Exclusive run is the record. |

No contrast, overflow, clip, console-error, invented-route, or honesty defect remains open.

## Next

`P7-T05/D14` is closed. Register and implement `P7-T05/D15` (Control Plane W6 Agents dossier) on the same task branch/lease. Do not auto-claim P6 / P7-T06 / P7-T07. W7–W12 and the legacy `styles.css` / `#/tasks` retirement remain unstarted. P7-T05 task acceptance is not complete until those waves land.
