# ADR-0023: Personal Readiness, Status, and Doctor Projection (P1-T05)

- Status: Accepted for P1-T05 implementation
- Date: 2026-07-25
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: Personal product application-service decision. Not a
  CognitiveOS specification requirement, registry REQ, schema, transition,
  vector, Profile claim, G0 claim, or B01-B12 claim.

## Context

P1-T03 provides Provider capability snapshots and digests. P1-T04 provides a
bounded, authenticated Personal daemon front door. CLI, Pi, and future UI need
one shared readiness fact source that distinguishes `blocked`, `degraded`, and
`ready` without rewriting static analysis success as runtime readiness.

Plan research text mentioned `cognitive-management`. That crate is Lane-RUN
owned. Prior Personal batches kept product surfaces in the composition root or
isolated crates to avoid taking Lane-RUN ownership.

## Decision

1. Implement `evaluate_personal_readiness` in
   `personal/apps/kernel-server/src/personal/readiness.rs` (Personal composition root).
2. Expose authenticated projections:
   - `GET /personal/status` and `GET /personal/readiness` (compact)
   - `GET /personal/doctor` (component facts, durations, guidance)
3. Require management-channel bearer; task-channel and unauthenticated callers
   fail closed. Cookie auth remains forbidden.
4. Component set for this batch:
   - required: `system`, `database`, `secret`, `provider`, `daemon`
   - optional/deferred: `pi` (`not_configured` until P1-T07)
5. Overall aggregation:
   - any required `blocked` → overall `blocked`
   - else any required `degraded` → overall `degraded`
   - else overall `ready`
6. `first_conversation_ready` is true only when overall is `ready` **and** Pi
   is `ready`. Pi remains `not_configured` in this batch, so first conversation
   stays false even when daemon-required components are ready.
7. Every projection includes:
   - `static_check_is_not_runtime_ready: true`
   - `profile_claim: "not-claimed"`
   - `gate_claim: "not-claimed"`
   - `authority_side_effects: false`
8. Secret material, bootstrap secret bytes, and opaque `SecretRef` strings are
   never serialized into status/doctor payloads.
9. Database check records file presence only; integrity campaigns remain
   `not-claimed`.

## Consequences

- P1-T06 CLI can call the same HTTP projections without inventing a second
  fact source or writing SQLite directly.
- P1-T07 can flip the Pi component from `not_configured` to real runtime
  checks without changing overall aggregation rules.
- CI Ubuntu/Windows-MSVC executes process and unit evidence. Local Windows GNU
  remains a non-supported host (P0-T01 linker exit 121).

## Rejected Alternatives

1. **Putting this service in `cognitive-management` immediately** — would take
   Lane-RUN ownership for a Personal product surface; deferred.
2. **Unauthenticated ready endpoints** — would expand the local attack surface
   beyond ADR-0019/0022.
3. **Claiming ready from static CI/schema success** — violates the acceptance
   requirement that static checks are not runtime ready.
4. **Embedding SecretRef or probe response bodies** — violates secret
   redaction invariants.
