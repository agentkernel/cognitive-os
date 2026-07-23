# V02 CA D-016 minimum operation-member decision

- Date: 2026-07-23
- Classification: owner governance decision; docs-only
- Decision: **all eight candidates are required before D-016 may enter closure review**

The required candidates are `status.inspect`, `session.create_restricted`,
`capability.revoke`, `execution.stop`, `effect.reconcile`, `system.configure`,
`gateway.configure`, and `diagnostics.configure`.

Each must independently complete identity, request/result, authority, effect,
target/readback/verifier where applicable, error map, AUDIT responsibility,
consumer, registration review, and later machine-registration gates. This
decision does not register a member or set, change D-016/D-022 status, or
authorize implementation.
