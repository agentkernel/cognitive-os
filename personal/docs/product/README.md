# CognitiveOS Personal product design

- Status: canonical stable product-design index
- Current requirements:
  [Personal 2.0 OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Prior accepted decision:
  [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Product-direction amendment: owner-confirmed `/grill-me` design tree,
  2026-08-28, then
  [journey-subtraction workshop](personal-2.0-opc-journey-subtraction-workshop-2026-08-28.md)
  2026-08-28/29
- Current interaction prototype:
  [**personal-20-opc-e2e (post journey-subtraction)**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e.canvas.tsx)
- Archived historical V2 (not current chrome):
  [pre-subtraction history](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Prototype identity: the current canvas is post journey-subtraction, not V2.
  Archived V2 is historical chrome in the history folder. Canvas-only HITL and
  daemon authority path remain.
- Cursor-openable copy (IDE detection path; not a second product baseline):
  [personal-20-opc-e2e](C:\Users\wuron\.cursor\projects\d-agent-kernel\canvases\personal-20-opc-e2e.canvas.tsx)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence
- Current-status owner:
  [PROGRESS.md](../../../docs/plan/PROGRESS.md) `Current snapshot`
- Task/Gate owner:
  [PERSONAL-DEVELOPMENT-PLAN.md](../../../docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)
- Preserved release record: [Personal Linux 1.0](linux-1.0-scope.md)

This directory owns stable Personal product intent, user concepts, release
boundaries, information architecture, and user journeys. It does not own
implementation status, leases, campaign evidence, support claims, or Gate
results.

## Current direction

Personal 2.0 is a **Windows-first, owner-local, AI-native digital-staff
console for one-person companies and individual developers**. One human Owner
creates governed Projects in business language, supervises Project Members,
opens real deliverables, approves consequential actions, and verifies outcomes
without first learning Agent infrastructure.

The stable first-level anchors are **Today / Projects / Knowledge**, with
**Settings** at the bottom. Team and Inbox are not first-level destinations;
rosters, approvals, exceptions, and attention open in Project or Today
context. The desktop shell is a locked left / center / right layout: navigation,
operating canvas, and conversation. Conversation is always the third column;
there is no overlay “open conversation” control. A narrow canvas scrolls
horizontally and does not stack the three columns. Native mobile, pairing, and
cloud 24/7 chrome are 2.1 and are not drawn as current product chrome.

Outside a Project the conversational identity is the global **Personal
Assistant**; inside a Project it is the Owner, Project Manager, and Members'
**Project group conversation**. HITL is announced in chat and confirmed on the
center canvas preview; chat has no Approve control and no permanent “Don’t
ask again” grant. Timeboxed 「本周同一类对外不再问」 expires and is
revocable in Settings. `@` inserts only into the unsent draft.

The workbench does **not** draw a visible CEO six-step rail. Live Project is
a business-process axis plus the current stage. Today, after five-stage
create 验收, is one decision packet plus live-project run overview plus
chat — not a KPI card wall and not four swimlanes as default blocks.
Operations defaults to
**Candidate → Intent persisted → Fence → Execute → Independent verify →
Receipt** as backend discipline. Knowledge Context shows why each fragment
was selected; chat auto-admits to inspectable Memory. Secrets use SecretStore
takeover and never appear in chat. The prototype is an interactive spec, not
daemon `/ui/`. P0 is complete capabilities only: no demo Project, no X as
P0 hero.

A reusable **Role Runtime Template** becomes a Project-specific, long-lived
**Project Member Runtime definition**. Members are not shared across Projects;
only Templates may be reused. Each Task execution starts a disposable
**Agent process/Attempt**. “Digital staff / 数字员工” remains positioning
language, not an additional product object.

The direction is approved product semantics, not shipped capability. Most OPC
surfaces are **Requires-backend** and Windows remains unqualified. The delivered
Linux 1.0 product, current daemon-served `/ui/`, Provider Control Plane,
Resource Manager, Pi qualification, and dsh Path B are preserved factual
foundations only within their recorded boundaries. They are not the 2.0
product organization.

## Status vocabulary

| Label | Meaning |
|---|---|
| **Current implementation (Now)** | A repository-established capability. Exact current status still comes from `PROGRESS.md`. |
| **Adopted Personal 2.0 target** | Owner-approved Windows OPC product intent; never an implementation or support claim. |
| **Requires-backend** | A daemon, client, adapter, host, archive, or workflow capability is absent or insufficient. The UI must not fake an action. |
| **Requires-environment** | Acceptance needs a qualified Windows-native or campaign environment that does not yet exist. `not-run` is not pass. |
| **Deferred** | Explicitly outside the 2.0 success path, such as native mobile remote control or another Agent adapter. |

English product documents are canonical where a bilingual pair exists.
`*.zh-CN.md` files are faithful mirrors and link to the English source.

## Product corpus

### Core product and scope

| Document | Responsibility |
|---|---|
| [Requirements analysis](personal-2.0-opc-requirements-analysis.md) | complete owner-confirmed problem, JTBD, personas, principles, requirements, authority, metrics, supersession, and traceability |
| [Journey-subtraction workshop record](personal-2.0-opc-journey-subtraction-workshop-2026-08-28.md) | verbatim 2026-08-28/29 Q&A plus Keep/Cut/Park scheme snapshot; not Gate or implementation evidence |
| [Product design / PRD](product-design.md) | stable product thesis, priority, experience model, success measures, exclusions, and non-claims |
| [Personal 2.0 scope](personal-2.0-scope.md) | exact Windows-local inclusion, capability ledger, version boundary, and 2.1 deferral |
| [OPC product model](opc-product-model.md) | Project, Role Runtime Template, Project Member Runtime, Agent process/Attempt, Conversation, authority, and terminology |
| [User journeys](user-journeys.md) | five-stage first Project, daily Today, process-axis Project, approvals, recovery, knowledge, archive; X parked |
| [Long-running operations](long-running-operations.md) | Routine/Trigger, no-overlap, queue-latest, offline/missed, background choice, and receipts |

### Experience surfaces

| Document | Responsibility |
|---|---|
| [Web UI product design](web-ui-design.md) | OPC IA, app shell, Assistant/Project-group conversation, typed canvas, state matrix, and Requires-backend behavior |
| [Assistant, Member execution, and conversations](agent-integration-and-conversations.md) ([中文](agent-integration-and-conversations.zh-CN.md)) | global Assistant/Pi, Project group conversation, Project Members, disposable processes, and hidden DSH diagnostics |
| [Knowledge, Memory, and Vault](knowledge-memory-vault.md) | local archive, Context compression, Vault, feedback, Memory admission, correction, promotion, and forgetting |
| [Model Connections](account-hub.md) ([中文](account-hub.zh-CN.md)) | Provider templates/custom endpoints, explicit Member selection, SecretStore boundary, cost provenance, and warnings |
| [Provider Control Plane](provider-control-plane.md) | factual current Provider authority and the target Model Connections evolution |

### Factual foundations and external references

The documents in this subsection preserve Linux 1.0/current Provider/Resource
facts or advanced governance detail. They do **not** define the 2.0 navigation,
object hierarchy, or everyday experience.

| Document | Responsibility |
|---|---|
| [Cognitive resource model](cognitive-resource-model.md) | preserved six-family 1.0, capability-family boundary, and why OPC domain objects are not a generic Resource family |
| [Resource Manager](resource-manager-design.md) | factual common projections, family-native actions, knowledge indexing boundaries, and conflict behavior |
| [MCP capability governance](mcp-resource-family.md) ([中文](mcp-resource-family.zh-CN.md)) | assistant-led acquisition, review, pinning, per-Project/Member grants, underlying identities, and DSH base-tool prohibition |
| [Informative OSS matrix](oss-reference-matrix.md) | preserved exact 2026-08-27 research HEADs, the 2026-08-28 grilling-direction mapping note, and reuse/rejection boundaries; no new HEAD is qualified |

## Preserved and frozen material

- [Linux 1.0 scope](linux-1.0-scope.md) remains the finalized six-family,
  Pi-qualified 1.0 product boundary. The OPC rebaseline does not revise it.
- [Frozen 2026-08-27 agent-stewardship corpus](legacy-agent-stewardship-20260827/README.md)
  preserves the superseded cross-platform/external-Agent 2.0 target and its
  original index. It is not current semantics.
- Accepted ADR-0056/0058/0059 remain dated decisions. Current product semantics
  is amended by the 2026-08-28 owner requirements and the 2026-08-28/29
  workshop scheme. Architecture, ADR, formal plan, and handbook text that still
  reflects the 2026-08-27 object model or IA, or the V2 CEO-rail chrome, is
  **pending architecture/plan/handbook reconciliation**; accepted history is
  not rewritten here. Handbook generated pages were not regenerated in the
  workshop delivery.

## Fixed safety and claim boundaries

- The Rust daemon remains the sole authority writer. The Personal Assistant,
  Pi, DSH, Project Members, Agent processes, adapters, UI, MCP servers, and
  connectors are clients, candidate producers, or bounded executors.
- External mutation remains persist-before-dispatch Intent/Effect work under
  fencing and reconciliation. Independent verification remains required for
  completion.
- Secrets enter only approved Secret Stores through non-logging daemon paths.
  DSH/Pi Provider traffic is daemon-proxied.
- Project, Role Runtime Template, Project Member Runtime, Routine, Trigger,
  Attempt, Conversation, Vault, Model Connection, and cost reading are
  product/domain concepts, not new generic Cognitive Resource families or
  public Core schemas.
- Product adoption does not imply Windows support, DSH qualification,
  connector reliability, a Gate, release, Profile, business outcome, market
  validation, usability validation, 24/7 operation, or multi-Agent benefit.
