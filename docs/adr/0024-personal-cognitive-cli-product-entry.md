# ADR-0024: Personal `cognitive` CLI Product Entry (P1-T06)

- Status: Accepted for P1-T06 implementation
- Date: 2026-07-25
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: Personal product CLI decision. Not a CognitiveOS
  specification requirement, registry REQ, schema, transition, vector,
  Profile claim, G0 claim, or B01-B12 claim.

## Context

P1-T01..T05 provide XDG layout, SecretStore/Provider binding, capability
snapshots, a bounded authenticated Personal daemon, and readiness/status/doctor
projections. Operators still need a product-facing CLI that:

1. initializes layout and dual databases without becoming an authority writer;
2. binds Provider keys through SecretStore with hidden or file-based capture;
3. starts/stops the Personal daemon process;
4. consumes the same status/doctor projections as future Pi/UI clients.

`admin-cli` remains the deterministic emergency management path and must not be
removed or overloaded with Personal product verbs.

## Decision

1. Keep the `admin-cli` package, and add a second binary named `cognitive`.
2. Implement Personal verbs in `apps/admin-cli/src/personal_cli/` as a library
   module used by the `cognitive` bin and integration tests.
3. Supported verbs for this batch:
   - `cognitive init`
   - `cognitive status`
   - `cognitive doctor`
   - `cognitive daemon start|status|stop`
4. CLI is a **non-authority client**:
   - may create XDG directories and run adapter-local migration prepare;
   - may write non-secret Provider config and SecretStore material via
     `ProviderKeyService`;
   - may spawn `kernel-server --personal` and call HTTP projections;
   - must not open SQLite authority tables to advance Task/Effect/Verification.
5. Secret capture:
   - `--api-key-file <path|->` for automation and Windows hosts;
   - interactive hidden input on Unix via echo-off when available;
   - fail closed rather than accept a visible interactive secret prompt.
6. `--allow-ephemeral-secret-backend` is tests-only and never a production
   plaintext fallback.
7. Base URL normalization strips trailing `/` and rejects `http://`, embedded
   credentials, and whitespace with actionable errors.
8. Re-init without provider key flags is idempotent and does not delete
   existing data or secrets.
9. Every JSON report includes `profile_claim: "not-claimed"` and
   `gate_claim: "not-claimed"`.

## Consequences

- P1-T07 can call the same daemon projections and readiness facts.
- P1-T08 installer can invoke `cognitive init` / `cognitive daemon` after
  bundle verification without inventing a second CLI surface.
- Local Windows GNU remains a non-supported host (P0-T01 linker exit 121);
  CI Ubuntu/Windows-MSVC is the authoritative executable evidence path.

## Rejected Alternatives

1. **Replacing `admin-cli` with `cognitive`** — would risk the deterministic
   emergency management path.
2. **CLI direct authority SQLite writes for status** — would make the client an
   authority writer and bypass the Personal daemon auth surface.
3. **Environment-variable Provider keys** — violates Personal secret boundary.
4. **Visible interactive secret prompts as fallback** — expands leak surface.
