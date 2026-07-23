# V02 CA remaining owner decisions closure

- Decision ID: `V02-CA-OWNER-CLOSURE-01`
- Date: 2026-07-23
- Branch: `lane/ctr-v02-d016-min-member-set`
- Classification: owner-governance closure; docs-only
- Result: **remaining owner choices closed; evidence-dependent registration and implementation gates remain NO-GO**

## 1. Authority and boundary

The repository owner authorized the agent to resolve the remaining governance
choices using the least-authority, independently reviewable interpretation. This
authorization cannot manufacture a deployed consumer, independent review
report, final canonical bytes/digests, generated bindings, executed behavior, or
Profile evidence.

## 2. Accountable owner assignments

| Responsibility | Accountable owner | Separation rule |
|---|---|---|
| Management request/result release | HAL9001 | Never owns authoritative AUDIT persistence or target apply |
| Authoritative AUDIT service/store | HAL9002 | Never owns management result release or independent review |
| Security/privacy/cryptography/compliance review | HAL9003 | Does not implement reviewed SIG/AUDIT/TARGET assets; conflict disclosure required |
| System Configuration Authority | HAL9004 | Owns `system.configure` apply/readback/receipt; independent from HAL9001–HAL9003 |
| Gateway Configuration Authority | HAL9005 | Owns per-gateway-instance apply/readback/receipt; group rollout is coordination, not a single authority target |
| Diagnostics Policy Authority | HAL9006 | Owns diagnostics-policy apply/readback/receipt; sink/profile remain strong referenced dependencies |
| Independent TARGET verifier | HAL9007 | Read-only verifier for all three TARGET lines; never holds write authority |
| Management Session Authority | HAL9008 | Owns `session.create_restricted` issuance/current-version authority |
| Capability Authority | HAL9009 | Owns `capability.revoke` target/version/revocation authority |
| Execution Authority | HAL9010 | Owns `execution.stop` target and authoritative post-state |
| Effect Recovery Authority | HAL9011 | Owns `effect.reconcile` readback/reconcile/quarantine authority |
| SIG key-registry authority | HAL9012 | Owns descriptors, usages, rotation/revocation/recovery; certification root never signs business objects |

## 3. TARGET choices

- `system.configure` targets an exact `system`, `subsystem`, or `policy` strong
  reference selected by the future profile; no platform-default target exists.
- `gateway.configure` targets one gateway instance. Group rollout expands into
  individually versioned instance operations with independent receipts and
  partial-apply reconciliation.
- `diagnostics.configure` targets one diagnostics-policy object. Destinations,
  sinks, credentials, retention, and export profiles are strong references, not
  alternative target identities.
- HAL9007 performs independently authorized readback verification. Read
  authority never grants write or export authority.
- Unknown target, authority, consumer, verifier, risk, approval, writer epoch,
  or outcome fails closed. Risk is derived from exact target scope and parameter
  digest; no operation-wide default risk is allowed.

These are design choices only. No real apply consumer, endpoint, deployment,
schema, receipt, or behavior proof exists yet.

## 4. D-016 operation-member choices

All eight candidates are mandatory: `status.inspect`,
`session.create_restricted`, `capability.revoke`, `execution.stop`,
`effect.reconcile`, `system.configure`, `gateway.configure`, and
`diagnostics.configure`.

Each candidate remains `0.2.0-draft.1` and unregistered until its exact
descriptor/request/result triples, authority, Effect/idempotency, error map,
AUDIT responsibility, consumer, compatibility, and independent registration
review are complete. A name, route, private type, CLI verb, or implementation
reachability is not membership. No empty or partial operation set may publish.

## 5. SIG and AUDIT decisions

- HAL9003 is the appointed independent SIG reviewer and Compliance Officer,
  subject to conflict disclosure, exact input versions, signed findings, and a
  final-byte re-review.
- HAL9012 owns the governed key registry. Session, approval, checkpoint, and
  export key usages remain separate; revocation is immediate and stale cache
  never authorizes.
- The seventeen AUDIT owner decisions in the real-consumer docket are closed at
  governance level. Items 15 and 17 intentionally remain NO-GO for missing real
  consumer proof and final canonical bytes/digests.

## 6. D-022 and CA status

No owner-choice ambiguity now blocks preparation. D-016 and D-022 nevertheless
remain open/blocking because OPS/TARGET/SIG/AUDIT machine registrations, real
consumer/deployment evidence, HAL9003's independent review outputs, generated
bindings, and final immutable triples do not exist. CA-0 cannot be re-reviewed
to GO, and CA-1 through CA-8 cannot start.

## 7. Next executable evidence packets

1. HAL9003 conflict disclosure and design-review report over exact current
   inputs; later repeat over final bytes.
2. HAL9001/HAL9002 authenticated service boundary and deterministic
   record/stream/receipt accept-reject evidence.
3. HAL9004–HAL9007 TARGET consumer/readback/verifier evidence for each configure
   line.
4. Final-byte registration proposals, one family at a time, only after their
   consumer and independent-review gates pass.

Until those packets exist, exact registered assets remain none and Profile
`implemented = 0`.
