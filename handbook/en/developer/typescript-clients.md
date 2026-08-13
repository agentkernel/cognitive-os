---
doc_id: dev.clients-ts
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: packages/sdk-ts/src/client.ts
    symbols: ["TaskChannelClient", "ManagementChannelClient"]
  - path: packages/sdk-ts/src/channel.ts
  - path: packages/sdk-ts/src/watch.ts
  - path: packages/pi-cognitiveos/src/daemon-client.ts
    symbols: ["PersonalDaemonClient"]
  - path: packages/pi-cognitiveos/src/pi-route-observation.ts
    symbols: ["assemblePiRouteObservation"]
  - path: apps/agent-shell/src/session.ts
    symbols: ["ShellSession"]
tests:
  - packages/sdk-ts/src/client.test.ts
  - packages/pi-cognitiveos/src/daemon-client.test.ts
  - packages/pi-cognitiveos/src/pi-route-observation.test.ts
  - apps/agent-shell/src/session.test.ts
fingerprint: "sha256:a7816cf2d35618cc93c496cec19db841cf0269ae0927fc5c7e74e0e6d98162b7"
non_claims:
  - All TypeScript surfaces are candidate/observation clients; none can hold authority or complete Tasks.
---

# TypeScript clients

Three client layers, all strictly non-authority:

## `packages/sdk-ts` — AKP client SDK

Channel-isolated `TaskChannelClient`/`ManagementChannelClient` over generated contract types: request
envelopes carry protocol pin, idempotency key, canonical digests; responses map
registered error codes to typed errors. `channel.ts` prevents a task-channel
client from issuing management calls at the type level and at runtime.
`watch.ts` implements the bounded snapshot-first watch consumer with resume
cursors and gap detection (`RESUME_STALE` handling mirrors the daemon).
Transports: in-memory fake for tests plus loopback HTTP.

## `packages/pi-cognitiveos` — Pi extension client

`PersonalDaemonClient` does discovery (`daemon-endpoint.json` + bootstrap
secret), separate management/task session minting, health/status/doctor reads,
provider chat completion, resource projection/watch, and task watch — each with
bounded timeouts/sizes and typed `PERSONAL_*`/`PI_EXTENSION_*` errors. Each
completion dispatch attaches an opaque `campaign-…` correlation id header and
reports the measured loopback duration, the daemon-reported nested durations and
real token usage — or `not_available`; zeros are never fabricated.

Under an explicit campaign authorization the same dispatch also publishes one
`personal-pi-route-observation/1` record: five sequential Pi-domain stages from a
recorder that cannot open two stages at once, plus the two daemon-domain stages
nested inside the loopback wait and joined by the echoed correlation id. The
daemon echoes that id and reports preflight only when its own environment also
has `COGNITIVEOS_PI_ROUTE_OBSERVATION=enabled`; otherwise the nested pair degrades
to `not_available`. The two
clock domains are never added or subtracted; the only relation asserted across
them is containment. Daemon stages that are unreported, unechoed, mismatched,
half-reported or larger than the wait that contains them are dropped with a
reason rather than trimmed or estimated. Instrumentation is denied by default,
holds no filesystem or authority surface (a durable sink is an injected port, and
a sink inside a Personal root is refused), and publishes nothing that is not a
label, an opaque id, a duration or a counter. The extension registers the
provider bridge and tool policy documented in
[the Pi shell](../user/pi-shell.md).

The record also carries `requestMode`, `outcome`, `terminalStage` and a fixed
content-free `failureClass`. Completed requests require all five Pi stages;
cancelled/error requests retain only the exact measured prefix. The Provider
route is non-streaming (`stream:false`); `stream:true` is a stable refusal before
secret resolution. Measured usage has an in-process provenance marker created
only by the authenticated daemon-response parser, so an embedding runner cannot
publish self-asserted counters. That prevents instrumentation-side fabrication;
it does not cryptographically attest an upstream Provider's accounting.

## `apps/agent-shell` — session library

`ShellSession` drives preview → submit (admit) → attach/cancel against the AKP
surface with an explicit state machine, disconnected-buffer replay, and
idempotent submission (same preview digest ⇒ same task). It is a library with
tests, not a shipped TUI.

Shared invariants: no secret material ever reaches these layers (bearer tokens
are process-local session tokens, not Provider keys); every mutation-shaped call
carries an idempotency key; all list/watch surfaces are bounded; JSON parsing is
schema-shaped via `packages/contracts-ts` generated types with canonical-digest
parity against Rust (`tests/golden/`).
