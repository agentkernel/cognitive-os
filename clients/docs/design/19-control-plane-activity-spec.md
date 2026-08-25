# 19 — Activity Spec (Evidence Stream)

- Phase 2 (design-only)
- Date: 2026-08-24
- Contract: `06` §3.6, DD-11 (per-object timelines first; unified feed labeled + gated on BD-5), jobs J-I1/J-I2/J-R2. Activity is an **evidence stream**, not a raw log viewer: every row is an authority fact with identity, not a line of text.

---

## 1. Event typing (the seven rendered kinds)

Every Activity row is one of these kinds — the brief's required distinctions, mapped to real sources:

| Kind | Meaning | Real source(s) | Category |
|---|---|---|---|
| **Event** | ordered authority change | O13 audit replay, watch deltas | neutral |
| **Change** | governance mutation applied (binding set, key rotated, tool disabled, memory forgotten) | provider audit + mutation receipts | neutral/S1 |
| **Effect** | an Effect's stage/outcome/reconcile transition | `/task/effects`, O5 | by stage (S-map `22` §3) |
| **Error** | a failure class with reason (verify_failed, probe failure, denied dispatch) | evidence, effects, provider audit | S5 |
| **Intervention** | the owner acted (admission, ack, restore, forget) | admission records, audit | accent |
| **Verification** | independent verifier report | `/task/evidence` | S6/S5 |
| **Acceptance** | terminal acceptance record | `/task/evidence` | S6, evidence-linked |

Kinds are labeled with text + icon, never color alone. Rows carry: time, object identity (task/agent/provider/resource, mono short), kind, one-line fact, and a link to the object's detail at the right section.

## 2. The surface (wave 1, honest composition)

```text
┌──────────────────────────────────────────────────────────────────┐
│ Activity                                                          │
│ Coverage: provider-plane audit + this session's observed task     │
│ events. Not a complete authority event log (BD-5).                │
│ ┌─ kind ▾ · object ▾ · since ▾ ────────────────────────────────┐  │
│ │ 12:44  acceptance  task 9e02… accepted · report r-881…  [view]│ │
│ │ 12:43  verification task 9e02… verified · current       [view]│ │
│ │ 12:41  intervention task a3f9… admitted by local/owner  [open]│ │
│ │ 12:40  effect      task a3f9… e-1 EXECUTED ✓            [open]│ │
│ │ 12:20  change      provider deepseek-main key rotated   [open]│ │
│ │ 12:02  error       task 77be… VERIFY_FAILED             [open]│ │
│ └───────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────┘
```

- **Coverage banner (persistent, quiet):** what the stream is and is not. This is DD-11's honesty contract rendered as furniture — one caption line, always present in wave 1.
- **Composition order:** attention-relevant first is Home's job; Activity is **time-ordered** (investigation reading), with kind/object/since filters.
- **Progressive disclosure:** row → inspector (full fact set: digests, cursor, source projection, related object links) → object detail section. Technical detail is one disclosure down, never hidden, never dumped.

## 3. Per-object timelines

The same row grammar renders per-object slices (task detail Run section, agent dossier Activity, provider account Audit) — one component, filtered by identity. Investigation flows (Flow 7) pivot on these slices; the space-level stream is for "what happened lately across what I can see".

## 4. States

| State | Rendering |
|---|---|
| Empty | "Nothing recorded in this view yet" + coverage banner (the banner explains why empty is plausible) |
| Partial (a source failed) | failed source named in the banner ("provider audit unavailable — <reason>"); remaining rows unaffected |
| Stale | "as of <cursor/age>" + refresh |
| Bounded/truncated | "showing N of M (bounded window)" with the window named |
| BD-5 upgrade | when a unified authority feed lands, the banner is removed and kinds gain cross-domain coverage — a content change, not a redesign |

## 5. What Activity refuses

No infinite raw log (bounded windows with cursors); no fabricated completeness; no chat-like bubbles; no "AI activity" narratives the daemon didn't record; no secret-shaped strings ever (redaction is daemon-side first); no toast-style ephemeral notifications for authority facts (receipts persist here instead — DD-14).

---

*Row grammar and inspector are shared components (`23`); the Home attention queue reuses the kind grammar with priority ordering instead of time ordering.*
