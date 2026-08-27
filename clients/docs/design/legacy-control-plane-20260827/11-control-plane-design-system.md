# 11 — Control Plane Design System (Tokens)

- Status: adopted Personal 2.0 design-system specification; no code claim
- Updated: 2026-08-27
- Authority order: CognitiveOS Reality > Apple (`apple-design`) > UX/IA > Agent UX > frontend aesthetics. Token values follow Apple discipline: system font first, size-specific tracking, 4 pt spacing grid, restrained materials, motion that orients.
- Scope: token *definitions and rules*. Hex values are directional proposals for the visual phase to finalize against both themes and contrast gates; the **semantic structure is binding**, the exact hues are refinable.

## Personal 2.0 design-system amendment

The token proposal below becomes the seed of a **brand-new desktop system**,
not a promise to preserve the P7-T05 palette or dimensions.

### Semantic foundations

| Foundation | Binding target rule |
|---|---|
| Surfaces | `canvas / navigation / workspace / inspector / overlay`; solid by default, tonal depth before shadow |
| Type | human-readable UI face + compact data/identity mono; sentence case; tabular numerals |
| Spacing | 4 pt base; operational density with legible floors; no marketing-section gaps |
| Shape | small system radii; pills only for filters/status; no oversized rounded cards |
| Color | neutral field, one restrained selection accent, semantic state and provenance colors only |
| Motion | immediate feedback and orientation; no authority/progress inference; reduced-motion equivalent |
| Transparency | overlays only; solid replacement under reduced transparency |
| Focus | always-visible, high-contrast, not clipped by docked Shell/inspector |

### Provenance tokens

The state system and provenance system are independent:

- `provenance.native` — Agent/vendor-origin fact;
- `provenance.observed` — bounded process/transport observation;
- `provenance.governed` — daemon authority fact;
- `provenance.verified` — independent verification/acceptance fact.

Each uses label + icon/shape + subtle rule. Provenance color never substitutes
for state, and `Verified` is not a green synonym for "success".

### Three-region density

The target shell reserves tokens for navigation, primary workspace and
inspector. Widths are adaptive bands, not one fixed pixel contract. The
workspace is never constrained to a marketing content column. Master rows,
conversation messages, timelines, forms and inspectors each receive distinct
density floors; beginner summaries may be spacious, while expanded inspectors
remain compact and aligned.

### Agent Shell and conversation

Conversation content uses the same typography and state grammar as the rest of
the product. It avoids bubble decoration when a simple transcript row is
clearer. Agent identity, model/account route, provenance, attachment state and
Manage with Personal boundary remain visible. Native slots inherit common
spacing/focus/error tokens and cannot introduce an unrelated vendor theme into
the shell.

### Target-only states

`Requires-backend` has its own non-action treatment: neutral bordered
specification block, dependency label, and no hover/pressed styling. Loading
does not imply progress. Indeterminate work never receives a percentage.

Exact colors, type metrics and region widths below remain directional until
measured for light/dark/high-contrast themes. Their anti-glass, anti-gradient,
anti-card-wall, accessibility and density rules remain binding.

---

## 1. Typography

**Families**

| Token | Value | Role |
|---|---|---|
| `font.text` | system-ui stack (`-apple-system, "SF Pro Text", "Segoe UI", system-ui, sans-serif`) | everything by default |
| `font.mono` | ui-monospace stack (`"SF Mono", "Cascadia Mono", ui-monospace, monospace`) | digests, IDs, cursors, state labels, numerals in data columns, reason codes |

Mono is the **native texture of authority data** (DD-10, `09` §4): any value that is an identity, digest, cursor, epoch, revision, or exact count renders in mono. Prose never renders in mono.

**Scale** (size / tracking / leading as one set — never size alone):

| Token | Size | Tracking | Leading | Weight | Use |
|---|---|---|---|---|---|
| `type.title1` | 22 px | −0.02 em | 1.15 | 600 | space titles only |
| `type.title2` | 17 px | −0.015 em | 1.2 | 600 | detail header object name |
| `type.headline` | 13 px | −0.01 em | 1.3 | 600 | section titles, card titles |
| `type.body` | 13 px | 0 | 1.45 | 400 | reading text, forms |
| `type.row` | 12.5 px | 0 | 1.3 | 400 | dense table/list rows (operator default) |
| `type.label` | 11 px | +0.01 em | 1.25 | 500 | field labels, column headers (uppercase avoided; sentence case) |
| `type.caption` | 11 px | +0.01 em | 1.3 | 400 | muted reasons, timestamps |
| `type.mono-data` | 12 px | 0 | 1.35 | 400 | digests/IDs/cursors |
| `type.mono-label` | 11 px | +0.005 em | 1.3 | 500 | state labels, reason codes |

Rules: respect user text-size (layout in rem); tighten leading for dense rows, never below 1.25; no all-caps furniture (sentence case everywhere; the daemon's uppercase enums stay verbatim in mono, which is data, not furniture); numerals in data columns are tabular (`font-variant-numeric: tabular-nums`).

## 2. Color

**Structure (binding):** neutral base + one accent + seven state-category hues + semantic aliases. No brand gradients. No purple "AI" accent (banned by `web-ui-design.md` §5).

| Token group | Light (directional) | Dark (directional) | Use |
|---|---|---|---|
| `bg.canvas` | #F5F6F8 | #101318 | app background |
| `bg.surface` | #FFFFFF | #171B22 | content surfaces, lists |
| `bg.raised` | #FFFFFF | #1E242E | inspector, palette (floating layers only) |
| `fill.quiet` | #6B7280 | #9AA6B8 | muted text, reasons, timestamps |
| `fill.strong` | #111827 | #E8EDF5 | primary text |
| `accent` | #0A6CFF | #7EB6FF | primary action, selection, S2 active |
| `state.ready` | #1F9D55 | #4CC38A | S1 |
| `state.waiting` | #B7791F | #E5B567 | S3 |
| `state.attention` | #C05621 | #F0A35E | S4 |
| `state.blocked` | #C53030 | #F26D6D | S5 |
| `state.completed` | #4A5568 | #A0AEC0 | S6 (deliberately neutral — completion is proven by evidence link, not by green) |
| `state.unknown` | #718096 | #8B94A3 | S7 |

Rules: state hues appear in dots/rules/keylines, not in large fills; large surfaces stay neutral (calm); selection uses accent at 8–12% tint background + 1 px accent keyline; focus ring uses accent at 100%, 2 px, offset 2 px; all text/background pairs pass 4.5:1, state dots 3:1, both themes; `prefers-contrast: more` swaps tinted backgrounds for solid + border.

## 3. Spacing, sizing, density

4 pt base grid. Scale: `space.1=4, space.2=8, space.3=12, space.4=16, space.5=20, space.6=24, space.8=32, space.10=40`.

| Token | Value | Use |
|---|---|---|
| `density.row.compact` | 28 px | data-dense lists (Work inventory, Activity) — operator default |
| `density.row.regular` | 36 px | object lists (Agents, Providers, Resources) |
| `density.row.form` | 40 px | form rows, settings rows |
| `layout.sidebar` | 232 px (min 200, max 264) | primary nav |
| `layout.inspector` | 320 px (min 280, max 400, resizable) | contextual inspector |
| `layout.strip` | 34 px | status strip |
| `layout.content-max` | none (fluid) | operator surfaces do not center-column; reading text blocks cap at 68 ch inside detail prose only |

Density rule: density comes from removing decoration, not from shrinking below these floors (`09` §6.4).

## 4. Radius, border, elevation, materials

| Token | Value | Rule |
|---|---|---|
| `radius.sm` | 4 px | chips, small buttons, digest cells |
| `radius.md` | 6 px | panels, inputs, menus |
| `radius.lg` | 8 px | overlays (palette, sheets) — the ceiling |
| `border.hairline` | 1 px, `fill.quiet` at 12–18% | separators; preferred over shadows |
| `border.state` | 1 px, category hue | left-edge rule on state-relevant blocks (timeline S5, attention rows) |
| `elevation.0` | none | default — surfaces are flat |
| `elevation.1` | 0 1px 2px rgba(0,0,0,.18) | floating layers only (palette, menus, inspector when overlapping) |
| `material.floating` | `bg.raised` at 92–96% + backdrop blur ≤ 20 px, saturate 150–180% | **only** palette / menus / strip-overlap; never content surfaces; removed under `prefers-reduced-transparency` (solid `bg.raised`) |

Anti-abuse guards (binding): no stacked translucency; no gradient fills in operational surfaces; no shadow as separation (use hairlines); no glass cards; no blur behind content.

## 5. Motion

| Token | Value | Use |
|---|---|---|
| `motion.instant` | 80 ms ease-out | pointer-down press feedback (scale 0.98 or fill darken) |
| `motion.fast` | 140 ms ease-out | hover states, disclosure chevrons |
| `motion.enter` | 200 ms ease-out | route/section entry (opacity + ≤4 px translate, same-path exit) |
| `motion.overlay` | spring, damping 1.0, response 0.3 | palette/menus/inspector; enter+exit same path, anchored to invoker |
| `motion.pulse` | 2.4 s ease-in-out loop, opacity 40→100→40% | **S2 Active dot only** |
| `motion.never` | — | no layout-shifting animations, no animated numbers, no skeleton shimmer loops, no parallax |

Global: interruptible always (nothing locks input); reduced-motion → cross-fades (`opacity` 120 ms) and static S2 dot; motion never carries information that isn't also static (watch state has text, not just pulse).

## 6. Iconography

- Set: one consistent line-icon family, 1.5 px stroke, 14/16 px sizes; object icons (task, agent, provider, memory, skill, tool, context, effect, evidence, event) + state shapes (per `22` §6) + action glyphs (play/refresh/link/copy/acknowledge).
- Rules: icons never replace labels in navigation or state; object icons mark *type* in lists/timelines; no decorative icons in empty states (a single quiet glyph allowed); no emoji in product UI.

## 7. Breakpoints & responsive strategy (desktop-first)

Primary design targets: **1440 / 1680 / 1920 px** desktop widths. Mobile is a degradation strategy, not a product.

| Width | Shell behavior |
|---|---|
| ≥1680 | sidebar + master + inspector + content; inspector docked right, resizable |
| 1440–1679 | sidebar + master + content; inspector overlays right as a floating layer (material.floating) with focus return |
| 1280–1439 | sidebar collapsible to icons-with-labels-on-hover; master narrows; inspector becomes overlay |
| 960–1279 | sidebar becomes a top-edge tab strip; master/detail stack (list → detail route); inspector becomes a bottom sheet |
| <960 (degradation only) | single column; spaces via top strip; master lists full-width; detail = own route; inspector = sheet; command palette full-width. Inspection is supported; dense operation is not promised (inherited boundary: no mobile workflow claim) |

Invariants at every width: status strip visible; current-location cues intact; state vocabulary unchanged; keyboard path unchanged; no hover-only controls (hover enhancements always have focus/keyboard equivalents).

## 8. Elevation-free depth model

Depth is conveyed by (in priority order): 1) hairline separators, 2) background steps (canvas → surface → raised), 3) state keylines, 4) one floating material for true overlays. Shadows are the *last* resort and capped at `elevation.1`. This is the anti-"card-on-card" guardrail.

---

*Token consumption: page specs 13–21 reference tokens by name; `24-control-plane-visual-direction.md` applies them to the three core pages; implementation phase converts this file into the token layer (format left to implementation: CSS custom properties or a TS token module — decided then, not now).*
