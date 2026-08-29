# 02 — Information architecture and app shell

- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Status: current interaction prototype is owner-approved v9 (2026-08-30); v8 is the prior approved baseline (not overwritten); v5–v7 and archived pre-v5 / V2 are historical chrome only
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v9**](personal-20-opc-e2e-optimized-v9.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](history/2026-08-28-pre-subtraction/README.md)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## Navigation brief

- Product job: reach the most important goal, deliverable, or Owner decision
  without navigating Agent infrastructure.
- Fixed first-level anchors: **Today / Projects / Knowledge**.
- Secondary anchor: **Settings**, fixed at the bottom.
- Project-scoped objects: Members, Roles, Tasks, Attempts, approvals,
  exceptions, recovery, capabilities, and diagnostics. Team and Inbox are not
  first-level destinations.
- Global conversation identity: Personal Assistant in the right column.
- Project conversation identity: Owner + manager + Members group, always the
  third column; no overlay “open conversation” control.
- Parked 2.1 chrome: native mobile, pairing, and cloud 24/7 are not drawn.
  A narrow canvas scrolls horizontally and does not stack the three columns.

## Target route design

```text
/today
/projects
/projects/:projectId
/projects/:projectId/setup
/projects/:projectId/detail
/projects/:projectId/members/:memberId?
/projects/:projectId/runs/:stageId?
/projects/:projectId/outputs/:outputId?
/projects/:projectId/canvas/:viewId?
/projects/:projectId/attention/:itemId?
/projects/:projectId/work/:taskId/:attemptId?
/knowledge
/knowledge/projects/:projectId
/settings
/settings/model-connections
/settings/cost-and-alerts
/settings/privacy-recovery
/settings/advanced-diagnostics
```

These are design routes, not current client/API claims. Member, run, output,
and attention routes are contextual deep links, not additional first-level
navigation. Live Project chrome is four submenus (详情 / 成员 / 运行 / 产出);
the list uses one 「打开」 plus text links.

## Desktop shell

| Region | Responsibility |
|---|---|
| Left navigation | Today, Projects, Knowledge; Settings at bottom; Team/Inbox are not L1 |
| Context header | location, selected Project/object, host/offline state, freshness. **No visible CEO six-step rail** (Ingest → Decide → Authorize → Execute → Verify → Report is backend discipline, not product chrome) |
| Main canvas | goal, openable deliverable, decision packet, plan, or structured preview |
| Right conversation | always the third column: global Assistant outside a Project; Project group inside |
| Context inspector | Member, source, evidence, version, authority, cost, or diagnostics inside the center column |

The shell locks left / center / right. Conversation is a peer of the canvas:
it interprets and proposes; the canvas displays governed objects and
confirmable daemon previews. HITL is announced in chat and linked to the
center preview; chat has no Approve control and no “Don’t ask again” grant.
The UI never turns chat text into authority.

## Assistant and Project-group behavior

The global Assistant explains, researches, recommends, navigates, and initiates
every management flow (highest UX privilege). It still writes only through a
daemon-issued preview, Owner confirm, and receipt. Entering a Project switches
the primary conversation to its group. The manager speaks by default. Members speak proactively only when
mentioned, delivering, handing off, blocked, or requesting a decision.

`@manager` asks for status or delegation. `@member` asks or redirects bounded
work. `@member` creates a formal Task revision, not a shadow plan. `@` inserts
only into the unsent draft; it never sends, approves, or writes authority. Any work-changing message becomes a Task or revision before
execution. Unsent drafts are preserved by Assistant/Project context; navigation
cannot merge, clear, or send them.

## Orientation and continuity

- Deep links restore anchor, Project/object label, selected view, and state.
- Back restores filters, sort, selection, scroll, and unsent draft.
- Loading preserves shell and last-known safe facts.
- Missing, deleted, archived, permission-denied, and offline routes explain the
  condition and offer a stable destination.
- Opening a contextual roster, attention item, source, or diagnostic preserves
  the Project location beneath it.
- Route changes focus the main heading; closing a dialog/sheet restores trigger
  focus.

## Narrow Windows behavior

The three columns stay locked. A narrow canvas scrolls horizontally; left
navigation and the right conversation do not become drawers, sheets, or an
overlay. Goal, current Project, state, primary action, and unsent draft remain
in their columns. No essential control depends on hover, drag, blur, or
motion. This is not native mobile or 2.1 remote operation.

## Capability honesty

An absent product action is labelled `Requires-backend` with the missing
dependency and next valid route. An unqualified native/external path is
`Requires-environment`. Neither produces an active-looking control, optimistic
state transition, fake receipt, or support claim. There are no Connect /
Install / Confirm fake buttons. Architecture and formal-plan route names
remain pending reconciliation.
