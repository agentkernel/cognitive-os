# ADR-0002: SQLite (WAL) as the First Authoritative Store

- Status: Accepted for the reference implementation baseline
- Date: 2026-07-20
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: reference implementation decision. This ADR binds this
  repository's implementation only; it is NOT a CognitiveOS specification
  requirement. The specification constrains store *behavior* (atomicity,
  fail-closed durability, CAS), not store technology.

## Context

The v0.1 target is single-node R0/R1 (`docs/plan/DEVELOPMENT-PLAN.md`). The
store must provide: atomic commit of state change + event append in one
transaction (`state-and-transition-contract.md`), compare-and-swap on
object versions (`STATE_CONFLICT`), an append-only event log, an outbox for
at-least-once delivery, snapshots, and a provable fail-before-effect path
when persistence is unavailable (`STATE_STORE_UNAVAILABLE`,
vector `state-store-degradation.json`, [REQ-REC-003]). Crash-recovery tests
(M4) must be able to kill the process at arbitrary points and replay.

## Decision

SQLite in WAL mode, accessed through `rusqlite` from `cognitive-store`
(dependency added in M2), is the first transactional store for governed
objects, events, Effects, outbox and snapshots.

Binding rules for the adapter:

1. One SQLite transaction per authoritative commit: object row CAS update +
   event append + outbox insert commit together or not at all.
2. `PRAGMA journal_mode=WAL`, `synchronous=FULL` on authority databases;
   durability shortcuts are allowed only in tests that say so.
3. CAS is enforced with `WHERE version = ?expected`; zero affected rows maps
   to `STATE_CONFLICT` without side effects.
4. Any failed commit surfaces `STATE_STORE_UNAVAILABLE` and the operation
   fails closed before dispatch; no in-memory buffering of governed writes.
5. Single writer connection per authority database; readers use snapshots.
   This models the single-node authority; distributed stores are out of
   scope until M9.

## Alternatives considered

### PostgreSQL from day one

Rejected for v0.1: an external server complicates the install story
(C0/C1 single-node), Windows CI, and crash-point fault injection. The port
traits keep a Postgres adapter possible later without kernel changes.

### Embedded KV store (sled, redb, RocksDB)

Rejected: multi-table transactional semantics, ad-hoc queries for
inspection tooling (admin-cli), and mature crash-recovery behavior are
exactly SQLite's strengths; KV stores would re-implement transactions.

### Event-sourcing-only (log as the sole truth, no object tables)

Rejected as a store topology: authoritative current-state rows with CAS are
required for cheap deterministic gates; the event log remains the
reconstruction source (replay-digest test, M2), which is compatible with
object tables.

## Consequences

Single-node write throughput is bounded by the single writer; acceptable for
R0/R1. Backup/restore and file-permission handling become part of the M6
readiness case. Fault injection can use file-level tricks (read-only remount,
disk-full simulation) to hit the fail-closed path. Migration to another
store later must reproduce the same port-level semantics and pass the same
vectors — technology change, contract constant.

## Compliance checks

M2 exit: concurrent CAS test (exactly one winner), atomic state+event
commit test, replay-digest stability, no-in-place-event-edit negative.
M4 exit: crash-point suite and `state-store-degradation` fail-closed
evidence under `artifacts/evidence/faults/`.
