# V02-CA-OPS-01 Draft Release Notes

- Proposed release: `0.2.0-draft.1`
- Status: all four docs-only designs merged; AUDIT owner-authorized security/audit/compliance review completed with limited provenance; OPS registration eligibility NO-GO; not published or specified
- Operation-set digest: `unresolved/not computed`
- Classification: docs-only; no machine registration or implementation

## Added

- operation-set and descriptor governance model;
- intended closed-core candidates;
- intended critical, explicitly negotiated extension candidates;
- nested identity verification from specification set through request/result schemas;
- per-operation request/result, channel, authority, target/readback, risk, error, audit, and migration obligations;
- fail-closed negative-test plan with G0–G6 rejection stages.
- TARGET source audit and proposed structural governance for the three configure
  candidates; the governed-object outer model is reusable, while target bodies,
  consumers, readback/verifiers, receipts, risk/approval, errors, and audit
  bindings remain unresolved.
- SIG source audit and proposed detached-signature envelope family with distinct
  session and approval profiles, domains, projections, key usages, replay rules,
  verification order, receipt responsibilities, and 40 planned negatives.
- Owner-confirmed pure strict Ed25519-only, governed strong-ref key registry,
  platform-root-signed registry manifest, depth-one tenant delegation, distinct
  session/approval leaf usages, 24-hour rotation overlap, immediate revocation,
  exact receipt/AUDIT split, tier/session/replay rules, and 19 future SIG errors.
- AUDIT source audit and owner-confirmed Event-plus-closed-record carrier,
  platform/tenant-domain stream partition, fenced contiguous sequence,
  previous-record digest chain, signed periodic checkpoints, minimized records,
  policy-derived retention floor, independent legal-hold release, deterministic
  redaction, and signed canonical export manifest.
- Proposed dedicated checkpoint/export signing usages and exact future AUDIT
  error responsibilities; all remain unregistered and independently reviewable.

## Breaking

- an open operation string is no longer sufficient to dispatch under the proposed v0.2 model;
- unknown or unnegotiated operations fail closed;
- an old negotiation epoch cannot acquire v0.2 operations or extensions;
- reconnect requires a new negotiation and authorization revalidation;
- operation-set, descriptor, or schema drift terminates or suspends the epoch;
- `diagnostics.configure`, `gateway.configure`, and `system.configure` require critical extension negotiation and TARGET closure.
- no URI, open JSON, private row/DTO, Event payload, vector fixture, route,
  caller value, or catalog projection can become a configuration authority
  target during migration or negotiation.
- a v0.1 session or approval signature string cannot be interpreted as a v0.2
  detached signature or upgraded in place; new profiles require a new epoch,
  reauthentication/reissuance or rechallenge/redecision, and exact digest pins.
- an Event, transition record, receipt, `audit_ref`, SQLite row, outbox, boolean,
  log, trace, telemetry item, wall clock, UUID order, or autoincrement value
  cannot be promoted into an authoritative audit record or continuity proof;
- an old epoch cannot enable the audit critical extension, and export cannot
  proceed without exact authorization, redaction, checkpoint, high-watermark,
  manifest, and signing-profile pins.

These are proposed v0.2 design consequences, not active runtime behavior.

## Unchanged

- all v0.1 bytes, digests, assets, and negotiation epochs;
- 273 registered requirements;
- 55 registered errors;
- 61 registered schemas;
- 84 vectors;
- 59 pass and 25 not-run;
- self-check 40;
- matrix non-empty implementation count 70;
- Profile implemented = 0;
- D-016 remains open;
- D-022 remains blocking;
- CA-1 through CA-8 remain blocked.

## Candidate status

All five intended-core and three intended-extension candidates are `blocked`.
The TARGET audit did not select or register a system, gateway, or diagnostics
target profile. There is no published core, extension, descriptor, target
profile, state domain, set digest, or machine membership in this release note.

The owner selected the SIG algorithm/key/trust/domain/projection/error model at
the docs-only design level. No envelope, key descriptor/registry manifest,
profile, receipt, replay ledger, error, schema, extension, or digest is machine
registered; independent security review remains pending.

The owner selected the AUDIT carrier/stream/integrity/checkpoint/retention/
legal-hold/redaction/export model at the docs-only design level. No audit record,
stream, checkpoint, policy, export manifest, signature profile/key usage,
persistence port, error/category, schema, extension, or digest is registered;
the owner-authorized security/audit/compliance review component is completed,
with the provenance limitations below.

The 2026-07-23
[registration-readiness audit](V02-CA-OPS-REGISTRATION-ELIGIBILITY-AUDIT.md)
revalidated every mandatory descriptor binding and found no eligible member.
It also found no independently useful foundation asset whose identity,
publication, digest, empty-set semantics, activation order, errors, and
cross-family bindings are uniquely fixed. No release asset is added by that
audit.

The owner-authorized AUDIT security/audit/compliance review found no blocking
design defect after PR #54 merged. It is not an external human, third-party, or
GitHub review and does not register an AUDIT profile. SIG independent
security/cryptography review remains pending.
