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
  - path: apps/agent-shell/src/session.ts
    symbols: ["ShellSession"]
tests:
  - packages/sdk-ts/src/client.test.ts
  - packages/pi-cognitiveos/src/daemon-client.test.ts
  - apps/agent-shell/src/session.test.ts
fingerprint: "sha256:f43c186f4add0d27d98de529de9be44b526aa1cc8ab23b3b9313038b33ceaa79"
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
completion dispatch attaches an opaque `campaign-…` correlation id header
(client-side metadata the daemon ignores) and reports measured loopback and
daemon-supplied Provider-network durations plus real token usage — or
`not_available`; zeros are never fabricated. The
extension registers the provider bridge and tool policy documented in
[the Pi shell](../user/pi-shell.md).

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
