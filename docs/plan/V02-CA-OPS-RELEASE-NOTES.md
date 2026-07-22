# V02-CA-OPS-01 Draft Release Notes

- Proposed release: `0.2.0-draft.1`
- Status: design proposal for owner review; not published or specified
- Operation-set digest: `unresolved/not computed`
- Classification: docs-only; no machine registration or implementation

## Added

- operation-set and descriptor governance model;
- intended closed-core candidates;
- intended critical, explicitly negotiated extension candidates;
- nested identity verification from specification set through request/result schemas;
- per-operation request/result, channel, authority, target/readback, risk, error, audit, and migration obligations;
- fail-closed negative-test plan with G0–G6 rejection stages.

## Breaking

- an open operation string is no longer sufficient to dispatch under the proposed v0.2 model;
- unknown or unnegotiated operations fail closed;
- an old negotiation epoch cannot acquire v0.2 operations or extensions;
- reconnect requires a new negotiation and authorization revalidation;
- operation-set, descriptor, or schema drift terminates or suspends the epoch;
- `diagnostics.configure`, `gateway.configure`, and `system.configure` require critical extension negotiation and TARGET closure.

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

All five intended-core and three intended-extension candidates are `blocked`. There is no published core, extension, descriptor, set digest, or machine membership in this release note.
