---
doc_id: dev.daemon-http-surface
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/server.rs
    symbols: ["serve_personal_loopback", "PersonalDaemonConfig"]
  - path: apps/kernel-server/src/personal/auth.rs
    symbols: ["LocalSessionAuthority", "ChannelClass"]
  - path: apps/kernel-server/src/personal/bounds.rs
  - path: apps/kernel-server/src/personal/readiness.rs
    symbols: ["evaluate_personal_readiness"]
  - path: apps/kernel-server/src/personal/provider_proxy.rs
  - path: apps/kernel-server/src/personal/route_observation.rs
    symbols: ["observation_response_headers"]
tests:
  - apps/kernel-server/tests/p1_t04_personal_daemon.rs
  - apps/kernel-server/tests/p1_t05_personal_readiness.rs
  - apps/kernel-server/tests/p1_t07_provider_proxy.rs
  - apps/kernel-server/tests/p9_t07_route_observation.rs
fingerprint: "sha256:3d8e78c2d0a1aeb0ec5881c93eb1268a3e99b335dd2f5077a6d3a661a0b57dad"
non_claims:
  - Route inventory lives in the generated HTTP reference; this page explains composition, not completeness.
---

# Daemon and HTTP

## Startup order (load-bearing)

`serve_personal_loopback`: lexical loopback check → XDG layout → database
preparation/migrations → `daemon.lock` acquisition → one `SqliteAuthorityStore`
open (+ a separate `SchedulerRepository` connection to the same file) → recovery of
consumed worker handoffs → one bounded ArtifactStore at `data_dir()/artifacts` →
native Tool descriptor/router composition sharing that CAS → bootstrap secret load/create →
TCP bind → atomic `daemon-endpoint.json` publication → one periodic scheduler
worker → thread-per-connection serving. No scheduler pass runs before the listener and
endpoint exist, so a Task admitted by this process can be observed by a later
pass. The worker owns the scheduler connection, runs serial fixed-delay 250 ms
passes behind a non-reentrant gate, logs and retries pass-level failures, and is
explicitly cancelled, unparked, and joined on orderly exit. Row-local failures
remain isolated inside each pass. There is still no HTTP shutdown route (see
[execution-chain status](./execution-chain-status.md)).

## Authentication

Two credential planes, deliberately unrelated:

- **Local channel bearers** (this surface): `POST /local/session` exchanges the
  per-boot bootstrap secret for a `management` or `task` token; every
  authenticated route checks channel binding first. Process-local, 12 h/30 min
  expiries, no per-action scopes. Bootstrap and session tokens each use 256 bits
  from the OS CSPRNG; entropy failure or an invalid/repeated probe fails before
  file/session creation, with no PID/time/hash fallback. Bootstrap reload accepts
  only the current lowercase `boot-32hex-32hex` shape, so legacy predictable or
  malformed non-empty credentials stop startup instead of being grandfathered.
- **Privileged management sessions** (`admin-cli`): JSON documents validated by
  `cognitive-management` — a separate plane, not interchangeable with local
  bearers.

## Request hygiene

Fixed bounds before routing: 1 MiB body (8 MiB hard read), 16 KiB/64 headers,
10 s/30 s timeouts, 32/16 connection caps, Cookie rejection, optional Host
validation — each with a registered error code. Routing is handwritten prefix
matching on `METHOD /path` strings across `server.rs`, `task_api.rs`, and
`resource_api.rs` (the generated [HTTP reference](../reference/http-api.md)
enumerates the full table and channels).

The management Resource surface exposes a read-only lifecycle-preconditions
document, sealed Context-source admission, Memory remember/review/forget, and
Skill import/inspect/bind/supersede/revoke. Mutations require a management
bearer; task bearers fail before handlers run. Successful creation responses
use HTTP status `201`, and durable rows remain inspectable after restart.
The task channel reads the latest daemon-authored Memory/Skill consumption
through `GET /task/resource/v1/consumption?task_ref=…`: exact pins, session
linkage, and `reuse_of` only. `query_text` and `skill_binding_id` are
rejected as restatement. Forgotten, revoked, or digest-drifted pins fail
closed before the response, and Memory/Skill bodies never appear.

## Projections

Readiness evaluates six components from filesystem/config facts (`blocked |
degraded | ready` + `first_conversation_ready`); it never sends a Provider
request. It does resolve the configured `secret_ref` against the SecretStore,
because a reachable backend does not mean the reference still points at a
stored item: a dangling reference reports `secret_ref_resolves: false` and
blocks with `provider_secret_unresolvable`, and a backend that cannot answer
blocks with `provider_secret_store_unavailable`. Resolved material is dropped
immediately and never enters a fact. Resolution uses the already-loaded
Provider config snapshot; it never reloads `provider.json` and combine a newer
secret reference with the older provider/model/digest facts. One status or
doctor evaluation binds one SecretStore: the secret probe and the provider
`secret_ref` resolve share that backend, skip `get` when the probe already
proved the backend cannot answer, drop material immediately, and do not cache
readiness across requests (no stale-ready TTL). Doctor adds redacted
six-resource/vault/operability sections. The Provider
proxy validates config + selected model, resolves the secret in memory, and
forwards via the bounded Rustls transport. Successful proxy responses always
carry `X-CognitiveOS-Provider-Network-Nanos`. Nested preflight timing and the
correlation echo are denied unless `COGNITIVEOS_PI_ROUTE_OBSERVATION=enabled`
and the request carries one well-formed opaque correlation id; malformed or
duplicate ids are ignored, the product body is unchanged, and the observer
writes nothing. The private one-shot Unix socket
(`POST /chat/completions`) serves only the daemon-launched Pi candidate process
and forbids Authorization headers.

## Non-Personal skeleton

`kernel-server --once/--serve` is an M0-era AKP/shell HTTP skeleton (placeholder
semantics, errors as HTTP 200). It is not the Personal surface; treat it as
historical scaffolding used by SDK live tests.
