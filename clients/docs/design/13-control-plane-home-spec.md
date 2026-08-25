# 13 — Home Spec (Attention Surface)

- Phase 2 (design-only)
- Date: 2026-08-24
- Contract: DD-03 (attention surface, not dashboard); jobs J-K1/J-K3/J-R1; Flow 1 in `07`; state language `22`; shell `12` (composed surface, CS layout).
- **Design thesis:** Home answers three questions in reading order — *Is the system ready? What needs me? What is in flight?* — and nothing else. Every element is a navigable authority fact or one action. There are no metrics about metrics.

---

## 1. Information architecture of the surface

Five regions, fixed order, individually collapsible (collapse state is session-local):

| # | Region | Question answered | Source (verified) | Default density |
|---|---|---|---|---|
| R1 | **Readiness** | Is the system ready? What component is worst? | `/personal/status` ≡ `/personal/readiness` | one line, expandable |
| R2 | **Needs attention** | What requires me, in priority order? | composed from readiness + effects (unknown/failed) + alerts + degraded bindings + stale watch | up to 5 rows + "N more" |
| R3 | **Current work** | What is in flight right now? | task inventory (BD-3-honest: session-observed + envelope list, labeled) | up to 4 rows |
| R4 | **Waiting on input** | What is paused on a precondition I control? | interpretation `clarification_required`, awaiting-admission tasks, pending reconciliation | folded into R2 until volume justifies separation (see §4) |
| R5 | **Recent evidence** | What finished verifiably (or failed verification) lately? | per-task `GET /task/evidence` for recently observed tasks | up to 3 rows |
| R6 | **Critical changes** | What consequential governance mutations happened recently? | provider-plane audit + mutation receipts (key rotated, binding changed, tool quarantined, restore applied) | rendered as the **top-labeled group inside R2**, not a separate region (see below) |

R6 resolution (brief conformance): the brief requires Critical changes on Home. A separate region duplicated R2 and risked a changelog junk drawer; instead, R2's queue carries a `change` kind group — consequential mutations (S5/S4-relevant: key removal, binding change, tool quarantine/revoke, restore) render at the top of Needs attention with their kind label, then age out on acknowledge/next-visit. Design decision recorded here and in `25` (DC-3).

## 2. Wireframe (1440 px)

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ ● daemon ready   principal://local/owner · mgmt+task · 27m   2 alerts ▏watch: live ▏⌘K │
├───────────┬──────────────────────────────────────────────────────────────┤
│ Home      │  Home                                                        │
│ ●───────  │                                                              │
│ Work      │  READINESS                                                   │
│ Agents    │  ● ready — all components nominal        last checked 12s ago │
│ Providers │  └─ expand ▾  system ● · database ● · secret ● · provider ◆ · │
│ Resources │     daemon ● · pi ●                        → System detail    │
│ Activity  │                                                              │
│ System    │  NEEDS ATTENTION                                             │
│           │  ◆ provider deepseek-main degraded — discovery failed 2h ago  │
│           │    → Repair provider                                  [open] │
│           │  ■ task a3f9… effect OUTCOME_UNKNOWN — reconcile pending 41m  │
│           │    → Open run timeline                                [open] │
│           │  ◆ 1 budget alert — deepseek-main at 82% (advisory)   [ack]  │
│           │  2 more ▾                                                      │
│           │                                                              │
│           │  CURRENT WORK                                                │
│           │  ● task a3f9…  workspace search — running 4m · agent pi       │
│           │  ○ task b71c…  workspace search — awaiting admission          │
│           │  Inventory is partial — daemon task listing is envelope-only  │
│           │  (BD-3). Recently observed this session.        → Open Work   │
│           │                                                              │
│           │  RECENT EVIDENCE                                             │
│           │  ✓ task 9e02… verified · report r-881… · 26m ago      [view]  │
│           │  ■ task 77be… verification failed · VERIFY_FAILED · 1h  [view]│
│           │                                                              │
└───────────┴──────────────────────────────────────────────────────────────┘
```

## 3. Region specs

### R1 Readiness line

- Default: one line — overall category + word + worst-component reason (if any) + "last checked" age. Expandable (chevron, `motion.fast`) to the six-component row (system/database/secret/provider/daemon/pi), each a state-system chip linking to System.
- Honesty: doctor sub-sections are placeholder-backed; the expanded row carries a quiet caption "six-resource / headless-vault / operability probes are not wired over HTTP (BD register)" when those facts are shown.
- Never: a gauge, a score, a percentage, a green banner.

### R2 Needs attention (the queue)

- Row anatomy: category dot + object-type icon + object label (mono short ID) + reason (one sentence, cause-first) + age + one next action (link or class-B action like alert ack).
- Order: blocked/failed (S5) → attention (S4) → waiting-on-owner (S3) → stale (watch/data). Stable within refresh; new arrivals insert in rank, never reshuffle existing rows under the cursor.
- Cap: 5 rows + "N more" disclosure → Activity (filtered) or the owning space.
- Empty state (designed, calm): "Ready. Nothing needs you." + last-checked age. No confetti, no illustration.

### R3 Current work

- Rows: task short-ref + objective (when BD-3 provides; otherwise `workspace search` draft-type label) + state (S2/S3) + agent + age.
- Honesty banner (persistent until BD-3): "Inventory is partial — daemon task listing is envelope-only." This is a capability-honesty feature, not an apology.
- Row → Work detail. Region footer → Work space.

### R4 Waiting on input

- Folded into R2 at current volumes (a waiting item *is* an attention item with S3). If the operator's task volume grows (multi-agent era), R4 separates as its own region between R2 and R3. The separation rule is defined now so the future change is structural, not visual: R4 = items whose precondition is *owner input* (clarification, admission); R2 keeps items whose precondition is *system state*.

### R5 Recent evidence

- Rows: S6/S5 + task short-ref + disposition (`verified` / `verification failed`) + report digest chip (copyable) + age → Work detail Evidence section.
- Cap 3; footer → Activity.
- Rule: this region is the product's trust signature — it shows proof, not activity. "What finished" without evidence does not appear here (it appears in R3/R2 with its honest state).

## 4. Density control (the brief's §5 questions, answered)

| Content | Default | Progressive disclosure | Goes to Work | Goes to Activity |
|---|---|---|---|---|
| Readiness overall | one line | component row | — | — |
| Attention items | 5 rows | "N more" | task items link to Work detail | full queue lives here too (alerts) |
| In-flight tasks | 4 rows | region link | the inventory itself | — |
| Waiting-on-input | inside queue | R4 separation rule | — | — |
| Recent evidence | 3 rows | region link | per-task evidence section | evidence stream |
| Critical changes | — (cut) | — | — | — |

## 5. States of the whole surface

| State | Rendering |
|---|---|
| First run (nothing configured) | R1 expanded by default; R2 contains the setup path as attention rows ("No provider account — create one →"); R3/R5 show their designed empty states ("No work observed yet — create a task in Work →"). No onboarding wizard; the queue *is* the guide. |
| Loading | regions render their frames with `loading` lines; no spinner-only surface; strip stays live |
| Disconnected (daemon unreachable) | strip daemon cell S5; regions keep last-good content labeled "last known, <age>"; readiness line becomes "daemon unreachable — retry ↻" |
| Denied (session expired) | inline gate over the surface (destination visible behind), per shell §5 |
| Partial (some projections failed) | failed regions render S7 with the failed source named; healthy regions unaffected |
| Stale | region-level "as of <cursor/age>" labels; never silent |

## 6. Interaction & behavior

- Auto-refresh: none in wave 1 (watch is process-local; polling policy is a visual-phase/implementation decision against daemon cost — recorded as Open Question OQ-2 in `25`). Manual refresh affordance per region + global `r`.
- All rows keyboard-reachable (`j/k` within region, `tab` between regions); every row has exactly one primary action; queue rows may add one class-B inline action (ack).
- No row ever navigates on single click without selection feedback; selection is instant, content follows.

## 7. What Home must never become (review guards)

1. A dashboard (charts, KPI tiles, trend sparklines).
2. A notification center (alerts appear once, in the queue, with ack).
3. A marketing/welcome surface.
4. A log tail (that's Activity's job, with evidence semantics).
5. A place where "unknown" is hidden to preserve calm (calm = *stable*, not *silent* — `09` §6).

---

*Wireframe fidelity: layout/hierarchy/behavior are binding; spacing/type/color execute via `11` tokens; visual treatment in `24`.*
