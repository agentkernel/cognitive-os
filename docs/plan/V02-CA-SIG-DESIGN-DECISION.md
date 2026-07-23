# V02-CA-SIG-01 Session and Approval Signature Design Decision

- Decision ID: `V02-CA-SIG-01`
- Date: 2026-07-22
- Status: **High-Assurance extension design; does not block Ordinary Core tracer development; final review and registration pending**
- Baseline: `origin/main@42d609b2f49e2db641f46aa99b6cc9a538a7f4fd`
  (PR #52 merge; main CI run `29922529556` Ubuntu/Windows success)
- Inputs: [V02-CA-GOV-00](V02-CONFIGURATION-AUTHORITY-NORMATIVE-SURFACE-AUTHORIZATION.md),
  [V02-CA-OPS-01](V02-CA-OPS-DESIGN-DECISION.md), and
  [V02-CA-TARGET-01](V02-CA-TARGET-DESIGN-DECISION.md)
- Structural governance:
  [ADR-0012](../adr/0012-v02-detached-signature-profile-governance.md)
- Classification: docs-only structural design; no machine registration
- Owner-confirmed profile family/version: `cognitiveos.detached-signature-envelope/0.2`,
  `0.2.0-draft.1`; digest `unresolved/not computed`
- Session profile version/digest: `0.2.0-draft.1`;
  `unresolved/not computed`
- Approval profile version/digest: `0.2.0-draft.1`;
  `unresolved/not computed`

## 1. Decision and status boundary

This decision designs the Configuration Authority signature obligations for
exactly two existing fields:

1. `PrivilegedManagementSession.authority_signature`;
2. `ManagementApprovalDecision.authority_signature`.

A reusable detached-signature envelope/profile family is structurally sound,
but a single generic signing profile is not. Session issuance and approval
decisions have different authorities, replay boundaries, signed schemas,
domains, projections, key usages, lifetimes, and business authorization gates.
The owner-confirmed family therefore has two object-specific profiles:

- `cognitiveos.signature.management-session-authority/0.2`, domain
  `management-session-authority/0.2`;
- `cognitiveos.signature.management-approval-authority/0.2`, domain
  `management-approval-authority/0.2`.

The repository owner has now selected the algorithm, key infrastructure, trust
model, rotation/revocation rules, domains/projections, receipt boundary, tiered
approval semantics, session renewal rules, exact future errors, and
critical-extension/replay model below. The prior technical-choice blocker is
closed at the design level. Both profiles remain unregistered and unselectable
until independent security review and later machine registration assign exact
schema/profile bytes and digests.

This decision registers no requirement, error, schema, state domain,
transition, vector, generated binding, operation, extension, specification
set, implementation, evidence artifact, or Profile claim. It does not modify
the two existing string fields. It executes no behavior vector and changes no
existing vector `expected` value.

The following remain distinct and must each be proven independently:

1. canonical digest integrity;
2. cryptographic signature validity;
3. signer and key authorization;
4. trust-root validity;
5. rotation and revocation status;
6. current session or approval business authorization.

Success at one layer never implies success at another.

## 2. Source-audit method and precedence

The audit used the repository precedence and fail-closed rule:

1. current registered schemas, registries, transitions, and vectors;
2. pinned Core, RFC, and normative standards;
3. approved GOV plus merged OPS/TARGET design inputs;
4. tracked implementation only as evidence of private behavior, never as a
   source of cross-boundary authority.

Inspected machine assets include the privileged session, approval request,
approval decision, capability, proposal, AKP envelope, governed-object, and
object-reference schemas; the requirement and 55-code error registries; and
the management session, approval, self-authorization, independent-approval,
configuration, fallback, recovery, and channel-isolation vectors.

## 3. Existing machine and implementation facts

### 3.1 `PrivilegedManagementSession` field audit

| Field or group | Current registered fact | Signature consequence |
|---|---|---|
| `schema_version` | exact `cognitiveos.privileged-management-session/0.1` | a future signed schema must use a new immutable identity/digest; the v0.1 string shape cannot silently become a v0.2 envelope |
| `session_id`, `object_version`, `management_domain` | required; patterned ID/domain; version >= 1 | all are included in the session signed projection and replay boundary |
| `session_authority` | required URI reference | names an authority but does not resolve a key, trust root, or current authority status |
| `human_principal`, `actor_chain_digest` | required principal ref and digest | included; the full ActorChain must still be resolved and authorized independently |
| authentication/activity context refs | required URI references | included; references still require current authorized resolution |
| scope domains/actions/resources | required closed scope object | included in full; a valid signature cannot widen these bounds |
| `risk_ceiling` | R0-R3 upper bound | included; it is not action risk or approval |
| `policy_version`, `revocation_epoch` | required integers | included and compared to current authority state after cryptographic verification |
| issue/activity/idle/absolute times | required | included; current-time expiry is rechecked after signature verification |
| `state` | pending/active/expired/revoked/closed | included; only current active state may proceed |
| optional step-up/isolation fields | schema-known optional fields | retained if present; never dropped by canonicalization |
| `session_digest` | required SHA-256-shaped string | no registered digest domain or projection binds it to the session fields |
| `authority_signature` | required `string`, `minLength: 16` | no algorithm, key, domain, schema, projection, encoding, trust, rotation, revocation, or receipt contract |

### 3.2 `ManagementApprovalRequest` source audit

The request schema fixes the challenge card: request ID, proposal ref/digest,
risk class, tiered confirmation surface, human and proposer principals,
proposer ActorChain digest, independent channel identity, challenge digest,
method, single use, request time, and expiry. R2/R3 require a `session_ref`.

It does not define a request content-digest profile or a signature profile. A
future approval signature therefore must bind the exact referenced request and
challenge through an immutable request schema/digest contract; a URI reference
alone is insufficient. Natural language remains only a challenge trigger.

### 3.3 `ManagementApprovalDecision` field audit

| Field or group | Current registered fact | Signature consequence |
|---|---|---|
| `schema_version`, `decision_id`, `object_version` | required | included in the approval signed projection and replay boundary |
| `proposal_ref`, `proposal_digest` | required | included; proposal digest closure remains unresolved until its exact domain/projection includes target profile and parameters |
| `request_ref`, `single_use` | required only for R1 approve by current conditional | included when present; future approve profiles must close request/challenge binding for each applicable tier |
| `session_ref` | required | included; the referenced session is revalidated independently |
| `decision` | approve/deny/challenge/expired | included; signature validity cannot turn a non-approve decision into approval |
| deciding authority and approver principal/ActorChain | required | included; authority, independence, and key usage are independently checked |
| independent/step-up fields | conditional or optional | included when present; booleans and strings are assertions, not proof |
| `policy_version`, `risk_class` | required | included; current policy and actual action risk are revalidated |
| challenge, decision, expiry times | required | included; freshness and expiry are business checks |
| `decision_digest` | required SHA-256-shaped string | no registered domain or projection binds it to the decision fields |
| `authority_signature` | required `string`, `minLength: 16` | same missing cryptographic facts as the session field |

### 3.4 Related machine facts and non-authority boundaries

- `AuthorizationCapability.signature` is also an open string. This SIG batch
  does not redesign the capability cryptography or use it as precedent.
- `ManagementActionProposal.parameters` is open JSON; `proposal_digest` and
  `parameters_digest` are shaped digests without a complete SIG/TARGET domain
  closure. They are inputs, not proof of exact target authorization.
- AKP request/result envelopes provide schema/payload/result digest slots and
  critical extensions, but do not register a session or approval signature
  profile, key resolver, trust root, or verification receipt.
- `MGMT-CONFIG-001` supplies `authority_signature_valid: true` as a scenario
  input. It does not define a verifier or profile.
- `MGMT-APPROVAL-005` requires an independent signed decision as an expected
  property. It does not define the signature input or key system.

Tracked implementation facts are explicitly private and insufficient:

- the management session parser checks the generated schema shape, digest
  string shape, and `authority_signature` length, while its module states that
  cryptographic verification is not implemented;
- the R1 approval test helper emits the literal `os-authority-signature` and
  the approval gate checks challenge, expiry, replay, and proposer independence
  without cryptographic verification;
- Rust and TypeScript canonical libraries can build the standard section 12
  signature preimage. They do not select an algorithm, resolve a key, validate
  trust, verify a signature, or create a cross-boundary profile.

String length, JSON parsing, schema validity, digest-shaped text, fixture
builders, private helpers, caller booleans, and canonical preimage construction
are not cryptographic proof.

## 4. Shared envelope family and object-specific profile decision

### 4.1 Owner-confirmed reusable envelope

A future machine-registration PR may propose a closed detached envelope with
these logical fields:

| Field | Owner-confirmed obligation |
|---|---|
| `signature_profile` | immutable `(asset_id, complete SemVer, sha256 digest)` |
| `algorithm` | exact case-sensitive ASCII identifier in the selected profile's closed allowed set |
| `key_id` | non-caller-selected identifier resolved only by the selected authority resolver profile |
| `signed_domain` | exact profile domain; never `generic`, `object`, or `payload` |
| `signed_schema_digest` | exact digest of the object-specific signed projection schema |
| `signed_content_digest` | recomputed cross-check for the canonical signed projection |
| `negotiation_epoch_digest` | exact epoch binding selected by authenticated negotiation; never caller supplied |
| `signature_bytes` | encoding fixed by the selected algorithm/profile pair |

The envelope deliberately does not accept a caller-provided public key,
resolver, trust root, projection, excluded path, algorithm alias, or fallback
algorithm. Those facts come from the digest-pinned profile and negotiation
epoch. An unknown field cannot silently change security behavior.

### 4.2 Separate profiles are mandatory

The session and approval profiles have separate:

- asset IDs, SemVer, and digests;
- allowed signer authorities and key usages;
- domains and signed schema digests;
- projections and exclusions;
- replay keys, lifetimes, and version checks;
- policy, revocation, and negotiation epoch bindings;
- business authorization rules and receipts.

The same signature bytes, key authorization decision, or profile selection
cannot be replayed between the two object types. Shared envelope shape is not a
shared authorization domain.

## 5. Owner-confirmed algorithm, key, and trust decision

These choices are design-authoritative owner decisions for this v0.2 packet.
They are not machine contracts until separately registered and digest-pinned.

### 5.1 Pure Ed25519 only

- The closed allowed set contains exactly case-sensitive identifier `Ed25519`.
- It means pure RFC 8032 Ed25519, not `Ed25519ctx` or `Ed25519ph`.
- The algorithm signs the canonical section 12 signature input directly. No
  application SHA-256 or other prehash is added; the algorithm's internal
  hashing remains part of Ed25519.
- Public keys are exactly 32 raw bytes and signatures exactly 64 raw bytes,
  encoded as unpadded base64url.
- Verification is strict: reject wrong lengths, non-canonical point/scalar
  encodings, `S >= L`, small-order public keys or signature points, and any
  verification result not accepted by a strict Ed25519 verifier.
- PEM, DER, hex, standard base64, algorithm aliases, `none`, caller-selected
  algorithms, fallback, and downgrade are forbidden.
- Any future algorithm addition or removal creates a new SemVer/digest and
  migration; P-256 is not in the v0.2 initial allowed set.

### 5.2 Governed authority-key registry

- `key_id` is the existing strong-reference structure `(id, object_version,
  content_digest)`, not a caller string or resolver URL.
- It resolves through a governed CognitiveOS authority-key registry to exactly
  one digest-pinned descriptor containing the Ed25519 public key, owner and
  deciding authority, one key usage, validity interval, status, rotation
  generation, predecessor/successor refs, revocation epoch, tenant/domain
  bounds, and descriptor digest.
- Session and approval use distinct leaf keys. A leaf descriptor has exactly
  one usage: `management-session-signing` or
  `management-approval-signing`. A proposer or workload cannot own either
  usage.
- Private-key custody may be external KMS/HSM, but custody cannot define or
  override key identity, resolver behavior, trust roots, status, or usage.
- Caller-provided keys, resolver URLs, trust anchors, cache rows, or private
  database rows are forbidden authority inputs.

### 5.3 Platform-rooted registry manifest and tenant delegation

- The platform governance root is the single trust anchor. Its key usage is
  only `authority-key-certification`; it cannot sign sessions or approvals.
- The root is offline or HSM-protected and signs an immutable
  `AuthorityKeyRegistryManifest` with domain `authority-key-registry/0.2`.
- Each new negotiation epoch pins the registry manifest asset ID, complete
  SemVer, digest, and certification profile. The manifest lists every usable
  key descriptor/delegation and its status/usage/bounds.
- Tenant authorities are never self-anchored roots. A tenant authority is valid
  only through a platform-root-signed, digest-pinned, maximum-depth-one
  delegation that monotonically narrows tenant, management domain, scope, risk,
  key usage, validity, and revocation epoch.
- Missing, cross-tenant, expired, revoked, expanded, or indeterminate
  delegation fails closed.

### 5.4 Rotation, revocation, and current status

- Key status is one of `scheduled`, `active`, `retiring`, `revoked`, or
  `expired`.
- For each `(authority, key_usage)`, exactly one key is `active`. New signatures
  use only that key.
- When a successor becomes active, the predecessor enters `retiring` and may
  verify only objects signed before successor activation, for at most 24 hours
  and never beyond the object's or key's own expiry.
- `revoked` is immediate and has no grace period. Existing signed sessions and
  approvals cannot authorize new work after revocation.
- Authorization and commit both resolve the current authoritative descriptor,
  manifest, delegation, and revocation epoch. Resolver unavailability,
  ambiguity, or stale status fails closed; no stale-cache authorization is
  allowed.
- Historical audit may record that a signature was valid at signing time, but
  historical validity cannot restore current authorization.

These selections close the algorithm/key/trust choice at the docs-only design
level. Independent security review, exact machine schemas/digests, registration,
generated bindings, implementation, and behavior evidence remain pending.

## 6. Complete signature binding matrix

Every row is mandatory. “Owner-confirmed” means docs-only design, not machine
registration or runtime authority.

| # | Binding | Session profile | Approval profile | Status |
|---|---|---|---|---|
| 1 | profile identity | `cognitiveos.signature.management-session-authority/0.2` | `cognitiveos.signature.management-approval-authority/0.2` | owner-confirmed, unregistered |
| 2 | profile SemVer/digest | owner-confirmed `0.2.0-draft.1`; digest computed at registration | same version; independent digest computed at registration | design-closed; unregistered |
| 3 | allowed algorithm set | exactly pure Ed25519 | exactly pure Ed25519 | owner-confirmed |
| 4 | algorithm identifier | exact `Ed25519` | exact `Ed25519` | owner-confirmed |
| 5 | signature encoding | 64 raw bytes, unpadded base64url; strict verification | same | owner-confirmed |
| 6 | key ID shape | strong ref to session-signing descriptor | strong ref to approval-signing descriptor | owner-confirmed; schema unregistered |
| 7 | deterministic key resolution | governed authority-key registry only | same, distinct leaf | owner-confirmed; registry unregistered |
| 8 | trust-root binding | platform registry manifest or max-depth-one tenant delegation | same | owner-confirmed; manifest unregistered |
| 9 | key owner/authority | current descriptor/delegation binds `session_authority` | current descriptor/delegation binds `deciding_authority` | owner-confirmed |
| 10 | key usage/purpose | exact `management-session-signing`; exclusive leaf | exact `management-approval-signing`; exclusive leaf | owner-confirmed |
| 11 | rotation | active successor; predecessor retiring <= 24 hours and only for pre-activation objects | same | owner-confirmed |
| 12 | revocation | immediate, no grace; future authorization denied | same | owner-confirmed |
| 13 | verification-time status | authoritative manifest/descriptor/delegation read at authorization and commit; no stale cache | same, independently | owner-confirmed receipt obligation |
| 14 | signature domain | `management-session-authority/0.2` | `management-approval-authority/0.2` | owner-confirmed, independent |
| 15 | signed schema identity | future v0.2 session projection schema/digest | future v0.2 approval projection schema/digest | unregistered |
| 16 | signed projection | closed session binding record containing profile/algorithm/key/schema/epoch metadata plus complete subject except `/authority_signature` | closed approval binding record additionally containing session/request/revocation bindings plus complete subject except `/authority_signature` | owner-confirmed; section 7; schema unregistered |
| 17 | excluded paths | subject source excludes only `/authority_signature`; binding record has no signature-bytes member | subject source excludes only `/authority_signature`; binding record has no signature-bytes member | owner-confirmed; digest exclusions are separate |
| 18 | content-digest cross-check | recompute `session_digest` plus signed-projection digest domain `management-session-signed-projection/0.2` | recompute `decision_digest` plus `management-approval-signed-projection/0.2` | owner-confirmed domains; schemas unregistered |
| 19 | canonical encoding | RFC 8785 under `cognitiveos.canonical-json/0.1` | same | existing standard reusable |
| 20 | replay resistance | current session ID/version/profile/domain/epochs/times; old/cross-profile binding rejected | atomic decision/proposal/request/session/profile/epoch consumption ledger; R3 pair consumed together | owner-confirmed; ledger unregistered |
| 21 | object version | exact signed `object_version`; every accepted activity/renewal/change creates CAS version and re-sign | exact signed `object_version`; change creates new decision version and re-sign | owner-confirmed |
| 22 | policy/revocation/negotiation epoch | policy and revocation signed; specification/registry/negotiation pins in binding record and rechecked | policy, revocation, specification, registry, and negotiation pins in binding record and rechecked | owner-confirmed; carriers unregistered |
| 23 | ActorChain/principal | human principal and ActorChain digest signed | approver principal and ActorChain digest signed | resolution/current authorization separate |
| 24 | proposal/session/challenge | later proposals bind the exact current verified session ID/version/digest | exact proposal/request/challenge plus session ID/version/digest; R3 decisions bind the same tuple | owner-confirmed rule; proposal/TARGET assets unregistered |
| 25 | verification order | G0-G5 before any management authorization | G0-G5 before approval is counted | owner-confirmed; section 10 |
| 26 | verification receipt | shared `SignatureVerificationReceipt` with session subject/profile facts | same schema with approval subject/profile facts | owner-confirmed SIG responsibility; schema unregistered |
| 27 | audit responsibility | SIG owns receipt facts/schema; AUDIT owns authoritative carrier/sequence/atomic slot | same | owner-confirmed split; AUDIT pending |
| 28 | stage-to-error mapping | exact existing codes plus owner-confirmed SIG codes in section 12 | same | design-closed; registry unmodified |
| 29 | critical extension/negotiation | `cognitiveos.ext.signature.management-session-authority` plus `cognitiveos.ext.authority-key-registry`, both critical | `cognitiveos.ext.signature.management-approval-authority` plus registry extension, both critical | owner-confirmed; extension assets unregistered |
| 30 | finite compatibility/migration | v0.1 string never upgrades in place; reauthenticate and reissue v0.2 session | v0.1 string never upgrades in place; rechallenge and create v0.2 decision | owner-confirmed |
| 31 | authorization non-expansion | valid signature proves bytes/key only | valid signature proves bytes/key only | mandatory invariant |

## 7. Canonical domains, projections, and exclusions

### 7.1 Common construction

For each object, processing is:

1. select the exact profile, allowed algorithm, signed schema, canonical
   profile, and negotiation epoch;
2. validate the complete received semantic object before projection;
3. form the explicitly declared content-digest projection and recompute the
   object digest;
4. form the closed object-specific signed binding record from profile-selected
   metadata and the subject projection, then validate its projection schema;
5. canonicalize the signed projection with RFC 8785;
6. recompute the envelope `signed_content_digest` with the object-specific
   signed-projection digest domain;
7. build the section 12 input using the object-specific signature domain and
   exact algorithm identifier;
8. resolve and validate the key/trust status, then verify signature bytes.

No implementation may sign a display digest, pretty JSON, source/transport
bytes, an open payload, or implicitly reparsed content. No application SHA-256
or other prehash is added before pure Ed25519.

### 7.2 Session projections

Owner-confirmed session content-digest domain:
`management-session-content/0.2`.

The owner-confirmed `session_digest` projection includes every schema-known session field
except exactly:

- `/session_digest`;
- `/authority_signature`.

The owner-confirmed session subject projection includes every schema-known field,
including `/session_digest` and all present optional fields, except exactly:

- `/authority_signature`.

The owner-confirmed session signed projection is a closed binding record with:

- profile asset ID, SemVer, and digest;
- algorithm, key ID, signed domain, signed schema digest, and canonical profile;
- exact negotiation epoch ID/digest and specification-set digest;
- the complete session subject projection above.

Its cross-check digest domain is
`management-session-signed-projection/0.2`. The record therefore binds schema
version, session ID, object version, management domain, session authority,
human principal, ActorChain digest, authentication/activity context, all scope
bounds, risk ceiling, policy version, revocation epoch, issue/activity/expiry
facts, lifecycle state, optional step-up/isolation facts, the recomputed
session digest, and the exact v0.2 negotiation/profile/key context.

Any unknown field is rejected by the selected closed schema. A missing optional
field and a present field are distinct semantic values; defaults are not
inserted. Renewal, last-activity change, scope/risk/policy/revocation/state
change, expiry change, or any other signed-field change requires a new object
version, recomputed digest, and new signature.

### 7.3 Approval projections

Owner-confirmed approval content-digest domain:
`management-approval-decision-content/0.2`.

The owner-confirmed `decision_digest` projection includes every schema-known approval
decision field except exactly:

- `/decision_digest`;
- `/authority_signature`.

The owner-confirmed approval subject projection includes every schema-known field,
including `/decision_digest` and all present conditional/optional fields,
except exactly:

- `/authority_signature`.

The owner-confirmed approval signed projection is a closed binding record with:

- profile asset ID, SemVer, and digest;
- algorithm, key ID, signed domain, signed schema digest, and canonical profile;
- exact negotiation epoch ID/digest and specification-set digest;
- current revocation epoch used for the decision;
- exact session ref, object version, and session digest;
- exact approval-request ref, request schema digest, and request content digest;
- the complete approval subject projection above.

Its cross-check digest domain is
`management-approval-signed-projection/0.2`. The record binds decision
identity/version, proposal ref/digest, request ref and single-use fact when
present, exact session version/digest, decision, deciding authority, approver
principal/ActorChain, independence and step-up facts when present, policy,
risk, revocation and negotiation epochs, challenge, safe reason codes,
decision/expiry times, the recomputed decision digest, and the exact
profile/algorithm/key/schema context.

For an `approve` decision, the owner-confirmed rules in section 9 close the
design-level request/challenge semantics for R1, R2, and R3. The current v0.1
R1 conditional remains insufficient as a machine carrier. Future registration
must materialize the confirmed cross-tier shapes, and proposal digest closure
must prove that the exact target/profile/parameters digest is covered.

### 7.4 Signed schema identity

The current v0.1 schemas cannot be used as the final v0.2 signed schema because
their `authority_signature` member is only a string and they do not register
the detached envelope, external epoch/request/session bindings, or closed
binding-record projections. Future registration must create new immutable
schema/profile identities and digests that materialize the owner-confirmed
binding records. This design does not assign those digests or mutate the current
schemas.

## 8. Session-specific business verification

After cryptographic verification, the authority must still verify:

- the `session_authority` is current and authorized to issue this management
  domain and scope;
- the signing key owner is bound to that authority and has the session-signing
  usage at verification time;
- session ID/version and signed digest identify the exact current version;
- human principal and resolved ActorChain match the authenticated peer;
- authentication and activity contexts are current and authorized;
- domains/actions/resources cover the requested action and target;
- the action risk does not exceed the signed ceiling;
- policy version and revocation epoch equal the current authority values;
- issued/last-activity/idle/absolute-expiry facts admit the current time;
- state is current `active`; pending/expired/revoked/closed never authorizes;
- negotiation epoch and profile triple are current and critical semantics were
  preserved.

Initial issuance uses `object_version = 1`. Every authority-accepted activity
update changes signed `last_activity_at`, performs CAS, increments object
version, recomputes digests, and re-signs. Authorization and commit must observe
the same exact current session version; an intervening change fails closed.

Ordinary renewal may update activity/idle state only within the original
absolute expiry and cannot expand scope or risk. Scope/risk attenuation may keep
the session ID but requires a new CAS version and signature. Scope/risk
expansion, absolute-expiry extension, or authority/domain change requires fresh
authentication and a new session ID. Revocation or closure immediately commits
a new signed version and invalidates every older version. Pending Effects retain
their original idempotency, fencing, and reconciliation duties and are not
auto-committed.

Reconnect establishes a new authenticated negotiation/session binding and a
new session; it does not restore an old bearer or session authority. A session
signed by a retiring key is usable only until the earlier of the object's own
expiry and the 24-hour rotation overlap. Key revocation is immediate. Old
object, policy, revocation, key-status, or negotiation versions fail closed.

Session signature verification must complete before management authorization,
proposal authorization, Effect creation, dispatch, state mutation, or commit.
A valid signature does not prove the session is current, the key is currently
authorized, scope covers the operation, risk is sufficient, target authority
exists, or approval has been satisfied.

## 9. Approval-specific business verification

After cryptographic verification, the authority must still verify:

- decision ID/version and digest are exact and current;
- proposal ref resolves to the exact proposal digest;
- proposal digest closes the exact operation, target/profile, parameter schema
  and digest, expected versions, idempotency key, session, policy, risk, and
  approval requirements in the future registered proposal profile;
- request ref and challenge digest resolve to the OS-issued, unexpired,
  single-use challenge for this proposal/session and confirmation surface;
- decision is exactly `approve` and is not expired;
- deciding authority is current and authorized for the risk tier;
- key owner has approval-signing usage and is independent from the proposer and
  proposing workload/ActorChain;
- approver principal and ActorChain are authenticated and current;
- policy version, risk class, step-up method, and R1/R2/R3 completion surface
  satisfy current policy;
- the referenced session remains valid and covers the exact action/target;
- the approval has not been used for another proposal, session, profile,
  target, parameters digest, object version, or negotiation epoch.

Every `approve` binds the OS-issued request, proposal digest, challenge digest,
and exact session ID/version/digest and is single-use. Principal, ActorChain,
key owner, and authority delegation are all checked; a boolean independence
claim is not proof.

- R1 permits `chat_structured`, `trusted_surface`, or `dual_independent`, uses an
  OS-held channel identity, and requires an approver independent from the
  proposer/workload.
- R2 permits only `trusted_surface` or `dual_independent`, requires policy-
  approved step-up, and requires one independent signed decision.
- R3 permits only `dual_independent` and requires two signed decisions whose
  principals, ActorChains, and approval leaf keys are pairwise distinct. Both
  decisions bind the same proposal/request/session/profile tuple and are
  consumed atomically.
- R0 `policy_auto` is not human approval. If an R0 decision object is emitted,
  it is an authority-signed policy decision and never counts as R1-R3 approval.

For every tier, `decided_at < expires_at <= request.expires_at`; the finite
request lifetime comes from the digest-pinned approval policy. Any expired
request/decision, invalid signature, inconsistent R3 pair, natural-language
message, reusable token, old proposal digest, proposer-controlled key, or merely
schema-valid decision is rejected.

## 10. Verification order and fail-closed behavior

The refined stages are:

- G0: framing, channel, and authenticated peer;
- G1: negotiation epoch, specification set, and signature critical extension;
- G2: signature profile, algorithm, signed schema identity, and domain;
- G3: canonical projection, object/content digests, signature encoding, and
  signature bytes;
- G4: deterministic key resolution, trust root, key status, key usage, and
  signer authority;
- G5: session/approval plus capability, risk, target, and independence business
  authorization;
- G6: verification receipt, authoritative audit, and authority commit.

Rules:

1. profile, algorithm, schema, domain, and resolver selection finish before
   business authorization;
2. canonicalization, digest, encoding, or signature failure terminates before
   authorization, Effect, dispatch, mutation, or commit;
3. signature success is followed by full current session, approval, capability,
   risk, target, policy, revocation, and epoch checks;
4. critical-extension selection or a new epoch triggers authorization
   revalidation;
5. a verification receipt proves only which checks ran and their result; it is
   neither authorization nor completion proof;
6. receipt/audit persistence or authority commit failure cannot report success.

### 10.1 Critical signature extensions

The owner-confirmed critical extension IDs are:

- `cognitiveos.ext.signature.management-session-authority`;
- `cognitiveos.ext.signature.management-approval-authority`;
- `cognitiveos.ext.authority-key-registry`.

All are `critical: true` and receive complete SemVer/digest identities during
machine registration. A new negotiation epoch pins the applicable signature
profile, signed schema, registry manifest, algorithm set, and extensions. An
old epoch cannot enable them. Unknown, missing, mismatched, or stripped
critical semantics fail before payload/business authorization.

### 10.2 Replay and atomic consumption

A current session is not a single-use token and may authorize multiple in-scope
operations. Use of an old/revoked session version or cross-profile/epoch
signature is replay/stale binding and is rejected.

Approval consumption uses an authoritative CAS ledger keyed by at least
decision ID/version, proposal digest, request digest, session ID/version/digest,
approval-profile digest, and negotiation epoch. An R3 pair is consumed as one
atomic approval set. Repeated delivery of the same idempotency request may
return the already committed equivalent result, but cannot redispatch, create a
second Effect, or consume approval twice. Ordinary repeat cryptographic
verification is not itself replay; reuse of consumable authority is.

Ledger, receipt, audit, or authority-commit unavailability fails with the exact
registered persistence semantics and cannot report success.

## 11. Verification receipt and audit responsibility

The owner confirms that future SIG machine registration owns one shared
`SignatureVerificationReceipt` schema. It supports both subject types while
retaining their exact object/profile/domain identities and must include or
strongly bind:

- receipt ID/version and verification time;
- object kind, ID, object version, and object digest;
- profile ID/version/digest, signed schema digest, signed domain, and algorithm;
- key ID, resolved key descriptor/version/digest, resolver profile/version, and
  trust-root identity/digest;
- key owner, usage, status, status-source version/freshness, rotation generation,
  and revocation epoch used;
- recomputed content and signed-content digests;
- negotiation epoch and specification-set digest;
- result, earliest failed stage, and exact registered error when available;
- correlation/causation references and receipt digest.

The receipt is a closed tagged union with `receipt_scope` equal to either
`subject_verified` or `input_rejected`. `subject_verified` carries every
applicable subject/profile/key fact above. `input_rejected` is mandatory for a
failure before the object, profile, signed schema, or key has become trusted; it
must not copy unvalidated semantic identity fields. It instead binds the
authenticated channel/peer context, negotiation epoch when established, exact
earliest failed stage, registered safe error when available, and a domain-
separated digest of the received envelope bytes or validated framing value. Any
fact not established before the failed stage is absent, never guessed, defaulted,
or copied from untrusted input as authority.

Successful and failed verification attempts both create a safe receipt. It must
not contain private key material, secret content, signature material usable for
replay, or protected subject content. The receipt is not independently signed;
its integrity comes from its registered digest plus authoritative persistence
and the later AUDIT contract. Receipt persistence completes before business
authorization; persistence failure denies the request.

A receipt is evidence of a verification attempt, not proof of session validity,
approval sufficiency, authorization, dispatch, commit, or completion.

SIG owns the verification-fact profile and its handoff slot. AUDIT owns the
authoritative audit carrier/profile/persistence port, sequence/high-watermark,
retention, sensitivity, export, tamper resistance, and atomic commit. That slot
remains pending until AUDIT design and later machine registration. An Event open
payload, AKP `audit_ref`, transition record, outbox row, or private database row
cannot fill it.

## 12. Stage-to-registered-error responsibility

Existing codes are reused only when their registered descriptions exactly
match. The owner confirms the future SIG-specific codes below. They remain
unregistered proposals until the later machine-registration batch edits the
error registry and generated bindings.

### 12.1 Exact existing-code reuse

| Failure | Stage | Existing registered code | Exact boundary |
|---|---|---|---|
| unknown signature critical extension | G1 | `CRITICAL_EXTENSION_UNKNOWN` | extension unknown and critical |
| stripped/lossy signature extension | G1 | `PROTOCOL_MAPPING_INCOMPLETE` | gateway cannot preserve critical semantics |
| profile/specification version outside finite window | G1/G2 | `VERSION_UNSUPPORTED` | unsupported protocol/specification window, not arbitrary algorithm failure |
| pinned protocol payload schema digest mismatch | G1/G2 | `PROTOCOL_SCHEMA_DIGEST_MISMATCH` | payload schema only; signed projection schema uses a new SIG code |
| schema-invalid future envelope/projection | G2/G3 | `SCHEMA_MISMATCH` | only after the future schema is registered |
| recomputed declared content digest mismatch | G3 | `DIGEST_MISMATCH` | declared registered digest only |
| current session/approval object version differs | G5 | `STATE_CONFLICT` | authoritative expected-version/CAS mismatch |
| self-signing or proposer-entangled approval | G4/G5 | `MANAGEMENT_SELF_AUTHORIZATION_DENIED` | self-authorization only |
| missing independent approval | G5 | `MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED` | required independent decision absent/invalid |
| expired/revoked session | G5 | `MANAGEMENT_SESSION_EXPIRED` / `MANAGEMENT_SESSION_REVOKED` | session business state |
| session domain/action/resource outside scope | G5 | `MANAGEMENT_SCOPE_MISMATCH` | exact scope mismatch |
| required step-up absent | G5 | `MANAGEMENT_STEP_UP_REQUIRED` | exact challenge condition |
| state/Event/authority commit unavailable | G6 | `STATE_STORE_UNAVAILABLE` | registered authoritative state/Event persistence only |
| authoritative AUDIT port persistence unavailable | G6 | future `AUDIT_STORE_UNAVAILABLE` | AUDIT-owned persistence before audit commit/receipt; unregistered until the AUDIT error batch |

### 12.2 Owner-confirmed future SIG errors

Only `SIGNATURE_KEY_RESOLUTION_FAILED` is `retryable: true`. Every other code
below is `retryable: false`; recovery requires corrected input, reissuance, key
or policy repair, or a new negotiation rather than automatic replay of the same
request.

| Future error code | Stage | Exact meaning | Retryable |
|---|---|---|---|
| `SIGNATURE_PROFILE_UNKNOWN` | G2 | profile asset ID is unknown in the selected specification set | false |
| `SIGNATURE_ALGORITHM_UNSUPPORTED` | G2 | algorithm is not exact `Ed25519` in the selected profile | false |
| `SIGNATURE_ALGORITHM_DOWNGRADE_DENIED` | G1/G2 | peer/message attempted fallback, aliasing, or weaker algorithm selection | false |
| `SIGNATURE_ENCODING_INVALID` | G3 | public key/signature encoding or raw length violates the profile | false |
| `SIGNATURE_DOMAIN_MISMATCH` | G2/G3 | signed domain differs from the object-specific profile domain | false |
| `SIGNATURE_SIGNED_SCHEMA_MISMATCH` | G2 | signed projection schema identity/digest differs from the profile pin | false |
| `SIGNATURE_PROJECTION_MISMATCH` | G3 | signed projection or declared exclusion set differs from the registered profile | false |
| `SIGNATURE_VERIFICATION_FAILED` | G3/G4 | strict pure-Ed25519 verification fails | false |
| `SIGNATURE_KEY_UNKNOWN` | G4 | strong key ref is absent from the selected registry manifest | false |
| `SIGNATURE_KEY_RESOLUTION_FAILED` | G4 | authoritative registry/descriptor resolution is temporarily unavailable | true |
| `SIGNATURE_KEY_REVOKED` | G4 | current descriptor/delegation status is revoked | false |
| `SIGNATURE_KEY_EXPIRED` | G4 | key validity interval has ended | false |
| `SIGNATURE_KEY_ROTATED_OUT` | G4 | predecessor is outside the 24-hour overlap or signed after successor activation | false |
| `SIGNATURE_KEY_USAGE_DENIED` | G4 | key owner/usage/authority bounds do not authorize this subject profile | false |
| `SIGNATURE_TRUST_ROOT_MISMATCH` | G4 | registry manifest/delegation does not chain to the pinned platform root | false |
| `SIGNATURE_REPLAY_DETECTED` | G3/G5 | consumed approval, old session version, or cross-profile/epoch signature was reused | false |
| `SIGNATURE_NEGOTIATION_EPOCH_MISMATCH` | G1/G5 | object/extension/profile/registry binding differs from the current epoch | false |
| `SIGNATURE_REVOCATION_EPOCH_MISMATCH` | G4/G5 | signed or resolved revocation epoch differs from current authority state | false |
| `SIGNATURE_POLICY_VERSION_MISMATCH` | G5 | signed approval/session policy version is not current | false |

## 13. Planned negative matrix

Every case is **planned/not executed**. Future vectors must be new assets and
must not modify any existing `expected` value.

Common rejection oracle:

```text
dispatches = 0
effects_created = 0
business_state_mutations = 0
commits = 0
success_receipts = 0
```

| # | Planned negative | Earliest stage | Status |
|---|---|---|---|
| 1 | missing signature | G2 | planned/not executed |
| 2 | unknown signature profile | G2 | planned/not executed |
| 3 | profile version/digest mismatch | G1/G2 | planned/not executed |
| 4 | unsupported algorithm | G2 | planned/not executed |
| 5 | algorithm downgrade | G1/G2 | planned/not executed |
| 6 | malformed signature encoding | G3 | planned/not executed |
| 7 | unknown key | G4 | planned/not executed |
| 8 | key-resolution unavailable | G4 | planned/not executed |
| 9 | revoked key | G4 | planned/not executed |
| 10 | expired/rotated key outside compatibility window | G4 | planned/not executed |
| 11 | key not authorized for session signing | G4 | planned/not executed |
| 12 | key not authorized for approval signing | G4 | planned/not executed |
| 13 | trust-root mismatch | G4 | planned/not executed |
| 14 | wrong signature domain | G2/G3 | planned/not executed |
| 15 | wrong signed schema digest | G2 | planned/not executed |
| 16 | wrong signed projection | G3 | planned/not executed |
| 17 | undeclared excluded path | G3 | planned/not executed |
| 18 | signed content digest mismatch | G3 | planned/not executed |
| 19 | invalid signature | G3/G4 | planned/not executed |
| 20 | session signature replayed as approval | G2/G3 | planned/not executed |
| 21 | approval signature replayed across proposal/session | G5 | planned/not executed |
| 22 | session object version changed after signing | G5 | planned/not executed |
| 23 | session expired/revoked after signing | G5 | planned/not executed |
| 24 | proposal digest mismatch | G5 | planned/not executed |
| 25 | challenge digest mismatch | G5 | planned/not executed |
| 26 | approval expired | G5 | planned/not executed |
| 27 | single-use approval replay | G5 | planned/not executed |
| 28 | approver not independent from proposer | G4/G5 | planned/not executed |
| 29 | ActorChain mismatch | G5 | planned/not executed |
| 30 | policy version mismatch | G5 | planned/not executed |
| 31 | revocation epoch mismatch | G4/G5 | planned/not executed |
| 32 | old negotiation epoch | G1 | planned/not executed |
| 33 | signature critical extension stripped | G1 | planned/not executed |
| 34 | verification performed after authorization | G5 | planned/not executed |
| 35 | verification performed after Effect creation | G5 | planned/not executed |
| 36 | rejection followed by dispatch | G5 | planned/not executed |
| 37 | rejection followed by Effect | G5 | planned/not executed |
| 38 | rejection followed by state mutation | G5 | planned/not executed |
| 39 | rejection followed by commit | G6 | planned/not executed |
| 40 | rejection followed by success receipt | G6 | planned/not executed |
| 41 | G1/G2 rejection receipt copies untrusted object/profile/key identity as authoritative fact | G1/G2 | planned/not executed |
| 42 | AUDIT-port persistence failure is reported as `STATE_STORE_UNAVAILABLE` or followed by visible success | G6 | planned/not executed |

Additional positives and negatives must cover exact canonical bytes, both
object-specific domains, both projection schemas, algorithm identifiers,
key-usage separation, rotation overlap boundaries, resolver staleness, trust
delegation, and receipt redaction. They remain future registration/CFR work.

## 14. Compatibility and migration

SIG inherits the finite OPS window and adds these requirements:

- a native v0.2 epoch selects exact specification, operation, signature
  profile, signed schema, Ed25519-only algorithm set, authority-key registry
  manifest, platform root/delegation, and critical-extension digests;
- the proposed native point is exact `0.2.0-draft.1`; any temporary adapter may
  cover only exact `0.2.0-draft.1` and `0.2.0-draft.2` and is removed at
  `0.2.0-draft.3`; no adapter exists now;
- a v0.1 `authority_signature` string is migration input only. It is not parsed
  as a detached envelope, granted an algorithm, or accepted under v0.2;
- a v0.1 session must be reauthenticated and reissued as a new v0.2 session
  version; a v0.1 approval must be rechallenged and decided as a new v0.2
  decision;
- old epochs cannot silently enable a signature profile or new trust root;
- reconnect does not restore old session or approval authority;
- missing key/trust/projection facts cause rejection or quarantine, never
  defaults to a platform, tenant, process, caller, or cached key;
- rotation accepts a predecessor only for objects signed before successor
  activation and only within the owner-confirmed 24-hour overlap; revocation
  has no migration grace period;
- in-flight Effects retain their original session, approval, idempotency,
  fencing, reconciliation, and audit obligations;
- algorithm, key usage, trust-root, projection, exclusion, domain, status, or
  error changes are breaking and receive new SemVer/digest and migration notes.

No profile, adapter, resolver, key registry, trust root, receipt, or migration
implementation exists as a result of this decision.

## 15. Authorization non-expansion proof

Effective authorization remains the intersection:

```text
operation membership
∩ negotiation epoch and critical extensions
∩ authenticated peer and channel
∩ valid signature profile and bytes
∩ current signer/key/trust authorization
∩ current session scope and state
∩ capability bounds
∩ risk ceiling and approval policy
∩ approval independence/freshness/single use
∩ target authority
```

The proof obligations are:

- signature validity cannot create operation membership, session scope,
  capability, risk allowance, approval, target authority, or completion;
- key possession cannot create signer authority or key usage;
- trust-root validity cannot create current key status or business authority;
- session and approval profiles have disjoint domains/usages and cannot replay;
- an operation name, URI, open JSON, boolean, receipt, vector fixture, private
  row/DTO/helper, or schema-valid object grants nothing;
- extension/profile selection triggers authorization revalidation;
- unknown, stale, mismatched, revoked, unavailable, or indeterminate inputs deny
  before dispatch, Effect, mutation, commit, or success receipt;
- audit/receipt success cannot replace authority commit or acceptance proof.

## 16. Owners, blockers, and evidence limitation

### Owner/reviewer roles

- decision owner: repository owner;
- security/cryptography reviewer: algorithm set, encoding, domain, projection,
  downgrade, key usage, and negative matrix;
- identity/KMS reviewer: key ID, resolver, trust roots, rotation, revocation,
  cache/freshness, and outage behavior;
- management-session authority reviewer: issuance, renewal, revocation, and
  business gate;
- approval authority reviewer: R1/R2/R3 challenge, independence, and single-use
  closure;
- OPS/TARGET reviewers: proposal/target/profile/parameters digest closure;
- AUDIT reviewer: verification receipt carrier and atomic persistence slot.

The repository owner confirmed the technical selections in sections 5-12 on
2026-07-22 through the active governance session. This records owner decisions;
it is not a GitHub review of the resulting new PR head and does not claim an
independent security/cryptography review. No review exception from PR #50, #51,
or #52 applies to this PR.

### Blocking facts

- independent security/cryptography review of the owner-confirmed selections;
- future v0.2 envelope, session, approval, binding-record, key descriptor,
  registry manifest/delegation, receipt, and replay-ledger schema identities and
  exact digests;
- proposal and approval-request digest/domain/projection closure;
- machine registration of the three critical extensions, profiles, key usages,
  registry manifest certification, and negotiation-epoch carrier;
- registration of the 19 owner-confirmed SIG errors and generated bindings;
- authoritative AUDIT carrier/sequence/atomic persistence slot;
- TARGET profile and exact proposal-to-target/parameters closure.

### Evidence limitation

This batch is a static source audit and docs-only design. Repository checks,
builds, and ordinary unit tests validate integrity only. They do not execute a
new behavior vector, perform cryptographic verification, register a profile,
prove key/trust infrastructure, implement Configuration Authority, or support a
Profile claim.

## 17. GO/NO-GO and downstream order

- `GO`: this docs-only SIG packet contains owner-confirmed technical selections
  and is materialized for independent security review.
- `NO-GO`: SIG machine registration, TARGET/OPS/AUDIT registration,
  Configuration Authority implementation, behavior execution, and Profile
  claim.

The algorithm/key/trust/domain/projection/error choices are closed at the
docs-only design level. Both profiles remain unregistered and unusable until
independent security review, exact machine registration, generated bindings,
and future negative vectors land without modifying old `expected` values.

Downstream order remains:

1. owner/security review and merge of this SIG design packet;
2. AUDIT design;
3. independent OPS/TARGET/SIG/AUDIT machine-registration batches;
4. independent CA-0 re-review;
5. explicit CA-0 GO;
6. only then, implementation;
7. Management CFR after real implementation exists.

SIG merge does not register Ed25519, a key/trust profile, signature profile,
extension, receipt, or error; unblock a configure operation; approve target
authority; or authorize implementation.

Preserved state:

- 273 requirements, 55 errors, 61 schemas, 84 vectors;
- 59 pass, 25 not-run, self-check 40;
- matrix non-empty implementation count 70;
- Profile implemented = 0;
- D-016 open;
- D-022 blocking;
- CA-1 through CA-8 blocked;
- all three configure candidates blocked;
- machine contracts unregistered;
- Configuration Authority implementation not provided;
- new behavior not executed.
