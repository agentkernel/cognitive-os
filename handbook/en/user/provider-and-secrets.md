---
doc_id: user.provider-and-secrets
locale: en
kind: guide
audience: [user]
status: implemented
generated: false
sources:
  - path: crates/cognitive-secret/src/store.rs
    symbols: ["SecretStore", "SecretRef"]
  - path: crates/cognitive-secret/src/backend_select.rs
  - path: crates/cognitive-secret/src/provider_service.rs
    symbols: ["ProviderKeyService"]
  - path: crates/cognitive-secret/src/provider_transport.rs
    symbols: ["ProviderHttpRequest"]
  - path: apps/kernel-server/src/personal/provider_proxy.rs
  - path: crates/cognitive-secret/src/endpoint_trust.rs
    symbols: ["TrustedEndpoint", "ProviderKind"]
  - path: apps/kernel-server/src/personal/provider_control_plane.rs
tests:
  - crates/cognitive-secret/tests/p1_t02_provider_secret.rs
  - crates/cognitive-secret/tests/p1_t03_provider_discovery.rs
  - apps/kernel-server/tests/p1_t07_provider_proxy.rs
  - apps/kernel-server/tests/p9_t07_route_observation.rs
  - apps/kernel-server/tests/p8_t13_provider_control_plane.rs
  - crates/cognitive-secret/tests/p8_t13_endpoint_trust.rs
fingerprint: "sha256:266356c80282656671b6d64bddd68874dc48286b076fc08ded8074c2af474b67"
non_claims:
  - Best-effort in-memory zeroization is not a side-channel or mlock guarantee. Headless encrypted-vault operation is a design target. The Windows backend does not imply a supported Windows install route (B01-W has not been executed).
---

# Provider and secrets

## Where your key lives — and where it can never appear

Your Provider API key enters through hidden input or stdin during `cognitive init`
and is stored **only** in an approved OS secret store: the Linux Secret Service
(via `secret-tool`, session D-Bus) or, on Windows hosts, the Windows Credential
Manager (via a fixed, audited PowerShell helper invoked from the absolute system
path; secret material travels only over the helper's stdin/stdout, persistence is
local-only, and blobs are capped at 2560 bytes). Configuration keeps an opaque
reference (`SecretRef`), never material. The enforced no-go zones — process
arguments, ordinary config, SQLite, logs, CI/test output, evidence, and the Pi
process environment — are covered by focused tests and source scans.

Backend selection is probe-based and fail-closed: on any other platform (macOS
today), or when the keyring/credential store is locked or unusable, every secret
operation refuses; there is deliberately no plaintext fallback. Rotation:
`cognitive init --rotate-key`. Named control-plane accounts use
`cognitive provider key set|rotate|remove --api-key-file` instead of putting a
key on argv.

## How Provider traffic flows

Clients never talk to the Provider. The daemon owns egress:

1. `POST /provider/v1/chat/completions` (management channel, Pi path) validates
   the request. When a control-plane binding exists for `agent://personal/pi`,
   the model must match that binding and there is no fallback. Unbound agents
   still use `provider.json` and `selected-model.json`. Public `stream:true` is
   forwarded as SSE; Pi conversations and private-candidate stay unary and, when
   a Pi binding exists, use that bound account rather than `provider.json`. Model
   mismatch still fails closed. DeepSeek harness uses the independent
   `POST /provider/v1/dsh/chat/completions` route (`agent://personal/dsh`).
2. The daemon resolves the `SecretRef` in memory and attaches the bearer header.
3. `RustlsProviderTransport` enforces HTTPS-only, no redirects, no URL user-info,
   header CR/LF rejection, a 1 MiB response cap, and a caller timeout. Public
   `stream:true` reads HTTP/1.1 TLS records directly so the first SSE event is
   not held until the last event. Hermetic additional roots replace the platform
   CA store for that transport instance; production loads platform roots once.
4. Unary proxy success responses carry an `X-CognitiveOS-Provider-Network-Nanos`
   header (daemon-measured Provider network time only). Streaming success omits
   that header because the total is unknown when SSE headers flush; it still
   reports `X-CognitiveOS-Daemon-Preflight-Nanos`. Clients may send one
   opaque `campaign-<32 lowercase hex>` `x-cognitiveos-correlation-id` request
   header. The daemon never persists it. When
   `COGNITIVEOS_PI_ROUTE_OBSERVATION=enabled` and that header is well-formed, the
   success response also echoes the id and reports
   `X-CognitiveOS-Daemon-Preflight-Nanos` (config/selected-model/SecretStore,
   disjoint from the network exchange). Malformed or duplicate correlation
   headers are ignored and the product body is unchanged.
5. The private Pi candidate completion uses the same daemon-owned proxy: it
   strips `tools`/`tool_choice` before forward, accepts one text choice that may
   include `role=assistant`, and refuses `tool_calls`. Adapter stderr on
   `daemon.log` is redacted (`sk-` / `api_key=` / `token=`).

Discovery (`cognitive init`) probes `GET /models` plus a chat/stream/tool/cancel
campaign and persists a non-secret capability snapshot with an identity digest; the
selected model must match that snapshot.

## Honest limits

- The readiness projection checks configuration/backend presence, not a live
  Provider round-trip — `ready` does not prove your key is currently valid.
- The `secret-tool` probe cannot distinguish an unlocked collection; a locked
  keyring surfaces as unavailable at first real use.
- Rotation clears the old item before storing the new one; a crash between the two
  requires re-entering the key.
