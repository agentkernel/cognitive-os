# 21 — Command Layer (⌘K)

- Status: adopted Personal 2.0 command-layer target
- Updated: 2026-08-27
- Contract: DD-09 (speed layer, not a space); navigation-IA reference (command palettes need a real action/retrieval model and must not become junk drawers); capability honesty (class A/B only — the palette never exposes class-C/D capabilities).

## Personal 2.0 command-layer amendment

The palette accelerates the six-space IA and global Agent Shell:

- switch to Home, Agents, Work, Library, Activity or Settings;
- navigate to an Agent/native conversation when the Adapter projection is
  available;
- open a known Goal, Plan revision, Task/attempt, Library item, account or
  timeline item;
- focus the global Agent Shell prompt for explanation/proposal, or navigate to
  the selected Agent's native composer;
- enter a current-backed governed flow;
- open a target-only capability's explanation, labeled `Requires-backend`.

The palette never sends an Agent message, converts conversation to Work,
imports credentials, performs resource writeback or executes a consequential
mutation directly. It can navigate to those review surfaces. Class-B current
actions may execute inline only with a durable/visible receipt.

Search scope is always declared: loaded client projections, current Adapter
history projection, or a real backend query. No result cannot mean no object
exists when coverage is partial. MCP entries and rules never expose host Agent
session controls.

Keyboard contract: `Ctrl/⌘+K`, predictable group order, arrows, Enter, Escape,
typeahead, complete screen-reader semantics, visible focus and exact focus
return. Reduced motion uses a short fade. The palette is an accelerator—not a
replacement for navigation, global Agent Shell, or full-depth inspectors.

The destination examples later in this file use the old seven-space IA and are
superseded by this section; their honesty, grouping and focus rules remain
adopted.

---

## 1. What ⌘K is for here

The operator's speed path across eight verbs from the brief, each mapped to real capability:

| Verb | In the palette | Backed by |
|---|---|---|
| **Navigate** | spaces, sections, saved filters | routes |
| **Search** | objects by exact ID/ref (task_ref, account, agent, binding, memory id, tool op id) + label substring over loaded lists | client-side over loaded projections (no server search exists — BD-6 honesty: palette searches *known* objects and says so) |
| **Inspect** | "inspect <object>" → opens the object at its inspector/detail | routes |
| **Create** | New task; Add provider account; Remember (memory); Import skill | class-A flows (each lands on its governed route, never executes in the palette) |
| **Configure** | change binding (from an agent/account context); set budget; set price | class-A flows, context-preselected |
| **Run** | attach/detach watch; refresh projections; bounded probe (provider) | class-B actions, executed with confirmation-free but receipt-visible feedback |
| **Repair** | rotate key; re-run probe; acknowledge alert; rebind (lands in flow) | class-A/B in context |
| **Verify** | "verify <task>" → opens Work detail Evidence section | route |

Two structural rules:

1. **The palette executes only class-B actions inline.** Class-A actions always land on their governed flow (preview→confirm). The palette is a *fast path to the right surface*, not a mutation console — this keeps preview-before-mutation inviolable.
2. **Class-C/D capabilities are absent, not disabled.** There is no "cancel task" entry at all (BD-1); discovering its absence happens in the Work detail header where the honest line + CLI path lives. (Rationale: a palette entry that says "unavailable" trains operators to expect the verb; the detail surface is where that expectation is educated. Recorded as a deliberate deviation from "disabled with reason" — the reason lives where the action would belong.)

## 2. Anatomy

```text
┌────────────────────────────────────────────────────────────┐
│ ⌘K  Search objects, actions, destinations…            esc  │
├────────────────────────────────────────────────────────────┤
│ ▸ ACTIONS (context: task a3f9…)                            │
│   Attach watch · Copy task ref · Open evidence             │
│ ▸ OBJECTS                                                  │
│   task a3f9c2… — running · pi                              │
│   provider deepseek-main — active                          │
│ ▸ DESTINATIONS                                             │
│   Work · Providers · Activity · System                     │
└────────────────────────────────────────────────────────────┘
```

- Grouped results: Actions (contextual) → Objects → Destinations → Help (handbook pointers). Empty query shows recents + the current context's actions.
- States (per navigation-IA requirements): closed/opening/open; query empty / loading / results / **no results** ("no known object matches — inventory is partial (BD-3)" when relevant) / permission-blocked (denied → session gate) / execution success-error (receipt line, inline).
- Presentation: floating material (`material.floating`), anchored top-center, `motion.overlay` spring (damping 1.0, response 0.3), same-path exit, focus returned to invoker. Reduced motion → instant cross-fade.
- Keyboard: full operation; `↑↓` move, `return` executes/navigates, `esc` closes; mouse fully equivalent (no hover-only content).

## 3. Scope discipline (anti-junk-drawer)

- The palette indexes **only what the spaces contain** (IA-bound scope, DD-09). New space content automatically appears; nothing palette-only may exist.
- Ranking: exact ID match > contextual action > recent > label substring. No fuzzy magic that surfaces surprising objects; operator tools prize predictability over cleverness.
- No secret-bearing input via the palette ever (key entry lives in its governed form); no free-text task creation (the governed chain owns that).

## 4. Relationship to search honesty

The palette's index is the loaded projection set. In Tier-1 (BD-3), object search covers session-known tasks + envelope lists; the no-results state names the boundary. This is the command-layer instance of the product rule: **unknown is a value, shown, with its reason.**

---

*Component execution of the palette (focus trap, list semantics, receipts) is specified in `23-control-plane-component-spec.md` §Command.*
