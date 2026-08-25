# 24 — Visual Direction

- Phase 2 (design-only)
- Date: 2026-08-24
- Authorities: `apple-design` (primary), `09` (product translation), `frontend-design` (auxiliary — distinctiveness discipline, used under Apple restraint and operator density). Tokens: `11`. State language: `22`.
- This is the visual thesis for the whole surface, applied concretely to the three core pages (Home, Work, Work Detail). It is a *direction*, not final art: final palettes/hues are produced and contrast-verified in the visual phase against `11` §2's gates.

---

## 1. Thesis: "The Instrument"

The Control Plane should feel like a **precision instrument owned by one person** — a calm, physical, legible surface where every value is real. Not a cockpit from a movie, not a SaaS console, not an AI startup page. The reference temperament is Apple's own pro tools (the quiet of Finder, the density of Xcode's inspectors, the honesty of Terminal) crossed with the discipline of a well-made hardware instrument: a bezel that tells system truth, a face that never twitches, and markings that mean exactly one thing.

Three words govern every choice: **Calm** (stable frame, neutral field, nothing performs), **Precise** (mono for identity, exact vocabulary, aligned numerals), **Dense** (information per glance, disclosed in layers).

## 2. The signature: the Authority Record

Every product needs one element it is remembered by. Ours is the **dual-lane Run timeline with its evidence block** (`15` §3, §4): authority facts on a solid rail, observations on a quiet hollow rail, and the verification/acceptance record as the terminal artifact. No generic admin tool has it because no generic tool distinguishes *what the authority recorded* from *what was observed* — that distinction is this product's reason to exist, made visible. The visual phase spends its craft budget here first: node grammar, lane separation, gap spans, digest chips, the evidence block's quiet finality.

Supporting signature texture: **the mono ledger** — digests, refs, epochs, cursors set in tabular mono with copy affordances, giving the whole surface the feel of a verifiable ledger rather than a feed.

## 3. How the three core pages speak one language

| Element | Home | Work | Work Detail |
|---|---|---|---|
| First read | readiness line (one line of truth) | the inventory (stable columns) | the header (identity+state) |
| Center of gravity | needs-attention queue | master list | Run timeline |
| Proof | recent evidence rows | evidence marks on rows | Evidence section |
| Motion | none beyond disclosure | selection instant; changed-dot | timeline append without scroll theft |
| Empty | "Ready. Nothing needs you." | "No work observed yet." + New task | designed per-section states |

Shared grammar (the sameness that makes it one language): same strip, same state chips, same mono ledger chips, same hairline+background-step depth, same one-primary-action discipline, same cause-first copy voice.

## 4. Light and dark

System-following, both first-class (DD-12). Neither is "the brand": the hierarchy is the brand. Dark is the operator's low-light environment; light is the daytime desk. State hues are tuned per theme to hold category contrast (dots 3:1, labels 4.5:1); `prefers-contrast: more` trades tints for solid+border. The shipped dark-only theme was a starting point, not an identity.

## 5. What we refuse (the generic-failure checklist, applied to ourselves)

1. **No cream+serif+terracotta editorial look** (AI-default #1) — wrong register for an instrument.
2. **No near-black + acid accent** (AI-default #2) — the shipped dark theme already drifts here; the accent is dialed to a system blue (`accent`, `11` §2), and state colors carry meaning instead.
3. **No broadsheet hairline-newspaper look** (AI-default #3) — density here is tabular, not editorial.
4. **No glassmorphism / Liquid Glass cosplay** — one floating material, only for true overlays (`11` §4).
5. **No dashboard furniture** — no KPI tiles, sparklines, donut charts, trend arrows (DD-03).
6. **No AI-magic signifiers** — no sparkles, no gradient "thinking" blobs, no robot iconography.
7. **No marketing voice** — copy is cause-first operator language (`22` §5).

## 6. The one justified risk

**We let the state vocabulary own the color budget almost entirely.** In most products, brand color dominates; here, the only saturated hues on a resting screen are *state* — a green that means ready, a red that means blocked, a blue that means live. The risk: the product could read as "colorless admin". The justification: in a supervision tool, color that means nothing is noise that trains operators to ignore color that means something. The payoff is that a single red square on a calm field is instantly, correctly alarming — the interface's signal-to-noise ratio *is* the aesthetic. This is the deliberate, defensible risk; everything around it stays quiet and disciplined (Chanel rule applied: we removed the decorative accent everywhere else).

## 7. Craft bar (the details that make it feel inevitable)

- Numerals tabular and right-aligned in data columns; timestamps show age + absolute on hover/focus.
- Digest chips truncate medially (`b91e…77`), copy full value, never wrap.
- Hairlines instead of shadows; background steps (canvas→surface→raised) instead of cards.
- Selection is instant and local; authority facts load into stable frames — the interface never lies about what is known yet.
- Press feedback on pointer-down everywhere (`motion.instant`); nothing waits for release to acknowledge the human.
- Every page survives its own states (empty/partial/stale/denied/disconnected/not-run) with the same typography and spacing as its happy path — states are designed, not defaulted.

---

*Gate: this direction is reviewed in `25-control-plane-ux-review.md` (Apple Review §4). If the visual phase produces anything that could be re-skinned into a generic admin by swapping the logo, it has failed this direction (operational-dashboard reference: domain-specific signals or failure).*
