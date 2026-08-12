---
doc_id: dev.store-migrations
locale: en
kind: reference
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-store/src/personal_db.rs
    symbols: ["authority_migration_plan", "prepare_personal_databases"]
  - path: crates/cognitive-store/src/migration.rs
    symbols: ["execute_sqlite_migration_plan"]
  - path: crates/cognitive-store/src/sqlite/store.rs
    symbols: ["SqliteAuthorityStore"]
  - path: crates/cognitive-store/src/scheduler.rs
    symbols: ["SchedulerRepository", "acquire_eligible_lease"]
tests:
  - crates/cognitive-store/tests/p1_t01_layout_migrations.rs
  - crates/cognitive-store/tests/m2_acceptance.rs
  - crates/cognitive-store/tests/p2_t03_worker_authorization.rs
fingerprint: "sha256:be0b8ab807fc1890b60d2e3782bd85f7a72eb640274cb7898b17e3ed2b0e7dd7"
non_claims:
  - Cross-database atomicity between authority and installation SQLite files is explicitly not claimed.
---

# Store and migrations

`cognitive-store` is the single-writer SQLite WAL adapter behind the kernel ports.
Two databases under XDG state: **authority** (migrations v1–v23) and
**installation** (v1–v4). No cross-database atomicity is claimed; preparation
orders authority first and names the backup path on a second-phase failure.

## Authority migration map (v1–v23)

| Versions | Adds |
|---|---|
| v1 | governed objects (CAS rows), append-only events/records, budgets, outbox, intents (idempotency-unique), fencing singleton, checkpoints, user intents, interpretations, task contracts, loop progress facts |
| v2–v3 | scheduler entries; v3 rebuilds to PK `(task_ref, contract_epoch)` preserving leases |
| v4–v9 | operation candidate proposals, daemon operation descriptors + authorization snapshots, worker iteration authorizations (WIA) with one-time consumption and scheduler-lease bindings |
| v10–v11 | fixed post-states, verification requests/reports, continuation authorizations + lease-bound consumption |
| v12–v15 | context requests/views, workspace context sources (role/trust CHECKs), authorization/revocation fact sets, scheduler execution policies |
| v16–v20 | Memory candidates/decisions/objects, FTS5 derived index, tombstones (forget → +expire → +supersede), version lineage |
| v21–v23 | Skill packages/revisions/bindings, binding revocations, revision lineage |

Nearly every durable table carries BEFORE UPDATE/DELETE triggers that abort with
"append-only"; the only derived table is `memory_search_fts` (rebuildable; searches
run an authority-filter CTE before `MATCH`).

**Load-bearing nuance**: `SqliteAuthorityStore::open` bootstraps schema constants
v1–v17 only; v18–v23 tables exist only after `prepare_personal_databases` runs the
versioned plan (production paths and P4 tests always do).

## Migration engine

Plans are validated (strictly increasing versions, digest self-consistency) before
any side effect. `DryRun` executes against a `VACUUM INTO` scratch copy; `Apply`
writes a timestamped backup, then runs all pending migrations inside **one**
immediate transaction with recorded-row digest verification, replay-skip safety,
and a `PRAGMA quick_check` gate before commit. Preparation holds an exclusive
`migration.lock` (stale lock after a crash requires manual removal).

## Concurrency model

One `Mutex<Connection>` per store instance; WAL + `synchronous=FULL` asserted at
open; read-only opens model degraded volumes (writes fail closed as
`STATE_STORE_UNAVAILABLE`, reads and replay stay alive). Scheduler leases are
transactional CAS: eligibility requires `runnable` past `next_eligible` or an
expired lease reclaimed at a strictly higher epoch; release demands the exact
`(owner, epoch)`; consumption of WIA/continuation authority is bound to the exact
active leased row in the same transaction.
