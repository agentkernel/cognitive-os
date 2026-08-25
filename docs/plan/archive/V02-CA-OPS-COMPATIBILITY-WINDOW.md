# V02-CA-OPS-01 Compatibility Window Proposal

- Status: finite docs-only design; AUDIT owner-authorized review component completed; OPS registration eligibility NO-GO; SIG independent review and machine registration pending
- Proposed native set: exact `0.2.0-draft.1`
- Operation-set digest: `unresolved/not computed`
- Machine status: unregistered

## 1. Finite window

| Consumer mode | Accepted proposed set versions | Final support point | Notes |
|---|---|---|---|
| native v0.2 | exact `0.2.0-draft.1` | superseded by an explicitly published successor | exact set/descriptor/schema digests required |
| v0.1 migration adapter | exact `0.2.0-draft.1` and `0.2.0-draft.2` only | removed at `0.2.0-draft.3` | proposal only; adapter does not exist |
| v0.1 native epoch | no v0.2 set or extension | unchanged | v0.1 epoch cannot be upgraded in place |

`latest`, `0.x`, wildcard ranges, mutable branches, and indefinite extension are forbidden. Independent review may require a successor Draft number, but no pinned Draft is rewritten and the final window remains finite and explicit.

## 2. Selection rules

A receiver selects a set only when all of the following hold:

- exact specification-set and operation-set identities are available;
- requirement, schema-bundle, suite, descriptor, and request/result schema digests verify;
- every selected critical extension is understood and preserved;
- no core or extension collision exists;
- mapping is lossless for required governance semantics;
- authorization is revalidated in the new epoch.
- every selected configure descriptor pins an independently reviewed target
  profile, authority mapping, request/result schemas, consumer,
  readback/verifier, receipt, risk/approval policy, and audit slot.
- every selected session/approval signature profile pins an exact profile,
  signed schema, domain, projection/exclusions, pure Ed25519-only set, governed
  key-registry manifest, platform root/delegation, status policy, receipt slot,
  and critical-extension digest;
- authorization is revalidated after signature-profile/epoch selection, and a
  v0.1 signature string is never treated as a v0.2 detached envelope.
- the selected audit critical extension pins exact record, stream, checkpoint,
  retention, redaction, export and commit-receipt profiles, all schemas/digests,
  finite checkpoint thresholds, retention-policy values, governed checkpoint/
  export signing keys, and future error mappings;
- stream identity, tenant/compartment scope, expected high-watermark, writer
  epoch/fencing, previous-record digest, and checkpoint signature verify before
  authority commit or recovery progress;
- export pins the authorized filter, source checkpoint/high-watermark,
  deterministic redaction profile, ordered digest list, manifest and signature.

Failure uses `VERSION_UNSUPPORTED`, `CRITICAL_EXTENSION_UNKNOWN`, or `PROTOCOL_MAPPING_INCOMPLETE` only where the registered meaning exactly applies. Unknown-operation, unnegotiated-operation, and epoch-specific error closure remains unresolved.

## 3. Breaking changes within the window

Adding/removing/renaming a core member, changing descriptor semantics or binding,
changing risk/approval/authority/error mapping, changing signature algorithm/key
usage/trust root/domain/projection/exclusions, changing audit carrier/topology/
sequence/chain/checkpoint/signature/retention/legal-hold/redaction/export,
changing criticality, or removing support is breaking. A pinned Draft cannot be
modified in place; it receives a new complete SemVer and digest plus migration
note.

## 4. Current non-claim

No compatibility adapter, negotiation profile, operation set, extension,
target profile, configuration state domain, consumer, readback/verifier,
receipt, or digest is implemented or registered by this proposal. The TARGET
audit keeps all three configure candidates blocked, and all eight OPS
candidates remain blocked.

No detached-signature envelope, session/approval signature profile, Ed25519
profile, key descriptor/registry manifest, trust-root/delegation asset,
rotation/revocation service, replay ledger, error set, or verification receipt
is implemented or registered. Owner-confirmed design does not change the
existing string fields' v0.1 schema meaning.

No authoritative audit record, stream, checkpoint, retention/redaction policy,
legal-hold authority binding, export manifest/signature, audit persistence port,
error/category, critical extension, or digest is implemented or registered.
Owner-confirmed AUDIT design does not change Event, transition, receipt,
`audit_ref`, SQLite, vector, evidence, or Profile meaning.

An old or private target representation is not inside the window merely because
its URI, operation spelling, or JSON can be parsed. A future target adapter must
list exact source/target asset identities and remain finite; loss of target
authority, CAS, epoch, consumer, verification, receipt, approval, audit, or
critical-extension semantics fails closed.

## 5. Registration-readiness clarification

The proposed window does not make `0.2.0-draft.1` a published identity and does
not authorize an empty operation set. The 2026-07-23 eligibility audit found no
eligible member and no owner-closed foundation identity. Consequently there is
currently no selectable native set, adapter input, descriptor, extension, or
epoch to which this window can apply.
