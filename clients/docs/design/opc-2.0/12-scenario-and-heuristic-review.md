# 12 — Scenario and heuristic review

## Review status

This is the static design review protocol for the current corpus and Canvas.
It is not an executed human-usability, browser/DOM, accessibility-conformance,
backend, Windows-host, or product acceptance result.

## Heuristic contract

Review each scenario for:

- visible system/host/source status;
- business-language match;
- one primary action;
- cancel/back/deny/narrow/retry/resume/undo where real;
- constraints and daemon preview before consequential work;
- recognition over recall and preserved context;
- repeated-use speed;
- exact error recovery and retained input;
- keyboard/focus/target/reduced-motion behavior;
- capability honesty and evidence provenance.

Severity 4 blocks design acceptance; repeated severity 3 is repaired before
visual polish. Findings cite the exact scene/step/state.

## Static scenarios

| ID | Task | Non-happy condition | Static success condition |
|---|---|---|---|
| UX-01 | create first Project | research fails, draft preserved | reach review/receipt model without optional setup or false activation |
| UX-02 | inspect Today | partial/stale Project source | Owner can state what needs action and what is unknown |
| UX-03 | revise Project plan | preview becomes stale | edits persist and confirm is replaced by re-preview |
| UX-04 | create role/employee | permission narrowed | Blueprint/Assignment/Employee remain distinct |
| UX-05 | switch Assistant/employee | both have unsent drafts | exactly one recipient active; both drafts preserved |
| UX-06 | import Knowledge | parser failure and secret detected | original preserved; secret routes out of Knowledge |
| UX-07 | approve external work | user rejects/narrows | structured preview, consequence, receipt path; chat not authority |
| UX-08 | recover unknown Effect | retry unsafe | reconcile explanation; no blind redispatch |
| UX-09 | resume missed Routine | publish occurrence missed offline | low-risk vs consequential catch-up is separated |
| UX-10 | inspect DSH | sandbox unqualified/update failed | exact artifact/health/rollback; no native UI or false support |
| UX-11 | change Provider binding | quota unknown/budget stopped | global/Project/employee/Task precedence and actual unknown visible |
| UX-12 | archive/delete Project | same-disk restore only | archive-first, export, impact, second confirm, no disaster-backup claim |
| UX-13 | keyboard navigation | drawer/dialog/composer open | no trap; visible focus; recipient and location preserved |
| UX-14 | narrow window/reduced motion | long copy and errors | primary job/state remains visible without hover/motion dependency |
| UX-15 | view Requires-backend | backend absent | explanation and dependency only; no active-looking action |

## Friction budget

- daily low-risk navigation: one selection/shortcut, no confirmation;
- reversible approved-boundary manager adjustment: direct action plus history;
- role/plan/binding/budget changes: structured review and confirmation;
- external/public/security/destructive work: scope, consequence, source,
  reversibility, Owner confirmation, and receipt;
- first Project value: resumable setup, optional detail deferred;
- Inbox triage: stable priority list and detail, no modal chain.

## State coverage audit

The prototype must visibly exercise:

- empty: Project or Knowledge;
- loading: Project/setup/import;
- partial/stale: Today or Provider;
- permission: Knowledge/import or binding;
- error: setup/import/connector;
- unknown: Effect or usage;
- offline/missed: Routine;
- long-running: research/import/Attempt;
- success: daemon receipt example labelled prototype;
- archived: Project;
- Requires-backend: at least one unavailable action on every target-only
  surface.

## Evidence ladder

1. **Static design review:** this corpus and Canvas TypeScript structure.
2. **Canvas interaction check:** scene switching, drafts, setup states, and no
   network/external package use.
3. **Future browser review:** role/label selection, focus order, error states,
   narrow windows, reduced motion/contrast.
4. **Future backend integration:** real daemon previews, receipts, and
   capability gaps.
5. **Future qualified Windows fixed denominator:** Phase 11 acceptance.
6. **Separate human research:** first-time, returning, recovery, keyboard, and
   comprehension observation with target users.

No lower level is promoted into a higher-level claim.

## Current static disposition

The corpus provides a complete review target but this row is `not-run` until
the new Canvas exists and the documented static/type checks execute. Results
must be appended to the task's single running report immediately after each
check.
