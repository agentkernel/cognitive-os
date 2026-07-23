# V02 CA D-016 minimum operation-member decision

- Date: 2026-07-23
- Classification: owner governance decision; docs-only
- Superseded by: ADR-0014 / `V02-ORDINARY-CORE-01`
- Decision: **five operations are required for Ordinary Core; three configure operations are High-Assurance extensions**

The Ordinary Core candidates are `status.inspect`, `session.create_restricted`,
`capability.revoke`, `execution.stop`, and `effect.reconcile`.

`system.configure`, `gateway.configure`, and `diagnostics.configure` remain
mandatory only for the High-Assurance extension scope. They do not block
Ordinary Core development, registration review, or release.

Each Core member must independently complete identity, request/result, authority, effect,
target/readback/verifier where applicable, error map, AUDIT responsibility,
consumer, registration review, and later machine-registration gates. This
decision does not register a member or set, change D-016/D-022 status, or
authorize a machine-registration or Profile claim. ADR-0014 separately opens
test-first internal tracer implementation before final registration.
