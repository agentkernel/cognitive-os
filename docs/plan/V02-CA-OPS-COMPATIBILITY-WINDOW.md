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

Failure uses `VERSION_UNSUPPORTED`, `CRITICAL_EXTENSION_UNKNOWN`, or `PROTOCOL_MAPPING_INCOMPLETE` only where the registered meaning exactly applies. Unknown-operation, unnegotiated-operation, and epoch-specific error closure remains unresolved.

## 3. Breaking changes within the window

Adding/removing/renaming a core member, changing descriptor semantics or binding, changing risk/approval/authority/error mapping, changing criticality, or removing support is breaking. A pinned Draft cannot be modified in place; it receives a new complete SemVer and digest plus migration note.

## 4. Current non-claim

No compatibility adapter, negotiation profile, operation set, extension, or digest is implemented or registered by this proposal. All eight candidates remain blocked.
