# V02-CA-AUDIT-01 Authoritative Audit Design Decision

- Decision ID: `V02-CA-AUDIT-01`
- Date: 2026-07-22
- Status: **merged; owner-authorized security/audit/compliance review completed with limited provenance; machine registration pending**
- Baseline: `origin/main@0a30ac70769f0501f7928d96f55f17636eaa9888`
  (PR #53 merge; main CI run `29930557168` Ubuntu/Windows success)
- Inputs: [V02-CA-GOV-00](V02-CONFIGURATION-AUTHORITY-NORMATIVE-SURFACE-AUTHORIZATION.md),
  [V02-CA-OPS-01](V02-CA-OPS-DESIGN-DECISION.md),
  [V02-CA-TARGET-01](V02-CA-TARGET-DESIGN-DECISION.md), and
  [V02-CA-SIG-01](V02-CA-SIG-DESIGN-DECISION.md)
- Structural governance:
  [ADR-0013](../adr/0013-v02-authoritative-audit-governance.md)
- Classification: docs-only structural design; no machine registration
- Proposed family version: `0.2.0-draft.1`; every digest is
  `unresolved/not computed`

## 1. Decision and status boundary

This decision designs the Configuration Authority authoritative-audit contract.
It does not register an audit schema, profile, stream, checkpoint, export
manifest, retention/redaction policy, signature profile, key usage, error,
critical extension, operation, state domain, transition, vector, generated
binding, implementation, evidence artifact, or Profile claim.

The repository owner confirmed the bounded technical selections in sections
5-12 on 2026-07-22:

1. an existing `Event` is the required cross-boundary outer envelope, but only
   a future closed `AuthoritativeAuditRecord` payload/profile is the
   authoritative audit carrier;
2. platform audit and tenant audit are separate streams; tenant streams are
   partitioned by tenant, management domain, and audit-profile digest;
3. each stream has one current fenced sequence authority, contiguous logical
   sequence, a previous-record digest chain, and signed periodic checkpoints;
4. checkpoint and export signatures use distinct dedicated usages and the
   governed key registry; external WORM anchoring is optional hardening and a
   Merkle profile is deferred;
5. retention floors come from an exact digest-pinned policy, not invented
   calendar values; legal-hold release requires a different same-or-higher
   compliance authority;
6. records are minimized at creation; redaction is a registered deterministic
   derived-view operation and never edits authoritative bytes;
7. exports use ordered canonical JSON records and a signed canonical manifest,
   bind the source checkpoint/high-watermark, and are themselves audited.

These are owner-confirmed docs-only selections, not registered contracts or an
independent review. Exact schema bytes, asset digests, policy values, checkpoint
thresholds, key descriptors, and all machine bindings remain pending. The audit
profile is unavailable until those bindings close in a separate registration
batch.

PR #53's merge does not establish independent SIG security review. OPS, TARGET,
SIG, and this AUDIT design do not authorize implementation. D-016 remains open,
D-022 remains blocking, CA-1 through CA-8 remain blocked, and Profile
implemented remains zero.

## 2. Source audit and precedence

The audit applied the repository precedence and fail-closed rule:

1. registered schemas, registries, transitions, and vectors;
2. pinned Core and normative standards;
3. owner-approved GOV and merged OPS/TARGET/SIG design inputs;
4. tracked implementation only as implementation-private evidence.

### 2.1 REQ-AUDIT source status

`REQ-AUDIT-001` and `REQ-AUDIT-002` are registered as `specified` and point to
the Core companion. They require traceability of every committed governed Effect
and protection against ordinary-Agent tamper, with ordering integrity,
retention, sensitivity controls, and export audit. Their only registered vector
mapping is static `SPEC-CONTRACT-COVERAGE-001`. In
`docs/traceability/matrix.yaml`, both entries have `impl: []`, `impl_tests: []`,
and `evidence: []`.

The registered requirements are behavior authority. They do not themselves fix
a machine carrier, stream topology, record schema, digest projection,
checkpoint, retention duration, export format, or atomic persistence port.

### 2.2 Machine-asset facts

- There is no `*audit*.schema.json`, `*receipt*.schema.json`, audit transition
  table, or registered authoritative-audit profile.
- `event.schema.json` has a governed header, correlation/causation, payload
  schema digest, and open `payload|payload_ref`. It has no closed audit fields,
  stream sequence, previous-record digest, checkpoint, retention/export
  profile, or atomic audit slot.
- `state-transition-record.schema.json` fixes a committed transition's request,
  actor, authority, reason, evidence, before/after versions, table pin, and
  optional Event ref. Its `metadata` is open and cannot add audit authority.
- `akp-result-envelope.audit_ref` is an optional URI reference. It is not a
  strong reference, commit receipt, continuity proof, or authorization fact.
- `Effect.receipt_ref` is execution evidence. `VerificationReport` fixes a
  verifier result and post-state. Neither is an audit record.
- `GovernedObjectHeader` supplies identity/version, strong governance refs,
  sensitivity, compartments, retention, provenance, lineage, and content
  digest. It is reusable as the Event outer governance skeleton, but does not
  define an audit payload or audit behavior.
- `common-defs.schema.json` has no `audit` error category.
- All 55 registered errors were reviewed in section 13. None is a general
  audit-integrity or export error.

### 2.3 Vector facts

Existing vectors contain facts such as `audit_required: true`,
`audit_chain_closed: true`, replay to a high-watermark, management denial,
state-store failure, crash reconciliation, and watch gap rejection. They do not
define an audit record, stream, chain algorithm, checkpoint signature,
retention/redaction policy, export manifest, or persistence port. Watch stream
sequence and `WATCH_CURSOR_STALE` are watch contracts, not audit-stream
authority. No existing vector `expected` is changed by this decision.

### 2.4 Implementation-private facts

- SQLite commits object CAS, Event, transition record, budget debit, and outbox
  rows in one transaction. `events.sequence` and `record_seq` are internal
  autoincrement values; update/delete triggers make those two tables append-only.
- Replay consumes internal Event sequence, detects non-increasing sequence and
  per-object version gaps, and returns an implementation projection
  high-watermark. This is not a registered audit-stream algorithm.
- `DenialAudit` is an in-memory server-side DTO containing stage, code,
  principal, action, and purpose. No authoritative persistence port consumes it.
- `ReadinessFacts.audit_available`, adapter `audited: true`, log text, and other
  booleans are assertions, not persisted audit proof.
- The runtime Event assembler publishes a full Event envelope from a committed
  internal Event and a supplied governed header. Its payload remains open and
  it creates no audit profile.

SQLite triggers, rows, transaction IDs, autoincrement values, wall-clock time,
UUID order, private DTOs, booleans, logs, telemetry, and traces cannot define or
prove this cross-boundary contract.

## 3. Evidence-type boundaries

| Artifact | What it can prove after its own contract validates | What it never proves by existence alone |
|---|---|---|
| Event | one immutable governed observation envelope and payload-schema pin | authoritative audit payload, complete stream order, authorization, Effect closure, or commit |
| state-transition record | one committed legal transition and before/after version | denied attempts, complete management decision, audit continuity, retention, or export |
| `SignatureVerificationReceipt` | SIG checks performed for an exact signed subject/profile | current business authorization, audit persistence, state commit, or completion |
| Effect/external receipt | observed execution evidence for an Intent/idempotency binding | acceptance, Verification, audit closure, or authority commit |
| VerificationReport | verifier result for fixed criteria and post-state | dispatch history, authorization, audit continuity, or commit |
| authoritative audit record | one minimized authority fact at one stream sequence | correctness of every referenced artifact or authorization by itself |
| audit stream/checkpoint | ordered continuity and a signed high-watermark for one stream | correctness of record semantics or another stream's completeness |
| audit export | an authorized, bounded projection of source records through a fixed high-watermark | original business authorization, Effect completion, or Profile conformance |
| telemetry/trace/log | operational observation, if retained | authoritative state, audit record, ordering integrity, or completion |

No row in this table substitutes for another.

## 4. Proposed profile family and identities

Future machine registration may propose these independent assets, all initially
at exact `0.2.0-draft.1` with independently computed digests:

| Proposed asset | Responsibility | Current status |
|---|---|---|
| `cognitiveos.audit.configuration-authority-record/0.2` | closed `AuthoritativeAuditRecord` payload/profile | owner-confirmed design; unregistered |
| `cognitiveos.audit.configuration-authority-stream/0.2` | stream identity, writer, sequence, chain, high-watermark | owner-confirmed design; unregistered |
| `cognitiveos.audit.configuration-authority-checkpoint/0.2` | sealed checkpoint and checkpoint signature | owner-confirmed design; unregistered |
| `cognitiveos.audit.configuration-authority-retention/0.2` | retention floor, legal hold, compaction eligibility | owner-confirmed model; policy values unresolved |
| `cognitiveos.audit.configuration-authority-redaction/0.2` | deterministic query/export derived projection | owner-confirmed design; unregistered |
| `cognitiveos.audit.configuration-authority-export/0.2` | export manifest, source range, ordering, signing | owner-confirmed design; unregistered |
| `cognitiveos.audit.configuration-authority-commit-receipt/0.2` | post-commit strong audit/authority receipt | owner-confirmed design; unregistered |

The proposed critical extension is
`cognitiveos.ext.audit.configuration-authority`. A future registration must pin
the complete family, record/checkpoint/export schemas, policies, key usages, and
error mapping in one new negotiation epoch. This document does not register the
extension.

## 5. Carrier and governed-envelope decision

### 5.1 Required relationship

Every authoritative audit fact is a future closed
`AuthoritativeAuditRecord`. Every cross-boundary representation is an existing
Event envelope whose:

- `header.type` remains `Event`;
- governed header supplies envelope identity/version, authority, tenant/platform
  scope, sensitivity, compartments, retention, provenance, and lineage;
- `schema_digest` pins the exact audit-record payload schema;
- `payload` is accepted only through the selected closed audit-record profile;
- `subject` and correlation/causation remain envelope routing facts; the payload
  carries the strong subject bindings;
- envelope content digest and payload record digest are independently
  recomputed under their registered domains.

An Event with another payload profile is not an audit record. An audit payload
without its committed Event and stream slot is incomplete. Reusing the Event
and GovernedObjectHeader skeleton does not silently modify either v0.1 schema.

### 5.2 Record taxonomy

The future closed taxonomy contains at least:

- admission, authentication, authorization, signature-verification, approval,
  target-resolution, and denial decisions;
- pre-dispatch authorization, dispatch attempt, observed outcome,
  outcome-unknown, reconciliation, Verification, authority commit, authority
  abort, and quarantine;
- recovery barrier/resume, correction/supersession, retention/legal-hold,
  redaction/query access, export decision/completion/failure, and chain closure.

Taxonomy values are not operation membership. In particular, this design does
not add an `audit.export` operation.

## 6. Complete record binding matrix

Every row is mandatory unless a future closed record-kind schema declares it
inapplicable. Optional-by-caller is forbidden.

| # | Binding | Owner-confirmed obligation |
|---|---|---|
| 1 | record profile | exact asset ID, complete SemVer, digest |
| 2 | record schema | exact schema asset ID/version/digest |
| 3 | record identity | stable record ID, record version, record kind |
| 4 | record digest | recomputed canonical record digest |
| 5 | Event envelope | strong Event ref plus envelope schema/content digest |
| 6 | stream profile | exact stream-profile ID/version/digest |
| 7 | stream identity | canonical scope tuple; never caller URI |
| 8 | stream position | positive contiguous sequence and previous-record digest |
| 9 | writer authority | strong authority ref, writer epoch, fencing token |
| 10 | scope | platform or tenant; tenant ID required only for tenant scope |
| 11 | management domain | exact domain bound to the stream |
| 12 | sensitivity | monotone maximum of record and bound facts |
| 13 | compartments | union of required compartments; no caller removal |
| 14 | retention | policy strong ref/digest, computed floor, earliest review time |
| 15 | legal hold | hold state plus setting/releasing authority record refs |
| 16 | actor | initiating/effective/workload/device principal strong refs as applicable |
| 17 | ActorChain | strong ref, object version, chain digest |
| 18 | deciding authority | strong ref/version/digest and decision role |
| 19 | decision | allow/deny/challenge/commit/abort/quarantine or kind-specific result |
| 20 | stage | earliest failed or successfully completed deterministic stage |
| 21 | registered error | exact code/category/retryability when applicable |
| 22 | safe reason | registered reason code; no protected free text |
| 23 | operation | operation-set/descriptor triple when applicable |
| 24 | target | target-profile triple and subject strong refs |
| 25 | parameters | schema triple plus canonical parameters digest, never secret body |
| 26 | session | exact session ID/version/digest strong binding |
| 27 | capability | applicable capability strong refs/version/revocation epoch |
| 28 | approval | request/decision strong refs, tier, challenge and consumption set |
| 29 | signature receipt | exact `SignatureVerificationReceipt` strong ref/digest |
| 30 | proposal | proposal strong ref/digest and policy/risk facts |
| 31 | correlation | correlation ID and causation ID/ref |
| 32 | idempotency | stable key digest/binding; secret key value may be tokenized |
| 33 | fencing | writer/executor epoch and token binding where applicable |
| 34 | before state | subject strong ref, state, object version, content digest |
| 35 | after state | subject strong ref, state, object version, content digest |
| 36 | transition | strong transition-record ref/digest where committed |
| 37 | Intent | strong Intent ref/digest before external dispatch |
| 38 | Effect | strong Effect ref/version/digest and lifecycle state |
| 39 | dispatch | attempt number, executor profile, dispatch fact ref |
| 40 | external receipt | strong or registered evidence ref/digest, never acceptance |
| 41 | reconciliation | result, report strong ref/digest, outcome certainty |
| 42 | Verification | report strong ref/version/digest and fixed-post-state ref |
| 43 | authority commit | CAS winner, committed versions, Event/transition/audit refs |
| 44 | negotiation | epoch ID/digest, specification/operation/audit-profile pins |
| 45 | critical extension | exact extension ID/version/digest and preservation result |
| 46 | correction | superseded record strong ref and correction authority |
| 47 | timestamps | observed/committed times as evidence only, never ordering authority |
| 48 | minimization | content class and deterministic redaction/reference rule applied |

Unknown fields fail schema validation. A record kind cannot omit a required
binding by moving it into Event open payload, transition metadata, a URI, log
text, or private storage columns.

## 7. Canonical domains, projections, and exclusions

Owner-confirmed proposed domains are:

- `authoritative-audit-stream/0.2` for canonical stream identity;
- `authoritative-audit-record-content/0.2` for a record;
- `authoritative-audit-checkpoint-content/0.2` for a checkpoint;
- `authoritative-audit-checkpoint-signature/0.2` for its signature input;
- `authoritative-audit-redacted-view/0.2` for a derived view;
- `authoritative-audit-export-manifest/0.2` for a logical export manifest;
- `authoritative-audit-export-signature/0.2` for manifest signature input.

All use RFC 8785 and the registered canonical standard. Future schemas must
declare exact JSON Pointer exclusions. The initial design permits only each
object's explicitly named self-digest and detached-signature bytes to be
excluded from that object's digest projection. Stream sequence,
previous-record digest, profile/schema/negotiation pins, subject bindings,
sensitivity, compartments, retention/legal-hold facts, and correction refs are
never excluded.

Checkpoint and export signatures use pure strict RFC 8032 `Ed25519` with the
same strict encoding rules selected by SIG, but distinct single-usage leaf keys
named `audit-checkpoint-signing` and `audit-export-signing`. They resolve through
the governed authority-key registry and platform-rooted manifest. The SIG
platform root remains certification-only and never signs checkpoints or
exports. These new usages/profiles remain AUDIT machine-registration work and
require independent security review.

Wrong domain, projection, exclusion set, schema digest, canonical bytes, or
signature fails before query/export success, recovery progress, business
commit, or success receipt.

## 8. Stream topology, sequence, and integrity

### 8.1 Stream identity and partition

The canonical stream tuple is:

```text
(scope_domain, tenant_id-or-null, management_domain, audit_profile_digest)
```

Platform records use a dedicated platform stream and contain no tenant body.
Tenant records use a tenant stream. Cross-tenant stream injection is forbidden.
A platform governance decision that affects tenants records only minimized
strong refs/digests in the platform stream and emits tenant-scoped records in
each affected tenant stream when tenant-visible facts are required. A global
mixed-tenant stream and per-object streams are not selected.

### 8.2 Sequence authority

Each stream has exactly one current sequence authority or an explicit future
consensus implementation conforming to the same single-winner contract. It
assigns contiguous positive sequence numbers beginning at one. The stream
authority state includes current sequence, current record digest, writer epoch,
and fencing token. Assignment is a CAS against the expected high-watermark.

Wall-clock time, UUID lexical order, database row ID, SQLite autoincrement, and
outbox order never define cross-node or cross-implementation audit order.

### 8.3 Record chain and failure behavior

- sequence one has an explicitly registered genesis value;
- every later record includes the immediately preceding record digest;
- duplicate, gap, regression, stale append, fork, or previous-digest mismatch
  is an integrity failure;
- ordinary Agents/workloads cannot write, update, delete, truncate, reorder, or
  repair the stream;
- correction appends a new correction record with a strong supersession ref;
  original bytes remain immutable;
- any integrity ambiguity enters recovery barrier/quarantine and cannot advance
  beyond the last verified checkpoint/high-watermark.

### 8.4 Checkpoints

A digest-pinned checkpoint policy fixes positive finite
`max_records_between_checkpoints` and `max_duration_between_checkpoints` values.
A checkpoint is required at the first reached bound and additionally on writer
epoch change, key rotation, negotiation-epoch termination, export boundary,
recovery handoff, and orderly stream sealing.

The checkpoint binds stream/profile identities, sequence range, record count,
genesis/predecessor link, high-watermark sequence and record digest, writer
epoch, checkpoint policy, previous checkpoint strong ref/digest, creation
authority, and signature profile/key. It is signed by the dedicated checkpoint
key. Signature verification and chain verification are both required.

Merkle segments are not selected for the initial profile. External immutable or
WORM anchoring is permitted as defense in depth but is not initial conformance
proof. Either change requires a new profile version/digest and migration.

## 9. Persistence responsibility and atomicity

### 9.1 Future authoritative port

Future machine registration must define an `AuthoritativeAuditPort` or
equivalent closed port with separate operations for:

- denial commit;
- governed business commit with an atomic audit slot;
- external-Effect stage append;
- checkpoint/seal;
- authorized query/export and export-self-audit.

An `AuditCommitReceipt` is returned only after durable commit and strongly binds
record ID/version/digest, stream ID, sequence, previous digest, resulting
high-watermark, Event ref/digest, and any joined state/transition/Effect refs.
The receipt grants no authorization and cannot replace the committed record.

### 9.2 Denial

Authorization, admission, signature, approval, target, negotiation, and
pre-authority failures persist a safe minimized denial record before returning a
result on which the caller may rely. If denial-audit persistence fails, the
system returns an audit persistence failure; it never reports business success.

The denial oracle is:

```text
denial_audit_commits = 1
dispatches = 0
effects_created = 0
business_state_mutations = 0
authority_business_commits = 0
success_receipts = 0
```

The audit commit is intentionally not hidden behind an ambiguous `commits = 0`.

### 9.3 Successful governed commit

For a successful authoritative state change, one authority transaction must
commit all applicable elements or none:

```text
object/state CAS
+ transition record
+ Event
+ required SignatureVerificationReceipt handoff
+ authoritative audit record and stream CAS
+ outbox seed
+ success-result/commit receipt visibility
```

The success result becomes visible only after commit. State without audit,
audit success without authority state, a premature success receipt, or an Event,
transition, outbox, or receipt standing in for the audit slot is forbidden.

### 9.4 Audit-store failure

If the audit slot cannot persist, the joined authority transaction rolls back.
No state, transition, Event, outbox, approval consumption, or success receipt is
visible. `STATE_STORE_UNAVAILABLE` remains exact only for its registered
state/Event persistence condition. A future `AUDIT_STORE_UNAVAILABLE` owns an
audit-port-specific failure; no existing code is broadened by this document.

## 10. External Effect and recoverable closure

An external side effect cannot share a transaction with local audit storage.
The authoritative chain therefore records, in order:

1. persisted Intent and pre-dispatch authorization/audit fact;
2. dispatch attempt with original idempotency and fencing binding;
3. observed outcome, or explicit `OUTCOME_UNKNOWN`;
4. reconciliation attempt/result;
5. Verification against a fixed post-state;
6. authority commit, abort, compensation decision, or quarantine;
7. final chain-closure record.

The pre-dispatch record is durable before the executor is invoked. A crash after
dispatch but before an outcome record produces `OUTCOME_UNKNOWN`, never a blind
retry or success. Recovery installs the new fencing epoch, verifies the last
signed checkpoint and chain to the high-watermark, replays only committed
records, reconciles pending Effects with the original idempotency key, rechecks
SIG/session/approval/capability/target authority, and only then continues.

Crash, retry, reconnect, or replay cannot create a duplicate Effect, consume an
approval twice, skip an audit stage, or manufacture completion. A missing or
stale Verification blocks commit. Compensation is a separately authorized new
Intent and audit chain.

## 11. Sensitivity, minimization, redaction, and isolation

### 11.1 Monotone protection

Record and Event sensitivity are at least the maximum protection of every bound
subject/evidence item and policy. Compartments are the required union; a writer
cannot remove one. Tenant scope and compartments are reauthorized on every
read, resume, redaction, export, and recovery operation. Existence-safe denial
is required where policy protects resource existence.

### 11.2 Creation-time minimization

The authoritative record may contain principal/ActorChain refs, decision,
stage, registered error, safe reason, versions, digests, and minimized outcome
facts. It must not contain private keys, bearer/session secrets, credentials,
complete protected payloads, cross-tenant content, raw signature bytes, or other
replayable signature material. Protected detail is represented by a separately
governed strong ref and digest after independent authorization.

### 11.3 Redaction

Authoritative bytes are never redacted in place. Query/export produces a
derived view under an exact digest-pinned redaction profile. The derived view
binds source record strong ref/digest, profile triple, exact included/excluded
paths, safe reason, deciding authority, and derived digest. Caller-selected
field deletion is forbidden. Unauthorized or lossy redaction fails and is
audited. A redaction view is not a substitute for its source record.

## 12. Retention, legal hold, compaction, and export

### 12.1 Retention floor

The base profile does not invent a universal number of days or years. Every
stream selects an exact digest-pinned retention policy that defines finite
minimums by scope, risk, jurisdiction, record kind, and sensitivity. Missing or
ambiguous policy fails closed.

The effective retention floor is the maximum of the audit policy and all bound
subject, Effect, Verification, incident, contractual, and applicable legal
obligations. A subordinate tenant policy may lengthen but cannot shorten a
platform or applicable minimum. `expires_at` is the earliest compaction-review
time, not an automatic deletion instruction.

### 12.2 Legal hold

A platform or tenant compliance authority may set a hold within its scope. A
hold is indefinite until explicitly released. Release requires a different
principal and ActorChain acting as the same-or-higher compliance authority. The
audited subject, record writer, workload, or hold setter cannot release it.
Setting, rejecting, and releasing a hold are authoritative audit records.

### 12.3 Compaction and tombstones

No record is silently deleted at expiry. Physical compaction is allowed only for
a checkpoint-sealed complete segment, after the retention floor and every hold
clear. A successor compaction/tombstone record and signed checkpoint preserve
stream/range identity, record count, first/last record digests, the canonical
ordered-record-digest aggregate, predecessor/successor continuity, policy and
hold decisions, and compaction authority. Selective removal or an unsealed
segment is forbidden. Until that future compaction profile is registered,
authoritative records remain retained.

### 12.4 Export

Export requires a new authorization decision fixing tenant/platform scope,
compartments, query/filter, redaction profile, stream set, sequence ranges,
source signed checkpoints, and exact source high-watermarks. It mutates no
source record.

The proposed logical format is:

- `records.ndjson`: records or registered redacted views in ascending stream
  sequence; each line is exact RFC 8785 UTF-8 bytes followed by one LF, no BOM;
- canonical `manifest.json`: export/profile identities, authorizing principal
  and authority strong refs, filters, tenant/compartment scope, per-stream range,
  count, ordered record/view digests, source checkpoint/high-watermark,
  omissions/compaction tombstones, redaction pins, and export digest;
- detached pure-Ed25519 manifest signature using the dedicated
  `audit-export-signing` key/profile.

Archive path, compression, and timestamps do not affect the logical manifest
digest. Verification detects omission, duplicate, reorder, gap, stale/drifting
high-watermark, signature failure, and cross-tenant leakage. Export success is
recorded in the source audit stream and proves neither the original business
authorization nor Effect completion or Profile conformance.

## 13. Audit of all 55 registered errors

Existing codes are reused only for their exact registered condition.

| Registered code | AUDIT ruling |
|---|---|
| `STATE_CONFLICT` | exact only for expected-version/CAS conflict; not gap, fork, or tamper |
| `STATE_STALE_OBSERVATION` | no audit-integrity reuse; only declared stale observation |
| `STATE_STORE_UNAVAILABLE` | exact only for authoritative state/Event persistence failure |
| `CONTEXT_INCOMPLETE` | no audit reuse |
| `CONTEXT_BUDGET_EXCEEDED` | no audit reuse |
| `CONTEXT_AUTH_DENIED` | exact only for existing context authorization condition |
| `AUTH_CAPABILITY_ATTENUATION_VIOLATION` | exact only for capability expansion |
| `AUTH_CAPABILITY_EXPIRED` | exact only for capability lease expiry |
| `EFFECT_OUTCOME_UNKNOWN` | exact for uncertain external outcome; not audit failure |
| `EFFECT_RECOVERY_QUARANTINED` | exact for unrecoverable Effect outcome |
| `EFFECT_IDEMPOTENCY_CONFLICT` | exact for same key/different parameters |
| `PROTOCOL_MAPPING_INCOMPLETE` | exact for lossy required audit semantics across a mapping |
| `PROTOCOL_SCHEMA_DIGEST_MISMATCH` | protocol payload schema only; not audit-record schema |
| `VERSION_UNSUPPORTED` | exact for unsupported protocol/specification compatibility window |
| `CRITICAL_EXTENSION_UNKNOWN` | exact for unknown critical audit extension |
| `SCHEMA_MISMATCH` | exact for malformed object after an audit schema is registered |
| `DIGEST_MISMATCH` | exact for recomputed declared canonical record digest only; chain semantics need dedicated errors |
| `RESOURCE_BUDGET_EXHAUSTED` | no audit-integrity reuse |
| `PROFILE_CIM_CALIBRATION_MISMATCH` | no audit reuse |
| `PROFILE_LEARNING_PROMOTION_DENIED` | no audit reuse |
| `PROFILE_EMBODIED_OBSERVATION_STALE` | no audit reuse |
| `KNOWLEDGE_SOURCE_INVALIDATED` | no audit reuse |
| `KNOWLEDGE_POISON_QUARANTINED` | no audit reuse |
| `KNOWLEDGE_MAINTENANCE_BOUNDED` | no audit reuse |
| `PERFORMANCE_REPORT_INCOMPLETE` | no audit reuse |
| `MANAGEMENT_SESSION_EXPIRED` | exact session business denial; denial still requires audit |
| `MANAGEMENT_SESSION_REVOKED` | exact session business denial; denial still requires audit |
| `MANAGEMENT_STEP_UP_REQUIRED` | exact management challenge condition |
| `MANAGEMENT_SCOPE_MISMATCH` | exact management scope mismatch |
| `MANAGEMENT_SELF_AUTHORIZATION_DENIED` | exact only for self-authorization or altering authoritative audit |
| `MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED` | exact missing-independent-approval condition |
| `AGENT_PACKAGE_VERIFICATION_FAILED` | no audit reuse |
| `AGENT_ADAPTER_BYPASS_DETECTED` | no audit-integrity reuse |
| `AGENT_COMPATIBILITY_DEGRADED` | no audit reuse |
| `MEMORY_ADMISSION_DENIED` | no audit reuse |
| `MEMORY_SCOPE_PROMOTION_REQUIRED` | no audit reuse |
| `MEMORY_DERIVATION_INVALIDATED` | no audit reuse |
| `RESOURCE_NOT_DISCOVERABLE` | no audit reuse |
| `CONTEXT_RESOLUTION_STAGNATED` | no audit reuse |
| `CATALOG_VERSION_STALE` | no audit reuse |
| `CATALOG_MATCH_INCONCLUSIVE` | no audit reuse |
| `NO_AUTHORIZED_OPERATION_CANDIDATE` | no audit reuse |
| `SEMANTIC_SERVICE_UNAVAILABLE` | no audit reuse |
| `SEMANTIC_MATCH_INCONCLUSIVE` | no audit reuse |
| `MODEL_EGRESS_DENIED` | no audit reuse |
| `SEMANTIC_BUDGET_EXHAUSTED` | no audit reuse |
| `SHELL_TARGET_AMBIGUOUS` | no audit reuse |
| `SHELL_TARGET_NOT_FOUND` | no audit reuse |
| `SHELL_PREVIEW_STALE` | no audit reuse |
| `INTENT_CLARIFICATION_REQUIRED` | no audit reuse |
| `INTENT_VERSION_SUPERSEDED` | no audit reuse |
| `CANCEL_PENDING` | no audit reuse |
| `CANCEL_TOO_LATE` | no audit reuse |
| `WATCH_CURSOR_STALE` | watch cursor only; never audit export/high-watermark |
| `SHELL_CHANNEL_BINDING_MISMATCH` | exact task/management channel mix only |

### 13.1 Owner-confirmed future AUDIT errors

The following exact responsibilities are proposed under a future `audit`
category. Names remain unregistered and machine-registration pending.

| Future code | Exact responsibility | Retryable |
|---|---|---|
| `AUDIT_PROFILE_UNKNOWN` | selected audit profile absent from specification set | false |
| `AUDIT_SCHEMA_DIGEST_MISMATCH` | record/checkpoint/export schema differs from profile pin | false |
| `AUDIT_DIGEST_DOMAIN_MISMATCH` | wrong record/checkpoint/export digest domain | false |
| `AUDIT_PROJECTION_MISMATCH` | projection or exclusion set differs from registered profile | false |
| `AUDIT_STREAM_IDENTITY_MISMATCH` | record scope tuple differs from selected stream | false |
| `AUDIT_SEQUENCE_DUPLICATE` | sequence already has a committed record | false |
| `AUDIT_SEQUENCE_GAP` | next sequence is greater than high-watermark plus one | false |
| `AUDIT_SEQUENCE_REGRESSION` | sequence is below the committed high-watermark | false |
| `AUDIT_STREAM_FORK_DETECTED` | competing digest exists for the same stream position | false |
| `AUDIT_PREVIOUS_RECORD_DIGEST_MISMATCH` | record does not link to current chain head | false |
| `AUDIT_HIGH_WATERMARK_MISMATCH` | claimed high-watermark differs from verified stream state | false |
| `AUDIT_CHECKPOINT_MISMATCH` | checkpoint range/count/head/previous checkpoint is inconsistent | false |
| `AUDIT_CHECKPOINT_SIGNATURE_INVALID` | dedicated checkpoint signature/profile verification fails | false |
| `AUDIT_RECORD_MISSING` | required denial/stage/commit record is absent | false |
| `AUDIT_ORPHAN_BINDING` | audit/state/Event/transition/receipt binding lacks its required counterpart | false |
| `AUDIT_TAMPER_DETECTED` | update/delete/truncation/reorder/in-place correction detected | false |
| `AUDIT_WRITER_UNAUTHORIZED` | writer authority, usage, epoch, or fencing is invalid | false |
| `AUDIT_STORE_UNAVAILABLE` | authoritative audit persistence is temporarily unavailable | true |
| `AUDIT_ATOMIC_COMMIT_FAILED` | required audit and authority batch did not commit as one unit | true |
| `AUDIT_RECEIPT_SUBJECT_MISMATCH` | receipt subject/version/digest differs from bound record | false |
| `AUDIT_SENSITIVITY_DOWNGRADE` | record/view/export lowers sensitivity or compartments | false |
| `AUDIT_REDACTION_POLICY_VIOLATION` | redaction is unauthorized, caller-defined, lossy, or profile-mismatched | false |
| `AUDIT_SECRET_LEAKAGE_DETECTED` | forbidden secret/protected/replayable material is present | false |
| `AUDIT_CROSS_TENANT_ACCESS_DENIED` | read/write/export crosses tenant or compartment boundary | false |
| `AUDIT_RETENTION_POLICY_MISMATCH` | effective floor or policy digest is missing/mismatched | false |
| `AUDIT_LEGAL_HOLD_VIOLATION` | hold set/release/compaction violates authority or independence | false |
| `AUDIT_COMPACTION_POLICY_VIOLATION` | segment removal lacks eligible sealed tombstone/checkpoint proof | false |
| `AUDIT_EXPORT_UNAUTHORIZED` | export query/scope/filter lacks current authorization | false |
| `AUDIT_EXPORT_INCOMPLETE` | omission, duplicate, gap, or missing tombstone/checkpoint detected | false |
| `AUDIT_EXPORT_ORDER_MISMATCH` | export order differs from verified stream sequence | false |
| `AUDIT_EXPORT_HIGH_WATERMARK_MISMATCH` | source changed or manifest binds wrong high-watermark | false |
| `AUDIT_EXPORT_SIGNATURE_INVALID` | export-manifest signature/profile verification fails | false |
| `AUDIT_NEGOTIATION_EPOCH_MISMATCH` | audit/profile/policy binding differs from current epoch | false |
| `AUDIT_CRITICAL_EXTENSION_MISMATCH` | required audit extension was stripped or digest-mismatched | false |
| `AUDIT_RECOVERY_BARRIER_REQUIRED` | recovery cannot verify chain/checkpoint to required high-watermark | true |

Only transient store availability, joined atomic retry, and a recoverable barrier
are retryable, and only after the underlying authority state changes. Replaying
the same unsafe request is never an automatic repair.

## 14. Planned positive and negative matrix

All cases are **planned/not executed**. Future vectors are new assets; no old
`expected` value changes.

Planned positives cover safe denial persistence, one atomic governed commit,
complete external-Effect closure, deterministic replay to a signed checkpoint,
legal-hold protection, authorized redacted query, and signed complete export.

| # | Planned negative | Earliest responsibility | Status |
|---|---|---|---|
| 1 | missing required audit record | persistence/closure | planned/not executed |
| 2 | unknown audit profile | negotiation/profile | planned/not executed |
| 3 | profile version/digest mismatch | negotiation/profile | planned/not executed |
| 4 | wrong audit record schema digest | schema | planned/not executed |
| 5 | malformed audit record | schema | planned/not executed |
| 6 | wrong record digest domain | canonical digest | planned/not executed |
| 7 | wrong digest projection | canonical digest | planned/not executed |
| 8 | undeclared excluded path | canonical digest | planned/not executed |
| 9 | record content digest mismatch | canonical digest | planned/not executed |
| 10 | wrong stream identity | stream admission | planned/not executed |
| 11 | cross-tenant stream injection | stream admission | planned/not executed |
| 12 | duplicate sequence | sequence CAS | planned/not executed |
| 13 | sequence gap | sequence CAS | planned/not executed |
| 14 | sequence regression | sequence CAS | planned/not executed |
| 15 | stream fork | chain verification | planned/not executed |
| 16 | previous-record digest mismatch | chain verification | planned/not executed |
| 17 | high-watermark mismatch | chain verification | planned/not executed |
| 18 | checkpoint digest mismatch | checkpoint | planned/not executed |
| 19 | record omitted from checkpoint | checkpoint | planned/not executed |
| 20 | append after stale high-watermark | sequence CAS | planned/not executed |
| 21 | ordinary Agent attempts update | writer authorization | planned/not executed |
| 22 | ordinary Agent attempts delete | writer authorization | planned/not executed |
| 23 | log truncation | recovery/checkpoint | planned/not executed |
| 24 | silent in-place correction | immutable record | planned/not executed |
| 25 | denial without audit | denial closure | planned/not executed |
| 26 | denial audit contains protected content | minimization | planned/not executed |
| 27 | signature receipt missing before business authorization | SIG handoff | planned/not executed |
| 28 | signature receipt version/digest mismatch | SIG handoff | planned/not executed |
| 29 | session/approval/ActorChain mismatch | business authorization | planned/not executed |
| 30 | state commit without audit | atomic commit | planned/not executed |
| 31 | audit commit without state/transition commit | atomic commit | planned/not executed |
| 32 | Event present but audit missing | atomic commit | planned/not executed |
| 33 | transition record present but audit missing | atomic commit | planned/not executed |
| 34 | rejection followed by dispatch | pre-authority oracle | planned/not executed |
| 35 | rejection followed by Effect | pre-authority oracle | planned/not executed |
| 36 | audit failure followed by business mutation | atomic failure | planned/not executed |
| 37 | audit failure followed by authority commit | atomic failure | planned/not executed |
| 38 | audit failure followed by success receipt | result visibility | planned/not executed |
| 39 | external Effect dispatch without pre-dispatch audit | Effect closure | planned/not executed |
| 40 | outcome unknown reported as success | Effect closure | planned/not executed |
| 41 | reconciliation audit chain gap | Effect closure | planned/not executed |
| 42 | Verification missing or stale | commit gate | planned/not executed |
| 43 | sensitivity downgrade | protection | planned/not executed |
| 44 | unauthorized redaction | redaction | planned/not executed |
| 45 | secret or replayable signature material exported | minimization/export | planned/not executed |
| 46 | retention shorter than policy | retention | planned/not executed |
| 47 | legal hold bypass | legal hold | planned/not executed |
| 48 | unauthorized export | export authorization | planned/not executed |
| 49 | export omission | export completeness | planned/not executed |
| 50 | export duplicate/reorder/gap | export ordering | planned/not executed |
| 51 | export crosses tenant/compartment | export isolation | planned/not executed |
| 52 | export high-watermark drift | export snapshot | planned/not executed |
| 53 | old negotiation epoch | negotiation | planned/not executed |
| 54 | critical audit extension stripped | negotiation | planned/not executed |
| 55 | audit store unavailable | persistence | planned/not executed |
| 56 | recovery resumes past audit gap | recovery barrier | planned/not executed |
| 57 | wall-clock ordering substituted for logical sequence | ordering | planned/not executed |
| 58 | Event/open payload/private row/vector boolean treated as audit proof | authority boundary | planned/not executed |

Precise oracles:

- pre-authority failure: no dispatch, Effect, business mutation, authority
  business commit, or success receipt;
- denial: exactly one safe denial-audit commit is allowed and required, with no
  business commit;
- tamper/gap/recovery failure: recovery barrier or quarantine, never progress
  beyond the last verified high-watermark;
- export failure: no source mutation, no cross-tenant content, and no success
  export receipt.

Static checks, builds, ordinary unit tests, and existing CI runs are repository
integrity evidence only and are not execution of this planned matrix.

## 15. Compatibility and migration

- Native selection is exact `0.2.0-draft.1` with record/stream/checkpoint/
  retention/redaction/export/receipt profiles and all digests pinned.
- Any temporary adapter is limited to exact `0.2.0-draft.1` and
  `0.2.0-draft.2` and removed at `0.2.0-draft.3`; none exists now.
- An old negotiation epoch cannot acquire the audit critical extension.
- Existing Event rows, transition records, outbox rows, `DenialAudit`,
  `audit_ref`, logs, telemetry, and vectors are migration inputs only. They are
  not backfilled or reclassified as authoritative audit records.
- Imported legacy facts, if accepted later, receive new record IDs/sequences,
  explicit source provenance and loss declarations; they never claim historical
  continuity that cannot be verified.
- Before enabling the new epoch, recovery verifies or establishes a genesis
  checkpoint and inventories in-flight Effects. Missing continuity enters
  quarantine.
- In-flight Effects retain original idempotency, fencing, session/approval,
  Verification, and audit obligations.
- Profile, topology, sequence, chain, signature, checkpoint, retention,
  redaction, export, or error changes are breaking and receive new SemVer,
  digest, release notes, and migration.

No adapter, migration, persistence port, key, profile, or policy is implemented
by this decision.

## 16. SIG receipt handoff and responsibility split

SIG owns the future `SignatureVerificationReceipt` facts and its exact subject,
profile, algorithm, key, trust, status, digest, epoch, result, and failed-stage
bindings. AUDIT owns its authoritative record carrier, sequence, checkpoint,
retention, sensitivity, export, and atomic persistence slot.

Receipt persistence completes before business authorization. The audit record
strongly binds the receipt ID/version/digest and independently binds the subject
ID/version/digest. A missing or mismatched receipt denies. The receipt is not
independently signed, grants no authority, and cannot prove audit persistence or
business commit.

## 17. Authorization non-expansion proof

Effective authorization remains the intersection:

```text
operation membership
∩ negotiation epoch and critical extensions
∩ authenticated peer/channel
∩ current SIG session/approval verification
∩ current capability, risk, policy and revocation state
∩ target authority
∩ audit writer/read/export authority
∩ retention, sensitivity and compartment policy
```

An audit profile, record, sequence, checkpoint, signature, receipt, Event,
transition, export, log, URI, row, or boolean grants no operation membership,
session scope, capability, risk allowance, approval, target authority, Effect
completion, acceptance, or Profile claim. Audit failure narrows availability; it
never creates a fallback authority path. Query/export authority cannot mutate
the source stream or authorize the business action being audited.

## 18. Owners, blockers, and evidence limitation

### Owner/reviewer roles

- repository owner: design and version boundary;
- security/cryptography reviewer: chain, signatures, key usages, downgrade and
  tamper model;
- audit/store/recovery reviewer: sequence authority, atomic slot, checkpoints,
  crash and replay behavior;
- compliance/privacy reviewer: retention policy, legal-hold independence,
  minimization, redaction, compartments, and export;
- SIG reviewer: receipt handoff and key-registry compatibility;
- OPS/TARGET reviewers: operation/target/Effect/Verification binding closure.

The owner confirmed the technical selections through the active governance
session. This is not a GitHub review of the resulting PR head. No review or
merge exception from PR #50 through #53 applies to this AUDIT PR, machine
registration, CA-0, implementation, or CFR.

After PR #54 merged, the repository owner expressly authorized the preceding
agent to review the exact merged design from security, audit, and compliance
perspectives. That review found no blocking design defect. It may be recorded as
an owner-authorized AUDIT review completed, but it is not an external human,
third-party, independent cryptography, or GitHub review and does not expand any
machine-registration or implementation authority.

### Blocking facts

- exact future schema/profile/policy bytes and all digests;
- positive finite checkpoint thresholds and actual retention-policy values;
- governed checkpoint/export key descriptors and registered usages;
- registration of the audit critical extension, future `audit` error category,
  errors, generated bindings, and persistence port;
- SIG independent security review and registered receipt;
- OPS/TARGET/SIG machine registrations and complete mutual binding;
- new vectors and real deterministic behavior execution.

### Evidence limitation

This batch is a static source audit and docs-only design. It does not execute a
new behavior vector, register a profile, implement Configuration Authority,
prove storage or cryptographic behavior, create conformance evidence, or support
a Profile claim.

## 19. GO/NO-GO and downstream order

- `GO`: AUDIT docs-only design is merged and its owner-authorized
  security/audit/compliance review is completed with the provenance limitation
  above.
- `NO-GO`: AUDIT machine registration, OPS/TARGET/SIG registration,
  Configuration Authority implementation, behavior execution, and Profile
  claim.

Downstream order remains:

1. OPS/TARGET/SIG/AUDIT four independent machine-registration lines, with SIG
   independent security/cryptography review before SIG registration;
2. independent CA-0 re-review;
3. explicit CA-0 GO;
4. only then, implementation;
5. Management CFR after real implementation exists.

AUDIT merge does not make Event, transition records, receipts, `audit_ref`,
SQLite rows, booleans, logs, sequence values, hash proposals, retention rules,
or export proposals into machine contracts.

Preserved state:

- 273 requirements, 55 errors, 61 schemas, 84 vectors;
- 59 pass, 25 not-run, self-check 40;
- matrix non-empty implementation count 70;
- Profile implemented = 0;
- D-016 open;
- D-022 blocking;
- CA-1 through CA-8 blocked;
- all eight operation candidates and three configure candidates blocked;
- OPS/TARGET/SIG/AUDIT machine contracts unregistered;
- Configuration Authority implementation not provided;
- new behavior not executed.
