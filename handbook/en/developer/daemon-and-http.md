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
tests:
  - apps/kernel-server/tests/p1_t04_personal_daemon.rs
  - apps/kernel-server/tests/p1_t05_personal_readiness.rs
  - apps/kernel-server/tests/p1_t07_provider_proxy.rs
fingerprint: "sha256:b374a22c7174b283175001f4c20e61a10c41284bf90d0890b14c85f75695c87b"
non_claims:
  - Route inventory lives in the generated HTTP reference; this page explains composition, not completeness.
---

# Daemon and HTTP

## Startup order (load-bearing)

`serve_personal_loopback`: lexical loopback check → XDG layout → database
preparation/migrations → `daemon.lock` acquisition → one `SqliteAuthorityStore`
open (+ a separate `SchedulerRepository` connection to the same file) → recovery of
consumed worker handoffs → native Tool descriptor/router composition → one
bounded ArtifactStore at `data_dir()/artifacts` → bootstrap secret load/create →
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
  expiries, no per-action scopes.
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
secret reference with the older provider/model/digest facts. Doctor adds redacted
six-resource/vault/operability sections. The Provider
proxy validates config + selected model, resolves the secret in memory, and
forwards via the bounded Rustls transport; the private one-shot Unix socket
(`POST /chat/completions`) serves only the daemon-launched Pi candidate process
and forbids Authorization headers.

## Non-Personal skeleton

`kernel-server --once/--serve` is an M0-era AKP/shell HTTP skeleton (placeholder
semantics, errors as HTTP 200). It is not the Personal surface; treat it as
historical scaffolding used by SDK live tests.
