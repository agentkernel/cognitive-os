# P7-T05/D24 Control Plane W12 accessibility / QA — rendered browser review

- 状态：running report（D24 rendered-review exit）
- 日期：2026-08-26
- 任务：`P7-T05` Non-blocking Web UI（Control Plane redesign）
- Slice：`P7-T05/D24` — Control Plane W12 accessibility / QA gate
- Slice 状态：`done` after this review
- Lease：`lease/personal/P7-T05/d14-rendered-review`
- Branch：`personal/P7-T05-d14-rendered-review`
- Draft PR：[cognitive-os#274](https://github.com/agentkernel/cognitive-os/pull/274)
- Reviewed tree：`b30314f3` (`feat(P7-T05): add the W12 shell keyboard layer and coarse-pointer targets`)
- Change class：`implementation-only`
- Claim ceiling：`hypothesis`
- Non-claims：local browser observation only; not a product Gate; no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. `pnpm test` in `clients/pc/web` at the reviewed tree: **305/305 pass** (38 files).
2. `pnpm run build`: **pass** (137 modules). Bundle: CSS `index-Beyig3Pd.css` 30.63 kB, JS `index-CaaeTavH.js` 443.06 kB.
3. Bundle SHA-256 of that clean rebuild:
   - JS `sha256:1101d854de0857fbb01e22b804180b10173a57a1f05816ef686d21ffa65bc291`
   - CSS `sha256:7cf84a8ff1041a2ba4bb7468ca3cf06c10d441a6cb6752653f871e1bc3af511b`
4. Exclusive rendered browser review (one driver, one fixture server, CDP port 9355): headless Chrome **151.0.7922.174** over CDP against `http://127.0.0.1:8795/ui/`. Harness, fixture server, JSON report and screenshots stay in ignored `d:\tmp\cp-review-w4\` (not Git). `REVIEW_ONLY=a11y-`.
5. W12 a11y matrix: **15/15 cells** and **107/107 assertions**, with `overflow=0`, `clipped=0`, `consoleErrors=0`, and **0 contrast findings on screen and offscreen** in every cell.
6. One product finding was found and fixed before the passing run: clipboard `writeText` rejections were uncaught promises on Copy task ref / DigestChip. Copy now fails closed without a console error.

## Matrix coverage

- `g` then `w` / `a` land on Work and Agents
- keyboard-only Work path: `j` → Enter → `]`×3 to Evidence → Copy task ref (light+dark)
- `/` opens the palette; `/` inside a filter field does not
- Escape on Work detail returns to the inventory
- reduced-motion light+dark; `prefers-contrast: more`
- 1100 px top-strip nav (design-12 960–1279 band)
- tab walk: visible stops have a focus ring
- Home 1920 light+dark; Work detail Evidence at 960

## Keyboard honesty

The shell layer is observation-only: chords never mint authority. Typing in a field keeps its keys. The open palette keeps its own keys. Native `EventSource` / watch attach is unchanged. Clipboard copy is best-effort and never an uncaught rejection.

## Honest not-run

Coarse-pointer 44 px targets are CSS `@media (pointer: coarse)` and were not emulated in this Chrome pass (fine pointer). Headless clipboard permission is not granted; the path proves the Copy control is reachable and that a denied clipboard no longer surfaces as a console error. No Gate / release / Profile claim.

## Findings

None carried. The uncaught-clipboard defect was fixed in the reviewed tree.

## Next

`P7-T05/D24` is closed. Register and implement `P7-T05/D25` (legacy `styles.css` / `#/tasks` retirement) on the same task branch/lease. Do not auto-claim P6 / P7-T06 / P7-T07. P7-T05 task acceptance is not complete until that retirement lands. P7-T07 remains blocked on owner-only B01-W prerequisites.
