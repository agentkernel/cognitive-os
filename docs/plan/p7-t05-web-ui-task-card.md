# P7-T05 Web UI Task Card

- Task: `P7-T05`
- Title: Non-blocking Personal Web UI
- Phase: 7 Productization, post-1.0
- Status: `in-progress` (D10 live-proven on linux-002; D01–D09 delivered; Draft until acceptance)
- Approved checkout: `D:\cognitiveos-clients` (`pc/web/`, official
  `agentkernel/cognitiveos-clients`)
- Lease: `lease/personal/P7-T05/web-ui-sidebar-fix` (active; prior control-panel lease closed 2026-08-23)
- Stack ADR: [ADR-0053](../adr/0053-personal-web-ui-stack.md)
- Route inventory: [web-ui-route-inventory.json](../architecture/personal/web-ui-route-inventory.json)
- Priority: non-blocking for Linux 1.0 / `GMVP-LINUX`
- Product design: [Web UI product design](../product/personal/web-ui-design.md)
- Architecture: [Web UI architecture](../architecture/personal/web-ui-architecture.md)
- Formal plan anchor: [P7-T05 in `plan.md`](./plan.md#p7-t05--web-ui)

This card decomposes the existing formal task without changing its status,
dependencies or release scope. Delivery remains one formal task, one branch,
one Draft PR and one lease; D01-D09 are task-internal slices, not independent
tasks or PRs.

## 1. Objective

Deliver a localhost-only, single-owner Web UI client that lets an operator inspect
installed Agents, configure and probe Provider accounts, bind one fixed Provider
account/model to an Agent, and supervise or inspect Agent/Task activity through
daemon projections and typed operations.

The UI must remain useful for diagnostics when Provider, SecretStore, Pi, Agent,
sidecar or worker state is unavailable. It must never become an authority writer
or imply completion from a Provider response, process exit or client receipt.

## 2. Dependencies and gates

Formal implementation dependencies remain `P2-T08` and `P7-T03` as recorded in
the Personal plan. Before implementation starts, the task also requires:

- client readiness review and a technical stack ADR;
- stable, authenticated daemon management and Task routes for the projections;
- legal/licensing review for the selected frontend dependencies;
- declared client validation route and browser security baseline;
- confirmed external repository path `cognitiveos-clients/pc/web/`;
- an explicit decision for browser session storage, origin/CORS/CSRF handling and
  the local static-bundle serving path.

D01 accepted [ADR-0053](../adr/0053-personal-web-ui-stack.md), froze the
[route inventory](../architecture/personal/web-ui-route-inventory.json), and
declared the validation route. Owner approved cloning the official existing
repository. Approved checkout: `D:\cognitiveos-clients` (`pc/web/`). Do not
recreate `clients/**` in this repository and do not implement the SPA in
`apps/cognitiveos-console`.

## 3. In-scope acceptance

### Agent inventory and lifecycle

- List and inspect installed Agent package, installation, registration, instance,
  sidecar, execution and process identities separately.
- Show health, adapter/protocol digest, lifecycle state, Provider binding state,
  current Task count, drift and blocked reason.
- Expose typed pause, resume, cancel, stop, restart and quarantine controls only
  when the daemon projection allows them; show preview, expected version and
  reconciliation state for mutations. If a typed service is unavailable, render
  the control as unavailable/not-run; do not add a generic lifecycle endpoint.

### Provider configuration and connectivity

- Create, update/rotate and remove named OpenAI, Anthropic and OpenAI-compatible
  accounts through the daemon control plane.
- Keep API keys exclusively in the approved SecretStore path; prove browser,
  URL, storage, log and error redaction with focused negatives.
- Run explicit bounded reachability and model/capability probes, showing
  duration, source, error class, catalog revision and next action.
- Preserve the last catalog and binding after a failed refresh; render unknown
  usage/pricing as unknown or `cost_unavailable`.
- Render the daemon's exact account status/error classes distinctly, including
  usable, degraded, revoked, locked/unresolvable and unknown outcomes when those
  facts are available; these are UI groupings, not new persisted states.

### Agent Provider binding

- Show and mutate at most one active fixed `account + provider + model` binding
  per Agent instance with revision/CAS and idempotency handling.
- Reject unbound, revoked, degraded or stale bindings fail closed.
- Prove there is no fallback, automatic routing or per-request Provider override.

### Runs and activity

- Render Task, Run, Process, Effect, Evidence and Event projections separately.
- Support cursor-based watch, reconnect, dedupe, snapshot refresh on gaps and an
  explicit stale/disconnected state.
- Show bounded, escaped and daemon-redacted Process/Agent output with its source
  identity; never treat output, stream close or exit code as completion.
- Distinguish queued/running/blocked/reconciling/verifying/completed/failed/
  cancelled/quarantined; never infer completion in the browser.

### Client quality

- Keyboard-complete navigation and forms, visible focus, semantic tables and
  status announcements for watch updates.
- Loading, empty, denied, stale, disconnected, unknown and not-run states for
  every primary projection.
- Pinned deterministic static build, content-security policy and no runtime CDN
  dependency.

## 4. Task-internal delivery slices

| Slice | Focus | Required exit evidence |
|---|---|---|
| D01 | Readiness, technical ADR, route/contract inventory, session/serving decision and threat model | **done:** ADR-0053 accepted; inventory + Node negatives; Origin/Referer allowlist and `GET /ui` missing-bundle `not_available` on the daemon front door. |
| D02 | SPA shell, local auth/session bootstrap, Home and Agent inventory/detail | management/task channel isolation, redacted readiness, Agent identity separation |
| D03 | Provider account forms, SecretStore handoff and connectivity/model probes | secret-leak negatives, endpoint policy errors, probe result semantics |
| D04 | Agent Provider binding and revision-aware confirmation | fixed binding, CAS/idempotency, no-fallback negatives |
| D05 | Task/Run/Process/Effect/Evidence/Events views and lifecycle controls | watch reconnect/gap behavior, typed mutation preview, completion non-claim |
| D06 | Accessibility, responsive layout, security hardening and performance instrumentation | keyboard/accessibility checks, CSP, bounded latency measurements, dependency review |
| D07 | Integration, supported validation, docs sync and final acceptance assessment | exact revision validation, focused negatives, task docs and closure record |
| D08 | Live control-panel completion: Provider key SecretStore handoff, binding CAS, Task preview/admit/watch | focused negatives; SPA unit/DOM; no invented lifecycle HTTP |
| D09 | Exact-revision Linux UI driver through key entry, bind, and Agent Task run | live SecretStore/UI driver; redaction; cleanup; hypothesis only |
| D10 | Owner-reported sidebar no-op plus Provider key / Agent binding verification | failure-first hash-nav tests; in-place session gate; linux-002 click-through; key/bind take-effect evidence |

The first implementation slice changes the task status to `in-progress` under the
normal task lease. Design-only work before that point does not claim execution.

## 5. Focused negative tests

At minimum, the final task must include negatives for:

- API key or SecretRef appearing in DOM, URL, storage, logs, telemetry, errors or
  support output;
- session or bootstrap material appearing in localStorage, sessionStorage,
  IndexedDB, browser history or exported client state;
- browser attempting direct SQLite, SecretStore, filesystem or Provider access;
- management bearer used on Task routes or Task bearer used on management routes;
- stale Agent/binding/projection version accepted by a mutation;
- unbound, revoked or degraded Agent dispatched to a Provider;
- fallback, alternate per-request Provider selection or arbitrary header injection;
- lost watch cursor causing a fabricated final state rather than snapshot reload;
- process exit, Provider response, Pi event or HTTP receipt rendered as Task
  completion;
- unbounded, unescaped or unredacted Process/Agent output displayed or executed;
- cancelled/detached observation incorrectly stopping durable work;
- unsafe markup or script returned in Agent/Provider/Event text executing in the UI.

## 6. Validation and evidence boundary

Supported validation must run on the declared browser/client environment and on
the exact revision under review. It should cover static build, unit/component
tests, route-contract tests, browser integration journeys, security negatives,
accessibility checks and bounded watch/probe behavior. Linux-native daemon
validation remains governed by the existing Personal environment route; a local
browser smoke test cannot substitute for daemon evidence.

All local, fixture and ordinary CI results remain implementation evidence only.
No slice or task result may be promoted to a Gate, release, Profile or
Agent-benefit claim. Unavailable environment or credentials are recorded as
`not-run`/`not_available`, not as pass.

## 7. Non-goals and rollback

The task does not add remote administration, multi-user RBAC, OAuth/SSO,
marketplace acquisition, Multi-Agent orchestration, MCP/dynamic Tool management,
Windows UI parity, a second API/DB writer or any Linux 1.0 release dependency.

The client can be disabled or removed without changing authority data. A failed
client bundle or route rollout must leave CLI, Pi Shell, daemon and Provider
control-plane paths usable. Browser caches and UI telemetry are disposable;
authority records and audit facts remain daemon-owned.

## 8. Completion checklist

- [x] Product and architecture documents linked from both Personal README indexes.
- [x] Technical stack ADR and client readiness decision accepted (ADR-0053; checkout missing ⇒ D02+ blocked).
- [x] D01 inventory maps UI capabilities to approved daemon channels; missing ops are `unavailable`/`not-run`.
- [x] Missing daemon operations recorded as unavailable/not-run; no generic browser transition invented.
- [x] Focused security, channel-isolation, stale-version and completion negatives pass (D01 inventory + SPA unit/DOM redaction).
- [x] Supported browser/client validation passes on the exact task revision (Linux daemon-served `/ui/`; live Provider key entry required for D09).
- [x] Accessibility, CSP, dependency and bounded performance checks authored in `pc/web/` (keyboard/focus, CSP meta, pinned MIT deps, fetch latency display).
- [x] Formal plan, task status, docs impact and final report are synchronized.
- [x] P7-T05 remains explicitly non-blocking for Linux 1.0 and no Gate/release claim is added.
