# 02 — Information architecture and app shell

## Navigation/IA brief

- Product job: operate Projects and resolve the next Owner decision.
- Primary objects: Project, employee, Inbox item, Knowledge source, Installed
  Agent, Provider binding, Task/Attempt.
- Top-level model: hub-and-spoke Windows app with Project-scoped master/detail.
- Global navigation: Today / Projects / Team / Knowledge / Inbox; Settings
  anchored at the bottom.
- Context rail: global Personal Assistant or selected employee Conversation.
- Current location: active navigation, page title, and Project/employee identity.
- Mobile replacement: none in 2.0; narrow Windows layouts use sheets/routes.

## Route design

```text
/today
/projects
/projects/:projectId/briefing
/projects/:projectId/setup
/projects/:projectId/work/:taskId/:attemptId?
/team
/team/:employeeId
/team/:employeeId/conversation
/knowledge
/knowledge/projects/:projectId
/inbox
/inbox/:itemId
/settings
/settings/installed-agents
/settings/providers
/settings/usage
/settings/privacy-recovery
```

These are target client routes, not current route/API claims.

## Desktop shell

| Region | Responsibility |
|---|---|
| Left rail | stable destinations, Inbox count with textual severity, Settings at bottom |
| Top context | page title, selected Project/object, host/offline state, freshness |
| Main | one surface's primary task and action |
| Right rail | Personal Assistant or employee Conversation; never both composers active |
| Inspector/sheet | source, version, authority, evidence, cost, and advanced capability facts |

Settings never competes with daily work. Providers and Installed Agents are
Settings groups, not top-level product destinations. Task/Attempt remains a
Project drilldown rather than a global Work bucket.

## Personal Assistant rail

The rail explains, navigates, researches, and proposes. A candidate change
opens a central structured preview. The right rail does not hold the
confirm/approve button for a consequential action and cannot write authority.

Selecting an employee switches to that employee's Personal-owned Conversation.
The active recipient is visible. Independent drafts persist; context switching
never sends or clears them.

## Orientation and continuity

- Deep links restore active nav, Project/employee label, selected tab, and
  object state.
- Back returns to prior filters, sort, selection, and scroll.
- Route loading preserves stable chrome and last-known safe facts.
- 404/deleted/archived/permission/offline routes state why and provide a stable
  destination.
- Unsaved setup/form navigation preserves a draft or explicitly warns what
  cannot be retained.
- Route changes focus the main heading; closing a sheet returns focus to its
  trigger.

## Narrow Windows behavior

Below the desktop width, left navigation becomes a labelled drawer and the
right Conversation becomes a sheet or dedicated route. The page retains the
current object and primary action. Tables become priority lists/details; no
hover-only controls remain. This is a responsive desktop window, not the 2.1
native mobile/remote product.

## Capability honesty

An absent backend route does not produce an active-looking control. The design
uses:

- status with `Requires-backend`;
- explanation of the missing dependency;
- link to the applicable Settings/status surface;
- no fake optimistic transition or success receipt.
