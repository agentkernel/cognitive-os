# ADR-0020: Personal Provider Config Binding to SecretStore

- Status: Accepted for P1-T02
- Date: 2026-07-25
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: Personal product secret/config decision. This ADR freezes
  Provider configuration and SecretStore binding for the reference
  implementation; it is not a CognitiveOS specification requirement, registry
  REQ, schema, transition, vector, or Profile claim.

## Context

P0-T05 froze `SecretStore::{probe, put, get, delete}` and opaque `SecretRef`
with fail-closed backends (ADR-0018). P1-T02 must let a Personal daemon manage
a DeepSeek (or OpenAI-compatible) Provider API key without placing secret bytes
in configuration, SQLite, environment variables, argv, logs, or evidence.

Provider configuration must survive daemon restart by reloading a non-secret
document that references only an opaque `SecretRef`. Production selection must
never fall back to the ephemeral test double.

## Decision

1. Persist Provider configuration as a fixed JSON document under the Personal
   XDG config directory (`provider.json`) containing only:
   - `schema_version`
   - `provider_id`
   - `base_url` (absolute `https://` only; no embedded credentials)
   - `secret_ref` (opaque handle)
   - optional `selected_snapshot_digest` (for later P1-T03 probe snapshots)
2. Implement `ProviderKeyService` in `cognitive-secret` to bind
   `SecretStore` put/rotate/delete/get with the config repository. Config is
   written only after a successful secret put. Secret material is never written
   to the config file.
3. Production backend selection (`select_production_secret_store`) never returns
   `SecretStoreClass::EphemeralTestDouble`. On Linux it prefers
   `LinuxSecretToolStore` when probe reports Available; otherwise it selects
   `UnavailableSecretStore` (fail closed).
4. `LinuxSecretToolStore` is the native mutating adapter. It drives FreeDesktop
   Secret Service via `secret-tool`, passing secret material only on stdin.
   Attributes identify items; `SecretRef` encodes those non-secret attributes.
5. Hidden input for CLI is provided as `read_secret_material_from_reader`.
   Terminal echo-off product wiring remains P1-T06; this helper must not log or
   export secret bytes.
6. This crate remains isolated: no kernel/store/runtime dependency and no
   authority writes.

## Consequences

- P1-T03 can probe models using `ProviderKeyService::resolve_provider_material`
  without inventing a second secret API.
- CI exercises Provider binding and redaction negatives via the simulated
  backend. Native `secret-tool` is exercised only when a Linux session bus and
  tool are present; absence remains fail-closed.
- Headless Linux without a user session bus remains unsupported for native
  secret storage until a future decision.
- This ADR does not claim G0, B01-B12, Profile conformance, or real Provider
  key storage in CI.

## Rejected Alternatives

1. **Storing API keys in provider.json (plaintext or encrypted)** — violates
   Personal secret invariants and backup/export safety.
2. **Using the ephemeral test double as a production fallback** — silently
   weakens native-store guarantees (ADR-0018).
3. **Making Pi or CLI write secrets directly to disk** — clients are
   non-authority and must not own secret persistence.
4. **Embedding credentials in base_url** — rejected at validation time.