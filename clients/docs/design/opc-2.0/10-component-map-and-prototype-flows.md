# 10 — Component map and prototype flows

## App composition

```text
OpcAppShell
  SidebarNav
  ContextHeader
  RouteStateBoundary
    TodayView
    ProjectsMasterDetail
    ProjectSetup
    TeamMasterDetail
    KnowledgeMasterDetail
    InboxMasterDetail
    SettingsWorkbench
  ConversationRail
  InspectorSheet
  ReceiptDrawer
  CloseBackgroundDialog
```

This is design composition, not an implementation module contract.

## Core components

| Component | Job | Required states |
|---|---|---|
| `ProjectBrief` | summarize goal, phase, manager, today, team, Inbox, results, cost | loading, partial, stale, offline, archived |
| `AttentionItem` | explain why Owner action is needed | approval, input, block, unknown, missed, budget |
| `EmployeeCard` | goal, responsibility, state, next, verified, cost | working, waiting, blocked, offline, runtime-unqualified |
| `RoleBlueprintPreview` | proposed role and Assignment diff | candidate, validating, stale, confirmed/rejected |
| `ConversationRail` | one recipient and retained draft | loading, offline, permission, error, success |
| `SingleComposer` | submit to exactly one visible recipient | idle, dirty, sending, failed-preserved, switched-preserved |
| `KnowledgeSourceRow` | source/provenance/rights/index state | importing, duplicate, failed, stale, excluded, indexed |
| `ApprovalPreview` | daemon-issued exact diff and consequence | current, stale, editing, rejected, applying, receipt |
| `AttemptLedger` | preserve each Task execution try | queued, running, failed, unknown, verified |
| `RoutineStatus` | active/latest queued/missed/coalesced facts | scheduled, running, offline, missed, waiting-owner |
| `InstalledAgentDossier` | DSH artifact/runtime truth | healthy, partial, drifted, unqualified, update/rollback |
| `UsageBudgetPanel` | Project/member/Task budget and source-labelled use | advisory, warning, stopped, quota unknown |
| `CapabilityGap` | honest missing dependency | Requires-backend / Requires-environment; never executable |

## Interaction rules

- Selection remains visible when an inspector opens.
- Lists preserve filters, sort, selection, and scroll.
- Dialogs manage focus and return it to the trigger.
- A stale preview cannot confirm.
- Loading never clears last-known safe facts without explanation.
- Error never clears form input or Conversation drafts.
- Unknown never becomes zero, healthy, done, or retryable.
- Stop/retry/resume appear only if the backend declares them.
- Native/Observed/Governed/Verified badges never imply confidence.

## Prototype navigation

The external Canvas prototype provides a top scene switcher for:

1. Today;
2. Project briefing;
3. Guided setup;
4. setup preview/re-preview;
5. Team role and employee card;
6. employee Conversation;
7. Knowledge import;
8. Inbox approval/recovery;
9. Settings > Installed Agents;
10. Settings > Providers;
11. Settings > Usage;
12. State lab.

Each scene includes at least one non-happy state. A global control switches
online/offline and normal/partial/unknown perspectives. Requires-backend items
are explanatory callouts, not buttons.

## Single-composer prototype behavior

Personal Assistant and employee tabs share one visible rail. Switching:

- changes the composer recipient label;
- stores current draft by recipient;
- restores the selected recipient's draft;
- does not submit or merge;
- keeps the central page unchanged;
- announces the recipient change.

## Prototype non-effects

The Canvas uses only `cursor/canvas`, React hooks re-exported there, and host
theme tokens. It performs no network request, storage write, daemon call,
filesystem mutation, import, approval, or Provider/Agent action. All receipts
are explicitly labelled prototype examples.
