# V02 CA SIG owner-authorized agent design review

- Review ID: `V02-CA-SIG-AGENT-REVIEW-01`
- Date: 2026-07-23
- Reviewer: owner-authorized Codex agent
- Exact role: design/security source audit; **not** independent human or third-party review
- Baseline: `origin/main@1605d3f785678c4fbe56b0851b81c5016813421d`
- Result: **two blocking design ambiguities corrected; HAL9003 independent review and final-byte review remain required**

## Scope and method

The review compared `V02-CA-SIG-01`, ADR-0012, the canonical encoding/digest
standard, the later AUDIT owner decisions, TARGET/OPS boundaries, error
responsibilities, replay/rotation rules, and planned negatives. It treated
registered machine assets as authoritative and later owner decisions as
superseding earlier docs-only proposals. It did not inspect or claim private-key
material, execute cryptography, register assets, or produce behavior evidence.

## Findings

### AR-SIG-001 — AUDIT persistence used the state-store error

- Severity: blocker for future registration.
- Evidence: SIG G6 mapped ledger/receipt/audit/authority unavailability as one
  `STATE_STORE_UNAVAILABLE` responsibility; the later AUDIT decision explicitly
  reserves future `AUDIT_STORE_UNAVAILABLE` for authoritative AUDIT-port
  persistence and forbids broadening the state-store code.
- Resolution: split state/Event/authority persistence from AUDIT-port
  persistence; use future `AUDIT_STORE_UNAVAILABLE` only for the latter.
- Status: corrected at docs-only design level; error remains unregistered.

### AR-SIG-002 — early failure receipts required untrusted subject facts

- Severity: blocker for a closed receipt schema.
- Evidence: the shared receipt required object/profile/key facts even when the
  earliest failure was an unknown profile, malformed encoding, or unresolved
  key. Those fields cannot be authoritative at G1/G2.
- Resolution: one closed receipt schema uses a tagged union. The
  `input_rejected` variant contains only authenticated framing/channel context,
  established epoch facts, earliest stage, safe error, and a domain-separated
  received-input digest. Unestablished semantic facts are absent.
- Status: corrected at docs-only design level; schema remains unregistered.

## Areas reviewed with no contrary design finding

- pure Ed25519 without application prehash;
- strict encoding and object-specific domain separation;
- session/approval/checkpoint/export key-usage separation;
- platform-root certification-only model and depth-one tenant delegation;
- immediate revocation, bounded rotation overlap, and no stale authorization;
- exact projection/exclusion model and critical-extension downgrade defense;
- approval replay ledger and atomic R3 consumption;
- signature validity not expanding business authorization;
- lower-to-upper SIG/AUDIT/OPS digest direction.

These are design observations, not cryptographic verification or independent
approval.

## Residual blockers

1. HAL9003 conflict disclosure, method, signed findings, and independent review
   over the corrected exact commit.
2. Final candidate schemas/projections, canonical bytes, preimages, digests, and
   final-byte re-review.
3. Real key-registry/KMS boundary and resolver freshness/outage evidence.
4. Machine registration, generated bindings, new negative vectors, real
   implementation, behavior execution, and Profile evaluation in that order.

SIG machine registration and all downstream implementation remain NO-GO.
