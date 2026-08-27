# 01 — Control Plane Product Model

- Status: adopted Personal 2.0 product model; historical 2026-08-24 analysis retained
- Updated: 2026-08-27
- Method: product-skills stack (problem-validation lens, JTBD, opportunity mapping, scope cutting); stark `ux-design` product-type matching; `ai-agent-ux` supervision model. CognitiveOS reality (audited) outranks every framework on conflict.
- Inputs: [Current State Map](control-plane-current-state.md), [Capability Inventory](control-plane-capability-inventory.md), canonical `docs/product/personal/*`.

## Adopted Personal 2.0 target

This revision supersedes the earlier product-model boundaries in this file where
they differ. The analysis below remains useful rationale, but the adopted target
is no longer a read-only governance console separated from conversation.
Decision carriers:
[ADR-0056](../../../docs/adr/0056-personal-2-0-desktop-control-plane.md) and
[ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md).

**Product model:** the Personal desktop is the owner's primary entry into a
local cognitive operating environment. It combines a global **Agent Shell**
with a precise Control Plane. Conversation stays native to each Agent by
default; the owner chooses **Manage with Personal** when a conversation becomes
governed work. Manage with Personal projects the daemon's durable
`Goal -> Plan revision -> Task -> attempt`
chain without turning chat text, Agent reasoning, or process output into
authority.

The stable first-level IA is:

`Home / Agents / Work / Library / Activity / Settings`

- **Home** resumes the owner: recent conversations, work needing attention,
  readiness, and evidence-backed outcomes.
- **Agents** installs/connects Agents, hosts Adapter-backed embedded
  conversation/history, and exposes Runtime facts in beginner-first dossiers.
- **Work** contains Goals, Plan revisions, Tasks/attempts, Context, Effects,
  verification, and multi-Agent orchestration projected by the daemon.
- **Library** groups Memory, Skills, Tools, and MCP by operator task. Personal
  2.0 has seven real families overall: Memory, Skill, Tool, Context, Task,
  Runtime/Process, and MCP. Context and Task belong in Work;
  Runtime/Process belongs in Agents. Model, Permission, Artifact, Budget,
  Evidence, and Event remain cross-cutting objects, not families.
- **Activity** is one provenance-aware timeline spanning `Native`, `Observed`,
  `Governed`, and `Verified` facts.
- **Settings** owns Account Hub (Providers/models/credentials/quotas/cost),
  System, appearance, accessibility, and diagnostics. Providers and System are
  not first-level destinations.

The desktop shell is three-region and desktop-primary: navigation, primary
workspace, and contextual inspector. The global Agent Shell is reachable
without leaving the selected object. A command palette accelerates known
destinations and real actions; it does not compensate for unclear IA.

Agent integration is Adapter-based. Vendor-native conversation/history appears
behind a common internal projection and capability matrix, with explicit native
slots for bounded display metadata/artifacts where an Agent supports more.
Slots cannot inject actions, executable markup/scripts, credentials, or
authority-shaped state; vendor actions use Control Plane-owned typed controls.
Installing or connecting an Agent should take no more than three understandable
steps and end at a real first chat. Disconnect and uninstall remain distinct
choices.

Account Hub supports target acquisition tiers—OAuth/subscription, API key,
user-directed import under
[ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md),
and custom gateway—while preserving daemon-only SecretStore/proxy custody.
Only today's API-key/provider path is current implementation. OAuth,
subscription import, credential import, quotas, and any unsupported cost/model
projection are **Requires-backend**, never implied by an active control.

Resources may be federated observations and, where a typed daemon workflow
exists, governed bidirectional writeback. The Agent Shell may suggest a conflict
resolution; only the daemon may issue preview, collect confirmation, persist
Intent/Effect, dispatch, and verify. MCP is a first-class target family, but
MCP plus rules does not control host Agent sessions.

Current implementation remains the P7-T05 seven-route SPA documented in
[Current State Map](control-plane-current-state.md): Home / Work / Agents /
Providers / Resources / Activity / System, with its recorded API limits. Target
screens must label unsupported controls **Requires-backend** and must not show
fake buttons, fake progress, or inferred completion.

---

## Historical 2026-08-24 product-model analysis

## 1. Why a Control Plane exists at all

The product's own documents already answer the existential question; the redesign's job is to test whether the answer still holds:

- CognitiveOS Personal is "a local **operating system for cognitive resources**… a unified control plane above the host OS" (`docs/architecture/personal/README.md:17-19`). The daemon holds authority; every experience surface is a client.
- The Web UI exists to give the owner "one local, read-first place to answer four questions" (agents usable? providers reachable? bindings? what is running/changed/blocked/verified?) (`docs/product/personal/web-ui-design.md:19-24`).
- The system's differentiating promise is **auditable, budgeted, recoverable, never falsely completed** agent work (`handbook/en/user/what-is-personal.md:24-25`).

A control surface is not optional decoration on this product. The daemon's authority decisions (admission, CAS, fencing, verification, reconciliation) are invisible without one. **The Control Plane is where the owner supervises the authority that supervises the agents.** If that surface is a wall of raw JSON (current state, §10 of the Current State Map), the product's core promise is undelivered to its primary user.

### Problem framing, honestly validated

Applying the problem-validation discipline to *this redesign* (not to a new product — the product exists and is owner-directed):

| Dimension | Evidence | Read |
|---|---|---|
| Frequency | Every operator session starts at the UI; the four product questions are per-session questions | Daily-use surface |
| Intensity | Current UI cannot list tasks, cannot show live state, renders authority projections as raw JSON; the flagship governance story (preview→admit→verify) is invisible except as digests | High — core value undelivered |
| Existing workaround | The owner already routes around the UI: `cognitive` CLI verbs, admin-cli, raw endpoint probes, checkpoint reports as status sources | Strong workaround in active use |
| "Willingness to pay" | Owner directed and funded a full redesign phase with a curated skill stack (`docs/design/skill-manifest.md`) | Demonstrated |

Verdict: the problem (operator surface does not deliver the product's supervision value) is real, frequent, and currently worked around via CLI. This is not an opinion-driven redesign.

---

## 2. Candidate product models

The user asked to compare at least: Dashboard, AI Agent Management Console, AI Operations Center, Cognitive System Control Plane, Personal AI Operating Environment, Agent Workbench — and to choose by JTBD, user tasks, system capability, and future direction, **not** by fit to the existing UI.

### Model A — Dashboard

*Essence: a glanceable summary of metrics and states.*

- JTBD fit: answers "is everything okay?" but not "what is the agent doing, why, what do I do about it". Fails investigation and intervention jobs entirely.
- Capability fit: the API surface is object- and event-centric (tasks, effects, evidence, bindings), not metric-centric. Budgets are observe-only; there are almost no meaningful aggregate metrics to chart honestly.
- Apple fit: dashboards push toward card walls and KPI strips — explicitly banned by the accepted visual direction (`web-ui-design.md:160-165`: no card walls, no ornamental dashboard strips).
- Verdict: **Rejected as the product model.** A status summary can be one *view* (Home), never the organizing idea.

### Model B — AI Agent Management Console

*Essence: CRUD + lifecycle administration of agent objects (like a SaaS admin panel for agents).*

- JTBD fit: covers configuration jobs (bind, install, enable) but centers *objects*, not *work*. The operator's daily questions are about work in flight and verification, not object CRUD.
- Capability fit: agent lifecycle over HTTP is **NOT AVAILABLE** (CLI-only today); task control verbs are NOT AVAILABLE. A console model would make the UI's most prominent verbs the ones the backend cannot honor — a capability-honesty trap.
- Apple fit: "admin console" pulls toward SaaS-admin furniture (settings rows, management tables) — the exact genericness the brief forbids.
- Verdict: **Rejected as primary model.** Management capability is one dimension of the product, not its identity.

### Model C — AI Operations Center

*Essence: monitor live operations, triage exceptions, intervene, recover (NOC/SRE lineage).*

- JTBD fit: strong on supervision ("what's happening/blocked/needs attention") and recovery jobs. The priority-queue + status-board patterns fit.
- Capability fit: partial — watch is process-local and thin, no unified activity feed, no task control verbs. An ops center without intervention verbs is an ops *viewer*.
- Risk: "operations center" aesthetics (metric walls, incident furniture) drift toward the dashboard failure mode; and it under-weights *governance* — previews, admissions, bindings, evidence — which is what makes this product different from a process monitor.
- Verdict: **Strong partial fit.** The supervision grammar of an ops center must be absorbed, but it is not the product's identity.

### Model D — Cognitive System Control Plane

*Essence: the operator surface of a local authority. Every view projects authority state; every action is a typed, previewed, admitted governance operation; every claim carries evidence.*

- JTBD fit: the documented jobs are governance jobs — preview before mutation, admit exact digests, bind fixed provider routes, review memory candidates, verify outcomes from independent evidence (`product-design.md:92-139`). Supervision questions ("what is it doing, is it done, can I trust it") are answered *by* authority projections (task lifecycle, effect reconciliation, verification disposition), not by process watching.
- Capability fit: exact. The API surface *is* a control-plane surface: previews, admissions, CAS revisions, bindings, lifecycle envelopes, evidence, observation planes with named zeros. The model requires no invented capability.
- Future fit: multi-agent (P6), new resource families, and hardware evolution all enter through the same control-plane grammar (authority, preview, admission, evidence). The model scales because it mirrors the architecture.
- Apple fit: a control plane done the Apple way is *calm authority* — few, precise, well-labeled surfaces; direct manipulation where safe; honest states. It is the opposite of a marketing dashboard and compatible with density.
- Risk: "control plane" is abstract; it must be given a concrete object grammar (§4) or it becomes a synonym for "admin panel".
- Verdict: **Recommended primary model.**

### Model E — Personal AI Operating Environment

*Essence: the whole local AI environment as one product — shell, agents, resources, settings.*

- This is the identity of **CognitiveOS Personal as a whole**, not of its operator surface. The Control Plane is one client next to the Shell and CLI. Adopting it as the UI's model would blur the boundary the architecture depends on (UI = client, not environment) and pull the design toward duplicating Shell/runtime functions.
- Verdict: **Rejected as the Control Plane model** (it is the product's name, not the surface's model).

### Model F — Agent Workbench

*Essence: a workbench where the user builds/debugs agent behavior directly (canvas + inspector lineage).*

- Workbenches center a *manipulated artifact* (code, canvas, document). Here the artifacts of interest (tasks, effects, evidence) are authority-owned; the user inspects and governs them but does not hand-edit them. Center-of-gravity mismatch: a workbench model would invite direct-manipulation fantasies the daemon cannot honor.
- Verdict: **Rejected as primary model.** Workbench *patterns* (inspector, stable panes) remain useful inside detail views.

### Model G — Synthesis check: is there a better model?

A honest synthesis of B+C+D: **"a supervision and governance surface for a local authority that runs agents"** — i.e., Model D with the ops-center supervision loop absorbed as its daily-use loop, and console-grade management absorbed for the domains that genuinely support it (Providers, Tools, Skills, Memory). That synthesis *is* Model D stated precisely; no eighth model emerged from the comparison.

---

## 3. Product model decision

> **The CognitiveOS Control Plane is the operator surface of the owner's local cognitive authority.**
> It projects the daemon's authority state (agents, tasks, resources, providers, effects, evidence) into legible, supervisable form; it turns owner intent into typed, previewed, admitted governance operations; and it treats every outcome as a claim that must carry its evidence. It is read-first, intervention-capable where the daemon exposes typed verbs, and honest-unavailable everywhere else.

One-line test for any future design decision: **does this surface help the owner supervise or govern the authority, with evidence — or is it furniture?**

What this means concretely:

1. **State before actions.** Every object view leads with authority state (lifecycle, health, revision, verification) before controls.
2. **Actions are governance operations, not CRUD.** Where an action exists, it carries preview/CAS/idempotency semantics inherited from the daemon. Where it doesn't exist, the surface says so (`not-run`), with the reason and the owning gap.
3. **Evidence is a first-class citizen.** "Completed" is never a green badge; it is a linked verification/acceptance record. This is the product's anti-"AI slop" guarantee made visible.
4. **Supervision loop is the daily loop.** Home and Activity serve the ops loop (what's happening / blocked / needs me), but they answer it with *authority facts* (states, effects, evidence), not metrics theater.

Relationship to the canonical product IA: this model **confirms the canonical five spaces' spirit** (Home/Agents/Tasks/Resources/Activity) while rejecting their current literal implementation. The re-evaluation of first-level navigation continues in `05-control-plane-ia-options.md` under this model, not under the shipped nav.

---

## 4. Who uses it

### The persona reality of a single-owner product

CognitiveOS Personal is explicitly single-owner, local, loopback (`web-ui-design.md:30-35`). The five candidate user types from the brief are therefore **not five people — they are five modes of one person**, the owner. Designing five personas would be fiction; designing five *modes* is honest and operationally useful (modes differ in frequency, risk tolerance, and speed needs).

| Mode | What the owner is doing | Frequency | Risk posture | Design consequence |
|---|---|---|---|---|
| **Individual Operator** (PRIMARY) | Supervising agent work: what's running, what's blocked, what finished, what needs a decision | Daily, many times | Wants truth fast; allergic to false "completed" | The product's center of gravity: supervision loop, honest states, fast scan |
| **AI Power User** (SECONDARY) | Curating cognitive resources: memory review, skill pinning, tool availability, context inspection | Weekly | Deliberate, reversible-first | Dense master/detail, preview-before-mutation |
| **System Operator** (SECONDARY) | Owning the installation: readiness, doctor, backup/restore, upgrades, provider health | Occasionally, high stakes when it happens | Cautious; wants recovery paths and proof | Guided-but-honest flows, deterministic recovery links |
| **Agent Builder** (DEFERRED) | Registering/qualifying new agents/adapters (dsh today; P6 multi-agent tomorrow) | Rare today | Expert | Read-mostly surfaces now; designed headroom, no invented lifecycle UI |
| **Developer** (OUT OF SCOPE as primary) | Consuming contracts/APIs, debugging integrations | — | — | Served by CLI/handbook; the Control Plane must not contort to serve API developers |

**Primary user: the Individual Operator** — the owner in supervision mode. When a conflict arises between "helps configure the system once" and "helps supervise work every day", supervision wins. This matches the documented primary user ("wants one local place to understand… what is running, what changed and whether a result is actually complete", `product-design.md:29-34`) and sharpens it.

### Jobs the surface is hired for (summary — full analysis in `02-control-plane-jtbd.md`)

1. Supervise: what is happening, why, what is blocked, what needs me.
2. Verify: is it actually done — by whose evidence.
3. Investigate: what happened, what did it touch, what did it cost.
4. Intervene (within typed authority): stop/redirect/repair/approve — or learn honestly that the verb does not exist yet.
5. Govern: what may my agents use; change it safely (preview, CAS, audit).
6. Steward: install, ready, back up, restore, upgrade the system itself.

---

## 5. Scope discipline (scope-cutting applied)

Appetite statement for the redesign's first implementation wave (post-design phases; recorded here so IA options are shaped by scope, not fantasy):

- **Must serve (P0):** the supervision loop (system state, task/run inventory, attention queue), task/run/evidence detail, provider+binding governance, resource family browsing with real depth, honest unavailable states, session ergonomics.
- **Should serve (P1):** live-ish updates within actual daemon capability, notification of alerts, command/keyboard speed paths, memory/skill curation depth.
- **May serve later (P2):** agent lifecycle control (backend-blocked), unified cross-domain activity feed (backend-blocked), budgets-as-controls (backend-blocked), multi-agent surfaces (P6 not-started).
- **Never (product boundary):** multi-user/RBAC, remote access, browser secret custody, generic lifecycle routes, dashboard-metric theater, chat UI. (Inherited non-goals: `provider-control-plane.md:155-160`, `web-ui-design.md:213-222`.)

Explicitly **deleted** from the current surface (scope hammer, justified in the decision log):

- "Simulate cursor gap" as product UI (developer affordance).
- Raw-JSON panels as the primary presentation (demote to an inspector affordance, never the default reading).
- Bindings as a first-level nav peer (fold into the governance story; see IA options).
- Session as a first-level nav item (it is a gate/utility, not a space).

Explicitly **deferred** (not deleted): everything backend-blocked in the Capability Inventory §12 matrix, each recorded as a named backend dependency rather than a UI promise.

---

## 6. Non-negotiable product constraints the model inherits

1. Daemon is the sole authority writer; the UI never becomes one (A1).
2. No secrets in the browser beyond one-time, memory-only entry (A5, ADR-0053).
3. Preview before mutation; admission binds digests; stale facts require new preview.
4. Completion is decided by independent verification; the UI renders dispositions, never inference.
5. Capability honesty: unavailable renders as unavailable, with the missing dependency named.
6. Loopback, single owner, same-origin serving; no CORS, no cookies, no CDN.
7. The Control Plane is a client; its caches are presentation state and must degrade to stale markers, never to fabricated authority.

These constraints are not limitations on the design — they *are* the design's material.
