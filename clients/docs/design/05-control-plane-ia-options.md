# 05 — Control Plane IA Options

- Status: adopted Personal 2.0 IA outcome; historical options retained
- Updated: 2026-08-27
- Method: stark `ux-design` (route topology, navigation model matrix, IA object rules), `desktop-app-archetypes`, `usability-pattern-matrix`; evaluated against the product model (`01`), JTBD (`02`), capability model (`03`), conceptual model (`04`), and the audited current state. Apple fit is judged by the `apple-design` principles (clarity, hierarchy, restraint, direct manipulation, wayfinding), not by styling.
- Framing rule from the brief: nothing is first-level "because it is today". Every first-level candidate must answer: **why first-level? why not second-level? why not a contextual action? why not a command?**

## Adopted IA outcome (Personal 2.0)

The option analysis below is retained as the 2026-08-24 reasoning record, but
its seven-space recommendation is superseded. The adopted desktop IA is:

`Home / Agents / Work / Library / Activity / Settings`

| Destination | Why first-level | What moved beneath it |
|---|---|---|
| Home | resume and attention are the primary entry job | readiness summaries, recent conversations, current managed work |
| Agents | conversation and trust begin with an actor | Runtime, install/connect, Adapter capability matrix, native history, display/artifact slots |
| Work | durable Goals/Plans/Tasks are the governable work | Context, attempts, Effects, verification, multi-Agent orchestration |
| Library | recurring resource discovery and curation needs a stable home | Memory, Skills, Tools, MCP; the other target families stay in Work/Agents |
| Activity | cross-object investigation is a distinct repeated job | one provenance timeline: Native/Observed/Governed/Verified |
| Settings | low-frequency account/system configuration should not compete with daily work | Account Hub/Providers, System/readiness detail, appearance, accessibility, diagnostics |

### Why the former peers moved

- **Providers -> Settings / Account Hub:** account acquisition, credentials,
  models, quotas and cost are configuration and stewardship. Provider state may
  surface contextually on Home, Agents, Work and Library.
- **System -> Settings / System:** readiness remains globally visible, but deep
  diagnostics and stewardship are low-frequency.
- **Resources -> Library:** "Library" expresses the owner job and holds Memory,
  Skills, Tools, and MCP without implying one generic Resource object. The
  seven-family product count is preserved across the IA: Context/Task live in
  Work and Runtime/Process lives in Agents.
- **Tasks -> Work:** Work includes Goal, Plan revisions, Tasks, attempts and
  Context; it is not merely a task table.
- **Runtime -> Agents; Context -> Work:** both are contextual identities, not
  Library families in the target IA.

### Layers, not destinations

- The **global Agent Shell** persists across destinations and returns to the
  selected Agent/conversation.
- **Manage with Personal** is an explicit mode transition from conversation
  into Work.
- The **command palette** accelerates destinations, known objects and
  current-backed actions.
- **Search** is scoped to known projections and declares coverage; it does not
  imply backend-global search.

The P7-T05 current implementation still has seven current routes
Home/Work/Agents/Providers/Resources/Activity/System. Those are evidence, not
the target navigation. Migration must preserve working deep links or provide
honest redirects; it must not label a target-only page as implemented.

---

## 0. Shared evaluation criteria

| Criterion | What it measures | Weight driver |
|---|---|---|
| Product Model fit | Does the structure express "operator surface of a local cognitive authority"? | `01` §3 |
| JTBD fit | Does it serve the ranked jobs (supervision > verification > investigation > governance > stewardship)? | `02` §4 |
| Capability honesty | Can the center surfaces be backed by real API today (or honest states)? | `03` |
| Scalability | Does it survive: more agents (P6), more tasks, more resource depth? | future direction |
| Apple fit | Clarity, deference, depth, hierarchy, wayfinding, restraint | `09` |
| Agent UX fit | Supervision legibility, stop/override reachability, audit trail, trust calibration | `08` |
| CognitiveOS fit | Identity discipline, observation≠authority, preview→admit grammar | `04` §3 |

The current IA, audited: flat sidebar `Home / Agents / Providers / Bindings / Tasks / Activity / Resources / Session` (Current State Map §3). Its structural defects: no task inventory to anchor Tasks; Activity mis-scoped to provider JSON; Bindings unsanctioned as a peer; Session is a utility; Resources is a stub browser. The options below are redesigns, not renames.

---

## Option A — Agent-centric IA

**Product model expression:** the Control Plane as the place where you supervise *your agents*; everything hangs off the actor.

```text
Sidebar:  Agents (default landing) · Providers · Resources · System · Activity
Agent detail = workspace:  Overview | Current work | Runs | Binding | Resources | Activity
```

- **Core objects:** Agent (dossier) is the root object; Task/Run appear as agent children; Provider/Binding as agent governance; Resources as "what this agent may use".
- **Primary workflow:** open Agents → pick the actor → see state, current work, health → drill to a run → verify evidence.
- **Strengths:** matches the naive mental model ("what is my agent doing"); trust anchors naturally per actor; agent detail becomes a genuine workspace.
- **Weaknesses:** builds the primary axis on a **projection** (Agent is composed, not authority); today's backend cannot answer "agent's current/recent work" (SidecarSession not HTTP-exposed; only dsh has a runtime snapshot; no per-agent task listing — BD-2/BD-3); the attention problem (blocked *tasks*, system readiness) has no natural home; with exactly two qualified agents (Pi, dsh) the root list is over-structured for years; multi-agent futures (P6) would need work that spans agents, which this IA fractures.
- **Scalability:** poor→medium. Agent count scales; work volume does not (runs nested per agent hide the global "what needs me").
- **Apple fit:** medium — clean object hierarchy, but the landing surface answers the wrong question for daily use.
- **Agent UX fit:** medium — per-agent supervision is good; system-level supervision (the operator's rank-1 job) is orphaned.
- **CognitiveOS fit:** weak — inverts the authority grammar (Task is the governable unit; Agent is a view).
- **Verdict:** rejected as primary. Its genuine value (agent dossier with current work) is absorbed as a *detail* pattern in the recommendation.

## Option B — Task/Work-centric IA

**Product model expression:** the Control Plane as the place where you supervise *work*; actors and resources are the work's context.

```text
Sidebar:  Work (default landing: tasks/runs) · Agents · Providers · Resources · Activity · System
Task detail = workspace:  Overview | Intent & Contract | Context | Run (timeline) | Effects | Evidence
```

- **Core objects:** Task/Run root; Agent, Provider, Resources as relational context of the work.
- **Primary workflow:** open Work → the inventory (running / blocked / needs verification / recent) → open a task → run timeline → evidence.
- **Strengths:** serves rank-1..4 jobs directly; matches the daemon's authority grammar exactly; the Run presentation object (`04` §1.3) gets a natural home; attention queue is native to the list.
- **Weaknesses:** the center page depends on **BD-3 (task inventory projection)** — today's only task list is a 64-row envelope with no objective text; without BD-3 the landing list is thin on day one; "what is my agent doing" needs a pivot; resource curation jobs are second-class.
- **Scalability:** high for work volume; flat for everything else.
- **Apple fit:** high — one clear subject, deep hierarchy, quiet chrome.
- **Agent UX fit:** high for in-flight supervision; weaker for "which actor" questions.
- **CognitiveOS fit:** highest — structure mirrors authority.
- **Verdict:** strongest spine; its dependency risk (BD-3) is real but honest (the list can ship envelope-first with named-zero states, deepening when BD-3 lands).

## Option C — System/Control-centric IA

**Product model expression:** the Control Plane as the instrument panel of the authority itself: readiness, governance, policy, stewardship.

```text
Sidebar:  System (default: readiness/doctor/stewardship) · Governance (providers, bindings,
          budgets, tool policy) · Work · Resources · Activity
```

- **Core objects:** System health, governance objects (accounts/bindings/budgets/policies), then work.
- **Primary workflow:** verify system readiness → repair governance → then look at work.
- **Strengths:** the most "control plane"-literal; excellent for System Operator mode and degraded-mode recovery; governance domains cohere (Providers+Bindings+Budgets+Tool policy in one place).
- **Weaknesses:** the daily supervision loop (rank-1 jobs) is demoted below system furniture; landing on System reads as admin panel, not operator surface; readiness is a *checkpoint*, not a *destination* — you verify it and leave.
- **Scalability:** medium; governance objects grow slowly, work grows fast and is underweighted.
- **Apple fit:** medium-low — drifts toward settings/admin furniture; the brief explicitly bans SaaS-admin feel.
- **Agent UX fit:** low for in-flight supervision.
- **CognitiveOS fit:** medium — honest about authority, but misreads what the authority is *for* (the work).
- **Verdict:** rejected as primary. Its governance cluster is absorbed as the Providers domain and System space in the recommendation.

## Option D — Supervision IA (attention-first Home + work-centered spaces) — synthesis

**Product model expression:** the Control Plane as the owner's supervision surface: land on *what needs you*, pivot to *the work*, *the actor*, *the resources*, *the proof*.

```text
Sidebar:  Home · Work · Agents · Providers · Resources · Activity · System
(+ persistent status strip; Session as utility chrome, never a nav peer;
 Command palette ⌘K as the speed layer over all objects/actions)

Home      = attention surface: readiness summary, needs-attention queue
            (blocked/degraded/failed/unknown-outcome/alert), current work strip.
            NOT a dashboard: no metric cards; every row is a navigable authority fact.
Work      = task/run inventory + detail (Intent·Contract·Context·Run timeline·Effects·Evidence)
Agents    = actor inventory + dossier (7 identities, binding, current/recent work, activity)
Providers = egress governance: accounts · models · bindings · usage · budgets · alerts · audit
Resources = four families with family-specific depth: Memory · Skills · Tools · Context
Activity  = evidence/event stream: per-object timelines first; unified feed when BD-5 lands
System    = readiness/doctor detail, backup/restore, session, about/diagnostics
```

- **Core objects:** all of `04`, with Task/Run as the daily center and Agent as the trust center.
- **Primary workflow:** land on Home → triage the attention queue → pivot into Work/Agent/Provider detail → verify evidence → govern via preview→admit actions.
- **Strengths:** serves the ranked jobs in order; every space is backable today or honestly deferred (capability model §1-2); conforms to the canonical five-space product IA *in spirit* while fixing its implementation (Providers promoted with sanction from P8-T13's depth; Bindings folded into Providers **and** agent dossiers contextually; Session demoted to chrome); scales by adding depth inside spaces rather than new peers.
- **Weaknesses:** Home risks re-introducing dashboard thinking (mitigated by the no-metric-cards rule: Home is a **queue + state**, not charts); seven peers is the upper bound of calm navigation (mitigated by grouping: System carries low-frequency stewardship; status strip carries global state so spaces stay clean).
- **Scalability:** high. New agents → Agents rows; new work → Work; new families → Resources sections; multi-agent (P6) → Work gains grouping, not restructuring.
- **Apple fit:** high — one primary subject per space, hierarchy through depth, quiet persistent chrome, direct manipulation where safe (master/detail, inspector), restraint elsewhere.
- **Agent UX fit:** highest — supervision loop native (Home/Work), override path visible where verbs exist, audit/evidence first-class (Activity), trust calibration via honest states.
- **CognitiveOS fit:** high — spaces map to authority domains; identity discipline preserved; observation≠authority enforced in Work/Run detail lanes.
- **Verdict:** recommended. Full specification in `06-control-plane-recommended-ia.md`.

---

## Comparative matrix

| Criterion | A Agent-centric | B Work-centric | C System-centric | D Supervision |
|---|---|---|---|---|
| Rank-1 job (state+attention) | weak | strong | medium | **strongest** |
| Verify completion (evidence) | medium | strong | weak | **strong** |
| Actor trust (agent dossier) | **strong** | medium | weak | strong |
| Backable today (honesty) | weak (BD-2/3) | medium (BD-3) | strong | **strong** |
| Scales with work volume | weak | **strong** | weak | strong |
| Scales with agent count | strong | medium | medium | strong |
| Dashboard-template risk | low | low | medium | low (with guardrail) |
| SaaS-admin risk | medium | low | **high** | low |
| Canonical-IA continuity | breaks | bends | breaks | **keeps & fixes** |

## First-level admissibility record (the brief's four questions, applied)

| Candidate | First-level? | Why not second-level / contextual / command |
|---|---|---|
| Home (attention) | Yes | The landing answer to "what needs me" cannot be second-level; it is the supervision loop's entry. |
| Work (Tasks/Runs) | Yes | The daily object; second-level would bury the product's purpose. |
| Agents | Yes | Trust center; but a *space of dossiers*, not the landing axis (Option A rejected). |
| Providers | Yes | Deepest governance domain (accounts/models/bindings/usage/budgets/alerts/audit); P8-T13 made it a first-class operator reality. Canonical IA allowed this as a dedicated operator view (`web-ui-design.md:41-48`). |
| Bindings | **No** | A relation, not a space: lives inside Providers (by account) and Agents (by actor), and in mutation flows. Current first-level placement rejected. |
| Resources | Yes | Four families, one envelope; family depth below. |
| Activity | Yes | The evidence stream; per-object timelines live in detail pages, the space owns cross-object reading (unified feed gated on BD-5, honestly). |
| System | Yes | Readiness detail, doctor, backup/restore, session, diagnostics — low-frequency, high-stakes; grouped so daily spaces stay calm. |
| Session | **No** | A gate + utility chrome (status strip affordance), never a destination. |
| Search/Command | **No (it is a layer)** | ⌘K palette over all objects/actions — a speed layer, not a space (navigation-IA reference: command palettes must not become junk drawers for failed IA). |
| Notifications | **No** | Alerts surface in Home's attention queue + status strip; no separate notification center in wave 1 (alerts are pull-based; BD honesty). |

---

*Decision between options is recorded in `06-control-plane-recommended-ia.md` and logged in `10-control-plane-design-decisions.md` (DD-01).*
