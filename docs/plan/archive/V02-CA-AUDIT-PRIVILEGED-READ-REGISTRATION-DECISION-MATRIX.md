# V02 CA AUDIT Privileged-Read Registration Decision Matrix

- Decision packet: `V02-CA-AUDIT-PRIVILEGED-READ-REG-01`
- Date: 2026-07-23
- Branch: `lane/ctr-v02-audit-privileged-read-registration`
- Baseline: `origin/main@117df63dfd435f57cac8b700e11a200517f56d0d`
- Classification: independent AUDIT owner-governance closure audit; docs-only
- Result: **machine-registration NO-GO; exact registered assets = none**

## 1. Boundary and result

This packet is independent from OPS, TARGET, and SIG registration.  It does not
re-open or alter PR #56's OPS conclusion.  It records the current authoritative
facts for the future privileged-read audit carrier and the mandatory bindings
that are still unclosed.

No registered AUDIT schema, error, port, profile, extension, key usage, vector,
generated binding, implementation, evidence artifact, or Profile claim exists
at this baseline.  A prose design, an Event, a log, trace, telemetry, SQLite
row, transition record, receipt, private DTO, or `audit_ref` is not an
authoritative audit contract.

Consequently this batch registers **no** machine asset or member and executes
no new AUDIT or OPS behavior vector.  It does not create a fake, future, zero,
or unresolved digest.

## 2. Existing owner-confirmed design inputs

The following facts were already selected by `V02-CA-AUDIT-01` and ADR-0013 at
the docs-only design level.  They are not final bytes, registrations, or a
substitute for the itemized closure below:

| Topic | Existing design input |
|---|---|
| carrier | closed `AuthoritativeAuditRecord` inside the existing Event outer envelope |
| record kind | future exact kind `privileged_read_decision` |
| port responsibility | future exact `AuthoritativeAuditPort.commit_privileged_read_decision` |
| proposed family identities | record, stream, checkpoint, retention, redaction, export, and commit-receipt assets listed in §4 of `V02-CA-AUDIT-01` |
| stream tuple | `(scope_domain, tenant_id-or-null, management_domain, audit_profile_digest)` |
| integrity | single fenced writer, contiguous sequence, previous-record digest, CAS high-watermark, signed checkpoints, recovery barrier |
| digest model | RFC 8785, versioned domains, only named self-digest and detached-signature exclusions |
| signing design | distinct strict Ed25519 checkpoint/export usages through the governed key registry; SIG root is certification-only |
| retention/export | digest-pinned retention policy, independent legal-hold release, derived redaction views, signed export manifest |
| static DAG | lower AUDIT assets never reverse-pin OPS/TARGET/SIG; later upper assets may pin closed lower triples |

The exact current machine result remains none: there is no AUDIT schema or
generated Rust/TypeScript binding, `REQ-AUDIT-001/002` have no implementation
or implementation-test path in the traceability matrix, and the 55-code error
registry has no `audit` category or AUDIT error.

## 3. Mandatory owner-decision closure audit

The repository records no later, contrary governance decision.  However, the
following decisions have not been closed as final, independently consumable,
canonical-byte machine contracts.  A total authorization cannot replace any
row.

| # | Mandatory binding | Current evidence | Closure result |
|---:|---|---|---|
| 1 | `privileged_read_decision` exact asset ID, complete SemVer, publication status | only the record-kind spelling is selected; proposed family is unregistered and `0.2.0-draft.1` has no final bytes | blocked |
| 2 | `commit_privileged_read_decision` is an AUDIT machine-port responsibility | design names the future responsibility, but no closed port schema/trait/binding exists | blocked |
| 3 | exact record, stream, receipt, checkpoint, retention, redaction, and export asset graph | identities are design candidates only; no complete registered graph or bytes | blocked |
| 4 | minimum success, denial, challenge, and error record fields | the general binding matrix exists, but no record-kind schema fixes applicability/minimization per terminal outcome | blocked |
| 5 | existence-hiding minimization | selector digest/safe reason are selected in prose, but no closed schema/policy proves absent subject, tenant, authority, and protected facts cannot enter concealed records | blocked |
| 6 | tenant/platform partition tuple | design tuple exists, but no stream schema canonicalizes `scope_domain`, tenant nullability, and management-domain constraints | blocked |
| 7 | sequence, prior digest, checkpoint, fencing, writer authority, CAS, recovery | design selections exist; no stream/checkpoint/receipt assets or consumer enforce them | blocked |
| 8 | durable-commit ordering for audit success reads and denials | design requires commit before visibility, but no privileged-read port contract/receipt implements the ordering | blocked |
| 9 | persistence-failure error, category, retryability, stage, and zero-side-effect oracle | proposed `AUDIT_STORE_UNAVAILABLE` is unregistered; no audit category/error asset or exact privileged-read failure schema exists | blocked |
| 10 | canonical digest domains/projections/self-digest and detached-signature exclusions | proposed domains exist; exact JSON Pointer exclusions and final schema-valid objects do not | blocked |
| 11 | checkpoint/export signing usage, algorithm, key registry, and SIG dependency | Ed25519 and distinct usages are design-selected; key descriptors/usages and SIG independent review/receipt are pending | blocked |
| 12 | retention floor, legal hold, redaction, export policy | policy model is selected, but actual finite policy values and profiles are absent | blocked |
| 13 | one-way AUDIT to OPS/TARGET/SIG digest DAG | direction is selected, but no lower AUDIT triple exists; no upper triple may be filled with a placeholder | blocked |
| 14 | freeze/activation order for requirement set, schema bundle, record/stream/receipt, suite/profile, and new epoch | directional rule exists, but no frozen lower AUDIT assets or new epoch exists | blocked |
| 15 | real independent consumer | no current cross-boundary consumer of a complete privileged-read record/stream/receipt contract is identified; private types and existing audit-adjacent artifacts are excluded | **NO-GO** |
| 16 | AUDIT error responsibility | all existing errors were audited; none may broaden into the required audit-port/integrity responsibilities, and new errors cannot register without an eligible consumer | blocked |
| 17 | final canonical bytes and repository-tool digests | no final schema-valid bytes or computed digests exist for any proposed AUDIT asset | **NO-GO** |

## 4. Required asset graph if a later batch closes every gate

The permitted direction is only:

```text
requirements
    ↓
record / stream / commit-receipt / checkpoint / retention schemas
    ↓
AUDIT profile and extension
    ↓
specification set
    ↓
suite/profile selection
    ↓
new negotiation epoch
```

No lower AUDIT asset may digest-pin an OPS, TARGET, or SIG asset.  A later OPS
descriptor may pin final registered AUDIT record/stream/receipt triples only
after they exist.  Other unregistered families remain blockers, not values.

## 5. Error ruling

`STATE_STORE_UNAVAILABLE` stays limited to its registered authoritative
state/Event persistence condition. `DIGEST_MISMATCH` stays limited to a
recomputed declared canonical digest. `SCHEMA_MISMATCH`,
`PROTOCOL_SCHEMA_DIGEST_MISMATCH`, `EFFECT_OUTCOME_UNKNOWN`, and management
codes retain their registered responsibilities. None is a substitute for an
AUDIT stream, writer, integrity, receipt, persistence, retention, or export
failure.

The design's proposed AUDIT error list, including `AUDIT_STORE_UNAVAILABLE`, is
not registered. Adding an `audit` category or any code now would create a
machine surface without an independently useful consumer and final asset bytes,
so it is forbidden by this NO-GO.

For a future terminal privileged-read denial, the intended oracle remains one
safe durable denial audit commit and zero dispatches, Effects, business state
mutations, authority business commits, success receipts, or partial results.
That is a design obligation, not behavior evidence from this batch.

## 6. Preserved blockers and evidence boundary

- D-016 remains open; D-022 remains blocking; IMP-01 remains effective.
- SIG independent security/cryptography review remains pending.
- CA-1 through CA-8 remain blocked; Configuration Authority implementation is
  not provided.
- OPS/TARGET/SIG/AUDIT machine contracts remain unregistered.
- Existing vector `expected` values are unchanged; no behavior vector is run.
- Pins remain 273 requirements / 55 errors / 61 schemas / 84 vectors; vector
  state is 59 pass / 25 not-run; self-check is 40; non-empty matrix
  implementation paths are 70; Profile `implemented = 0`.

## 7. Next permitted entry

A future owner-governance batch must provide a real independent consumer and
then confirm every row in §3 individually with final schema-valid canonical
bytes and repository-computed digests.  Only an independently reviewed,
registered AUDIT record/stream/commit-receipt closure may unblock the later OPS
`status.inspect` envelope/epoch/compatibility decision.
