# V02 CA OPS `status.inspect` Decision Matrix

- Decision packet: `V02-CA-OPS-STATUS-INSPECT-01`
- Date: 2026-07-23
- Branch: `lane/ctr-v02-ca-ops-foundation-closure`
- Baseline: `origin/main@117df63dfd435f57cac8b700e11a200517f56d0d`
- Foundation input: [V02-CA-OPS-FOUNDATION-01](V02-CA-OPS-FOUNDATION-DECISION-MATRIX.md)
- Classification: owner-governance member-closure audit; no machine registration
- Status: identity/classification, request/result, authority/security/verifier, error map, and audit responsibility confirmed; AUDIT machine-registration blocker forces member registration NO-GO; envelope/epoch binding not advanced

## 1. Boundary

This packet audits one candidate only. Owner confirmation fixes individual
facts without creating a descriptor, core membership, publication, selection,
authorization, implementation, behavior result, or Profile claim.

Private Rust request/report types, fallback dispatch, channel classification,
`OperationSummary`, routes, CLI behavior, and existing vectors remain gap
evidence only. They do not determine any field below.

## 2. Confirmed operation identity and classification

| Property | Owner-confirmed value |
|---|---|
| operation name | `status.inspect` |
| descriptor asset ID | `cognitiveos.operation-descriptor.management.status.inspect/0.2` |
| operation SemVer | `0.2.0-draft.1` |
| publication state | `draft` + `unpublished` |
| selection state | `selectable: false` |
| membership classification | `core_candidate`; not a core member |
| Effect class | `pure` |
| mutability | `read_only` |
| Intent/Effect | forbidden |
| external executor | forbidden |
| business state mutation | forbidden |

Adding a write, external Effect, or non-read-only semantic is breaking and
requires a new operation SemVer and descriptor digest.

## 3. Confirmed request selector

| Property | Owner-confirmed value |
|---|---|
| request schema asset ID | `status-inspect-request.schema.json` |
| complete SemVer | `0.2.0-draft.1` |
| schema version | `cognitiveos.management.status-inspect-request/0.2` |
| publication state | `draft` + `unpublished` |
| required fields | exactly `schema_version`, `subject_ref` |
| unknown fields | rejected (`additionalProperties: false`) |

The exact semantic selector is:

```json
{
  "schema_version": "cognitiveos.management.status-inspect-request/0.2",
  "subject_ref": {
    "kind": "weak",
    "id": "<UUIDv7>",
    "freshness": { "mode": "latest_authorized" },
    "resolution": {
      "strategy": "authority_current",
      "resolve_at": "authorization",
      "pin_result": true,
      "on_unavailable": "fail_closed"
    }
  }
}
```

Exactly one governed object is selected. Wildcards, lists, filters,
caller-selected projections, historical `as_of`, local-cache fallback, and
caller-supplied tenant/domain/type hints are forbidden. Session, scope,
deadline, correlation, and other envelope facts are not duplicated in the
payload. Resolution must produce and internally pin an authoritative strong
reference before a result is formed.

This is an owner-confirmed proposed request contract, not a created or
registered schema. Its canonical bytes and digest do not yet exist.

## 4. Confirmed result projection

| Property | Owner-confirmed value |
|---|---|
| result schema asset ID | `status-inspect-result.schema.json` |
| complete SemVer | `0.2.0-draft.1` |
| schema version | `cognitiveos.management.status-inspect-result/0.2` |
| publication state | `draft` + `unpublished` |
| digest domain | `status-inspect-result/0.2` |
| exact digest exclusion | `/result_digest` only |
| unknown fields | rejected (`additionalProperties: false`) |

Required fields are exactly:

```text
schema_version
subject_ref
read_authority_ref
state_domain
state_table_ref
state
result_digest
```

`subject_ref` is the current authoritative strong reference produced by request
resolution. `state_table_ref` is an exact `(asset_id, complete SemVer, digest)`
triple, and `state` must be a member of that digest-pinned table. The sole
high-watermark is `subject_ref.object_version`, paired with the same
`subject_ref.content_digest` from one authoritative read.

No global, Event, AUDIT, watch, SQLite, wall-clock, UUID-order, fencing, event
count, last-event, log, or private-report watermark is exposed. The result is
limited to governed objects whose state domain and transition table are
registered. It supports neither global ordering nor stream recovery and offers
no caller-selected projection.

This result contract remains proposed only. No schema bytes or digest exist.

## 5. Confirmed read authority source

The selected, digest-pinned `state-domains` registry is the sole source of the
authority role for `state_domain`. The resolved object's
`GovernedObjectHeader.authority_ref` strong reference is the sole authority
instance. Both the registry role and the header authority instance must match;
missing, stale, conflicting, or unresolvable authority facts fail closed.

The result's `read_authority_ref` is exactly the header's `authority_ref`. The
authority store must return the header, current state, and pinned transition-
table triple in one consistent read. The returned `subject_ref`, `state`, and
`result_digest` are formed from that same read point.

Event replay, watch projections, catalogs, `OperationSummary`, caches, replica
fallbacks, SQLite views, and private `InspectReport` values are not authority
sources. If the registered state domain uses a registered consensus or
arbitration protocol, `authority_ref` identifies that protocol authority. The
caller cannot select or override the authority.

This decision establishes a semantic binding only. It does not register a
state-domain profile, authority protocol, request/result schema, descriptor, or
member.

## 6. Confirmed capability, permission, and risk policy

The operation risk class is fixed at `R0` while the operation remains `pure`,
`read_only`, and limited to the confirmed minimum status projection. The
operation does not by itself require per-invocation approval. A current pinned
policy may require additional step-up or deny the read, but it cannot grant or
expand authority. Any change to the risk class, permission surface, or default
approval behavior requires a new operation SemVer and descriptor digest.

Authorization requires a current, active, unexpired, and unrevoked
`PrivilegedManagementSession` whose exact scope contains:

```text
management_domain: cognitiveos.management.status
action:            status.inspect
resource:          exact stable identity of the selected governed object
```

The effective capability is independently intersected across the authenticated
ActorChain. It must be current and unrevoked and must bind action
`status.inspect`, the same exact resource, the deciding receiver as audience,
the exact `ActivityContext` purpose, the descriptor/request-schema identities,
and the `latest_authorized` selector. Delegation is disabled with
`delegation.depth_remaining = 0`. Wildcard, tenant-wide, domain-wide, inferred,
descriptor-derived, membership-derived, discovery-derived, and reachability-
derived grants are forbidden. Explicit deny wins and absence of an exact grant
denies by default.

This permission authorizes only the confirmed minimum status projection. It
does not grant object-body, history, discovery, catalog, watch, Event, AUDIT,
or any other read. Session, capability, current policy version, and revocation
epoch are revalidated both immediately before the authoritative read and
immediately before releasing the result. Invalidation at either point returns
no result. The exact denial codes remain deliberately open for the complete
stage-to-error decision.

## 7. Confirmed existence-hiding rules

Before the caller passes the complete session, capability, purpose, resource,
and policy authorization checks, the following cases are caller-observably
indistinguishable: no current object exists; an object exists but read or
discovery is unauthorized; tenant or scope is wrong; or the object's domain,
type, or authority is outside caller visibility.

All such cases use one future stage-to-error row with the same code, category,
retryability, stage, transport status, envelope shape, response-size class, and
retry hints. This decision deliberately does not name that code. The response
must not return or imply tenant, domain, type, authority, strong reference,
version, digest, state, candidate count, or a permission-versus-absence reason.

Only after the caller is fully authorized for the exact stable object identity
may the response distinguish true absence from authority unavailability or a
state/table/consistent-read failure. Internal decision logic may retain the
true reason only for a future registered authoritative AUDIT carrier; the
caller-visible correlation identifier format cannot encode it.

Caches, catalogs, Events, watch projections, replicas, timing classes, and
error translation cannot form a secondary existence oracle. Compatibility
adapters cannot restore concealed information. Weakening this equivalence is a
breaking security change requiring a new operation SemVer and security review.

## 8. Confirmed authoritative readback and verifier

The sole authoritative readback is the deciding `read_authority_ref` authority
store returning the governed-object header, canonical object bytes, current
state, and state-table triple from the same consistent read point. A second
read is forbidden because it could race the snapshot it purports to verify.

The deciding read authority itself is the sole semantic verifier and applies a
deterministic result-release gate. Before release it verifies all of the
following:

1. selector stable ID equals the header ID;
2. header authority equals `read_authority_ref`;
3. canonical object bytes recompute to the strong reference content digest;
4. object version is current at that read point;
5. state domain is registered;
6. state-table `(asset_id, complete SemVer, digest)` is registered and matches;
7. state belongs to that pinned table;
8. every result field comes from the same read point;
9. result digest recomputes in domain `status-inspect-result/0.2`, excluding
   only `/result_digest`; and
10. session, capability, current policy, and revocation state remain valid
    immediately before release.

Clients, caches, replicas, Events, watch projections, catalogs, SQLite views,
and private DTOs are not semantic verifiers. Failure of any gate produces no
partial or degraded result. The exact code remains open for the stage-to-error
decision.

The result proves only the authority-current state at that consistent read
point. It does not claim freshness at client receipt or global ordering.
External clients may validate schema, result digest, and state-table membership
only; until envelope/SIG/AUDIT bindings close, they cannot claim independent
authority-provenance verification. If the authority store cannot atomically
supply and internally verify the required material, this candidate remains
NO-GO; a second read or replica fallback cannot repair it.

## 9. Confirmed complete stage-to-error map

The exact stage mapping is:

| Stage | Condition | Exact error |
|---|---|---|
| G0 | management peer, principal, ActorChain, or authentication context cannot authenticate | new `MANAGEMENT_AUTHENTICATION_FAILED` |
| G0 | task/management channel contexts mixed | existing `SHELL_CHANNEL_BINDING_MISMATCH` |
| G1 | major version outside compatibility window | existing `VERSION_UNSUPPORTED` |
| G1 | old, unknown, or superseded epoch | confirmed future `NEGOTIATION_EPOCH_MISMATCH` |
| G1/G2 | specification/operation-set identity mismatch | confirmed future `OPERATION_SET_MISMATCH` |
| G1 | schema-bundle digest mismatch | existing `PROTOCOL_SCHEMA_DIGEST_MISMATCH` |
| G1 | unknown critical extension | existing `CRITICAL_EXTENSION_UNKNOWN` |
| G1 | duplicate/conflicting/colliding/shadowing extension | confirmed future `EXTENSION_DUPLICATE`, `EXTENSION_IDENTITY_CONFLICT`, `EXTENSION_OPERATION_COLLISION`, or `EXTENSION_SHADOWING_DENIED`, exactly by condition |
| G1 | adapter loses mandatory semantics | existing `PROTOCOL_MAPPING_INCOMPLETE` |
| G2 | unknown operation | confirmed future `OPERATION_UNKNOWN` |
| G1/G2 | known but unnegotiated operation | confirmed future `OPERATION_NOT_NEGOTIATED` |
| G2 | descriptor identity/version/digest mismatch | confirmed future `OPERATION_DESCRIPTOR_MISMATCH` |
| registration/G2 | error map incomplete | confirmed future `OPERATION_ERROR_MAP_INCOMPLETE`; registration/selection forbidden |
| G3 | request fails selected schema | existing `SCHEMA_MISMATCH` |
| G2/G3 | request schema differs from pinned digest | existing `PROTOCOL_SCHEMA_DIGEST_MISMATCH` |
| G3 | canonical bytes differ from declared digest | existing `DIGEST_MISMATCH` |
| G4 | session expired or revoked | existing `MANAGEMENT_SESSION_EXPIRED` or `MANAGEMENT_SESSION_REVOKED` |
| G4 | current policy requires step-up without using protected existence facts | existing `MANAGEMENT_STEP_UP_REQUIRED` |
| G4 | management domain/action outside session scope | existing `MANAGEMENT_SCOPE_MISMATCH` |
| G4 | self-authorization | existing `MANAGEMENT_SELF_AUTHORIZATION_DENIED` |
| G4 | capability expired or illegally expanded | existing `AUTH_CAPABILITY_EXPIRED` or `AUTH_CAPABILITY_ATTENUATION_VIOLATION` |
| G4/G6 | capability revoked | new `AUTH_CAPABILITY_REVOKED` |
| G4/G6 | bounded revalidation cannot close one current policy/revocation context | new `MANAGEMENT_AUTHORIZATION_CONTEXT_STALE` |
| G4 | absent, hidden, cross-scope, or resource permission/policy denial | new `STATUS_INSPECT_SUBJECT_UNAVAILABLE` |
| G5 | domain role, header authority, and deciding read authority cannot close | new `STATUS_INSPECT_AUTHORITY_UNAVAILABLE` |
| G5/G6 | one-point authoritative read or deterministic verification unavailable | new `STATUS_INSPECT_READ_UNAVAILABLE` |
| G6 | state domain/table unregistered, or state outside pinned table | new `STATUS_INSPECT_STATE_BINDING_INVALID` |
| G6 | canonical object/result digest mismatch | existing `DIGEST_MISMATCH` |
| G6 | result differs from selected descriptor/result-schema binding | confirmed future `OPERATION_RESULT_BINDING_MISMATCH` |
| G6 | result fails selected JSON Schema | existing `SCHEMA_MISMATCH` |

The seven newly confirmed errors are not yet registered:

| Code | Category | Retryable | Exact responsibility |
|---|---|---:|---|
| `MANAGEMENT_AUTHENTICATION_FAILED` | `auth` | false | management peer/identity/ActorChain/authentication context fails before operation-payload processing |
| `AUTH_CAPABILITY_REVOKED` | `auth` | false | capability used for this decision is revoked at authorization or result release |
| `MANAGEMENT_AUTHORIZATION_CONTEXT_STALE` | `auth` | true | bounded revalidation cannot close one current policy/revocation context; retry only with a fresh authorization snapshot |
| `STATUS_INSPECT_SUBJECT_UNAVAILABLE` | `auth` | false | no caller-visible, currently authorized subject; absence, hiding, and denial are not distinguished |
| `STATUS_INSPECT_AUTHORITY_UNAVAILABLE` | `state` | true | registered domain role, header authority, and an available deciding read authority cannot close; retry only after governance/authority recovery |
| `STATUS_INSPECT_READ_UNAVAILABLE` | `state` | true | authority cannot provide or verify all same-read-point material; replica fallback forbidden |
| `STATUS_INSPECT_STATE_BINDING_INVALID` | `protocol` | false | state domain/table identity is unregistered or state is outside the pinned table; byte digest mismatch remains `DIGEST_MISMATCH` |

Every row has this zero-business-side-effect oracle:

```text
dispatches = 0
effects_created = 0
business_state_mutations = 0
authority_business_commits = 0
success_receipts = 0
partial_results = 0
```

G5/G6 may perform one authoritative read but cannot produce a business effect
or success result. After AUDIT registration, a runtime failure must produce
exactly one safe denial audit commit; no such behavior is executed or claimed
now. `STATE_CONFLICT`, `STATE_STORE_UNAVAILABLE`, `CONTEXT_AUTH_DENIED`,
`RESOURCE_NOT_DISCOVERABLE`, `SHELL_TARGET_NOT_FOUND`, and all `EFFECT_*`
errors are not substitutes for these responsibilities.

## 10. Confirmed audit responsibility and TRUE-NO-GO

The future authoritative AUDIT family must add the exact record kind
`privileged_read_decision` and the exact `AuthoritativeAuditPort`
responsibility `commit_privileged_read_decision`. Every terminal
`status.inspect` response has exactly one authoritative record:

- success uses `decision=allow` at G6;
- denial, challenge, or error records the corresponding decision, earliest
  deterministic failed stage, and exact registered error.

A success record binds the operation-set, descriptor, request/result schema,
and negotiation-epoch identities; ActorChain, session, capability, policy, and
revocation versions; deciding `read_authority_ref`; subject strong reference;
state-table triple and state; canonical request-parameters digest; result
digest; and correlation/causation. A concealed failure carries only a minimized
selector digest and safe internal reason. It cannot expose subject, tenant,
authority, or existence to the caller or an unauthorized audit reader.

Tenant subjects use the matching tenant stream partitioned by management domain
`cognitiveos.management.status`. Platform subjects, plus G0 failures for which
tenant cannot safely be established, use the platform stream. The audit record
must durably commit before either success or failure becomes visible. Audit
persistence failure suppresses the original response and, after AUDIT machine
registration, uses `AUDIT_STORE_UNAVAILABLE`.

A successful pure read has no business-state commit. Its audit commit is not a
joined business write, but it must bind the same subject strong reference,
object version, and result digest and complete before result release. Event,
log, trace, telemetry, SQLite rows, and the current URI-only result-envelope
`audit_ref` are not substitutes.

The eventual OPS descriptor must digest-pin registered AUDIT record, stream,
and commit-receipt triples. Those machine assets do not exist. Future,
unresolved, zero, or fake digests are forbidden, and this OPS batch cannot merge
AUDIT registration. Therefore the owner confirmed a TRUE-NO-GO for
`status.inspect` machine registration in this batch.

## 11. Mandatory decision intentionally not advanced

Envelope, negotiation epoch, and compatibility binding remains open. It is not
inferred after the AUDIT blocker. Its sole next entry is a fresh owner decision
after an independent AUDIT registration batch has registered the exact
privileged-read record/port/receipt responsibilities and final digests.

No request/result schema, descriptor, error, set member, generated binding, or
vector is created from the confirmed prose decisions.

## 12. Closure decision

| Candidate | Eligibility | Exact reason |
|---|---|---|
| `status.inspect` | `blocked / registration NO-GO` | required AUDIT record/stream/commit-receipt triples and privileged-read commit responsibility are unregistered; envelope/epoch binding remains open |

Until every row closes, no request/result schema, descriptor, set member,
error, generated binding, vector, or machine registration is eligible.

## 13. Preserved state

- Exact registered assets in this batch: none.
- All eight OPS candidates remain blocked.
- OPS/TARGET/SIG/AUDIT machine contracts remain unregistered.
- SIG independent security/cryptography review remains pending.
- D-016 remains open; D-022 remains blocking; CA-1 through CA-8 remain blocked.
- New OPS behavior execution: none; Profile `implemented`: `0`.
