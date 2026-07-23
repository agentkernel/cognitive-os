# V02 CA AUDIT real-consumer owner docket

- Docket ID: `V02-CA-AUDIT-CONSUMER-DOCKET-01`
- Date: 2026-07-23
- Classification: governance preparation only; no machine asset or registration
- Baseline: `lane/ctr-v02-audit-privileged-read-registration@3792d915a73a28187e9740648f6e0d753f286957`
- Status: **owner-confirmed design direction: candidate A; consumer proof incomplete**

## 1. Boundary

This docket executes only WP-1 preparation in the v0.2 governance-unblock plan.
It does not select a consumer, assign a schema ID, approve a policy value, or
create an AUDIT record, stream, receipt, schema, error, profile, extension,
binding, vector, implementation, or claim. All such assets remain
unregistered.

The consumer gate applies to the exact future
`privileged_read_decision` record plus its stream and commit receipt. An Event,
log, trace, telemetry, SQLite row, private DTO, fixture, code generator, or CI
enumeration is not a real consumer.

## 2. Consumer qualification gate

The owner may select a candidate only when all six statements have attached,
reviewable evidence:

1. Consumption crosses a real boundary rather than a private module boundary.
2. Input contains a complete `(asset_id, SemVer, digest)` triple.
3. The consumer produces a deterministic accept or reject result.
4. Removing a required record, stream, or receipt fact makes that consumer fail.
5. The consumer has an independent owner, lifecycle, and failure responsibility.
6. The consumer is not a fixture, template, generated binding, log/Event,
   SQLite row, or CI enumeration.

Failure of any statement is a consumer-gate **NO-GO**, not a request to weaken
the future AUDIT carrier.

## 3. Candidate fact packets

| Candidate | Possible boundary and exact consumption role | Preconditions before owner may select | Principal risks | Current result |
|---|---|---|---|---|
| A. `status.inspect` result-release audit gate | Management result-release component consumes the committed privileged-read decision, stream position, and receipt before releasing a success, denial, challenge, or error result. | A separately owned authoritative audit store; exact result-release failure oracle; AUDIT lower assets must not reverse-pin OPS; existence hiding must be shown for every terminal shape. | Subject/tenant/authority existence leakage; OPS↔AUDIT digest cycle; suppressing a safe denial on audit failure. | **Owner-selected 2026-07-23 as the first design direction; no current independent consumer proof.** |
| B. Independent compliance/export verifier | A separately operated verifier consumes canonical exported records, checkpoints, and manifests and deterministically accepts or rejects an export. | A real controlled export boundary; independent verifier owner; complete omission/duplicate/reorder/high-watermark handling; exact use of `privileged_read_decision`. | Redaction leakage; export omission/reordering; checkpoint/key rotation gaps. | **Candidate only; no current controlled export consumer proof.** |
| C. Recovery/high-watermark verifier | A recovery authority separate from the writer/store consumes exact records, stream positions, receipts, and checkpoints before authorizing resume. | Separate recovery authority; explicit proof it consumes privileged-read facts rather than only a generic chain; durable ordering/fork/gap failure authority. | Treating generic chain validation as a privileged-read consumer; unsafe recovery after store failure. | **Candidate only; no current independent consumer proof.** |

Candidate A is now the owner-selected design direction. It still requires the
owner decision record below, named independent ownership, and an audit/privacy
reviewer before it can pass the consumer gate. B and C remain unselected.

## 4. Owner decision record — real consumer

Complete one record for each candidate considered. A blanket approval is invalid.

| Field | Required decision/evidence |
|---|---|
| Candidate | A, B, or C above; one record per candidate |
| Consumer owner | **Owner-appointed 2026-07-23:** HAL9001 is Management Operations API Owner for the result-release gate; HAL9002 is Authoritative Audit Service Owner for durable audit persistence. They are distinct accountable owners and neither role may be the AUDIT packet author alone. |
| Boundary | **Owner-confirmed role model, 2026-07-23:** independently deployed Management API and Authoritative Audit Service communicate through an authenticated, version-pinned internal service API; actual endpoint/deployment evidence still required |
| Input triple | Exact future `(asset_id, SemVer, digest)` accepted by the consumer; may remain `owner decision required` until final bytes exist, but then selection cannot pass |
| Deterministic result | Accepted/rejected outcomes and the responsible failure path |
| Required facts | Each record/stream/receipt field whose removal causes rejection |
| Failure oracle | Safe outcome on missing, stale, mismatched, reordered, or persistence-failed input |
| Independence review | **Owner-appointed 2026-07-23:** HAL9003 is Security & Privacy Reviewer, independent of HAL9001 and HAL9002. Conflict disclosure, review scope, date, methods, and conclusion remain required before a review claim. |
| Owner decision | `selected` — candidate A, 2026-07-23; accountable owners and boundary still required |

## 5. Seventeen itemized AUDIT owner decisions

Each row is independently required. “Selected consumer” does not close any row;
final canonical bytes and repository-computed digests remain later gates.

| # | Decision | Current status | Required owner record before it may advance |
|---:|---|---|---|
| 1 | `privileged_read_decision` identity, SemVer, publication status | **owner-confirmed 2026-07-23:** `cognitiveos.audit.configuration-authority-record/0.2`, `0.2.0-draft.1`, `unregistered` | No digest before final bytes; publication remains forbidden pending every later gate |
| 2 | `commit_privileged_read_decision` machine-port responsibility | **owner-confirmed 2026-07-23:** `AuthoritativeAuditPort.commit_privileged_read_decision` is an AUDIT responsibility owned by HAL9002; HAL9001 may release only after its accepted receipt | No trait/schema/binding exists yet; failed or mismatched receipt blocks release |
| 3 | record/stream/receipt/checkpoint/retention/redaction/export graph | **owner-confirmed 2026-07-23:** lower = record/stream/commit receipt; upper = checkpoint/retention/redaction/export; lower must close before upper begins | All remain `0.2.0-draft.1` candidates/unregistered; lower→upper only; no placeholder digest |
| 4 | terminal-outcome field applicability and minimization | **owner-confirmed 2026-07-23:** success binds result facts; denial/challenge contain only safe selector/reason or challenge summaries; error binds safe stage/registered code/retryability; all carry kind/time/partition/writer-epoch/integrity facts | Caller cannot select or remove mandatory fields; no schema exists yet |
| 5 | existence-hiding policy | **owner-confirmed 2026-07-23:** denial/challenge/error records exclude subject, tenant, authority, target existence, and protected facts; only irreversible safe selector digest, normalized safe reason category, and required integrity facts may remain | Deterministic minimization proof and schema enforcement remain later gates |
| 6 | tenant/platform partition tuple | **owner-confirmed 2026-07-23:** `(scope_domain, tenant_id-or-null, management_domain, audit_profile_digest)`; platform scope uses null tenant, tenant scope requires a tenant ID; cross-tenant/global mixed streams are forbidden | Canonical schema enforcement remains later |
| 7 | integrity, writer, CAS, and recovery obligations | **owner-confirmed 2026-07-23:** single fenced writer, contiguous sequence, previous-record digest, CAS high-watermark, writer epoch, signed checkpoint, and recovery barrier; gap/fork/stale epoch/checkpoint mismatch fails closed | Asset/schema/consumer enforcement remains later |
| 8 | durable audit-before-visibility ordering | **owner-confirmed 2026-07-23:** HAL9001 releases terminal results only after HAL9002 durably commits record, stream position, and receipt; failure/mismatch fails closed with zero dispatch, Effect, business commit, success receipt, or partial result | Port/receipt enforcement remains later |
| 9 | persistence failure responsibility and oracle | **owner-confirmed 2026-07-23:** future `AUDIT_STORE_UNAVAILABLE` is limited to authoritative AUDIT-port persistence before commit/receipt, retryable only before any dispatch/Effect/business commit/visible result; never reuse `STATE_STORE_UNAVAILABLE`; possible external outcome enters unknown/reconcile | Error remains unregistered pending later gates |
| 10 | domains, projections, exclusions | **owner-confirmed 2026-07-23:** RFC 8785; `authoritative-audit-record-content/0.2` and `authoritative-audit-stream/0.2`; only record self-digest and detached-signature bytes may be excluded; sequence/predecessor/partition/writer epoch/receipt/high-watermark remain bound; repository tool computes final SHA-256 | Final schema bytes remain absent |
| 11 | checkpoint/export key usage and SIG dependency | **owner-confirmed 2026-07-23:** strict Ed25519; distinct `audit-checkpoint-signing` and `audit-export-signing` usages via future SIG registry; certification root never signs business objects; lower AUDIT assets do not static-pin SIG object-specific upper digests | SIG assets/review remain pending |
| 12 | retention, legal hold, redaction, export policy | **owner-confirmed 2026-07-23 draft:** 7-year minimum retention; checkpoint every 10,000 records or 15 minutes, plus epoch/key/export/recovery triggers; Compliance Officer sets hold and Compliance Officer + HAL9002 release it; deterministic derived views only; signed export manifest consumed by independent verifier | Legal/compliance review and machine policy assets remain required |
| 13 | AUDIT→OPS/TARGET/SIG DAG | owner decision required | Lower-to-upper dependency proof; no placeholder digest |
| 14 | freeze and activation order | owner decision required | Requirement, bundle, lower assets, suite/claim, and epoch ordering |
| 15 | real independent consumer | **NO-GO** | A completed §4 record passing all six qualification statements |
| 16 | AUDIT error responsibility | owner decision required | Exact carrier and consumer before any new category/code consideration |
| 17 | final bytes and tool-computed digests | **NO-GO** | Schema-valid immutable bytes and repository-tool digest evidence after prior gates |

## 6. Completion rule and stop condition

WP-1 preparation is complete only when a candidate fact packet and all seventeen
itemized decision records are ready for owner review. It is **not** a
machine-registration GO. If an owner decision, consumer proof, or independent
review is absent, retain `owner decision required` and exact registered assets =
none.
