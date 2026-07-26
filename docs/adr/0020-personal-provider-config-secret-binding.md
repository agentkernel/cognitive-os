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
in configuration, SQLite, argv, logs, or evidence. ADR-0018 records one
default-deny P0-T06 local-development exception for initial Pi-child
environment delivery after native-store resolution; it is not a general
Provider environment-key policy.

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
7. A P0-T06 caller using the ADR-0018 exception must load this repository from
   an explicit non-secret Personal Provider config directory, verify
   `provider_id == "deepseek"`, and resolve material only after its non-secret
   Pi admission checks pass. The repository continues to persist only the
   opaque `SecretRef`; it never records that the exception was used.

## Consequences

- P1-T03 can probe models using `ProviderKeyService::resolve_provider_material`
  without inventing a second secret API.
- CI exercises Provider binding and redaction negatives via the simulated
  backend. Native `secret-tool` is exercised only when a Linux session bus and
  tool are present; absence remains fail-closed.
- Headless Linux without a user session bus remains unsupported for native
  secret storage until a future decision.
- This ADR does not claim G0, B01-B12, Profile conformance, or real Provider
  key storage in CI. The P0-T06 exception remains local-only, unavailable on
  Windows, and expires at the P2 boundary.

## Rejected Alternatives

1. **Storing API keys in provider.json (plaintext or encrypted)** — violates
   Personal secret invariants and backup/export safety.
2. **Using the ephemeral test double as a production fallback** — silently
   weakens native-store guarantees (ADR-0018).
3. **Making Pi or CLI write secrets directly to disk** — clients are
   non-authority and must not own secret persistence.
4. **Embedding credentials in base_url** — rejected at validation time.
5. **Reusing Pi's own config directory as Personal Provider configuration** —
   rejected because `PI_CODING_AGENT_DIR` and the Personal config repository
   have different ownership and security meanings.
