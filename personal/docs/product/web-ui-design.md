# Personal 2.0 OPC Control Plane product design

- Status: adopted target; implementation remains capability-gated
- Current client: `clients/pc/web/`, daemon-served at `/ui/`
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Interaction corpus: [clients OPC design](../../../clients/docs/design/opc-2.0/README.md)
- Interaction baseline:
  [**Owner-approved interaction baseline (2026-08-28)**](../../../clients/docs/design/opc-2.0/personal-20-ai-ceo-e2e-optimized-v2.canvas.tsx)
- Baseline identity: same V2 files (not a v3). Owner accepted the 2026-08-28
  competitive-informed overwrite: visible CEO loop (Ingest → Decide →
  Authorize → Execute → Verify → Report), Today decision packet plus four
  exception swimlanes, canvas-only HITL, and daemon authority path. This is
  not the pre-overwrite overlay-conversation / stacked-column V2.
- Cursor-openable copy (IDE detection path; not a second product baseline):
  [personal-20-ai-ceo-e2e-optimized-v2](C:\Users\wuron\.cursor\projects\d-agent-kernel\canvases\personal-20-ai-ceo-e2e-optimized-v2.canvas.tsx)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## 1. Reality and capability honesty

| Boundary | UI truth |
|---|---|
| **Current implementation (Now)** | The delivered daemon-served UI has Linux-era Home, Work, Agents, Providers, Resources, Activity, and System surfaces with bounded real capabilities. |
| **Adopted Personal 2.0 target** | Windows OPC shell with stable Today, Projects, and Knowledge anchors; bottom Settings; locked left / center / right columns; global Assistant outside Projects; Project group conversation always in the right column. |
| **Requires-backend** | Project/Role/Member authority, Personal conversations/archive, typed canvas composition, Pi Assistant composition, hidden managed DSH, Routine/missed-run, contextual attention/approval, Vault ingestion/retrieval, Model Connections, capability acquisition, and OPC projections. |
| **Requires-environment** | Windows host, background/tray, DSH sandbox, connector, and final OPC validation require qualified Windows-native routes not supplied by ordinary CI or this prototype. |

The prototype is an interaction specification. A control that lacks a backend
is labelled `Requires-backend` and appears as explanatory status or a route to
the dependency—not a button that pretends to execute.

## 2. UX decision brief

- **Job:** understand the most important Project goal/result/decision, supervise
  Project Members, open deliverables, and intervene safely.
- **User mode:** first-time Owner during setup; daily returning operator
  afterward.
- **Frequency/risk:** daily scan plus occasional high-risk approvals, Provider
  changes, publishing, permissions, and deletion.
- **Pattern:** locked three-column shell; Today exception-first scan; Projects
  and Knowledge master-detail; Project group always in the right column plus
  governed center canvas; contextual Member/attention inspectors; searchable
  grouped Settings; guided setup with review. Narrow canvas scrolls
  horizontally and does not stack.
- **Primary action:** the next Project-specific action, never a generic
  "Manage".
- **Recovery:** preserve filters, selection, drafts, form input, Project setup,
  and last-known facts across errors and offline state.
- **Required states:** empty, loading, partial, stale, permission, error,
  unknown, offline, missed, long-running, success, archived.

## 3. Task ergonomics contract

- **Core task:** find what needs the Owner and resolve it with evidence.
- **Cognitive load:** visible Project/Member identity, reason, consequence,
  freshness, cost basis, and next action; no recall of internal IDs.
- **Control model:** edit/narrow/deny/confirm; stop/pause/retry/resume only when
  the daemon supports them; export and audit for consequential work.
- **Speed path:** stable sidebar, recents, preserved list filters, keyboard
  navigation, Project switcher, and persistent object links.
- **Error prevention:** safe defaults, constraints before input, daemon-issued
  diff preview, stale-version invalidation, destructive separation.
- **Evidence plan:** corpus and Canvas source/static checks passed; Canvas
  runtime/render, NVDA, host-theme contrast, and 200% real layout remain
  `not-run`. Keyboard, reduced-motion, browser state, and qualified Windows
  scenarios remain later. Owner approval is not usability, accessibility,
  backend, Gate, release, qualification, or acceptance evidence.

## 4. App shell and route map

```text
/today
/projects
/projects/:projectId
/projects/:projectId/setup
/projects/:projectId/canvas/:viewId?
/projects/:projectId/members/:memberId?
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

These are design routes, not existing HTTP/SPA claims. Current implementation
routes remain factual only in the archived as-built design corpus and code.

The desktop shell locks three columns:

1. a stable left navigation with Settings anchored at the bottom; Team and
   Inbox are not first-level destinations;
2. a central source-linked operating canvas (stable Project report by default,
   then the X loop, setup, People, Operations, or Knowledge as selected);
3. a right-column conversation that is always the third column: global Personal
   Assistant outside a Project, Project group inside one.

Conversation is never an overlay and there is no “open conversation” control.
A narrow canvas scrolls horizontally; the three columns do not stack, collapse
into drawers, or become labelled sheets. Native mobile, pairing, and cloud
24/7 chrome are 2.1 and are not drawn as current product chrome. Missing
actions are labelled `Requires-backend` or `Requires-environment`; there are
no Connect / Install / Confirm fake buttons.

## 5. Surface responsibilities

### Today

Today is one decision packet plus four exception swimlanes, not a KPI wall.
The returning surface opens with consequence, reversibility, alternatives,
kernel truth, and why option A is first, then orders:

1. **Needs you** — consequential Owner decisions (for example Package A review);
2. **Can continue** — work that does not require the Owner now;
3. **Unknown** — missing actuals, Effects, or verification;
4. **Missed** — offline or coalesced Routine facts.

Cost is estimated or actual; actual unknown is never shown as zero. Member
activity is a **Working / Queued / Waiting** table; queued is not running.
Every item states Project, responsibility, state, next action, verification,
cost basis, and freshness. Provenance chips are Observed / Proposed / Governed
/ Verified.

### Projects

Uses a stable list and Project operating canvas. A Project opens the stable
operating-report template first, then the X loop when that Project needs it.
The default template
prioritizes goal hierarchy, current phase, manager summary, today's work,
Members, attention/approval items, latest openable artifacts/evidence, cost
basis, and freshness. Package inspect (for example Package A) shows a thread
preview plus acceptance; planned is not published. Tasks/Attempts are a
drilldown, not the default mental model.

The Project Manager may version the stable template for that Project. An
ad-hoc question lets the Assistant/manager read real results and compose a
temporary view from approved typed components. It is not saved unless pinned
or made a template. Generated code/`eval`, invented values, and hiding
goal/acceptance state, failed/not-run work, Owner decisions, source, or
freshness are prohibited.

### Guided setup

Uses broad automatic web research plus conversational and structured sections.
States are
`local-draft`, `daemon-draft`, `researching`, `waiting`, `re-preview`,
`creating`, `failed`, and `active-receipt`. The review screen has editable
charter/goals/output contracts/team/plan/Provider/capabilities/permissions/
HITL/triggers and an exact daemon-issued diff before activation. One cycle is
simulated before the preview. External research is untrusted and never
executes or expands permission.

### Contextual Members and Roles

Opened from a Project, not first-level navigation. The object chain is Role
Template → Member → Task → disposable process. Process death does not delete
the Member. The surface shows the current Project Manager, reusable Role
Runtime Templates, and Project Member Runtime definitions with
goal/responsibility/state/next/accepted-deliverable/cost/freshness facts.
Activity is Working / Queued / Waiting; queued is not running. Member detail
contains work, Conversation, Memory, grants, diagnostics, and history without
equating Member identity with a process.

### Operations

Working is in-progress observation, not completion. Queued is not running.
Unknown Effects stay unknown until reconciliation. The default working view is
**Candidate → Intent persisted → Fence → Execute → Independent verify →
Receipt**. Missing pause/stop/retry controls are labelled `Requires-backend`,
not drawn as live buttons.

### Knowledge

Shows Owner-shared knowledge, Project Vaults, Member-private Memory,
provenance, import/index status, conflicts, exclusion, correction, and forget.
Context shows a **Why this fragment** table; Memory is not silent auto-ingest.
A Vault is Markdown files with stable links. Obsidian is an optional companion
and is not embedded in the app. Credentials route to SecretStore takeover and
never enter Knowledge or chat.

### Contextual attention and approval

Opens from Today or the affected Project. It uses a priority list and selected
detail for approvals, requested input, permissions, blocks, failures, unknown
Effects, missed runs, and cost warnings. Approval is a structured action
preview; group explanation is secondary. There is no permanent Inbox anchor.

### Settings

Searchable groups cover Personal Home, Model Connections, Cost & Alerts,
Notifications, Privacy & Recovery, and Advanced Diagnostics. Model Connections
offers mainstream Provider quick templates plus custom URL/compatibility-mode/
key/model setup. Member creation always requires an explicit Provider/model
choice; recommendations cannot bind silently.

DSH/Pi identity, exact version, provenance, health, qualification,
update/rollback, and affected work appear only for recovery or advanced
diagnostics. There is no Installed Agents destination, Agent/Harness store,
native DSH/Pi UI, consumer subscription/billing surface, or broad
marketplace/family console.

## 6. Assistant and Project-group conversation contract

Outside a Project the visible conversation is with the global Personal
Assistant. Inside a Project the visible conversation is the Owner, manager,
and Members' Project group; it is not a single-recipient employee rail:

- the manager speaks by default;
- a Member speaks proactively only when mentioned, submitting a deliverable,
  handing off, blocked, or requesting a decision;
- `@manager` can request progress or delegation;
- `@member` can ask or temporarily redirect work inside the approved boundary;
- work-changing text becomes a formal Task or revision before it has authority;
- ordinary process traces remain collapsed behind work objects;
- drafts persist across Project/Assistant context changes and cannot send on
  navigation;
- Member/manager output remains candidate/observation;
- HITL is announced in the right-column chat and linked to the center-canvas
  preview; chat has no Approve control and no “Don’t ask again” grant;
- consequential suggestions open a daemon preview in the central surface;
- no raw chain-of-thought or fabricated confidence percentage is shown.

The Personal Assistant layers explanation: a one-sentence answer, expandable
basis/sources/scope, then an audit link. Uncertainty is stated as missing or
conflicting evidence with a concrete next step.

## 7. Form and confirmation contract

Project setup, Role/Member creation, imports, Model Connections, capability
grants, cost-policy edits, and approval use visible labels, constraints before
entry, validation after blur/submit, preserved values, exact field errors,
async status, and keyboard focus on the first error. High-risk changes use a
review step with edit links and consequence/reversibility/source/cost facts.

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
P7-T05 review evidence does not transfer to this target. Owner approval of the
accepted competitive-informed V2 interaction baseline is not usability,
accessibility,
backend, Gate, release, qualification, or acceptance evidence. Canvas
runtime/render, NVDA, host-theme contrast, and 200% real layout remain
`not-run`.
