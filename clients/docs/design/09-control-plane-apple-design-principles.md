# 09 — Control Plane Apple Design Principles

- Phase: Product Redesign Phase 1 (design-only)
- Date: 2026-08-24
- Authority: `apple-design` skill is the PRIMARY DESIGN AUTHORITY in this stack; this document translates it into binding principles for the Control Plane, then runs the Apple Design Review over the recommended IA (`06`). It also inherits the already-accepted visual direction of `web-ui-design.md` §5 (Apple-inspired, system-like; no purple "AI" gradients, no card walls, no ornamental dashboard strips) — that direction stands; this document sharpens it.
- **Non-negotiable framing:** Apple design here ≠ glassmorphism, Liquid Glass everywhere, blur, giant gradients, oversized rounded cards, marketing landing pages, or empty whitespace. Target feel: **Calm, Dense, Precise, Professional.**

---

## 1. The eight principles, translated for a control plane

Apple's eight design principles (*Principles of Great Design*) applied to this product:

| Principle | Translation for the Control Plane |
|---|---|
| **1. Purpose** | Every surface exists to answer an operator question (§3 of `06`). Anything that doesn't answer "what needs me / what is it doing / what proves it / what can I safely change" is not built. The deleted list (raw-JSON-as-default, debug affordances, Bindings-as-space) is Purpose applied. |
| **2. Agency** | The owner stays in control through *real* levers: preview→admit, CAS, typed lifecycle verbs where they exist. Forgiveness is honest: re-bind reverses a binding; a tombstone is forever and says so; there is no fake undo. Confirmation is reserved for genuinely consequential acts (key removal, restore, revoke) — not routine reads. |
| **3. Responsibility** | Secrets never rendered; untrusted agent/provider text escaped; capability honesty (not-run with dependencies) is a safety feature, not a limitation apology. The UI anticipates misuse: fallback/per-request-override are explained as policy, not offered as rejectable traps. |
| **4. Familiarity** | Master/detail, inspector, sidebar, command palette, timeline — macOS-native spatial grammar (source list → content → inspector). Same-looking things behave the same: every state badge, every reason code, every "not available" renders identically everywhere. One deviation is allowed only with proof — none is currently proposed. |
| **5. Flexibility** | Desktop-primary; narrow windows stay operable (inspection-first at small widths); keyboard-complete; respects reduced motion/transparency/contrast and text sizing. No mobile product claim (inherited boundary). |
| **6. Simplicity — not minimalism** | Density is not the enemy; *unearned* elements are. A dense table with stable columns is simpler than a sparse card wall that forces vertical scanning. "Burying everything in one place looks minimal but isn't simple" — hence family depth in Resources instead of one flat browser. |
| **7. Craft** | Every spacing, state, and label is deliberate and defensible: exact reason codes, copyable stable IDs, aligned numeric columns, timestamps with real semantics. Jittery loads, shifting layouts, and ambiguous badges read as carelessness — banned. |
| **8. Delight** | The chosen emotion is **calm confidence**: the surface feels like a well-kept instrument. Delight comes from truth legibility (the evidence chain rendered beautifully), instant response, and motion that orients — never from decoration. |

## 2. Clarity / Deference / Depth on this IA

- **Clarity:** text is legible at density (system font, size-specific tracking, leading tuned tighter for data rows); state vocabulary is text+shape+color, never color alone; labels are specific ("Work", "Providers", "Evidence") not clever; every number is honest (unknown ≠ 0).
- **Deference:** chrome is quiet — one status strip, one sidebar; content is the authority data, not the frame. No ornamental illustration in operational surfaces; empty states are invitations with one action, not artwork.
- **Depth:** hierarchy through *real layers*: space → master → inspector → detail route → evidence. Navigation conveys position (selected space, object identity, breadcrumb of digests/refs). Layers are navigational, not visual-effects stacked.

## 3. Motion and response policy (operator-tool calibration)

The `apple-design` motion system is calibrated down for a dense operator tool — the principles that survive are about *response and orientation*, not play:

1. **Kill latency (kept, strengthened):** feedback on pointer-down (pressed states on rows/buttons); no artificial transition waits on the input path; optimistic *presentation* of selection (selection is local, instant) while *authority* facts load with explicit loading states — never conflating the two.
2. **Interruptibility (kept in spirit):** no modal animation locks; route changes are instant or short cross-fades; a user who clicks elsewhere mid-transition lands there immediately.
3. **Spatial consistency (kept):** inspector opens from the selection and closes back into it; detail routes enter/exit along the same path; the command palette originates from its invocation affordance.
4. **Springs/momentum (mostly not applicable):** this is a pointer-and-keyboard tool with few gesture-driven surfaces; where drag exists (column resize, future timeline scrub), 1:1 tracking and velocity handoff apply. Default spring when needed: critically damped (`damping 1.0`, response `0.3–0.4`).
5. **Ambient motion:** at most one ambient cue (e.g. live watch connectivity), subtle, and removed under `prefers-reduced-motion` — inherited from `web-ui-design.md:174-176`.
6. **Reduced motion/transparency/contrast:** cross-fades instead of slides; frostier/solid surfaces; near-solid backgrounds with defined borders — all three media queries are design requirements, not post-hoc fixes.

## 4. Materials, color, typography (direction for the visual phase)

- **Materials:** translucency is reserved for *floating functional layers only* (command palette, inspectors over content, the status strip when content scrolls under it). Content surfaces are solid. Never stack light translucency on translucency; never blur for atmosphere. This is the anti-glassmorphism guardrail with the legitimate use preserved.
- **Color:** quiet neutral base (the shipped cool-neutral dark direction is compatible; **light/dark adaptive is a visual-phase decision**, logged DD-12); semantic color is spent almost entirely on the state vocabulary (ready/degraded/blocked/unknown/not-run; verified/failed) with text+shape redundancy; accent color reserved for the primary action and selection. No gradients in operational surfaces.
- **Typography:** system font first (it ships optical sizing and tracking tables); size-specific tracking (tighten large text, near-zero body, slightly positive for small data labels); leading inverse to size, tightened for dense rows; hierarchy from weight+size+leading as a set; monospace reserved for digests/IDs/curson values, used generously — it is the native texture of authority data. Layout scales with the user's text-size setting (rem/em spacing).
- **Iconography:** sparse, semantic, never decorative; state icons have text partners; no icon walls.

## 5. Apple Design Review of the recommended IA (the brief's §16 checklist)

Review of `06` against the failure modes the brief names:

| Check | Verdict | Note |
|---|---|---|
| Over-carded? | **Pass** | Cards are banned as list/navigation furniture; identity cards survive only inside the Agent dossier where each card is a distinct authority identity (justified by content, not aesthetics) |
| Over-rounded? | **Pass (direction)** | Corner radius is a visual-phase token; direction set: small, system-like radii; no pill-shaped furniture |
| Over-blur? | **Pass** | Materials rule §4: translucency only for floating layers |
| Over-gradient? | **Pass** | Banned in operational surfaces |
| Dashboard-template feel? | **Guarded pass** | Home is a queue + state, no metric cards (DD-03); the risk is real and is the visual phase's first review gate |
| SaaS-admin feel? | **Guarded pass** | System space is the only admin-like area and is deliberately low-frequency; Providers is governance, not settings furniture — its pages lead with state and bindings, not forms |
| AI-startup slop? | **Pass** | No sparkle/magic vocabulary, no "AI is working" theater, no confidence percentages; the product's vocabulary is authority vocabulary |
| Apple style at the expense of operator efficiency? | **Pass** | Where restraint conflicts with scan speed, scan speed wins (stable columns, dense rows, keyboard model, command palette); the eight principles themselves demand this (Purpose, Craft) |
| Wayfinding (where am I / where can I go / how do I get out)? | **Pass** | Selected space + object identity + strip; deep links; designed 404; back/forward preserves list context |
| Feedback four kinds (status/completion/warning/error)? | **Pass with translation** | "Completion" is always evidence-linked (never a bare success toast for authority acts); confirmations are receipts with digests |

## 6. The Calm–Dense resolution (the brief's core tension)

"Calm" and "Dense" reconcile through hierarchy, not through removal:

1. **Calm is the chrome; dense is the content.** One quiet frame (strip + sidebar); inside it, information-dense masters and inspectors.
2. **Calm is the default reading; density is one disclosure away.** Row → inspector → detail → evidence: each layer doubles the density without raising the resting noise floor.
3. **Calm is stability.** Nothing moves that didn't change: stable column order, stable queue ordering, no layout shift on refresh, selection preserved. A dense surface that holds still reads calmer than a sparse one that twitches.
4. **Dense means *informative*, not *small*.** Row height and type stay legible; density comes from removing decoration and redundancy, not from shrinking text below comfort.

## 7. Accessibility commitments (binding on the visual phase)

Inherited from `web-ui-design.md:191-193` and strengthened: keyboard-complete operation (lists, inspector, palette, dialogs); visible focus with sufficient contrast; semantic tables/forms; status announcements for watch updates; color-independent state vocabulary; respects reduced motion/transparency/contrast; touch targets sized for real use where narrow layouts are used; disabled controls always explain why (class-C actions are text, not disabled buttons — which removes the "why is this disabled" class of failure entirely).

---

*Decisions originating here are logged in `10-control-plane-design-decisions.md` (DD-03, DD-10, DD-12, DD-14).*
