# 23 — Component Taxonomy & Core Component Specs

- Status: adopted Personal 2.0 component specification; no implementation claim
- Updated: 2026-08-27
- Inputs: state language `22`; tokens `11`; shell `12`; page specs 13–21
- Boundary: component technology remains an implementation decision

## Personal 2.0 component amendment

The component taxonomy expands around the adopted desktop model:

| Area | Required target components |
|---|---|
| Shell | ThreeRegionShell, GlobalAgentShell, ShellDock, ContextAnchor, CommandPalette |
| Agents | AgentMaster, AgentWorkspace, ConversationTranscript, Composer, AdapterCapabilityMatrix, NativeSlot, InstallConnectFlow, RuntimeDossier, DisconnectUninstallReview |
| Work | GoalMaster, PlanRevisionList, TaskAttemptTree, ManagePreview, OrchestrationMap, ContextInspector |
| Library | LibraryFamilyIndex, FamilyMasterDetail, McpServerInspector, FederatedSourceCompare, ConflictResolutionPreview, WritebackReceipt |
| Activity | ProvenanceTimeline, ProvenanceMarker, SourceCoverage, TimelineInspector |
| Settings | SettingsGroups, AccountHub, AcquisitionTierPicker, ConsentReview, CredentialPresence, ModelQuotaCost, SystemReadiness |
| Honesty | RequiresBackendBlock, CurrentImplementationNote, TargetDeltaNote, ProgressFact, CoverageBanner |

### Behavioral contracts

- **ConversationTranscript:** source-ordered messages, explicit Agent/vendor
  identity, bounded rendering, attachment refs, loading/sync/error states,
  keyboard navigation and no hidden-reasoning presentation.
- **ManagePreview:** preserves conversation, names proposed durable objects,
  Context/resource losses and participating Agents; submits only a real daemon
  preview/admission.
- **AdapterCapabilityMatrix:** each row renders three separate fields:
  runtime condition (`Supported|Unsupported|Unavailable|Unknown`), delivery
  status (`Now|Requires-backend`), and support path
  (`vendor-native|managed-adapter|MCP-cooperative|observable-only|unqualified`),
  plus source, freshness, reason, and optional `Requires-core` dependency.
- **NativeSlot:** bounded display/artifact renderer owned and versioned by the
  Adapter. It accepts display-safe data only; it cannot inject actions,
  executable markup/scripts, credentials, or authority-shaped state. Any
  vendor-specific action is rendered by a Control Plane-owned component tied to
  typed capability semantics.
- **TaskAttemptTree:** hierarchy and dependency state without fake percent
  progress; selection opens provenance timeline and inspector.
- **ProvenanceTimeline:** keeps Native/Observed/Governed/Verified distinct while
  time-aligning them; never claims a global order unsupported by sources.
- **McpServerInspector:** distinct server/package/connection/capability/binding/
  health/quarantine identities; source, trust, transport, revision/freshness,
  advertised Tool/Context/Skill candidates and admitted mappings; no
  host-session-control affordance.
- **FederatedSourceCompare:** both revisions, provenance and conflict; Agent
  suggestion is a candidate; writeback requires daemon preview/effect/receipt.
- **AcquisitionTierPicker:** active only for delivery status `Now`. Target-only
  tiers use RequiresBackendBlock, not disabled buttons.
- **RequiresBackendBlock:** non-interactive, dependency-specific and visually
  distinct from loading, permission denied and temporarily disabled.

### Component bans

The existing bans remain, with these additions: fake step progress,
thinking-animation-as-status, generic ChatBubble walls, active-looking target
controls, provider-secret previews, hidden provenance, and MCP host-session
controls.

The earlier component list remains useful for current Task/Provider/System
facts. Where names assume the old seven-space shell, this amendment controls.

---

## 1. Taxonomy

| Category | Contains | Owns |
|---|---|---|
| **Foundation** | tokens (type/color/space/radius/border/elevation/motion/icon/density/breakpoint), redaction pipe, untrusted-text escaping, mono-data primitive | honesty + theming primitives |
| **Layout** | Shell, StatusStrip, Sidebar, MasterRegion, InspectorRegion, DetailRoute, ComposedSurface | frame + regions |
| **Navigation** | PrimaryNavItem, SecondaryNav (section anchors), ContextualLink (object→object), BreadcrumbRef (digest chain), DeepLink | wayfinding |
| **Data Display** | DataTable (stable columns), FactGrid, KeyValueMono, DigestChip (copyable), TimestampWithAge, CountBadge | dense honest data |
| **Object Display** | AgentRow, WorkRow, ResourceItem, ProviderRow, IdentityCard (the 7 runtime identities), BindingRow | object grammars |
| **State** | StateDot (7 categories, shape+color), StateLabel (verbatim mono), StateChip (dot+label), ReasonLine, HonestyNote (S7/BD reference), CoverageBanner | the state system made visible |
| **Interaction** | PrimaryButton, QuietButton, DangerButton (separated), ConfirmCheckbox (exact-tuple copy), DisclosureChevron, CopyAffordance, FilterBar, SavedViewChip | actions & controls |
| **Feedback** | ReceiptLine (post-action digest/id), InlineError (field-near), StaleMarker, DisconnectedBanner, LivePulse (S2 only) | response & freshness |
| **Overlay** | CommandPalette, Menu, Sheet (narrow widths), ConfirmSurface (consequential acts) | floating layers (only material.floating users) |
| **Command** | PaletteInput, PaletteGroup, PaletteRow (action/object/destination), DisabledReason (for context actions) | ⌘K layer |
| **Inspector** | InspectorPanel, FactSection, RelatedLinks, ActionBlock (class-A/B/C rendering rules) | the 5-minute layer |
| **Timeline** | RunTimeline (dual-lane), TimelineNode (authority/observation), GapSpan, WatchHeader (state+cursor+attach/detach) | the Run reading |
| **Evidence** | EvidenceBlock (report/acceptance/digests), VerificationDisposition, ArtifactRefList, ProvenanceChain (candidate→decision→object) | proof made legible |

Forbidden components (taxonomy-level bans): MetricCard/KpiTile, ChartWidget (wave 1), Toast (authority facts persist as receipts/rows), ModalChain, CardWall, HeroBanner, ChatBubble, SparkleBadge.

## 2. Core component specs

### Navigation / Sidebar / Toolbar

- **Sidebar:** seven PrimaryNavItems; selected = keyline+tint+weight (triple signal); counts only on Home/Activity (S4 numerals, vanish at 0); footer = ⌘K + wordmark + honesty caption. States: full / icon-rail / top-strip (per `11` §7). Keyboard: `g+<key>` mnemonics; aria-current on selection.
- **Toolbar (space-level):** space title (`type.title1`) + primary action (one only) + filter affordance; never a row of competing buttons. Detail routes: header per `15` §2 (identity + state + actions), persistent.

### Command Palette

Per `21`: grouped rows, context-first ranking, class-B inline execution only, receipts inline, full keyboard model, focus return, all palette states (empty/loading/results/no-results-with-BD-note/denied/error).

### Master List (DataTable)

- Stable columns (defined per space, never reordered live); compact density default (`density.row.compact`); row = selection target (single action) + one primary affordance; sort/filter reflected in visible chips; selection preserved by object ID across refresh; changed-since-view marker (quiet dot) instead of live re-sorting; column resize is pointer-draggable with 1:1 tracking + keyboard alternative.
- States: loading (static skeleton bars, no shimmer), empty (how data arrives + one action), partial (facet-level S7), stale (footer age), denied, disconnected (last-good labeled).

### Detail View

Header (identity + state + actions) + section navigator (scroll-spy anchors) + content sections + facts inspector. Rules: persistent header; deep-linkable sections; back preserves master context; every digest/ID copyable.

### Inspector

The 5-minute layer: object name + state + reason; FactSections (mono values, copyable); RelatedLinks (contextual nav); ActionBlock — class-A opens its governed flow, class-B executes inline with receipt, class-C renders the not-available text + CLI path (never a disabled button, DD-08). Inspector never edits in place.

### Status (the state-system component)

StateChip = StateDot (category shape+color) + StateLabel (verbatim domain word, mono) + optional ReasonLine (muted, one sentence). Usage rules per `22` §4 (row/header/queue/badge/timeline variants). Unmapped domain words render S7 + "unmapped state" note — enforced here at component level.

### Agent Row

State dot (source-labeled lifecycle projection) + display identity + binding state (callable/blocked/unbound) + current-work link or "none observed" + open affordance. Never synthesizes status from process liveness (observation ≠ authority).

### Work Row

State chip + short ref (mono) + draft-type/objective (Tier-dependent, `14` §1) + agent + age + evidence mark (✓ evidence-linked / ■ failed / — none). Tier-1 footer honesty line is part of the list component's contract.

### Run Timeline

Dual-lane (`15` §3): authority lane (solid nodes: transitions, admission, verification, acceptance) / observation lane (hollow `obs`-labeled nodes: process facts). GapSpan (dotted, "no recorded facts") for unobserved spans. WatchHeader: state + cursor + attach/detach (detach = observation-only caption). New nodes append without moving read position; "N new" pill; jump-to-latest explicit. Truncation renders "showing N of M (bounded)".

### Evidence Block

Verification disposition (status + report ref + digest chip + currency flag + completed-at) + acceptance record (terminal transition ref/digest + currency) + artifact refs. The only component where "completed/verified" language may render; it always carries its refs. 404 → designed absent state ("no terminal evidence recorded" + meaning + Run link).

### Provider State

Account status chip (active/degraded/revoked) + secret presence chip (present/absent/unknown — never a value) + probe fact (class + duration + age) + catalog revision. Composite, since provider truth is multi-fact; each facet separately sourced.

### Resource Item

Family-specific row grammar (per `18`): Memory = id/scope/version/expiry-or-tombstone; Skill = package/revision/binding-status; Tool = op-id/risk/lifecycle/readiness-with-caveat. Shared rule: envelope-only facets labeled.

### Empty State

One quiet glyph (optional) + what this is + how data arrives + one primary action. No illustrations, no marketing voice. Examples: "No providers. Add an account to give agents a model route." / "No effects recorded — this task attempted no external mutation."

### Error State

Cause-first sentence + stable error class (mono) + what was preserved + next action + (technical) copy-details affordance. Never "Something went wrong". Field-near for forms (InlineError); region-level for projections; page-level only when the whole route failed.

### Loading State

Stable frame + static skeleton bars (layout-identical to loaded content); no spinner-only surfaces; cancellable where the underlying request is; long loads show elapsed time.

### Unavailable State (S7)

The product's signature state: hollow dot + verbatim label + one-line reason + named dependency (BD-n) or source + (where a CLI path exists) the CLI verb. Rendered with the same craft as success — it is information, not absence.

## 3. Component-level accessibility contract

Semantic tables for masters; rows are buttons/links with real focus; DisclosureChevron is a button controlling visible/hidden content; palette = combobox+listbox semantics with focus management; timeline nodes are list items with text equivalents; StateChip text is always present (screen readers never rely on dot shape); all copy affordances announce completion via live region; focus visible at 2 px accent ring; touch targets ≥ 24 px desktop / 44 px coarse pointers.

---

*This taxonomy is the boundary between design and implementation: Phase 3 (when authorized) implements exactly these categories; anything missing is added here first via a design-decision entry.*
