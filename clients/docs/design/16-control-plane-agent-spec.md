# 16 — Agent Spec (Actor Inventory & Dossier)

- Status: adopted Personal 2.0 Agent target plus current P7-T05 dossier reality
- Updated: 2026-08-27
- Target: Adapter conversation/capability projection, typed Control Plane
  actions, and Agent runtime engine supervision
- Current implementation: read-mostly runtime inventory/inspect, bindings, and
  dsh runtime snapshot; lifecycle remains unavailable over HTTP

## Personal 2.0 Agent spec

Agents is both the conversational center and the trust center. Each Agent uses a
vendor-specific Adapter behind a common internal projection.

### Inventory

Rows show Agent identity, connection/install state, Adapter compatibility,
account/model route, last native conversation, current managed work when known,
and one honest next action. "No work observed" is never translated to idle.

### Install/connect flow (three steps maximum)

1. **Choose:** signed catalog entry or supported existing installation.
2. **Review:** source/signature, Adapter capabilities, native slots, account and
   resource boundaries, retained data, and every `Requires-backend` limitation.
3. **Connect:** perform the typed daemon operation and open conversation.

Success is a real first chat response. A successful download, registration,
process start or connection probe is not first-chat success. Catalog,
signature, install/register/lifecycle and embedded conversation are target
capabilities and currently `Requires-backend` unless a verified route backs the
specific step.

### Agent workspace

| Section | Target content |
|---|---|
| Conversation | embedded native history through the Adapter common projection; source identity and freshness |
| Native slots | display-safe vendor metadata and artifact renderers; no actions, executable markup, or credentials |
| Manage with Personal | explicit conversation-to-Goal/Plan preview; never implicit |
| Agent runtime engine | beginner label for package, installation, registration, instance, sidecar, execution, process—exact Runtime/Process terms stay distinct in the inspector |
| Capabilities | common matrix: conversation/history, attachments, models, MCP, resource observation/writeback, managed-work participation |
| Accounts | effective global/Agent/conversation account/model route; repair opens Settings/Account Hub; override hierarchy Requires-backend beyond today's fixed binding |
| Work | Goal/Task/attempt links projected by the daemon |
| Activity | Agent-filtered Native/Observed/Governed/Verified timeline |

### Common projection and native slots

The common projection defines only semantics every supported Adapter can
declare honestly: conversation identity, message identity/order, role, content
kind, timestamps, attachment refs, capability availability, source/freshness
and errors. Missing support is explicit. Native slots may render bounded vendor
history metadata or artifacts only. They cannot inject controls, action
handlers, executable markup/scripts, credentials, or authority-shaped state,
and they may disappear without changing common state. Vendor-specific actions
use Control Plane-owned components whose typed capability entry defines
preconditions, request semantics, result/receipt, and recovery. Undelivered
actions remain `Requires-backend`.

### Capability matrix contract

Every capability row has independent fields:

| Field | Values | Rule |
|---|---|---|
| Runtime condition | `Supported`, `Unsupported`, `Unavailable`, `Unknown` | evaluated from current adapter/runtime facts; includes reason and freshness |
| Delivery status | `Now`, `Requires-backend` | product delivery truth; `Requires-core` is a separate contract dependency note |
| Support path | `vendor-native`, `managed-adapter`, `MCP-cooperative`, `observable-only`, `unqualified` | origin and claim boundary; never inferred from runtime condition |

Runtime conditions mean:

- `Supported`: the integration declares the capability and has enough current
  information to evaluate it;
- `Unsupported`: the origin/integration does not provide it;
- `Unavailable`: it exists but current authentication, runtime, connection,
  policy, version, or dependency blocks it;
- `Unknown`: the integration cannot establish support or usability.

Support-path preference is `vendor-native -> managed-adapter ->
MCP-cooperative`. `MCP-cooperative` is a bounded MCP-plus-rules fallback and
cannot establish host-session inventory/control semantics. `observable-only`
can populate read state but exposes no action. `unqualified` is informative
only and cannot be presented as supported.

### Disconnect versus uninstall

- **Disconnect:** stop using the connection/Adapter route while retaining the
  installation and source-owned history according to real semantics.
- **Uninstall:** destructive lifecycle preview covering running work, Effects,
  retained history/data, credentials, package bytes and rollback/recovery.

Neither operation is currently an HTTP-backed Control Plane action. Both are
`Requires-backend`; do not render active controls.

The P7-T05 read-mostly dossier and dsh/runtime facts below remain current
implementation evidence. They are the fallback when conversation, catalog,
lifecycle, Work linkage or native history is unavailable—not proof those
target capabilities exist.

---

## Historical 2026-08-24 current-backed dossier specification

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
