# V02 CA TARGET consumer investigation docket

- Docket ID: `V02-CA-TARGET-CONSUMER-DOCKET-01`
- Date: 2026-07-23
- Classification: WP-6 candidate investigation only; no target registration
- Status: **High-Assurance extension preparation; does not block Ordinary Core**

## 1. Boundary

This docket is the permitted preparation for TARGET’s three independently
governed configure operations. It selects the target models recorded below but
does not define a generic target, register an operation, create a request/result
schema, or reuse a URI, route, row, vector, or private DTO as authority evidence.

Each row must have its own real apply consumer, authority, independent readback,
verifier, receipt, and owner. Evidence for one operation cannot close another.
HAL9007 may be the accountable verifier owner for all three lines, but each line
must have a distinct verifier identity/version, criteria, pinned postcondition,
and evidence packet.

## 2. Shared consumer qualification test

For each candidate, the assigned consumer owner must prove all of the following:

1. the target is a strong identity with an independent authoritative owner;
2. the apply consumer is real and separate from the proposal/request carrier;
3. readback is independently authorized and not an echo of the apply request;
4. a deterministic verifier evaluates a pinned postcondition;
5. version/CAS and writer epoch are enforced before apply and commit;
6. partial apply, unknown outcome, cancellation, and reconciliation have a
   responsible authority;
7. the authority receipt binds target, versions, epoch, Effect, verification,
   Event, and future AUDIT slot; and
8. target discovery, membership, extension selection, or readback does not
   expand authorization.

Any missing statement leaves that operation absent from an operation or
extension set.

## 3. Independent investigation records

| Candidate | Owner must determine | Current result |
|---|---|---|
| `system.configure` | Exact system/subsystem/policy target kind; authority source; real apply consumer; independent readback/verifier; risk/approval mapping; authority receipt and recovery model. | **Owner-confirmed 2026-07-23:** HAL9001 initiates; HAL9004 owns exact system/subsystem/policy apply/readback/receipt; HAL9007 independently verifies. Deployment/consumer evidence remains required. |
| `gateway.configure` | Exact instance/group granularity; routing/trust/egress authority; real consumer; fan-out/partial-apply policy; independent readback/verifier; receipt and reconciliation model. | **Owner-confirmed 2026-07-23:** HAL9005 owns per-instance authority; group rollout decomposes to versioned instance operations; HAL9007 independently verifies. Deployment/consumer evidence remains required. |
| `diagnostics.configure` | Exact policy/sink/profile target; sensitivity/retention/export authority; real consumer; independent readback/verifier; receipt and partial-sink recovery model. | **Owner-confirmed 2026-07-23:** HAL9006 owns a diagnostics-policy target; sink/profile/credential/retention/export are strong refs; HAL9007 independently verifies. Deployment/consumer evidence remains required. |

For every candidate the owner record must attach target identity, authority,
payload/result shape, expected version/CAS, writer epoch, consumer owner,
readback authority, verifier identity/version, risk/approval/capability mapping,
negative oracle, error responsibility, SIG/AUDIT dependencies, and finite
migration/compatibility policy.

## 4. Required negative oracle

Before registration, the consumer owner must show a deterministic fail-closed
outcome for missing/wrong target authority, stale version, wrong writer epoch,
missing consumer/readback/verifier/receipt, insufficient scope/capability/risk/
approval, unnegotiated extension, old epoch, partial apply, audit failure, and
unknown outcome. In a pre-dispatch denial:

```text
dispatches = 0
effects_created = 0
business_state_mutations = 0
commits = 0
success_receipts = 0
```

This is a design obligation only. No new vector or behavior result is created by
this docket.

## 5. Stop condition

Do not create a generic configuration target, target profile, descriptor,
extension, error, generated binding, vector, or implementation until one
candidate independently passes §2 and its full owner record is reviewed. All
three configure operations are owner-confirmed as mandatory for D-016/D-022;
none may borrow another line's consumer or verifier evidence.
