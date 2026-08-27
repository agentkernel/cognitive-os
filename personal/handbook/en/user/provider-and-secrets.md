---
doc_id: user.provider-and-secrets
locale: en
kind: guide
audience: [user]
status: implemented
generated: false
sources:
  - path: personal/crates/cognitive-secret/src/store.rs
    symbols: ["SecretStore", "SecretRef"]
  - path: personal/crates/cognitive-secret/src/backend_select.rs
  - path: personal/crates/cognitive-secret/src/provider_service.rs
    symbols: ["ProviderKeyService"]
  - path: personal/crates/cognitive-secret/src/provider_transport.rs
    symbols: ["ProviderHttpRequest"]
  - path: personal/apps/kernel-server/src/personal/provider_proxy.rs
  - path: personal/crates/cognitive-secret/src/endpoint_trust.rs
    symbols: ["TrustedEndpoint", "ProviderKind"]
  - path: personal/apps/kernel-server/src/personal/provider_control_plane.rs
  - path: docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md
  - path: personal/docs/product/account-hub.md
  - path: personal/docs/product/account-hub.zh-CN.md
  - path: personal/docs/product/mcp-resource-family.md
  - path: personal/docs/product/mcp-resource-family.zh-CN.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
tests:
  - personal/crates/cognitive-secret/tests/p1_t02_provider_secret.rs
  - personal/crates/cognitive-secret/tests/p1_t03_provider_discovery.rs
  - personal/apps/kernel-server/tests/p1_t07_provider_proxy.rs
  - personal/apps/kernel-server/tests/p9_t07_route_observation.rs
  - personal/apps/kernel-server/tests/p8_t13_provider_control_plane.rs
  - personal/crates/cognitive-secret/tests/p8_t13_endpoint_trust.rs
fingerprint: "sha256:c1144234892f49dea6e3209262a6b13ad2def8768c603417e5f9e32d1ac351a6"
non_claims:
  - Best-effort in-memory zeroization is not a side-channel or mlock guarantee. Headless encrypted-vault operation is a design target. The Windows backend does not imply a supported Windows install route (B01-W has not been executed).
  - Account Hub credential import is an adopted Personal 2.0 target; no concrete browser-profile, Agent credential-file, subscription, or OAuth import mechanism is implemented.
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
key on argv. Full operator steps (accounts, trust flags, bindings, usage,
observe-only budgets) are in
[Provider Control Plane](provider-control-plane.md). The localhost Web UI is a
daemon client for those same management routes; there is no desktop panel.

## Account Hub credential-import target (`Requires-backend`)

ADR-0055 permits a future user-directed import path, but authorizes no concrete
import mechanism:

1. The user initiates each import, sees the **exact source** and target approved
   SecretStore before any read, and consents separately for that source. There
   is no background, speculative, or bulk credential scan.
2. The Rust daemon alone reads the named source and writes the target
   SecretStore. Material exists only in daemon process memory between those
   operations. The UI and every Agent/sidecar receive no raw material.
3. Raw material never enters argv, environment variables, ordinary
   configuration written by CognitiveOS, SQLite, UI output, Agent Context,
   logs, CI/test output, evidence, support data, or chat. Audit records contain
   only redacted source kind, target store, time, and outcome.
4. Keeping the source is the default. Secure source deletion occurs only when
   the user explicitly chooses it for that import.

Browser-cookie/profile decryption, third-party Agent credential-file parsing,
subscription-token import, and OAuth capture are all `Requires-backend`.
Current `cognitive init`, `--api-key-file`, and the existing `/ui/` manual key
handoff remain separate current API behavior; none proves that Account Hub
source import exists.

The adopted MCP seventh-family target uses the same isolation: connection
credentials stay in an approved SecretStore and raw material never reaches the
Control Plane, Agent, sidecar, package metadata, ordinary configuration,
SQLite, Context, logs, evidence, or chat. The family backend remains
`Requires-backend`/`Requires-core`.

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
- Named control-plane accounts store an OpenAI-compatible API root, not a chat
  RPC path. Pasting `…/v1/chat/completions` is stripped to `…/v1`; other paths
  fail closed as `PROVIDER_ENDPOINT_PATH_FORBIDDEN`.
