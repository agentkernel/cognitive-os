# ADR-0022: Personal Bounded Daemon and Local Auth (P1-T04)

- Status: Accepted for P1-T04 implementation
- Date: 2026-07-25
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: Personal product local-daemon implementation decision.
  Implements the transport/auth/bounds surface frozen by ADR-0019. Not a
  CognitiveOS specification requirement, registry REQ, schema, transition,
  vector, Profile claim, or Gate claim.

## Context

ADR-0019 froze Personal listen policy, channel-scoped bearer bootstrap,
resource bounds, and the threat model. P1-T01 placed runtime paths under the
XDG layout. P1-T04 must replace the unauthenticated synthetic composition root
for Personal operation with a fail-closed loopback front door.

## Decision

1. **Entry flag:** `kernel-server --personal` enables the Personal surface.
   Existing `--once` / `--serve` M5 synthetic routes remain available without
   `--personal` so M5 HTTP/SSE evidence stays valid.
2. **Listen policy:** loopback TCP only (`127.0.0.1` / `[::1]`). Non-loopback
   binds are refused. UDS remains the product default design (ADR-0019); this
   batch implements the loopback test/tooling path required for CI on Linux and
   Windows/MSVC.
3. **Single-instance lock:** `runtime/cognitiveos/daemon.lock` via exclusive
   create-new; Drop removes the lock on clean shutdown.
4. **Bootstrap secret:** written only under
   `runtime/cognitiveos/local-bootstrap.secret` (mode `0600` on Unix). Not in
   SQLite, Secret Service, env, argv, or logs.
5. **Session issue:** `POST /local/session` with bootstrap secret mints a
   channel-scoped bearer (`task` | `management`). Cookies are forbidden.
6. **Channel routes:** `/management/*` and `/task/*` require matching bearer;
   cross-channel use returns `SHELL_CHANNEL_BINDING_MISMATCH`.
7. **Bounds:** ADR-0019 table enforced for body size, header block size/count,
   concurrent connections, in-flight requests, header read timeout, and body
   read timeout. Slow partial headers/bodies fail closed with
   `PERSONAL_REQUEST_READ_TIMEOUT` (HTTP 408). Excess concurrency fails closed
   with `CONNECTION_LIMIT_EXCEEDED` or `IN_FLIGHT_LIMIT_EXCEEDED` (HTTP 429)
   and creates no authority side effects. Non-`once` accepts run on worker
   threads so concurrent limit probes can exercise shared counters.
8. **Host / CSRF controls:** reject non-loopback `Host`; reject `Cookie` auth.
9. **Non-claims:** no Task scheduler, Memory, MCP, full readiness projection
   (P1-T05), Provider proxy, G0, B01-B12, or Profile `implemented`.

## Consequences

- P1-T05 can attach readiness projections behind the authenticated front door.
- P1-T06 CLI can call bootstrap + status without inventing a second auth path.
- CI Ubuntu/Windows-MSVC is the execution authority for process-level tests on
  hosts where the local GNU linker is non-supported (P0-T01).
