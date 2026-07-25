# ADR-0019: Personal Daemon Transport, Local Auth Bootstrap, and Threat Model

- Status: Accepted for P0-T07 design freeze
- Date: 2026-07-25
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: Personal product local-daemon boundary decision. This ADR
  freezes transport, authentication bootstrap, resource bounds, and the threat
  model for CognitiveOS Personal. It is not a CognitiveOS specification
  requirement, registry REQ, schema, transition, vector, Profile claim, Gate
  claim, or business-route implementation.

## Context

Personal architecture requires an authenticated local path from Pi/CLI clients
to a single Rust daemon that is the only authority writer. ADR-0003 already
selects HTTP JSON + SSE as the single-node external API for the reference
implementation. The current `apps/kernel-server` surface is a hand-written,
unbounded HTTP skeleton that accepts synthetic routes, refuses non-loopback
binds in `--serve` mode, and does not yet implement Personal session bootstrap
or resource limits.

P0-T07 must freeze how Personal listens, authenticates local clients, separates
task and management channels, and bounds request cost **before** P1-T04
implements a bounded daemon. This ADR deliberately does not implement business
routes, Provider APIs, Task admission, or Secret Store backends.

Related decisions already accepted:

- ADR-0003: HTTP JSON + SSE envelope transport for single-node external API.
- ADR-0017: Personal SQLite migration/backup/recovery boundary; XDG path roles
  are design-only until P1-T01.
- ADR-0018: SecretStore port and fail-closed secret handling; secrets never ride
  transport logs, tokens, or evidence.

## Decision

### 1. Transport selection

1. **Default Personal listen path (Linux first release target):** HTTP/1.1 over
   a Unix domain socket (UDS) at a daemon-private runtime path derived from the
   XDG runtime layout frozen by P1-T01, for example
   `$XDG_RUNTIME_DIR/cognitiveos/daemon.sock`.
2. **Socket and directory permissions:** parent directory `0700`, socket file
   `0600`, owned by the same local user as the daemon. The daemon must refuse to
   start if the runtime directory is group/world writable or the socket path is
   a symlink outside the expected private tree.
3. **Envelope and watch:** UDS carries the same ADR-0003 mapping: JSON request
   envelopes on POST and SSE for watch streams. Envelope semantics remain
   authority; HTTP status is transport-only.
4. **Optional loopback TCP:** `127.0.0.1` or `[::1]` only, for automated tests
   and temporary local tooling. Default product start uses UDS. Binding
   `0.0.0.0`, LAN interfaces, public interfaces, or non-loopback addresses is
   forbidden for Personal v1.
5. **Disabled by default:** no listen socket is opened until an explicit local
   start path (`cognitive daemon start` / equivalent P1 lifecycle) after
   successful init and readiness preflight. Install alone must not expose a
   listener.
6. **Rejected for Personal v1 default:** remote TLS termination as the primary
   product surface, unauthenticated LAN HTTP, WebSocket as the primary watch
   transport, bare non-HTTP framing over UDS, and any client-side authority
   writer.

### 2. Channel bootstrap and authentication

1. **Two disjoint channels** remain mandatory:
   - Task channel root: `/task/` (and existing shell-facing roots that map to
     task-channel credentials).
   - Management channel root: `/management/`.
2. Credentials, caches, storage keys, and session material for the two channels
   must never be interchangeable. Cross-presentation fails closed with the
   registered code `SHELL_CHANNEL_BINDING_MISMATCH` (REQ-SHELL / REQ-AKP
   channel isolation; vector `shell-channel-isolation-003.json`).
3. **Local session issuance (design freeze for P1-T04):**
   - After local process identity is established (same-user peer on UDS, or
     loopback-only TCP plus a one-time bootstrap secret readable only from the
     private runtime tree), the daemon issues **channel-scoped** session tokens.
   - Each token binds at least: channel class (`task` | `management`), local
     principal id, issue logical version/epoch, idle timeout, absolute expiry,
     and a non-reusable session id.
   - Management sessions additionally remain subject to privileged management
     session scope/risk/revocation rules already registered for management
     verbs; a transport token alone never authorizes R2/R3 actions.
4. **Presentation:** `Authorization: Bearer <token>` (or equivalent single
   header). Cookies are forbidden for Personal daemon auth so browser CSRF
   cookie rules do not apply. Tokens must not appear in URLs, query strings,
   argv, environment variables, logs, or evidence digests.
5. **Bootstrap secret location:** only under the private runtime directory
   (mode `0600`). Not in SQLite authority tables, Provider config, Pi
   `auth.json`, or Secret Service (Secret Service is for Provider API keys per
   ADR-0018, not for local channel tokens).
6. **Expiry and revoke:** idle and absolute expiry both apply; restart or
   explicit revoke invalidates outstanding tokens. Renewal requires
   re-authentication against the local bootstrap path and produces a new
   session version or session id.
7. Clients (Pi Extension, CLI, future local UI) are non-authority: they only
   send requests and render projections. They must not write SQLite, advance
   Task/Effect/Verification, or mint capabilities.

### 3. Resource bounds (contract for P1-T04)

P1-T04 must enforce these ceilings; values may be tightened later but not
removed:

| Bound | Personal v1 baseline |
|---|---|
| Max request body | 1 MiB default; hard ceiling <= 8 MiB |
| Max header block | 16 KiB; max 64 headers |
| Read/header timeout | 10 s default |
| Request body read timeout | 30 s default |
| Idle connection timeout | 60 s default |
| Max concurrent connections | 32 total; <= 16 per channel |
| Max concurrent in-flight requests | 16 total |
| Session absolute lifetime | <= 12 h |
| Session idle lifetime | <= 30 min |
| Watch SSE keep-alive | required; stale cursor forces snapshot |

Oversized, slow, or excess concurrency requests fail closed with a stable
registered or protocol error and create no authority side effects.

### 4. Threat model (must-cover)

The following threats are in scope for Personal local-daemon exposure. Each
row states the required control. Implementation evidence is owned by P1-T04
and later security tests; this ADR freezes the required control surface only.

| Threat | Attack surface | Required control |
|---|---|---|
| **CSRF** | Browser on the same host posts to loopback/UDS HTTP | No cookie auth; require explicit `Authorization` bearer; reject requests that carry browser `Origin`/`Referer` from non-local product origins unless a future local UI ADR defines a stricter allowlist; prefer UDS so ordinary web origins cannot open the socket |
| **DNS rebinding** | Page on attacker DNS resolves to `127.0.0.1` and calls TCP loopback | Default to UDS; for TCP loopback require `Host` in `{127.0.0.1, localhost, [::1]}` (plus optional explicit port); reject absolute external hosts; do not enable CORS `*` |
| **Token theft** | World-readable socket, leaked logs, shared env, backup of runtime dir | Socket/dir modes `0600`/`0700`; tokens only in Authorization header; redacted logs; runtime dir not copied into support bundles without redaction; short idle/absolute expiry; revoke on daemon restart |
| **Channel confusion** | Task token used on management route or vice versa | Disjoint roots + channel-bound tokens; fail with `SHELL_CHANNEL_BINDING_MISMATCH`; never share caches/credential stores across channels |
| **Replay** | Captured HTTP request resent | Absolute/idle session expiry; effecting operations require envelope idempotency keys already required by AKP; management privileged actions re-check session version/revocation epoch; no token in URL that can be cached by intermediaries |
| **Confused deputy / local malware same-user** | Another process of the same user | Same-user is inside the trust boundary of Personal v1; residual risk accepted and documented. Cross-user isolation relies on OS user separation + socket modes. Compromised-user malware is out of scope for v1 claims |
| **Remote exposure** | Accidental non-loopback bind or port forward | Refuse non-loopback TCP binds; disabled-by-default listener; no product docs that recommend LAN bind |
| **Secret material on wire logs** | Provider keys or tokens in debug traces | ADR-0018 redaction; transport layer never logs Authorization values or secret refs material |

Out of scope for Personal v1 claims (explicit non-claim): multi-tenant remote
SaaS hardening, browser-based public Console on the open internet, compromised
root/kernel adversaries, and formal Profile conformance.

### 5. Relationship to existing reference server

- `apps/kernel-server` remains the composition-root experiment for M5/M6 HTTP
  skeletons. Personal P1-T04 may extend it or introduce a Personal-facing
  binary/entrypoint, but must preserve: loopback/UDS-only listen policy,
  channel separation, and the bounds above.
- Synthetic routes in the current skeleton are not a product API surface and
  must not be treated as completed Personal transport work.
- No registry, schema, transition, or vector change is authorized by this ADR.
  If a future bound requires a new machine error code, that change goes through
  Lane-CTR as a corrective registration, not through this design freeze.

## Rejected alternatives

1. **LAN or public HTTP as Personal default** - expands remote attack surface
   without product requirement.
2. **Cookie/session browser auth for the daemon** - invites CSRF on loopback.
3. **Single shared bearer for task and management** - violates channel isolation
   and `SHELL_CHANNEL_BINDING_MISMATCH` discipline.
4. **Storing local session tokens in SQLite authority tables or Secret Service**
   - confuses authority durability with ephemeral local auth; secrets store is
   reserved for Provider material (ADR-0018).
5. **Unbounded body/header/concurrency until we need limits** - current
   skeleton already demonstrates the hazard; bounds are a P0 decision.
6. **Treating ADR-0002 or ADR-0017 as the transport decision** - those ADRs are
   store/migration decisions; transport remains ADR-0003 + this Personal
   binding.

## Consequences

- P1-T04 can implement the bounded daemon and local auth tests against a frozen
  listen/auth/threat surface without reopening transport choice.
- P1-T01 must place the runtime socket under the private XDG runtime tree and
  document backup/support redaction so tokens are not shipped in bundles.
- G0 still requires remaining Phase 0 items (notably P0-T03 owner decision and
  P0-T06 Pi PoC). Completing P0-T07 alone does **not** claim G0, B01-B12, or
  Profile `implemented`.
- Owner license/platform/distribution decisions (P0-T03) remain independent;
  this ADR does not authorize redistribution packaging or public release.

## Compliance / acceptance for P0-T07

P0-T07 is design-complete when:

1. This ADR is merged and referenced from the formal Personal ledger.
2. Threat rows for CSRF, DNS rebinding, token theft, channel confusion, and
   replay are present with required controls (table above).
3. No business routes, Provider integration, or live daemon auth implementation
   are claimed as done by this task.
4. Follow-on implementation work is explicitly owned by P1-T04 (and path layout
   by P1-T01).
