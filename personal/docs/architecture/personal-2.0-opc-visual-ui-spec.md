# Personal 2.0.0 OPC Visual UI specification (daemon `/ui/`)

# 个人 2.0.0 OPC 视觉 UI 规格（daemon `/ui/`）

- Status: **informative** specification / 非实现 / 非 support / 非 Gate
- Formal task: `P13-T12/D01` (documentation-only Delivery Slice). `P13-T12/D02`
  executes the rendered qualification; this document only says what "correct"
  looks like and how to judge it.
- Change class: `implementation-only` documentation — the product contract,
  the IA and the machine contracts are unchanged; nothing here adds a route,
  DTO, state, or authority.
- Frozen design prototype (not the product): owner-approved canvas v9
  `clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v9.canvas.tsx`
  (read only; never regenerated; v8 untouched).
- Product origin: daemon-served `/ui/` (`clients/pc/web/`). Vite preview is not
  the product origin. Canvas screenshots are never acceptance.
- Design system of record: the existing `clients/pc/web/src/tokens.css`
  (`--cp-*` namespace) and the seven-category state system in
  `clients/pc/web/src/state/stateMap.ts`. **This document does not create a
  second design system, a second token namespace, or a parallel canvas.** It
  expresses every rule against existing token names; the few additions it
  needs are listed once in §12 as *proposed* and are not applied to CSS by
  this slice.
- Upstream that wins on conflict: [AXIOMS](../../../docs/governance/AXIOMS.md)
  → product docs ([web-ui-design](../product/web-ui-design.md),
  [personal-2.0-scope](../product/personal-2.0-scope.md) §3.1–§3.6,
  [user-journeys](../product/user-journeys.md)) → core contracts →
  [v9 → daemon mapping](personal-2.0-opc-v9-implementation-mapping.md) →
  formal plan cards. Design corpus: [opc-2.0 README](../../../clients/docs/design/opc-2.0/README.md),
  [00 maintenance index](../../../clients/docs/design/opc-2.0/00-maintenance-index.md),
  [09 state / accessibility / visual system](../../../clients/docs/design/opc-2.0/09-state-accessibility-and-visual-system.md),
  [10 component map](../../../clients/docs/design/opc-2.0/10-component-map-and-prototype-flows.md),
  [11 design-to-code matrix](../../../clients/docs/design/opc-2.0/11-design-to-code-and-backend-matrix.md).
  Earlier Apple-led principles are reused, not restated:
  [legacy 09 Apple design principles](../../../clients/docs/design/legacy-control-plane-20260827/09-control-plane-apple-design-principles.md),
  [legacy 11 design system](../../../clients/docs/design/legacy-control-plane-20260827/11-control-plane-design-system.md),
  [legacy 22 state system](../../../clients/docs/design/legacy-control-plane-20260827/22-control-plane-state-system.md),
  [legacy 24 visual direction](../../../clients/docs/design/legacy-control-plane-20260827/24-control-plane-visual-direction.md).
- Companion: [v9 module-by-module comparison checklist](personal-2.0-opc-v9-ui-comparison-checklist.md)
  (the per-module judgement sheet D02 fills in).
- Lease: `lease/personal/P13-T12/visual-spec`. Claim ceiling `hypothesis`.
- Non-claims: nothing here proves rendered behaviour, contrast, NVDA
  announcement, 200% layout, Windows native chrome, usability, Gate, release,
  Profile, or `P11-T15` acceptance. NVDA / 200% / host-theme / State Lab nine
  states × nine surfaces stay **not-run** until `P13-T12/D02`.

---

## 0. How to use this document / 怎么用

1. Implementation windows (P13-T04..T11 `/ui/` slices) read §3–§11 before
   touching `app.css` or a view; every rule names the token it consumes.
2. `P13-T12/D02` reads §6 (layout cells), §7 (keyboard/focus cells), §9
   (nine × nine cells) and §5 (theme cells) as the definition of *pass*, then
   records each cell in the companion checklist on one exact `/ui/` revision.
3. A rule that cannot be met without new backend authority is **not** met by a
   disabled or decorative control; it is rendered as the `Requires-backend` /
   `Requires-environment` pattern of §9.10.
4. Contradictions between the frozen prototype and the product documents are
   recorded in §13 *Drift observed*; this document does not decide them.

Vocabulary: **Now** / **Current** = repository-established `/ui/` behaviour;
**2.0 target** = adopted design; **Requires-backend** / **Requires-environment**
as in the architecture README. `not-run` is never pass.

---

## 1. Scope and non-claims / 范围与非声明

**In scope.** Typography scale; spacing, grid and density; color roles for the
light, dark and high-contrast host themes with contrast targets; the locked
three-column shell and its narrow-window rule; 200% zoom behaviour; focus ring
and keyboard order; motion and reduced motion; the nine State Lab states
(`loading` / `empty` / `working` / `error` / `success` / `partial` / `blocked` /
`unknown` / `offline`) as visual patterns on the nine surfaces (`today` /
`create` / `projects` / `members` / `runs` / `outputs` / `hitl` / `knowledge` /
`settings`); component states; the proposed-token table.

**Out of scope (unchanged by this document).** The IA (Today / Projects /
Knowledge + bottom Settings; Team and Inbox are not L1; HITL only on the
Project canvas with a Today deep link; `state-lab` under Settings → Advanced,
hidden by default); the Dual Track hashes of the mapping §6.0; copy locale
(see §13); backend facts; any CSS or TSX edit; canvas regeneration.

---

## 2. Design stance / 设计立场

The product feeling is **calm, dense, precise, professional** ([09](../../../clients/docs/design/opc-2.0/09-state-accessibility-and-visual-system.md)
"Visual direction"; [web-ui-design §9](../product/web-ui-design.md)). The
Apple-led principles already adopted for the Control Plane
([legacy 09](../../../clients/docs/design/legacy-control-plane-20260827/09-control-plane-apple-design-principles.md))
apply verbatim; the judgeable consequences for `/ui/` are:

| Principle | Judgeable rule on `/ui/` |
|---|---|
| Purpose | A surface leads with goal → expected result → openable deliverable → acceptance/evidence → Owner decision → source/freshness → next action ([09](../../../clients/docs/design/opc-2.0/09-state-accessibility-and-visual-system.md), corpus rule 4). Status is second; configuration last. |
| Agency | One dominant action per task surface (`.cp-button--primary`); edit / narrow / deny / stop stay visible at consequential boundaries; a control exists only when the daemon backs it. |
| Responsibility | Secrets, bearer tokens, SecretRefs, prompts and raw Provider traffic never reach the DOM, URL, storage, chat, Vault, export, log or evidence. Unknown is never rendered as `0`, healthy, complete or retryable. |
| Familiarity | Same-looking things behave the same: one `StateDot` grammar, one `.cp-stateview` family, one `.cp-table` row grammar, one focus ring, on every surface. |
| Flexibility | Desktop-first; narrow windows scroll horizontally and stay operable; keyboard-complete; honours reduced motion / reduced transparency / more contrast / text size. Not a mobile product. |
| Simplicity, not minimalism | Density comes from removing decoration, never from shrinking type below the floors in §3 and §4. No card walls, KPI tiles, glass, gradients, purple "AI" accent, staff-card mosaics, fake command centre or marketing whitespace. |
| Craft | Hairlines over shadows; tabular numerals in data; stable column order; no layout shift on refresh; selection preserved across refresh; copy states what failed, where, what is retained, whether retry is safe, and the one next action. |
| Delight = calm confidence | Truth legibility and instant response. No "thinking" animation, no confidence percentage, no sparkle vocabulary. |

Provenance (`Native` / `Observed` / `Governed` / `Verified`) describes origin or
authority, never confidence or progress; it never borrows a state hue.

---

## 3. Typography / 字体与字号

Families and scale are the existing tokens; **no new scale is introduced**.

| Role on `/ui/` | Token(s) | Value (from `tokens.css`) | Where it appears |
|---|---|---|---|
| UI text | `--cp-font-text` | `-apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", system-ui, sans-serif` | everything by default; Windows renders Segoe UI |
| Authority data (ids, digests, cursors, exact counts, daemon enum words, HTTP paths) | `--cp-font-mono` | `"SF Mono", "Cascadia Mono", ui-monospace, …` | `.cp-mono`, `.cp-factgrid dd`, `.cp-chip`, `.cp-strip` |
| Space title (one per page: 今日 / 项目列表 / 项目详情 / 成员管理 / 运行管理 / 产出管理 / 知识 / 设置 / 创建项目 · ①–⑤) | `--cp-size-title1` · `--cp-leading-title` · `--cp-track-title` | 22 px / 1.15 / −0.02 em, weight 600 | `.cp-page-head h2` |
| Detail object name (Project name in 详情 header, Member name in 成员配置 header, decision packet title, current-initialisation title in ③) | `--cp-size-title2` | 17 px, weight 600 | `.cp-detail-title` |
| Section heading inside a work surface (岗位名单 / 这一环 / 将做什么 / 已验收产出 / 模型连接 …) | `--cp-size-headline` · `--cp-leading-headline` · `--cp-track-tight` | 13 px / 1.3 / −0.015 em, weight 600 | `.cp-section-title`, `.cp-stateview h3`, `.cp-rail h2` |
| Body / form text | `--cp-size-body` · `--cp-leading-body` | 13 px / 1.45, weight 400 | default |
| Dense rows (tables, lists, ledgers, run counts) | `--cp-size-row` · `--cp-leading-row` | 12.5 px / 1.3 | `.cp-table`, `.cp-factgrid`, `.cp-receipt`, `.cp-honesty` |
| Labels, state words, reasons, timestamps, captions | `--cp-size-label` · `--cp-track-label` | 11 px / +0.01 em, weight 500 (labels) or 400 (reasons) | `.cp-field > span`, `.cp-table th`, `.cp-reason`, `.cp-chip`, `.cp-strip` |
| Large numerals (Today counts, run counts) | `--cp-size-title2` + `font-variant-numeric: tabular-nums` | 17 px tabular | run-count strongs; v9 draws 20 px — the token scale wins (§13-b) |

Rules.

- Sentence case everywhere. Daemon uppercase enum words stay verbatim **in
  mono** (data, not furniture). No all-caps furniture; the only permitted
  uppercase-tracking chrome is a provenance chip.
- Layout is in `rem`/`em` so the user's Windows text-size setting scales it;
  at 200% zoom (§6.4) no floor below is reduced.
- Reading prose caps at `72ch` (`p { max-width: 72ch }`); tables, ledgers and
  the operating canvas are fluid and never centred in a marketing column.
- Long localised labels (zh-CN and en both occur in the corpus) wrap with
  `overflow-wrap: anywhere`; they never truncate a Project name, a state word
  or the primary action. Two-line clamps are allowed only for secondary
  submenu labels (v9 `.projects-submenu button > span`).
- Numbers in data columns are tabular. An unknown value is the word
  `unknown` / `说不清`, never `0`, `—` alone, or an empty cell.

---

## 4. Spacing, grid, density / 间距、网格、密度

4 pt base grid, existing tokens only: `--cp-space-1..10` = 4 / 8 / 12 / 16 /
20 / 24 / 32 / 40 px.

| Element | Spacing rule | v9 reference (for the reviewer; not pixel-binding) |
|---|---|---|
| Main region padding | `--cp-space-5` top, `--cp-space-6` sides, `--cp-space-10` bottom (`.cp-main`) | v9 `.main-content` 18 px |
| Work surface / panel (`.cp-panel`, `.cp-confirm`, `.cp-stateview`) | `--cp-space-3` × `--cp-space-4` inner padding; one hairline `--cp-line`; radius `--cp-radius-lg` (8 px ceiling) | v9 `.work-surface` 14 px / radius 7 |
| Vertical rhythm between surfaces on a page | `--cp-space-4` (`.cp-panel` margin) | v9 `.scene-stack` gap 14 |
| Section heading spacing | `--cp-space-5` above, `--cp-space-2` below (`.cp-section-title`) | v9 `.section-heading` bottom hairline + 10 px |
| Dense data rows | 28 px row floor (`density.row.compact`), 6 px × `--cp-space-2` cell padding (`.cp-table td`) | v9 `td` 9 × 8 |
| Object rows (member list, output list, Project rows) | 36 px row floor | v9 `.member-list button` |
| Form rows (Settings, wizard fields) | 40 px floor; field label above, `--cp-space-3` between fields (`.cp-field`) | v9 `.field` |
| Pointer targets | fine pointer: the row floors above; coarse pointer (`pointer: coarse`) or any control on the HITL canvas, the wizard nav, or the rail composer: ≥ 44 × 44 px | v9 uses 44 px everywhere; `/ui/` keeps compact desktop density and reaches 44 px on the consequential controls and under coarse pointers |
| Radius | `--cp-radius-sm` 4 (chips, digest cells) · `--cp-radius-md` 6 (buttons, inputs, nav items, panels inside panels) · `--cp-radius-lg` 8 (floating layers, stateview, panel) — **8 px is the ceiling**; pills (`999px`) only for status tags and filter chips (§12 proposed token) | v9 radius 6–7; tags pill |
| Elevation | `--cp-elevation-1` only on true floating layers (palette, dialogs, fixed inspector). Content surfaces are flat, separated by hairlines. | v9 has no shadow |

Density comes from hierarchy: calm chrome (strip + nav), dense content
(tables, ledgers), one disclosure deeper for runtime detail (`<details
class="cp-details">`, v9 `.trace-fold`). Nothing shrinks below the row floors.

---

## 5. Color roles for light / dark / high-contrast / 三种宿主主题的颜色角色

`/ui/` follows the host: `color-scheme` + `prefers-color-scheme` select the
light or dark block of `tokens.css`; `prefers-contrast: more` applies the
existing high-contrast overrides. There is one neutral field, one accent, and
state hue is spent **only** on state (dots, keylines, tag borders, thin
left rules) — never on large fills.

### 5.1 Role → token

| Role | Token | Light | Dark | Rule |
|---|---|---|---|---|
| App background | `--cp-canvas` | `#f5f6f8` | `#101318` | behind everything |
| Content surface (panels, tables, nav, rail) | `--cp-surface` | `#ffffff` | `#171b22` | flat, hairline-separated |
| Floating layer (palette, dialog) | `--cp-raised` | `#ffffff` | `#1e242e` | only with `--cp-elevation-1` |
| Primary text | `--cp-ink` | `#111827` | `#e8edf5` | ≥ 4.5:1 on canvas and surface |
| Secondary text (reasons, captions, timestamps, ledes) | `--cp-quiet` | `#5b6472` | `#9aa6b8` | measured ≥ 4.5:1 on canvas, surface and tints |
| Accent as fill (primary button, selection keyline, active nav rule, active dot) | `--cp-accent` + `--cp-accent-ink` | `#0a6cff` / `#ffffff` | `#7eb6ff` / `#0b1220` | text on accent ≥ 4.5:1 |
| Accent as text (links) | `--cp-link` | `#0b5ed7` | `#7eb6ff` | ≥ 4.5:1 on canvas, surface, tints; never `--cp-accent` as text |
| Selection / hover tint | `--cp-accent-soft` | 10 % accent | 14 % accent | with a 2 px accent keyline on selected rows/nav |
| Focus ring | `--cp-focus` | `#0a6cff` | `#7eb6ff` | ≥ 3:1 against every adjacent colour it is drawn over (§7) |
| Hairline / strong hairline | `--cp-line` / `--cp-line-strong` | 10 % / 18 % ink | 10 % / 20 % ink | separators; borders of panels and inputs |
| State hue (shape only) | `--cp-ready` `--cp-waiting` `--cp-attention` `--cp-blocked` `--cp-completed` `--cp-unknown` | see `tokens.css` | see `tokens.css` | dots, keylines, tag borders; ≥ 3:1 against the surface as non-text UI |
| State text | `--cp-waiting-text` `--cp-attention-text` `--cp-blocked-text` `--cp-unknown-text` | measured ≥ 4.5:1 | measured ≥ 4.5:1 | the label next to a dot; `ready` / `active` / `completed` labels stay `--cp-ink` |
| State tint (thin rule blocks: receipt, honesty note) | `--cp-ready-tint` … `--cp-unknown-tint` | 8–10 % | 12–14 % | never stacked on another tint (the digest-chip rule in `app.css`) |

### 5.2 Nine-state tone mapping (v9 `STATE_TONES` → `/ui/` categories)

v9 draws states with five tones (`neutral` / `good` / `warn` / `bad` / `info`).
`/ui/` has seven display categories. The binding mapping — one system, not
two — is:

| v9 `StateKey` | v9 tone | `/ui/` `StateCategory` (`stateMap.ts`) | Dot shape (`StateDot`) | State text token |
|---|---|---|---|---|
| `loading` | info | `unknown` (load state `loading`) | hollow circle | `--cp-unknown-text` |
| `empty` | neutral | `unknown` (load state `empty`) | hollow circle | `--cp-quiet` |
| `working` | info | `active` | pulse circle (static under reduced motion) | `--cp-ink` |
| `error` | bad | `blocked` | square | `--cp-blocked-text` |
| `success` | good | `ready` (receipt) or `completed` (evidence-linked) | filled circle / check | `--cp-ink` |
| `partial` | warn | `attention` | diamond | `--cp-attention-text` |
| `blocked` | bad | `blocked` | square | `--cp-blocked-text` |
| `unknown` | bad | `unknown` | hollow circle | `--cp-unknown-text` |
| `offline` | warn | `attention` (load state `stale`) | diamond | `--cp-attention-text` |

`info` therefore never becomes a new hue: loading/working reuse accent (active)
or unknown. `success` is never a green flood; completion is proven by the
evidence link, and `--cp-completed` is deliberately neutral.

### 5.3 Contrast targets (what D02 measures)

| Pair | Target | Themes |
|---|---|---|
| body / row / label text on canvas, surface, raised, every tint | ≥ 4.5:1 | light, dark, high-contrast |
| title1 / title2 (≥ 17 px, weight 600) | ≥ 3:1 (AA large) but the tokens are chosen to reach 4.5:1 | all |
| `--cp-accent-ink` on `--cp-accent` (primary button) | ≥ 4.5:1 | all |
| `--cp-link` on canvas / surface / `--cp-accent-soft` / `--cp-ready-tint` | ≥ 4.5:1; inside `.cp-receipt` the link is ink + underline (measured 4.29:1 otherwise) | all |
| focus ring against the control fill *and* the surface behind it | ≥ 3:1 | all |
| state dot / keyline / tag border against its surface | ≥ 3:1 (non-text UI) | all |
| disabled control text | no target; disabled controls are sequence gates only (§10.1) and carry a visible reason next to them | all |

### 5.4 High-contrast and forced colours

- `prefers-contrast: more` (already in `tokens.css`): `--cp-line` becomes
  `--cp-line-strong`; every state tint becomes `transparent` so a state block is
  border + text only. v9 adds the same rule (`border-color: var(--text)` on
  surfaces); on `/ui/` this is satisfied by the strong hairline.
- Windows High Contrast (`forced-colors: active`, Edge/Chromium on Windows):
  system colours replace the palette. Requirements: state is still legible
  because every state carries text + shape; focus uses the system
  `Highlight`; the primary button keeps a visible border (`ButtonBorder`);
  the pulse dot is static; no information is conveyed by background alone.
  `/ui/` has no `forced-colors` block today — recorded as §12 proposed and
  §13-g.
- Reduced transparency: `.cp-honesty` and `.cp-receipt` already fall back to
  `--cp-surface`; the palette scrim must fall back to a solid `--cp-raised`.

### 5.5 Dark theme specifics

Dark is not an inverted light theme: canvas → surface → raised are three
steps upward (`#101318` → `#171b22` → `#1e242e`); the accent lightens to
`#7eb6ff` with dark accent-ink; state hues are the lighter variants in
`tokens.css`; shadows become `0 1px 2px rgba(0,0,0,.5)` and remain floating-only.

---

## 6. Layout: locked three columns / 三栏锁定

### 6.1 Frame

```text
strip   (34 px, --cp-surface, mono labels)                    grid-area strip
side    (232 px; 200 px < 1440)      main (minmax(0,1fr))     rail (280 px; 240 px < 1440)
        Today / Projects (+ 详情/成员/运行/产出 submenu when a live Project is open) / Knowledge
        … Settings anchored at the bottom (`.cp-side-foot`), ⌘K button
        main = `#main` (tabindex -1): PageHeader → HonestyNote → surfaces
        rail = `AssistantRail` (Personal Assistant outside a Project; Project group inside one)
```

Rules (product truth: [scope §3.1](../product/personal-2.0-scope.md),
[web-ui-design §4](../product/web-ui-design.md), [09 Windows window behavior](../../../clients/docs/design/opc-2.0/09-state-accessibility-and-visual-system.md)):

1. The three columns are **locked**. The rail is always the third column; it
   is never an overlay, drawer, sheet or "open conversation" control.
2. The rail is hidden — leaving two columns — for empty Home (no Project);
   v9 hides it **only** there. `/ui/` today also hides it on the create wizard
   `#/projects/new` and on creating-only Today (P12-T02 / P12-T05); scope §3.1
   says the create ring's right chat defaults to the Personal Assistant. The
   two-column set is therefore a Now-vs-target gap recorded in §13-i, not a
   rule of this specification.
3. Settings sits at the bottom of the side column; Team, Inbox and a
   standalone `#/hitl` are not navigation.
4. Current Project, conversation identity, state word and primary action each
   stay inside their own column at every width.

### 6.2 Widths and the shell minimum

| Viewport width | Side | Rail | Main | Behaviour |
|---|---|---|---|---|
| ≥ 1440 px | 232 px | 280 px | `minmax(0,1fr)`, max 1680 px content | reference desktop |
| 1100–1439 px | 200 px | 240 px | fluid | `.cp-inspector` may float over the rail edge as a fixed layer with focus return |
| < 1100 px (proposed `--cp-shell-min-width: 1100px`, v9 `.shell { min-width: 1100px }`) | 200 px | 240 px | main keeps `minmax(576px, 1fr)` (v9) | **the page scrolls horizontally**; the grid does not reflow; columns never stack |

The narrow-window rule is therefore: the app root (`.cp-app`) gets
`overflow-x: auto` and `min-width: var(--cp-shell-min-width)`; the strip
stays visible; sticky actions never cover a focused field or an error. **The
current `app.css` `@media (max-width: 1279px)` block that stacks strip / side
/ main / rail and turns the nav into a top tab strip is non-conforming to this
rule** and to the product documents; it is recorded in §13-a. D02 records the
narrow cells against this specification, not against the current CSS.

### 6.3 Inside the main column

- Page header = one `title1` + one quiet lede; ledes state what the surface is
  *not* (not Home, not an Inbox, not a KPI wall) only where the product docs
  require the honesty.
- Live Project pages show the four-destination `ProjectWorkNav`
  (详情 / 成员 / 运行 / 产出) as a `.cp-sectionnav` under the header, and the
  same four items appear as the Projects submenu in the side column (v9
  `.projects-submenu`). Both render the current destination with
  `aria-current` and the accent keyline.
- Master–detail surfaces (成员, 产出) use `.cp-master` + selected detail:
  unselected = empty detail (never default-first); switching Project clears
  the selection.
- Process axis (② / ④ / 详情 / 运行): a horizontal `role="list"` of stage
  nodes with `grid-auto-flow: column`, `overflow-x: auto` inside the panel,
  `aria-current="step"` on the current stage, `data-mark="auth|verify"`
  rendered as a `--cp-waiting` / accent keyline. The axis scrolls inside its
  panel; it never forces the shell wider.

### 6.4 200 % zoom

At 200 % browser zoom on a 1440 × 900 window the CSS viewport is 720 × 450 px,
i.e. below the shell minimum. Required behaviour:

1. The shell keeps its columns and the page scrolls horizontally (§6.2); the
   strip, side and rail keep their widths; nothing stacks.
2. Every text container has `min-width: 0` and `overflow-wrap: anywhere`; no
   text is clipped or overlaps; no fixed-height container hides content
   (`.cp-palette` uses `max-height` + scroll).
3. Type floors (§3) and target floors (§4) are unchanged; the focus ring is
   fully visible (not clipped by `overflow: hidden` parents — the wizard
   viewport must give the ring room or use `outline-offset` inside).
4. The dialog (`role="dialog"`) fits the CSS viewport with internal scroll and
   its primary action remains reachable by keyboard.
5. No horizontal scroll *inside* the main column except the process axis and
   wide tables, which scroll within their own panel.

---

## 7. Focus ring and keyboard order / 焦点环与键盘顺序

### 7.1 Focus ring

- One ring for everything: `:focus-visible { outline: 2px solid var(--cp-focus);
  outline-offset: 2px; border-radius: var(--cp-radius-sm); }` (existing). v9
  draws 3 px; the token rule wins (§13-c); a `--cp-focus-width` token is
  proposed (§12) so D02 can measure one value.
- The ring must reach ≥ 3:1 against the control and the surface behind it in
  all three themes (§5.3) and must not be clipped by `overflow: hidden`
  ancestors, the fixed inspector or the strip.
- Focus is never removed on pointer interaction; selection tint and focus
  ring may coexist (row `aria-selected` tint + ring).
- Hover-only, colour-only, motion-only or drag-only essential actions are
  forbidden; every hover enhancement has a focus/keyboard equivalent.

### 7.2 Landmarks and order

Tab order follows DOM order, which is the visual order:

1. skip link (`a.skip` → focuses `#main`);
2. status strip cells (`.cp-strip-cell`, buttons/links only);
3. side column: brand (not focusable) → `nav[aria-label="Primary"]` Today →
   Projects → (submenu 详情 → 成员 → 运行 → 产出 when open) → Knowledge →
   `.cp-side-foot` Settings → ⌘K button;
4. main `#main` (tabindex −1; receives focus on route change and via the skip
   link) → page content in reading order;
5. rail (`aside`): thread → composer textarea → mention buttons → send /
   review controls.

Landmarks: `banner`-like strip, `navigation` (side), one `main`, one
`complementary` (rail, labelled with the conversation identity). Each page has
exactly one `h2` space title (`h1` is the brand). Tables carry a `caption`.

### 7.3 Route change and focus restoration

- On hash change the shell moves focus to `#main` and the page title is
  announced; the side column keeps its scroll position and `aria-current`.
- Opening a contextual inspector or dialog moves focus into it; `Escape` and
  every close control return focus to the trigger (v9 dialogs, `CommandPalette`).
- Lists preserve filter, sort, selection and scroll across a refresh; a
  selection whose row disappears is dropped, not re-pointed.
- Loading and error never erase form input, drafts or last-known facts; after
  an error the first invalid field receives focus and the error summary is
  announced (`role="alert"`).

### 7.4 Widget patterns (as in v9, kept on `/ui/`)

| Widget | Pattern | Keys |
|---|---|---|
| Wizard step dots (①), member-init order (③) | `role="tablist"` / `role="tab"` with `aria-selected`; unreachable steps `disabled` with a visible reason; hidden slides `inert` + `aria-hidden` | Tab into the selected tab; Arrow ← → between reachable tabs; Enter/Space activates |
| Segmented controls (period toggle, Knowledge tabs, member config tabs) | `role="tablist"` when they switch panels (roving `tabindex`), else `role="group"` + `aria-pressed` | Arrow ← → (tablist); Tab across (group) |
| Member list / output list | `role="listbox"` + `role="option"` + `aria-selected`; nothing selected by default | Arrow ↑ ↓, Home/End, Enter selects |
| Process axis | `role="list"` of buttons with `aria-current="step"` | Tab across stages; Enter opens the stage |
| Tables | `<table class="cp-table">` with `caption`, `th[scope]`, selectable rows `aria-selected`; wide tables scroll inside a `tabindex="0"` wrapper labelled by the caption | Tab to the wrapper, arrows scroll |
| Dialogs (edit-confirm, roster, runtime, rail review, palette) | `role="dialog"` + `aria-modal` + `aria-labelledby` / `aria-describedby`; focus trapped; scrim solid under reduced transparency | Esc cancels and restores; Enter on the primary confirms |
| Disclosure (`.cp-details`, v9 `.trace-fold`) | `<details>/<summary>`; collapsed by default for process traces, advanced diagnostics and `state-lab` | Enter/Space toggles |
| Composer | `textarea` labelled; Enter sends, Shift+Enter newline; `@` suggestions keyboard-reachable and inserted only into the unsent draft | Enter / Shift+Enter / Arrow in suggestions / Esc closes |
| Canvas fields (`SyncedField`) | `input`/`textarea` with `aria-label` naming the field and the Enter-to-notify behaviour; Enter opens the confirm dialog, nothing is written until confirmed | Enter / Shift+Enter |
| Live regions | `aria-live="polite"` for progress, state banners, "written" receipts; `role="alert"` for errors; streaming messages never steal focus or flood announcements | — |

### 7.5 NVDA key paths (defined here, executed by D02)

For each of the nine surfaces the key path is: land on the page (Ctrl+Home) →
H to the space title → Tab through the primary action → D/landmark to the rail
→ read the composer label → Esc/back. The HITL canvas adds: read the
preview facts list, the checkbox label (本周此类不再问), the four actions and
their disabled reasons; the wizard adds the tablist and the status live
region; Settings adds the password field ("does not echo").

---

## 8. Motion and reduced motion / 动效与减少动效

| Token | Use on `/ui/` |
|---|---|
| `--cp-motion-instant` 80 ms | pointer-down press (`.cp-button:active`), row selection |
| `--cp-motion-fast` 140 ms | hover fills, disclosure chevrons, nav tint |
| `--cp-motion-enter` 200 ms | route/section entry (opacity + ≤ 4 px translate), dialog and palette enter (same path on exit, anchored to the trigger); the wizard rail slide (v9 280 ms) uses this token |
| `cp-pulse` 2.4 s | the `active` dot only |

Rules: feedback starts on pointer-down and is interruptible; routine
navigation never bounces; no layout-shifting animation, animated numbers,
skeleton shimmer loops or parallax; motion never carries information that is
not also static. **No animation delays approvals, Stop, reconciliation,
validation or recovery.** Long-running work shows durable facts (current
step, elapsed basis, responsible Member), never a decorative "thinking"
animation.

`prefers-reduced-motion: reduce` (existing in `app.css` and v9): pulse static,
palette/dialog appear without animation, nav/table/button transitions off,
wizard rail jumps (no slide), `scroll-behavior: auto`. Generated runtime
samples appear at once instead of slot by slot (v9 honours this).

---

## 9. Nine states × nine surfaces / 九态 × 九表面

`StateKey` and `SurfaceKey` are the v9 State Lab axes ([mapping §6.4](personal-2.0-opc-v9-implementation-mapping.md));
`state-lab` itself is Settings → Advanced, hidden by default, never L1.
Every product surface must be able to show each of the nine states with the
**real** layout (no static mock), using the patterns below. Each pattern names
its component (`clients/pc/web/src/components/states.tsx`,
`state/StateDot.tsx`, `state/HonestyNote.tsx`, `components/ReceiptLine.tsx`)
and what its copy must say (from v9 `stateMessage` and the [09 state grammar](../../../clients/docs/design/opc-2.0/09-state-accessibility-and-visual-system.md)).

### 9.1 State patterns

| State | Pattern / component | Must communicate | Allowed controls | Forbidden |
|---|---|---|---|---|
| `loading` | `LoadingState` (`.cp-stateview`, `role="status"`, hollow dot) shown **beside** the last safe projection, never replacing input or drafts | exact source being read; that the last projection stays visible; leaving keeps the draft | cancel only if the read is really cancellable | spinner-only; skeleton shimmer; progress %; erasing facts |
| `empty` | `EmptyState` (`.cp-stateview` dashed border, one `h3`, one paragraph, one `.cp-next` primary action) | why no object exists (no admitted object; no demo rows) and the one first-value action | exactly one primary action (`.cp-button--primary`) | KPI tile; sample/demo rows; two competing primaries |
| `working` | `StateDot category="active"` + verbatim daemon state word in a `.cp-chip`; durable facts in a `.cp-factgrid`; "进行中不是完成 / Working is not completion" | output contract, durable step, responsible Member, artifacts so far, real controls | Stop / pause only when the daemon backs them | fake success; percentage; process exit as completion |
| `error` | `ErrorState` (`.cp-stateview`, `role="alert"`, blocked square) | what failed, where, impact, retained input/work, whether retry is safe, one next action | retry only when `retryable`; edit; escalate | blank page; toast-only; wiping form input |
| `success` | `ReceiptLine` (`.cp-receipt`, ready rule + tint, ink link) or `completed` check when evidence-linked | changed object, receipt/evidence digest, freshness, next valuable action | open evidence; next action | green flood; "done" without evidence; toast without receipt |
| `partial` | `attention` diamond + `.cp-honesty` coverage note | which facts are available, which source/facet is missing, coverage; the gap is not "ready" | continue with the available part | rendering the gap as ready; hiding the missing facet |
| `blocked` | blocked square + `.cp-honesty`/`ErrorState` naming the dependency (permission, input, dependency) | that done work is safe; exactly what is blocking; where to go (deep link) | go to the Project / Settings; narrow scope | fake Confirm; blind retry; disabled buttons standing in for the explanation |
| `unknown` | `UnavailableState` / unknown hollow dot + `.cp-honesty` | "unknown is not 0, not success"; why a conclusion is unavailable; that retry is blocked until reconciliation | view retained work; reconcile when backed | `0`; success; healthy; blind retry |
| `offline` | `attention` diamond with `stale` word + last-known time; content remains visible and labelled stale | host/network state; last-known facts and their time; retained work; that external actions cannot run | read last facts; edit local drafts (add-member duty, process text) | external approve/send/import/handoff; "24/7 cloud will catch up" |

`Requires-backend` / `Requires-environment` (§9.10) is the tenth, cross-cutting
pattern: an explanatory `.cp-honesty` block with the dependency label and no
hover/pressed styling — never a disabled or decorative button.

### 9.2 Surface × state grid (what each cell must show)

Each row is one `SurfaceKey`; each cell names the host element (D02 selector),
then the state-specific expectation. Primary actions come from v9
`SURFACE_CONTEXT.firstAction`.

| Surface (v9 / `/ui/` host) | loading | empty | working | error | success | partial | blocked | unknown | offline |
|---|---|---|---|---|---|---|---|---|---|
| `today` — `#/` `[data-page=opc-today]` | LoadingState beside last Project list; decision packet still clickable | only-create (empty Home, rail hidden, one "Start create"); creating-only = "Continue create", no packet | live rows with `active` dot; packet collapsed when nothing pending | ErrorState: list read failed; retained facts | receipt in packet after canvas confirm (deep-linked) | rows present, packet source missing → coverage note | "stopped on a dependency; go to that Project's runs" deep link | counts read `unknown`, never `0`; no blind refresh loop | rows labelled stale + time; "去处理这一件拍板" disabled with reason (external) |
| `create` — `#/projects/new` `[data-page=opc-create-wizard]` | step content loading beside the draft; draft never erased | ① with nothing confirmed: "由你填写" tag, no suggestions yet | ③ generating runtime slot by slot with `<progress>` + live region; ④/⑤ running "进行中不是完成" | ④ 不通过 / ⑤ 失败 → back to the named stage; input retained | ⑤ 核对通过 → "验收，进入今日" is the **only** success | ③ some Members seated; unseated = pending, not ready | ④ owner not seated → cannot start; ① Provider unbound → "去设置" | ④/⑤ 说不清 → cannot pass, cannot accept | ④/⑤ cannot start test / joint; ①–③ text edits allowed |
| `projects` — `#/projects` `[data-page=opc-projects]` | list loading beside last list | "no Projects; create starts from Today" (no create button here) | live Project row with state word | list read failed | copied → inactive 副本 banner (not activated) | some rows lack detail → row-level unknown cells | Project blocked → row state `blocked` + deep link to runs | count/cost `unknown` | rows stale + time |
| `members` — `#/projects/:id/members` `[data-page=opc-project-members]` | roster loading beside last roster | live Project with no seat: "加人" primary; non-live: honest empty, no demo members | seat request in flight (`seat.request` → `seat.confirm`) | roster read failed | member joined → receipt `[data-region=opc-join-written]` | some tabs' facts missing → coverage note | no model bound → `pending` tag + "去设置" | seat status unknown ≠ seated | roster stale; duty text editable, join not sendable |
| `runs` — `#/projects/:id/runs` `[data-page=opc-project-runs]` | axis loading beside last axis | non-live Project: honest empty "未上线，没有今日执行"; counts never `0` when unknown | current stage `active`; "当前步骤"; counts from daemon | axis read failed | "验收回今日" receipt only on the last ring | occurrence ledger partial → coverage note (P13-T05) | stage `mark=auth|verify` → "去授权预览 / 去核对" | counts `unknown`, no completion claim | stale ledger + time |
| `outputs` — `#/projects/:id/outputs` `[data-page=opc-project-outputs]` | list loading beside last list | "还没有可打开的成果" → back to runs / continue create | "正在编排… 进行中不是完成" | outputs read failed | selected output opens (P13-T04) with evidence | draft exists, not accepted → coverage note | needs HITL → canvas deep link | "说不清是否可打开" → back to runs | stale list; open disabled with reason |
| `hitl` — `#/projects/:id?preview=` `[data-region=opc-hitl-actions]` | preview loading; actions disabled with reason "loading" | no pending preview → "chat only links; nothing to approve" | executing → fourth action **Stop** visible | preview read failed; nothing approved | approved → receipt pinned to the stage `[data-region=opc-hitl-written]` | preview facts partial → cannot confirm | narrowed → old preview void, needs new preview | preview validity `unknown` → cannot approve (not stale, not success) | offline → cannot approve external |
| `knowledge` — `#/knowledge` `[data-page=opc-knowledge]` | index loading beside last index | no Project → locked; live Project with no files → "导入资料" primary | import running (`importing`) | parse-fail: original kept, retry safe | indexed receipt; forget → tombstone (never revived) | files listed, some unindexed → coverage | secret detected → route to SecretStore, nothing enters the Vault | count `unknown` ≠ 0 | read last index; import not startable |
| `settings` — `#/settings` `[data-page=opc-settings]` | connections loading beside last table | no connection: template dropdown + key field, no fake Connect | handoff in flight; key field cleared on completion | failed: named cause ("SecretStore unavailable"), not "connected" | connected (source-labelled), usage actual/estimated | usage partial → labelled | policy revoke blocked → reason | usage/quota `unknown` ≠ 0 | handoff not startable; skip-week revoke allowed |

### 9.3 State copy grammar

Every state message answers, in this order: **what** (surface + object), **what
is retained**, **what you can do**, **how this surface exposes it** (v9
`StateBanner` `dl`: 你还剩什么 / 你可以做什么 / 这一屏怎么露). Copy never
says "thinking", "AI is working", a confidence figure, or a completion word
without evidence.

### 9.4–9.9 Surface-specific visual notes

- **Today.** Decision packet is a `.cp-region` (hairline, `--cp-radius-lg`),
  title2 question, four `dt/dd` facts (可逆性 / 备选 / 费用 / 为何先 A) in a
  four-column `.cp-factgrid`, one primary "去处理这一件拍板", one text action
  "以后再说". Run counts are three quiet cells (created / live / blocked) with
  title2 tabular numerals and a caption "样品 / unknown ≠ 0"; the period toggle
  is a `role="group"` segmented control. No KPI wall, no four swimlanes.
- **Create.** Setup header with title1 and a quiet paragraph; the confirm
  list is a slide rail with step dots; one primary per step (确认本项 /
  全员就位，进入测试 / 验收，进入今日); "离开并保留草稿" is secondary; a
  `Requires-backend` honesty block closes each step.
- **Projects.** One `.cp-panel` per Project: title2 name, quiet meta,
  three-row `.cp-factgrid` (目标 / 周期 / 费用), then one primary 打开 and
  text links 成员 / 运行 / 产出 (never four equal "查看" buttons).
- **Members / add-member / member-config.** `.cp-master` listbox left,
  detail right; identity (model, seated, stage) stays in the detail header;
  eight tabs in the product order; input tab read-only; `pending` tag when no
  model. No Install button anywhere.
- **Runs / outputs.** Run counts strip, process axis, stage detail with
  `.cp-factgrid` ledger, collapsed process-trace disclosure, acceptance only
  on the last ring. Outputs: listbox left, composition right, "请助手换一种展示"
  secondary; HITL-needing output deep-links to the canvas.
- **HITL canvas.** `.cp-confirm` block: 将做什么 → 完整预览 / 差异 `dl` →
  checkbox 本周此类不再问 → actions row 批准 (primary, enabled only when
  pending ∧ fresh ∧ not executing ∧ not narrowed) / 改窄 / 拒绝 / 停 (only
  while executing). Stale / unknown notices are `.cp-honesty` blocks with the
  blocked square.
- **Knowledge.** Segmented tabs 项目资料 / 导入 / 为什么用这段 / 记忆; filters
  (scope, kind, keyword) as `.cp-filters`; result list with kind tags; Why
  table with 片段 / 为何选中 / 新鲜度; memory record with 忘记这条.
- **Settings.** Grouped panels: 模型连接 (template `select`, custom fields,
  password field with "does not echo" caption, one primary 交接密钥), 本周不再问
  (收回跳过), 通知与恢复, Advanced `<details>` (diagnostics, `state-lab`
  hidden by default).

### 9.10 Requires-backend / Requires-environment pattern

`.cp-honesty` (unknown left rule + tint, row size, quiet text) with the label
`Requires-backend` or `Requires-environment`, the missing capability in one
sentence, and — when one exists — the CLI or route that does work. It has no
hover or pressed styling, is not a button, and is never rendered as a disabled
Connect / Install / Approve / Confirm / Publish / Activate control.

---

## 10. Component states / 组件状态

### 10.1 Buttons

| Variant | Default | Hover | Active | Focus | Disabled |
|---|---|---|---|---|---|
| `.cp-button--primary` (one per surface) | `--cp-accent` fill, `--cp-accent-ink` text, weight 600, `--cp-radius-md` | fill darkened via `filter`/tint (no colour change that breaks 4.5:1) | `--cp-motion-instant` press | ring §7.1 | **only for sequence gates** (confirm before next, seat before test, fresh before approve); the reason is visible text next to the button; `opacity .56` |
| `.cp-button` (secondary) | `--cp-surface` fill, `--cp-line-strong` border, ink | `--cp-accent-soft` | press | ring | as above |
| text link / text button | `--cp-link` text, no fill | underline | — | ring | never disabled; absent instead |
| `.cp-button--danger` (拒绝 / 忘记 / 删除 second confirm) | ink text, `--cp-blocked` border | `--cp-blocked-tint` | press | ring | as above |

Capability gaps are never disabled buttons (§9.10). A disabled button always
has an adjacent visible reason (v9 `.flow-end` / `.wizard-status` live text).

### 10.2 Tags and chips

`.cp-chip` (mono label + `StateDot`) for daemon state words; status tags with
pill radius and a tone border (`--cp-ready|waiting|attention|blocked|unknown`)
and ink text — colour is on the border, the word carries the meaning.
Provenance chip: 1 px `--cp-line-strong` border, 11 px, uppercase-tracked,
`proposed` = waiting border, `governed` = accent border, `verified` = ready
border.

### 10.3 Navigation

Side nav item: 6 px × `--cp-space-2` padding, `--cp-radius-md`; hover
`--cp-accent-soft`; current = `--cp-accent-soft` fill + 2 px `--cp-accent`
left rule + weight 600. Submenu items indent under Projects with a
`--cp-line` left rule. Count badge (Today pending decisions) is a mono
`--cp-attention` numeral, never a red dot alone.

### 10.4 Lists, tables, rows

Rows: hairline bottom border; hover `--cp-accent-soft`; selected =
`--cp-accent-soft` + inset 2 px accent rule; a tint inside a tinted row is
dropped (digest chip rule). Empty table body is replaced by `EmptyState`, not
by an empty `<tbody>`. Wide tables scroll inside their panel.

### 10.5 Fields and forms

Label above (`--cp-size-label`, quiet), control 40 px floor, `--cp-line-strong`
border, `--cp-radius-md`, `--cp-surface` fill; invalid = `--cp-blocked` border +
`aria-describedby` error line in `--cp-blocked-text`; the password field never
echoes and clears after handoff; constraints are stated before entry; values
survive errors and route changes.

### 10.6 Dialogs and floating layers

`--cp-raised`, `--cp-radius-lg`, `--cp-elevation-1`, 1 px `--cp-line`; scrim
`rgba(16,19,24,.28)` (solid `--cp-raised` under reduced transparency); enter via
`--cp-motion-enter`; width `min(40rem, calc(100vw − 2rem))`; internal scroll.

### 10.7 Progress and live status

`<progress>` with `accent-color: var(--cp-accent)`, 8 px tall, always paired
with a text "n / m" and an `aria-label`; never an indeterminate percentage.
Live regions are polite except errors.

### 10.8 Disclosure

`.cp-details` summary 44 px hit area, quiet 12 px label; body quiet 13 px;
collapsed by default for traces, diagnostics and `state-lab`.

---

## 11. Density and disclosure order / 密度与披露顺序

1. Business language first; runtime terms (prompt, Skill, MCP, loop, engine
   name) one disclosure deeper; engine identity only in fault resolution or
   Advanced diagnostics.
2. Calm chrome, dense content: the strip and side column never gain badges,
   counters or decoration beyond the Today decision count and the Knowledge
   lock tag.
3. Nothing moves that did not change: stable column order, stable row order,
   selection preserved across refresh.
4. Reading text ≤ 72ch; data fluid; no centred marketing column.

---

## 12. Proposed tokens (not applied by this slice) / 建议新增 token（本切片不改 CSS）

| Proposed token | Value | Why | Status |
|---|---|---|---|
| `--cp-shell-min-width` | `1100px` | makes the narrow-window horizontal-scroll rule (§6.2) a single measurable number; v9 `.shell { min-width: 1100px }` | **proposed** |
| `--cp-layout-side` / `--cp-layout-side-narrow` | `232px` / `200px` | name the two side-column bands already hard-coded in `app.css` | **proposed** |
| `--cp-layout-rail` / `--cp-layout-rail-narrow` | `280px` / `240px` | name the two rail bands already hard-coded | **proposed** |
| `--cp-layout-main-min` | `576px` | main column floor inside the locked grid (v9) | **proposed** |
| `--cp-focus-width` | `2px` | one measurable ring width (v9 draws 3 px; spec keeps 2 px) | **proposed** |
| `--cp-target-min` | `44px` | consequential-control and coarse-pointer target floor (§4) | **proposed** |
| `--cp-radius-pill` | `999px` | status tags / filter chips only | **proposed** |
| `forced-colors: active` block | system colours (`CanvasText`, `Highlight`, `ButtonBorder`), static pulse | Windows High Contrast (§5.4) | **proposed** |

No new hue, no new type size, no new spacing step is proposed. The
`StateCategory` set stays at seven; the nine v9 `StateKey`s map onto it (§5.2).

---

## 13. Drift observed (recorded, not decided) / 观察到的漂移（只记录，不裁决）

| # | Observation | Sources | Resolution rule applied here |
|---|---|---|---|
| a | `app.css` `@media (max-width: 1279px)` stacks strip / side / main / rail and turns the nav into a top tab strip; the legacy design system ([legacy 11 §7](../../../clients/docs/design/legacy-control-plane-20260827/11-control-plane-design-system.md)) also allows stacking below 1280 px. Product docs, scope §3.1, 09 and v9 require **no stacking, horizontal scroll**. | `app.css` 1098–1145; legacy 11 §7; scope §3.1; 09 | Upstream product docs win: §6.2 rule is no-stack. Current CSS is non-conforming; D02 will record the narrow / 200% cells against §6.2. No P13 card currently owns the CSS change — flagged in the running report. |
| b | v9 base type is 14 px / h3 15 px / numerals 20 px; `tokens.css` is 13 px body / 13 px headline / 17 px title2. | v9 CSS; `tokens.css` | Tokens are the design system of record; the prototype is not a pixel replica (Phase 13 boundary). |
| c | v9 focus ring 3 px; `app.css` 2 px. | v9 `:focus-visible`; `app.css` 49–53 | Token rule (2 px) kept; `--cp-focus-width` proposed for one measurable value. |
| d | v9 shell bands 176 / minmax(576,1fr) / 348 at `min-width: 1100`; `app.css` 232 / fluid / 280 (200 / 240 below 1440). | v9 `.shell`; `app.css` 68–74, 1082 | `app.css` bands kept (legacy 11 §3 sidebar 232, inspector 280–400); only the 1100 px minimum and the 576 px main floor are adopted (§12). |
| e | [web-ui-design §4](../product/web-ui-design.md) lists design routes (`/today`, `/projects/:id/setup`, `/settings/model-connections` …); `/ui/` hashes are `#/`, `#/projects/new`, `#/settings` … | web-ui-design §4; mapping §6.0 | web-ui-design already declares them "design routes, not SPA claims"; the mapping §6.0 hashes are the checklist's route column. |
| f | v9 uses nine `StateKey`s with five tones; `/ui/` has seven `StateCategory`s. | v9 `STATE_TONES`; `stateMap.ts` | Mapped in §5.2; no second state system. |
| g | `tokens.css` handles `prefers-contrast: more` but not `forced-colors: active` (Windows High Contrast). | `tokens.css` 131–140 | §5.4 requirement; token block proposed in §12; D02 records the high-contrast host-theme cell against §5.4. |
| h | Copy locale: product docs and v9 name surfaces and tabs in Chinese (今日 / 项目 / 知识 / 设置; 职责 / 输入 / 输出 / 技能 / 工具 / 工作说明 / 周期与触发 / 连接与权限); `/ui/` renders English (`Today`, `Duty`, `Brief`, `Loop`, `Perms`). | scope §3.1; `memberTabs.ts`; `PrimaryNav.tsx` | The product-doc terms are the semantic authority for label meaning and order; whether 2.0.0 ships zh-CN, en, or both is a product-semantic decision **not** taken here. The checklist judges order and meaning, not language. |
| i | v9 keeps the Assistant rail on the create scenes (①–⑤ default to Personal Assistant); `/ui/` hides the rail on `#/projects/new` (P12-T02) and on creating-only Today (P12-T05). | v9 `chatHidden`; `AppShell.tsx` `hideAssistantRail`; scope §3.1 "create/members/test/joint right chat defaults to Personal Assistant" | Scope §3.1 says the create-ring chat defaults to the Assistant, i.e. the rail exists during create. Recorded as a Now-vs-target gap for the checklist (`M-CHAT-CANVAS`, `M-CREATE-*`); the spec §6.1 lists the current hide set as the Now behaviour and the product-doc rule as the target. Not decided here. |
| j | v9 has a standalone `hitl` scene; `/ui/` renders HITL as the `?preview=` canvas on `#/projects/:id` with a Today deep link. | v9 `SCENES`; mapping Owner decision #4 | Owner decision #4 wins (already resolved in the mapping); the checklist row `M-HITL` uses the canvas route. |
| k | v9 draws 44 px targets on every control; `app.css` keeps compact desktop rows and raises to 44 px only under `pointer: coarse`. | v9 CSS; `app.css` 1275–1285 | §4 rule: 44 px on consequential controls (HITL actions, wizard nav, composer) and under coarse pointers; compact elsewhere. D02 measures the consequential set. |
| l | The task brief cites `clients/docs/design/09|11|24-*.md`; on `main` those files are tracked only under `clients/docs/design/legacy-control-plane-20260827/`. | `git ls-files` | Links point at the tracked legacy paths; no untracked local file is linked. |

---

## 14. Acceptance of this document (D01 closing gate) / 本文的关闭门

`P13-T12/D01` is closed when this specification and the
[companion checklist](personal-2.0-opc-v9-ui-comparison-checklist.md) are
written, every relative link resolves to a Git-tracked file, `check:consistency`
/ `check:handbook` / generator `--check` / `check:rules` / `git diff --check`
pass, and required CI is green on the merged head. It creates no rendered,
contrast, NVDA, 200%, Windows, Gate, release or T15 claim; those are
`P13-T12/D02`, `P13-T13` and `P11-T15`.

End of specification. Informative only. Canvas v9 ≠ product. `/ui/` chrome ≠
Gate / release. Authority remains the daemon.
