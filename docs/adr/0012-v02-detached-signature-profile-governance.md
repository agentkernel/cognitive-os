# ADR-0012: v0.2 Detached Signature Profile Governance

- Status: Proposed for owner/security review
- Date: 2026-07-22
- Decision owners: repository owner, security/cryptography reviewer,
  identity/KMS owner, management-session authority, and approval authority
- Classification: structural docs-only design; no machine registration
- Baseline: `origin/main@42d609b2f49e2db641f46aa99b6cc9a538a7f4fd`
  (PR #52 merge)
- Decision packet: [V02-CA-SIG-01](../plan/V02-CA-SIG-DESIGN-DECISION.md)

## Context

The registered `PrivilegedManagementSession` and
`ManagementApprovalDecision` schemas require `authority_signature`, but each
field is only a string with `minLength: 16`. No machine asset fixes the
algorithm, allowed set, key ID/resolution, signature domain, signed schema,
projection/exclusions, encoding, trust root, rotation, revocation, verification
receipt, or general invalid-signature errors.

The canonical standard fixes the generic signature preimage construction but
requires the selected object contract to supply those facts. Existing
canonical helpers construct preimages only. Session parsing, fixture signatures,
schema validity, digest-shaped strings, and vector booleans do not perform
cryptographic verification. `MANAGEMENT_SELF_AUTHORIZATION_DENIED` covers
self-authorization, and `DIGEST_MISMATCH` covers a declared digest mismatch;
neither is a generic bad-signature code.

## Decision

1. Permit a future closed detached-signature envelope/profile family with
   profile triple, algorithm, key ID, signed domain, signed schema digest,
   signed content digest, and encoded signature bytes.
2. Do not permit caller-provided algorithms, keys, resolvers, trust roots,
   projections, or excluded paths. These are selected by a digest-pinned
   profile and negotiation epoch.
3. Define separate session and approval profiles. They have independent IDs,
   versions/digests, signature domains, signed schemas/projections, key usages,
   authorities, replay rules, lifetimes, receipts, and business checks.
4. Forbid `generic`, `object`, `payload`, or any cross-object signature domain.
   Session signature bytes cannot be replayed as approval bytes, and approval
   bytes cannot be replayed across proposal/session/profile bindings.
5. Use the canonical standard section 12 input over the exact object-specific
   signed projection. Do not sign display digests, pretty JSON, transport bytes,
   open payloads, or implicitly reparsed objects.
6. Explicitly exclude only `/authority_signature` from each subject projection.
   The content-digest projections separately exclude their own digest and
   `/authority_signature`. The final signed projection is a closed,
   object-specific binding record containing profile/algorithm/key/schema/epoch
   metadata plus that subject and its recomputed digest; approval also binds the
   exact session and request digests. Future schemas must register these facts
   and paths exactly.
7. Keep digest integrity, signature validity, key/signer authorization,
   trust-root validity, rotation/revocation status, and business authorization
   as independent fail-closed decisions.
8. Verify profile, algorithm, domain, schema, projection, canonical bytes,
   digest, signature, key resolution, trust, status, usage, and signer authority
   before session/approval business authorization and before any Effect,
   dispatch, mutation, or commit.
9. A verification receipt records the verification facts and result but grants
   no authorization and proves no completion. AUDIT separately owns the
   authoritative carrier and atomic persistence slot.
10. A new signature profile or algorithm set is a critical, digest-pinned v0.2
    negotiation input. Old epochs cannot enable it silently; reconnect and
    migration require reauthentication and authorization revalidation.
11. v0.1 signature strings are migration input only. Sessions are reissued and
    approvals are rechallenged/redecided under new v0.2 identities; neither is
    upgraded in place.
12. Keep both profiles blocked until owner/security review selects the exact
    algorithm set, key/resolver/trust system, rotation/revocation rules, future
    schema/profile digests, and complete registered errors.

## Bounded alternatives

The decision packet leaves these alternatives open because no current
digest-pinned source chooses among them:

- Ed25519 with a fixed raw-signature base64url encoding;
- P-256 ECDSA with fixed-width low-S encoding;
- a strictly pinned two-algorithm set only if compatibility evidence justifies
  its additional downgrade surface;
- a governed CognitiveOS key registry;
- an immutable external KMS/PKI resolver profile;
- platform-rooted tenant delegation only if tenant-scoped authority requires it.

Unknown algorithms/keys, resolver ambiguity or outage, stale status, trust-root
mismatch, revoked or rotated-out keys, unauthorized usage, and indeterminate
business authority all fail closed. No alternative is selected by this ADR.

## Consequences

- Shared structure reduces envelope duplication without collapsing semantic
  domains or authorization rules.
- Current v0.1 session/approval schemas remain unchanged and do not become
  cryptographic profiles.
- Future registration requires new immutable v0.2 profile/schema identities,
  exact projections, key/trust machine assets, error mappings, and new negative
  vectors; existing vector `expected` values remain unchanged.
- `AuthorizationCapability` cryptography, TARGET machine assets, AUDIT carrier
  fields, operation membership, implementation, CFR execution, and Profile
  claims remain outside this ADR.
- D-016 remains open. D-022 remains blocking. CA-1 through CA-8 remain blocked.

## Alternatives rejected

### One generic profile for every object

Rejected. It permits cross-object confusion and cannot express distinct signer
authority, projection, lifetime, replay, and business rules.

### Treat the existing strings as encoded signatures by convention

Rejected. Their encoding, algorithm, keys, domains, and projections are not
registered. Length and schema validity are not cryptographic proof.

### Reuse capability signature fields as precedent

Rejected. `AuthorizationCapability.signature` is itself an open string, and a
capability redesign is outside this batch.

### Let the caller provide a verification key or trust root

Rejected. It would let untrusted input choose its own authority.

### Reuse nearby errors for every signature failure

Rejected. Digest mismatch, self-authorization, unsupported specification
version, and invalid cryptographic signature are different semantics.

## Rollback and failure strategy

If owner/security review rejects the family, profiles, projections, or bounded
alternatives, no machine asset changes. Keep both signature fields as their
current v0.1 strings, keep the SIG profiles blocked, preserve all vector and
Profile states, and require a new docs-only SIG decision before registration.
Any later correction to a published v0.2 Draft creates a new SemVer/digest and
migration note; it never rewrites a pinned identity.
