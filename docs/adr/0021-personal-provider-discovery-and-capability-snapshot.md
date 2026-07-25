# ADR-0021: Personal Provider Discovery and Capability Snapshot

- Status: Accepted for P1-T03
- Date: 2026-07-25
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: Personal product Provider decision. This ADR freezes model
  discovery and active capability probing for the reference implementation; it
  is not a CognitiveOS specification requirement, registry REQ, schema,
  transition, vector, or Profile claim.

## Context

P1-T02 bound Provider configuration to SecretStore so daemons can resolve an
opaque API key after restart (ADR-0020). Personal still needs to:

1. Discover currently visible models from an OpenAI-compatible `/models` API
2. Actively probe chat, stream, tool-call candidate shape, and cancel
3. Persist a non-secret capability snapshot digest into `provider.json`

Catalog membership alone is insufficient: an HTTP 200 model list does not prove
chat, stream, tool-call, or cancel behavior. Real Provider keys must never enter
config, SQLite, env, argv, logs, or evidence.

## Decision

1. Keep Provider discovery and probes inside the isolated `cognitive-secret`
   crate so Personal does not take Lane-RUN ownership of `cognitive-runtime` or
   invent parallel secret APIs.
2. Inject HTTPS I/O through a `ProviderTransport` trait. Production daemons
   supply an HTTPS client at composition time; automated tests inject a hermetic
   mock. This crate remains free of HTTP client dependencies.
3. Attach secret material only as a short-lived `Authorization: Bearer …`
   header for egress. Request/response Debug/Display redacts Authorization and
   bodies. Snapshot digests are never derived from secret bytes.
4. Active probes are:
   - `list_models` → GET `/models`
   - `probe_chat` → minimal non-streaming chat completion
   - `probe_stream` → streaming chat shape
   - `probe_tool_call` → tools array; success means candidate-shaped
     `tool_calls` only (not Effect dispatch or Task completion)
   - `probe_cancel` → exchange with `cancel_requested`; transport timeout/abort
     is the pass signal
5. Persist only the product-local snapshot identity digest
   (`fnv1a64:…`) into `ProviderConfig.selected_snapshot_digest` via
   `ProviderKeyService::persist_selected_snapshot_digest`.
6. Classify HTTP failures as unauthorized/forbidden/not_found/rate_limited/
   server_error; treat HTTP 200 without tool_calls as capability_missing; treat
   selected-model-not-in-catalog as alias_drift; allow explicit
   `ManualFallback` selection when the operator supplies a model id.

## Consequences

- P1-T05 readiness/doctor can consume capability flags without re-implementing
  Provider HTTP semantics.
- CI covers positive and negative probe paths with synthetic material and a mock
  transport. Live DeepSeek network evidence is out of scope for this atomic
  batch and is not claimed.
- Production HTTPS client wiring remains a daemon composition concern (P1-T04 /
  P1-T06). Absence of a transport injection fails closed by not probing.
- This ADR does not claim G0, B01-B12, Profile conformance, or authority writes.

## Rejected Alternatives

1. **Putting discovery in `cognitive-runtime` immediately** — would cross into
   Lane-RUN ownership for a product-only surface; deferred until daemon wiring
   needs a shared runtime adapter.
2. **Trusting `/models` without active probes** — fails the Personal product
   requirement that capabilities are verified, not assumed.
3. **Storing raw probe response bodies or API keys in config** — violates secret
   and redaction invariants.
4. **Treating tool-call probe success as Effect execution** — tool requests are
   candidates only; dispatch remains Intent/Effect authority work.
