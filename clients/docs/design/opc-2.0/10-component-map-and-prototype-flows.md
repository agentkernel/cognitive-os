# 10 — Component map and v2 prototype flows

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

## App composition

```text
OpcAppShell  (locked left / center / right; overflow-x on narrow canvas)
  PrimaryNavigation
    Today
    Projects
    Knowledge
    Settings
  CenterColumn
    ContextHeader
    RouteStateBoundary
      TodayView
      ProjectsMasterDetail
      ProjectSetup
      ProjectOperatingCanvas
      KnowledgeMasterDetail
      SettingsWorkbench
    ContextInspector
      MemberRoster
      AttentionDetail
      SourceEvidenceDetail
      AdvancedDiagnostics
  RightColumnConversation
    AssistantOrProjectConversation
  ReceiptDrawer
  CloseBackgroundDialog
```

This is design composition, not an implementation module or API contract.

## Core components

| Component | Job | Required states |
|---|---|---|
| `GoalOutputHeader` | lead with goal, expected result, due/openable deliverable, acceptance | loading, partial, stale, accepted, attention |
| `AuthoritySpine` | **not default chrome** (archived V2). CEO six-step remains backend discipline, not a top rail | — |
| `DecisionPacket` | consequence, reversibility, alternatives, kernel truth, why A is first; collapse when nothing is pending | loading, stale, waiting-owner, receipt, collapsed |
| `ExceptionSwimlanes` | **not default Today blocks**. Needs you / Can continue / Unknown / Missed may merge into live-project overview rows | empty, partial, unknown, missed |
| `ProjectOperatingCanvas` | render stable source-linked Project report | loading, partial, stale, offline, archived |
| `TypedCanvasSection` | compose approved goal/artifact/evidence/timeline/decision/cost views | temporary, source-missing, pinned, template-saved |
| `ProjectGroupThread` | Owner/manager/Members group with bounded proactive speech | loading, offline, permission, failed-draft |
| `ContextComposer` | post to global Assistant or current Project group | idle, dirty, sending, failed-preserved, switched-preserved |
| `MemberCard` | responsibility, current work, next, accepted result, decision, cost | working, waiting, blocked, missed, runtime-unqualified |
| `RoleRuntimePreview` | exact Template plus Project Member Runtime diff | researching, candidate, validating, stale, confirmed/rejected |
| `AttentionItem` | explain exact Owner decision in Today/Project context | approval, input, block, unknown, missed, cost-warning |
| `ApprovalPreview` | daemon-issued diff and consequences | current, stale, editing, narrowed, rejected, applying, receipt |
| `AttemptLedger` | preserve Task tries and independent verification | queued, running, failed, unknown, verified |
| `RoutineStatus` | trigger, active/latest queued/missed/coalesced facts | scheduled, running, offline, missed, waiting-owner |
| `ContextBudgetView` | model-window priorities, omissions, truncation, freshness | fitting, compressed, conflict, required-source-missing |
| `MemoryLineage` | candidate, admission, correction, promotion, forget | candidate, admitted, conflict, promotion-preview, tombstoned |
| `WhyThisFragment` | why each Context excerpt was selected | selected, omitted, truncated, conflict, redacted |
| `AuthorityPath` | Candidate → Intent persisted → Fence → Execute → Independent verify → Receipt | candidate, persisted, fenced, executing, verifying, receipt |
| `ModelConnectionForm` | quick/custom connection plus explicit Member model | empty, checking, partial, secret-locked, saved |
| `CapabilityReview` | Skill/MCP source, version, permissions, grant, rollback | reviewing, confirm-required, compatible, failed, rollback |
| `CostReading` | actual/estimated/unknown source and warning | actual, estimated, unknown, warning, quota-unavailable |
| `EngineDiagnostics` | hidden DSH/Pi facts only for recovery | healthy, partial, drifted, unqualified, update/rollback |
| `CapabilityGap` | honest missing backend/environment | explanatory only; never executable |

## Typed canvas constraints

The approved component registry may project only real Project objects and
source-linked results. It cannot accept generated code, `eval`, arbitrary
script, hidden fetch, invented value, or untrusted layout instruction.
Mandatory goal/acceptance/failure/not-run/Owner-decision/source/freshness facts
cannot be suppressed. Ad-hoc canvases are temporary unless pinned or saved as
a Project template.

## Interaction rules

- Goal and deliverable precede state; state precedes configuration.
- Opening a contextual inspector preserves Project and selection.
- Lists preserve filters, sort, selection, and scroll.
- Dialogs restore trigger focus; `@` suggestions are keyboard accessible.
- Stale previews cannot confirm.
- Loading/error never erase last-known safe facts, form input, or drafts.
- Unknown never becomes zero, healthy, done, or retryable.
- Pause/stop/retry/resume appear only when backed.
- Native/Observed/Governed/Verified never imply confidence.
- A work-changing group message resolves to a Task/revision before execution.

## Owner-approved prototype source coverage

Current chrome is `personal-20-opc-e2e-optimized-v9` (2026-08-30). The numbered
list below is **archived V2 / pre-subtraction scene coverage**. It does not
define current create, Today, or live-Project chrome (no visible CEO rail; no
X/Twitter P0 hero; four swimlanes are not default Today blocks).

The owner-accepted competitive-informed v2 source provided navigation among
**(historical only; not current chrome)**:

1. visible CEO loop and Today decision packet plus four exception swimlanes
   (Needs you / Can continue / Unknown / Missed) — **not** v9 default chrome;
2. Project stable operating report first, then the X loop, plus right-column
   group conversation;
3. Project publish preview as the full AUTONOMY packet; no Confirm in chat;
   Package A thread preview plus acceptance; planned is not published;
4. temporary source-linked ad-hoc canvas with pin/save-template;
5. research-first setup and one-cycle simulation;
6. stale launch preview/re-preview;
7. Role Template and Project Member creation with explicit model;
8. Member Runtime comparison and rollback; process death does not delete the
   Member;
9. Knowledge Markdown Vault, optional Obsidian companion (not embedded),
   Context compression, and Memory lineage;
10. HITL announced in chat and linked to the center preview; no chat Approve;
    no “Don’t ask again”;
11. no-overlap/queue-latest/missed Routine; Working is not completion;
12. Model Connection gaps labelled `Requires-backend`; no Connect / Install /
    Confirm fake buttons;
13. Skill review and MCP exact-permission confirmation as labelled gaps;
14. hidden engine diagnostics;
15. X/Twitter publish/readback/reflection loop;
16. state/accessibility lab.

Each scene includes a non-happy state. Global controls may switch
online/offline and normal/partial/unknown views. No scene represents an old
formal-plan denominator. Native mobile, pairing, and cloud 24/7 chrome are
parked 2.1 and are not drawn.

## Conversation behavior

Outside a Project the composer targets the global Assistant in the right
column. Inside one it targets the Project group in that same third column;
`@manager` and `@member` route within that group. `@member` creates a formal
Task revision, not a shadow plan. Context switching
stores/restores separate drafts, never sends or merges, and announces the new
context. `@` inserts only into the unsent draft. HITL is announced in chat and linked to the center canvas; chat has
no Approve control and no “Don’t ask again” grant. Member work conversations
remain source drilldowns, not the shell's primary recipient model.

## Prototype non-effects

Static source checks found no network request, storage write, daemon call,
filesystem mutation, capability installation, approval, publication, or
Provider/Agent action. Example artifacts and receipts are labelled prototype
data. Canvas runtime/render, NVDA, host-theme contrast, and 200% real layout
remain `not-run`; this document claims no rendered behavior.
