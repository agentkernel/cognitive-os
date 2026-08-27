# Personal 2.0 OPC Control Plane product design

- Status: adopted target; implementation remains capability-gated
- Current client: `clients/pc/web/`, daemon-served at `/ui/`
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Interaction corpus: [clients OPC design](../../../clients/docs/design/opc-2.0/README.md)
- Product prototype: external Cursor Canvas
  `personal-2-opc-product-prototype.canvas.tsx`

## 1. Reality and capability honesty

| Boundary | UI truth |
|---|---|
| **Current implementation (Now)** | The delivered daemon-served UI has Linux-era Home, Work, Agents, Providers, Resources, Activity, and System surfaces with bounded real capabilities. |
| **Adopted Personal 2.0 target** | Windows OPC shell with Today, Projects, Team, Knowledge, Inbox, bottom Settings, and a global right Personal Assistant. |
| **Requires-backend** | Project/Role/Employee authority, Personal conversations/archive, Pi Assistant composition, managed DSH supply chain/runtime, Routine/missed-run, Inbox workflows, Vault ingestion/retrieval, binding hierarchy/budget enforcement, and OPC projections. |
| **Requires-environment** | Windows host, background/tray, DSH sandbox, connector, and final OPC validation require qualified Windows-native routes not supplied by ordinary CI or this prototype. |

The prototype is an interaction specification. A control that lacks a backend
is labelled `Requires-backend` and appears as explanatory status or a route to
the dependency—not a button that pretends to execute.

## 2. UX decision brief

- **Job:** understand the most important Project decision, supervise digital
  employees, and intervene safely.
- **User mode:** first-time Owner during setup; daily returning operator
  afterward.
- **Frequency/risk:** daily scan plus occasional high-risk approvals, Provider
  changes, publishing, permissions, and deletion.
- **Pattern:** hub-and-spoke app shell; Today priority stack; Projects/Team/
  Knowledge master-detail; Inbox approval queue; searchable grouped Settings;
  guided setup with review.
- **Primary action:** the next Project-specific action, never a generic
  "Manage".
- **Recovery:** preserve filters, selection, drafts, form input, Project setup,
  and last-known facts across errors and offline state.
- **Required states:** empty, loading, partial, stale, permission, error,
  unknown, offline, missed, long-running, success, archived.

## 3. Task ergonomics contract

- **Core task:** find what needs the Owner and resolve it with evidence.
- **Cognitive load:** visible Project/employee identity, reason, consequence,
  freshness, cost basis, and next action; no recall of internal IDs.
- **Control model:** edit/narrow/deny/confirm; stop/pause/retry/resume only when
  the daemon supports them; export and audit for consequential work.
- **Speed path:** stable sidebar, recents, preserved list filters, keyboard
  navigation, Project switcher, and persistent object links.
- **Error prevention:** safe defaults, constraints before input, daemon-issued
  diff preview, stale-version invalidation, destructive separation.
- **Evidence plan:** static prototype scenario review now; browser/runtime,
  keyboard, reduced-motion, state, and qualified Windows scenarios later.

## 4. App shell and route map

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

These are design routes, not existing HTTP/SPA claims. Current implementation
routes remain factual only in the archived as-built design corpus and code.

The desktop shell has:

1. a stable left navigation with Settings anchored at the bottom;
2. a page title and selected Project/object identity;
3. a central task surface;
4. a right Personal Assistant rail;
5. a single active composer zone.

On narrow windows, the sidebar and assistant become labelled sheets while the
current location, primary action, and unsent draft remain visible. This is a
responsive Windows window treatment, not a native mobile product.

## 5. Surface responsibilities

### Today

Shows today's plan, Owner decisions, Project health narrative, employee state,
missed work, and latest verified results. It avoids generic KPI tiles. Every
item states Project, responsibility, state, next action, verification, cost
basis, and freshness.

### Projects

Uses a stable list and Project briefing. The briefing prioritizes goal,
current phase, manager summary, today's work, Team, Inbox items, latest
artifacts/evidence, and spend. Tasks/Attempts are a drilldown, not the default
mental model.

### Guided setup

Uses conversational research plus structured sections. States are
`local-draft`, `daemon-draft`, `researching`, `waiting`, `re-preview`,
`creating`, `failed`, and `active-receipt`. The review screen has editable
charter/goals/metrics/team/plan/permissions/budgets/triggers and an exact
daemon-issued diff before activation.

### Team

Shows the current Project Manager, Role Blueprints, Assignments, employees, and
their goal/responsibility/state/next/verified/cost facts. The employee detail
contains work, Conversation, Memory, runtime diagnostics, and history without
equating employee identity with a process.

### Knowledge

Shows Owner-shared knowledge, Project Vaults, employee-private Memory,
provenance, import/index status, conflicts, exclusion, correction, and forget.
Credentials route to SecretStore setup and never enter Knowledge.

### Inbox

Uses a priority queue and selected-item detail for approvals, requested input,
permissions, blocks, failures, unknown Effects, missed runs, and budget
warnings. Approval is a structured action preview; chat explanation is
secondary.

### Settings

Searchable groups cover Personal Home, Installed Agents, Providers, Usage &
Budgets, Notifications, Privacy & Recovery, Diagnostics, and Advanced.
Installed Agents shows preinstalled managed DSH with source/version/health/
qualification/update/rollback. Pi is shown only in advanced Personal Assistant
diagnostics, not as an ordinary installed Agent.

## 6. Conversation and single-composer contract

The right rail may host the Personal Assistant or selected employee
Conversation, but only one composer is active:

- changing recipient requires an explicit tab/identity change;
- each recipient keeps an independent unsent draft;
- submit labels name the recipient;
- switching cannot send, merge, or discard a draft;
- employee output remains candidate/observation;
- consequential suggestions open a daemon preview in the central surface;
- no raw chain-of-thought or fabricated confidence percentage is shown.

The Personal Assistant layers explanation: a one-sentence answer, expandable
basis/sources/scope, then an audit link. Uncertainty is stated as missing or
conflicting evidence with a concrete next step.

## 7. Form and confirmation contract

Project setup, role creation, imports, Provider binding, budgets, and approval
use visible labels, constraints before entry, validation after blur/submit,
preserved values, exact field errors, async status, and keyboard focus on the
first error. High-risk changes use a review step with edit links and
consequence/reversibility/source/budget facts.

Only a daemon-issued preview is confirmable. Client or Assistant summaries are
clearly labelled candidates. A stale preview disables confirmation and
preserves edits for re-preview.

## 8. State system

Every surface specifies:

| State | UI requirement |
|---|---|
| empty | reason plus one first-value action |
| loading | stable shell/skeleton, exact source, cancel only if real |
| partial | available data, missing source, coverage |
| stale | last-known time, unsafe actions, refresh |
| permission | exact scope, consequence, deny/narrow path |
| error | failed object/stage, preserved input, retry/edit/support |
| unknown | explicit non-conclusion; no zero/healthy/success coercion |
| offline | local host/network state, retained work, reconnect |
| missed | missed/coalesced occurrences and risk-based next action |
| long-running | plan, current step, artifacts, elapsed basis, real controls |
| success | changed object, receipt/evidence, next action |
| archived | read/export/restore/delete pathways and stopped triggers |

## 9. Visual and accessibility direction

The target is **calm, dense, precise, professional**:

- Segoe UI/system fonts, stable alignment, restrained radius/shadow, no
  gradient/card wall, and no decorative AI glass;
- operational density from hierarchy, not tiny text;
- immediate feedback, spatially consistent and interruptible motion only where
  interaction warrants it;
- reduced-motion cross-fades/static continuity and high-contrast support;
- semantic landmarks/headings/forms/lists/tables/dialogs;
- visible focus, logical order, keyboard exit, non-color state labels, and
  sufficiently large pointer targets;
- no hover-only critical control.

## 10. Non-claims

No route, control, projection, Windows behavior, conversation, or Agent
capability described here is implemented by this document or Canvas. Existing
P7-T05 review evidence does not transfer to this target. Human usability and
accessibility conformance require later executed scenarios.
