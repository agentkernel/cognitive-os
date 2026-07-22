# V02-CA-OPS-01 Migration Plan

- Status: design with owner-confirmed SIG and AUDIT selections; independent reviews pending
- Source: existing v0.1 assets and epochs
- Target proposal: `0.2.0-draft.1`; digest `unresolved/not computed`
- Classification: docs-only; migration not implemented

## 1. Preservation and identity

- Preserve all v0.1 bytes, digests, evidence, and negotiation epochs.
- Create new v0.2 specification-set, operation-set, descriptor, signature-profile,
  signed-schema, audit-record/stream/checkpoint/retention/redaction/export/
  commit-receipt profile, schema-bundle, and suite identities; never rewrite
  v0.1 assets.
- Treat existing operation spellings only as migration inputs, not as target membership or authorization.
- Reject the same asset ID/SemVer with different bytes or digest.

## 2. Epoch transition

1. terminate or explicitly supersede the old epoch;
2. authenticate peers again;
3. select and verify the new specification set and nested manifests;
4. select the operation set, three critical SIG extensions, the critical AUDIT
   extension, session/approval
   signature profiles, pure Ed25519 set, governed registry manifest, and
   platform-root/delegation/status profiles, plus exact audit record/stream/
   checkpoint/retention/redaction/export profiles and policy/key pins;
5. verify and reissue the session or rechallenge/redecide approval under the new
   profile, then revalidate authorization, session scope, capabilities, risk,
   approval, target authority, and continuation;
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
- A v0.1 `authority_signature` string is not a detached envelope. Migration
  creates a newly authenticated/reissued session version and a newly challenged
  approval decision under distinct v0.2 signature profiles; no algorithm, key,
  trust root, domain, or projection is inferred from old bytes.
- Existing Event/transition/outbox/SQLite rows, `DenialAudit`, `audit_ref`,
  booleans, logs, traces, telemetry and vector audit facts are migration inputs
  only. They are never relabeled as authoritative audit records or used to
  fabricate historical chain continuity.
- The new audit epoch establishes or verifies a genesis checkpoint, exact stream
  partition, writer epoch/fencing and high-watermark before admitting work.
  Unverifiable continuity fails closed or enters quarantine.

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
- Caller-provided algorithms, keys, resolvers, trust roots, projection rules,
  signature booleans, schema-valid fixtures, or cached key rows are migration
  inputs only and never verification authority. Unknown or stale key/trust
  facts fail closed.
- A retiring predecessor verifies only objects signed before successor
  activation and only within 24 hours; revocation has no grace period. Session
  expansion or absolute-expiry extension creates a newly authenticated session,
  while approvals are rechallenged and atomically consumed under the new epoch.
- Imported legacy audit facts, if a future mapping accepts them, receive new
  record IDs/sequences with explicit source provenance and loss declarations.
  They do not backdate, fill gaps, or assert a checkpoint before verification.
- Retention migration selects exact policy values; expiry never silently
  deletes records. Legal holds survive migration. Redaction/export mappings pin
  exact source ranges, policies, checkpoints, high-watermarks and signatures.

## 5. Vector and evidence migration

- Preserve existing vector `expected` values.
- Add explicit v0.2 negotiation, identity-drift, extension, signature profile,
  algorithm downgrade, key/trust/rotation/revocation, cross-object replay,
  audit carrier/stream/sequence/chain/checkpoint/atomicity/retention/legal-hold/
  redaction/export, authorization-non-expansion, and pre-dispatch negative
  vectors in later registration/CFR batches.
- Keep planned vectors `not-run` until a runner executes them against real deterministic implementation and retains evidence.
- Do not migrate v0.1 evidence into a v0.2 Profile claim.

## 6. Adapter removal

The proposed adapter accepts only exact `0.2.0-draft.1` and `0.2.0-draft.2` target sets and is removed at `0.2.0-draft.3`. Independent review may require successor Draft identities, but no pinned Draft is rewritten and removal remains finite and published in advance.

## 7. Rollback

If any target identity, critical extension, descriptor, signature profile,
signed schema/projection, algorithm, key/trust/status, mapping, authorization,
target/readback, audit record/stream/checkpoint/policy/export, or atomic
obligation cannot be verified, migration fails closed before dispatch. v0.1
bytes remain intact; no partial v0.2 membership, session, approval, audit chain,
or success receipt is created.
