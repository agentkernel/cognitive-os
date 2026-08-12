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
fingerprint: "sha256:8c8e8688bc7913198ecace6ced13319d83d21e1d069945de81493b6b5f45646f"
non_claims:
  - Route inventory lives in the generated HTTP reference; this page explains composition, not completeness.
---

# Daemon and HTTP

## Startup order (load-bearing)

`serve_personal_loopback`: lexical loopback check → XDG layout → database
preparation/migrations → `daemon.lock` acquisition → one `SqliteAuthorityStore`
open (+ a separate `SchedulerRepository` connection to the same file) → recovery of
consumed worker handoffs → **one** private scheduler tick → bootstrap secret
load/create → TCP bind → atomic `daemon-endpoint.json` publication →
thread-per-connection serving. Recovery and the single tick run before the
endpoint appears; a scheduler error prevents listening entirely. There is no
shutdown route and no continuing scheduler thread (see
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
degraded | ready` + `first_conversation_ready`); it does not probe the Provider
live. Doctor adds redacted six-resource/vault/operability sections. The Provider
proxy validates config + selected model, resolves the secret in memory, and
forwards via the bounded Rustls transport; the private one-shot Unix socket
(`POST /chat/completions`) serves only the daemon-launched Pi candidate process
and forbids Authorization headers.

## Non-Personal skeleton

`kernel-server --once/--serve` is an M0-era AKP/shell HTTP skeleton (placeholder
semantics, errors as HTTP 200). It is not the Personal surface; treat it as
historical scaffolding used by SDK live tests.
