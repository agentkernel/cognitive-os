# ADR-0018: Personal SecretStore Boundary and Fail-Closed Backends

- Status: Accepted with a local development exception for P0-T06
- Date: 2026-07-25
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: Personal product secret-handling decision. This ADR freezes a
  reference-implementation port and PoC backends; it is not a CognitiveOS
  specification requirement, registry REQ, schema, transition, vector, or
  Profile claim.

## Context

Provider API keys for CognitiveOS Personal must never enter configuration,
SQLite, command lines, logs, or evidence. The Personal plan requires a native
Linux Secret Service path, with fail-closed behavior when the service is
absent, locked, or would need an interactive prompt that a non-interactive
daemon cannot complete.

On 2026-07-26, the decision owners approved a narrow P0-T06 local-development
exception: an already-configured native Secret Service key may be injected
into the initial Pi child environment only after an explicit CLI switch. This
exception is not a production design, containment claim, or authority grant;
it expires at the end of P2 and must be replaced or re-approved before then.

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
7. Secrets must not be written to SQLite, config files, argv, logs, test
   snapshots, or evidence digests. Config may store only opaque `SecretRef`
   identifiers after P1-T02.
8. The P0-T06 local-development exception is default-deny and is permitted
   only when all conditions hold:
   - the caller supplies the exact explicit development switch;
   - the host has an available Linux native Secret Service backend;
   - the configured Provider is `deepseek` and material resolves through
     `ProviderKeyService`, never through a parent-process environment variable,
     command line, file, or prompt;
   - the material is supplied only to the initial Pi child-process environment;
     Pi is uncontained and may pass its environment to descendants, so this is
     not a containment guarantee;
   - no Windows, CI, release, Gate, Profile, or production claim is made.
   The exception expires at the P2 exit. It must be removed, replaced by a
   local provider-auth proxy, or explicitly re-approved before P2 closes.

## Consequences

- P1-T02 can implement the real native Secret Service adapter and Provider
  configuration against a stable port without inventing a second secret API.
- CI can execute put/get/rotate/delete and leak-negative tests via the
  simulated backend without requiring gnome-keyring on every runner.
- Headless Linux without a user session bus remains unsupported until a future
  decision; first-release desktop user session is the intended path.
- This ADR does not claim G0, B01-B12, Profile conformance, production
  Provider key storage, Pi containment, or an approved production
  credential-delivery design.

## Rejected Alternatives

1. **Plaintext or encrypted SQLite secret table** — violates Personal
   invariants and makes backup/export leak material.
2. **Ambient environment-variable or config-file Provider keys** — forbidden:
   the P0-T06 exception permits only a fresh, explicit, initial-child injection
   after native Secret Service resolution and is not a general environment-key
   policy.
3. **Using the ephemeral test double as a product fallback** — would silently
   weaken native-store guarantees.
4. **Making Pi or CLI write secrets directly** — clients are non-authority and
   must not own secret persistence.
