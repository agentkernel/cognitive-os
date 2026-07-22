# V02-CA-OPS-01 Compatibility Window Proposal

- Status: finite design proposal for owner review
- Proposed native set: exact `0.2.0-draft.1`
- Operation-set digest: `unresolved/not computed`
- Machine status: unregistered

## 1. Finite window

| Consumer mode | Accepted proposed set versions | Final support point | Notes |
|---|---|---|---|
| native v0.2 | exact `0.2.0-draft.1` | superseded by an explicitly published successor | exact set/descriptor/schema digests required |
| v0.1 migration adapter | exact `0.2.0-draft.1` and `0.2.0-draft.2` only | removed at `0.2.0-draft.3` | proposal only; adapter does not exist |
| v0.1 native epoch | no v0.2 set or extension | unchanged | v0.1 epoch cannot be upgraded in place |

`latest`, `0.x`, wildcard ranges, mutable branches, and indefinite extension are forbidden. Owner review may change the concrete Draft numbers, but the final window must remain finite and explicit.

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

Failure uses `VERSION_UNSUPPORTED`, `CRITICAL_EXTENSION_UNKNOWN`, or `PROTOCOL_MAPPING_INCOMPLETE` only where the registered meaning exactly applies. Unknown-operation, unnegotiated-operation, and epoch-specific error closure remains unresolved.

## 3. Breaking changes within the window

Adding/removing/renaming a core member, changing descriptor semantics or binding, changing risk/approval/authority/error mapping, changing criticality, or removing support is breaking. A pinned Draft cannot be modified in place; it receives a new complete SemVer and digest plus migration note.

## 4. Current non-claim

No compatibility adapter, negotiation profile, operation set, extension,
target profile, configuration state domain, consumer, readback/verifier,
receipt, or digest is implemented or registered by this proposal. The TARGET
audit keeps all three configure candidates blocked, and all eight OPS
candidates remain blocked.

An old or private target representation is not inside the window merely because
its URI, operation spelling, or JSON can be parsed. A future target adapter must
list exact source/target asset identities and remain finite; loss of target
authority, CAS, epoch, consumer, verification, receipt, approval, audit, or
critical-extension semantics fails closed.
