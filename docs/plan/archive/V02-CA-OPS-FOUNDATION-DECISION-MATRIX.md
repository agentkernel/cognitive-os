# V02 CA OPS Foundation Decision Matrix

- Decision packet: `V02-CA-OPS-FOUNDATION-01`
- Date: 2026-07-23
- Branch: `lane/ctr-v02-ca-ops-foundation-closure`
- Baseline: `origin/main@117df63dfd435f57cac8b700e11a200517f56d0d`
- Classification: owner-governance decision; no machine registration
- Status: foundation decisions closed; consumer gate returned NO-GO; no machine asset registered; `status.inspect` member audit next

## 1. Boundary

The repository owner confirmed the exact IDs below. This confirmation fixes
names only. It does not publish, specify, select, register, digest-pin, or add
membership; it grants no authorization and creates no generated binding.

The three foundation schema IDs are conditional names only. They MUST NOT be
created until an independently useful real consumer is identified. A
descriptor-only template or an empty operation/extension set remains NO-GO.

## 2. Confirmed exact IDs

### Foundation schema IDs (conditional; not created)

| Asset | Exact ID | Current state |
|---|---|---|
| operation-set schema | `management-operation-set.schema.json` | owner-confirmed name; not created |
| operation-descriptor schema | `management-operation-descriptor.schema.json` | owner-confirmed name; not created |
| operation-extension-set schema | `management-operation-extension-set.schema.json` | owner-confirmed name; not created |

### Core operation set

| Asset | Exact ID | Current state |
|---|---|---|
| management core set | `cognitiveos.operation-set.management-core/0.2` | owner-confirmed name; unpublished and unregistered |

### Descriptor assets

| Candidate | Exact ID | Current state |
|---|---|---|
| `session.create_restricted` | `cognitiveos.operation-descriptor.management.session.create_restricted/0.2` | name only; candidate blocked |
| `status.inspect` | `cognitiveos.operation-descriptor.management.status.inspect/0.2` | name only; candidate blocked |
| `capability.revoke` | `cognitiveos.operation-descriptor.management.capability.revoke/0.2` | name only; candidate blocked |
| `execution.stop` | `cognitiveos.operation-descriptor.management.execution.stop/0.2` | name only; candidate blocked |
| `effect.reconcile` | `cognitiveos.operation-descriptor.management.effect.reconcile/0.2` | name only; candidate blocked |
| `gateway.configure` | `cognitiveos.operation-descriptor.management.gateway.configure/0.2` | name only; candidate blocked |
| `diagnostics.configure` | `cognitiveos.operation-descriptor.management.diagnostics.configure/0.2` | name only; candidate blocked |
| `system.configure` | `cognitiveos.operation-descriptor.management.system.configure/0.2` | name only; candidate blocked |

### Independent critical extension sets

| Candidate | Exact ID | Current state |
|---|---|---|
| `gateway.configure` | `cognitiveos.operation-extension-set.management.gateway.configure/0.2` | name only; candidate blocked |
| `diagnostics.configure` | `cognitiveos.operation-extension-set.management.diagnostics.configure/0.2` | name only; candidate blocked |
| `system.configure` | `cognitiveos.operation-extension-set.management.system.configure/0.2` | name only; candidate blocked |

Independent extension-set identities preserve separate TARGET/SIG/AUDIT
closure, review, publication, migration, and rollback boundaries. They do not
make any configure operation a member.

## 3. Confirmed version and publication state

The owner confirmed the following exact state for every ID in section 2:

| Property | Confirmed value |
|---|---|
| complete SemVer | `0.2.0-draft.1` |
| asset status | `draft` |
| publication status | `unpublished` |
| publication time | absent |
| canonical digest | absent / not computed |
| negotiation or selection | forbidden |
| membership | none |

This is a traceable draft identity decision, not publication or machine
registration. Before publication, final canonical bytes, an exact digest, and
all remaining gates below must close. Publication cannot mutate already
published or digest-pinned bytes under the same ID/SemVer.

## 4. Confirmed canonical digest contract

The owner confirmed these exact logical digest contracts. No digest is computed
until final schema-valid bytes exist.

| Asset kind | Canonical profile | Domain | Exact excluded pointer |
|---|---|---|---|
| operation set | `cognitiveos.canonical-json/0.1` | `management-operation-set/0.2` | `/set_digest` |
| operation descriptor | `cognitiveos.canonical-json/0.1` | `management-operation-descriptor/0.2` | `/descriptor_digest` |
| operation extension set | `cognitiveos.canonical-json/0.1` | `management-operation-extension-set/0.2` | `/extension_set_digest` |

For each asset, the complete semantic object is parsed and validated against
its selected closed schema before projection. Unknown fields, duplicate member
names, defaults, type coercion, and non-I-JSON values are rejected. The digest
projection contains every schema-known field except the one exact self-digest
pointer above. Membership, schema triples, channel, Effect/idempotency,
authority, target/readback, risk/approval, permission/capability, error, audit,
compatibility, migration, and negotiation bindings are never excluded.

The initial logical assets contain no embedded signature field. A future
signature is a detached asset that binds the exact content digest. Embedding a
signature, changing a domain, changing a projection, or adding/removing an
excluded path is breaking and requires a new complete SemVer and digest.

Archive layout, paths, compression, timestamps outside the logical asset,
pretty printing, source key order, and transport location never enter the
logical digest.

## 5. Confirmed membership and selection rules

The owner confirmed these fail-closed rules:

1. An operation set or operation extension set has at least one member. A
   zero-member set is invalid and cannot be published or machine-registered.
2. A descriptor becomes a member only after every mandatory binding is closed,
   the descriptor is `published`, its final digest is available, and its asset
   status is `candidate` or `approved`.
3. A `draft` set is always `selectable: false`, including after publication. It
   cannot enter a negotiation epoch.
4. All current `draft` + `unpublished` descriptor identities remain outside
   membership.
5. An unpublished candidate is absent from every machine set. It is not listed
   through `excluded_candidates` or another machine field that would make the
   operation known. Exclusion rationale remains in governance, release, and
   migration documents until the candidate is published and digest-pinned.
6. A foundation schema may be registered independently only after its real
   consumer gate closes. A schema never creates a set, member, selectable
   operation, or authorization fact.

Consequently, no current operation set, extension set, or member is eligible
for publication or machine registration.

## 6. Confirmed freeze and activation order

The owner confirmed this one-way order:

```text
requirement set
      -> schema bundle (including final request/result schemas)
      -> operation descriptor
      -> operation / extension set
      -> specification set
      -> conformance suite
      -> profile or claim selection and a new negotiation epoch
```

Rules:

1. Each layer references only already frozen, published, digest-complete lower
   assets.
2. The specification set pins the requirement set, schema bundle, and
   operation/extension sets. It does not contain a conformance-suite digest.
3. A conformance suite pins the specification-set digest and vector assets. A
   downstream Profile or claim manifest pins both the suite and specification
   set.
4. For runtime negotiation, the suite is not applicable content of the
   specification set. It is a downstream conformance-claim input.
5. A lower-layer digest change creates new upper-layer SemVer/digest identities;
   no published or pinned identity is recomputed in place.
6. A new negotiation epoch may activate only after all applicable assets are
   complete. Partial activation and insertion into an old epoch are forbidden.

This order removes a specification-set/suite digest cycle while retaining exact
claim pins.

## 7. Confirmed cross-family digest direction

The owner confirmed this static dependency direction:

```text
TARGET profiles --+
SIG profiles -----+--> OPS descriptor --> operation set --> specification set
AUDIT profiles ---+
```

Rules:

1. TARGET, SIG, and AUDIT static assets do not digest-pin an OPS descriptor,
   operation set, or specification set.
2. A lower static profile may define a required runtime field for an exact
   operation/specification binding, but its own profile digest does not contain
   a concrete upper-layer digest.
3. A runtime TARGET receipt, SIG binding record, or AUDIT record may carry the
   exact specification/operation/descriptor triples selected by the current
   negotiation epoch. Those are instance values, not reverse static pins.
4. An OPS descriptor may pin exact, already registered TARGET, SIG, and AUDIT
   profile triples where its semantics require them.
5. The specification set aggregates and pins the OPS set and all applicable
   TARGET, SIG, and AUDIT assets.
6. An unregistered family remains a blocker. Future, zero, unresolved,
   placeholder, or fake digests are forbidden.
7. The four machine-registration lines remain independent. A cross-family OPS
   member closes only after its required lower-family registrations.

This direction does not register TARGET, SIG, or AUDIT and does not combine
their registration work with OPS.

## 8. Confirmed error taxonomy

The owner confirmed the following future `protocol` errors. All are
`retryable: false`. They remain unregistered until a machine asset with a real
consumer is eligible; this decision does not edit `errors.yaml`, `common-defs`,
or generated bindings.

| Future code | Exact responsibility | Stage |
|---|---|---|
| `OPERATION_UNKNOWN` | operation name is absent from every published descriptor | G2 |
| `OPERATION_NOT_NEGOTIATED` | a published descriptor is not a member of the selected set/extension | G1/G2 |
| `OPERATION_SET_MISMATCH` | set triple differs from the specification/epoch pin, including same ID/SemVer with different digest | G1/G2 |
| `OPERATION_DESCRIPTOR_MISMATCH` | descriptor triple differs from the set member pin, including same ID/SemVer with different digest | G2 |
| `NEGOTIATION_EPOCH_MISMATCH` | epoch is unknown, expired, or superseded | G1 |
| `EXTENSION_DUPLICATE` | one exact extension triple occurs more than once | G1 |
| `EXTENSION_IDENTITY_CONFLICT` | one extension ID/SemVer resolves to different bytes or digest | G1 |
| `EXTENSION_OPERATION_COLLISION` | two selected extensions declare the same operation name | G1/G2 |
| `EXTENSION_SHADOWING_DENIED` | an extension operation name equals a core member name | G1/G2 |
| `OPERATION_RESULT_BINDING_MISMATCH` | result schema triple/domain differs from the descriptor and no external Effect can have occurred | G6 |
| `OPERATION_ERROR_MAP_INCOMPLETE` | a descriptor omits a mandatory stage/error responsibility and is rejected at asset admission | registration/G2 |

`DIGEST_MISMATCH` remains the exact code for recomputed bytes differing from a
declared digest. `SCHEMA_MISMATCH` remains the exact code for a value failing
its selected registered schema. If an effecting operation has dispatched and
its result is untrustworthy, `EFFECT_OUTCOME_UNKNOWN` applies instead of
`OPERATION_RESULT_BINDING_MISMATCH`.

The common no-business-side-effect oracle for registration, G1, G2, and a G6
failure where no external Effect can have occurred is:

```text
dispatches = 0
effects_created = 0
business_state_mutations = 0
authority_business_commits = 0
success_receipts = 0
```

After an authoritative AUDIT profile is registered, a runtime denial also
requires exactly one safe `denial_audit_commit`. That commit is not an authority
business commit.

## 9. Foundation consumer decision

The owner confirmed `NO-GO / not created / not registered` for all three
foundation schemas because no independent real consumer exists:

| Conditional schema | Consumer audit | Decision |
|---|---|---|
| `management-operation-set.schema.json` | no negotiation/admission path consumes a complete digest-pinned set; AKP `operation` remains open | NO-GO |
| `management-operation-descriptor.schema.json` | `OperationSummary` is a discovery projection; the private Rust descriptor and codegen are not cross-boundary consumers | NO-GO |
| `management-operation-extension-set.schema.json` | the current envelope entry is only `{id, critical}`; channel classification is not a set consumer | NO-GO |

The confirmed identity, version, digest, ordering, cross-family, and error
decisions remain owner-governance inputs. They do not justify a template,
empty set, generated binding, vector, or registry entry. A future batch must
re-prove a real consumer before using the conditional schema IDs.

## 10. Foundation result and next member audit

Foundation governance is bounded and decision-complete, but machine
registration remains NO-GO and exact registered assets remain `none`.

The next candidate is `status.inspect`. Its mandatory decisions remain, in
order: operation SemVer/classification; selector; result projection/version/
digest/high-watermark; read authority; capability/permission/risk;
existence-hiding; authoritative readback/verifier; complete error map; audit;
and envelope/epoch/compatibility. No later field is inferred from the private
implementation.

## 11. Preserved blockers and evidence boundary

- OPS/TARGET/SIG/AUDIT machine contracts remain unregistered.
- All eight OPS candidates remain blocked; `status.inspect` is not yet eligible.
- SIG independent security/cryptography review remains pending.
- Configuration Authority implementation is not provided; new OPS behavior is not executed.
- D-016 remains open; D-022 remains blocking; CA-1 through CA-8 remain blocked.
- Profile `implemented` remains `0`.
