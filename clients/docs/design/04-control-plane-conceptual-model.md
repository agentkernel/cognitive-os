# 04 — Control Plane Conceptual Model

- Phase: Product Redesign Phase 1 (design-only)
- Date: 2026-08-24
- Purpose: define the object grammar every IA option and user flow must speak — Agent, Task, Run, Event, Resource and their relations — as a **conceptual model**, not page sketches. Sources: canonical product model (`cognitive-resource-model.md`, `product-design.md`, `user-journeys.md`), the audited daemon/API reality, and the shipped SPA.
- Honesty rule: concepts are marked **(authority)** = persisted daemon truth, **(projection)** = derived view, **(product concept)** = documented product language without a dedicated persisted object today. The UI may compose projections freely; it may never present a projection as authority.

---

## 1. The objects

### 1.1 Agent — the actor (product concept composed from authority identities)

An Agent is **not a resource family and not a single row**. It is "a navigation and actor concept composed from Runtime identities" (`cognitive-resource-model.md:45-47`). Its authority substrate is a deliberate seven-identity decomposition (`cognitive-resource-model.md:249-272`):

| Identity | Meaning | Never confused with |
|---|---|---|
| Package | immutable Agent distribution + provenance | installed/trusted runtime |
| Installation | verified private bytes + acquisition lock | permission or registration |
| Registration | Personal policy + installation/sidecar binding | running instance |
| Instance | supervised logical Agent runtime | conversation or Task |
| Sidecar | versioned per-Agent protocol adapter (always a client) | authority service |
| Execution | Task/Loop/instance/epoch binding | process or final acceptance |
| Process | PID/handle + bounded host observations | execution success |

**Design consequence:** an "Agent" surface is a *composed dossier* over these identities plus its Provider binding and current Execution. The current UI already honors this (9 identity cards); the redesign keeps the discipline and adds the missing relational facts (current task, recent activity, binding health).

### 1.2 Task — the work (authority)

A Task is goal-directed work under governance: raw intent durably recorded → server-issued digest-bound preview → exact admission (CAS + principal) → bounded execution → independent verification → acceptance (`cognitive-resource-model.md:222-236`). API-visible states today: `ACTIVE → CANDIDATE_COMPLETE → COMPLETED`; the product's fuller interaction vocabulary (proposed/awaiting admission/queued/running/waiting/suspended/blocked/reconciling/verifying/completed/failed/cancelled/quarantined) is the *display* vocabulary, of which only a subset is currently observable per task.

A Task owns: intent chain (record → interpretation → preview), contract (scope, conditions, budget, deadline, allowed tools/domains), Context (request + resolved view + explicit losses), Effects, Evidence, and its lifecycle transitions.

### 1.3 Run — the execution trace (product concept / projection)

**The brief's most important finding: today there is no first-class persisted "Run" object on the operator API.** What exists:

- `AgentExecution` — the epoch-fenced Task/Loop/instance binding (authority, internal);
- `Process` — bounded host observations (authority, observation-only);
- `Effect` records, lifecycle transitions, watch events — the *trace*;
- the dsh runtime snapshot — per-session live state.

The canonical IA already places **Run** under Activity (`product-design.md:213-220`), i.e. the product language treats a Run as *the observable execution of a Task*. This redesign formalizes that as a **presentation object**:

> **Run (presentation object) = the task_ref-scoped composition of { lifecycle transitions, execution/epoch facts, process observations, effects, evidence, watch events } that tells one execution story.**

Run is therefore the answer to "what happened / what is it doing" — a *narrative projection over authority facts*, never an authority claim itself. (If a future backend phase materializes a first-class Run/execution listing — backend dependency BD-3/BD-4 — this model absorbs it without redefinition.)

### 1.4 Event — the fact (authority, cross-cutting)

An Event is an "ordered authority change and watch projection input" (`cognitive-resource-model.md:35-44`). Events carry sequence, identity, domain, type, digest (O13 audit replay exposes exactly these fields, fail-closed on cursor/digest/gap). Events are the atoms of Activity; they explain every transition of every object. They are not notifications (no notification system exists) and not logs (they are authority records, redacted at the boundary).

### 1.5 Effect — the mutation record (authority, cross-cutting)

An Effect is the persist-before-dispatch record for an external or irreversible mutation, with stage (`NOT_EXECUTED…VERIFIED…OUTCOME_UNKNOWN`), outcome class, and reconcile class. Effects are how the system answers "did the outside world actually change, and how do we know". An Effect is never complete because a process exited; reconciliation establishes outcome (`cognitive-resource-model.md:274-294` rule 8).

### 1.6 Intent / Preview / Admission — the governance chain (authority)

Raw intent (recorded before any probabilistic interpretation) → interpretation candidate (with ambiguities and information gaps) → server-issued canonical preview (digest) → admission (exact digest + versions + principal). This chain is the product's signature differentiator: **the owner admits exactly what executes**. It is the conceptual backbone of any "create work" flow.

### 1.7 Evidence & Verification — the proof (authority, cross-cutting)

Evidence is an immutable fact evaluated by an independent verifier; acceptance is a separate authority act. Terminal evidence per task is API-available (report refs/digests, currency flags, artifact refs). Conceptually: **completion = verification + acceptance, both independent of the agent**. The UI's job is to make that chain inspectable, never to compress it into a checkmark.

### 1.8 Resources — what work uses (six families, authority)

Memory (admitted durable knowledge: candidate→decision→object, versions, tombstones), Skill (immutable package/revision + bindings), Tool (static registry + lifecycle overlay + per-task exposure), Context (per-task authorized input view with explicit losses), Task (§1.2), Runtime/Process (§1.1). Cross-cutting: Budget, Permission, Model, Artifact, Intent/Effect, Evidence, Event.

### 1.9 Provider / Model / Binding — the egress governance (authority)

ProviderAccount (kind, endpoint trust, secret_ref opaque, status active/revoked/degraded, catalog revision), Model catalog entries (source, pricing, cost_unavailable honesty), AgentBinding (one fixed account+model per agent, CAS revision, no fallback/override by design), Usage/Budget/Alert/Audit (budgets observe-only).

---

## 2. The relationship graph

```text
                                ┌─────────────────────────────┐
                                │           OWNER             │
                                │  (principal://local/owner)  │
                                └──────────────┬──────────────┘
                                               │ admits / governs / supervises
                                               v
   ProviderAccount ──Binding(CAS)──▶ AGENT ◀── composed of ── Package · Installation ·
   (kind, trust, secret_ref)        (actor)                    Registration · Instance ·
        │                             │                        Sidecar · Execution · Process
        │ egress (daemon proxy)       │ executes (epoch-fenced)
        v                             v
      Model                        TASK (authority) ──▶ CONTEXT (request→view, losses explicit)
   (catalog, pricing)               │  │  │                    ▲ consumes admitted
                                    │  │  └── Budget           │ MEMORY (candidate→object,
                                    │  └────── Intent chain:   │   versions, tombstones)
                                    │    record→interpret→     │ SKILL (package/revision,
                                    │    preview→admit         │   bindings)
                                    v                          │ TOOL (registry, lifecycle,
                              EXECUTION / RUN                   │   exposure, selection)
                              (trace projection)                │
                                    │ produces                  │
                                    v                           │
                              EFFECT (stage, outcome, reconcile)│
                                    │ judged by                 │
                                    v                           │
                    EVIDENCE → VERIFICATION → ACCEPTANCE (independent)
                                    │
                                    v
                          EVENT stream (ordered authority
                          changes; watch/O13 replay input)
```

Reading the graph as the brief's five questions:

| Question | Answer in this model |
|---|---|
| Is Agent the subject? | Agent is the **actor** — a composed identity, never a container of truth. |
| Is Task the work? | Yes — the only governable unit of work; everything execution-related hangs off it. |
| Is Run the execution instance? | Run is the **execution trace** of a Task — a first-class *presentation* object over authority facts, not a persisted entity today. |
| Is Event the fact? | Yes — ordered authority change; the atom of Activity and the input to watch. |
| Is Resource the dependency? | Resources are **what work is allowed to use** — admitted, bound, exposed, budgeted; never ambient. |

---

## 3. The five relational rules the UI must never break

1. **Identity discipline.** Package ≠ Installation ≠ Registration ≠ Instance ≠ Sidecar ≠ Execution ≠ Process. A UI that merges them (e.g. "agent status" that is actually process liveness) destroys the product's diagnostic value. Every surface labels which identity a fact belongs to.
2. **Observation ≠ authority.** Process alive, SSE stream open, agent text emitted, exit code 0 — none of these advance or complete a Task. The UI renders them in an "observation" lane, visually separated from the authority lane (lifecycle, effects, verification).
3. **Content ≠ permission.** An installed agent, an enabled skill, a selected model, a discovered tool — none grant capability. Surfaces show capability as the *intersection* of registration + binding + lifecycle + exposure, never as presence.
4. **Preview binds admission.** Any mutation flow displays the digest/version it will admit; stale → new preview. The UI never edits a preview into a different mutation silently.
5. **Unknown is a value.** Every projection field can be unknown/not-run; the model carries it through to display (§3 of the capability model).

---

## 4. Conceptual-model decisions that shape IA (input to 05/06)

- **Task is the gravitational center of daily supervision; Agent is the gravitational center of trust.** The IA must let the operator pivot between "the work" and "the actor" in one step, from either direction (task → its agent; agent → its current/recent tasks).
- **Run is how humans narrate Task execution.** Even without a backend Run entity, the UI needs a Run-shaped reading experience (timeline of transitions + effects + evidence + process facts). This argues for a Run detail *view* composed per task_ref.
- **Event/Effect/Evidence are cross-cutting explainers.** They deserve a coherent *activity/evidence* presentation, but per-object (in context) first, unified feed second (BD-5).
- **Provider/Model/Binding is one governance story**, not three objects: account → catalog → binding → usage/alert. IA should treat it as one domain.
- **Resources are four families with different reading needs** (Memory = knowledge with provenance; Skill = packages with revisions; Tool = capability with risk/lifecycle; Context = per-task view). One generic "Resources" browser flattens exactly the differences that matter; family-specific depth is required even under a shared envelope.

---

*Feeds: `05-control-plane-ia-options.md` (structures over this grammar), `06-control-plane-recommended-ia.md`, `07-control-plane-user-flows.md`, `08-control-plane-agent-ux.md`.*
