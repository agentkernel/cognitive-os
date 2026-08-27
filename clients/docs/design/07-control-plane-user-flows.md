# 07 — Control Plane Core User Flows

- Status: adopted Personal 2.0 flows; historical P7-T05 flows retained
- Updated: 2026-08-27
- Method: stark `ux-design` flow mapping (entry → first action → decision → feedback → success → recovery), `task-ergonomics` contracts for the three highest-risk flows, `usability-pattern-matrix` pattern choices. IA per `06`; capabilities and honesty classes per `03`. Every flow names its backend dependencies and its not-run branches.
- Convention: **[A]** class-A governed mutation · **[B]** class-B safe action · **[C]** deferred verb (renders not-available + CLI path) · **[D]** never rendered.

## Personal 2.0 core flows

The flows below are the adopted target. Existing Flow 0–10 later in this file
remain useful current-backed subflows, especially task admission, Provider
binding, evidence inspection, and degraded recovery.

### T1 — Open Personal and resume

`desktop entry -> Home -> recent Agent conversation or highest-priority Work
item -> Agents/Work -> useful next action`.

Home has no marketing hero and no metric wall. It shows only sourced,
actionable rows. If the data needed for a complete resume view is unavailable,
the row states its coverage; it never invents recency or progress.

### T2 — Install or connect an Agent in at most three steps

1. Choose a signed catalog entry or a supported existing installation.
2. Review source/signature, Adapter capability matrix, requested account and
   resource boundaries; confirm the current-backed operation.
3. Connect and land directly in that Agent's conversation.

Success is the **first real chat response**, not "installed" or "connected".
Failure preserves selections and offers the exact recovery. Unsupported catalog,
install, registration, lifecycle, and conversation operations render
`Requires-backend`; the current UI must not offer fake buttons. Disconnect keeps
installation/history according to source semantics; uninstall previews retained
data and is a separate destructive choice.

### T3 — Converse natively, then choose Manage with Personal

`Agents -> select Agent -> embedded native conversation/history -> ordinary
messages remain native -> Manage with Personal -> daemon preview ->
confirm/admit -> Work`.

The Adapter provides a common message/history projection and capability matrix.
Vendor-specific metadata/artifacts may appear in display-only native slots;
actions always use Control Plane-owned controls backed by typed capability
semantics. Manage with Personal names what will become durable, shows
losses/unsupported fields, and never treats hidden reasoning or chat text as a
Plan. Goal/Plan/attempt creation is `Requires-core + Requires-backend` where
public machine semantics are needed.

### T4 — Supervise managed and multi-Agent work

`Work -> Goal -> current Plan revision -> Tasks/attempts -> one provenance
timeline -> inspect Effects/Evidence -> revise Plan or resolve blocker`.

The daemon assigns roles and handoffs. A Plan revision never edits history;
prior revisions stay inspectable. Multi-Agent disagreement is shown as
candidate alternatives with source identity. Only recorded authority state can
show progress; missing denominators render unknown, not a percentage.

### T5 — Add an account in Account Hub

`Settings -> Account Hub -> choose acquisition tier -> consent/review ->
daemon-owned credential path -> verify account -> inspect models/quota/cost ->
optionally bind Agent`.

Tiers: OAuth/subscription, API key, user-directed import, custom gateway.
Current implementation supports only its verified API-key/custom endpoint
subset. All other tiers, subscription semantics, import readers, quota and cost
facets are `Requires-backend`. Import follows
[ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md):
exact source and target shown before read, per-source consent, retention default,
optional secure deletion, redacted audit, no secret return to the UI.
Target routing precedence is global default -> Agent override -> conversation
override. A current native session changes only after explicit rebind/restart;
all override and rebind semantics are `Requires-backend` beyond today's fixed
Agent binding.

### T6 — Browse MCP as a first-class Library family

`Library -> MCP -> server/package/connection/capability/binding/health/
quarantine -> candidate mappings into Tool/Context/Skill -> typed daemon
workflow`.

MCP discovery does not imply authorization. MCP plus rules does not control a
host Agent session. Every unsupported operation is `Requires-backend`; current
Tool/MCP fixture facts must not be promoted into a product control.

Integration preference is `vendor-native API -> managed Adapter ->
MCP-cooperative`. MCP plus rules is used only as a declared cooperative
fallback. After first authorization, automatic reconciliation is allowed only
for the exact unchanged grant and client/target/trust boundary. Permission
expansion, a new client, trust-boundary or target expansion, or an incompatible
update requires a fresh daemon preview and confirmation. Reload/restart is a
separate supported action with its own capability condition and receipt; it is
never hidden inside auto-reconciliation.

### T7 — Resolve a federated resource conflict

`Library/Work conflict -> compare Personal and Agent-native versions with
provenance -> ask Agent Shell for a resolution candidate -> daemon preview ->
confirm -> persist Intent/Effect -> writeback -> verify receipt`.

The Shell suggestion is never authority. If observation or writeback lacks a
typed backend path, the UI explains the target and stops before an action
affordance. No optimistic success and no fake progress.

### T8 — Investigate in one provenance timeline

`Activity or object detail -> filter Native/Observed/Governed/Verified ->
select item -> inspector shows source, identity, freshness and coverage ->
pivot to Agent/Work/Library/Settings`.

Native vendor events, process observations, daemon governance facts, and
independent verification remain visually and semantically distinct even when
time-aligned.

### Shared recovery contract

Every flow defines empty, loading, partial, stale, denied, disconnected,
conflict, error, success, and long-running states. User input is preserved.
Long-running views show only source-backed phase/progress and controls that
really exist. Keyboard-only and reduced-motion/transparency paths are complete.

---

## Flow 0 — Session bootstrap (gates everything)

- **Entry:** any route without a live session → inline gate (not a redirect; the destination stays visible behind the gate, preserving orientation).
- **Path:** paste bootstrap secret (non-echoing, memory-only) + principal (default `principal://local/owner`) → daemon issues channel-scoped bearers → land on the originally intended route.
- **Feedback:** which channels were issued; expiry shown in the status strip.
- **Recovery:** wrong secret → stable error class, no lockout theater, secret field cleared; daemon unreachable → disconnected state with retry.
- **Known cost (recorded, not hidden):** sessions are memory-only and die on reload; re-paste is required. BD-9 tracks any ergonomic improvement; the flow never offers "remember me" — it is architecturally forbidden (ADR-0053 §3).
- **States:** gate, issuing, denied, disconnected, issued, expired (mid-session expiry → inline re-gate with the current route preserved).

## Flow 1 — Morning supervision (the daily loop; rank-1 job)

- **Entry:** `#/` (Home).
- **First screen:** readiness line (system/db/secret/provider/daemon/pi), needs-attention queue, current-work strip.
- **First action:** scan the attention queue. Each row = what / why (reason code) / since when / next action.
- **Decisions:** (a) nothing needs me → leave (success = calm exit); (b) a row needs reading → open its inspector/detail; (c) a row needs governance → follow into the owning flow (binding repair → Flow 5; provider degraded → Flow 4; unknown-outcome effect → Flow 7).
- **Feedback:** every drill-in keeps the queue context (master/detail; back returns to the same scroll/filter).
- **Recovery:** readiness degraded → one click to System/doctor detail with the failing component pre-expanded.
- **States exercised:** ready/empty-attention (designed calm state), partial (some components not probed — labeled), stale (cursor age), disconnected.
- **Ergonomics contract:** returning operator must reach "the one thing that needs me" in ≤2 interactions; queue order is priority-then-recency, stable across refreshes.

## Flow 2 — Verify a finished task (the trust flow; rank-2 job)

- **Entry:** Home current-work strip, Work inventory, or deep link `#/work/:taskRef`.
- **Path:** task detail → Overview shows lifecycle state **and** verification disposition side by side (never merged) → Evidence section: verification report ref/digest, currency flag, acceptance record, artifact links.
- **Decision:** evidence current + acceptance present → done; evidence missing/stale → the surface says exactly that (`artifacts_current:false`, or evidence absent) with the owning reason.
- **Recovery:** evidence not found (404 `TASK_EVIDENCE_NOT_FOUND`) → designed state: "no terminal evidence recorded for this task" + what that means + link to Run timeline to see where it stopped.
- **Anti-pattern guard:** the words "completed/complete" never appear without the acceptance record link; agent-side success signals (process exit, stream close) appear only in the observation lane, labeled as observation.
- **States:** verified / verification failed / unknown outcome / evidence missing / still running (with watch state).

## Flow 3 — Govern a new Provider account end-to-end [A/B]

- **Entry:** Providers → "Add account".
- **Path (documented order, kept):** display name + kind → endpoint (openai_compatible only) → trust checkboxes (private-network / insecure-http, each with its consequence sentence; confirmation checkbox required when scoped) → create **[A]** → secret entry (memory-only, set vs rotate chosen by current secret presence) **[A]** → bounded probe (models/refresh) **[B]** → catalog review (source/pricing honesty) → (optional) manual model add **[A]**.
- **Feedback:** each stage's result with stable error classes; probe failure → account `degraded`, catalog preserved, next actions (retry probe / add model manually / check key).
- **Success:** account `active` with a discovered catalog; the "bind an agent" next action offered (lands in Flow 5 with account preselected).
- **Recovery:** key rejected → field cleared, no retention, error class + guidance; trust scope error → exact flag that must be reconfirmed (409 semantics).
- **States:** per-stage loading/empty/error/denied; the flow is resumable — an account created but never keyed sits in the list as `revoked` with a repair affordance (this is the shipped behavior, kept).

## Flow 4 — Repair a degraded Provider

- **Entry:** Home attention row "provider degraded / secret unresolvable", or Providers list status.
- **Path:** account detail → Overview names the exact cause class (`provider_secret_unresolvable` vs discovery failure vs trust) → offered repairs: rotate key **[A]**, re-run probe **[B]**, reconfirm trust **[A]**.
- **Feedback:** repair result inline; account status re-projected.
- **Recovery:** repair fails → cause + CLI equivalent + audit link. Delete is separate, binding-guarded, and never the suggested repair.
- **Rule:** a successful probe never upgrades an account to `active` by itself; status vocabulary is daemon-owned (the UI never computes its own health).

## Flow 5 — Bind / rebind an Agent to a Provider model [A]

- **Entry:** Provider detail → Bindings section, or Agent dossier → Binding section (both land on the same flow, preselected from origin).
- **Path:** select agent (pi/dsh — the qualified set) → account → model (catalog-filtered; endpoint-servability enforced daemon-side) → **revision-aware preview**: exact agent instance, account ID, model ID, expected revision (CAS), trust state, consequence for running work → confirm checkbox naming the exact tuple → submit.
- **Feedback:** new revision + dispatchability (callable/blocked) shown; on 409 `PROVIDER_BINDING_REVISION_STALE` → the surface says the binding changed under you, re-reads, and re-issues the preview (never auto-retries a stale CAS).
- **Deliberate negatives (kept from shipped UI, redesigned as education not traps):** fallback and per-request override are not options; the flow states the policy once, plainly, instead of offering rejectable checkboxes.
- **Recovery:** model not in catalog → manual-add path; binding blocked (account revoked) → repair-the-account link.
- **dsh apply:** where the dsh runtime is ACTIVE, "apply to running dsh" is offered with its own gate facts (runtime state, process alive, catalog membership, expected revision) and its 4 s acknowledgement semantics; failure → exact error class.

## Flow 6 — Create and watch a governed Task [A]

- **Entry:** Work → "New task".
- **Path:** objective → record → interpretation review (objectives/constraints/forbidden/assumptions; **ambiguities and information gaps are first-class review content**, `clarification_required` is a designed branch, not an error) → preview: exact contract (scope, conditions, budget, deadline, allowed tools/domains, agent/binding, context policy) with digest → admit (principal-bound).
- **Feedback:** `task_ref` + contract digest/epoch; land on task detail, watch attached (attach/detach is observation-only and says so).
- **Success:** task admitted; the Run timeline begins populating (authority lane: transitions; observation lane: process facts).
- **Recovery:** preview rejected → which field/why; admission rejected (409) → stale versions → new preview; principal mismatch (403) → session explanation.
- **Honesty bounds:** wave 1 offers the workspace-search draft type (shipped capability); the flow is designed so additional draft types slot in without restructuring. Task cancel/pause render **[C]** with the CLI path (BD-1).
- **States:** recording / interpreting / clarification_required / previewed / admitting / admitted / watch live-stale-disconnected / detached.

## Flow 7 — Investigate a blocked or failed Task

- **Entry:** Home attention row, Work inventory filter (blocked/failed/unknown-outcome), or deep link.
- **Path:** task detail → Run timeline: the last authority transition + the effect(s) in `OUTCOME_UNKNOWN`/`VERIFY_FAILED` → Effects section: stage, outcome class, reconcile class, `fixed_post_state_ref` presence → Evidence section: what verification exists.
- **Decision:** reconcile-pending → wait/watch (the daemon reconciles; the UI shows `pending_reconciliation` with age); failed with cause → the cause class and the owning surface (provider? tool quarantined? budget stop?) → route to repair flow.
- **Recovery:** observation plane families (o2/o4/o5/o13) available per task for authorization/scheduler/effect/audit-replay facts, rendered as bounded diagnostic cards with their denominators and negative controls.
- **Rule:** the investigation surface never proposes a verdict; it assembles authority facts and lets the operator conclude. "Retry" is **[C]** (BD-1).

## Flow 8 — Degraded-system recovery (System Operator mode)

- **Entry:** status strip shows degraded/blocked, or daemon unreachable mid-session.
- **Path:** System → readiness detail: failing component + facts + guidance → doctor detail (with placeholder sub-sections labeled as not-probed) → recovery actions are **[B]** links/guidance (the daemon repairs via CLI/host, not the browser); backup/restore live under Stewardship.
- **Locked-secret case:** provider/agent unavailable while secret backend locked → the surface explains the state and the unlock path (TTY/systemd per install mode) without offering any secret field it cannot honor.
- **Rule:** the Control Plane degrades to a diagnostic client, never to a dead page (inherited requirement, `web-ui-design.md:36-39`).

## Flow 9 — Curate cognitive resources (Power User mode)

- **Skill:** Resources → Skills → import (local path, bounded) → inspect revision (digest, provenance, compatibility) → bind (CAS) / revoke (with reason). Content≠permission annotation standing.
- **Memory:** Resources → Memory → list (envelope honesty) → object explain (provenance chain: candidate→decision→object; scope; expiry) → remember (governed, retention-capped) / forget (tombstone consequence stated). Search and proposal-review render per BD-6 state.
- **Tool:** catalog → descriptor (risk, digest, readiness with the readiness≠wired annotation) → enable/disable **[A]**; quarantine **[A]** with its one-way semantics stated; revoke **[A]** as terminal.

## Flow 10 — Budget alert handling

- **Entry:** status strip alert count / Home attention row / Providers → account Usage.
- **Path:** alert (80% warning / 100% exceeded, deduped) → usage detail (tokens/cost, unknown-as-unknown, metering source) → acknowledge **[B]**.
- **Honesty frame:** budgets are **observe-only** — the UI says alerts never block or reroute; there is no "enforce" toggle to fake (BD-8).

---

## Task ergonomics contracts (the three highest-risk flows)

### Contract A — Flow 5 (rebind agent) · risky, occasional

```md
- Core task: change an agent's fixed provider binding
- User mode: returning power user; risk: security/cost-impacting, reversible by re-binding
- Success metric: zero wrong-binding admissions; stale-CAS recovery without support
- Cognitive load: exact IDs/versions visible at confirm; nothing recalled from memory
- Control model: primary = Confirm binding (with exact tuple + expected revision);
  secondary = change selection, cancel; no undo (re-bind is the undo) — stated
- Speed path: agent/account preselected from origin; model list catalog-filtered
- Error prevention: catalog membership + endpoint servability + CAS + confirm naming
  the exact tuple; consequences for running work shown pre-submit
- Recovery: 409 stale → re-read + new preview; revoked account → repair link
- State matrix: loading catalog / empty catalog (manual add path) / stale CAS /
  denied / disconnected / success (new revision shown) / not-run (capability probe)
- Evidence plan: rebind pi twice concurrently → one wins, one gets the stale path
```

### Contract B — Flow 6 (create governed task) · guided, moderate risk

```md
- Core task: admit exactly the intended work
- User mode: operator; risk: bounded external mutation via allowed tools
- Success metric: admitted contract == reviewed preview (digest shown at both steps)
- Cognitive load: preview is the single review object; ambiguities resolved before admit
- Control model: primary = Admit (digest-bound); secondary = edit objective,
  resolve ambiguity, abandon (abandon is always safe pre-admission)
- Speed path: draft type preselected (wave 1: workspace-search); task_ref copyable
- Error prevention: digest-bound admission; stale versions → new preview;
  principal bound server-side
- Recovery: preview rejected → field-level cause; admission rejected → versions re-read
- State matrix: per-stage loading / clarification_required / previewed / admitted /
  watch live/stale/disconnected / detached / cancel [C] honest
- Evidence plan: create → admit → watch → verify completion via Flow 2
```

### Contract C — Flow 1 (supervision loop) · daily, high-frequency

```md
- Core task: decide in seconds whether the system needs me
- User mode: returning operator; risk: low (read-mostly) with rare high-stakes pivot
- Success metric: ≤2 interactions to the highest-priority attention item
- Cognitive load: queue rows carry what/why/when/next-action; no recall
- Control model: primary = open top attention item; secondary = per-row next action
- Speed path: stable order, keyboard navigation, deep links, ⌘K to any object by ID
- Error prevention: read-first; the only actions on Home are class-B
- Recovery: degraded readiness → System detail pre-expanded; disconnected →
  last-good labeled
- State matrix: ready-empty (calm) / attention items / partial probes / stale /
  disconnected / denied
- Evidence plan: seed one degraded provider + one unknown-outcome effect → both
  surface with reason codes and correct next actions
```

---

*Pattern selections logged: Home = status board + priority queue (not metric dashboard); Work/Agents/Providers/Resources = master/detail + inspector; Task creation = guided setup with review step; Activity = timeline; System = searchable grouped settings archetype for stewardship. Full pattern rationale in `05`/`06`; Apple treatment in `09`; agent-supervision treatment in `08`.*
