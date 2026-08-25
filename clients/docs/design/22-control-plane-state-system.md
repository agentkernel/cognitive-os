# 22 — Control Plane State System

- Phase 2 (UX / Interaction / Visual design — no implementation)
- Date: 2026-08-24
- Contract inputs: `03-control-plane-capability-model.md` §3 (verified state vocabularies + honesty rules), `04` (observation ≠ authority), `08` (trust calibration), `09` (color never alone). This document defines the **one display grammar** every surface uses for status. It is the first component the visual phase must implement, ahead of pages.
- Hard rule (inherited + hardened): the UI never mints authority states. It maps daemon vocabularies onto a small set of **display categories**; the exact domain word is always rendered as text next to the category signal.

---

## 1. The two-layer model: category × label

Every status rendering = **State Category** (visual semantics, 7 total) + **Domain Label** (the daemon's own word, verbatim, lowercase mono).

```text
[ ● Ready ]        ← category dot+word (visual semantics)
  ready · component=provider   ← domain label(s) verbatim
```

Why two layers: the daemon's vocabularies are domain-specific and must stay exact (honesty); operators need cross-surface scan consistency (a red thing always means the same *kind* of thing). Category carries the scan; label carries the truth.

## 2. The seven display categories

| # | Category | Meaning (operator reading) | Required signal set |
|---|---|---|---|
| S1 | **Ready** | Nominal; verified or admitted; nothing needed | green dot + "Ready"-family label |
| S2 | **Active** | Work in flight / live now (running, live watch, active binding) | blue accent dot + label; subtle pulse allowed ONLY here (see motion rules) |
| S3 | **Waiting** | Intentionally paused on a precondition (queued, awaiting admission, clarification_required, pending reconciliation) | amber hollow dot + label |
| S4 | **Attention** | Degraded but serviceable; needs the owner soon (degraded provider, budget warning, stale watch) | amber filled dot + label |
| S5 | **Blocked / Failed** | Cannot proceed / failed; needs the owner now (blocked, failed, verify_failed, revoked, crashed) | red dot + label |
| S6 | **Completed** | Terminally and verifiably done — **always evidence-linked** | neutral check + label + evidence link affordance |
| S7 | **Unavailable / Unknown / Not-run** | The fact itself is unavailable, unknown, or was never run | gray hollow dot + label + reason/dependency on hover-inspector |

Design rules:

1. **S7 is a first-class citizen**, styled as deliberately as S1–S6. Never blank, never zero, never implied-ready. This is where this product differs from every dashboard template.
2. **S6 Completed never renders without its evidence link.** A bare "Completed" string is a defect (DD-08 lineage, J-F1).
3. **S2 Active is the only category permitted motion** (a slow 2.4 s ease-in-out opacity pulse on the dot, 40%→100%→40%), because "live" is the one state where motion carries meaning. Removed under `prefers-reduced-motion` (static filled dot instead).
4. **Waiting vs Attention vs Blocked is the product's core triage distinction** — waiting = the system intends to continue; attention = degraded, acting soon helps; blocked/failed = intervention required. Copy must never blur these.
5. **Color is never the only signal:** dot shape differs per category (filled/hollow/check), label is always present, and icon+text accompany every row-level usage.

## 3. Domain → category mapping (verified vocabularies only)

| Domain object | Daemon vocabulary (source: `03` §3) | Category | Label shown |
|---|---|---|---|
| Readiness overall | `ready` / `degraded` / `blocked` | S1 / S4 / S5 | verbatim |
| Readiness component | `ready` / `degraded` / `blocked` / `not_configured` | S1 / S4 / S5 / S7 | verbatim |
| `first_conversation_ready` | true / false | S1 / S5 | "first conversation ready" / "not ready" |
| Task lifecycle (store) | `ACTIVE` / `CANDIDATE_COMPLETE` / `COMPLETED` | S2 / S3(+evidence note) / S6 | verbatim + evidence link on COMPLETED |
| Task display vocabulary (product) | proposed, awaiting admission, queued, waiting, suspended, reconciling, verifying | S3 | verbatim |
| | running | S2 | verbatim |
| | blocked, failed | S5 | verbatim |
| | completed | S6 (+evidence link) | verbatim |
| | cancelled, quarantined | S5 (neutral-red) | verbatim |
| Effect stage | `EXECUTED` `RECONCILED` | S1 | verbatim |
| | `PROPOSED` `AUTHORIZED` `EXECUTING` | S2 (executing) / S3 (proposed, authorized) | verbatim |
| | `NOT_EXECUTED` | S7 | verbatim |
| | `DENIED` `VERIFY_FAILED` | S5 | verbatim |
| | `OUTCOME_UNKNOWN` `MISSING` | S5 + emphasis (this is the post-crash honesty state) | verbatim |
| Effect reconcile | `pending_reconciliation` `must_reconcile` | S3 / S4 | verbatim |
| | `closed` / `not_applicable` | S1 / S7 | verbatim |
| Provider account | `active` / `degraded` / `revoked` | S1 / S4 / S5 | verbatim |
| Provider readiness | `secret_ref_resolves: true/false/unknown` | S1 / S5 / S7 | "secret present/resolvable" · "secret unresolvable" · "secret state unknown" |
| Binding | `active` + dispatchable / `active` + blocked / `revoked` | S1 / S5 / S7 | "callable" / "blocked — <reason>" / "revoked" |
| Tool lifecycle | `enabled` / `disabled` / `quarantined` / `revoked` | S1 / S7 / S5 / S7 | verbatim (quarantine one-way note) |
| Tool readiness | `execution_ready` with caveat | S1 + annotation | "execution-ready (call chain not production-wired)" |
| dsh runtime | `ACTIVE` / `INACTIVE` / `CRASHED` | S2 / S7 / S5 | verbatim |
| Agent adapter (CLI/store only) | `Registered` / `Active` / `Paused` / `Stopped` | S7 / S2 / S3 / S7 | verbatim + "via CLI" source tag |
| Watch | `live` / `stale` / `disconnected` / `reconciling` / `unknown` | S2 / S4 / S5 / S3 / S7 | verbatim + cursor age |
| Session | issued / expiring (<5 min idle) / expired / absent | S1 / S4 / S7 / S7 | "session · <principal> · expires in Nm" |
| Load/display (client) | loading / ready / empty / denied / disconnected / unknown / not-run | — / S1 / S7 / S5 / S5 / S7 / S7 | sentence-form state copy, not just a word |
| Budget/alert | `warning_80` / `exceeded_100` | S4 / S5 (+ "advisory — never blocks" annotation) | verbatim |
| Observation plane | `observed_zero:true` with named negative control | S7 | "none observed (control: <name>)" |

Unmapped-new-state rule: if the daemon ever returns a state word this table doesn't know, the UI renders **S7 + verbatim label + "unmapped state" inspector note**. Never guess a color for an unknown word.

## 4. Signal anatomy (per usage)

| Usage | Composition |
|---|---|
| Row state (lists) | category dot + domain label (mono) + reason code (muted, truncated) |
| Header state (detail hero) | category dot + label + one-line plain reason + source projection + last-updated |
| Queue row (Home) | category dot + object type icon + object label + reason + age + next action |
| Badge (counts) | numeral + category color **and** label on hover/focus; no color-only badges |
| Timeline node | category dot on the rail + event label + time; S5 nodes get a left-edge rule in the content block |
| Evidence-linked completion | S6 check + "Verified" + report digest chip (copyable) |

## 5. Copy grammar

- State labels: daemon verbatim, lowercase, mono font (e.g. `degraded`, `OUTCOME_UNKNOWN`).
- Human reason: one sentence, plain, cause-first: "Secret reference no longer resolves — the Secret Store item is gone." Not "An error occurred".
- Next action: a verb phrase linking to the repair surface: "Rotate key →", "Open doctor →", "View evidence →".
- Forbidden copy: "Success!", "Oops", "Something went wrong", "AI is thinking", any exclamation in operational surfaces, "completed" without its evidence link.

## 6. Accessibility & theming rules

- Dot shapes: S1 filled circle, S2 filled circle (+pulse), S3 hollow circle with dot, S4 filled triangle-dot hybrid (diamond), S5 filled square, S6 check mark, S7 hollow circle. Shape + color + text triple-redundant.
- Minimum contrast: labels 4.5:1; dots 3:1 against their background, in both themes.
- `prefers-contrast: more`: dots gain a 1 px ring in the label color; category backgrounds drop to near-solid.
- `prefers-reduced-motion`: S2 pulse removed (static dot); no other category uses motion.
- Category colors are tokenized (`22` values feed `11-control-plane-design-system.md` §3) and must pass both themes; the *category* (not the hue) is the API of this system.

---

*Consumed by: every page spec (13–21), the design tokens (11), and the component spec (23). Any surface inventing an eighth category or rendering an unmapped state as green fails the UX Review gate (25).*
