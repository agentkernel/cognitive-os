# 08 — Control Plane Agent UX

- Status: adopted Personal 2.0 Agent UX; historical framework analysis retained
- Updated: 2026-08-27
- Method: `ai-agent-ux` (AUTONOMY framework, autonomy dial, trust ramp, undo architecture), `ai-trust-transparency` (GLASS, calibrated trust, explanation layering), `ai-error-resilience` (RECOVER, error taxonomy, blast radius). Applied to the supervision reality of CognitiveOS: **the daemon governs; agents act; the owner supervises.** CognitiveOS reality overrides the frameworks where they conflict (e.g. these frameworks assume the UI can stop the agent; here the honest answer is class-C).

## Personal 2.0 Agent UX

### Global Agent Shell

The desktop always offers a candidate-only global Agent Shell. It explains
current daemon/adapter facts, compares sources, proposes a next action, asks the
daemon for an authoritative preview, and returns focus to the affected object.
It is not the vendor-native conversation surface and does not flatten Agents
into one generic chatbot.

The **Agents** workspace owns each embedded native conversation, composer,
attachments, history, and Manage with Personal entry. Each Adapter supplies:

- a common conversation/history projection;
- capability flags with source and freshness;
- declared native slots for display-safe vendor metadata and artifacts;
- explicit unsupported states rather than emulation.

Adapter-specific render slots are display/artifact renderers only. They cannot
inject actions, buttons, executable markup or scripts, credentials, or
authority-shaped state. Every vendor-specific action uses a Control Plane-owned
control whose behavior is backed by typed capability semantics. If that action
is not delivered, the Control Plane renders `Requires-backend` explanatory
content rather than asking a render slot to simulate it.

### Capability matrix dimensions

The matrix keeps three independent axes:

| Axis | Allowed values | Meaning |
|---|---|---|
| Runtime condition | `Supported` / `Unsupported` / `Unavailable` / `Unknown` | what the current adapter/runtime can establish now |
| Delivery status | `Now` / `Requires-backend` | whether Personal currently delivers the projection or action |
| Support path | `vendor-native` / `managed-adapter` / `MCP-cooperative` / `observable-only` / `unqualified` | how the capability is supplied and what claim ceiling applies |

`Requires-core` may annotate a future public contract dependency, but it never
replaces delivery status. `Unsupported` is not a synonym for
`Requires-backend`; `Unavailable` is a runtime blockage, not a roadmap status;
and `unqualified` never exposes an action.

The Agent workspace labels the active Agent and account/model route. Switching
Agents never merges histories or credentials. Native history remains
source-owned and retains provenance. The global Shell may explain that state,
but it neither sends a native turn nor owns the conversation.

### Native conversation versus Manage with Personal

Native conversation is the default low-friction path. **Manage with Personal is
explicit** and is required before a conversation becomes daemon-governed work.
Its daemon preview identifies the proposed Goal, Plan inputs, Context, resource
permissions, participating Agents, budget and known losses. The daemon—not the
Agent or client—creates durable Goal/Plan/Task authority. This target boundary
is `Requires-core + Requires-backend` where public machine semantics are
needed.

### Multi-Agent supervision

For managed work, the owner sees:

1. daemon-issued participant roles and authority bounds;
2. source-preserving candidate contributions;
3. explicit handoffs and dependencies;
4. disagreements as alternatives, never hidden synthesis;
5. Tasks/attempts and Effects under their real state machines;
6. independent verification and acceptance.

Agents never transfer leases, credentials, host-session control, or completion
authority to one another. A failure in an upstream attempt blocks or
re-evaluates dependents through daemon state; the UI does not infer cascading
progress.

### Intervention and trust

- Current-backed preview, confirmation, revoke, rebind, disable and detach
  actions use their real daemon semantics.
- Target-only pause/cancel/lifecycle/orchestration controls are
  `Requires-backend` explanatory slots, not active or disabled-looking buttons.
- Progress is a recorded plan/task/attempt state or unknown. Spinners communicate
  loading only; they never imply the Agent is advancing.
- Explanations telescope from beginner summary to full inspector. They show
  source, evidence, limits and alternatives—not hidden chain-of-thought.
- The one timeline marks `Native`, `Observed`, `Governed`, and `Verified`.
- Disconnect and uninstall are distinct. Uninstall requires a retained-data and
  blast-radius preview; disconnect preserves the installation unless the
  source's typed semantics say otherwise.

### Federated-resource assistance

The Shell may explain a conflict and propose a resolution. It cannot directly
write Personal or Agent-native resource state. A real writeback follows daemon
preview -> owner confirmation where required -> persisted Intent/Effect ->
dispatch -> verification. MCP capabilities do not grant control of the host
Agent session.

The current P7-T05 SPA has no embedded conversation, Goal/Plan, multi-Agent, or
federated-writeback projection. Those target elements remain
`Requires-backend`; current agent dossiers continue to display only the
evidence recorded in [Agent Reality Map](31-agent-reality-map.md).

---

## Historical 2026-08-24 framework application

## 1. The inversion, stated for this product

Classical UX: the user acts, the system responds. Agentic UX: **the system acts, the user supervises** (`ai-agent-ux` core principle). CognitiveOS adds a third actor that changes everything: the **daemon authority** between the owner and the agents. The owner never supervises agents directly — they supervise the *authority's record of* what agents proposed, were admitted to do, did, and had verified.

This is the product's deepest UX asset: **supervision here is evidence-based, not vibes-based.** The design's job is to not squander it.

## 2. AUTONOMY framework, mapped to reality

| Letter | Framework question | CognitiveOS answer today | Design consequence |
|---|---|---|---|
| **A** Action Preview | Can the user see what the agent will do before? | Yes — better than the framework imagines: server-issued, digest-bound preview + admission (`/task/preview` → `/task/admit`) | The preview is the centerpiece of task creation (Flow 6); mutations elsewhere show exact IDs/versions/CAS before confirm |
| **U** User Override | Can the user stop/modify/redirect mid-execution? | **Honestly limited**: no task cancel/pause HTTP, no agent lifecycle HTTP (BD-1/BD-2). What exists: detach observation (never cancels), binding removal (stops future egress), tool disable/quarantine (stops future dispatch), key removal (revokes provider) | Override UI is designed as *real levers on future work* + honest class-C rendering of missing verbs + CLI path. **The stop-button rule applies with a twist: since instant stop does not exist over HTTP, the UI says so before the run starts** (admission screen states the control set) |
| **T** Tiered Authority | Does freedom scale with risk? | Yes — product-native: Tier 0 silent / Tier 1 first-use capability lease / Tier 2 explicit confirmation (`product-design.md:107-110`); workspace bounds; tool exposure per task; fixed provider binding as egress cap | Tier semantics visible wherever an action is gated; the autonomy dial is **structural** (what the contract allows), not a settings slider |
| **O** Observable State | Can the user see what it's doing now? | Partial — task watch (process-local, empty snapshot), observation plane O2–O13, dsh runtime snapshot; no live deltas on resource watch (BD-4) | Run timeline with authority/observation lanes; watch state (live/stale/disconnected) always visible; never a spinner pretending to be progress |
| **N** Narrated Reasoning | Does it explain why? | Structurally: interpretation candidates carry objectives/constraints/assumptions/**ambiguities/information gaps**; but raw agent reasoning is not an operator API fact | Surface interpretation + ambiguities as first-class review content; never fabricate a "reasoning" narrative the daemon didn't record |
| **O** Outcome Verification | Can the user verify completion? | Yes — the product's signature: independent verification + acceptance, terminal evidence API | Evidence-linked completion everywhere; "completed" is always a link, never a badge (Flow 2) |
| **M** Memory of Actions | Complete searchable log? | Partial — events/effects/evidence per task; provider-plane audit; **no unified cross-domain feed, no management-action audit beyond providers** (BD-5) | Activity is honest about its coverage; per-object timelines first |
| **Y** Yield to Humans | Does it know when to stop and ask? | Yes — `clarification_required` interpretation status; blocked/degraded states with reason codes; admission required before execution | Clarification is a designed branch of task creation, not an error; blocked states always name the blocker and the next action |

## 3. The autonomy dial, translated

The framework wants a user-facing dial. In CognitiveOS the dial is not a preference slider — it is the **contract itself**: allowed tools, workspace scope, budget, deadline, max retries, binding. Design consequences:

1. **Task creation surfaces the dial as the preview.** "What this task may do" is a readable section of the preview (tools, scope, budget), not fine print.
2. **Standing autonomy = bindings + tool availability + workspace policy.** These live in Providers/Resources/System governance surfaces; changing them is class-A with consequence copy.
3. **Dialing down is instant and honest where the backend allows** (disable a tool, revoke a binding, remove a key — all real verbs). Dialing *up* requires the full preview/confirm path. Where instant-down would require the missing lifecycle verbs, the UI says exactly that (BD-1/BD-2) — this satisfies the framework's cardinal rule truthfully rather than theatrically.
4. **Trust ramp:** new agent registrations start at minimal capability by architecture (content ≠ permission; nothing is ambient). The UI mirrors this: fresh objects render their *zero-capability* state explicitly ("installed, no binding, no tool exposure — this agent can do nothing yet" is a designed empty-capability state, not a warning).

## 4. Trust calibration (GLASS applied)

The goal is **calibrated trust**, not maximum trust:

- **Ground in sources:** every state claim is traceable — projections carry their source route and cursor; evidence carries report refs/digests; the inspector always answers "says who?". This is the L4 auditable-trace posture, appropriate because the audience is the system's owner-operator (professional decision support row of the confidence matrix).
- **Layer explanations:** 5-second (row state + reason code) → 5-minute (inspector: projection facts, binding/epoch context) → 50-minute (detail: full transitions, effects, evidence, observation families). Never all at once; never buried.
- **Advertise limitations:** not-run/not-backed/unknown are visible vocabulary, and each names its backend dependency (BD-n). The system's *known unknowns* are part of the display (e.g. "observation covers this task only", "budgets are advisory").
- **Show confidence:** this product trades probabilistic confidence for **authority disposition** — verification status, reconcile class, currency flags. Where genuine uncertainty exists (manual models, stale watch, unprobed doctor sections), it is labeled. No percentage theater.
- **Support override:** correction paths are governance actions (rebind, revoke, forget, disable) with preview/confirm — friction matched to risk (task-ergonomics risk table).

**Trust Erosion Event preparedness:** the design assumes the *system* will sometimes report unknown-outcome effects, failed verification, degraded providers. Each has a designed state with cause + next action (Flows 4, 7). A control plane that surfaces its own uncertainty *builds* trust; one that smooths it over destroys the product's reason to exist (J-F1..J-F4).

## 5. Error and failure design (RECOVER applied)

Error taxonomy for this surface, with the product-honest response:

| Error type | Local instance | Response pattern |
|---|---|---|
| Confident hallucination | "Completed" without acceptance | Eliminated structurally: completion = evidence-linked disposition only |
| Stale knowledge | stale watch cursor, cached projection | stale state with cursor/age + refresh path; disconnected never fabricates finals |
| Context misread | interpretation ambiguities | `clarification_required` branch; owner resolves before admission |
| Partial answer | truncated projections (`transitions_truncated`, samples caps) | truncation flags visible with denominators |
| Formatting/envelope | three daemon error shapes | one client normalization layer; UI-level error vocabulary is the display vocabulary (§3 of `03`) |
| Refusal overreach | fail-closed denials (403/409 classes) | denial states name the policy (channel binding, CAS, exposure digest) and the legitimate path |
| Confidence inversion | probe success ≠ capability; connection ≠ usable | documented in place ("a successful network connection never upgrades an account to usable") |

**Blast radius audit (per action class):**

| Action | Worst case | Containment |
|---|---|---|
| Admit task | bounded external mutation via allowed tools | preview + budget + workspace bounds + effects reconciliation + independent verification |
| Set/rotate key | wrong key → provider egress fails closed | daemon-side store; probe verifies; account degrades visibly |
| Rebind agent | wrong model/account for future work | CAS + preview + one-fixed-binding; re-bind reverses; audit row |
| Remove binding/key | agent non-callable | explicit non-callable state + repair link; guarded by active-work copy |
| Revoke tool/skill | future dispatch denied | reason required; revocation visible; (tool quarantine is one-way — stated before confirm) |
| Restore backup | overwrite current authority data | digest/compat preflight; 409 classes designed into copy; secret-exclusion stated |
| Forget memory | durable tombstone | consequence stated; tombstone prevents resurrection |

**Fallback hierarchy for the whole surface:** degrade to diagnostic client (Flow 8) → transparent limitation (not-run with dependency) → alternative path (CLI verb named) → graceful exit (designed dead-end states with next action). The cliff-edge "Something went wrong" is banned.

## 6. Multi-agent coordination (P6 headroom, honestly bounded)

Multi-agent is `not-started` (P6-T01..T04). The IA does not design its UI; it **reserves the grammar**: agent identity labels on every task/run/event row (so "which agent did what" is answerable when P6 lands); Work list grouping by agent as a view option; no cross-agent orchestration UI. If P6 ships, disagreement surfacing and handoff visualization become a design phase of their own against real backend objects — not speculation now.

## 7. Agent-UX review checklist applied to the recommended IA (the brief's §17)

| Question | Answer in this design |
|---|---|
| Does the user always know what the agent is doing? | Work/Run detail: current authority state + last observed fact, watch state. Where the backend can't say (Pi live activity), the surface says what is knowable and what isn't (BD-2) |
| Why it's doing it? | Intent chain (record→interpret→preview) on the task; ambiguities explicit |
| Which step? | Lifecycle transitions + effects stages; no fake stepper — the timeline shows *recorded* steps only |
| Did it succeed? | Verification + acceptance disposition, evidence-linked |
| Did it fail? | Failed/unknown/verify-failed states with reason class and next action |
| What after failure? | Flow 7: reconcile watch, cause routing, CLI path for missing verbs |
| Can the user intervene? | Real levers (binding/tool/key verbs) + honest class-C for cancel/pause + CLI guidance |
| Can the user recover? | Flows 4/7/8; restore under Stewardship |
| Can the user mis-operate? | Confirmations name exact IDs/versions/consequences; destructive separated; CAS prevents blind overwrite; no undo theater where undo doesn't exist |
| Enough trust evidence? | Every claim sourced; every state has its reason; limitations advertised |

---

*Apple-layer treatment of these supervision surfaces is in `09-control-plane-apple-design-principles.md`; decisions logged in `10`.*
