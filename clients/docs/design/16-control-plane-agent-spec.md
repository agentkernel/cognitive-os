# 16 — Agent Spec (Actor Inventory & Dossier)

- Phase 2 (design-only)
- Date: 2026-08-24
- Contract: `06` §3.3, `04` §1.1 (seven identities), jobs J-K2 + canonical job 6, capability reality: Agents are **read-mostly** — lifecycle verbs are class-C (BD-2); HTTP-visible agent facts = runtime inventory/inspect + bindings + dsh runtime snapshot.
- Design thesis: the Agent surface is a **dossier**, not a profile page and not a control panel. Its job is trust: *what is this actor, what may it use, what is it doing, what has it verifiably done.*

---

## 1. The observation / control / configuration boundary (explicit, per the brief)

| Kind | What | Where it lives here |
|---|---|---|
| **Observation** | identity facts, registration/lifecycle projections, dsh runtime snapshot, binding dispatchability, current/recent work links, activity/evidence slices | everywhere (default) |
| **Control** | typed verbs that change the running actor | **class-C only today**: pause/resume/stop/restart/quarantine render as not-available + CLI path (DD-08). The only live "controls" adjacent to an agent are governance mutations on *its dependencies* (binding change, tool disable) — and those live in Providers/Resources, linked contextually |
| **Configuration** | install/register/upgrade paths | CLI-owned; the dossier shows the *result* (identities, digests) and links the CLI verb; never a fake configure form |

This boundary is stated once in the dossier header ("Lifecycle control runs through `cognitive` — BD-2") as calm expectation-setting, then enforced by absence everywhere else.

## 2. Inventory (master)

```text
┌────────────────────────────────────────────────────────────────┐
│ Agents                                                          │
│ ┌──────────────────────────────────────────────────────────────┐│
│ │ ● pi        registered · instance supervised                  ││
│ │   binding: deepseek-main / deepseek-chat · callable           ││
│ │   current: task a3f9… (running, 4m)                  [open ›] ││
│ │                                                               ││
│ │ ○ dsh       runtime ACTIVE · process alive · 1 session        ││
│ │   binding: deepseek-main / deepseek-chat · callable           ││
│ │   current: none observed                             [open ›] ││
│ └──────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

Rows lead with **actor + governability + current work** — not with package metadata. Row anatomy: state dot (by best-available lifecycle projection, source-labeled) · display identity · binding state (callable/blocked/unbound) · current work link when known (BD-2/BD-3 honest: "none observed", never "idle"). Identity depth (versions, digests) is dossier content, not list noise.

## 3. Dossier (detail route `#/agents/:id`)

```text
┌──────────────────────────────────────────────────────────────────┐
│ pi   ● registered · supervised        binding callable · rev 4   │
│ Lifecycle control runs through `cognitive` CLI (BD-2)            │
├──────────┬───────────────────────────────────────────────────────┤
│ sections │ OVERVIEW — the seven identities as distinct fact cards │
│ Overview │  package 3f2a… · installation 9c1b… · registration …  │
│ Current  │  instance pi-01 · sidecar sc-77… (digest) · execution │
│ work     │  epoch 3 · process pid 4812 alive (obs) · task link   │
│ Binding  │  (each card names what it is NOT: "process liveness   │
│ Capab.   │   is not task completion")                            │
│ Activity │ CURRENT WORK — execution/task linkage when projected; │
│ Evidence │  else S7 "not observable over HTTP (BD-2/BD-3)"       │
│          │ BINDING — account·model·revision·dispatchability;     │
│          │  [Change binding →] (Providers flow, agent preselected)│
│          │ CAPABILITIES — tool exposure (per-task), workspace     │
│          │  scope, model route; content≠permission annotation     │
│          │ ACTIVITY — actor-scoped activity slice → Activity      │
│          │ EVIDENCE — recent verifications involving this actor   │
└──────────┴───────────────────────────────────────────────────────┘
```

Section rules:

1. **Overview** — the seven identity cards (package / installation / registration / instance / sidecar / execution / process), each with digest/value, source projection, and its "never-confused-with" caption where confusion is dangerous (process vs execution vs task). The shipped 9-card discipline is preserved; cards here are *identity documents*, not decoration (DD-10 exception, justified).
2. **Current work** — observation-lane content; when the daemon doesn't project it, S7 with the dependency named. Never inferred from process liveness.
3. **Binding** — governance summary + the change action that enters the Providers flow with this agent preselected (contextual nav, DD-04).
4. **Capabilities** — what this actor may use: tool exposure (with task scope), model route (bound account/model), workspace scope. Standing annotation: "Installed ≠ permitted. Capability = registration + binding + exposure + lifecycle."
5. **Activity / Evidence** — filtered links into Activity (actor slice) and recent evidence involving this actor. No duplicated machinery — these are contextual views of the Activity space.
6. **dsh variant:** the dsh dossier adds the runtime snapshot block (state ACTIVE/INACTIVE/CRASHED, sessions, fencing epoch, process liveness — observation-labeled) and the Apply affordance state (linking to the binding flow), per the shipped dsh integration.

## 4. States

| State | Rendering |
|---|---|
| Empty inventory | "No agents registered." + how registration happens (install journey pointer) — read-only guidance, no fake button |
| Lifecycle unknown (Pi without HTTP projection) | S7 + "lifecycle is CLI-observable; HTTP projection is BD-2" |
| Unbound agent | binding section = designed zero-capability state ("no binding — this agent cannot call a model") + bind action |
| Binding blocked (account revoked/degraded) | S5 row + repair link to the account |
| dsh CRASHED | S5 + last-seen facts + "restart via `cognitive dsh …`" guidance (class-C) |

## 5. What this surface refuses

No fake lifecycle buttons; no "agent status" synthesized from process liveness; no chat affordance (the Shell owns conversation); no marketplace/install UI (acquisition is a governed install journey, CLI/daemon-owned today); no per-request provider override (policy, stated once).

---

*Binding mutations execute in `17-control-plane-provider-spec.md`; activity slices render per `19`.*
