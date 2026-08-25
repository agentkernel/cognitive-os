# 06 — Control Plane Recommended IA

- Phase: Product Redesign Phase 1 (design-only)
- Date: 2026-08-24
- Decision: **Option D — Supervision IA** (analysis in `05-control-plane-ia-options.md`; logged as DD-01 in `10-control-plane-design-decisions.md`).
- This document is the IA *specification* for the next phase (UX/visual design). It defines structure, routes, states, and the navigation system — no components, no code, no visual styling.

---

## 1. The shell

```text
┌──────────────────────────────────────────────────────────────────┐
│ STATUS STRIP (persistent, one line): daemon ●ready · session     │
│ principal · channel health · active-alerts count · watch state   │
├────────────┬─────────────────────────────────────────────────────┤
│ SIDEBAR    │  SPACE                                              │
│            │                                                     │
│ Home       │   ┌───────────────────────────┬─────────────────┐  │
│ Work       │   │ MASTER (list/queue/timeline)│ INSPECTOR       │  │
│ Agents     │   │ stable, filterable,        │ (contextual:    │  │
│ Providers  │   │ keeps position             │  selected item) │  │
│ Resources  │   └───────────────────────────┴─────────────────┘  │
│ Activity   │                                                     │
│ System     │                                                     │
│            │                                                     │
│ ────────── │                                                     │
│ ⌘K Command │                                                     │
└────────────┴─────────────────────────────────────────────────────┘
```

1. **Status strip** — persistent one-line system truth: daemon reachability, overall readiness (`ready|degraded|blocked`), session principal + expiry, unacknowledged alert count, watch connectivity. It is the "instrument bezel": always visible, never a page. It answers J-K1's cheapest layer without navigating.
2. **Sidebar** — seven spaces (below), specific labels (per Apple guidance: name contents, avoid vague umbrellas), selected state unmistakable (not font-weight-only). No badge spam: only counts that change behavior (needs-attention on Home; unacknowledged alerts).
3. **Master + Inspector** — every list-like space uses a stable master list with an inspector for the selected object (desktop-primary). The inspector is where *reading* deepens; full detail *routes* exist for deep work and deep links.
4. **Command palette (⌘K)** — speed layer over destinations, objects (by exact ID), and the class-A/B actions of the current context. Disabled commands show reasons. It never exposes actions the backend lacks (class C/D honesty holds here too).
5. **Session** — utility chrome: the strip shows session state; re-authentication is a focused gate (modal-less inline panel pattern, as shipped) reachable from the strip, never a sidebar peer.

Navigation model (per navigation-IA brief): hub-and-spoke spaces + master/detail inside spaces + command/search layer + status strip as current-location anchor. Route-state rules (loading/empty/partial/stale/404/denied/disconnected/first-run) are specified per space below and are part of IA, not visual afterthoughts.

---

## 2. Route map (proposed SPA routes; HashRouter constraint inherited until the daemon gains an SPA fallback — recorded as a technical constraint, not a design choice)

| Route | Space | Content |
|---|---|---|
| `#/` | Home | attention surface |
| `#/work` | Work | task/run inventory (master) + selection inspector |
| `#/work/:taskRef` | Work | task detail: Overview · Intent & Contract · Context · Run · Effects · Evidence |
| `#/work/:taskRef/run` | Work | the Run reading (timeline composition of the task's execution trace) |
| `#/agents` | Agents | actor inventory |
| `#/agents/:id` | Agents | dossier: Identity · Binding · Current work · Activity |
| `#/providers` | Providers | accounts master + inspector |
| `#/providers/:id` | Providers | account detail: Overview · Models · Bindings · Usage · Audit |
| `#/resources` | Resources | family hub (four honest family cards = navigation, not decoration) |
| `#/resources/memory` … | Resources | Memory / Skills / Tools / Context family lists |
| `#/resources/memory/:id` etc. | Resources | family object detail (explain views) |
| `#/activity` | Activity | activity reading: attention-relevant events; per-object deep timelines linked; unified feed labeled with its BD-5 honesty state |
| `#/system` | System | readiness + doctor detail |
| `#/system/stewardship` | System | backup / restore (with their irreversibility framing) |
| `#/session` | (utility) | session gate — reachable, never in sidebar |

Deep-link rules: every detail route is a stable deep link; list state (filter/sort/selection) survives back/forward; unknown routes render a designed 404 with a way home (fixing the current empty-main defect).

---

## 3. The seven spaces

### 3.1 Home — the attention surface

- **Primary question:** what needs me, and is the system ready? (J-K1, J-K3, J-R1)
- **Content contract (anti-dashboard rule):** no charts, no KPI cards. Three composed regions:
  1. **Readiness line** — overall + per-component states in the daemon's vocabulary, each linking to System detail. Unknown/not-probed rendered as such (doctor sub-sections are placeholder-backed — named, not hidden).
  2. **Needs attention** — a priority queue of authority facts: degraded/blocked components, failed or unknown-outcome Effects, unacknowledged alerts, degraded bindings, stale watches. Every row: what, why (reason code), since when, and one next action (a link, or a class-A/B action where one exists).
  3. **Current work** — the in-flight tasks strip (from the task inventory, BD-3-honest: until BD-3, this strip shows *known* task_refs from this session/watch with an explicit "inventory is partial" note — never fabricated completeness).
- **Empty state:** a ready system with no attention items is a designed state ("ready; nothing needs you") — calm, not celebratory.
- **Why this is not a dashboard:** every element is a navigable authority fact or an action; nothing is a metric about metrics.

### 3.2 Work — tasks and runs

- **Primary question:** what work exists, what state is it in, what happened? (J-K2, J-I1, J-I2, J-C2-honest)
- **Master:** task inventory — columns: task_ref (short), objective (when BD-3 provides; otherwise honest envelope), state (daemon vocabulary), agent, age, evidence disposition. Filters: state, agent. Until BD-3: envelope list + "recently observed this session" section, both labeled.
- **Detail** (`#/work/:taskRef`) — six sections:
  1. **Overview:** state, contract digest/epoch, agent identity, budget facts, verification/acceptance disposition (evidence-linked).
  2. **Intent & Contract:** the record→interpret→preview→admit chain with digests, ambiguities, and information gaps from interpretation.
  3. **Context:** the resolved Context view — selected sources, versions, and **explicit losses** (omitted/truncated/conflict/stale/budget) when the projection provides them; otherwise an honest not-backed state.
  4. **Run:** the timeline reading — lifecycle transitions + process observations + watch events in one ordered, identity-labeled lane-pair (**authority lane** vs **observation lane**, visually distinct per conceptual-model rule 2). Watch: attach/detach (detach never cancels), reconnect with stale marker, cursor visible.
  5. **Effects:** stage/outcome/reconcile per effect; unknown outcomes foregrounded (J-R2).
  6. **Evidence:** terminal evidence — verification report refs/digests, currency, acceptance record; artifact links.
- **Actions:** class A/B where they exist (admit follow-ups through the governed chain; detach; copy IDs). Class C (cancel/pause/resume/retry) rendered as **Not available over HTTP** with the CLI verb path named (BD-1). Never a disabled button pretending.
- **Creation:** "New task" is the governed chain as a guided flow: objective → interpretation review (ambiguities surfaced) → preview (exact contract, budget, tools, context policy) → admit. Wave 1 keeps the shipped workspace-search draft type; the draft builder is generalized only when the contract surface genuinely supports it (honesty over breadth).

### 3.3 Agents — actor inventory and dossiers

- **Primary question:** which actors exist, how is each governed, what is it doing? (J-K2, canonical job 6)
- **Master:** instance ID, display identity, lifecycle/health (labeled by identity source — registration state ≠ process liveness), binding state (callable/blocked), current task when known.
- **Dossier** (`#/agents/:id`) — four sections:
  1. **Identity:** the seven runtime identities as distinct cards (package/installation/registration/instance/sidecar/execution/process) with digests — the shipped 9-card discipline retained.
  2. **Binding:** current fixed account+model, revision, dispatchability, change action (lands in Providers flow with the agent preselected).
  3. **Current work:** execution/task linkage when the projection provides it; otherwise named not-backed (BD-2/BD-3).
  4. **Activity:** the actor-scoped slice of Activity.
- **Controls:** the five lifecycle verbs render class-C (not available over HTTP; CLI path named). The dossier header states this once, calmly — an operator expectation-setting line, not an apology wall.
- **dsh:** the dsh runtime snapshot (state/process/sessions) renders as that agent's observation lane, labeled as dsh-specific.

### 3.4 Providers — egress governance

- **Primary question:** which accounts/models exist, are they reachable, who uses them, what do they cost? (canonical jobs 1; J-C1, J-I3)
- **Structure:** accounts master + detail. Detail sections:
  1. **Overview:** kind, endpoint (redacted), network scope, trust grants, status (active/revoked/degraded + `secret_ref_resolves`), catalog revision, last probe (class, duration, error class).
  2. **Models:** catalog with source (discovered/manual — manual visibly less certain), pricing (unknown = `cost_unavailable`, never zero), refresh (bounded probe), manual add, set-price.
  3. **Bindings:** this account's bindings; set/change with **revision-aware preview** (exact agent, account, model, expected revision, consequences for running work); remove (with the dsh overlay consequence named).
  4. **Usage:** token/cost events and aggregates, unknown-as-unknown; budgets (observe-only, labeled as advisory), alerts (acknowledge = class B).
  5. **Audit:** this account's audit events.
- **Key handling:** the secret-entry affordance (memory-only, non-echoing, discarded) exactly as constrained by ADR-0053; set/rotate/remove with explicit consequence copy. No key-shaped string ever rendered.
- **Creation flow:** validate → trust confirmation (when required) → persist → secret input → store → probe → verify — the documented order (`web-ui-design.md:97-107`), kept.

### 3.5 Resources — the four families, with family-native depth

- **Primary question:** what do my agents know, and what may they use? (canonical jobs 2-4)
- The hub is four navigation entries, not a generic browser (conceptual model §4: flattening the families is the current design's failure).
- **Memory:** list (envelope, limit honesty) + object explain (candidate→decision→object provenance, scope, purpose, version, expiry, tombstone state) + remember/forget actions (class A) + search labeled with BD-6 state.
- **Skills:** packages/revisions list, binding explain (scope, target, status, revocation reason), import (bounded local path flow), bind/revoke (class A, CAS). Content≠permission reminder is a standing annotation, not a dismissible banner.
- **Tools:** the 7-family registry with descriptor digest, risk class, lifecycle state, execution-readiness (with the "execution-ready ≠ production-wired" annotation), per-task exposure. Enable/disable/quarantine/revoke = class A with consequence copy (quarantine is one-way except revoke — stated).
- **Context:** per-task views linked from Work; a standalone Context browser is deferred (no standalone HTTP surface found — named, not faked).

### 3.6 Activity — the evidence stream

- **Primary question:** what happened, in what order, with what proof? (J-I1, J-I2, J-R2)
- Wave 1 (backend-honest): **attention-ordered reading** composed from real sources — unacknowledged alerts, failed/unknown effects, recent terminal evidence, watch-observed transitions — each row linking to its object detail. Plus per-object timelines reachable from Work/Agents detail.
- The **unified cross-domain feed** is shown as its BD-5-honest state: until the daemon aggregates, Activity says what it is ("provider-plane audit + observed task events") and what it is not ("not a complete authority event log").
- Audit (provider plane) lives here in full; management mutations outside the provider plane are labeled as not audited over HTTP (inventory §9).
- Presentation: a timeline, not a log viewer — grouped by object, identity-preserving (Task/Effect/Evidence/Process labels per `web-ui-design.md:183-184`), cursor/stale semantics visible.

### 3.7 System — readiness, doctor, stewardship

- **Primary question:** is the substrate healthy, and how do I fix/backup/restore it? (J-R1, canonical job 7)
- **Readiness/doctor:** full component detail (system, database, secret, provider, daemon, pi), doctor facts/guidance; sub-sections (six-resource, headless-vault, operability) rendered as their real placeholder state with the probes-not-wired note.
- **Stewardship:** backup (secret-excluding, digest-bound, with the "keys are never in archives" statement), restore (with its 409 failure classes as designed copy), both class-A flows with preview-first framing. Upgrade/uninstall: CLI-only, rendered as guidance (class C).
- **Session:** current principal/channel state, expiry, re-authenticate gate, clear-session (with the daemon-side "no revoke endpoint" honesty note, BD-7).

---

## 4. Density and disclosure strategy (the brief's §18 constraint)

High density without high cognitive load — the chosen instruments:

| Instrument | Where | Rule |
|---|---|---|
| Progressive disclosure | everywhere | summary → inspector → full route; the 5-second/5-minute/50-minute layering (trust skill) maps to row → inspector → detail |
| Master/detail + inspector | Work, Agents, Providers, Resources | list keeps position; selection never navigates away for reading |
| Timeline | Work/Run, Activity | authority lane vs observation lane; grouped by object |
| Status vocabulary | global | text + shape + color (never color alone); blocked/failed/unknown/not-run are distinct, calm, and explained |
| Inline actions | rows/inspector | only class B; class A always confirms with exact IDs; class C is text + CLI guidance, not a button |
| Command palette | global ⌘K | destinations + objects by ID + contextual class-A/B actions; disabled reasons shown |
| Keyboard model | global | full keyboard operation (a11y target inherited from `web-ui-design.md:191-193`); list navigation, inspector, palette, confirm/cancel |
| Saved filters | Work, Activity | wave-1: named filter presets; persisted server-side only if a route exists (else session-local, labeled) |
| Badges/counts | sidebar, strip | only counts that change behavior (attention, alerts) |

Explicitly rejected density anti-patterns (from the pattern matrix and Apple review): card walls as navigation, nested cards in tables, modal chains, hover-only controls, decorative metric tiles, "AI sparkle" signaling.

---

## 5. Route state matrix (IA-level, binding on visual design)

Every space/route must render all applicable states with real copy:

| State | Requirement |
|---|---|
| Loading | stable layout; no spinner-only screens; skeletons only where layout is stable |
| Empty (authoritative) | says how data arrives + the one next action (e.g. Providers empty → create account) |
| Partial | names what is missing and what still works |
| Stale | cursor/age visible; refresh path; never silent |
| Denied (401/403) | says which channel/principal; links to session gate |
| Disconnected | says the daemon is unreachable; keeps last good state labeled |
| Unknown / not-run | first-class, explained, sourced |
| 404 | designed page, way home |
| First-run | readiness-led guidance (System → Providers → first task), skip paths preserved |

---

## 6. Continuity with the canonical product IA

This recommendation **keeps the canonical five spaces** (Home, Agents, Tasks→renamed *Work*, Resources, Activity — rename is a label decision, logged DD-06) and makes two sanctioned evolutions:

1. **Providers promoted to first-level** — anticipated by `web-ui-design.md:41-48` ("dedicated operator view"); earned by P8-T13's depth (7 sub-surfaces).
2. **System added** — readiness/doctor/stewardship/session were homeless in the canonical five; the shipped UI parked their fragments in Home/Session. System is their honest grouping.

Bindings (current first-level) folds into Providers and Agents. Session demotes to chrome. **No canonical space is deleted.** If the owner prefers strict canonical labels, the delta is exactly two labels (Work→Tasks, +System) — recorded as an open decision point (DD-06) rather than assumed.

---

*Feeds: `07-control-plane-user-flows.md` instantiates this IA as flows; `08` applies Agent UX; `09` applies Apple principles; decisions in `10`.*
