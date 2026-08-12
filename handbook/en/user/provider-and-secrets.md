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
tests:
  - crates/cognitive-secret/tests/p1_t02_provider_secret.rs
  - crates/cognitive-secret/tests/p1_t03_provider_discovery.rs
  - apps/kernel-server/tests/p1_t07_provider_proxy.rs
fingerprint: "sha256:616fc34313bb4014816cd476a65db4a5e66de09b5a1da232c44cedd4ee8d47d3"
non_claims:
  - Best-effort in-memory zeroization is not a side-channel or mlock guarantee. Only the Linux Secret Service backend is production-selected today; headless encrypted-vault operation is a design target.
---

# Provider and secrets

## Where your key lives — and where it can never appear

Your Provider API key enters through hidden input or stdin during `cognitive init`
and is stored **only** in the Linux Secret Service (via `secret-tool`, session
D-Bus). Configuration keeps an opaque reference (`SecretRef`), never material. The
enforced no-go zones — process arguments, ordinary config, SQLite, logs, CI/test
output, evidence, and the Pi process environment — are covered by focused tests and
source scans.

On platforms without a production backend (Windows/macOS today) or with a locked or
absent keyring, every secret operation fails closed; there is deliberately no
plaintext fallback. Rotation: `cognitive init --rotate-key`.

## How Provider traffic flows

Clients never talk to the Provider. The daemon owns egress:

1. `POST /provider/v1/chat/completions` (management channel) validates the request
   against `provider.json` and `selected-model.json` — streaming and model mismatch
   are rejected.
2. The daemon resolves the `SecretRef` in memory and attaches the bearer header.
3. `RustlsProviderTransport` enforces HTTPS-only, no redirects, no URL user-info,
   header CR/LF rejection, a 1 MiB response cap, and a caller timeout.

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
