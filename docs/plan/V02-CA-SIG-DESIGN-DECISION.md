# V02-CA-SIG-01 Session and Approval Signature Design Decision

- Decision ID: `V02-CA-SIG-01`
- Date: 2026-07-22
- Status: **materialized for owner/security review; both profiles blocked**
- Baseline: `origin/main@42d609b2f49e2db641f46aa99b6cc9a538a7f4fd`
  (PR #52 merge; main CI run `29922529556` Ubuntu/Windows success)
- Inputs: [V02-CA-GOV-00](V02-CONFIGURATION-AUTHORITY-NORMATIVE-SURFACE-AUTHORIZATION.md),
  [V02-CA-OPS-01](V02-CA-OPS-DESIGN-DECISION.md), and
  [V02-CA-TARGET-01](V02-CA-TARGET-DESIGN-DECISION.md)
- Structural governance:
  [ADR-0012](../adr/0012-v02-detached-signature-profile-governance.md)
- Classification: docs-only structural design; no machine registration
- Proposed profile family/version: `cognitiveos.detached-signature-envelope/0.2`,
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
The proposed family therefore has two object-specific profiles:

- `cognitiveos.signature.management-session-authority/0.2`, domain
  `management-session-authority/0.2`;
- `cognitiveos.signature.management-approval-authority/0.2`, domain
  `management-approval-authority/0.2`.

Both profiles remain `blocked`. Current machine facts do not uniquely select an
allowed algorithm set, key infrastructure, trust roots, or complete
stage-to-error mapping. The profile IDs, versions, domains, envelope fields,
and projections below are design proposals for review, not registered assets.

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

### 4.1 Proposed reusable envelope

A future machine-registration PR may propose a closed detached envelope with
these logical fields:

| Field | Proposed obligation |
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

## 5. Algorithm, key, and trust alternatives

No current digest-pinned asset uniquely selects among these bounded options.
Owner and security review must choose one closed option or reject the family.

### 5.1 Algorithm alternatives

| Alternative | Proposed exact shape | Security/operational difference | Status |
|---|---|---|---|
| A: Ed25519 only | identifier `Ed25519`; 64 raw signature bytes; unpadded base64url | simple fixed-width encoding and one algorithm; requires compatible authority/KMS support and controlled public-key registration | unresolved |
| B: P-256 ECDSA only | identifier `ECDSA-P256-SHA256`; fixed 64-byte `r || s`, low-S enforced; unpadded base64url | broader enterprise/HSM availability; requires deterministic encoding and anti-malleability rules | unresolved |
| C: explicit two-algorithm set | both exact identifiers, with no aliases and per-key algorithm pin | migration flexibility but larger downgrade, interop, and testing surface; selection must be profile/epoch-pinned | unresolved; not preferred without a demonstrated compatibility need |

DER/raw guessing, algorithm aliases, `none`, caller-selected algorithms, and
unknown-algorithm fallback are forbidden. Algorithm selection or removal is a
profile-breaking change with new SemVer/digest and migration.

### 5.2 Key-resolution and trust alternatives

| Alternative | Deterministic resolution | Trust/rotation model | Status |
|---|---|---|---|
| K1: governed CognitiveOS authority-key registry | `key_id` resolves to one strong, digest-pinned key descriptor under the session/approval authority | platform root authorizes key owner, usage, validity interval, status, successor, revocation epoch, and resolver version | unresolved |
| K2: external KMS/PKI resolver profile | `key_id` plus selected resolver profile maps to exactly one external key/version; response is authenticated and pinned | trust-anchor set, certificate/key policy, status source, freshness, outage behavior, and rotation overlap are fixed by the profile/epoch | unresolved |
| K3: platform root with tenant-delegated signing authority | platform root signs a bounded delegation to a tenant authority; chain is digest-pinned and monotone | supports tenant authority but adds delegation, chain, cross-tenant isolation, and revocation complexity | unresolved; only if tenant-scoped session authority is required |

Caller-provided resolver URLs, trust anchors, public keys, cache entries, or
private database rows are forbidden authority inputs. Resolver unavailability,
ambiguous results, stale status, unknown keys, or indeterminate trust fail
closed. Rotation overlap must be finite and explicit. Revocation applies to
future decisions immediately; verification records the status source/version
and time used.

### 5.3 Owner/security decisions required

Review must uniquely decide:

1. the closed algorithm set and exact identifiers/encodings;
2. key descriptor and `key_id` shape;
3. the deterministic resolver and freshness/cache policy;
4. platform and tenant trust-root boundaries;
5. session-signing and approval-signing key usages;
6. rotation overlap and the meaning of rotated versus expired;
7. revocation source, epoch, propagation, and historical-verification policy;
8. exact new error codes for unresolved failure classes.

Until all eight close, neither profile may be registered or selected.

## 6. Complete signature binding matrix

Every row is mandatory. “Proposed” means docs-only design, not machine status.

| # | Binding | Session profile | Approval profile | Status |
|---|---|---|---|---|
| 1 | profile identity | `cognitiveos.signature.management-session-authority/0.2` | `cognitiveos.signature.management-approval-authority/0.2` | proposed, unregistered |
| 2 | profile SemVer/digest | `0.2.0-draft.1`; unresolved digest | same version; independent unresolved digest | blocked |
| 3 | allowed algorithm set | alternatives A/B/C | alternatives A/B/C; may be narrower | owner/security decision |
| 4 | algorithm identifier | exact selected ASCII identifier | exact selected ASCII identifier | unresolved |
| 5 | signature encoding | fixed by selected alternative | fixed by selected alternative | unresolved |
| 6 | key ID shape | authority-key descriptor or resolver-bound ID | approval-key descriptor or resolver-bound ID | unresolved |
| 7 | deterministic key resolution | selected K1/K2/K3 resolver | selected K1/K2/K3 resolver | unresolved |
| 8 | trust-root binding | session authority root/delegation | deciding authority root/delegation | unresolved |
| 9 | key owner/authority | must be authorized by current `session_authority` | must be authorized by current `deciding_authority` | proposed rule; registry absent |
| 10 | key usage/purpose | exact `management-session-signing` usage | exact `management-approval-signing` usage | proposed; distinct usages |
| 11 | rotation | finite overlap, explicit successor/version | finite overlap, explicit successor/version | unresolved infrastructure |
| 12 | revocation | current status/epoch before use | current status/epoch before use | unresolved infrastructure |
| 13 | verification-time status | record source/version/freshness/time | same, independently | proposed receipt obligation |
| 14 | signature domain | `management-session-authority/0.2` | `management-approval-authority/0.2` | proposed, independent |
| 15 | signed schema identity | future v0.2 session projection schema/digest | future v0.2 approval projection schema/digest | unregistered |
| 16 | signed projection | closed session binding record containing profile/algorithm/key/schema/epoch metadata plus complete subject except `/authority_signature` | closed approval binding record additionally containing session/request/revocation bindings plus complete subject except `/authority_signature` | proposed; section 7 |
| 17 | excluded paths | subject source excludes only `/authority_signature`; binding record has no signature-bytes member | subject source excludes only `/authority_signature`; binding record has no signature-bytes member | proposed; digest exclusions are separate |
| 18 | content-digest cross-check | recompute `session_digest` plus signed-projection digest domain `management-session-signed-projection/0.2` | recompute `decision_digest` plus `management-approval-signed-projection/0.2` | domains/schemas unregistered |
| 19 | canonical encoding | RFC 8785 under `cognitiveos.canonical-json/0.1` | same | existing standard reusable |
| 20 | replay resistance | session ID/version/profile/domain/epochs/times | decision ID/version/proposal/request/challenge/session/profile/domain/times/single-use | proposed |
| 21 | object version | exact signed `object_version`; renewal/change creates new version and re-sign | exact signed `object_version`; change creates new decision version and re-sign | proposed |
| 22 | policy/revocation/negotiation epoch | policy and revocation are signed; negotiation epoch externally pinned and rechecked | policy signed; revocation and negotiation bindings require future profile/envelope/request closure | partial; blocked |
| 23 | ActorChain/principal | human principal and ActorChain digest signed | approver principal and ActorChain digest signed | resolution/current authorization separate |
| 24 | proposal/session/challenge | not part of session issuance object; later proposals bind the verified session version/digest | proposal digest, session ref, request ref, challenge digest, and single-use semantics | proposal/request digest closure blocked |
| 25 | verification order | G0-G5 before any management authorization | G0-G5 before approval is counted | proposed; section 10 |
| 26 | verification receipt | session-specific receipt slot | approval-specific receipt slot | logical design only; carrier unregistered |
| 27 | audit responsibility | SIG supplies verification facts; AUDIT owns authoritative carrier/atomic slot | same | AUDIT blocker |
| 28 | stage-to-error mapping | exact existing session business errors plus unresolved crypto errors | exact approval/self-authorization errors plus unresolved crypto errors | incomplete |
| 29 | critical extension/negotiation | session profile triple and algorithm set are critical pins | approval profile triple and algorithm set are critical pins | extension assets unregistered |
| 30 | finite compatibility/migration | v0.1 string never upgrades in place; reissue v0.2 session | v0.1 string never upgrades in place; create a new v0.2 decision | proposed |
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
is added before the selected algorithm unless the registered algorithm profile
explicitly requires it.

### 7.2 Session projections

Proposed session content-digest domain:
`management-session-content/0.2`.

Proposed `session_digest` projection includes every schema-known session field
except exactly:

- `/session_digest`;
- `/authority_signature`.

Proposed session subject projection includes every schema-known field,
including `/session_digest` and all present optional fields, except exactly:

- `/authority_signature`.

The proposed session signed projection is a closed binding record with:

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

Proposed approval content-digest domain:
`management-approval-decision-content/0.2`.

Proposed `decision_digest` projection includes every schema-known approval
decision field except exactly:

- `/decision_digest`;
- `/authority_signature`.

Proposed approval subject projection includes every schema-known field,
including `/decision_digest` and all present conditional/optional fields,
except exactly:

- `/authority_signature`.

The proposed approval signed projection is a closed binding record with:

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

For an `approve` decision, future profile registration must close the exact
request/challenge rule for R1, R2, and R3. The current R1 conditional is not a
complete cross-tier signature profile. Proposal digest closure must prove that
the exact target/profile/parameters digest is covered; otherwise the approval
profile remains blocked.

### 7.4 Signed schema identity

The current v0.1 schemas cannot be used as the final v0.2 signed schema because
their `authority_signature` member is only a string and they do not register
the detached envelope, external epoch/request/session bindings, or closed
binding-record projections. Future registration must create new immutable
schema/profile identities and digests and must decide where those binding facts
are carried. This design does not assign those digests or mutate the current
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

Renewal creates a new object version and signature. Revocation, closure, scope
change, risk change, policy/revocation advance, activity/expiry update, or key
rotation outside the finite overlap invalidates future use of the old version.
Reconnect establishes a new authenticated negotiation/session binding; it
does not restore an old bearer or session authority. Old object, policy,
revocation, key-status, or negotiation versions fail closed.

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

R1 structured chat confirmation remains one-shot and machine-bound. R2 requires
a policy-approved trusted surface. R3 requires the registered dual-independent
surface and principals. Natural language, a chat message, reusable approval
token, expired challenge, old proposal digest, proposer-controlled key, or a
schema-valid decision is never approval.

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

## 11. Verification receipt and audit responsibility

A future SIG verification receipt must include or strongly bind:

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

It must not contain private key material, secret content, or make a failed
verification replayable. A receipt is evidence of a verification attempt, not
proof of session validity, approval sufficiency, authorization, dispatch,
commit, or completion.

SIG owns the verification-fact profile and its handoff slot. AUDIT owns the
authoritative audit carrier/profile/persistence port, sequence/high-watermark,
retention, sensitivity, export, tamper resistance, and atomic commit. That slot
is unresolved until AUDIT design and later machine registration. An Event open
payload, AKP `audit_ref`, transition record, outbox row, or private database row
cannot fill it.

## 12. Stage-to-registered-error responsibility

Existing codes are reused only when their registered descriptions exactly
match. A later registration batch must add or decide exact codes for every
unresolved row; this docs-only batch does not edit the registry.

| Failure | Stage | Current exact mapping | Responsibility |
|---|---|---|---|
| unknown signature critical extension | G1 | `CRITICAL_EXTENSION_UNKNOWN` | exact when the extension is unknown and critical |
| stripped/lossy signature extension | G1 | `PROTOCOL_MAPPING_INCOMPLETE` | exact for lossy gateway mapping |
| profile/specification version outside finite window | G1/G2 | `VERSION_UNSUPPORTED` | exact only for unsupported protocol/spec-set major/window |
| payload/signed schema differs from protocol pin | G1/G2 | `PROTOCOL_SCHEMA_DIGEST_MISMATCH` | exact only when it is the pinned payload schema |
| schema-invalid future envelope/projection | G2/G3 | `SCHEMA_MISMATCH` | usable only after that schema is registered |
| recomputed declared content digest mismatch | G3 | `DIGEST_MISMATCH` | exact for the registered digest |
| self-signing or proposer-entangled approval | G4/G5 | `MANAGEMENT_SELF_AUTHORIZATION_DENIED` | exact self-authorization semantics only |
| missing independent approval | G5 | `MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED` | exact where current approval is absent/invalid for independence policy |
| expired/revoked session | G5 | `MANAGEMENT_SESSION_EXPIRED` / `MANAGEMENT_SESSION_REVOKED` | exact business state failures |
| session domain/action/resource outside scope | G5 | `MANAGEMENT_SCOPE_MISMATCH` | exact scope failure |
| required step-up absent | G5 | `MANAGEMENT_STEP_UP_REQUIRED` | exact challenge condition |
| receipt/audit/authority commit unavailable | G6 | `STATE_STORE_UNAVAILABLE` | exact for authoritative persistence failure; AUDIT-specific closure pending |
| unknown signature profile | G2 | none proven exact | SIG registration owner |
| unsupported algorithm inside an otherwise supported set | G2 | none proven exact | do not reinterpret `VERSION_UNSUPPORTED` |
| algorithm downgrade or profile/algorithm mismatch | G2 | none proven exact | SIG/negotiation owner |
| malformed signature byte encoding | G3 | none proven exact | schema code may cover shape only, not general cryptographic failure |
| wrong signature domain | G2/G3 | none proven exact | `DIGEST_MISMATCH` is not a generic domain error |
| wrong signed schema/projection/exclusion set | G2/G3 | none proven exact | profile/schema registration owner |
| unknown key or resolver unavailable/ambiguous | G4 | none proven exact | key infrastructure owner |
| revoked, expired, or rotated-out key | G4 | none proven exact | key status owner |
| unauthorized key usage/owner | G4 | none proven exact | signer authority owner |
| trust-root mismatch | G4 | none proven exact | trust policy owner |
| invalid cryptographic signature | G3/G4 | none proven exact | do not reinterpret `DIGEST_MISMATCH` or self-authorization |
| signature replay | G3/G5 | none proven exact | object-specific profile owner |
| session/approval object-version mismatch | G5 | no complete exact mapping | `STATE_CONFLICT` is exact only for registered authority CAS semantics |
| policy/revocation/negotiation epoch mismatch | G1/G5 | no complete exact mapping | session business codes do not cover every epoch mismatch |

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

Additional positives and negatives must cover exact canonical bytes, both
object-specific domains, both projection schemas, algorithm identifiers,
key-usage separation, rotation overlap boundaries, resolver staleness, trust
delegation, and receipt redaction. They remain future registration/CFR work.

## 14. Compatibility and migration

SIG inherits the finite OPS window and adds these requirements:

- a native v0.2 epoch selects exact specification, operation, signature
  profile, signed schema, algorithm-set, resolver, trust-root, and critical
  extension digests;
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

No SIG owner approval or GitHub review is claimed by this authoring batch. No
review exception from PR #50, #51, or #52 applies to this PR.

### Blocking facts

- selected closed algorithm set and exact encoding;
- key ID/descriptor, deterministic resolver, and trust roots;
- key ownership, usage, rotation, revocation, and verification-time status;
- future v0.2 session/approval/envelope/projection schema identities/digests;
- proposal and approval-request digest/domain/projection closure;
- negotiation extension/profile identities and epoch carrier;
- complete exact error registry mapping;
- verification receipt schema and authoritative AUDIT slot;
- TARGET profile and exact proposal-to-target/parameters closure.

### Evidence limitation

This batch is a static source audit and docs-only design. Repository checks,
builds, and ordinary unit tests validate integrity only. They do not execute a
new behavior vector, perform cryptographic verification, register a profile,
prove key/trust infrastructure, implement Configuration Authority, or support a
Profile claim.

## 17. GO/NO-GO and downstream order

- `GO`: this docs-only SIG packet is materialized for owner/security review.
- `NO-GO`: SIG machine registration, TARGET/OPS/AUDIT registration,
  Configuration Authority implementation, behavior execution, and Profile
  claim.

Both profiles remain blocked until all 31 matrix rows close, owner/security and
key/trust reviewers select exact alternatives, every unresolved error receives
an exact registered responsibility, machine assets are independently reviewed
and registered, and future negative vectors are added without modifying old
`expected` values.

Downstream order remains:

1. owner/security review and merge of this SIG design packet;
2. AUDIT design;
3. independent OPS/TARGET/SIG/AUDIT machine-registration batches;
4. independent CA-0 re-review;
5. explicit CA-0 GO;
6. only then, implementation;
7. Management CFR after real implementation exists.

SIG merge does not register a signature profile, approve an algorithm or trust
root, unblock a configure operation, approve target authority, or authorize
implementation.

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
