# 09 — State, accessibility, and visual system

- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Status: current interaction prototype is owner-approved v5 (2026-08-29); archived pre-v5 and V2 are historical chrome only
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v5**](personal-20-opc-e2e-optimized-v5.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](history/2026-08-28-pre-subtraction/README.md)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## State grammar

| State | Must communicate |
|---|---|
| empty | why no object exists and one first-value action |
| loading/researching | exact source/work, retained state, partial result, safe-leave/cancel basis |
| partial | available facts, missing source/facet, coverage |
| stale | last-known fact/time, unsafe actions, refresh or re-preview |
| waiting-owner | exact input/decision, consequence, retained work |
| permission | exact scope, reason, benefit/risk, deny/narrow/grant paths |
| error | failed object/stage, retained input/work, safe retry/edit/escalation |
| unknown/reconciling | unavailable conclusion and why retry/success is blocked |
| offline | host/network/dependency state and retained work |
| queued/running | output contract, durable step, responsible Member, artifacts, real controls; queued is not running; Working is not completion |
| missed/coalesced | occurrence, reason, denominator, expiry, catch-up choice |
| success | changed object, receipt/evidence, next valuable action |
| archived | stopped triggers and read/export/restore/delete paths |
| Requires-backend | missing product capability and no fake action |
| Requires-environment | unexecuted native/external qualification and no support claim |

Status uses text plus shape/icon and source/freshness; color is secondary.
Native/Observed/Governed/Verified describes provenance or authority, never
confidence or decorative progress. Unknown cost is not zero; process exit is
not completion.

## Error, uncertainty, and explanation

Messages answer what failed, where, impact, retained work, whether retry is
safe, and one next action. The Assistant layers explanation:

1. concise result, affected goal/deliverable, and scope;
2. expandable basis, sources, freshness, omissions, and uncertainty;
3. audit/evidence detail and exact governed object.

Conflicting evidence remains conflict. The UI exposes neither hidden
chain-of-thought nor uncalibrated numerical confidence. A polished summary
cannot hide failed, not-run, stale, partial, or unknown work.

## Accessibility contract

- semantic landmarks, headings, lists, tables, forms, fields, and buttons;
- distinct labelled navigation, conversation, inspector, and one main region;
- visible labels/instructions and programmatically connected field errors;
- error summary and focus on the first invalid field;
- visible high-contrast focus, logical order, and keyboard-accessible `@`
  suggestions and canvas controls;
- Escape/cancel closes dialogs/sheets and restores trigger focus;
- tabs, disclosures, dialogs, comboboxes, grids, and live regions follow their
  actual ARIA interaction patterns;
- no hover-only, color-only, motion-only, or drag-only essential action;
- adequate targets and spacing;
- zoom/reflow and long localized labels preserve Project, state, and action;
- reduced motion/transparency and high contrast preserve hierarchy;
- streaming/group messages do not steal focus or flood announcements.

## Visual direction

Desired feeling: **calm, dense, precise, professional**.

- Segoe UI/system type, readable line height, and purposeful density;
- stable split layout for canvas plus conversation, contextual inspectors, and
  aligned lists;
- goals and openable deliverables receive stronger hierarchy than status;
  configuration receives the least;
- restrained border, radius, shadow, material, and motion;
- no purple AI gradient, glass-card wall, giant rounded tiles, decorative KPI
  dashboard, identical staff-card mosaic, fake command center, or marketing
  whitespace;
- one dominant action per task surface; edit/narrow/deny remain visible at
  consequential boundaries;
- typed ad-hoc canvas components share tokens and state grammar rather than
  arbitrary generated styling.

## Motion and feedback

Feedback begins immediately, remains interruptible where policy allows, and
originates from its trigger. Routine navigation avoids bounce. Reduced motion
uses short cross-fades or static continuity. No animation delays approvals,
reconciliation, stop controls, validation, or recovery. Long-running work shows
durable facts, not a decorative “thinking” animation.

## Windows window behavior

The three columns stay locked. A narrow canvas scrolls horizontally; the
sidebar and conversation do not become drawers, sheets, or an overlay. Sticky
actions do not cover focused fields or errors. Current Project, conversation
identity, state, and primary action remain in their columns.

This adaptive desktop layout is not native mobile, pairing, or 2.1 cloud 24/7
chrome. Those surfaces are parked and are not drawn as current product chrome.

## Review boundary

The owner-accepted competitive-informed v2 source review checks navigation,
the CEO loop rail, decision packet, exception swimlanes, Assistant/Project-group
switching, `@` routing into the unsent draft, typed canvas states,
setup/recovery states, capability honesty, keyboard order, focus restoration,
reduced motion, contrast, and long copy. Static review cannot prove rendered
accessibility conformance, human
usability, or formal acceptance; those remain future executed evidence under a
reconciled plan. Canvas runtime/render, NVDA, host-theme contrast, and 200%
real layout remain `not-run`.
