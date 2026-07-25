# ADR-0018: Personal SecretStore Boundary and Fail-Closed Backends

- Status: Accepted for P0-T05 PoC
- Date: 2026-07-25
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: Personal product secret-handling decision. This ADR freezes a
  reference-implementation port and PoC backends; it is not a CognitiveOS
  specification requirement, registry REQ, schema, transition, vector, or
  Profile claim.

## Context

Provider API keys for CognitiveOS Personal must never enter configuration,
SQLite, Pi, environment variables, command lines, logs, or evidence. The
Personal plan requires a native Linux Secret Service path, with fail-closed
behavior when the service is absent, locked, or would need an interactive
prompt that a non-interactive daemon cannot complete.

P0-T05 must freeze the daemon-facing API before P1-T02 builds Provider
configuration on top of it. A previous environment-only `secret-tool` session
does not freeze a Rust port or provide automated leak negatives in CI.

## Decision

1. Introduce an isolated workspace crate `cognitive-secret` that owns the
   Personal SecretStore surface and PoC backends. It must not depend on
   kernel, store, runtime, management, or authority writers.
2. Freeze the daemon-facing trait methods as
   `SecretStore::{probe, put, get, delete}` plus an opaque `SecretRef`.
   Attribute-keyed `put` replaces existing material and is the rotate path.
3. `SecretMaterial` never implements revealing `Debug`/`Display`. Errors never
   embed secret bytes. There is no plaintext fallback method on the trait.
4. Production Personal must use a native backend class only. The process-local
   `SimulatedSecretServiceStore` / `EphemeralSecretStore` is an automated PoC
   and test double (`SecretStoreClass::EphemeralTestDouble`) and must not be
   selected as a product backend.
5. When no usable native backend is available, Personal init must fail closed
   via `UnavailableSecretStore` or equivalent probe rejection. Locked and
   prompt-unavailable modes also fail closed for daemon use.
6. `LinuxSecretServiceProbe` classifies native readiness from session-bus
   signals on Linux and reports unavailable on non-Linux hosts. Mutating FreeDesktop Secret Service I/O is delivered by P1-T02
   (`LinuxSecretToolStore` / ADR-0020); P0-T05 freezes the port and fail-closed semantics only.
7. Secrets must not be written to SQLite, config files, env, argv, logs, test
   snapshots, or evidence digests. Config may store only opaque `SecretRef`
   identifiers after P1-T02.

## Consequences

- P1-T02 can implement the real native Secret Service adapter and Provider
  configuration against a stable port without inventing a second secret API.
- CI can execute put/get/rotate/delete and leak-negative tests via the
  simulated backend without requiring gnome-keyring on every runner.
- Headless Linux without a user session bus remains unsupported until a future
  decision; first-release desktop user session is the intended path.
- This ADR does not claim G0, B01-B12, Profile conformance, or production
  Provider key storage.

## Rejected Alternatives

1. **Plaintext or encrypted SQLite secret table** — violates Personal
   invariants and makes backup/export leak material.
2. **Environment-variable or config-file Provider keys** — forbidden by plan
   and redaction rules.
3. **Using the ephemeral test double as a product fallback** — would silently
   weaken native-store guarantees.
4. **Making Pi or CLI write secrets directly** — clients are non-authority and
   must not own secret persistence.
