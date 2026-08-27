# Personal 2.0 OPC interaction-design corpus

- Status: current target design; backend and Windows host mostly unimplemented
- Date: 2026-08-27
- Product decision:
  [ADR-0059](../../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Product source:
  [Personal product index](../../../../personal/docs/product/README.md)
- Prototype: external Cursor Canvas
  `personal-2-opc-product-prototype.canvas.tsx`
- Prior corpus:
  [frozen pre-OPC design](../legacy-control-plane-20260827/README.md)

## Design thesis

Personal is a calm Windows business console for one Owner to operate governed
Projects and digital employees. Stable navigation provides orientation;
Project briefings explain the business; Inbox concentrates decisions and
recovery; the right Personal Assistant translates intent into candidates and
daemon previews. Advanced Agent/resource mechanics remain one disclosure
deeper.

The design uses:

- guided setup with review for first Project creation;
- priority stack for Today and Inbox;
- master/detail for Projects, Team, Knowledge, Installed Agents, and Providers;
- plan preview + progress + artifacts for long-running work;
- searchable grouped Settings;
- one active assistant/employee composer with retained drafts.

## Numbered documents

| # | Document | Responsibility |
|---:|---|---|
| 01 | [Product model and JTBD](01-product-model-and-jtbd.md) | Owner evidence boundary, Project/Role/Employee model, jobs, outcome, terminology |
| 02 | [IA and app shell](02-information-architecture-and-app-shell.md) | routes, sidebar, right assistant, location, responsive window behavior |
| 03 | [Today, Projects, and briefing](03-today-projects-and-briefing.md) | attention narrative, Project list, briefing, Task/Attempt drilldown |
| 04 | [Guided Project setup](04-guided-project-setup.md) | conversational research, draft/preview/confirm/receipt, form states |
| 05 | [Team, roles, and conversations](05-team-roles-employees-and-conversations.md) | manager, Blueprint, Assignment, employee cards, single composer |
| 06 | [Knowledge, Vault, and Memory](06-knowledge-vault-and-memory.md) | import, source rights, indexing, archive retrieval, correction/forget |
| 07 | [Inbox and long-running work](07-inbox-approval-and-recovery.md) | approvals, unknown/reconcile, Routines, offline/missed, background choice |
| 08 | [Settings and advanced operations](08-settings-agents-providers-and-usage.md) | Installed Agents/DSH, Providers, binding, budgets, usage, privacy |
| 09 | [State, accessibility, and visual system](09-state-accessibility-and-visual-system.md) | every non-happy state, keyboard/focus, restrained Apple-inspired craft |
| 10 | [Component map and prototype flows](10-component-map-and-prototype-flows.md) | composable UI primitives and switchable prototype scenarios |
| 11 | [Design-to-code and backend matrix](11-design-to-code-and-backend-matrix.md) | current sources, target dependencies, Requires-backend behavior |
| 12 | [Scenario and heuristic review](12-scenario-and-heuristic-review.md) | static review protocol, state coverage, future executed evidence |

## Cross-cutting rules

1. Every surface covers empty, loading, partial, stale, permission, error,
   unknown, offline, missed, success, and archived states where applicable.
2. A missing backend is explanatory `Requires-backend`, never a decorative or
   disabled active-looking action.
3. Cards answer goal, responsibility, state, next action, latest verified
   evidence, cost basis, source, and freshness.
4. Agent self-report, process exit, Provider success, Tool result, or engine
   checkpoint never receives a Verified treatment.
5. Only daemon-issued previews can be confirmed. Assistant explanations and
   employee proposals remain candidates.
6. Secrets, bearer tokens, resolvable SecretRefs, prompts, and raw Provider
   traffic never appear in the DOM, URL, browser storage, export, or evidence.
7. Exactly one composer is active; switching Assistant/employee preserves
   drafts and cannot send.
8. The target is Windows desktop. Responsive narrow-window behavior is not a
   native mobile or 2.1 remote design.

## Design evidence boundary

The prototype and this corpus provide design coverage only. They do not prove
backend feasibility, Windows integration, DSH qualification, connector
reliability, accessibility conformance, or human usability. Phase 11 owns
implementation and fixed-denominator validation; all tasks remain separately
claimable.
