# 14 — Work Spec (Task & Run Inventory)

- Status: adopted Personal 2.0 Work target; historical Task/Run spec retained
- Updated: 2026-08-27
- Contract: `06` §3.2 (Work space), jobs J-K2/J-I1/J-I2, capability reality: **BD-3** (no rich task list; Resource Manager `list?family=task` is envelope-only, limit 64; no objective text, no state field beyond `health:"contracted"`). This spec is therefore written in two honesty tiers: **Tier-1 (ships on today's API)** and **Tier-2 (activates when BD-3 lands)** — same layout, deeper columns. The layout never lies about which tier it is in.
- Layout: MID (master–inspector–detail) per shell `12` §4.

## Personal 2.0 Work model

Work is not a task inventory alone. It is the explicit managed-work projection:

`Goal -> Plan revision -> Task -> attempt -> Effect -> Verification/Acceptance`

Context belongs here because it explains what managed work received. One or
more Agents may participate only under daemon orchestration.

### Master hierarchy

- **Goal groups** are the top scan unit: objective, current Plan revision,
  attention state, participating Agents, and latest verified outcome.
- Expanding a Goal shows immutable **Plan revisions** with reason for revision.
- Each revision groups **Tasks**, and each Task groups its **attempts**.
- Filters operate on Goal state, Agent, provenance, attention, and recency.
- Stable IDs and exact daemon labels remain available in the inspector.

The current backend has no Goal, Plan-revision or first-class attempt projection
and only an envelope task list. Therefore:

- current Task rows may render in a clearly labeled "Current implementation"
  view using real P7-T05 facts;
- Goal/Plan/attempt grouping, orchestration, rich inventory and revision controls
  are `Requires-backend`;
- the UI never synthesizes Goal or Plan from conversation text;
- partial task inventory never claims completeness.

### Manage with Personal entry

Manage with Personal begins from an Agent conversation. The target flow reviews
the proposed Goal, Plan inputs, Context, resources, participating Agents,
budget, side effects and known losses before any durable admission. Abandoning
before admission leaves native conversation unchanged. Successful admission
returns stable refs and opens Work. This target flow is
`Requires-core + Requires-backend` where Goal/Plan/attempt public semantics are
needed, beyond the current Task record/interpret/preview/admit chain.

### Progress and control

Progress means recorded Goal/Plan/Task/attempt state, not elapsed animation,
token stream or an Agent narrative. If no bounded denominator exists, show
phase/state and last recorded fact rather than a percentage. Pause, cancel,
retry, re-plan and orchestration actions are active only when a typed daemon
service and allowed-action fact exist; otherwise their target slots say
`Requires-backend` without button styling.

The Tier-1/Tier-2 task honesty mechanics below remain applicable inside the
Current implementation view, but no longer define the target Work IA.

---

## 1. Master (the inventory)

```text
┌─────────────────────────────────────────────────────────────────────────┐
│ Work                                              [+ New task]    ⌘K    │
│ ┌─ Filter: state ▾ (all) · agent ▾ (all) · ▣ this session only ───────┐ │
│ │                                                                      │ │
│ │ ● a3f9c2…  workspace search        running      pi      4m     ›     │ │
│ │ ○ b71c04…  workspace search        awaiting…    —       12m    ›     │ │
│ │ ✓ 9e02f1…  workspace search        completed ✓  pi      26m    ›     │ │
│ │ ■ 77be0a…  workspace search        failed       pi      1h     ›     │ │
│ │ ■ d455e7…  workspace search        OUTCOME_UNKNOWN pi   41m    ›     │ │
│ │                                                                      │ │
│ │ Showing 5 known tasks · inventory is envelope-only (BD-3)            │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

### Columns (Tier-1 / Tier-2)

| Column | Tier-1 (today) | Tier-2 (BD-3) |
|---|---|---|
| State | from evidence/watch when observed this session; otherwise S7 `unknown` | daemon lifecycle field |
| Task | short `task_ref` (mono) | + objective text |
| Type | draft-type label (`workspace search`) | contract type |
| Agent | when known from session/watch | bound agent |
| Age | first-observed/updated | created/updated |
| Evidence | ✓/■/— when evidence loaded this session | verification disposition |

### Filter bar

- State filter: the state-system vocabulary (multi-select chips); Agent filter; "this session only" toggle (default ON in Tier-1 — it is the honest set).
- Sort: recency default; stable; no playful orderings.
- The filter bar always shows the active filter as visible chips (recognition over recall).

### The honesty footer (persistent in Tier-1)

"Showing N known tasks · inventory is envelope-only (BD-3)" — one quiet caption line. Tier-2 removes it. This line is the difference between a partial list and a lying one.

## 2. Inspector (selection)

Selecting a row (single action) opens the inspector — the 5-minute layer:

```text
┌─ Inspector ─────────────────────────────┐
│ task a3f9c2…                            │
│ ● running · 4m                          │
│ agent pi · draft workspace-search       │
│                                         │
│ contract  b91e…77  epoch 3              │
│ admitted  12:41:07  by local/owner      │
│                                         │
│ effects   2 executed · 0 unknown        │
│ evidence  not yet terminal              │
│ watch     live · cursor 118             │
│                                         │
│ [Open detail]        [Copy task ref]    │
│ Task cancel is not available over HTTP  │
│ yet — `cognitive …` (BD-1)              │
└─────────────────────────────────────────┘
```

- Facts: state+reason, agent, draft type, contract digest/epoch (mono, copyable), admission time/principal, effect rollup counts, evidence disposition, watch state.
- Actions: Open detail (primary), Copy task ref, Attach/detach watch **[B]**. Class-C block for cancel/pause (DD-08): text + CLI path, not a button.
- Inspector never edits. Editing happens in governed flows (preview→admit), never inline in a list.

## 3. Detail relationship

- Single action = select (inspector). Double action / `return` / "Open detail" = detail route `#/work/:taskRef` (replaces content region; master state preserved on back).
- The inspector is the *triage* layer; the detail is the *work* layer. Nothing in the inspector requires the detail to be open; nothing in the detail requires returning to the list.

## 4. New task entry

`+ New task` (primary action, top-right of master) opens the governed creation flow (Flow 6 in `07`): objective → interpretation review (ambiguities first-class) → preview (digest) → admit. This is a full-route flow, not a modal — admission is a consequential act and deserves a route, focus management, and a review step.

## 5. States

| State | Master rendering |
|---|---|
| Empty (no tasks ever observed) | "No work observed yet." + primary action "New task" + quiet note that the daemon's list is envelope-only |
| Loading | stable column skeleton (static bars, no shimmer) |
| Partial (evidence/envelope mismatch) | row-level S7 on the missing facet only |
| Stale | "as of <age>" in the footer + refresh affordance |
| Denied | channel explanation + session gate link |
| Disconnected | last-good list labeled; no fabrication |
| Tier transition (BD-3 lands) | footer disappears; columns deepen; no layout shift |

## 6. Behavior rules

1. The list is **stable**: rows never reorder while the pointer hovers; live updates mark rows changed-since-view with a quiet dot, re-sort on next explicit refresh.
2. Row selection is preserved across refresh by `task_ref`, not by index.
3. Counts in the filter bar reflect the *loaded* set, labeled as such in Tier-1.
4. Copy affordances: task_ref, contract digest, evidence digest — one click, mono, with copied confirmation (subtle, inline).

---

*Detail route specified in `15-control-plane-work-detail-spec.md`. The Run reading lives there; the inventory's job is triage, not narration.*
