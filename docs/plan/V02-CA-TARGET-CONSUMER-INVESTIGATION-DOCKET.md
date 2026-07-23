# V02 CA TARGET consumer investigation docket

- Docket ID: `V02-CA-TARGET-CONSUMER-DOCKET-01`
- Date: 2026-07-23
- Classification: WP-6 candidate investigation only; no target registration
- Status: **three consumer lines unproven; all machine-registration gates remain NO-GO**

## 1. Boundary

This docket is the permitted preparation for TARGET’s three independently
governed configure operations. It does not define a generic target, select a
target kind, register an operation, create a request/result schema, or reuse a
URI, route, row, vector, or private DTO as authority evidence.

Each row must have its own real apply consumer, authority, independent readback,
verifier, receipt, and owner. Evidence for one operation cannot close another.

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
| `system.configure` | Exact system/subsystem/policy target kind; authority source; real apply consumer; independent readback/verifier; risk/approval mapping; authority receipt and recovery model. | **Owner-confirmed 2026-07-23:** HAL9001 initiates management requests; an independent System Configuration Authority Owner must own apply/readback/verifier. Actual owner appointment and consumer evidence remain required. |
| `gateway.configure` | Exact instance/group granularity; routing/trust/egress authority; real consumer; fan-out/partial-apply policy; independent readback/verifier; receipt and reconciliation model. | **No real consumer identified; owner decision required.** |
| `diagnostics.configure` | Exact policy/sink/profile target; sensitivity/retention/export authority; real consumer; independent readback/verifier; receipt and partial-sink recovery model. | **No real consumer identified; owner decision required.** |

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
candidate independently passes §2 and its full owner record is reviewed. Three
configure operations remain required until the owner explicitly decides whether
all are necessary for D-022 closure.
