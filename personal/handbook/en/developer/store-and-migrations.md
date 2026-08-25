---
doc_id: dev.store-migrations
locale: en
kind: reference
audience: [developer]
status: implemented
generated: false
sources:
  - path: personal/crates/cognitive-store/src/personal_backup.rs
    symbols: ["write_personal_backup_archive", "restore_personal_backup_archive"]
  - path: personal/crates/cognitive-store/src/personal_db.rs
    symbols: ["authority_migration_plan", "prepare_personal_databases"]
  - path: personal/crates/cognitive-store/src/migration.rs
    symbols: ["execute_sqlite_migration_plan"]
  - path: personal/crates/cognitive-store/src/provider_control_plane.rs
  - path: personal/crates/cognitive-store/src/sqlite/store.rs
    symbols: ["SqliteAuthorityStore"]
  - path: personal/crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
  - path: personal/crates/cognitive-store/src/scheduler.rs
    symbols: ["SchedulerRepository", "acquire_eligible_lease"]
tests:
  - personal/crates/cognitive-store/tests/p1_t01_layout_migrations.rs
  - personal/crates/cognitive-store/tests/p8_t13_provider_store.rs
  - personal/crates/cognitive-store/tests/m2_acceptance.rs
  - personal/crates/cognitive-store/tests/p2_t03_worker_authorization.rs
fingerprint: "sha256:1427222bbb2f4cf3873cc9af6067e4ec3e38d0aed97adc4a1c51b2ceb6b30bf5"
non_claims:
  - Cross-database atomicity between authority and installation SQLite files is explicitly not claimed.
---

# Store and migrations

`cognitive-store` is the single-writer SQLite WAL adapter behind the kernel ports.
`SqliteAuthorityStore` is cloneable: clones share one connection mutex so the
Personal daemon can hand the same writer to HTTP Task admission and the periodic
scheduler tick. Two databases under XDG state: **authority** (migrations v1–v25) and
**installation** (v1–v4). No cross-database atomicity is claimed; preparation
orders authority first and names the backup path on a second-phase failure.

## Authority migration map (v1–v25)

| Versions | Adds |
|---|---|
| v1 | governed objects (CAS rows), append-only events/records, budgets, outbox, intents (idempotency-unique), fencing singleton, checkpoints, user intents, interpretations, task contracts, loop progress facts |
| v2–v3 | scheduler entries; v3 rebuilds to PK `(task_ref, contract_epoch)` preserving leases |
| v4–v9 | operation candidate proposals, daemon operation descriptors + authorization snapshots, worker iteration authorizations (WIA) with one-time consumption and scheduler-lease bindings |
| v10–v11 | fixed post-states, verification requests/reports, continuation authorizations + lease-bound consumption |
| v12–v15 | context requests/views, workspace context sources (role/trust CHECKs), authorization/revocation fact sets, scheduler execution policies |
| v16–v20 | Memory candidates/decisions/objects, FTS5 derived index, tombstones (forget → +expire → +supersede), version lineage |
| v21–v23 | Skill packages/revisions/bindings, binding revocations, revision lineage |
| v24 | append-only Memory/Skill consumption records keyed by Task/epoch/request/session |
| v25 | Provider Control Plane accounts, models, bindings, usage events/aggregates, budgets, alerts, audit |

Nearly every durable table carries BEFORE UPDATE/DELETE triggers that abort with
"append-only"; the only derived table is `memory_search_fts` (rebuildable; searches
run an authority-filter CTE before `MATCH`).

**Load-bearing nuance**: `SqliteAuthorityStore::open` bootstraps schema constants
v1–v17 only; v18–v25 tables exist only after `prepare_personal_databases` runs the
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

Task admission reuses those existing v1–v3 tables; it adds no migration or
parallel scheduler. `insert_task_contract_with_execution_bootstrap` repeats the
writer-fence and contract-epoch CAS inside one immediate authority transaction,
then inserts the TaskContract event, registered `START` Loop admission/event,
its governed Task projection at registered `DRAFT` without a second
`(object_id, INITIAL)` event, hard Budget, and
`(task_ref, contract_epoch)` runnable scheduler row. A conflict in any late
member rolls the earlier inserts back; a crash after a successful commit
reopens all five prerequisites. Startup recovery can idempotently repair an
older current contract missing only Task, Loop, Budget, or scheduler work in one
fenced transaction. Existing rows are validated and never replaced or reset;
stale contract epochs cannot be repaired.

Verification start reuses the existing fixed-post-state/request tables and adds
no migration. One immediate transaction verifies the writer, current contract,
closed Effect version, shared row bindings, and Loop CAS, then inserts both
append-only rows and commits `ACT -> VERIFY`; any late conflict rolls everything
back.

Verified Task completion adds no migration or dedicated acceptance table.
Canonical decision bytes live in Artifact CAS. Two immediate transactions reuse
the existing governed-object/event/transition-record tables and recheck current
contract, exact fixed state, newest report, complete closed Effect set, and
fencing before the candidate and final acceptance CAS updates.

Resource Manager list/inspect helpers (`list_non_tombstoned_memory_objects`,
`load_non_tombstoned_memory_object`, `list_skill_bindings`) are inherent store
reads over those same v16–v23 rows. They add no migration and invent no seventh
family table.

## User backup archive

`write_personal_backup_archive` copies config/data/state/artifact files into a
digest-bound directory archive and writes a Memory/Skill export sidecar. It
skips `authority.sqlite`, secret-named paths, and `provider-config.json`.
Restore preflight checks schema, completeness, and part digests, then overlays
live files from a staging tree with snapshot rollback. Focused tests record
byte-equal restored files and a finite restore wall time as hypothesis-only
facts. This is not a SQLite dump and does not claim Gate/RTO/RPO results.
