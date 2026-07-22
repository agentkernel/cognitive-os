# V02-CA-OPS-01 Management Operation-Set Design Decision

- Decision ID: `V02-CA-OPS-01`
- Date: 2026-07-22
- Status: **materialized for owner review; all candidates blocked**
- Design SemVer: `0.2.0-draft.1` (**proposal only; not a published or specified identity**)
- Set digest: `unresolved/not computed`
- Classification: docs-only structural design under [V02-CA-GOV-00](V02-CONFIGURATION-AUTHORITY-NORMATIVE-SURFACE-AUTHORIZATION.md) and [ADR-0009](../adr/0009-v02-configuration-authority-surface-expansion-governance.md)
- Governance ADR: [ADR-0010](../adr/0010-v02-management-operation-set-governance.md)
- Release and migration companions: [release notes](V02-CA-OPS-RELEASE-NOTES.md), [compatibility window](V02-CA-OPS-COMPATIBILITY-WINDOW.md), [migration plan](V02-CA-OPS-MIGRATION-PLAN.md)

## 1. Decision and status boundary

The selected design model is:

> a finite v0.2 closed core plus digest-pinned, explicitly negotiated, versioned extension sets.

The intended core candidates are:

1. `capability.revoke`
2. `effect.reconcile`
3. `execution.stop`
4. `session.create_restricted`
5. `status.inspect`

The intended critical-extension candidates are:

1. `diagnostics.configure`
2. `gateway.configure`
3. `system.configure`

Every candidate has at least one unresolved mandatory binding. Therefore:

- all eight candidates are `blocked`;
- there is no design-approved core member;
- there is no design-approved extension member;
- the target classification is design intent, not machine membership;
- the eight strings MUST NOT be registered as a set in bulk;
- vector spelling, reachability, or idempotency facts do not establish a business, wire, membership, or authorization contract.

This decision does not register a requirement, error, schema, transition, vector, descriptor, operation set, specification set, generated binding, implementation, evidence artifact, or Profile claim. No new behavior vector was executed. D-016 remains open; D-022 remains a blocker; CA-1 through CA-8 remain blocked.

## 2. Authorization non-expansion

Operation membership means only that an operation descriptor is present in the selected, digest-pinned operation set. Effective authorization is always the intersection:

```text
membership
∩ negotiation epoch
∩ channel binding
∩ session scope
∩ capability bounds
∩ risk ceiling
∩ approval policy
∩ target authority
```

An operation name, descriptor, reachability vector, API route, or CLI entry MUST NOT widen any term in that intersection. `OperationDescriptor` and `AuthorizationCapability` remain separate types and separate decisions. A reserved management name received on a task channel MAY be rejected first with `SHELL_CHANNEL_BINDING_MISMATCH`; reservation of the spelling is neither membership nor authorization.

Authorization is revalidated after extension selection and before dispatch. Any missing, stale, unknown, or mismatched authorization input fails closed with zero dispatch, Effect creation, business mutation, commit, or success receipt.

## 3. Existing source and machine-fact limits

The current AKP request `operation` and session `actions` members are open strings. AKP result `result`/`continuation` and management proposal `action`/`parameters` are open. `OperationSummary` is a discovery projection. None is a complete descriptor or membership registry.

| Operation | Business source | Existing machine facts | Vector fixes | Vector does not fix | Target class | Status / blocker |
|---|---|---|---|---|---|---|
| `session.create_restricted` | Core fallback; AKP §10.1; RFC-0001 §7.5 | session schema; open operation/actions; no issuance request/result | spelling and reachability | issuance wire, SIG, bootstrap authorization, complete errors | intended core | `blocked`: OPS + SIG |
| `status.inspect` | `ReadState`; fallback | catalog/summary projections only; no inspect request/result | spelling and reachability | selector, read authority, result, errors | intended core | `blocked` |
| `capability.revoke` | capability revocation invariant; fallback | capability schema; no revoke request/result | spelling and reachability | target, expected version, receipt, complete errors | intended core | `blocked` |
| `execution.stop` | `Cancel`; Core §18.3; fallback | fallback and idempotency vectors; shell control is not a management payload | spelling and idempotency scenario | target, reason, deadline, result, authorization | intended core | `blocked` |
| `effect.reconcile` | `ReconcileEffect`; Effect lifecycle | Effect/Verification references; no management wire | spelling | request/result, session/proposal/audit binding | intended core | `blocked` |
| `gateway.configure` | fallback gateway configuration | spelling/channel classification only | spelling and reachability | complete TARGET contract | intended critical extension | `blocked` |
| `diagnostics.configure` | fallback diagnostics configuration | spelling/channel classification only | spelling and reachability | complete TARGET contract | intended critical extension | `blocked` |
| `system.configure` | generic configure; proposal/gate | configuration/channel vectors; open action/payload | spelling, one R1 scenario, task-channel denial | system target, payload, consumer, readback, general risk | intended critical extension | `blocked` |

## 4. Closed-core rules

The future v0.2 core MUST be finite, explicit, and closed:

- no wildcard or prefix membership;
- each member receives owner review independently;
- operation names are globally unique across core and selected extensions;
- a manifest with a collision is rejected before payload processing;
- caller, plugin, private DTO, route, or local configuration cannot inject membership;
- a candidate cannot enter core until its descriptor and every mandatory binding close;
- add, remove, rename, semantic, binding, risk, approval, authority, or error changes are breaking;
- a pinned Draft is immutable; correction creates a new complete SemVer and digest;
- the member array is sorted by `(operation_name ASCII bytes, operation_semver, descriptor_digest)`.

The intended-core list in this document is not the closed core manifest. The future machine-registration batch must register each eligible member and prove closure; blocked members are excluded.

## 5. Versioned extension rules

Each extension MUST declare:

- globally namespaced extension ID;
- complete SemVer;
- `critical: true|false`;
- schema or explicit opaque-value rule;
- deterministic member list;
- descriptor asset triples and descriptor digests;
- operation-set digest pin;
- compatibility and migration references;
- authorization revalidation requirements.

`diagnostics.configure`, `gateway.configure`, and `system.configure` are intended to be critical because losing their target, approval, or audit semantics could widen governance. This is a design recommendation, not registered criticality.

The receiver MUST reject, before business payload processing:

- an unnegotiated extension operation;
- an unknown critical extension;
- a duplicate extension;
- extension shadowing of core;
- the same extension ID/version with different bytes or digest;
- a gateway that strips a critical extension;
- any attempt to use a v0.2 extension from an old epoch.

An unknown critical extension uses `CRITICAL_EXTENSION_UNKNOWN`. A gateway unable to preserve critical semantics uses `PROTOCOL_MAPPING_INCOMPLETE`. No existing code is reassigned to unknown-operation, unnegotiated-operation, or epoch-specific failures unless its registered meaning is exact.

## 6. Minimal descriptor contract for future registration

The future machine-registration batch MUST fix, as machine-verifiable data, at least:

- descriptor asset ID, complete SemVer, publication status, and SHA-256 digest;
- operation name and semantic version;
- core or extension membership;
- extension identity, version, and criticality where applicable;
- request schema asset triple `(asset_id, SemVer, digest)`;
- result schema asset triple `(asset_id, SemVer, digest)`;
- request and result digest domains;
- allowed channel;
- Effect class;
- idempotency contract;
- cancellation contract;
- unknown-outcome contract;
- reconciliation contract;
- risk and approval policy reference;
- permission and capability constraints;
- authority source;
- target profile reference;
- readback/verifier reference;
- stage-to-registered-error mapping;
- audit responsibility or registered audit-reference slot;
- transport/envelope binding;
- deprecation and removal policy;
- the authorization non-expansion invariant.

URI strings, open JSON, private rows/DTOs, examples, caller values, vector fixtures, or `OperationSummary` MUST NOT fill these fields.

## 7. Identity, digest, set, and epoch

### 7.1 Asset identity

The suggested first design SemVer is `0.2.0-draft.1`. It is not published or specified by this document. Each future published asset is identified by:

```text
(asset_id, complete SemVer, sha256 digest)
```

The same ID/SemVer with different bytes or digest is rejected. Published, pinned, negotiated, signed, or evidence-cited bytes are immutable.

### 7.2 Canonical logical manifest

The operation-set digest covers an RFC 8785 canonical logical manifest containing:

- set ID, complete SemVer, status, and publication time;
- canonical encoding profile;
- deterministic member entries and descriptor triples;
- explicit closed-core list;
- extension identities, versions, members, and criticality;
- requirement-set digest;
- schema-bundle digest;
- applicable-suite digest;
- selected critical-extension set;
- compatibility, migration, and negotiation-profile references;
- exclusions and rationale;
- explicit self-digest and signature exclusions.

Archive path/order, compression, timestamps, pretty printing, source-key order, and transport location do not affect the logical digest. The exact operation-set digest domain remains unresolved and must be registered; it cannot be inferred from an example.

### 7.3 Binding direction

```text
negotiation epoch
  -> specification set
    -> operation set
      -> operation descriptor
        -> request/result schemas
```

The dynamic epoch is not included in the static specification-set digest. An epoch binds at least a unique ID, authenticated peers, selected specification set, operation-set digest, schema-bundle digest, selected critical extensions, mapping profile, creation time, expiry/termination rule, and superseded epoch.

### 7.4 Verification order

Verification occurs in this order:

1. framing, authenticated peer, and channel;
2. epoch;
3. specification set;
4. requirement, schema, and suite nested manifests;
5. operation set;
6. extensions;
7. descriptor;
8. request/result schemas;
9. membership and negotiation;
10. payload contract;
11. session, capability, risk, permission, and target authority;
12. Effect creation and dispatch.

Any mismatch fails closed before payload-dependent side effects.

## 8. Binding matrix

All rows use the proposed design version `0.2.0-draft.1`; all set digests are `unresolved/not computed`. “Intended” is not machine membership.

### 8.1 Operational bindings

| Operation | Class | Request/result | Channel | Authority | Target/readback | Error mapping | Negotiation | Deferred owner | Status |
|---|---|---|---|---|---|---|---|---|---|
| `session.create_restricted` | intended core | unresolved OPS + SIG issuance request/result | authenticated management bootstrap | session issuance authority, unresolved SIG proof | session version, expiry, revocation, and SIG slots | session codes partly exist; signature/issuance closure unresolved | selected core plus bootstrap epoch | SIG/security | `blocked` |
| `status.inspect` | intended core | unresolved selector/result | privileged management | governed read authority | snapshot, version, digest, high-watermark | not-found, denial, and result closure unresolved | selected core | read-authority/security | `blocked` |
| `capability.revoke` | intended core | unresolved target/request/result | privileged management | capability authority | expected version and revocation receipt | `STATE_CONFLICT` partly applicable; revocation closure unresolved | selected core | security/policy | `blocked` |
| `execution.stop` | intended core | unresolved management stop request/result | privileged management | AgentExecution authority | authoritative post-state | `CANCEL_PENDING`/`CANCEL_TOO_LATE` partly applicable; remainder unresolved | selected core | execution authority | `blocked` |
| `effect.reconcile` | intended core | unresolved management reconcile request/result | privileged management | Effect/recovery authority | Effect, Verification, and readback | outcome/quarantine/state-conflict codes partly applicable | selected core | AUDIT/recovery | `blocked` |
| `gateway.configure` | intended critical extension | unresolved TARGET | privileged management | unresolved TARGET | unresolved TARGET slot | unresolved TARGET | explicit extension plus authorization revalidation | TARGET/AUDIT | `blocked` |
| `diagnostics.configure` | intended critical extension | unresolved TARGET | privileged management | unresolved TARGET | unresolved TARGET slot | unresolved TARGET | explicit extension plus authorization revalidation | TARGET/AUDIT | `blocked` |
| `system.configure` | intended critical extension | unresolved TARGET | privileged management; task channel denied | unresolved TARGET | unresolved TARGET slot | channel code exists; remainder unresolved | explicit extension plus authorization revalidation | TARGET/AUDIT | `blocked` |

### 8.2 Traceability, rejection, compatibility, and proof

| Operation | Set version / digest | Normative source | Exact current machine source | Risk / permission | Non-expansion proof | Earliest rejection stage | Compatibility / migration | Owner / reviewer | Blocker and evidence limitation |
|---|---|---|---|---|---|---|---|---|---|
| `session.create_restricted` | `0.2.0-draft.1`; `unresolved/not computed` | Core fallback; AKP §10.1; RFC-0001 §7.5 | session schema plus fallback vector spelling; open AKP operation/actions | bootstrap issuance must be independently authorized; risk policy unresolved | membership cannot issue a session; issuance authority and SIG must both pass | G1/G2, then G4 | finite window in compatibility plan; new epoch and session reissuance in migration plan | OPS owner; SIG/security reviewer | OPS+SIG closure absent; spelling/reachability only; planned tests not executed |
| `status.inspect` | `0.2.0-draft.1`; `unresolved/not computed` | Core `ReadState`; fallback | fallback vector spelling; catalog/summary projection | privileged read permission; tenant/scope and existence-hiding unresolved | membership does not grant read scope or turn a projection into authority | G2/G3/G4 | finite window; old views remain migration input only | OPS owner; read-authority/security reviewer | selector/result/error closure absent; no inspect wire evidence |
| `capability.revoke` | `0.2.0-draft.1`; `unresolved/not computed` | capability revoke invariant; fallback | capability schema plus fallback vector spelling | capability authority, expected version, anti-self-escalation | membership cannot revoke; capability authority and CAS must pass | G2/G3/G4 | finite window; existing capability bytes preserved | OPS owner; security/policy reviewer | request/receipt/error closure absent; no revoke wire evidence |
| `execution.stop` | `0.2.0-draft.1`; `unresolved/not computed` | `Cancel`; Core §18.3; fallback | fallback/idempotency vector spelling; shell-control schema is a different channel contract | execution authority, target scope, reason/deadline policy | membership does not select a target or authorize cancellation | G0/G2/G3/G4 | finite window; shell-control input is not silently retyped | OPS owner; execution-authority reviewer | management payload/result incomplete; vector scenarios are not descriptor evidence |
| `effect.reconcile` | `0.2.0-draft.1`; `unresolved/not computed` | `ReconcileEffect`; Effect lifecycle | Effect/Verification schemas and transition facts; fallback spelling | recovery authority, original idempotency/fencing, audit closure | membership cannot resolve unknown outcome or bypass recovery authority | G2/G3/G4/G6 | finite window; in-flight Effects retain old epoch and reconcile rules | OPS owner; AUDIT/recovery reviewer | management wire and audit responsibility incomplete; no behavior execution |
| `gateway.configure` | `0.2.0-draft.1`; `unresolved/not computed` | fallback configure gateway | fallback vector spelling and channel classification only | critical target-specific risk/approval/permission unresolved | membership and reachability cannot create gateway target authority | G1/G2/G3/G4 | critical explicit extension; old epoch denied; TARGET migration required | OPS owner; TARGET/AUDIT reviewer | complete TARGET contract absent; no consumer/readback evidence |
| `diagnostics.configure` | `0.2.0-draft.1`; `unresolved/not computed` | fallback configure diagnostics | fallback vector spelling and channel classification only | critical target-specific risk/approval/permission unresolved | membership and reachability cannot create diagnostics target authority | G1/G2/G3/G4 | critical explicit extension; old epoch denied; TARGET migration required | OPS owner; TARGET/AUDIT reviewer | complete TARGET contract absent; no consumer/readback evidence |
| `system.configure` | `0.2.0-draft.1`; `unresolved/not computed` | generic configure; proposal/gate | `MGMT-CONFIG-001`, shell-channel denial, open proposal action/parameters | critical target-specific risk/approval/permission unresolved | name and R1 example cannot create system target authority; task channel remains denied | G0/G1/G2/G3/G4 | critical explicit extension; old epoch denied; TARGET migration required | OPS owner; TARGET/AUDIT reviewer | system target/payload/consumer/readback closure absent; existing scenario is not general authority evidence |

## 9. Fail-closed negative plan

All cases below are **planned/not executed**. New vectors must be added in a future machine-registration/CFR batch; existing `expected` values are immutable.

Stages:

- G0: framing/channel/peer
- G1: epoch/specification set/extensions
- G2: operation set/descriptor/schema identity
- G3: request contract
- G4: session/capability/risk/permission/target
- G5: pre-dispatch
- G6: result/reconcile/commit

Common pre-dispatch oracle:

```text
dispatches = 0
effects_created = 0
business_state_mutations = 0
commits = 0
success_receipts = 0
```

| # | Planned negative | Stage | Planned oracle / responsibility | Status |
|---|---|---|---|---|
| 1 | unknown operation | G2 | reject before request parsing; exact error unresolved | planned/not executed |
| 2 | known but unnegotiated operation | G1/G2 | reject; exact error unresolved | planned/not executed |
| 3 | operation-set digest mismatch | G2 | `DIGEST_MISMATCH` only if registered mapping is exact | planned/not executed |
| 4 | specification/operation-set mismatch | G1/G2 | reject nested identity mismatch | planned/not executed |
| 5 | descriptor version/digest drift | G2 | terminate/suspend epoch; reject before payload | planned/not executed |
| 6 | schema-bundle drift | G1/G2 | `PROTOCOL_SCHEMA_DIGEST_MISMATCH` where exact | planned/not executed |
| 7 | old, unknown, or superseded epoch | G1 | reject; epoch-specific code unresolved | planned/not executed |
| 8 | unknown critical extension | G1 | `CRITICAL_EXTENSION_UNKNOWN` | planned/not executed |
| 9 | outside session scope | G4 | `MANAGEMENT_SCOPE_MISMATCH` where exact | planned/not executed |
| 10 | capability/risk/permission unsatisfied | G4 | condition-specific registered denial; no generic reuse | planned/not executed |
| 11 | channel mismatch | G0 | `SHELL_CHANNEL_BINDING_MISMATCH` where exact | planned/not executed |
| 12 | request contract mismatch | G3 | `SCHEMA_MISMATCH` or exact digest error | planned/not executed |
| 13 | result contract mismatch | G6 | reject success; reconcile/quarantine as required | planned/not executed |
| 14 | target/readback missing | G4/G6 | reject or quarantine; TARGET responsibility unresolved | planned/not executed |
| 15 | error mapping incomplete | G2/G3 | descriptor cannot be registered or selected | planned/not executed |
| 16 | gateway strips critical extension | G1 | `PROTOCOL_MAPPING_INCOMPLETE` | planned/not executed |
| 17 | operation name treated as authorization | G4 | deny with common zero-side-effect oracle | planned/not executed |
| 18 | reachability treated as permission | G4 | deny with common zero-side-effect oracle | planned/not executed |
| 19 | dispatch before rejection | G5 | test fails if `dispatches != 0` | planned/not executed |
| 20 | Effect before rejection | G5 | test fails if `effects_created != 0` | planned/not executed |
| 21 | state mutation before rejection | G5 | test fails if business mutation occurs | planned/not executed |
| 22 | commit before rejection | G5/G6 | test fails if commit occurs | planned/not executed |
| 23 | success receipt after rejection | G6 | test fails if a success receipt exists | planned/not executed |

Registered codes potentially usable only when their definitions exactly match are `DIGEST_MISMATCH`, `PROTOCOL_SCHEMA_DIGEST_MISMATCH`, `VERSION_UNSUPPORTED`, `CRITICAL_EXTENSION_UNKNOWN`, `SCHEMA_MISMATCH`, `PROTOCOL_MAPPING_INCOMPLETE`, `MANAGEMENT_SCOPE_MISMATCH`, `MANAGEMENT_SELF_AUTHORIZATION_DENIED`, `SHELL_CHANNEL_BINDING_MISMATCH`, `EFFECT_OUTCOME_UNKNOWN`, `EFFECT_RECOVERY_QUARANTINED`, `CATALOG_VERSION_STALE`, and condition-specific capability/management errors.

Unknown operation, unnegotiated operation, and epoch-specific failures do not currently have a proven exact error closure. Their ownership remains unresolved; this decision does not repurpose a nearby code.

## 10. Release, compatibility, and migration decision

The release delta is documented in [V02-CA-OPS-RELEASE-NOTES.md](V02-CA-OPS-RELEASE-NOTES.md). The proposed finite compatibility window is documented in [V02-CA-OPS-COMPATIBILITY-WINDOW.md](V02-CA-OPS-COMPATIBILITY-WINDOW.md). Migration is documented in [V02-CA-OPS-MIGRATION-PLAN.md](V02-CA-OPS-MIGRATION-PLAN.md).

No adapter, set, descriptor, schema, epoch, or migration implementation exists as a result of those documents.

## 11. GO/NO-GO and next owners

### Design-materialization result

- `GO`: the docs-only design packet is complete enough for owner review.
- `NO-GO`: machine registration, implementation, and behavior execution.

### Per-candidate closure gate

A candidate remains blocked until all mandatory descriptor bindings close, exact machine assets are separately registered, new negative vectors are added without changing old `expected`, and the registration PR passes its own review and CI.

### Downstream order

1. owner review and merge of this OPS design packet;
2. TARGET design for the three configure candidates;
3. SIG design;
4. AUDIT design;
5. four separate machine-registration batches and generated bindings;
6. independent CA-0 re-review;
7. explicit CA-0 GO;
8. only then, implementation and Management CFR.

OPS merge alone does not unblock TARGET implementation, CA-1 through CA-8, or any behavior vector.

## 12. 2026-07-23 registration-readiness audit

After PR #54 merged and main CI `29937238562` succeeded on Ubuntu and Windows,
the owner authorized a security-first registration eligibility audit. The result
is recorded in
[V02-CA-OPS-REG-READINESS-01](V02-CA-OPS-REGISTRATION-ELIGIBILITY-AUDIT.md):

- all eight candidates remain `blocked` after every mandatory binding was
  rechecked;
- no operation member is eligible for machine registration;
- no descriptor/set foundation is eligible without unresolved owner choices;
- no machine asset, implementation, behavior result, or Profile claim changed.

The owner-authorized AUDIT security/audit/compliance review component is
completed, but it is not an external human, third-party, or GitHub review. SIG
independent security/cryptography review remains pending. Neither fact supplies
an unregistered TARGET, SIG, or AUDIT triple to an OPS descriptor.

## 13. Preserved state

- v0.1 assets and identities unchanged;
- 273 requirements, 55 errors, 61 schemas, 84 vectors;
- 59 pass, 25 not-run, self-check 40;
- matrix non-empty implementation count 70;
- Profile implemented = 0;
- D-016 open;
- D-022 blocking;
- CA-1 through CA-8 blocked;
- machine contracts unregistered;
- implementation not provided;
- new behavior not executed.
