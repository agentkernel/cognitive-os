# ADR-0013: v0.2 Authoritative Audit Governance

- Status: Accepted as docs-only design; owner-authorized security/audit/compliance review completed with limited provenance; machine registration pending
- Date: 2026-07-22
- Decision owners: repository owner, security reviewer, audit/store/recovery
  owner, compliance/privacy owner, SIG owner, and Configuration Authority
- Classification: structural docs-only design; no machine registration
- Baseline: `origin/main@0a30ac70769f0501f7928d96f55f17636eaa9888`
  (PR #53 merge)
- Decision packet: [V02-CA-AUDIT-01](../plan/V02-CA-AUDIT-DESIGN-DECISION.md)

## Context

`REQ-AUDIT-001/002` specify Effect traceability, tamper resistance, ordering
integrity, retention, sensitivity control, and export audit. No registered
schema or profile defines an authoritative audit record, stream, checkpoint,
export manifest, retention/redaction policy, or atomic persistence port.

The Event envelope has an open payload; a transition record covers only a
committed transition; a SIG receipt proves only signature-verification facts;
an Effect receipt and VerificationReport have different responsibilities; AKP
`audit_ref` is an optional URI. SQLite sequences/triggers, `DenialAudit`, outbox
rows, booleans, logs, traces, telemetry, and existing vector expectations are
implementation or scenario facts, not cross-boundary audit authority.

Configuration Authority cannot report success if its required audit record is
missing. External Effects also require a recoverable multi-record closure rather
than a fictional single database transaction with the external system.

## Decision

1. Reuse the registered Event as the required cross-boundary outer envelope,
   with GovernedObjectHeader as its governance skeleton. Require a separate
   future closed `AuthoritativeAuditRecord` payload/profile; an open Event
   payload is never sufficient.
2. Keep Event, transition record, SIG receipt, Effect/external receipt,
   VerificationReport, audit record, stream/checkpoint, export, and telemetry
   as distinct artifacts. Existence of one never proves another.
3. Partition audit into one platform stream and tenant streams identified by
   tenant, management domain, and audit-profile digest. Do not use a mixed
   global tenant stream or fragmented per-object streams.
4. Require one current fenced sequence authority per stream, contiguous logical
   sequence, previous-record digest chaining, explicit genesis, CAS
   high-watermark, duplicate/gap/regression/fork detection, and append-only
   correction through supersession.
5. Require periodic signed checkpoints under a digest-pinned finite threshold
   policy. Use a dedicated `audit-checkpoint-signing` key/profile through the
   governed key registry. Do not require Merkle segments or external WORM
   anchoring in the initial profile; either is a future breaking profile.
6. Require distinct `audit-export-signing` usage/profile. Export canonical
   ordered JSON records plus a canonical signed manifest that binds filters,
   tenant/compartment scope, record digests, source checkpoints, and exact
   high-watermarks. Audit the export itself. This does not create an
   `audit.export` operation member.
7. Minimize authoritative records at creation. Forbid secrets, protected bodies,
   cross-tenant content, raw signature bytes, and replayable material. Use
   strong refs/digests for protected detail. Redaction creates a deterministic
   registered derived view and never edits authoritative bytes.
8. Derive retention from an exact digest-pinned policy rather than inventing a
   universal calendar floor. Effective retention is the maximum applicable
   subject, Effect, Verification, incident, contractual, platform, tenant, and
   legal obligation. Missing policy fails closed.
9. Allow platform or tenant compliance authority to set legal hold within its
   scope. Release requires a different principal/ActorChain acting as the same
   or higher compliance authority. The subject, writer, workload, or hold setter
   cannot release it.
10. Do not silently delete expired records. Compaction requires a complete
    checkpoint-sealed segment, cleared retention/hold gates, and a successor
    tombstone/checkpoint preserving range, count, ordered digest aggregate,
    continuity, policy, and authority proof.
11. Define a future authoritative persistence port with distinct denial,
    governed-commit, external-Effect-stage, checkpoint, and export operations.
    A denial record commits before returning a reliable denial. A successful
    state change atomically joins state CAS, transition record, Event, required
    SIG receipt handoff, audit record/stream CAS, outbox, and post-commit result
    visibility.
12. For external Effects, persist pre-dispatch audit before dispatch, then append
    attempt, outcome/unknown, reconciliation, Verification, commit/abort/
    quarantine, and final closure records. Recovery verifies the checkpoint and
    chain before proceeding and reuses original idempotency/fencing facts.
13. Use RFC 8785 and explicit versioned domains/projections/exclusions. Use pure
    strict Ed25519 for checkpoint/export signatures with distinct leaf keys; the
    SIG platform root remains certification-only.
14. Add a future `audit` error category and exact audit failure codes in the
    decision packet. Reuse existing errors only for their registered meanings.
15. Select the audit family only through a new critical, digest-pinned v0.2
    negotiation extension. Old epochs cannot enable it silently.
16. Add new positive/negative vectors later without modifying existing
    `expected` values. Do not implement or claim conformance before all four
    machine-registration lines and a fresh CA-0 GO.

## Owner-confirmed selection status

The repository owner confirmed the carrier, stream, integrity/checkpoint,
retention/legal-hold, redaction, and export choices on 2026-07-22. This closes
those alternatives at the docs-only design level. It does not register machine
assets, set policy numbers, compute digests, or substitute for independent
review of the resulting PR head.

After PR #54 merged, the repository owner expressly authorized the preceding
agent to review the exact merged design from security, audit, and compliance
perspectives. The review found no blocking design defect. Its provenance is
owner-authorized agent review, not external human, third-party, independent
cryptography, or GitHub review. It does not register an AUDIT asset, close SIG
review, authorize CA-0, or provide implementation or behavior evidence.

## Consequences

- The current Event schema remains unchanged and does not become an audit
  profile by convention.
- Existing SQLite atomicity and append-only triggers are reusable implementation
  inputs, not the normative port or cross-node integrity algorithm.
- A safe denial has one audit commit but no business commit. A success has one
  joined authority/audit commit and no pre-commit success receipt.
- Audit unavailability narrows system availability. It never permits buffering,
  partial success, telemetry fallback, or unaudited business mutation.
- Initial integrity verification is sequential and operationally simpler than a
  Merkle/WORM requirement, while signed checkpoints provide independently
  verifiable high-watermarks. A later scalable proof profile remains possible
  only through new identity/version/migration.
- Retention and checkpoint numeric values remain machine-registration inputs
  selected by reviewed, digest-pinned policies; their absence keeps the profile
  blocked.
- Future registration requires new schemas/profiles, exact domains and
  exclusions, key usages, errors, generated bindings, persistence-port contract,
  and new vectors.
- D-016 remains open. D-022 remains blocking. CA-1 through CA-8 remain blocked.

## Alternatives considered

### Treat Event or transition metadata as the audit record

Rejected. Open payload/metadata cannot fix mandatory audit fields, continuity,
retention, export, or atomic responsibility.

### One global mixed-tenant stream

Rejected. It expands leakage and failure blast radius and makes tenant export
and compartment proof unnecessarily risky.

### One stream per object

Rejected. It fragments decision/Effect closure and cannot efficiently prove a
complete management-domain history.

### Merkle checkpoints or mandatory external WORM in the first profile

Not selected. Both can strengthen large-range or off-platform proofs but add
material implementation, recovery, key, and operational complexity not fixed by
current facts. They require a later versioned profile.

### A universal retention duration

Rejected. No current machine or owner fact uniquely supplies a lawful number
for every tenant, jurisdiction, risk class, and record kind. A pinned policy is
mandatory and cannot shorten stronger obligations.

### Redact or delete authoritative bytes in place

Rejected. It destroys continuity and permits caller-controlled evidence
rewriting. Derived redacted views and verifiable compaction tombstones preserve
the authority boundary.

### Unsigned export manifest

Rejected. Source checkpoints prove stream state but do not independently bind
the export authorization, filter, redaction, and assembled manifest identity.

## Rollback and failure strategy

If independent review rejects a selection, no machine asset changes. Keep the
audit family unregistered, keep old Events/records/rows as their existing types,
preserve all vector and Profile states, and revise the docs-only decision before
registration. Any correction after publication creates a new SemVer/digest and
migration note; it never rewrites a pinned identity.
