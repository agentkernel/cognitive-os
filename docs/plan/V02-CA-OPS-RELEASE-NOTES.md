# V02-CA-OPS-01 Draft Release Notes

- Proposed release: `0.2.0-draft.1`
- Status: OPS and TARGET merged; SIG docs-only design materialized for owner/security review; not published or specified
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
- Bounded Ed25519/P-256 and governed-registry/external-resolver/trust-delegation
  alternatives; no algorithm, key infrastructure, or trust root is selected.

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

The SIG audit did not select an algorithm set, key resolver, trust root, or
registered profile. The proposed envelope and object-specific session/approval
profiles remain blocked and have unresolved digests.
