# 40 — Phase 3 First Implementation Slice

- Phase 2.5 (planning only — no implementation authorized)
- Date: 2026-08-24
- Selection criteria from the brief: vertically useful · real backend support · no fake APIs · no broad architecture rewrite · demonstrates the new Apple Control Plane design · establishes reusable design primitives.

---

## 1. The slice

> **"Foundation + Providers": the new app shell (status strip, 7-space sidebar, session chrome) on the token/state/data foundation, with the Providers space fully delivered end-to-end — accounts master (triage), account detail (Overview · Models · Bindings · Usage · Audit), create flow, key handoff, bounded probe, binding set/remove with CAS, dsh apply in context — and every other space shipping as an honest placeholder page.**

One sentence: *the owner can run the complete provider-governance journey on the new instrument, and every other space visibly exists in its honest empty state.*

## 2. Why this slice is first (evidence, not preference)

1. **Real backend support, zero gates.** Every Provider route is implemented and verified (`28` §4; `29` §4): accounts CRUD, key set/rotate/remove, models refresh/add/set-price, bindings set/remove with CAS, usage/budgets/alerts/audit, dsh apply. No BD dependency stands in this slice.
2. **Vertically useful on day one.** The provider journey is the current product's most complete and most-used operator flow (P8-T13 shipped it; P7-T05 D08/D09 proved it live). The slice improves a flow the owner actually runs.
3. **No fake APIs, by construction.** The slice consumes only verified routes; its honesty cases are the *hardest* in the product (secret handling, CAS conflicts, trust reconfirm, degraded states) — if the design language survives Providers, it survives everything else.
4. **No broad rewrite.** The slice keeps and reuses the strongest existing code: `channels.ts`, `session.ts`, `policy.ts` (CAS derivation, dispatch/apply gates, redaction, cost honesty), `probe.ts` (trust gate, error classification) — all tested today (`36` §1).
5. **Demonstrates the new design.** Strip + sidebar + master + inspector + state chips + mono ledger + governed mutation flows (preview→confirm with exact tuple+revision) + receipts — the full design vocabulary appears in one space.
6. **Establishes the reusable primitives.** Tokens, state system, data/projection layer, error normalizer, master list, inspector, confirm surface, receipt line, empty/error/loading/unavailable states — every later wave consumes exactly these.
7. **Executes DD-04 early.** The Bindings fold happens here, removing the current IA's worst drift while its logic (CAS gates) is reused intact.

## 3. Explicitly in / out

**In scope:**
- Shell (strip cells: daemon/readiness/session/alerts/watch/⌘K-affordance as display; ⌘K itself is Wave 10), sidebar with 7 spaces, session gate chrome, designed 404, placeholder pages for Home/Work/Agents/Resources/Activity/System (honest empty states, not fake content).
- Foundation: tokens (dark first; light follows in the same token pass or a fast-follow per OQ-4), state system components, data/projection layer, error normalization + route whitelist + stub detection, `secret_ref` presence-only display policy.
- Providers space complete per `17` (list triage; five detail sections; create; key handoff; probe; manual model; set-price; binding set/remove with CAS + consequence preview; dsh apply gated; usage/budgets/alerts with advisory annotations; audit).
- Tests: keep all 9 existing test files passing; add network-layer tests (mock fetch: channel binding, stub detection, error normalization, redaction); component tests for the new primitives; the Provider journey DOM tests.

**Out of scope (explicitly):** Home composition, Work inventory/detail, Run timeline, Agents dossier, Resources pages, Activity stream, System pages, ⌘K palette, watch streaming, any backend change, any new dependency that isn't justified (OQ-6 decides the data-layer approach), light theme polish (token structure supports it; hues finalized at OQ-4).

## 4. Acceptance for the slice (design-level; implementation phase refines into PR checks)

1. The provider journey runs end-to-end on the new shell against the real daemon (create → key → probe → bind → usage → alert ack → audit), with all existing policy/probe/channel tests green.
2. Every state in `22` that Providers can produce is rendered by the state system (active/degraded/revoked, secret present/absent/unknown, CAS-stale, trust-reconfirm, cost_unavailable, alert warning/exceeded).
3. No raw-JSON default presentation anywhere in the space (Raw projection tab retained per DD-10).
4. Placeholder pages render honest empty states; no dead buttons; no class-C verbs rendered as controls.
5. Accessibility: keyboard-complete through the whole space; focus visible; state never color-only.
6. Honesty: no invented route is called (route whitelist); 200-stubs render as not-run; redaction negatives still pass.

## 5. Why not the alternatives

- **Home first:** composition-heavy, needs the data layer *and* produces a read-only surface; weaker proof of the action model; higher risk of dashboard drift under pressure.
- **Work first:** its center page is BD-3-gated (envelope-only inventory) — the first slice would ship with an honesty footer at its thinnest, on the product's most important surface. Better to arrive at Work with proven primitives and (ideally) BD-3 scheduled.
- **Shell-only foundation:** no operator value; violates "vertically useful".
- **Resources first:** real but lower-frequency value; the mutation grammar (import/bind/revoke) is simpler than Providers' — weaker stress-test of the design language.

## 6. Preconditions (owner)

1. Client-repo write access resolved (or an owner-run publication path) — otherwise the slice is local-only.
2. OQ-1 label decision (Work vs Tasks) can trail this slice (sidebar labels are data, not structure).
3. OQ-6 data-layer decision made at slice start (recommendation: hand-rolled projection store, `36` §3).

---

*Readiness verdict and full blocker list: `41-implementation-readiness.md`.*
