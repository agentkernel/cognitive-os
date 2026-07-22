# V02-CA-TARGET-01 Configuration Target Authority Design Decision

- Decision ID: `V02-CA-TARGET-01`
- Date: 2026-07-22
- Status: **materialized for owner review; all three candidates blocked**
- Baseline: `origin/main@88d5374430263c52c7b67e3178dcd752ad984dbc` (PR #51 merge; main CI run `29915808901` success)
- OPS input: [V02-CA-OPS-01](V02-CA-OPS-DESIGN-DECISION.md) and [ADR-0010](../adr/0010-v02-management-operation-set-governance.md)
- Structural governance: [ADR-0011](../adr/0011-v02-configuration-target-authority-governance.md)
- Classification: docs-only structural design under [V02-CA-GOV-00](V02-CONFIGURATION-AUTHORITY-NORMATIVE-SURFACE-AUTHORIZATION.md) and [ADR-0009](../adr/0009-v02-configuration-authority-surface-expansion-governance.md)
- Target asset version/digest: `unresolved/not assigned`; no machine asset exists
- Operation-set digest: `unresolved/not computed`

## 1. Decision and status boundary

This decision audits and designs the target-authority obligations for exactly
three OPS intended critical-extension candidates:

1. `system.configure`
2. `gateway.configure`
3. `diagnostics.configure`

The source audit proves that the registered governed-object model can provide a
common governance skeleton, but no existing registered object uniquely defines
any of the three configuration targets. Every candidate still lacks mandatory
operation-specific facts, including at least target selection, payload,
consumer, readback/verifier, receipt, risk/approval, error, negotiation, or
audit closure. Therefore:

- all three candidates remain `blocked`;
- their intended critical-extension classification remains design intent only;
- there is no design-approved or machine-registered target profile;
- there is no design-approved or machine-registered extension member;
- there is no configuration object family or state domain registered here;
- there is no request/result schema, digest domain, consumer, verifier, receipt,
  or error mapping registered here;
- TARGET materialization does not make any operation reachable or authorized.

This decision registers no requirement, error, schema, state domain,
transition, vector, descriptor, operation set, extension, specification set,
generated binding, implementation, evidence artifact, or Profile claim. No new
behavior vector was executed. Existing vector `expected` values are unchanged.

## 2. Source-audit method and precedence

The audit applies the repository precedence and fail-closed interpretation:

1. registered schemas, registries, transition tables, and vectors;
2. pinned Core and normative standards;
3. OPS/GOV decisions and ADRs;
4. implementation-private code only as evidence of what exists, never as a
   source that creates a cross-boundary contract.

The following registered sources were inspected:

- `governed-object-header.schema.json` and `object-reference.schema.json`;
- `management-action-proposal.schema.json`;
- `intent.schema.json`, `effect.schema.json`, and
  `verification-report.schema.json`;
- `event.schema.json` and the append-only Event/audit standard;
- AKP request/result envelopes;
- authorization capability, privileged session, and management approval
  request/decision schemas;
- `state-domains.yaml`, registered errors, Core authority/CAS/Effect rules, and
  the operation/authorization separation standard;
- `MGMT-CONFIG-001`, `MGMT-FALLBACK-008`, and
  `SHELL-CHANNEL-ISOLATION-003` as exact vector facts, without promoting their
  examples or reachability expectations into target contracts.

The tracked implementation was searched for the three operation names and
potential consumers. The only exact implementation uses are management-name
channel classification and task-channel denial. The deterministic management
plane implements `status.inspect`, `execution.stop`, `capability.revoke`, and
`effect.reconcile`; it implements no configure consumer. No application,
runtime, store, or SDK path provides an authoritative system, gateway, or
diagnostics configuration target, apply consumer, readback API, or verifier.

## 3. Existing machine facts and reuse decision

### 3.1 Field-by-field governed-object audit

| Required target fact | Existing exact source | Reusable fact | Missing fact / decision |
|---|---|---|---|
| target identity | `GovernedObjectHeader.id`; `ObjectReference.strongReference` | UUIDv7 identity and strong `(id, object_version, content_digest)` pin | no registered system/gateway/diagnostics target type or target-selection rule |
| authority | `GovernedObjectHeader.authority_ref`; Core `REQ-AUTH-001`/`REQ-STATE-001` | strong authority reference plus unique-current-authority invariant | no registered authority object/profile or mapping from an operation target to its authority |
| version/CAS | header `object_version`; proposal/AKP expected versions; Core `REQ-STATE-003` | expected version can compare to the target's authoritative `object_version` | no target domain means the comparison object and request binding are not yet unique |
| writer epoch | Effect `fencing_token`; authority-store fencing behavior; recovery rules | stale writers must be fenced and checked at the commit sink | no mandatory configure request/receipt epoch field or target-specific epoch source is registered |
| payload domain | proposal `parameters_digest`; Intent/Effect parameter digest; canonical digest standard | canonical digest and idempotency mechanics are reusable | proposal `parameters` is open; no per-operation schema, projection, or versioned digest domain exists |
| consumer | none | no reusable machine fact | exact target consumer, accepted profile versions, apply protocol, and authority boundary are missing |
| readback/verifier | `VerificationReport` generic verifier/version/fixed-post-state shape | a later target verifier can produce a VerificationReport | no target readback projection, criteria, verifier identity/version, or freshness rule exists |
| receipt | Effect `receipt_ref`; AKP result/result_ref/observed_versions; Core receipt-as-evidence rule | a receipt remains evidence and cannot replace verification/commit | no authority receipt schema fixes target, previous/new version, epoch, or causal refs |
| audit atomicity | Event/state same-transaction rule; Core audit properties | state/Event atomicity and append-only behavior are reusable | authoritative audit carrier/profile/persistence port and receipt audit slot remain an AUDIT blocker |

### 3.2 Reuse result

The existing governed-object model is **reusable only as the outer governance
skeleton**. A future target profile must use stable identity, strong references,
explicit authority, scope, policy, sensitivity, retention, lineage, canonical
digest, and object version. Generic CAS, fencing, Intent/Effect/Verification,
reconciliation, Event, and fail-closed persistence rules remain applicable.

The model is **not sufficient as a complete target contract**. No existing
object body or state domain can be renamed or reinterpreted as one of the three
targets. The following are explicitly non-authoritative fillers:

- a URI or API/CLI route;
- `proposal.target_refs`, open `proposal.parameters`, or an AKP open payload;
- an Intent target string or open postcondition expression;
- an Effect `receipt_ref` or Event open payload;
- an SQLite row, `StoredObject.body`, private `CommitReceipt`, private
  `InspectReport`, or another implementation DTO;
- caller/plugin values, vector reachability, `OperationSummary`, or catalog
  projections.

### 3.3 Structural direction

[ADR-0011](../adr/0011-v02-configuration-target-authority-governance.md)
proposes the bounded direction:

- reuse the governed-object outer model;
- do not create a semantically empty generic target;
- permit a later, separately reviewed structural proposal for operation-specific
  target profiles or a general authority-managed configuration state domain;
- do not add a sixth execution lifecycle;
- register no structure until each operation's consumer and readback semantics
  are uniquely fixed.

This is a proposed structural direction, not an approved machine object family.

## 4. Per-operation source audit

### 4.1 `system.configure`

Existing facts:

- `MGMT-CONFIG-001` fixes an R1 example using the spelling
  `system.configure`, expected version 7, a stable idempotency key, an approval,
  the generic Effect flow, and a resulting version 8.
- `SHELL-CHANNEL-ISOLATION-003` fixes task-channel denial with
  `SHELL_CHANNEL_BINDING_MISMATCH`.
- the runtime management-name classifier recognizes the spelling only to keep
  it off the task channel.
- proposal/AKP/Intent shapes can carry open target, parameter, version, digest,
  session, and approval references.

Those facts do not establish a general system target. `MGMT-CONFIG-001` is one
scenario and cannot determine whether the target is a platform policy bundle,
network configuration, service-control configuration, or another governed
state object. It does not fix the payload schema, true consumer, readback,
postcondition, general risk classification, or approval policy.

Bounded alternatives for later owner/consumer review:

1. a platform policy/configuration object consumed by a deterministic platform
   configuration authority;
2. an explicitly scoped subsystem configuration object, with
   `system.configure` rejected as too broad unless a subtype is selected;
3. removal or renaming of this candidate if no unique cross-subsystem authority
   can be defined.

Decision: no alternative is selected here. A generic "system" authority is not
derived from `MGMT-CONFIG-001`; `system.configure` remains blocked.

### 4.2 `gateway.configure`

Existing facts:

- `MGMT-FALLBACK-008` fixes spelling and deterministic fallback reachability as
  an expected future behavior;
- the runtime management-name classifier recognizes the spelling for channel
  isolation;
- no registered gateway instance, gateway configuration authority, request
  body, result, consumer, readback, verifier, receipt, or target-specific error
  mapping exists;
- no tracked implementation consumes a `gateway.configure` request.

Bounded alternatives for later owner/consumer review:

1. one gateway-instance configuration object;
2. one gateway deployment/group configuration object with explicit fan-out and
   partial-apply semantics;
3. a routing/trust policy object consumed by gateways but owned by a separate
   policy authority.

These alternatives have different identity, CAS, fencing, consumer,
verification, rollout, and partial-apply semantics. Spelling and channel
classification cannot choose among them. `gateway.configure` remains blocked.

### 4.3 `diagnostics.configure`

Existing facts:

- `MGMT-FALLBACK-008` fixes spelling and deterministic fallback reachability as
  an expected future behavior;
- the runtime management-name classifier recognizes the spelling for channel
  isolation;
- no registered diagnostics target, payload, consumer, readback, verifier,
  receipt, or target-specific error mapping exists;
- telemetry, logging, traces, and implementation-private configuration do not
  constitute an authority contract.

Bounded alternatives for later owner/consumer review:

1. a diagnostics collection policy;
2. a diagnostic sink/export binding;
3. a collection profile fixing enabled signals, sampling, redaction, retention,
   and authorized export.

These alternatives differ materially in sensitivity, retention, export,
egress, credential, and partial-external-apply risk. They must not be collapsed
into an opaque telemetry configuration. `diagnostics.configure` remains
blocked.

## 5. Complete TARGET binding matrix

Every row is mandatory. “Reusable” means a later contract may reference the
existing fact; it does not mean the operation or target is registered.

| # | Binding | `system.configure` | `gateway.configure` | `diagnostics.configure` |
|---|---|---|---|---|
| 1 | target identity | unresolved among system/subsystem/policy alternatives; future strong target ref required | unresolved gateway instance/group/policy granularity; future strong target ref required | unresolved policy/sink/profile kind; future strong target ref required |
| 2 | authority source/ref | governed header strong authority ref reusable; actual system authority missing | governed header strong authority ref reusable; actual gateway authority missing | governed header strong authority ref reusable; actual diagnostics authority missing |
| 3 | target state/domain | no registered system configuration state/domain | no registered gateway configuration state/domain | no registered diagnostics configuration state/domain |
| 4 | expected version/CAS object | compare against selected target `object_version`; target not selected | compare against selected target `object_version`; target not selected | compare against selected target `object_version`; target not selected |
| 5 | writer epoch/fencing source | authority-store epoch mechanics reusable; configure binding missing | authority-store plus external gateway consumer fencing required; binding missing | authority-store plus any external sink/collector fencing required; binding missing |
| 6 | request parameters schema | absent; R1 vector example is not a schema | absent | absent; private telemetry config forbidden |
| 7 | request/result digest domain | absent; must be operation/profile-specific and versioned | absent; must bind gateway target profile and apply result | absent; must bind policy/sink/profile and sensitive result projection |
| 8 | real consumer | absent | absent | absent |
| 9 | readback/API or authority projection | absent | absent | absent |
| 10 | verifier identity/version | generic VerificationReport reusable; system verifier absent | generic VerificationReport reusable; gateway verifier absent | generic VerificationReport reusable; diagnostics verifier absent |
| 11 | authority receipt | absent | absent | absent |
| 12 | receipt target/new version/epoch | no machine carrier | no machine carrier | no machine carrier |
| 13 | Intent/Effect/Verification/Event/audit refs | generic process relations partially reusable; target binding and AUDIT slot missing | same; external apply causality additionally unresolved | same; export/sensitivity audit additionally unresolved |
| 14 | idempotency binding | proposal/Intent/Effect key+digest reusable; scope `(operation,target,profile,parameters,epoch)` unregistered | same; rollout/fan-out equivalence unresolved | same; sink/profile external apply equivalence unresolved |
| 15 | cancellation | pre-dispatch cancel must have zero effects; post-apply contract unresolved | rollout cancellation and partial apply unresolved | collector/sink cancellation and partial apply unresolved |
| 16 | unknown outcome | Effect `OUTCOME_UNKNOWN` reusable; system-specific query absent | reusable; gateway query/readback absent | reusable; diagnostics query/readback absent |
| 17 | reconciliation/quarantine | generic flow reusable; system reconciliation authority/query absent | generic flow reusable; per-instance/group reconciliation absent | generic flow reusable; sink/export reconciliation absent |
| 18 | risk class | no general mapping; R1 vector is one example only | no mapping; trust/egress/routing/blast-radius drivers unresolved | no mapping; sensitivity/retention/export/credential drivers unresolved |
| 19 | permission/capability constraints | must bind exact target/action/parameter bounds; no target profile | must bind instance/group/policy plus rollout bounds; absent | must bind signal/sink/export/retention bounds; absent |
| 20 | approval policy | tiered approval schemas reusable; payload-to-risk/approval rule absent | absent; multi-target/egress/trust changes need explicit policy | absent; sensitive collection/export/retention changes need explicit policy |
| 21 | management channel | management only; task channel denial explicitly fixed | management name classified; classification is not membership | management name classified; classification is not membership |
| 22 | critical-extension negotiation | intended critical only; exact extension ID/version/digest absent | intended critical only; exact extension ID/version/digest absent | intended critical only; exact extension ID/version/digest absent |
| 23 | stage-to-error mapping | channel/digest/CAS/Effect subsets exist; target/consumer/readback/receipt errors unresolved | same plus gateway apply/partial rollout errors unresolved | same plus sensitive export/sink/partial apply errors unresolved |
| 24 | audit responsibility slot | AUDIT contract pending | AUDIT contract pending; gateway apply facts required | AUDIT contract pending; sensitivity/retention/export facts required |
| 25 | migration/finite compatibility | new v0.2 target identity and epoch required; source mapping unresolved | new v0.2 target identity and epoch required; instance/group migration unresolved | new v0.2 target identity and epoch required; policy/sink/profile migration unresolved |
| 26 | authorization non-expansion proof | operation/descriptor/readback cannot create target write authority | gateway identity/extension selection cannot widen target/capability | diagnostics identity/readback/export discovery cannot widen authority |

## 6. Future request, readback, verifier, and receipt obligations

No field below is registered by this document. A later machine-registration
proposal cannot proceed unless it fixes, with exact asset identities and
digests, at least:

### Request

- operation and negotiated extension identity;
- strong target reference and strong target-authority reference;
- expected target version and writer epoch;
- request schema triple and exact parameters digest domain;
- proposal, session, approval, authorization, and capability bindings;
- stable idempotency key and declared equivalence scope;
- risk class, approval policy reference, deadline, and cancellation contract;
- Intent creation and Effect-class binding before dispatch.

### Readback and verifier

- an authority readback projection or API independent from the apply request;
- read authorization distinct from write authorization;
- exact projection schema/version/digest and freshness/high-watermark rules;
- deterministic verifier identity, version, criteria, and fixed post-state;
- consumer/profile identity so readback proves the intended consumer applied
  the intended target version rather than merely storing a proposal.

### Authority receipt

A future receipt must be a registered machine carrier and must include or
strongly bind:

- operation/extension identity;
- target strong reference, target authority reference, previous version, new
  version, and writer epoch;
- request/result schema triples and parameter/result digests;
- idempotency key and committed Effect reference;
- readback projection and VerificationReport references;
- committed Event and authoritative audit references/sequence slot;
- decision (`committed`, `aborted`, `outcome_unknown`, or `quarantined`) with
  no success form before authority commit.

An executor receipt remains execution evidence. It cannot be this authority
receipt unless a future registered contract proves all required facts.

## 7. Risk, permission, and approval decision

The registered R0-R3 approval carriers are reusable, but no current source
uniquely assigns a general risk class to any configure operation. A later target
profile must classify risk deterministically from target scope and
machine-validated parameters before approval:

- `system.configure`: blast radius, authority/policy changes, connectivity,
  persistence, rollback, and safety-control impact;
- `gateway.configure`: routing, trust roots, credentials, egress, protocol
  mapping, tenant reach, deployment fan-out, and rollback/partial apply;
- `diagnostics.configure`: signal sensitivity, collection volume, redaction,
  retention, destination, authorized export, credentials, and cross-tenant or
  cross-boundary egress.

Unknown or indeterminate risk fails closed. The operation name, an R1 example,
or a session risk ceiling cannot supply the action's risk. The ceiling is only
an upper authorization bound. Approval must be bound to the exact proposal,
target version, parameter digest, policy version, and challenge. R2/R3 continue
to require their registered trusted/independent surfaces; SIG closure remains
separate.

## 8. Authorization non-expansion proof

Effective authorization remains the intersection:

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

The proof obligations are:

- `OperationDescriptor` and `AuthorizationCapability` remain separate types and
  separate checks;
- a reserved management name is not operation-set membership;
- membership is not target write authority;
- target identity, target descriptor, extension selection, API/CLI route,
  catalog discovery, or successful resolution cannot widen session scope or
  capability bounds;
- readback authorization does not imply write authorization, and write
  authorization does not imply export authorization;
- a weak reference must resolve under current governance and be pinned as a
  strong reference before mutation or Effect creation;
- extension selection triggers full authorization revalidation before payload
  dispatch;
- task-channel use of a reserved management name may fail first with
  `SHELL_CHANNEL_BINDING_MISMATCH` without establishing membership;
- any missing, stale, unknown, mismatched, or indeterminate term denies before
  dispatch, Effect creation, business mutation, commit, or success receipt.

## 9. Management channel and critical negotiation

All three candidates are management-channel-only design intents. The existing
runtime classifier is an isolation mechanism, not a dispatcher or operation
registry. Task-channel credentials remain unable to invoke them.

Future registration must bind each candidate to a globally namespaced,
versioned, digest-pinned critical extension and an operation descriptor. The
new negotiation epoch must pin specification set, operation set, extension,
descriptor, request/result schemas, and target-profile identities. An old epoch
cannot gain a configure operation. A gateway cannot strip or remap the critical
extension, target authority, approval, audit, or readback semantics. After
extension selection, authorization is revalidated against the exact target and
parameters.

No extension ID, version, criticality flag, descriptor, target-profile asset,
or digest is registered here.

## 10. Stage and registered-error responsibility

OPS G0-G6 stages are retained:

- G0: framing/channel/peer;
- G1: epoch/specification set/extensions;
- G2: operation set/descriptor/schema identity;
- G3: request contract;
- G4: session/capability/risk/permission/target;
- G5: pre-dispatch/consumer apply;
- G6: result/readback/verify/reconcile/audit/commit.

TARGET responsibilities are placed within those stages:

- T0/G4: target identity, authority, current version, and epoch;
- T1/G2-G3: target/request/result schemas and digest domains;
- T2/G4-G5: consumer/profile and readback/verifier availability;
- T3/G5-G6: apply, cancellation, unknown outcome, and reconciliation;
- T4/G6: authority receipt, Event/audit linkage, and atomic commit.

| Failure | Stage | Existing registered code usable only when exact | Responsibility / unresolved fact |
|---|---|---|---|
| task/management channel mismatch | G0 | `SHELL_CHANNEL_BINDING_MISMATCH` | exact for mixed task/management credentials |
| unknown critical extension | G1 | `CRITICAL_EXTENSION_UNKNOWN` | exact before payload processing |
| gateway strips required critical semantics | G1 | `PROTOCOL_MAPPING_INCOMPLETE` | exact only for lossy protocol mapping |
| unsupported specification major/window | G1 | `VERSION_UNSUPPORTED` | exact where the finite window is violated; generic old-epoch code remains unresolved |
| schema-bundle/request schema digest mismatch | G1/G2 | `PROTOCOL_SCHEMA_DIGEST_MISMATCH` | exact for payload schema pin mismatch |
| set/descriptor/content digest mismatch | G2/G3 | `DIGEST_MISMATCH` | exact when the selected contract defines that digest |
| schema-invalid request/result | G3/G6 | `SCHEMA_MISMATCH` | exact after future schemas are registered |
| stale expected target version | G4 | `STATE_CONFLICT` | exact for expected-version mismatch |
| idempotency key reused with different parameters | G3/G4 | `EFFECT_IDEMPOTENCY_CONFLICT` | exact |
| session domain/action/resource outside scope | G4 | `MANAGEMENT_SCOPE_MISMATCH` | exact; target existence must not leak |
| step-up or independent approval missing | G4 | `MANAGEMENT_STEP_UP_REQUIRED` / `MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED` | exact where their registered conditions apply |
| external apply may have occurred | G6 | `EFFECT_OUTCOME_UNKNOWN` | exact; reconcile or quarantine, never blind retry |
| recovery cannot safely close outcome | G6 | `EFFECT_RECOVERY_QUARANTINED` | exact |
| authority state/Event commit unavailable | G6 | `STATE_STORE_UNAVAILABLE` | exact for authoritative persistence failure; AUDIT-specific atomic failure remains pending |
| unknown target or non-authority target | G4 | none proven exact | new target contract must assign responsibility; do not reuse a nearby code |
| target authority mismatch | G4 | none proven exact | authorization/target-profile owner |
| missing or wrong writer epoch | G4/G5 | none proven exact | current private paths use conflict semantics, but registered `STATE_CONFLICT` is version-specific |
| consumer missing/profile mismatch | G4/G5 | none proven exact | descriptor/target registration blocker |
| readback/verifier/receipt missing | G4/G6 | none proven exact | descriptor/target registration blocker |
| risk class indeterminate or ceiling insufficient | G4 | no complete general mapping | policy/security owner; do not collapse into a generic denial without exact meaning |
| audit carrier/write/commit failure | G6 | partially `STATE_STORE_UNAVAILABLE` | exact AUDIT carrier/atomic mapping remains an AUDIT responsibility |

Unknown operation, unnegotiated operation, generic old epoch, target identity,
authority mismatch, writer-epoch mismatch, consumer/profile mismatch,
readback/verifier absence, receipt incompleteness, and target-specific
partial-apply failures do not have proven complete registered error closure.
They remain unresolved responsibilities; this decision does not repurpose a
semantically adjacent code.

## 11. Fail-closed negative plan

All cases are **planned/not executed**. Future vectors must be new assets; no
existing vector `expected` may be changed.

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
| 1 | unknown target | T0/G4 | deny without existence leak; exact error unresolved | planned/not executed |
| 2 | target ref exists but is not authority-managed | T0/G4 | common zero-side-effect oracle; exact error unresolved | planned/not executed |
| 3 | target authority mismatch | T0/G4 | common oracle; exact error unresolved | planned/not executed |
| 4 | expected version missing | T1/G3 | future request schema rejection; `SCHEMA_MISMATCH` only after registration | planned/not executed |
| 5 | stale expected version | T0/G4 | `STATE_CONFLICT`; no dispatch | planned/not executed |
| 6 | wrong writer epoch | T0/G4 | fence before dispatch/commit; exact error unresolved | planned/not executed |
| 7 | parameters schema pin missing | T1/G2-G3 | reject before parameter interpretation | planned/not executed |
| 8 | parameters digest/domain mismatch | T1/G3 | `DIGEST_MISMATCH` where exact | planned/not executed |
| 9 | consumer missing | T2/G4-G5 | descriptor/target not selectable; exact error unresolved | planned/not executed |
| 10 | consumer and target profile mismatch | T2/G4-G5 | no apply; exact error unresolved | planned/not executed |
| 11 | readback missing | T2/G4 | no dispatch because postcondition cannot close | planned/not executed |
| 12 | verifier identity/version missing | T2/G4 | no dispatch; descriptor registration blocker | planned/not executed |
| 13 | postcondition cannot be proven | T3/G6 | no commit or success receipt; reconcile/quarantine | planned/not executed |
| 14 | receipt lacks target/new version/epoch | T4/G6 | reject success; exact receipt error unresolved | planned/not executed |
| 15 | receipt lacks Effect/Verification/Event/audit refs | T4/G6 | reject success; AUDIT responsibility unresolved | planned/not executed |
| 16 | target scope exceeds session | G4 | `MANAGEMENT_SCOPE_MISMATCH`; common oracle | planned/not executed |
| 17 | capability does not cover target | G4 | condition-specific authorization denial; common oracle | planned/not executed |
| 18 | risk ceiling insufficient | G4 | deny/challenge; exact general code mapping unresolved | planned/not executed |
| 19 | approval policy insufficient | G4 | registered step-up/independent code where exact | planned/not executed |
| 20 | task channel invokes configure | G0 | `SHELL_CHANNEL_BINDING_MISMATCH`; common oracle | planned/not executed |
| 21 | extension not negotiated | G1/G2 | reject before payload; exact operation error unresolved | planned/not executed |
| 22 | old epoch requests configure | G1 | reject before payload; epoch-specific error unresolved | planned/not executed |
| 23 | gateway strips critical extension | G1 | `PROTOCOL_MAPPING_INCOMPLETE`; common oracle | planned/not executed |
| 24 | external apply occurred and result is unknown | T3/G6 | `EFFECT_OUTCOME_UNKNOWN`; original key reconcile/quarantine | planned/not executed |
| 25 | audit write or authority commit fails | T4/G6 | no success; rollback local visibility or unknown/reconcile after external apply | planned/not executed |
| 26 | dispatch occurs before rejection | G5 | test fails if `dispatches != 0` | planned/not executed |
| 27 | Effect exists before rejection | G5 | test fails if `effects_created != 0` | planned/not executed |
| 28 | state mutates before rejection | G5 | test fails if business mutation occurs | planned/not executed |
| 29 | commit occurs before rejection | G5/G6 | test fails if `commits != 0` | planned/not executed |
| 30 | success receipt appears after rejection | G6 | test fails if `success_receipts != 0` | planned/not executed |

Operation-specific future negatives must additionally cover:

- system target ambiguity and an R1 example incorrectly generalized to a
  platform-wide authority;
- gateway instance/group ambiguity, partial fan-out, trust/route loss, and
  target-version/readback disagreement;
- diagnostics policy/sink/profile ambiguity, sensitive signal enablement,
  retention expansion, unauthorized export, redaction loss, and partial sink
  apply.

## 12. Compatibility and migration impact

TARGET does not change the proposed OPS compatibility window or publish a
target asset. It adds mandatory closure conditions:

- a blocked candidate is absent from any machine operation/extension manifest;
- future v0.2 target profiles receive new asset IDs, complete SemVer, digests,
  and an explicit finite compatibility window;
- v0.1 URI targets, open proposal parameters, vector inputs, private rows/DTOs,
  or implementation configuration files are migration inputs only;
- migration creates a new target identity/version and authority decision; it
  never upgrades an old URI or private row in place;
- missing target authority, consumer, readback, verifier, risk/approval, audit,
  or lossless mapping causes reject or quarantine, never a default to platform,
  current tenant, current process, or public scope;
- a new negotiation epoch pins target profiles and revalidates authorization;
- in-flight Effects retain their original epoch, target, idempotency, fencing,
  unknown-outcome, and reconciliation obligations;
- removal or semantic change of a target profile, authority mapping, consumer,
  readback, verifier, receipt, risk, approval, audit, or error binding is
  breaking and requires a new version/digest and migration note.

No adapter, target profile, state domain, consumer, readback, verifier, receipt,
or migration implementation exists as a result of this decision.

## 13. Owner, reviewers, blockers, and evidence limitation

### Owner/reviewer roles

- decision owner: repository owner;
- structural reviewer: governed-object/state authority maintainer;
- per-operation reviewer: the named real consumer owner, once one exists;
- security reviewer: capability/risk/approval and channel/negotiation boundary;
- AUDIT reviewer: authority receipt and atomic audit responsibility;
- SIG reviewer: signature bindings used by session/approval, separately.

No TARGET owner approval or GitHub review is claimed by this authoring batch.
PR #51's single-use owner exception does not apply to TARGET.

### Blocking facts

- unique system target selection;
- unique gateway target granularity and consumer;
- unique diagnostics target kind and consumer;
- per-operation payload/result schemas and digest domains;
- authority mapping, target state/domain, request/receipt epoch binding;
- readback projection and verifier profile;
- risk/approval/capability parameter mapping;
- complete stage/error closure;
- authoritative audit carrier/slot and atomic failure semantics;
- critical extension and operation descriptor machine registration;
- SIG and AUDIT downstream designs where referenced.

### Evidence limitation

Evidence in this batch is a static audit of registered assets and tracked
implementation facts. Static consistency checks, builds, and ordinary unit
tests validate repository integrity only. They do not execute a new behavior
vector, prove a target consumer, register a contract, demonstrate runtime
authorization, or support a Profile claim.

## 14. GO/NO-GO and downstream order

### Design-materialization result

- `GO`: TARGET docs-only decision and proposed structural ADR are complete
  enough for owner review.
- `NO-GO`: target machine registration, operation membership, implementation,
  behavior execution, and Profile claim.

### Per-candidate gate

Each candidate remains blocked until all 26 matrix rows close, the selected
target structure passes independent owner review, exact machine assets are
registered in a later PR, new negative vectors are added without modifying old
`expected`, and TARGET/OPS/SIG/AUDIT bindings are all mutually consistent.

### Downstream order

1. owner review and merge of this TARGET design packet;
2. SIG design;
3. AUDIT design;
4. four independent machine-registration batches, including TARGET assets;
5. independent CA-0 re-review;
6. explicit CA-0 GO;
7. only then, implementation and Management CFR.

TARGET merge alone does not approve an object family, register an operation,
unblock CA-1 through CA-8, or authorize implementation.

## 15. Preserved state

- v0.1 assets and identities unchanged;
- 273 requirements, 55 errors, 61 schemas, 84 vectors;
- 59 pass, 25 not-run, self-check 40;
- matrix non-empty implementation count 70;
- Profile implemented = 0;
- D-016 open;
- D-022 blocking;
- CA-1 through CA-8 blocked;
- all three configure candidates blocked;
- machine contracts unregistered;
- implementation not provided;
- new behavior not executed.
