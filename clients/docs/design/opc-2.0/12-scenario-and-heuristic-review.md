# 12 — Requirement-family scenario and heuristic review

- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v9**](personal-20-opc-e2e-optimized-v9.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](history/2026-08-28-pre-subtraction/README.md)
- Status: current interaction prototype is owner-approved v9 (2026-08-30); v8 is the prior approved baseline (not overwritten); v5–v7 and archived pre-v5 / V2 are historical chrome only
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

This is the review protocol for the Markdown corpus and the Owner-approved
post-overwrite V2 interaction baseline. It
is not an executed Canvas, browser/DOM, accessibility, usability, backend,
Windows-host, connector, or product-acceptance result.

## Heuristic contract

Review every scenario for:

- goal, expected output, acceptance, and evidence before state/configuration;
- correct global Assistant versus Project group identity;
- one clear primary action and contextual edit/narrow/deny path;
- daemon preview before consequential work;
- recognition over recall, preserved context, and repeated-use speed;
- exact empty/loading/partial/stale/permission/error/unknown/offline/missed/
  running/success/archived behavior;
- safe recovery with retained input and no blind retry;
- keyboard, focus, target, contrast, reduced-motion, and long-copy behavior;
- capability honesty, source/freshness, and independent-verification boundary.

Severity 4 blocks design acceptance; repeated severity 3 is repaired before
visual polish. Findings cite exact scene, step, state, and requirement family.

## Static scenario corpus

| ID | Requirement family | Non-happy condition | Static success condition |
|---|---|---|---|
| UX-01 | research-first setup | source fails/conflicts | draft and partial research preserved; no false activation |
| UX-02 | goal/output contract | business outcome uncontrollable | deliverable/evidence contract remains testable; no guaranteed result |
| UX-03 | Today | source partial/stale | Needs you / Can continue / Unknown / Missed remain distinguishable; actual unknown is not zero |
| UX-04 | Project report | failed/not-run work exists | template cannot hide it behind summary |
| UX-05 | ad-hoc canvas | requested view lacks data | typed source-linked components show omission; no invented value |
| UX-06 | group conversation | `@member` redirects work | formal Task/revision appears before execution; no shadow authority |
| UX-07 | Role/Member creation | permission narrowed/model unavailable | Template, Member Runtime, and process remain distinct |
| UX-08 | Runtime improvement | comparison fails | old version remains active; rollback path retained |
| UX-09 | Context compression | package exceeds model window | Task contract/fixed decisions remain; omissions visible |
| UX-10 | Memory/feedback | one rating suggests global preference | Project evidence only; versioned proposal required |
| UX-11 | Knowledge import | parser failure/secret detected | original retained; secret exits Knowledge path |
| UX-12 | approval | stale preview/reject/narrow | exact diff and choices; chat cannot confirm |
| UX-13 | unknown Effect | retry unsafe | reconciliation is visible; redispatch unavailable |
| UX-14 | Routine | overlap plus offline missed run | one active, latest queued, coalesced/missed denominator |
| UX-15 | Model Connection | quota/cost unknown | unknown is not zero; explicit Member selection; no silent binding |
| UX-16 | cost warning | threshold exceeded | warning visible; Personal does not automatically stop work |
| UX-17 | Skill/MCP | first MCP install expands permission | Skill auto-installs only after review; MCP first install/expansion needs exact version/permission confirmation and a scoped grant |
| UX-18 | hidden DSH/Pi | engine unqualified/update failed | advanced diagnosis/rollback only; no everyday engine UI |
| UX-19 | X/Twitter loop | connector drift/CAPTCHA/unknown publish | fail closed, receipt/readback separated, no evasion/retry |
| UX-20 | archive/delete | same-disk restore only | archive-first, export, impact, second confirm, no backup claim |
| UX-21 | keyboard/narrow window | conversation remains the third column | no trap; location, context identity, and action preserved; columns do not stack |
| UX-22 | capability gap | backend/environment absent | explanation/dependency only; no active-looking action |

These scenario IDs are local design-review labels, not formal task IDs or a
replacement acceptance denominator.

## Friction budget

- daily navigation and low-risk inspection: direct, no confirmation;
- Project group question or temporary canvas: one message/action, temporary by
  default;
- reversible manager adjustment inside envelope: direct plus version/history;
- primary goal, team, model, capability, permission, or external rule:
  structured preview and Owner confirmation;
- first MCP install/expansion: exact version and permission confirmation;
- public/security/destructive action: scope, consequence, source,
  reversibility, confirmation, Intent/Effect, and receipt;
- recovery: contextual list/detail, no modal chain or hidden retry.

## State coverage audit

The owner-approved post-overwrite v2 source is reviewed against all state classes from
[09](09-state-accessibility-and-visual-system.md), include at least one
`Requires-backend` treatment on every target-only surface, and use
`Requires-environment` for unqualified Windows/DSH/Provider/MCP/X behavior.
Prototype example artifacts and receipts must be labelled and must not imply a
daemon write.

## Evidence ladder

1. Markdown structure, links, terminology, and cross-document consistency:
   source/static checks passed.
2. Canvas source/import/type/API review: passed.
3. Canvas runtime interaction review: scenes, drafts, group routing, typed
   canvases, states, and zero external effects.
4. Browser accessibility and usability review, including NVDA, host-theme
   contrast, and 200% real layout.
5. Future backend integration using real previews, receipts, and gaps.
6. Future qualified Windows/external fixed denominator after formal-plan
   reconciliation.
7. Separate target-user research for activation, returning use, recovery,
   comprehension, trust, and willingness to pay.

No lower rung is promoted into a higher claim.

## Current disposition

The Owner-approved current chrome is `personal-20-opc-e2e-optimized-v9`
(2026-08-30). The competitive-informed V2 source remains historical provenance
(same V2 files, not a v3). It recorded a visible CEO loop and Today four
exception swimlanes; those are **not** current chrome. Canvas-only HITL and
daemon authority path remain. Owner
acceptance is recorded, and source/static checks passed. Canvas
runtime/render, NVDA, host-theme contrast, 200% real layout, and all
backend/external/qualified execution remain `not-run`. Owner approval does
not promote any of those results and is not usability, accessibility, backend,
Gate, release, qualification, or acceptance evidence.
