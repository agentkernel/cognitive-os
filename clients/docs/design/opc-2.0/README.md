# Personal 2.0 OPC interaction-design corpus

- Status: owner-approved target design; implementation and qualification
  remain capability-gated
- Date: 2026-08-28
- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Canonical product intent:
  [Product design](../../../../personal/docs/product/product-design.md)
- Version boundary:
  [Personal 2.0 scope](../../../../personal/docs/product/personal-2.0-scope.md)
- Product vocabulary:
  [OPC product model](../../../../personal/docs/product/opc-product-model.md)
- Current interaction prototype:
  [**personal-20-opc-e2e (post journey-subtraction)**](personal-20-opc-e2e.canvas.tsx)
- Archived historical V2 (not current chrome):
  [pre-subtraction history](history/2026-08-28-pre-subtraction/README.md)
- Prototype identity: current chrome is `personal-20-opc-e2e` after the
  2026-08-28/29 journey-subtraction workshop (five-stage create, process-axis
  Projects, Today as decision packet + run overview + assistant). Archived V2
  CEO-rail / X-hero files are historical only. Canvas-only HITL and daemon
  authority path remain. This is not the pre-overwrite overlay-conversation /
  stacked-column V2.
- Cursor-openable copy (IDE detection path; not a second product baseline):
  [personal-20-opc-e2e](C:\Users\wuron\.cursor\projects\d-agent-kernel\canvases\personal-20-opc-e2e.canvas.tsx)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence
- Archived pre-subtraction prototypes:
  [2026-08-28 history](history/2026-08-28-pre-subtraction/README.md)
- Frozen predecessor:
  [2026-08-27 Control Plane corpus](../legacy-control-plane-20260827/README.md)

## Design thesis

Personal 2.0 is a calm, dense, AI-native digital-staff console for one local
Owner. The interface is organized around goals and openable, acceptable
deliverables first, operational state second, and configuration last.

The stable first-level anchors are **Today / Projects / Knowledge**. Settings
is fixed at the bottom. Team and Inbox are not first-level destinations.
The shell locks left navigation, center canvas, and right conversation; a
narrow canvas scrolls horizontally and does not stack those columns. The
right column is always the conversation; there is no overlay “open
conversation” control. Native mobile, pairing, and cloud 24/7 chrome are 2.1
and are not drawn here.

The workbench does **not** draw a visible CEO loop. Live Project is the
business-process axis plus the current stage plus the project group.
Today, after five-stage create 验收, leads with one decision packet plus
live-project run overview (counts and period toggle) plus assistant. Four
exception swimlanes are not default Today blocks. Operations defaults to
**Candidate → Intent persisted → Fence → Execute → Independent verify →
Receipt**. Project publish preview is the full AUTONOMY packet on the canvas;
there is no Confirm in chat. Knowledge Context shows why each fragment was
selected; chat auto-admits to inspectable Memory. Secrets use SecretStore takeover
and never appear in chat. `@` inserts only into the unsent draft.

Outside a Project the global Personal Assistant is the conversation identity.
Inside a Project the primary workbench combines the Owner/manager/Members
group conversation with a source-linked operating canvas. HITL is announced in
chat and confirmed on the center canvas; chat has no Approve control and no
“Don’t ask again” grant. Member rosters, attention, approval, recovery, and
diagnostics open in context; they are not permanent navigation destinations.

Routine reporting uses one stable, versioned Project template. An ad-hoc
question composes a temporary canvas from approved typed components and real
Project results. It is not saved unless pinned or converted to a template, and
it can neither execute generated code nor invent or hide data.

## Numbered documents

| # | Document | Responsibility |
|---:|---|---|
| 01 | [Product model and JTBD](01-product-model-and-jtbd.md) | jobs, evidence boundary, Role Template/Member/Task/Attempt model, outcomes |
| 02 | [IA and app shell](02-information-architecture-and-app-shell.md) | three anchors, Settings, Assistant/group workbench, contextual surfaces, routes |
| 03 | [Today, Projects, and operating canvas](03-today-projects-and-briefing.md) | expected outputs, deliverables, Project template, ad-hoc canvas, reflection |
| 04 | [Research-first Project setup](04-guided-project-setup.md) | research, output contracts, team, capabilities, HITL, simulation, preview, receipt |
| 05 | [Roles, Members, and conversations](05-team-roles-employees-and-conversations.md) | manager, Runtime Templates, Project Members, group conversation, version/rollback |
| 06 | [Knowledge, Vault, Context, and Memory](06-knowledge-vault-and-memory.md) | source archive, model-window compression, admission, correction, promotion, forget |
| 07 | [Contextual attention and recovery](07-inbox-approval-and-recovery.md) | approval, unknown/reconcile, Routines, no-overlap, missed work, safe-point changes |
| 08 | [Settings, Model Connections, and capabilities](08-settings-agents-providers-and-usage.md) | quick/custom connection, explicit Member model, cost, Skill/MCP, diagnostics |
| 09 | [State, accessibility, and visual system](09-state-accessibility-and-visual-system.md) | complete state grammar, keyboard/focus, restrained Apple-informed craft |
| 10 | [Component map and prototype flows](10-component-map-and-prototype-flows.md) | typed canvas/component model and v2 prototype scenarios |
| 11 | [Design-to-code and backend matrix](11-design-to-code-and-backend-matrix.md) | current foundations, target dependencies, pending plan reconciliation |
| 12 | [Scenario and heuristic review](12-scenario-and-heuristic-review.md) | requirement-family scenarios and future executed evidence |

The legacy words retained in a few filenames preserve inbound links only; they
do not define current navigation or product objects.

## Cross-cutting rules

1. Use only Today, Projects, and Knowledge as first-level anchors; keep Settings
   at the bottom.
2. Keep `Role Runtime Template -> Project Member Runtime definition -> Task ->
   Attempt -> disposable Agent process` distinct. Members are not shared across
   Projects; only Templates may be reused. “Digital staff” is positioning
   language, not an extra object.
3. Conversation proposes; every work-changing message resolves to a daemon-owned
   Task or revision before it has authority.
4. Cards and canvases lead with goal, expected result, deliverable,
   acceptance/evidence, Owner decision, source, freshness, and next action.
5. A missing backend is explanatory `Requires-backend`; native or external
   qualification gaps are `Requires-environment`. Neither is a fake control.
6. Unknown is never zero, healthy, complete, or safe to retry. Agent self-report,
   process exit, Provider/Tool success, and engine checkpoints are not
   independent verification.
7. Secrets, bearer tokens, resolvable SecretRefs, prompts, and raw Provider
   traffic never appear in DOM, URL, browser storage, ordinary configuration,
   conversation, Vault, export, logs, or evidence.
8. Product cost is source-labelled actual, estimated, or unknown and may warn;
   Personal 2.0 does not automatically stop work at a product budget threshold.
9. DSH and Pi are hidden managed engines. There is no everyday engine store,
   alternate Harness selector, native engine UI, or native session sync.
10. The target is Windows desktop while the host is online. Responsive
    narrow-window behavior is not native mobile or 2.1 remote operation.

## Reconciliation and evidence boundary

Architecture, accepted ADR text, formal plan/task IDs, and handbook pages that
still encode the 2026-08-27 model are **pending architecture/plan/handbook
reconciliation**. They remain dated facts but do not override the current
product sources listed above.

This corpus and the current `personal-20-opc-e2e` prototype are
specifications only. They do not
prove backend feasibility, Windows integration, DSH/Pi qualification,
Provider/MCP/X connectivity, accessibility conformance, human usability, market
demand, business benefit, or a fixed-denominator acceptance result.
