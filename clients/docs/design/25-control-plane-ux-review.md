# 25 — UX Review & Design Review Gate

- Phase 2 (design-only)
- Date: 2026-08-24
- Method: five review gates run against the Phase-2 artifact set (11–24) under the Phase-1 contract (01–10 + current-state + capability inventory). Method sources: stark `ux-design` (heuristic audit, scenario tests), `apple-design` (principles + brief §18 checklist), `ai-agent-ux`/`ai-trust-transparency`/`ai-error-resilience` (§19 checklist), product-skills conformance (Phase-1 contract), WCAG-informed accessibility baseline.
- Gate rule (from the brief): **any gate fails → fix the design, do not implement.** Results below are honest: two items are marked CONDITIONAL PASS with named follow-ups rather than waved through.

---

## 1. Product Review (Phase-1 contract conformance)

| Contract item | Phase-2 artifact | Verdict |
|---|---|---|
| Product model = Cognitive System Control Plane | all surfaces lead with authority state, actions are governance operations | PASS |
| IA = Option D, seven spaces (DD-02) | shell `12`; space specs 13–20 | PASS |
| Home = attention surface, no dashboard (DD-03) | `13` — zero charts/KPIs; queue+state only | PASS |
| Providers first-level, Bindings folded (DD-04) | `17` (bindings as account section + agent-contextual entry) | PASS |
| Session = chrome (DD-05) | shell §5; no sidebar peer | PASS |
| Run = presentation object, dual lanes (DD-07) | `15` §3 | PASS |
| Class-C verbs = text + CLI path, not disabled buttons (DD-08) | `14` inspector, `15` header, `16` header | PASS |
| ⌘K = speed layer, IA-bound (DD-09) | `21` | PASS |
| Master/detail + inspector; JSON demoted (DD-10) | `12` §4, `23` (Raw projection as inspector tab) | PASS |
| Activity honest coverage (DD-11) | `19` coverage banner | PASS |
| No notification center (DD-14) | strip + queue only | PASS |
| Capability honesty (DD-15): no invented routes/states | every spec cites its backing route or names the BD dependency; state map `22` §3 uses verified vocabularies only | PASS |
| Scope: deleted/deferred/never lists (DD-17) | respected across specs | PASS |
| **Deviation recorded** | Work detail section order changed to supervision order (`15` §1); Home R6 rendered as queue group (`13` §1) | LOGGED as DC-1/DC-3 — refinements, contract sections all preserved |

**Product gate: PASS** (with two logged refinements).

## 2. UX Review (heuristic audit + scenario tests)

### 2.1 Heuristic audit (stark 14-point)

| # | Check | Result |
|---|---|---|
| 1 | Primary job obvious on first screen | PASS — Home opens on readiness + attention queue |
| 2 | First value without reading docs | CONDITIONAL — session bootstrap requires the bootstrap secret from the local filesystem; gate copy explains exactly where/what, but first-run friction is real (BD-9; OQ-5) |
| 3 | Returning user repeats core task faster | PASS — ⌘K, stable masters, saved filters, recents |
| 4 | Empty/loading/error/permission/success states defined | PASS — route-state matrix `06` §5 + per-spec state tables + `23` state components |
| 5 | One primary action per surface | PASS — toolbar discipline (`23` §2) |
| 6 | Risky actions confirmed/separated/recoverable | PASS — exact-tuple confirms; restore is highest-friction; no fake undo (consequences stated) |
| 7 | Form errors near fields | PASS — InlineError contract |
| 8 | Layout supports scanning | PASS — stable columns, triage ordering, mono ledger |
| 9 | Navigation matches mental model | PASS — spaces = operator questions; verified against JTBD ranks |
| 10 | Next step clear after every action | PASS — receipts + related links + next-action copy grammar |
| 11 | Repeated path accelerates | PASS — speed layer + preserved list context |
| 12 | Focus/touch/keyboard usable | PASS — shell keyboard model + component a11y contract |
| 13 | Validation/recovery near input, input preserved | PASS — CAS stale → re-read + new preview; forms preserve input |
| 14 | Friction only where risk justifies | PASS — reads are frictionless; mutations carry preview; restore carries the most friction |

### 2.2 Scenario tests (compact, per stark scenario discipline)

| Scenario | Path | Success criteria | Remaining risk |
|---|---|---|---|
| First run | gate → Home (setup rows in queue) → Providers → create account → key → probe → bind → first task | reaches admitted task without leaving the surface or reading docs | bootstrap-secret discovery (BD-9/OQ-5) |
| Returning operator | Home → top attention row → Work detail → evidence | ≤2 interactions to the top item; verdict without scrolling past Run | none material |
| Error/recovery | provider `provider_secret_unresolvable` → Home row → account → rotate key → probe | cause named; repair inline; state re-projects | none material |
| Keyboard-only | `g w` → `j/k` → `return` → `[` to Evidence → copy digest | full path without pointer; focus visible throughout | verify in implementation QA |
| Narrow width (1100 px) | sidebar → top strip; inspector → sheet | all facts reachable; state language intact | dense operation degraded by design (stated) |
| Degraded daemon | disconnect mid-watch → stale label → reconnect | never fabricates a final state; last-good labeled | none material |

**UX gate: PASS** (one conditional, tracked).

## 3. Apple Review (brief §18 checklist)

| Dimension | Verdict | Evidence |
|---|---|---|
| Clarity | PASS | type scale with size-specific tracking/leading; verbatim state labels + plain reasons; nothing decorative competes with data |
| Deference | PASS | chrome = strip + sidebar only; content is authority data; materials reserved for true overlays |
| Depth | PASS | real layers (space → master → inspector → detail → evidence); navigational, not visual-effects |
| Consistency | PASS | one state grammar (`22`), one row grammar, one copy grammar, one action model (A/B/C/D) |
| Direct Manipulation | PASS (calibrated) | selection instant/local; column resize 1:1; mutations are governed flows — direct manipulation is deliberately *not* applied to authority mutations (preview→admit is the correct indirection; `09` §1.2) |
| Feedback | PASS | pointer-down response; receipts with digests; four feedback kinds with "completion" always evidence-linked |
| Motion | PASS | motion budget spent on orientation + S2 liveness only; reduced-motion equivalents specified |
| Typography | PASS | system font; optical sizing respected; mono ledger as identity texture |
| Accessibility | PASS (see §5) | |
| Information density | PASS | density via stable columns + disclosure layers; floors respected (`11` §3) |
| **Apple-like but not an Apple-website clone** | PASS | no hero, no marketing sections, no glassmorphism, no gradient washes; the likeness is in discipline (hierarchy, restraint, response), not in website tropes |

**Apple gate: PASS.**

## 4. Agent UX Review (brief §19 checklist)

| Dimension | Verdict | Evidence |
|---|---|---|
| Agency | PASS | owner admits exactly what executes; dial = contract, surfaced at preview |
| Autonomy | PASS | tier semantics visible; standing autonomy = bindings/exposure, governed in place |
| Trust | PASS | calibrated via dispositions + evidence, never percentage theater; trust-ramp mirrored (zero-capability empty states) |
| Transparency | PASS | interpretation ambiguities/gaps first-class; no fabricated reasoning narratives |
| Error Recovery | PASS | RECOVER mapping (`08` §5); fallback hierarchy ends in diagnostic client, never a cliff |
| Human Override | PASS (honest) | real levers (binding/tool/key) + class-C truth for missing verbs + CLI paths; control set declared at admission |
| Blast Radius | PASS | per-action audit table (`08` §5) executed in specs (one-way quarantine stated pre-confirm; restore friction; binding consequences) |
| State Visibility | PASS | dual-lane Run; watch state always visible; unknown/not-run first-class |
| Evidence | PASS | Evidence Block is the only place "completed" may render |
| Verification | PASS | independent-verification disposition separated from lifecycle state everywhere |
| Acceptance | PASS | acceptance record rendered as its own artifact with refs/digests |

**Agent UX gate: PASS.**

## 5. Accessibility Review

| Item | Verdict |
|---|---|
| Keyboard-complete (nav, masters, inspector, palette, confirms) | PASS (spec-level; implementation QA must verify) |
| Visible focus (2 px accent ring, offset, never obscured by sticky UI) | PASS |
| Color never sole carrier (shape+text+color state system) | PASS |
| Contrast gates (labels 4.5:1, dots 3:1, both themes; more-contrast variant) | CONDITIONAL — token hues are directional; final values must be measured in the visual phase (OQ-4) |
| Reduced motion / transparency / contrast | PASS (specified per token) |
| Screen-reader semantics (tables, listbox/combobox palette, live regions for watch/receipts) | PASS (contract in `23` §3) |
| Text scaling (rem layout) | PASS |
| Touch targets (narrow/coarse-pointer degradation) | PASS (≥44 px coarse) |

**Accessibility gate: CONDITIONAL PASS** — contrast verification of final hues is the named follow-up.

## 6. Design Challenges (found against Phase 1 / the brief — recorded, not silently fixed)

| # | Challenge | Resolution | Status |
|---|---|---|---|
| DC-1 | Phase-1 `06` listed Work-detail sections in contract order; supervision needs Run/Effects/Evidence read first | Reordered to supervision order in `15`; all six sections preserved; logged as refinement | Closed (owner may revert order without structural change) |
| DC-2 | Phase-1 `06` §4 mentioned "saved filters"; no server persistence exists | Filters are session-local in wave 1, labeled; server persistence joins BD register if wanted | Closed |
| DC-3 | The Phase-2 brief lists "Critical changes" as required Home content; a separate region duplicated the attention queue | Rendered as the top `change`-kind group inside Needs attention (`13` §1) | **Open for owner** — if a standalone region is preferred, it is a layout change only |
| DC-4 | Palette "disabled with reason" guidance vs class-C absence | Class-C verbs are absent from ⌘K by design (educated at the detail surface instead); deviation recorded in `21` §1 | Closed |
| DC-5 | Phase-1 named "Work" label pending owner (OP-1) | All Phase-2 specs use "Work"; relabel to "Tasks" is a label-only change if OP-1 resolves that way | Open (owner) |

## 7. Open Questions (carried into the next phase)

| # | Question | Owner | Blocks |
|---|---|---|---|
| OQ-1 | Phase-1 OP-1..OP-4 (canonical labels; wave-1 depth order; theme sequencing; BD-3 scheduling) | owner | implementation sequencing |
| OQ-2 | Home/Work refresh policy: manual-only vs bounded polling (daemon cost unmeasured) | owner + backend | implementation |
| OQ-3 | Browser-side diagnostics bundle export (even redacted) — allowed or CLI-only? | owner (security-relevant) | System spec detail |
| OQ-4 | Final palette hues + measured contrast in both themes | visual phase | visual lock |
| OQ-5 | Session expiry UX: proactive re-auth prompt vs reactive gate | owner (security-relevant) | shell detail |

## 8. Gate summary

| Gate | Result |
|---|---|
| Product (Phase-1 conformance) | **PASS** |
| UX | **PASS** (1 conditional, tracked) |
| Apple | **PASS** |
| Agent UX | **PASS** |
| Accessibility | **CONDITIONAL PASS** (contrast measurement pending in visual phase) |

**Design Review Gate: PASSED.** No gate failed; two conditionals are tracked as named follow-ups (OQ-4, OQ-5) rather than waived. Per the brief, the design is cleared for the next phase; implementation remains unauthorized.

---

*Phase-2 artifact set: 11 (tokens) · 12 (shell) · 13 (Home) · 14 (Work) · 15 (Work detail) · 16 (Agent) · 17 (Provider) · 18 (Resources) · 19 (Activity) · 20 (System) · 21 (command layer) · 22 (state system) · 23 (components) · 24 (visual direction) · 25 (this review). Phase-1 contract: 01–10 + current-state + capability inventory.*
