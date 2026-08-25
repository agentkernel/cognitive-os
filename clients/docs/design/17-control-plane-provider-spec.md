# 17 — Provider Spec (Egress Governance)

- Phase 2 (design-only)
- Date: 2026-08-24
- Contract: `06` §3.4, Flows 3–5 in `07`, capability model D4 (the deepest live domain). Provider is a **governance domain, not settings** — its pages lead with state and bindings, not forms.
- Absolute rule: `secret_ref` renders as **present / absent / unknown-state only**. No secret value, no secret-shaped string, no partial key masking ("sk-…a1" is still secret material — banned). Key entry fields are memory-only, non-echoing, cleared on submit, never persisted (ADR-0053).

---

## 1. Accounts master

```text
┌──────────────────────────────────────────────────────────────────┐
│ Providers                                          [+ Add account]│
│ ┌────────────────────────────────────────────────────────────────┐│
│ │ ● deepseek-main   openai_compatible · private-net confirmed     ││
│ │   catalog rev 12 · secret present · probe ok 3h ago      [open] ││
│ │ ◆ backup-openai   openai_official                               ││
│ │   degraded — last discovery failed (auth) 2h ago         [open] ││
│ │ ■ old-account     anthropic_official                            ││
│ │   revoked — secret unresolvable                          [open] ││
│ └────────────────────────────────────────────────────────────────┘│
└───────────────────────────────────────────────────────────────────┘
```

Row anatomy: state dot + name + kind + network scope (when non-public) + catalog revision + secret presence + last probe (class + age). Sort: attention-first (revoked/degraded float above active), then name. The list itself is the triage — an operator sees broken egress before healthy egress.

## 2. Account detail — five sections (secondary nav inside the space)

```text
┌────────────────────────────────────────────────────────────────────┐
│ deepseek-main   ● active   kind openai_compatible · scope private  │
│ secret: present (resolvable) · catalog rev 12 · probe ok 3h ago    │
├──────────┬─────────────────────────────────────────────────────────┤
│ Overview │ OVERVIEW                                                │
│ Models   │  endpoint https://… (redacted) · trust grants:          │
│ Bindings │  private-network confirmed 2026-08-20 (reconfirm on     │
│ Usage    │  scope change) · last probe: reachability ok · 812ms ·  │
│ Audit    │  capability probe: not-run (bounded capability checks   │
│          │  are not exposed) · last discovery error: none          │
│          │  [Rotate key] [Remove key] [Delete account]             │
│          │ MODELS                                                  │
│          │  model              source        in $/M    out $/M     │
│          │  deepseek-chat      discovered    0.27      1.10        │
│          │  deepseek-reasoner  discovered    0.55      2.19        │
│          │  grok-beta          manual        cost_unavailable      │
│          │  [Refresh catalog (bounded probe)] [Add model manually] │
│          │ BINDINGS                                                │
│          │  agent pi → deepseek-chat · rev 4 · callable            │
│          │  agent dsh → deepseek-chat · rev 4 · callable           │
│          │  [Set / change binding]  (preview with CAS revision)    │
│          │ USAGE                                                   │
│          │  30d: 1.2M in · 84k out · $1.94 estimated (unknown ≠ 0) │
│          │  budgets: monthly $10 — 82% (advisory; never blocks)    │
│          │  alerts: 1 unacknowledged [ack]                         │
│          │ AUDIT                                                   │
│          │  account.created · key.rotated · binding.set · …        │
└──────────┴─────────────────────────────────────────────────────────┘
```

Section contracts:

1. **Overview** — state + cause class + trust facts + probe facts + the key/credential actions. Trust grants show their confirmation date and the re-confirm rule ("reconfirmed when scope broadens or HTTPS becomes HTTP"). Capability probe renders `not-run` (honest — the inventory rates it PARTIAL).
2. **Models** — catalog with source honesty (`discovered` vs `manual` — manual visibly less certain), pricing with `cost_unavailable` never rendered as 0/free. Refresh is an explicit bounded probe **[B]** with duration/error-class feedback; failed refresh preserves catalog + binding and says so.
3. **Bindings** — this account's bindings + the set/change flow: agent (qualified set pi/dsh) → model (catalog-filtered, endpoint-servability enforced) → **revision-aware preview** (exact agent instance, account, model, expected CAS revision, consequence for running work) → confirm naming the exact tuple. 409 stale → re-read + new preview (never silent retry). Remove binding names the consequence (agent non-callable; dsh overlay drops the models).
4. **Usage** — token/cost counters with metering source (`provider_reported`/`locally_estimated`/`unavailable`) and the standing annotation "unknown is not zero; alerts are advisory and never block" (BD-8). No charts in wave 1 — honest counters beat decorative graphs (DD-03 logic); a simple period table is sufficient.
5. **Audit** — this account's audit events (action/outcome/detail), newest first, bounded, with the provider-plane-only coverage note (management actions outside the provider plane are not audited over HTTP — inventory §9).

## 3. Credential flows (the strictest surface in the product)

- **Set/rotate/remove key:** password field (non-echoing, memory-only), submit → daemon → SecretStore; the field clears on submit; success feedback is "secret stored (present)" — the value is never re-displayed, re-fetched, or logged. `op` is chosen by current presence (set vs rotate); remove revokes the account (state consequence shown pre-submit: "Removing the key revokes this account; bindings become non-callable").
- **Create account:** the documented order — validate → trust confirmation (when scoped) → persist → secret input → store → probe → verify (Flow 3). A created-but-keyless account sits `revoked` with a repair affordance; that is a designed state, not an error.
- **Delete account:** guarded by active bindings — the confirmation names the blocking bindings; deleting is separate from repairing and never the suggested recovery.

## 4. Failure & recovery presentations

| Failure | Presentation |
|---|---|
| `provider_secret_unresolvable` | S5 + "the Secret Store item is gone" + Rotate-key repair + doctor link |
| Discovery failed (auth/reachability/model_discovery) | S4 + error class + last-good catalog preserved note + retry probe + manual-add path |
| Trust scope violation (409 reconfirm) | the exact flag to reconfirm, with its consequence sentence |
| Binding CAS stale (409) | "changed under you" + re-read + new preview |
| Model/endpoint mismatch | `PROVIDER_MODEL_ENDPOINT_MISMATCH` named + catalog repair path |
| SecretStore backend unavailable (503) | account actions degrade to S7 + System/doctor link; no key fields render at all |

## 5. What this domain refuses

No fallback/routing/load-balancing UI (policy non-goals); no per-request override; no OAuth/browser-login concepts; no hard-budget "enforce" toggle (budgets observe-only, BD-8); no raw provider responses; no embedded-credential or arbitrary-header affordances (daemon rejects them — the UI never offers them).

---

*The binding flow is shared with the Agent dossier (contextual entry, agent preselected). dsh "Apply to running panel" lives on the binding row when the runtime gate passes (state ACTIVE, process alive, model in catalog, expected revision) — shipped behavior, kept.*
