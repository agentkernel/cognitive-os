# 12 — Control Plane App Shell

- Status: adopted Personal 2.0 app-shell target; historical shell retained
- Updated: 2026-08-27
- Contract: IA per `06-control-plane-recommended-ia.md`; state language per `22`; tokens per `11`. Archetype decision (from `desktop-app-archetypes`): **monitoring cockpit with command-center chrome** — "priority stack + timeline + detail pane" inside, "status rail + command surface" as the frame. Explicitly NOT the SaaS dashboard layout (no KPI grid, no welcome hero, no card wall).

## Personal 2.0 app shell

The adopted shell replaces the earlier status-strip + seven-space cockpit:

```text
┌─ Navigation ─┬──────────── Primary workspace ────────────┬─ Inspector ─┐
│ Home         │ selected route/object                      │ summary     │
│ Agents       │ conversation · master/detail · Work ·      │ provenance  │
│ Work         │ Library · Activity · Settings              │ full facts  │
│ Library      │                                             │ actions     │
│ Activity     │                                             │ gaps        │
│ Settings     │                                             │             │
├──────────────┴─────────────────────────────────────────────┴─────────────┤
│ Global Agent Shell: explain · compare · propose · daemon preview · receipt│
└──────────────────────────────────────────────────────────────────────────┘
```

### Region ownership

- **Navigation:** six destinations, selected state, daemon/readiness summary,
  active Agent/conversation, and command-palette entry. Provider/System are not
  peers; they are Settings groups.
- **Primary workspace:** the selected product object and its dominant task.
  List/master-detail is default for repeated operations; conversation is
  default inside an Agent.
- **Inspector:** beginner summary first, then source, identity, freshness,
  capability matrix, provenance, digests, raw redacted facts, and current-backed
  actions. The inspector never becomes a second navigation tree.
- **Agent Shell:** globally reachable, candidate-only explainer/proposal layer.
  It can collapse to a labeled bar, expand without covering the active object,
  and preserve its local prompt draft. Vendor-native conversation/composer
  remains inside Agents. The Shell never stores credentials, sends a native
  turn by inference, or silently converts chat to Work.

### Shell behavior

1. Selected object and Agent context survive route changes. Native conversation
   state remains source-owned in the Agents workspace.
2. `Manage with Personal` from a native conversation or a Shell proposal asks
   the daemon for a preview and, after real admission, opens the resulting Work
   object. Without backend support it appears only as a
   `Requires-backend` specification—not a button.
3. `⌘/Ctrl+K` opens the command palette; all shell and palette behavior has a
   complete keyboard path and focus return.
4. Status cells link to Settings/System or Activity filters; they never become
   an ornamental top strip.
5. At wide desktop widths all three regions may coexist. At narrower widths,
   inspector and detail become explicit overlays/routes; navigation remains
   recoverable. No separate mobile-product claim is made.
6. Reduced motion replaces spatial transitions with short fades; reduced
   transparency makes overlays solid.

### Current-versus-target boundary

The P7-T05 SPA currently has a flat route shell and no embedded Agent
conversation. Its current seven routes remain documented in
[Current State Map](control-plane-current-state.md). This shell requires new
frontend structure and multiple backend projections; target-only portions are
`Requires-backend`.

---

## 1. Shell anatomy

```text
┌────────────────────────────────────────────────────────────────────────┐
│ STATUS STRIP  (34px, persistent, elevation.0, hairline bottom)         │
│ ● daemon ready   principal://local/owner · mgmt+task · expires 27m     │
│                        ⌄ 2 alerts      watch: live      ⌘K             │
├───────────┬────────────────────────────────────────────────────────────┤
│ PRIMARY   │  SPACE CONTENT                                             │
│ NAV       │                                                            │
│ (sidebar) │   ┌─ MASTER ────────────────┐  ┌─ INSPECTOR (context) ──┐ │
│           │   │ stable list / queue     │  │ selected object facts  │ │
│ Home      │   │                         │  │ state · reason · links │ │
│ Work      │   │                         │  │ actions (class A/B)    │ │
│ Agents    │   │                         │  └────────────────────────┘ │
│ Providers │   └─────────────────────────┘                             │
│ Resources │        (detail routes replace this region, keeping        │
│ Activity  │         strip + sidebar; inspector may persist)           │
│ System    │                                                            │
│           │                                                            │
│ ───────── │                                                            │
│ ⌘K        │  (command layer trigger; also global shortcut)             │
└───────────┴────────────────────────────────────────────────────────────┘
```

Five layers, each with one job:

| Layer | Job | Never |
|---|---|---|
| **Status strip** | global system truth + session truth + watch truth + alert count + ⌘K affordance | navigation, marketing, breadcrumbs |
| **Primary nav (sidebar)** | the seven spaces; current-location anchor | utilities, session, settings-as-peer (they live in System), badges beyond attention/alerts |
| **Secondary nav** | inside a space: family tabs (Resources), section anchors (Work detail), account sub-nav (Providers) | competing active states with the sidebar |
| **Contextual nav** | object-relative moves: task→its agent, agent→its binding, effect→its evidence, "next/previous" in a filtered list | duplicating primary nav |
| **Command layer (⌘K)** | speed: navigate / search / inspect / act (class A/B in context) | exposing class-C/D capabilities |

## 2. Status strip (the instrument bezel)

Left→right composition (each cell is a state-system rendering):

1. **Daemon cell:** S-category dot + `daemon ready|degraded|blocked|unreachable`. Click → System.
2. **Readiness cell:** overall readiness word + worst-component reason on hover-inspector. Click → System readiness detail.
3. **Session cell:** principal + channels (`mgmt+task`) + idle expiry countdown. Click → session gate/detail (System→Session). Expiring (<5 min) renders S4; expired renders S7 + re-gate affordance.
4. **Alerts cell:** unacknowledged count (numeral + label, S4 when >0). Click → Activity filtered to alerts.
5. **Watch cell:** `watch: live|stale|disconnected` (+cursor age). Click → the watch-bearing view or Activity.
6. **⌘K cell:** command layer affordance (also purely keyboard-driven).

Rules: one line, 34 px, `type.mono-label` for values; cells are buttons with plain destinations; the strip never scrolls away; it shows *truth*, so it is allowed to be quiet when everything is nominal (all S1 = visually near-silent).

## 3. Primary navigation (sidebar)

- Seven items, `type.body`, object icons (14 px), selected = accent keyline-left + tint background + label weight 600 (triple signal: color+weight+keyline).
- Counts: only Home (attention queue depth) and Activity (unacknowledged alerts) may carry numerals; both are S4-styled and disappear at zero. No other badges ever.
- Footer of sidebar: ⌘K item + product wordmark "CognitiveOS Personal" + the standing disclaimer "Daemon client · not an authority writer" (caption, quiet) — the product's honesty signature, kept from the shipped UI.
- Collapse: at 1280–1439 collapses to icon rail with label-on-hover; below that, top tab strip (see `11` §7).

## 4. Content region patterns

Three sanctioned content layouts (no fourth may be invented without a design-decision entry):

1. **Master–Inspector (MI):** stable master list + right inspector (Agents, Providers, Resources families, Activity). Selection is instant (local); facts load with skeleton-free stable layout.
2. **Master–Inspector–Detail (MID):** master + inspector + full detail route on double-action (Work). Detail replaces the content region, keeps strip+sidebar, preserves master state on return.
3. **Composed surface (CS):** Home and System only — vertically stacked named regions with their own disclosure rules (no master list).

## 5. Session layer

- Gate: inline panel over the intended destination (destination title visible, content gated) — the shipped sidebar-fix pattern, kept. Fields: principal (prefilled `principal://local/owner`), bootstrap secret (non-echoing, memory-only, cleared on submit), explicit copy: "This is the daemon bootstrap secret from `local-bootstrap.secret` — not a Provider API key."
- Post-issue: strip session cell becomes live; expired mid-use → inline re-gate at the current route with all local presentation state preserved.
- Honesty: "Clear session" clears client memory only; the strip notes daemon-side sessions expire by idle/absolute timeout (BD-7: no revoke endpoint).

## 6. Keyboard & focus model (shell-level)

- ⌘K palette; `/` focuses palette in search mode; `g then h/w/a/p/r/c/s` navigate spaces; `j/k` move in masters; `return` opens inspector, `return` again opens detail; `esc` unwinds one layer (palette → inspector → detail → master); `[`/`]` previous/next object in the current filtered master.
- Focus: visible 2 px accent ring, offset 2 px; focus moves to the main heading on route change; focus returns to invoker on overlay close; skip-link first in tab order.
- No keyboard traps outside true dialogs; the palette traps focus only while open and always returns it.

## 7. Responsive behavior (from `11` §7, shell rules)

| Width | Strip | Sidebar | Master | Inspector |
|---|---|---|---|---|
| ≥1680 | full | full labels | full | docked right, resizable 280–400 |
| 1440–1679 | full | full labels | full | floating overlay right |
| 1280–1439 | full | icon rail | narrowed | floating overlay |
| 960–1279 | full (cells compress to icons+counts) | top tab strip | stacked with detail | bottom sheet |
| <960 | compressed (daemon+alerts+⌘K) | top strip | full-width list | sheet |

Degradation is inspection-first: every fact remains reachable; dense multi-pane operation is a desktop feature and says so.

## 8. What the shell deliberately lacks (anti-SaaS-dashboard guards)

No hero/welcome banner; no KPI grid; no user avatar menu (single owner — identity is the session cell); no "getting started" checklist furniture after first-run (first-run guidance lives in Home's empty/attention states and dissolves when the system is ready); no marketing copy anywhere; no notification center (DD-14); no settings gear in chrome (System holds it).

---

*The shell binds every page spec that follows. Pages specify their content regions (MI/MID/CS), secondary/contextual nav, and per-region states — they do not redesign the frame.*
