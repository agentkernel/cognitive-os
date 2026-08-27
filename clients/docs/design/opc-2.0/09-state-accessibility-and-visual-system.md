# 09 — State, accessibility, and visual system

## State grammar

The same state words apply across every surface:

| State | Must communicate |
|---|---|
| empty | why there is no object and one first-value action |
| loading | exact source/work, stable content, safe leave/cancel basis |
| partial | present facts, missing source/facet, coverage |
| stale | last-known time, unsafe actions, refresh |
| permission | exact scope/reason/consequence and deny/narrow path |
| error | failed object/stage, preserved input/work, retry/edit/support |
| unknown | unavailable conclusion; never zero/healthy/success |
| offline | host/network/dependency state and retained work |
| missed | occurrence/reason/denominator and risk-based catch-up |
| queued/running | plan, current durable step, responsible employee, controls |
| success | changed object, receipt/evidence, next action |
| archived | stopped triggers and read/export/restore/delete paths |
| Requires-backend | absent product capability, dependency and no fake action |

Status uses text plus shape/icon and source/freshness; color is secondary.
Native/Observed/Governed/Verified is provenance/authority, not confidence or
percent progress.

## Error and uncertainty

Messages answer what failed, where, impact, what was retained, whether retry is
safe, and next action. The Personal Assistant layers explanation:

1. concise result and scope;
2. expandable basis/sources/uncertainty;
3. audit/evidence detail.

The UI does not expose chain-of-thought or numerical confidence without a
calibrated denominator. Conflicting evidence is shown as conflict, not averaged
into certainty.

## Accessibility contract

- semantic landmarks, headings, lists, tables, fields, and buttons;
- labelled navigation regions and one main region;
- visible labels, instructions, and connected field errors;
- error summary and focus to the first invalid field;
- visible high-contrast focus and logical keyboard order;
- Escape/cancel exits dialogs/sheets and restores trigger focus;
- tabs, disclosures, dialogs, comboboxes, and grids follow their actual ARIA
  interaction pattern;
- async progress/status uses appropriate live announcements without noise;
- no hover-only or drag-only essential action;
- adequate pointer target size/spacing;
- zoom/reflow and long labels do not obscure identity or primary action;
- reduced motion and reduced transparency preserve state and orientation.

## Visual direction

Desired feeling: **calm, dense, precise, professional**.

- Segoe UI/system type with size-specific tracking and readable line height;
- stable split layouts and master/detail alignment;
- restrained border, radius, shadow, and material;
- no purple AI gradient, glass card wall, oversized rounded tiles, decorative
  dashboard, or marketing whitespace;
- one visually dominant primary action per task surface;
- compact cards only when they answer Project/employee decisions;
- denser lists for comparison and triage;
- color tokens meet contrast in light/dark/high-contrast themes.

## Motion and feedback

Feedback begins immediately and remains interruptible where the user can
redirect it. Panels enter/exit along the same path and originate from their
trigger. Routine navigation avoids bounce. Reduced motion uses short
cross-fades/static changes and never removes status feedback.

No animation delays approvals, stop controls, form validation, or error
recovery. Long-running work shows actual durable facts, not a decorative
"thinking" animation.

## Windows window behavior

The desktop layout supports large and narrow windows. At narrow widths:

- sidebar -> labelled drawer;
- right conversation -> sheet/dedicated route;
- tables -> priority list + detail;
- sticky actions avoid covering focused fields/errors;
- current Project/recipient stays visible.

This adaptive layout is not native mobile or 2.1 remote support.

## Static acceptance

The prototype review checks every navigation target, single-composer state,
guided setup state, non-happy state, Requires-backend treatment, keyboard
order, focus labels, reduced-motion rules, and long-copy containment. Static
review cannot prove rendered accessibility conformance or human usability;
those remain Phase 11 executed validations.
