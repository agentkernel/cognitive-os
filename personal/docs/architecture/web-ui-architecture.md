# CognitiveOS Personal Web UI Architecture

- Status: informative post-1.0 target/design
- Formal task: `P7-T05` (non-blocking Web UI)
- Change class: `product-semantic + structural` documentation
- Product companion: [Web UI product design](../product/web-ui-design.md)
- Reuse: [System architecture](system-architecture.md), [Resource Manager](resource-manager-architecture.md), [Provider Control Plane](provider-control-plane.md)

This design places a static React/TypeScript/Vite client in the external
`cognitiveos-clients` repository under `pc/web/`, as already anticipated by the
Personal plan. It does not add a daemon, database writer, public contract or
alternate authority. [ADR-0053](../../../docs/adr/0053-personal-web-ui-stack.md) accepts
that stack, same-origin daemon serving under `/ui/`, memory-only sessions, and
the browser Origin/Referer allowlist. The route inventory is
[web-ui-route-inventory.json](web-ui-route-inventory.json). SPA
implementation lives in the approved checkout `D:\cognitiveos-clients\pc\web\`
(official `agentkernel/cognitiveos-clients`). This repository must not recreate
`clients/**` or implement the UI in `apps/cognitiveos-console`.

## 1. Topology and trust boundaries

```text
Browser (static SPA, localhost)
        |
        | authenticated management session
        | authenticated task session
        v
Personal daemon loopback front door
        |
        +--> ResourceApplicationService
        +--> Provider Control Plane
        +--> Agent/Runtime lifecycle services
        +--> Task/Run/Activity projections and watch
        |
        +--> SecretStore and Provider egress (daemon only)
        +--> Authority SQLite / Event / Effect / Evidence stores
```

The browser is an untrusted client of the local daemon. It may hold a short-lived
UI session token according to the existing management-session design, but it
must not receive the daemon bootstrap secret, a Provider key, a SecretRef that
can be resolved by the client, a sidecar bearer or an ambient filesystem
credential. The daemon remains the only authority writer.

The first delivery binds the API to numeric loopback and is intended for the
same owner account. Existing daemon front-door authentication, channel
separation, request bounds and error envelopes remain authoritative.
[ADR-0053](../../../docs/adr/0053-personal-web-ui-stack.md) binds origin, CORS and CSRF:
cookies stay forbidden; the product origin is daemon-served `/ui/` on loopback;
CORS is not used; a present `Origin`/`Referer` must be the daemon's own
loopback origin. The SPA must not assume that a browser bearer or same-origin
deployment is itself a security boundary. The daemon front door enforces the
Origin/Referer allowlist (`LOCAL_ORIGIN_HEADER_REJECTED`) and serves `GET /ui`
from `data_dir()/ui` (503 `not_available` when the bundle is absent).

Session material is memory-only by default. The client must not persist
management/task tokens, bootstrap material or SecretRefs in localStorage,
sessionStorage, IndexedDB, URLs or browser history.

## 2. Client modules

```text
app shell / route guard
  -> query cache and projection normalizers
  -> management client      (resources, providers, bindings, lifecycle)
  -> task client             (preview, admit, attach, watch; cancel only if typed HTTP exists)
  -> activity watch client   (cursor, reconnect, dedupe, stale marker)
  -> redaction and display policy
  -> forms, tables, timelines and confirmation surfaces
```

The client keeps management and Task channel credentials, retry state, caches,
watch cursors and operation sets separate. A local cache is presentation state;
it is discarded or marked stale when the daemon reports a revision mismatch or a
watch gap. Query results carry the authority object version/projection version,
cursor, source and non-claim fields needed for honest rendering.

The UI should use a small typed client package generated or hand-mapped from
approved daemon envelopes. It must not duplicate canonical JSON, transitions,
error registries or authorization algorithms. Any new public envelope, route or
error must use the contract lane before implementation.

## 3. Projection and route composition

The Web UI consumes existing or separately approved management/task routes. The
expected logical groups are:

| UI capability | Daemon authority source | Route family (logical) |
|---|---|---|
| readiness and doctor | readiness/doctor projections | `/personal/status`, `/personal/readiness`, `/personal/doctor` |
| six-family inventory | ResourceApplicationService | `/management/resource/v1/list`, `/management/resource/v1/inspect`; existing watch `/resource/v1/watch` |
| Provider accounts/models | Provider Control Plane | current account paths `/management/providers/accounts[/inspect|/update|/delete|/key]`; model paths `/management/providers/models[/refresh|/add|/set-price]`; usage/budgets/alerts/audit projections |
| Agent bindings | Provider Control Plane + Runtime domain | `/management/agent-bindings` and typed `/management/agent-bindings/remove` |
| Task preview/admit/watch | TaskApplicationService | existing Task management/task channel |
| lifecycle controls | typed Runtime/Task workflows | existing management lifecycle operations; no generic state transition |
| activity stream | event/effect/process/task projections | existing versioned watch surfaces |

These names are design-level route groups, not permission to introduce a second
API shape. Exact envelopes, pagination, stable errors, channel checks and
versioning follow existing Personal contracts, Lane-CTR, and the frozen
[web-ui-route-inventory.json](web-ui-route-inventory.json). If a required
operation is not currently exposed as a typed daemon service, P7-T05 must show
an explicit unavailable/not-run state and record the missing dependency; it must
not add a generic browser-driven transition endpoint. Current honest gaps:
Task cancel HTTP; Agent pause/resume/stop/restart/quarantine HTTP. Tool
quarantine HTTP is not an Agent-instance control.

## 4. Provider connectivity path

```text
UI form
  -> typed provider validation request
  -> daemon validates endpoint/trust and session authorization
  -> daemon resolves SecretRef in approved SecretStore
  -> bounded Provider discovery/capability probe
  -> daemon persists redacted result/audit and catalog revision
  -> UI receives status, duration, error_class and next_action
```

Plaintext key material exists only in daemon memory for the minimum egress
operation. The browser receives no key, raw header, prompt, completion or raw
Provider response. Endpoint policy, DNS validation, redirect policy, response
limits and insecure/private-network confirmation are daemon decisions.

The UI distinguishes reachability, authentication, model discovery and capability
results. A failed refresh retains the prior catalog and binding. Missing usage or
pricing remains unknown/cost-unavailable rather than being fabricated as zero.
The UI maps the exact daemon status/error classes into display groups; it does not
mint a new Provider lifecycle state in the browser.

## 5. Agent binding and runtime control

Binding changes use the existing Provider Control Plane model:

```text
AgentInstance + expected binding revision
  -> account_id + provider_kind + model_id
  -> daemon preview / policy / CAS / idempotency
  -> durable mutation and audit
  -> updated binding projection
```

The UI cannot select a different Provider per request and cannot infer a binding
from a Pi session, process, Provider response or `agent_end` event.

Runtime controls are typed operations. The daemon decides whether pause, resume,
cancel, stop, restart or quarantine is currently allowed, persists any required
Intent/Effect before external mutation, fences epochs and returns the authority
projection. The browser renders the operation as pending, unknown,
reconciling, verified or failed; it never converts a successful HTTP response
into Task completion.

## 6. Watch, reconnect and consistency

Activity and long-running detail pages consume a versioned watch with a cursor.
The client must:

1. record the last accepted cursor and projection version;
2. reconnect with bounded backoff after transport loss;
3. deduplicate events by the daemon event identity;
4. detect a cursor gap or revision mismatch;
5. request a fresh bounded snapshot before resuming the watch;
6. mark the view stale while the snapshot is pending.

Detach stops observation only. It does not cancel a Task or stop an Agent.
Unknown Effect outcomes and not-run fields are first-class display states.

Process and Agent output is an observation stream only. It is bounded by the
daemon, escaped as untrusted text, redacted before leaving the daemon boundary
and linked to its Process/Event identity. The UI does not treat a log line,
stream close or exit code as an Effect result or Task completion.

## 7. Security and privacy requirements

- No direct database, SecretStore, filesystem, shell or Provider network access
  from browser code.
- No secrets or bearer tokens in URLs, localStorage, session history, telemetry,
  crash reports, exported support bundles or DOM text.
- Redact sensitive fields at the daemon boundary as well as in the client; UI
  redaction is not the security boundary.
- Enforce management/task channel isolation and owner-local scope on every
  request; fail closed on missing, stale or mismatched session state.
- Treat all Agent names, Provider metadata, event text, tool output and model
  content as untrusted display data; escape HTML and never execute returned
  markup or scripts.
- Confirmation surfaces display exact target IDs, expected versions, operation
  class, idempotency identity and rollback/reconciliation expectation.
- Support diagnostics are redacted facts and digests only. Browser telemetry is
  opt-in, disabled by default and cannot contain content or credentials.

## 8. Failure and recovery behavior

The SPA must degrade to a diagnostic client when the Provider, SecretStore, Pi,
Agent, sidecar or Task worker is unavailable. It shows stable error classes,
source, duration and next action, and preserves the distinction between unknown,
blocked, failed and not-run.

On daemon restart, the client discards session-scoped state, re-authenticates
through the approved local path and reloads projections. It does not attempt to
replay mutations from browser memory. Retry is allowed only with the original
idempotency identity where the daemon contract explicitly permits it.

## 9. Performance and operability targets

P7-T05 should measure, rather than assume, initial shell load, first useful
projection, Provider probe feedback, watch reconnect and table pagination on the
declared client route. Measurement is split from Provider/model latency and does
not create a performance Gate.

The build must produce a deterministic static bundle with pinned dependencies and
an explicit content-security policy (`default-src 'self'`). ADR-0053 chooses
daemon-served same-origin `/ui/` assets. A separately launched Vite preview is
not the product serving path. No public listener, CDN or runtime package
download is required.

## 10. Current-versus-target boundary

ADR-0053 and the route inventory are accepted D01 inputs. The SPA lives in
`D:\cognitiveos-clients\pc\web\`. D01–D07 delivered the localhost shell, forms,
and daemon-served `/ui/`. D08 closes the operator control-panel gap: Provider
kinds/trust/key SecretStore handoff, binding CAS, and Task
intent.record/interpret/preview/admit/watch. This document still does not claim
a Gate, release, Profile, or Agent-benefit result. Live Provider/SecretStore
and a real Agent run remain evidence for D09.
