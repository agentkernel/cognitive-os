# 39 — Control Plane Implementation Waves

- Phase 2.5 (audit/planning only — no implementation authorized)
- Date: 2026-08-24
- Inputs: refactor verdicts (`36`), traceability (`35`), verified backend dependencies (`37`), design challenges (`38`), capability matrix (`29`).

---

## 1. Implementation strategy decision (brief §21)

| Option | Verdict |
|---|---|
| A. Incremental frontend refactor | **CHOSEN — with page-level replacement inside it** |
| B. Partial frontend rewrite | closest alternative; rejected as a *frame* because it implies discarding subsystems wholesale |
| C. Frontend + API refactor | rejected: no API change is required for the wave-1..12 set (`35` §10) |
| D. Broader architecture refactor | rejected: contradicted by evidence (the daemon surface is coherent; its gaps are additive, not structural) |

**Strategy statement:** one SPA stays in `cognitiveos-clients/pc/web/`; the **tested logic layer is kept** (channels/session/policy/probe/watch/identities/draft); the **view layer is replaced page-by-page** onto a new shell + token + state + data foundation; the only new backend work is the separately-gated BD register. Evidence basis: `36` §1-2 (the security/honesty logic is the app's best-tested code; the view layer's content model is what's wrong).

## 2. Wave ordering rationale (why this order, not the brief's default)

The brief's default order (Shell → design system → Home → Work → Work Detail → Agents → Providers → …) is reasonable but dependency analysis favors a different sequence:

1. **Foundation must land first** (tokens/state/data layer/shell) — every page depends on it.
2. **Providers goes before Home/Work** because: (a) it reuses the most existing tested logic (lowest risk proof of the new language); (b) it exercises the *hardest* honesty case (secrets + CAS mutations) early, validating the action model; (c) it executes DD-04 (Bindings fold), removing the nav drift at its root; (d) it has **zero backend gates**.
3. **Work/Work Detail come after** the data layer is proven and after the owner has had a window to schedule BD-3 (inventory depth) — the Work center page is the one surface with a real backend gate.
4. **Command palette is late** because it indexes the spaces — it must come after they exist (DD-09: IA-bound scope).
5. **Watch streaming** is decoupled into its own wave (client refactor against existing SSE; not on any page's critical path).

## 3. The waves

### Wave 0 — Repository & governance preparation (owner-dependent)
- Resolve client-repo write access (P7-T05/D10 remediation: cursor[bot] 403) — **owner action**; without it, all client work is local-only.
- Confirm OQ-1 (labels: Work vs Tasks; System space), OQ-6 (data-layer approach: hand-rolled projection store vs TanStack Query), OQ-4/5 can trail.
- Set up the design-doc → implementation traceability habit (each wave PR cites the design docs).

### Wave 1 — Foundation (frontend-only)
Token layer (`11`); state system components (`22`); error normalizer + route whitelist + 200-stub detector (`28` R-1/R-2); `secret_ref` display policy (R-5); data/projection layer (`36` §3: subscribable projection store with `{data, cursor, stale, source}`); new shell (`12`: strip, 7-space sidebar, gate chrome) with placeholder space pages; designed 404. **Exit:** all existing tests pass; shell renders all spaces with honest empty states; no page content replaced yet.

### Wave 2 — Providers space (refactor; the proving ground)
Accounts master (triage order) + account detail five sections (Overview/Models/Bindings/Usage/Audit) + create flow + key handoff + probes + binding set/remove with CAS (folded from BindingsPage; gates moved intact) + dsh apply in binding context. Retire `#/bindings` route (redirect). **Exit:** the full provider journey (create→key→probe→bind→usage→alerts→audit) runs on the new shell with the new state language; existing policy/probe tests pass unchanged.

### Wave 3 — Home (composition)
Readiness line + attention queue (composed: readiness + alerts + per-known-task evidence + binding dispatchability) + current work strip (session-observed + envelope, Tier-1 honesty) + recent evidence + critical-changes group. **Exit:** Flow 1 scenario test passes; every row navigates to its object.

### Wave 4 — Work inventory + New-task flow
Inventory master Tier-1 (envelope + session-observed, honesty footer) + inspector + filters + the governed creation flow as a full-route experience (ambiguities/`clarification_required` first-class). **Exit:** operator can find known tasks and create/admit a governed task without touching raw JSON.

### Wave 5 — Work Detail + Run timeline (the signature)
Persistent header; six sections (supervision order); dual-lane Run timeline (evidence transitions + observation/watch); Effects table; Evidence Block; Intent & Contract provenance chain (preview = digest + ephemeral copy, per `30` §5); Context section (pins when present, S7 otherwise); class-C control block. **Exit:** Flow 2/6/7 scenario tests pass; evidence-linked completion rule holds.

### Wave 6 — Agents dossier
Inventory rows + dossier (identity cards via kept `identities.ts`; binding section; capabilities; class-C lifecycle block; dsh runtime block). **Exit:** agent trust questions answerable; no fake controls.

### Wave 7 — Resources family pages
Family hub + Memory (list/explain/remember/forget) + Skills (import/bind/revoke/explain) + Tools (catalog/lifecycle with readiness caveat) + Context pointer page. **Exit:** all existing HTTP verbs reachable from UI; envelope limits labeled.

### Wave 8 — Activity
Seven-kind evidence stream (honest composition + coverage banner) + per-object timelines + alert ack inline. **Exit:** J-I1 satisfiable per object; coverage honestly stated.

### Wave 9 — System
Readiness detail + doctor (placeholder sub-sections honest) + stewardship (backup/restore flows) + session detail + about. **Exit:** Flow 8 passes; restore friction per spec.

### Wave 10 — Command layer (⌘K)
Palette per `21` (context actions → objects → destinations), class-B inline execution only. **Exit:** keyboard scenario test passes.

### Wave 11 — Watch streaming + refresh policy
Real `EventSource` wiring (kept controller semantics); bounded polling where watch is inert (OQ-2 decision); stale/gap semantics verified. **Exit:** live-within-reality updates; no fabricated finals.

### Wave 12 — Accessibility / QA gate
Keyboard-complete pass; contrast measurement on final hues (OQ-4); reduced-motion/transparency checks; scenario suite from `25` §2.2 re-run; Apple/Agent-UX review checklists re-run against the built product.

## 4. Backend-gated tracks (parallel, owner-scheduled — not waves of this plan)

| Track | Unblocks | Nature |
|---|---|---|
| BD-3 task inventory projection | Work Tier-2 columns, Home depth, Activity breadth | read-only projection route (Lane-CTR-light) |
| BD-4 watch deltas | live depth everywhere | daemon publish wiring |
| BD-5 unified audit feed | Activity unified stream | authority event projection |
| BD-2 agent lifecycle HTTP | dossier depth + controls | typed routes (bigger; fencing semantics) |
| BD-1 task control HTTP | intervention verbs | typed routes (bigger; reconciliation semantics) |
| BD-6/BD-7/BD-9/BD-10 | memory search; session logout; bootstrap ergonomics; projection-plane reconciliation | assorted |

---

*The first slice is defined in `40-phase3-first-slice.md`; readiness verdict in `41`.*
