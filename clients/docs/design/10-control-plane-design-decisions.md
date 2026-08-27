# 10 — Control Plane Design Decision Log

- Status: adopted Personal 2.0 decisions; historical DD-01–DD-17 retained
- Updated: 2026-08-27
- Format per decision: **Decision / Context / Options / Chosen / Why / Rejected alternatives / Consequences**. Decisions are design-phase decisions: they bind subsequent design phases (UX/visual/component specs) and become implementation input only when the owner authorizes implementation. None of these decisions modifies product code, contracts, or canonical docs; changing canonical product IA (`docs/product/personal/`) is a separate owner-approved `product-semantic` step.

## Personal 2.0 adopted decisions

These decisions supersede DD-02, DD-04, DD-05, DD-06, DD-11, DD-14,
DD-16, and the chat/multi-Agent/provider/MCP portions of DD-17 where they
conflict. The earlier entries remain the decision history of the
2026-08-24 P7-T05-centered proposal.

### DD-18 — Desktop entry = global Agent Shell + Control Plane

- **Chosen:** one desktop product with native Agent conversation by default and
  explicit Manage with Personal into daemon-governed Work.
- **Why:** first value is a real Agent conversation; governance becomes useful
  when consequential work begins.
- **Consequences:** conversation/history needs an Adapter common projection and
  display/artifact-only native slots. Goal/Plan conversion is never implicit and is
  `Requires-backend` until typed.

### DD-19 — Six first-level destinations

- **Chosen:** `Home / Agents / Work / Library / Activity / Settings`.
- **Consequences:** Providers and System move under Settings; Resources becomes
  Library; Context moves to Work; Runtime moves to Agents. Current seven routes
  remain factual implementation evidence, not target IA.

### DD-20 — Agent integration is Adapter-backed and first-chat complete

- **Chosen:** install/connect in no more than three understandable steps,
  signed-catalog posture, capability matrix, embedded native history, then a
  real first chat.
- **Consequences:** disconnect and uninstall are separate; unsupported catalog,
  install, lifecycle and conversation capabilities are `Requires-backend`.

### DD-21 — Managed Work model

- **Chosen:** daemon-owned `Goal -> Plan revision -> Task -> attempt`, with
  Context, Effects and Evidence, and daemon-orchestrated multi-Agent roles.
- **Consequences:** plan history is immutable/versioned; no fake progress;
  existing Task evidence is reused but does not prove Goal/Plan support.

### DD-22 — Seven target families, task-oriented placement

- **Chosen:** Personal 2.0 families are Memory, Skill, Tool, Context, Task,
  Runtime/Process, and MCP. Library contains Memory, Skills, Tools, and MCP;
  Work contains Context and Task; Agents contains Runtime/Process.
- **Consequences:** MCP gets a first-class Library page. Model, Permission,
  Artifact, Budget, Evidence, and Event remain cross-cutting objects, not
  families. The product organization does not claim a universal backend record.
  MCP + rules does not control host Agent sessions.

### DD-23 — Account Hub lives in Settings

- **Chosen:** tiered account acquisition: OAuth/subscription, API key,
  user-directed import under
  [ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md),
  and custom gateway; daemon SecretStore/proxy custody always.
- **Consequences:** only verified API-key/provider functions are current-backed.
  Import/OAuth/subscription/unsupported quotas and costs are
  `Requires-backend`.

### DD-24 — One provenance timeline

- **Chosen:** Activity and object timelines share
  `Native / Observed / Governed / Verified` provenance.
- **Consequences:** sources can align in time but never merge identities or
  authority. Unified cross-domain coverage remains Requires-backend.

### DD-25 — Federated observation and governed writeback

- **Chosen:** show source/revision/conflicts across Personal and Agent-native
  resources. Agent Shell may suggest resolution; daemon preview/confirm/Effect
  performs any supported writeback.
- **Consequences:** observation never grants write authority; unsupported
  writeback has no active control.

### DD-26 — Brand-new calm desktop system

- **Chosen:** three-region desktop, master/detail/inspector, command palette,
  full keyboard, reduced motion/transparency, calm-dense-precise-professional
  visual language.
- **Rejected:** Liquid Glass everywhere, giant gradients/cards, marketing
  spacing, card-wall dashboards and fake progress.

### DD-27 — Target controls use `Requires-backend`

- **Chosen:** target-only behavior is documented in place with dependency and
  non-interactive state. Current-backed controls alone may look actionable.
- **Why:** disabled buttons and simulated progress both overstate capability.

### DD-28 — Render slots never own actions

- **Chosen:** Adapter-specific render slots display bounded metadata/artifacts
  only. They cannot inject controls, executable markup/scripts, credentials, or
  authority-shaped state.
- **Consequences:** vendor-specific actions use Control Plane-owned components
  backed by typed capability semantics.

### DD-29 — Capability status has three independent axes

- **Chosen:** runtime condition is
  `Supported|Unsupported|Unavailable|Unknown`; delivery status is
  `Now|Requires-backend`; support path is
  `vendor-native|managed-adapter|MCP-cooperative|observable-only|unqualified`.
- **Consequences:** no runtime condition is used as a delivery claim, and no
  delivery gap is misreported as vendor `Unsupported`.

### DD-30 — MCP fallback and preauthorization are exact-scope only

- **Chosen:** preference is vendor-native API -> managed Adapter -> MCP plus
  rules as cooperative fallback. Automatic reconciliation is limited to an
  exact unchanged grant.
- **Consequences:** permission expansion, a new client, trust-boundary or target
  expansion, or an incompatible update requires fresh daemon preview and
  confirmation. Reload/restart remains a separate typed action and receipt.

---

## Historical 2026-08-24 decisions (superseded where noted)

## DD-01 — Product model: Cognitive System Control Plane

- **Context:** the brief demanded re-choosing the product model (Dashboard / Agent Management Console / AI Operations Center / Cognitive System Control Plane / Personal AI Operating Environment / Agent Workbench / other) by JTBD, capability and future direction — not by fit to the current UI.
- **Options:** models A–G in `01-control-plane-product-model.md` §2.
- **Chosen:** **D — the operator surface of the owner's local cognitive authority**, with the ops-center supervision loop absorbed as the daily loop and console-grade management absorbed where genuinely supported (Providers, Resources).
- **Why:** it is the only model that matches the authority architecture (daemon sole writer; UI = client), the governance grammar (preview→admit→verify), the evidence discipline, and the ranked jobs; it requires zero invented capability.
- **Rejected:** A (metrics theater, no honest denominators); B (centers object CRUD; its showcase verbs are the ones the backend lacks); C (supervision strong, but under-weights governance and drifts to NOC aesthetics); E (names the whole product, not this surface); F (centers a manipulable artifact the owner doesn't hand-edit).
- **Consequences:** every downstream surface leads with authority state; actions are governance operations; evidence is first-class; the one-line test "does this help supervise or govern the authority, with evidence" gates all future additions.

## DD-02 — IA: Option D (Supervision IA, seven spaces)

- **Context:** the current flat eight-peer sidebar drifted from the canonical IA and embeds structural defects (no task inventory anchor; Activity mis-scoped; Bindings unsanctioned; Session as a peer).
- **Options:** A Agent-centric, B Work-centric, C System-centric, D Supervision (`05-control-plane-ia-options.md`).
- **Chosen:** **D** — `Home · Work · Agents · Providers · Resources · Activity · System` + persistent status strip + ⌘K command layer.
- **Why:** serves the ranked jobs in order; every space backable today or honestly deferred; keeps the canonical five spaces in spirit while fixing their implementation; scales by depth, not by new peers.
- **Rejected:** A (primary axis on a projection; backend can't answer per-agent work today); B (strongest spine but its center page depends on BD-3 and orphans the attention question); C (buries daily supervision under admin furniture; SaaS-admin risk highest).
- **Consequences:** route map and per-space contracts per `06`; Home carries an explicit anti-dashboard guardrail; each space owns its route-state matrix.

## DD-03 — Home is an attention surface, not a dashboard

- **Context:** "Home" defaults to dashboard in most products; the brief warned against both "marketing dashboard" and "少等于好" (less-is-good) minimalism; the operator needs fast system comprehension.
- **Options:** (a) metric dashboard; (b) minimal welcome; (c) attention surface (readiness + needs-attention queue + current work strip).
- **Chosen:** (c). No charts, no KPI cards; every row is a navigable authority fact or an action.
- **Why:** the rank-1 job is "decide in seconds whether to act"; the honest data is discrete states and exceptions, not aggregates (budgets are observe-only; there is no metric denominator worth charting); `web-ui-design.md:160-165` already bans card walls/dashboard strips.
- **Rejected:** (a) decoration without decisions; (b) wastes the highest-frequency screen.
- **Consequences:** visual phase's first review gate is "did Home stay a queue, not a dashboard"; the queue's priority rules (blocked > failed/unknown-outcome > unacknowledged alerts > degraded > stale) are content rules, not styling.

## DD-04 — Providers first-level; Bindings folded

- **Context:** the shipped nav has Providers and Bindings as peers; canonical IA sanctions a dedicated provider operator view (`web-ui-design.md:41-48`) but not Bindings.
- **Options:** keep both; fold both under Resources; Providers first-level + Bindings folded into Providers (by account) and Agents (by actor).
- **Chosen:** Providers first-level with seven sub-surfaces (Overview/Models/Bindings/Usage/Audit per account + list); Bindings becomes a relation edited where it is understood.
- **Why:** Provider governance is the deepest live domain (accounts/keys/trust/catalog/bindings/usage/budgets/alerts/audit); Bindings-as-space separates a relation from both of its endpoints, producing the current page's complexity (it must re-establish agent+account+model context every visit).
- **Rejected:** status quo (nav drift); Resources-fold (Provider is not a sixth resource family — canonical rule).
- **Consequences:** binding changes always launch with one endpoint preselected; the Apply-to-dsh flow lives on the binding context; the old `#/bindings` route redirects.

## DD-05 — Session is chrome, not a space

- **Context:** Session is currently a sidebar peer; it is a gate and a utility, not a destination.
- **Chosen:** session state lives in the status strip; re-auth is an inline gate preserving the intended route; `#/session` remains addressable but unlisted.
- **Why:** wayfinding hygiene — destinations are things you *visit*, gates are things you *pass through*; sidebar real estate is the scarcest calm resource.
- **Consequences:** strip shows principal + expiry + channel health; memory-only session cost (re-paste on reload) is stated in-product once, with BD-9 recorded for any future ergonomic improvement.

## DD-06 — "Tasks" labeled "Work"; System space added (canonical IA delta — owner-confirmable)

- **Context:** canonical IA names five spaces (Home, Agents, Tasks, Resources, Activity). The recommendation renames Tasks→Work and adds System.
- **Chosen (design recommendation, flagged):** Work (the space contains Runs as a first-class reading; "Work" covers task inventory + run detail without a sixth peer); System (readiness detail, doctor, stewardship, session diagnostics — fragments currently homeless or parked in Home).
- **Why:** Run is where the daily story is read (`04` §1.3); readiness/doctor/backup need a permanent home that isn't the attention surface.
- **Rejected:** strict canonical labels (acceptable fallback — the delta is exactly two labels); separate Runs peer (over-fragmentation).
- **Consequences:** **this is the one decision that touches canonical product language**; if the owner prefers canonical purity, relabel Work→Tasks and fold System's readiness back to Home while keeping stewardship under Home→System link. Recorded as open point OP-1 for the owner; no canonical doc is modified by this phase.

## DD-07 — Run is a presentation object, not a backend entity

- **Context:** the brief asked whether Run is the execution instance. Audit: no first-class Run/execution listing exists on the operator API (BD-3/BD-4); Activity canonically contains Run.
- **Chosen:** formalize Run as a **task_ref-scoped presentation composition** (transitions + execution/epoch facts + process observations + effects + evidence + watch events) with an authority lane and an observation lane.
- **Why:** it answers "what happened / what is it doing" today, from real facts, without inventing an entity; if a backend Run listing lands later, the model absorbs it unchanged.
- **Rejected:** waiting for a backend Run entity (blocks the core supervision reading); presenting process output as the run (observation ≠ authority violation).
- **Consequences:** `#/work/:taskRef/run` is a designed route; its empty/partial/stale states are specified; the authority/observation lane separation is a hard visual rule.

## DD-08 — Deferred verbs render as "not available" text + CLI path, never as disabled buttons

- **Context:** task cancel/pause/resume/retry and agent lifecycle verbs have no HTTP route; the shipped UI shows `not-run` labels; a conventional UI would show disabled buttons.
- **Chosen:** class-C treatment: a short factual line ("Task cancel is not available over HTTP yet — use `cognitive …`; tracked as backend dependency BD-1"), placed where the control would live.
- **Why:** a disabled button implies a capability that exists but is momentarily off; that is false here. Text + path is honest, teaches the CLI fallback, and removes the "why is this disabled?" failure class. Consistent with the frozen route inventory's `ui_render: "not-run"`.
- **Rejected:** disabled buttons (dishonest affordance); hiding the verbs entirely (hides the product's own roadmap reality from the operator).
- **Consequences:** the verb set is stated on the admission screen *before* work starts (the stop-button rule, truthfully); when BD-1/BD-2 land, the same slots upgrade to class-A controls without redesign.

## DD-09 — Command palette (⌘K) is the speed layer, not a space

- **Context:** high-frequency operators need scan speed and keyboard flow; the navigation-IA reference warns against palettes as junk drawers.
- **Chosen:** palette over destinations, objects by exact ID, and the current context's class-A/B actions; disabled/missing actions show reasons or are absent per honesty class.
- **Why:** the repeated-use ergonomics requirement ("the repeated path gets faster"); keeps the sidebar at seven calm peers.
- **Rejected:** palette as feature-dump; a separate Search space (search is retrieval over the same index, folded into the palette).
- **Consequences:** palette scope is IA-bound (it can only reach what the spaces contain); keyboard model is a wave-1 requirement, not a fast-follow.

## DD-10 — Master/detail + inspector is the density pattern; JSON demoted to an inspector affordance

- **Context:** the current UI's primary display is raw JSON panels; the product needs high density without high cognitive load.
- **Chosen:** stable master lists + selection inspector + full detail routes; raw redacted JSON survives only as a per-object "Raw projection" inspector tab (clearly labeled, redaction-preserving, copy-safe).
- **Why:** operators do need raw truth occasionally (debugging, support bundles); but as the default reading it fails the product promise (illegible authority). The pattern matrix prescribes master/detail for compare+inspect work.
- **Rejected:** removing raw access (loses debugging value); keeping it default (status quo failure).
- **Consequences:** component architecture phase must produce the master/inspector/detail system before page work; JsonPanel-equivalent becomes a sanctioned debug affordance with redaction intact.

## DD-11 — Activity: per-object timelines first; unified feed labeled and gated on BD-5

- **Context:** Activity is canonically Run/Process/Effect/Evidence; the shipped Activity page is provider-plane JSON; no cross-domain event feed exists.
- **Chosen:** Activity space = attention-ordered reading composed from real sources (alerts, failed/unknown effects, terminal evidence, observed transitions) + links into per-object timelines; an explicit coverage label states what the feed is and is not until BD-5 lands.
- **Why:** honesty about coverage is itself the feature (the owner must know the audit trail's boundaries); per-object timelines are where investigation actually happens (Flow 7).
- **Rejected:** a "unified feed" assembled client-side from partial sources presented as complete (fabrication risk); keeping Activity as provider audit only (fails J-I1).
- **Consequences:** Activity's empty/partial states carry the coverage statement; BD-5 is the named upgrade path.

## DD-12 — Theme scope: direction recorded, final tokens deferred to visual phase

- **Context:** shipped UI is dark-only hand-rolled CSS; `web-ui-design.md` §5 accepts an Apple-inspired system-like direction with quiet cool neutrals; the D10 refinement exists only as an unpublished bundle.
- **Chosen (direction):** system-following light/dark with the same information hierarchy; the dark theme is not the identity — the hierarchy is. Final palette/type/tokens are visual-phase outputs.
- **Why:** Apple fit includes adaptivity (colors that adapt to light/dark — Craft); the operator's environment varies (desktop browser, ambient light).
- **Rejected:** dark-only as permanent (flexibility failure); shipping the unpublished D10 bundle as-is (it was Provider-page-scoped, not a system).
- **Consequences:** visual phase owns tokens (color/type/spacing/radius/motion); the state vocabulary must be specified semantically first (this phase's §3 in `03`) so theming cannot weaken it.

## DD-13 — HashRouter retained until the daemon grows an SPA fallback

- **Context:** daemon serves `/ui/` with per-asset lookup and no fallback (`/ui/providers` 404s); the SPA uses HashRouter.
- **Chosen:** keep HashRouter in wave 1; record "daemon SPA fallback + BrowserRouter" as a technical constraint candidate for a future backend slice (not assumed).
- **Why:** capability honesty extends to URL behavior; designing BrowserRouter URLs the daemon cannot serve would ship broken deep links.
- **Consequences:** deep-link rules in `06` are hash-routes; shareability is slightly uglier, reliability is absolute.

## DD-14 — No notification center in wave 1

- **Context:** alerts exist (pull-only); the brief lists notifications as an IA question.
- **Chosen:** alerts surface in the status strip count + Home attention queue + Providers usage context; no separate notification center; no push.
- **Why:** the backend delivery is pull-based; a notification center would be a new container around one trickle of data — furniture.
- **Rejected:** notification center (inverted complexity); toasts for authority events (ephemeral ≠ authority; receipts persist instead).
- **Consequences:** alert acknowledge is class-B in context; if real-time delivery ever lands (BD-4), the strip upgrades without a new space.

## DD-15 — Capability-honesty contract inherited and hardened

- **Context:** the shipped UI's honesty behavior (not-run labels, never-inferred completion, unknown≠zero) is the product's best trait; the audit also found daemon-side honesty risks (200-stub fallthroughs; three error envelopes; inert watch).
- **Chosen:** the four-class action model (A/B/C/D, `03` §2) is binding on all design and implementation; the client must whitelist known routes and treat unknown 200-stubs as not-run; one error-normalization layer; watch freshness always displayed.
- **Why:** a redesign that polishes over these gaps would actively destroy trust (J-F1..J-F4) — the opposite of the product's identity.
- **Consequences:** the design system's state components are specified before any page visual; BD-1..BD-9 are the named backend dependency register for future phases.

## DD-16 — Primary user = the Individual Operator mode; personas are modes, not people

- **Context:** the brief asked to analyze five user types and name a first user; the product is single-owner.
- **Chosen:** primary = Individual Operator (supervision); secondary = AI Power User (curation) and System Operator (stewardship); deferred = Agent Builder; out-of-scope-as-primary = Developer.
- **Why:** documented primary user + product boundary (single-owner loopback); five personas would be fiction — five *modes* of one owner is the honest model and produces real design guidance (mode-appropriate frequency/risk treatment).
- **Consequences:** when configuration and supervision conflict, supervision wins; mode-switching is zero-cost (same principal, same surface).

## DD-17 — Scope record: deleted, deferred, never

- **Deleted from the current surface:** "Simulate cursor gap" as product UI; raw-JSON as default presentation; Bindings as first-level; Session as first-level.
- **Deferred with named dependencies:** task/agent control verbs (BD-1/BD-2); task inventory projection (BD-3); live watch deltas (BD-4); unified activity feed (BD-5); memory search/review (BD-6); session lifecycle endpoints (BD-7); budget enforcement (BD-8); session bootstrap ergonomics (BD-9); daemon SPA fallback.
- **Deferred by product plan:** multi-agent UI (P6); non-Pi qualification surfaces; Windows/macOS.
- **Never (boundary):** multi-user/RBAC; remote access; browser secret custody; generic lifecycle routes; metric-dashboard theater; chat in the Control Plane.
- **Consequences:** future phases inherit this record; anything resurrected requires a new decision entry.

---

## Open points for the owner

| # | Question | Recommendation | Why it needs the owner |
|---|---|---|---|
| OP-1 | Canonical IA labels: keep "Tasks"/five spaces, or accept "Work"+System (DD-06) | Accept Work+System | touches canonical product language (`product-semantic` change class) |
| OP-2 | Wave-1 scope appetite: which spaces get full depth first | Work + Home + Providers polish; Resources depth second | sequencing is a delivery decision |
| OP-3 | Light+dark vs dark-first sequencing (DD-12) | system-following, both | visual identity call |
| OP-4 | Whether BD-3 (task inventory) should be scheduled as backend work before/with implementation | yes — it is the center page's data | backend scheduling is plan-owner authority |

---

*This log closes Phase 1. Phase 2 (UX/visual design) may add entries; it may not silently overturn DD-01..DD-17 — overturning requires an owner-visible amendment here.*
