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
  - path: apps/kernel-server/src/personal/fault_profile.rs
    symbols: ["handle"]
  - path: apps/kernel-server/src/personal/tool_lifecycle.rs
    symbols: ["handle"]
  - path: apps/kernel-server/src/personal/pinned_https.rs
    symbols: ["handle"]
  - path: apps/kernel-server/src/personal/observation.rs
    symbols: ["handle"]
  - path: apps/kernel-server/src/personal/user_backup.rs
    symbols: ["handle"]
  - path: apps/kernel-server/src/personal/task_api.rs
    symbols: ["TaskApi"]
tests:
  - apps/kernel-server/tests/p1_t04_personal_daemon.rs
  - apps/kernel-server/tests/p1_t05_personal_readiness.rs
  - apps/kernel-server/tests/p1_t07_provider_proxy.rs
  - apps/kernel-server/tests/p9_t07_route_observation.rs
  - apps/kernel-server/tests/p2_t24_effect_fault.rs
  - apps/kernel-server/tests/p2_t25_tool_lifecycle.rs
  - apps/kernel-server/tests/p2_t26_observation_plane.rs
  - apps/kernel-server/tests/p2_t27_backup_restore.rs
  - apps/kernel-server/tests/p2_t31_live_daemon_scheduler.rs
  - apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - apps/admin-cli/tests/p2_t33_private_candidate_host_path.rs
fingerprint: "sha256:fc791585387f86ec20adb777c94a1d47606d9faf059f597910936708ebdc7ab8"
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
pass. Public `POST /task/admit` persists owner-local Context authorization for
tenant `personal` so that later pass can resolve Context instead of skipping
before Pi. HTTP `TaskApi` clones that same `SqliteAuthorityStore` handle (shared
connection mutex) rather than opening a second writer per request, so the
periodic tick observes the facts admit just wrote. The worker owns the scheduler connection, runs serial fixed-delay 250 ms
passes behind a non-reentrant gate, logs and retries pass-level failures, and is
explicitly cancelled, unparked, and joined on orderly exit. Row-local failures
remain isolated inside each pass. There is still no HTTP shutdown route (see
[execution-chain status](./execution-chain-status.md)). `cognitive daemon start`
appends this process's stdout/stderr to `state/cognitiveos/daemon.log` (mode
`0600`); systemd `Type=simple` still uses the journal.

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
enumerates the full table and channels). Authenticated `POST /task/akp/dsh`
is a candidate-only DeepSeek Harness front door: sessions are process-local
and must be activated after start; daemon restart forgets them and fails
closed. Workspace* candidates reuse the existing public candidate admission
path. A dsh response never completes a Task.

The management Resource surface exposes a read-only lifecycle-preconditions
document, Memory remember/review/forget, and Skill import/inspect/bind/
supersede/revoke. Public remember accepts unsealed owner fields and the daemon
composes sealed headers from its persisted `GovernanceSeed`; a sealed
source+candidate envelope remains valid. Callers must not mint sealed headers
on the unsealed path. Mutations require a management
bearer; task bearers fail before handlers run. Successful creation responses
use HTTP status `201`, and durable rows remain inspectable after restart.
The task channel reads the latest daemon-authored Memory/Skill consumption
through `GET /task/resource/v1/consumption?task_ref=…`: exact pins, session
linkage, and `reuse_of` only. `query_text` and `skill_binding_id` are
rejected as restatement. Forgotten, revoked, or digest-drifted pins fail
closed before the response, and Memory/Skill bodies never appear. Session 2
and post-restart GET read the same durable row; a caller `query_text` POST
cannot replace those pins.

Management `POST/GET /management/resource/v1/fault-profile` persists a
default-off, campaign-authorized fixed fault profile for one `task_ref`.
Ordinary task callers are denied (`RESOURCE_FAULT_PROFILE_CHANNEL_FORBIDDEN`).
The task channel reads bounded Effect history through
`GET /task/effects?task_ref=…`: opaque original-key digest, stage,
outcome/reconcile class, mutation count 0/1 or absent when indeterminate, and
report refs. Receipts, raw parameters, and extra query fields fail closed.

Management `GET/POST /management/resource/v1/tool*` projects registered native
Tools with an overlay lifecycle (`enabled` / `disabled` / `quarantined` /
`revoked`), `execution_readiness`, and `agent_exposed`. Overlay state never
enters the immutable descriptor digest. Task-channel callers cannot mutate
lifecycle. `GET /task/resource/v1/tool/exposure` returns the least Agent
exposure set and digest; `POST /task/resource/v1/tool/selection` records a
receipt only when `candidate_set_digest` matches that digest and the selected
operation is exposed. Prompt/body/receipt restatement fails closed.

Management `GET/POST /management/resource/v1/http-origin` pins exact HTTPS
origins (`host` or `host:port`) for one `task_ref` under an authorized
campaign (`P2-T25` or `PERSONAL-PERF-EVAL-*`). The default allowlist is empty,
so production HttpFetchReadOnly staging fails closed until a pin exists. Pins
admit GET/HEAD only: no credentials, redirects, inherited proxy, or request
body. Ordinary task callers are denied
(`RESOURCE_PINNED_HTTPS_CHANNEL_FORBIDDEN`). Disabling
`native.registered-check.run` drops it from Agent exposure without inventing a
ProcessRun family.

The task channel reads bounded O2/O3/O4/O5/O13 observation through
`GET /task/observation?family=o2|o3|o4|o5|o13&task_ref=…` (alias
`GET /task/resource/v1/observation`). Empty collectors return `observed_zero`
with a named negative control rather than a silent default-zero. Prompt, body,
receipt, and capability query keys fail closed. O5 reuses the redacted
Intent/Effect history already served by `GET /task/effects` and still omits
raw parameters and receipts. O13 exports a durable audit cursor, event digest
chain, and bounded replay; a stale cursor, missing event, digest break, or
sequence gap fails closed. Management callers are denied
(`RESOURCE_OBSERVATION_CHANNEL_FORBIDDEN`): this is a read plane, not a second
authority API. Samples never include Context bodies or capability material.

Management `POST /management/resource/v1/backup` writes a secret-excluding
directory archive; `POST .../backup/preflight` verifies one `archive_id`
without mutation; `POST .../restore` overlays live files after snapshot and
rolls back on failure. Archives never copy authority SQLite, bootstrap
secrets, bearer files, or `provider-config.json`. Task-channel aliases are
403 `RESOURCE_BACKUP_CHANNEL_FORBIDDEN`.

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
forwards via the bounded Rustls transport. Unary proxy success responses carry
`X-CognitiveOS-Provider-Network-Nanos`. Public management `stream:true` is
forwarded as HTTP/1.1 SSE without waiting for the last event; streaming
success omits that network-nanos header because SSE headers flush first. Nested preflight timing and the
correlation echo are denied unless `COGNITIVEOS_PI_ROUTE_OBSERVATION=enabled`
and the request carries one well-formed opaque correlation id; malformed or
duplicate ids are ignored, the product body is unchanged, and the observer
writes nothing. The private one-shot Unix socket
(`POST /chat/completions`) serves only the daemon-launched Pi candidate process
and forbids Authorization headers. That private-candidate path strips
`tools`/`tool_choice` before forward, accepts one text choice that may include
`role=assistant` plus extra choice fields such as `finish_reason`, and refuses
`tool_calls` / `function_call`.

## Non-Personal skeleton

`kernel-server --once/--serve` is an M0-era AKP/shell HTTP skeleton (placeholder
semantics, errors as HTTP 200). It is not the Personal surface; treat it as
historical scaffolding used by SDK live tests.
