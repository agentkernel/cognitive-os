# V02-CA-OPS-01 Migration Plan

- Status: design proposal for owner review
- Source: existing v0.1 assets and epochs
- Target proposal: `0.2.0-draft.1`; digest `unresolved/not computed`
- Classification: docs-only; migration not implemented

## 1. Preservation and identity

- Preserve all v0.1 bytes, digests, evidence, and negotiation epochs.
- Create new v0.2 specification-set, operation-set, descriptor, schema, and suite identities; never rewrite v0.1 assets.
- Treat existing operation spellings only as migration inputs, not as target membership or authorization.
- Reject the same asset ID/SemVer with different bytes or digest.

## 2. Epoch transition

1. terminate or explicitly supersede the old epoch;
2. authenticate peers again;
3. select and verify the new specification set and nested manifests;
4. select the operation set and critical extensions;
5. revalidate authorization, session scope, capabilities, risk, approval, target authority, and continuation;
6. issue a new epoch ID with explicit creation and expiry/termination rules;
7. only then admit a payload.

Reconnect and continuation do not silently restore an old epoch. An old epoch cannot obtain a v0.2 extension.

## 3. Candidate admission during migration

- An intended core candidate enters a published core only after descriptor closure and independent registration review.
- `diagnostics.configure`, `gateway.configure`, and `system.configure` become negotiable only after critical-extension and TARGET closure.
- A blocked candidate is absent from the machine manifest even if its spelling is recognized.
- Caller/plugin/private DTO membership injection is rejected.
- A configure candidate also requires an independently reviewed, digest-pinned
  target profile and exact authority/consumer/readback/receipt bindings. TARGET
  design intent is not a machine target identity.

## 4. Mapping and quarantine

- Pin source and target specification/operation sets and mapping-profile digest.
- Record provenance, loss, defaults, owner decisions, and authorization revalidation.
- Reject lossy mapping of required authority, session scope, capability, target, readback, approval, audit, error, or critical-extension semantics with `PROTOCOL_MAPPING_INCOMPLETE` where exact.
- Missing authority, scope, target, or readback enters reject or quarantine; it never defaults to platform/current tenant/public authority.
- Preserve in-flight Effects under their original idempotency, fencing, unknown-outcome, reconciliation, and audit obligations.
- URI targets, open proposal parameters, vector inputs, private rows/DTOs,
  telemetry configuration, and catalog projections are migration inputs only.
  They cannot be upgraded in place into target authority.
- Migration to a future target profile creates a new governed target identity
  and version, records the deciding authority and mapping digest, and pins the
  real consumer and readback/verifier. Ambiguous system targets, gateway
  instance/group choices, and diagnostics policy/sink/profile choices fail
  closed or enter quarantine.

## 5. Vector and evidence migration

- Preserve existing vector `expected` values.
- Add explicit v0.2 negotiation, identity-drift, extension, authorization-non-expansion, and pre-dispatch negative vectors in later registration/CFR batches.
- Keep planned vectors `not-run` until a runner executes them against real deterministic implementation and retains evidence.
- Do not migrate v0.1 evidence into a v0.2 Profile claim.

## 6. Adapter removal

The proposed adapter accepts only exact `0.2.0-draft.1` and `0.2.0-draft.2` target sets and is removed at `0.2.0-draft.3`. Concrete versions may change during owner review, but removal remains finite and published in advance.

## 7. Rollback

If any target identity, critical extension, descriptor, schema, mapping, authorization, target/readback, or audit obligation cannot be verified, migration fails closed before dispatch. v0.1 bytes remain intact; no partial v0.2 membership or success receipt is created.
