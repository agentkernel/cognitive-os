# ADR-0003: HTTP JSON + SSE as the Single-Node External API

- Status: Accepted for the reference implementation baseline
- Date: 2026-07-20
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: reference implementation decision. This ADR binds this
  repository's implementation only; it is NOT a CognitiveOS specification
  requirement. AKP defines envelope semantics; transports are profiles.

## Context

The single-node deployment (M5) needs an external API for the task channel
(Shell/SDK), the management channel (admin-cli/Console), and watch streams
(snapshot + cursor, at-least-once). AKP envelope semantics — version
negotiation, schema digest pinning, idempotency keys, continuation cursors,
fail-closed critical extensions — are transport-independent
(`docs/standards/akp-envelope-and-http-profile.md`). Clients span Rust, TS,
browsers (future Console) and curl-level debugging.

## Decision

The single-node external API is HTTP/1.1+ with JSON envelope bodies for
request/response operations and Server-Sent Events for watch streams.

Binding rules:

1. Request/response AKP operations map to `POST` with a canonical-JSON-
   compatible envelope body; responses carry the enveloped AKP result.
2. Watch subscriptions map to SSE; the SSE resume position carries the AKP
   continuation cursor; `WATCH_CURSOR_STALE` forces a fresh snapshot.
3. HTTP status codes are transport signals only. The authoritative outcome
   is the enveloped result and registered error code; a 2xx never implies
   effect success ([REQ-GW-002] analog).
4. Task and management channels use disjoint endpoint roots and credentials
   (`SHELL_CHANNEL_BINDING_MISMATCH` on cross-use).
5. TLS termination and authentication middleware are deployment concerns of
   `apps/kernel-server`; the envelope never trusts transport identity alone.

## Alternatives considered

### gRPC / protobuf

Rejected for v0.1: canonical JSON is already the digest substrate
(ADR-0004); a second serialization would need its own canonicalization and
fixture story. Browser and curl ergonomics also favor HTTP JSON.

### WebSocket for watch

Rejected for the baseline: SSE gives ordered server-push with trivial
resume semantics matching the snapshot+cursor model, works through plain
HTTP infrastructure, and avoids bidirectional framing the protocol does not
need (client → server traffic is ordinary requests).

### Long polling

Rejected: resume and ordering semantics become bespoke; SSE standardizes
exactly this shape.

## Consequences

Streaming is server-to-client only; interactive bidirectional features must
be modeled as requests + watch, which matches the architecture's
proposal/approval shape anyway. SSE requires keep-alive and cursor-stale
handling in every client (covered by `packages/sdk-ts`). A future
distributed profile (M9) may add other transports without touching envelope
semantics.

## Compliance checks

M5 exit: envelope negotiation, digest-mismatch rejection, channel-binding
negative, watch resume with stale cursor, and cancel-pending closure all
executed over real HTTP+SSE against `apps/kernel-server`, with evidence
under `artifacts/evidence/`.
