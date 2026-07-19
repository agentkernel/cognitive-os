# Authentication, Authorization and Capability Standard

- Standard ID: `cognitiveos.standard.authn-authz-capability/0.1`
- Version: v0.1 Draft
- Machine version: `0.1.0-draft.1`
- Status: Draft Normative Standard
- Date: 2026-07-20
- Machine assets: `specs/schemas/authorization-capability.schema.json`,
  `authorization-delegation.schema.json`, `principal.schema.json`,
  `membership.schema.json`, `actor-chain.schema.json`,
  `privileged-management-session.schema.json`
- Normative sources: RFC-0001 sections 5-6, 19; Core companion

## 1. Scope and normative language

RFC 2119/8174 language applies. This standard fixes the deterministic
decision order for authorization in the reference implementation. Owning
requirements include [REQ-AUTH-001], [REQ-CAP-001] through [REQ-CAP-005],
[REQ-GOBJ-AUTHDEL-001..003], [REQ-MGMT-SESSION-001..003],
[REQ-MGMT-AUTHZ-001], [REQ-MGMT-GATE-001], [REQ-MGMT-TRUST-001]. It registers
no new requirement.

## 2. Decision order

Authorization MUST be evaluated deterministically in this order, failing
closed at the first violated step:

1. Authenticate the principal and resolve the full ActorChain; identity is
   never inferred from natural language ([REQ-SHELL-CHANNEL-001]: language is
   not a capability).
2. Resolve tenant and membership; tenant match alone grants nothing
   ([REQ-GOBJ-TENANT-001]).
3. Intersect every applicable capability: the effective right is the
   intersection of chain links, never the union.
4. Apply explicit deny: an explicit deny beats any allow (default deny when
   no applicable grant exists).
5. Check lease validity (`AUTH_CAPABILITY_EXPIRED` when outside the lease;
   expired leases are never extended for lack of a reading, ADR-0005).
6. Check scope and purpose binding against the requested object and action
   (`CONTEXT_AUTH_DENIED` / `MANAGEMENT_SCOPE_MISMATCH` on mismatch).

## 3. Monotone attenuation and revocation

A derived capability or delegation MUST NOT expand any parent bound: scope,
actions, risk, budget, or lease (`AUTH_CAPABILITY_ATTENUATION_VIOLATION`,
[REQ-CAP-002], vector `capability-attenuation.json`). Revocation takes effect
against future decisions immediately: a decision made from cached material
after revocation epoch advance is a defect (vector
`context-revocation-cache-reuse.json`, [REQ-CAP-005]; cache binding in
`context-resolution-and-cache.md`). Long-running dispatch paths MUST
re-validate capability at dispatch and at commit, not only at Context
resolution time (effect transition guards,
`specs/transitions/effect.transitions.json`).

## 4. Descriptor is not permission

An OperationDescriptor describes what an operation can do; an
AuthorizationCapability states what a subject may do. The two MUST be
distinct types, checked independently; possession of a descriptor or catalog
entry grants nothing (vector `catalog-effect-confusion.json`,
[REQ-CAT-DISCOVER-001], [REQ-CAT-MATCH-001]; binding rules
[REQ-CAT-BIND-001]).

## 5. Management channel

Privileged management actions additionally require a valid
PrivilegedManagementSession: expired or revoked sessions fail with
`MANAGEMENT_SESSION_EXPIRED` / `MANAGEMENT_SESSION_REVOKED`; step-up
requirements return `MANAGEMENT_STEP_UP_REQUIRED` as a challenge, not a
grant. Self-authorization is denied (`MANAGEMENT_SELF_AUTHORIZATION_DENIED`,
vector `management-untrusted-self-authorization.json`); independent approval
is enforced where required (`MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED`,
vector `management-independent-approval.json`). Task and management channels
keep disjoint credentials and caches ([REQ-SHELL-CHANNEL-001], vector
`shell-channel-isolation-003.json`).

## 6. Compliance checks

Every step in section 2 has at least one negative test asserting denial
before side effects. Capability intersection, attenuation and revocation
races are covered by vectors `cap-bind-001.json`, `cap-lease-003.json`,
`capability-attenuation.json`, `context-revocation-cache-reuse.json`.
Schema-valid capability documents alone prove nothing.
