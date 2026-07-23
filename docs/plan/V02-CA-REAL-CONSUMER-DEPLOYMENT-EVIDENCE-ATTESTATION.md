# V02 CA real-consumer and deployment evidence attestation

- Evidence packet ID: `V02-CA-CONSUMER-EVIDENCE-01`
- Prepared: 2026-07-23
- Status: **template only; no deployment or consumer evidence submitted**

## 1. Common evidence rule

Every owner-supplied packet must identify real deployed components, versions,
independent lifecycle/failure ownership, exact input triples, deterministic
accept/reject behavior, and reproducible evidence. Fixtures, mocks, templates,
private DTOs, database rows, logs alone, generated bindings, and ordinary CI do
not qualify.

Each evidence artifact requires digest, capture time, environment, redaction
method, reproduction command/runbook, owner attestation, and HAL9003 or HAL9007
independent assessment as applicable.

## 2. AUDIT consumer packet — HAL9001/HAL9002

Required evidence:

- independently deployed Management Result-Release Gate and Authoritative Audit
  Service identities, owners, versions, endpoints, and authenticated boundary;
- exact record/stream/commit-receipt `(asset_id, SemVer, digest)` inputs;
- deterministic acceptance and rejection results;
- proof that removal or mutation of each mandatory fact causes rejection;
- existence-hiding evidence for success/denial/challenge/error;
- persistence-failure oracle: zero dispatch, Effects, business commits, success
  receipts, and partial results;
- ordering/gap/fork/stale-epoch/high-watermark negative evidence.

HAL9001 and HAL9002 must separately attest their boundary and failure ownership.
HAL9003 must independently assess the packet.

## 3. TARGET packets — one independent packet per line

### `system.configure` — HAL9004 + HAL9007

Provide the exact system/subsystem/policy target profile, apply consumer,
version/CAS/writer epoch, separately authorized readback, verifier identity and
criteria, receipt, partial/unknown/reconcile behavior, and negative oracle.

### `gateway.configure` — HAL9005 + HAL9007

Provide per-instance authority evidence, group-to-instance decomposition,
fan-out/partial-apply handling, readback/verifier, receipt, trust/route loss,
rollback/reconcile, and negative oracle.

### `diagnostics.configure` — HAL9006 + HAL9007

Provide the diagnostics-policy authority, strong sink/profile/credential/
retention/export refs, sensitivity and cross-tenant controls, partial-sink
handling, readback/verifier, receipt, reconcile, and negative oracle.

HAL9007 must use three distinct verifier identities/versions and evidence sets;
one line cannot close another.

## 4. Submission manifest

Owners must add one row per submitted artifact. Blank or placeholder rows do not
constitute evidence.

| Line | Artifact ID/version | SHA-256 | Environment/time | Owner attestation | Independent assessment | Result |
|---|---|---|---|---|---|---|
| `REQUIRED` | `REQUIRED` | `REQUIRED` | `REQUIRED` | `REQUIRED` | `REQUIRED` | `REQUIRED` |

Until real artifacts populate this manifest and independently pass, AUDIT and
TARGET consumer gates remain NO-GO.
