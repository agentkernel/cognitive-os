# 03 — Control Plane Capability Model

- Status: adopted Personal 2.0 capability model; historical current-state model retained
- Updated: 2026-08-27
- Purpose: turn the audited [Capability Inventory](control-plane-capability-inventory.md) (current-state ratings) into the **product capability model** the redesign is allowed to design with: the capability domains, the operator actions inside each, the state vocabularies, and the honesty class of every action. This is the contract between product design and backend reality.
- Rule inherited from the workflow skill: **CognitiveOS Reality outranks every design authority.** Nothing below invents a route, a lifecycle, or a state.

## Personal 2.0 capability model amendment

The target uses six product spaces and separates **placement** from
**availability**. A target capability may be specified before implementation,
but it must be labeled `Requires-backend` and rendered as explanatory target
content—not as an enabled or disabled control.

| Target domain | Target capability | Current implementation | Target treatment |
|---|---|---|---|
| Home | resumable conversations, attention, readiness, verified outcomes | readiness and partial task/provider facts | compose only real facts; broader resume/attention projection Requires-backend |
| Agents | catalog install/connect, Adapter-backed native conversation/history, Runtime dossier | runtime envelopes, bindings, dsh snapshot; no lifecycle HTTP | current facts remain read-only; catalog/lifecycle/history projection Requires-backend |
| Work | Goal, Plan revisions, Tasks/attempts, Context, multi-Agent orchestration | governed task chain and bounded per-task facts; no Goal/Plan or rich inventory | current task view may ship; Goal/Plan/orchestration controls Requires-backend |
| Library | Memory, Skills, Tools, MCP | partial Memory/Skill/Tool APIs; MCP not a product family today | each unsupported family facet Requires-backend; MCP gets a first-class target page and Requires-core + Requires-backend |
| Activity | one Native/Observed/Governed/Verified timeline | provider audit + bounded per-task facts | coverage labels are mandatory; unified provenance feed Requires-backend |
| Settings | Account Hub, System, appearance/accessibility, diagnostics | API-key accounts, models, bindings, readiness, backup/restore | move current Provider/System views here; OAuth/subscription/import/custom gateway and rich quota/cost Requires-backend |
| Global Agent Shell | explain state, navigate/compare, propose next action, request daemon preview, suggest conflict resolution | no cross-space Control Plane Shell | Shell projection/action composition Requires-backend; vendor-native conversation remains in Agents |

### Capability classes for target design

1. **Now / direct** — a verified typed route/projection exists; an active
   control may be specified with its real preconditions.
2. **Now / composed** — the client may combine verified facts, with visible
   source and coverage limits. Composition is source/coverage metadata, not a
   separate delivery status.
3. **Requires-backend** — adopted target semantics without current typed
   support. Show the intended outcome, dependency, and non-interactive preview;
   never draw an active-looking control or progress indicator.
4. **Forbidden** — violates daemon-only authority, secret isolation, provider
   proxying, or host-session boundaries; never render.

The existing A/B/C/D taxonomy below is retained as the 2026-08-24 current-state
classification. For Personal 2.0 documents, `Requires-backend` replaces vague
"deferred verb" language and covers non-control projections as well as actions.

### Adapter capability matrix contract

Adapter capability rows do not use one overloaded status:

| Dimension | Values | Question answered |
|---|---|---|
| Runtime condition | `Supported`, `Unsupported`, `Unavailable`, `Unknown` | Can the current adapter/runtime establish this capability now? |
| Delivery status | `Now`, `Requires-backend` | Does Personal currently deliver the required projection/action? |
| Support path | `vendor-native`, `managed-adapter`, `MCP-cooperative`, `observable-only`, `unqualified` | Which integration path supplies the fact/action, and at what claim boundary? |

`Requires-core` is an additional contract dependency, not a runtime condition
or delivery status. Runtime `Unsupported` never means "not built yet";
`Unavailable` requires a current blockage reason; `Unknown` stays unknown.
Only a `Now` row with typed Control Plane action semantics can produce an
action control.

### Non-negotiable target capability boundaries

- Adapter projections normalize Agent conversation/history, but native slots
  render display-safe vendor metadata/artifacts only. They cannot inject
  actions, executable markup/scripts, credentials, or authority-shaped state.
  Vendor actions use Control Plane-owned controls backed by typed capability
  semantics.
- Conversation does not become managed work until the owner explicitly chooses
  Manage with Personal and the daemon returns durable Goal/Plan/Task facts.
- A federated resource writeback is current-backed only when a typed daemon
  preview/confirm/effect path exists. Shell suggestions are candidates.
- MCP discovery/configuration never grants Tool, filesystem, model, or host
  session authority.
- Account credentials enter only through approved daemon-owned paths. Import
  follows
  [ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md)
  and is currently Requires-backend.
- Progress is shown only from a real source with known semantics. Otherwise use
  a pending/unknown/Requires-backend state, never an estimated bar.

---

## Historical 2026-08-24 capability model

The sections below preserve the P7-T05-centered capability classification and
backend dependency analysis. The Personal 2.0 three-axis Adapter matrix and
six-space placement above supersede its product-domain framing.

## 1. Capability domains

The redesign organizes operator capability into seven domains. These are *product* domains (how the owner thinks), each mapped to its *authority source* (what the daemon actually exposes). They are candidates for IA treatment, not yet navigation.

| # | Domain | Operator question | Authority source (verified) | Honesty class |
|---|---|---|---|---|
| D1 | **System & Readiness** | Is the system ready? What is blocked? | `/personal/{health,status,readiness,doctor}`; doctor sub-sections placeholder | AVAILABLE core / PARTIAL detail |
| D2 | **Work (Tasks & Runs)** | What work exists, what state is it in, what happened? | `/task/*` (record/interpret/preview/admit/candidate/watch/evidence/effects/observation); Resource Manager `list?family=task` | AVAILABLE for governed chain / PARTIAL for inventory & live / NOT AVAILABLE for cancel·pause·retry |
| D3 | **Agents** | What is installed, what is it doing, how is it governed? | `resource/v1/list?family=runtime` + inspect; agent-bindings; dsh runtime snapshot | PARTIAL (read-mostly; lifecycle NOT AVAILABLE over HTTP) |
| D4 | **Providers & Models** | Which accounts/models exist, are they reachable, what do they cost? | `/management/providers/*`, `/management/agent-bindings`, `/management/{usage,budgets,alerts,audit}`, `/provider/v1/*` | AVAILABLE (budgets observe-only) |
| D5 | **Cognitive Resources** | What do my agents know and may use? | memory remember/forget/object; skill import/bind/revoke/explain; tool catalog/lifecycle/exposure/selection; Resource Manager envelope | AVAILABLE for memory/skill/tool cores / PARTIAL for search, review queues, context browsing |
| D6 | **Activity & Evidence** | What happened, in what order, with what proof? | `/task/{watch,evidence,effects,observation}`; `/management/audit` (provider-scoped) | PARTIAL (per-task strong; unified feed NOT AVAILABLE) |
| D7 | **System Stewardship** | How do I back up, restore, upgrade, recover? | `/management/resource/v1/{backup,backup/preflight,restore}`; context-authorization facts | AVAILABLE for backup/restore / PARTIAL for upgrade (CLI-only) / NOT AVAILABLE for service control |

Cross-cutting display objects (never domains of their own, per the canonical model): Budget, Permission, Model, Artifact, Intent/Effect, Evidence, Event.

---

## 2. Action model

Every operator action in the redesigned surface must belong to one of four honesty classes. This taxonomy is the redesign's capability-honesty contract, extending the shipped UI's existing behavior.

| Class | Meaning | UI treatment (binding rule) |
|---|---|---|
| **A — Governed mutation** | Typed daemon route exists; mutation goes through preview/CAS/idempotency where the daemon defines them | Full action UI: preview → confirm with exact IDs/versions → receipt → refreshed projection |
| **B — Direct safe action** | Typed route exists; read-only or low-risk (probe, acknowledge, list) | Inline action with immediate feedback |
| **C — Deferred verb** | The operation exists in the product model but has no typed HTTP route (task cancel; agent pause/resume/stop/restart/quarantine; session revoke) | Rendered as `Not available over HTTP` with the owning gap + CLI path; never a disabled button that pretends |
| **D — Forbidden by design** | Generic create/install/execute/complete; browser task completion; cross-channel access; campaign hooks | Never rendered as an action at all |

Current action census by class (from the inventory):

- **A:** provider account create/update/delete/key; model refresh/add/set-price; binding set/remove (CAS); dsh apply (CAS); task admit; memory remember/forget; skill import/bind/revoke; tool enable/disable/quarantine/revoke; tool selection; budget set/remove; backup; restore; context-authorization admission.
- **B:** all list/inspect/explain; readiness/doctor/status; usage/alerts/audit queries; alert acknowledge; watch attach/detach; dsh runtime read.
- **C:** task cancel/pause/resume/retry; agent install/activate/pause/resume/stop/recover/upgrade/rollback/uninstall (HTTP); session logout; memory content search; unified activity feed; management-action audit beyond provider plane.
- **D:** the eight forbidden routes; campaign-gated fault/http-origin hooks in product UI.

---

## 3. State model (the vocabularies the UI must render faithfully)

The redesign's state vocabulary is the union of the daemon's own vocabularies. No new authority states may be minted in the browser; display groupings must be labeled as such.

| Object | States (as verified) | Source |
|---|---|---|
| System readiness | overall `blocked \| degraded \| ready`; per-component `ready \| degraded \| blocked \| not_configured`; `first_conversation_ready` bool | `readiness.rs:43-81,177-190` |
| Task lifecycle | `ACTIVE → CANDIDATE_COMPLETE → COMPLETED` (store); no DRAFT over API | `continuation.rs:435,483` |
| Task interaction states (documented product vocabulary) | proposed / awaiting clarification / awaiting admission / queued / running / waiting / suspended / blocked / reconciling / verifying / completed / failed / cancelled / quarantined | `product-design.md:226-239`, `web-ui-design.md:140-144` — *display vocabulary; only a subset is currently observable per task* |
| Effect stage | `NOT_EXECUTED \| DENIED \| PROPOSED \| AUTHORIZED \| EXECUTING \| EXECUTED \| RECONCILED \| VERIFIED \| VERIFY_FAILED \| OUTCOME_UNKNOWN` (+ synthetic MISSING) | `observation.rs:761-831` |
| Effect outcome / reconcile | `not_executed \| executed \| failed \| indeterminate`; `not_applicable \| must_reconcile \| pending_reconciliation \| closed` | same |
| Provider account | `active \| revoked \| degraded` (+ secret presence; `secret_ref_resolves: true\|false\|unknown` in readiness) | `provider_control_plane.rs`, P2-T11 |
| Binding | `active \| revoked` + monotonic `revision` (CAS) | `provider_control_plane.rs:1299-1307` |
| Tool lifecycle | `enabled(1) \| disabled(2) \| quarantined(3) \| revoked(4)`; revoked terminal; quarantined↛enabled | `tool_lifecycle.rs:471-478` |
| dsh runtime | `ACTIVE \| INACTIVE \| CRASHED` + `process_alive` | `task_api.rs:766-803` |
| Agent adapter (not HTTP-exposed) | `Registered \| Active \| Paused \| Stopped` | `agent_adapter_manifest.rs:50` — *CLI/store only; UI must not present these as live HTTP facts* |
| Watch | `live \| stale \| disconnected \| reconciling \| unknown` (client controller) + daemon 409 resume-stale | `watch.ts`, `task_api.rs:1029-1066` |
| Load/display | `loading \| ready \| empty \| denied \| disconnected \| unknown \| not-run` | shipped SPA `App.tsx:27-32` — retained and extended by the redesign |

**State honesty rules (binding on all later docs):**

1. Unknown and not-run are first-class states, never zero, never blank, never green.
2. A process/observation state never upgrades a task/authority state.
3. Every state badge answers "source?" (which projection/fact) on drilldown.
4. Stale data is labeled stale with its cursor/age; a disconnected watch never fabricates a final state.

---

## 4. Capability → mode → priority matrix

Which capability domains serve which user modes, at what priority for the first implementation wave (feeds IA options):

| Domain | Operator (primary) | Power User | System Operator | Wave-1 priority |
|---|---|---|---|---|
| D1 System & Readiness | ●● | ● | ●● | P0 |
| D2 Work (Tasks & Runs) | ●● | ●● | ● | P0 |
| D6 Activity & Evidence | ●● | ● | ●● | P0 (per-task depth; unified feed deferred, backend dependency) |
| D4 Providers & Models | ● | ●● | ●● | P0 (already the strongest flow; keep and polish) |
| D3 Agents | ●● | ● | ● | P0 read depth; controls class-C honest |
| D5 Cognitive Resources | ● | ●● | ○ | P1 (memory/skill/tool depth; context browsing deferred) |
| D7 Stewardship | ○ | ○ | ●● | P1 (backup/restore surface; upgrade stays CLI) |

(●● primary serve, ● secondary, ○ incidental)

---

## 5. Named backend dependencies (the "do not design around these silently" list)

Each class-C capability gets a named dependency so future phases can schedule backend work instead of UI theater:

1. **BD-1 Task control route** — typed HTTP for cancel (and eventually pause/resume) on `TaskApplicationService.control`. Unblocks: task intervention UI.
2. **BD-2 Agent lifecycle routes** — typed management HTTP for the Pi sidecar lifecycle verbs that today exist only in admin-cli/runtime library. Unblocks: agent control UI.
3. **BD-3 Task inventory projection** — a real list/search over tasks (beyond the 64-row envelope) with objective text and state. Unblocks: the Work domain's list page.
4. **BD-4 Live watch deltas** — resource watch publishing mutation deltas; task watch snapshot populated. Unblocks: live UI without polling.
5. **BD-5 Unified activity/audit feed** — cross-domain, time-ordered authority events incl. non-provider mutations. Unblocks: the Activity domain as designed.
6. **BD-6 Memory search/review routes** — FTS-backed search and a proposal review queue over HTTP. Unblocks: memory curation UI.
7. **BD-7 Session lifecycle** — logout/revoke + expiry introspection. Unblocks: session chrome honesty.
8. **BD-8 Budget enforcement hooks** — if ever desired, enforcement in the proxy path; until then budgets render observe-only.
9. **BD-9 Browser session bootstrap ergonomics** — a sanctioned path that avoids pasting the bootstrap secret on every reload without weakening ADR-0053 (e.g. short-lived re-issue within idle window). Requires owner + security review; not assumed.

These dependencies are *recorded*, not scheduled; scheduling them is a plan-owner decision outside this design phase.

---

*Feeds: `04-control-plane-conceptual-model.md` (objects behind these capabilities), `05-control-plane-ia-options.md` (domains → structures), `07-control-plane-user-flows.md` (actions → flows).*
