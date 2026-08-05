# P3-T01/D01 real Context source and revocation blocker

- **Date:** 2026-08-05
- **Slice:** `P3-T01/D01`
- **Branch:** `lane/ctr-p3-t01-context-request-binding`
- **Lease:** `lease/personal/P3-T01/context-request-binding`
- **Status:** `blocked` for real source/retrieval completion; durable request/view
  admission and daemon-issued TaskContract binding remain implemented and tested.

## Completed durable boundary

The daemon now issues a sealed immutable `ContextRequest` before TaskContract
admission, persists it append-only, and binds its exact version-one strong
reference into the v0.4 TaskContract. Context request/view rows reject
unsealed payloads, row/payload mismatch, and a view whose request strong
reference does not match the persisted request digest.

Exact native Linux evidence is recorded in `PROGRESS.md`; this checkpoint does
not claim P3-T01 acceptance, B03, P2-T04 unblocking, Tool execution, progress,
evidence, acceptance, or Task completion.

## Non-negotiable remaining source boundary

P3-T01 requires real workspace, task, and evidence Context sources with
authorization before ranking and revocation negatives. The existing resolver
already enforces authorization before ranker body access when supplied a valid
`AuthzSnapshot`, but the daemon lacks two authoritative inputs required to
compose that snapshot and those candidates safely:

1. **Workspace Context ingestion:** There is no daemon-admitted, append-only
   workspace object with governed header, strong identity/digest, scope,
   conversation binding, provenance, and body representation. Existing
   `file:///workspace/...` values are operation/test targets, not governed
   Context objects. Pi text, client request bodies, and arbitrary filesystem
   reads must not be promoted to Context authority.
2. **Reconstructable authorization currency:**
   `DaemonAuthorizationSnapshotRow` records a completed authorization
   observation, not capability links, actor chain, membership, explicit deny
   facts, or a reusable decision. It cannot safely reconstruct an
   `AuthzSnapshot` after revocation. Creating an implicit daemon-root allow
   would bypass the capability/revocation model and is forbidden.

## Required next design and implementation exit

A subsequent owned P3 Context source slice must add, before attempting a real
Context resolution caller:

1. a daemon-only append-only workspace Context admission record with explicit
   provenance/trust classification and scope/conversation metadata;
2. narrow metadata-first task/evidence/workspace candidate queries, so body
   reads occur only after tenant/conversation filtering and authorization;
3. a durable authorization-snapshot source that reconstructs current
   actor-chain, membership, capability, explicit-deny, and revocation facts;
4. regressions proving revoked candidates never reach body loading or ranking,
   and required revoked Context fails closed without replacing an earlier
   immutable view.

The active P2-T04 lease retains `crates/cognitive-kernel/src/context.rs` and
the scheduler/Pi runtime boundary. P3 work must not modify those paths until
the leases are explicitly reconciled.
