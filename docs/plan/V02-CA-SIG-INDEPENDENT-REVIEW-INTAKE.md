# V02 CA SIG independent security/cryptography review intake

- Intake ID: `V02-CA-SIG-INDEPENDENT-REVIEW-01`
- Date: 2026-07-23
- Classification: review preparation only; no machine registration
- Status: **HAL9003 owner-appointed as independent reviewer; engagement evidence and final-byte review still required**

## 1. Purpose and non-claim

This intake starts WP-2 preparation for a genuinely independent SIG
security/cryptography review. It does not constitute a review, select a
reviewer, approve the SIG design, create a machine asset, or make an
implementation, behavior, or Profile claim.

The owner-authorized design work recorded in
`V02-CA-SIG-DESIGN-DECISION.md` is not an independent human, third-party, or
registration-grade cryptography review.

## 2. Reviewer eligibility and provenance

Before review begins, the owner must record:

- reviewer name, organization, role, and contact route;
- independence from the design author, registration implementer, and sole owner
  decision maker;
- conflicts of interest and their disposition;
- review scope, methods, tools, and exact input commits/assets;
- a commitment to findings with severity, evidence, recommendation, and
  accepted/rejected/disposition status.

GitHub approval may transport a review but cannot by itself establish reviewer
identity or independence.

The owner appointed HAL9003 on 2026-07-23. HAL9003 must not implement SIG or
AUDIT assets, must disclose conflicts, and must record the exact scope, inputs,
methods, findings, and final-byte re-review when bytes exist.

## 3. Required review inputs

Provide the reviewer the following, version-pinned materials:

1. `V02-CA-SIG-DESIGN-DECISION.md` and ADR-0012;
2. canonical encoding and digest standard;
3. authentication/authorization standard;
4. current AUDIT design and privileged-read decision matrix;
5. final candidate schemas, projections, binding records, canonical bytes,
   preimages, and digests when they exist;
6. key registry/delegation/recovery threat model;
7. planned positive and negative vectors; and
8. cross-family dependency and digest-cycle diagram.

Until final bytes exist, a reviewer may issue only a design-level conclusion.
Any change to bytes, SemVer, digest, algorithm, encoding, domain, projection,
exclusion, key lifecycle, receipt, error semantics, or compatibility window
requires re-review.

## 4. Required review checklist

The reviewer must address, at minimum:

- pure Ed25519, strict encoding/length/canonical point/scalar and small-order
  handling, with no application prehash;
- authority-key registry, certification root, maximum-depth-one delegation,
  resolver freshness, outage, rotation, revocation, and recovery;
- separation of session, approval, checkpoint, and export usages;
- domain separation and exact projections/exclusions, including self-digest and
  detached-signature bytes;
- signed binding records, receipt minimization/persistence, replay ledger, and
  R3 atomic consumption;
- old epoch, downgrade, alias, fallback, and compatibility behavior;
- checkpoint/export signing and cross-family digest direction; and
- planned negative coverage for compromise, replay, stale cache, revoked keys,
  malformed encodings, and stripped critical semantics.

The following are rejection conditions: generic domain; caller-selected
algorithm/key/resolver; extra application prehash; root directly signing
business objects; usage reuse; stale-cache authorization; revocation grace;
unsigned/unpinned checkpoint or export; imprecise projection/exclusion; old
epoch auto-upgrade; missing provenance; or a review that is not refreshed after
final-byte changes.

## 5. Required deliverables and acceptance

The independent reviewer must supply a signed report, provenance/independence
statement, threat model, domain/projection/exclusion matrix, key-lifecycle
review, negative-case coverage map, cross-family digest-cycle review, and a
blocking-findings ledger. After final bytes exist, the reviewer must add a
final-byte re-review statement naming exact input triples.

An eligible design-level report can close only the design-review activity. SIG
machine registration remains NO-GO until all applicable final bytes, digests,
consumer evidence, generated bindings, and independent final-byte review are
complete.
