//! SQLite (WAL) authority store adapter — the reference implementation of
//! the `cognitive-kernel` [`AuthorityStore`] port (ADR-0002).
//!
//! Binding rules implemented here (ADR-0002, all five):
//!
//! 1. One SQLite transaction per authoritative commit: object CAS update +
//!    event append + transition record + optional budget debit + outbox
//!    rows commit together or not at all.
//! 2. `PRAGMA journal_mode=WAL`, `synchronous=FULL` on authority databases
//!    (asserted at open; tests that shortcut durability must say so).
//! 3. CAS is enforced with `WHERE version = ?expected`; zero affected rows
//!    map to [`StorePortError::Conflict`] without side effects.
//! 4. Any failed commit surfaces [`StorePortError::Unavailable`]
//!    (`STATE_STORE_UNAVAILABLE` at the kernel gate) and fails closed;
//!    governed writes are never buffered in memory (REQ-REC-003).
//! 5. Single writer connection per authority database (the connection sits
//!    behind a mutex; readers can open read-only snapshots).
//!
//! Append-only enforcement (REQ-EVT-004) lives in the STORAGE layer:
//! `BEFORE UPDATE` / `BEFORE DELETE` triggers on `events` and
//! `transition_records` abort any rewrite attempt, from any connection.

use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, StateName, Version, WallTimestamp,
};
use cognitive_kernel::BudgetState;
use cognitive_kernel::ports::{
    AuthorityStore, CheckpointRow, CommitReceipt, CommittedEvent, HarnessStore, IntentChainStore,
    IntentRow, InterpretationRow, ObjectAdmission, OutboxEntry, ProgressFactRow, ProtocolStore,
    StorePortError, StoredBudget, StoredObject, TaskBinding, TaskContractRow, TransitionCommit,
    UserIntentRecordRow,
};
use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

/// Schema of the authority database. Two structural guarantees matter to
/// the contract: the event log and transition records are append-only
/// (triggers), and versions are positive integers (CHECK).
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS governed_objects (
  object_id  TEXT PRIMARY KEY,
  domain     TEXT NOT NULL,
  state      TEXT NOT NULL,
  version    INTEGER NOT NULL CHECK (version >= 1),
  body_json  TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS events (
  sequence       INTEGER PRIMARY KEY AUTOINCREMENT,
  event_id       TEXT NOT NULL UNIQUE,
  object_id      TEXT NOT NULL,
  domain         TEXT NOT NULL,
  object_version INTEGER NOT NULL CHECK (object_version >= 1),
  event_type     TEXT NOT NULL,
  canonical_json TEXT NOT NULL,
  UNIQUE (object_id, object_version)
) STRICT;

CREATE TRIGGER IF NOT EXISTS events_append_only_update
BEFORE UPDATE ON events
BEGIN SELECT RAISE(ABORT, 'append-only: committed events are immutable'); END;

CREATE TRIGGER IF NOT EXISTS events_append_only_delete
BEFORE DELETE ON events
BEGIN SELECT RAISE(ABORT, 'append-only: committed events are immutable'); END;

CREATE TABLE IF NOT EXISTS transition_records (
  record_seq     INTEGER PRIMARY KEY AUTOINCREMENT,
  record_id      TEXT NOT NULL UNIQUE,
  object_id      TEXT NOT NULL,
  domain         TEXT NOT NULL,
  object_version INTEGER NOT NULL CHECK (object_version >= 1),
  canonical_json TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS records_append_only_update
BEFORE UPDATE ON transition_records
BEGIN SELECT RAISE(ABORT, 'append-only: committed records are immutable'); END;

CREATE TRIGGER IF NOT EXISTS records_append_only_delete
BEFORE DELETE ON transition_records
BEGIN SELECT RAISE(ABORT, 'append-only: committed records are immutable'); END;

CREATE TABLE IF NOT EXISTS budgets (
  budget_id  TEXT PRIMARY KEY,
  state_json TEXT NOT NULL,
  version    INTEGER NOT NULL CHECK (version >= 1),
  created_at TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS outbox (
  outbox_sequence INTEGER PRIMARY KEY AUTOINCREMENT,
  event_id        TEXT NOT NULL REFERENCES events(event_id),
  destination     TEXT NOT NULL,
  dispatched_at   TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS intents (
  intent_id              TEXT PRIMARY KEY,
  idempotency_key        TEXT NOT NULL UNIQUE,
  parameters_digest      TEXT NOT NULL,
  action                 TEXT NOT NULL,
  target                 TEXT NOT NULL,
  effect_object_id       TEXT NOT NULL UNIQUE,
  expected_state_version INTEGER NOT NULL,
  grant_epoch            INTEGER NOT NULL,
  capability_set_version INTEGER NOT NULL,
  task_ref               TEXT,
  contract_epoch         INTEGER,
  canonical_json         TEXT NOT NULL,
  CHECK ((task_ref IS NULL) = (contract_epoch IS NULL))
) STRICT;

CREATE TRIGGER IF NOT EXISTS intents_append_only_update
BEFORE UPDATE ON intents
BEGIN SELECT RAISE(ABORT, 'append-only: persisted intents are immutable'); END;

CREATE TRIGGER IF NOT EXISTS intents_append_only_delete
BEFORE DELETE ON intents
BEGIN SELECT RAISE(ABORT, 'append-only: persisted intents are immutable'); END;

CREATE TABLE IF NOT EXISTS fencing (
  id    INTEGER PRIMARY KEY CHECK (id = 1),
  epoch INTEGER NOT NULL CHECK (epoch >= 1)
) STRICT;

INSERT OR IGNORE INTO fencing (id, epoch) VALUES (1, 1);

CREATE TABLE IF NOT EXISTS checkpoints (
  checkpoint_seq       INTEGER PRIMARY KEY AUTOINCREMENT,
  checkpoint_id        TEXT NOT NULL UNIQUE,
  loop_object_id       TEXT NOT NULL,
  event_high_watermark INTEGER NOT NULL,
  fencing_epoch        INTEGER NOT NULL,
  canonical_json       TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS checkpoints_append_only_update
BEFORE UPDATE ON checkpoints
BEGIN SELECT RAISE(ABORT, 'append-only: checkpoints are immutable'); END;

CREATE TRIGGER IF NOT EXISTS checkpoints_append_only_delete
BEFORE DELETE ON checkpoints
BEGIN SELECT RAISE(ABORT, 'append-only: checkpoints are immutable'); END;

CREATE TABLE IF NOT EXISTS user_intent_records (
  record_seq                 INTEGER PRIMARY KEY AUTOINCREMENT,
  record_id                  TEXT NOT NULL UNIQUE,
  conversation_or_scope_ref  TEXT NOT NULL,
  actor_chain_digest         TEXT NOT NULL,
  raw_expression             TEXT NOT NULL,
  recorded_at                TEXT NOT NULL,
  intent_authority_ref       TEXT NOT NULL,
  intent_digest              TEXT NOT NULL,
  canonical_json             TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS user_intents_append_only_update
BEFORE UPDATE ON user_intent_records
BEGIN SELECT RAISE(ABORT, 'append-only: user intent records are immutable'); END;

CREATE TRIGGER IF NOT EXISTS user_intents_append_only_delete
BEFORE DELETE ON user_intent_records
BEGIN SELECT RAISE(ABORT, 'append-only: user intent records are immutable'); END;

CREATE TABLE IF NOT EXISTS intent_interpretations (
  interpretation_seq          INTEGER PRIMARY KEY AUTOINCREMENT,
  interpretation_id           TEXT NOT NULL UNIQUE,
  user_intent_record_id       TEXT NOT NULL,
  recorded_status             TEXT NOT NULL CHECK (recorded_status IN ('candidate','clarification_required')),
  material_ambiguity_count    INTEGER NOT NULL CHECK (material_ambiguity_count >= 0),
  supersedes_interpretation   TEXT,
  interpretation_digest       TEXT NOT NULL,
  canonical_json              TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS interpretations_append_only_update
BEFORE UPDATE ON intent_interpretations
BEGIN SELECT RAISE(ABORT, 'append-only: interpretation candidates are immutable'); END;

CREATE TRIGGER IF NOT EXISTS interpretations_append_only_delete
BEFORE DELETE ON intent_interpretations
BEGIN SELECT RAISE(ABORT, 'append-only: interpretation candidates are immutable'); END;

CREATE TABLE IF NOT EXISTS task_contracts (
  contract_seq           INTEGER PRIMARY KEY AUTOINCREMENT,
  contract_id            TEXT NOT NULL UNIQUE,
  task_ref               TEXT NOT NULL,
  contract_epoch         INTEGER NOT NULL CHECK (contract_epoch >= 1),
  user_intent_record_id  TEXT NOT NULL,
  interpretation_id      TEXT NOT NULL,
  accepted_by            TEXT NOT NULL,
  contract_digest        TEXT NOT NULL,
  canonical_json         TEXT NOT NULL,
  UNIQUE (task_ref, contract_epoch)
) STRICT;

CREATE TRIGGER IF NOT EXISTS task_contracts_append_only_update
BEFORE UPDATE ON task_contracts
BEGIN SELECT RAISE(ABORT, 'append-only: task contracts are immutable'); END;

CREATE TRIGGER IF NOT EXISTS task_contracts_append_only_delete
BEFORE DELETE ON task_contracts
BEGIN SELECT RAISE(ABORT, 'append-only: task contracts are immutable'); END;

CREATE TABLE IF NOT EXISTS loop_progress_facts (
  progress_seq        INTEGER PRIMARY KEY AUTOINCREMENT,
  loop_object_id      TEXT NOT NULL,
  iteration           INTEGER NOT NULL CHECK (iteration >= 1),
  status              TEXT NOT NULL CHECK (status IN ('advanced','none','uncertain','blocked')),
  action_fingerprint  TEXT NOT NULL,
  evidence_refs_json  TEXT NOT NULL,
  recorded_at         TEXT NOT NULL,
  fencing_epoch       INTEGER NOT NULL,
  UNIQUE (loop_object_id, iteration)
) STRICT;

CREATE TRIGGER IF NOT EXISTS progress_facts_append_only_update
BEFORE UPDATE ON loop_progress_facts
BEGIN SELECT RAISE(ABORT, 'append-only: progress facts are immutable'); END;

CREATE TRIGGER IF NOT EXISTS progress_facts_append_only_delete
BEFORE DELETE ON loop_progress_facts
BEGIN SELECT RAISE(ABORT, 'append-only: progress facts are immutable'); END;
";

/// SQLite-backed [`AuthorityStore`].
pub struct SqliteAuthorityStore {
    conn: Mutex<Connection>,
}

fn unavailable(what: &str) -> impl FnOnce(rusqlite::Error) -> StorePortError + '_ {
    move |err| StorePortError::Unavailable {
        detail: format!("{what}: {err}"),
    }
}

fn is_constraint_violation(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(failure, _)
            if failure.code == rusqlite::ErrorCode::ConstraintViolation
    )
}

fn corrupt(what: &str, err: impl std::fmt::Display) -> StorePortError {
    StorePortError::Unavailable {
        detail: format!("stored value unusable ({what}): {err}"),
    }
}

impl SqliteAuthorityStore {
    /// Open (creating if needed) an authority database in WAL mode with
    /// `synchronous=FULL`, and install the schema.
    pub fn open(path: &Path) -> Result<Self, StorePortError> {
        let conn = Connection::open(path).map_err(unavailable("open"))?;
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .map_err(unavailable("set journal_mode"))?;
        if !journal_mode.eq_ignore_ascii_case("wal") {
            return Err(StorePortError::Unavailable {
                detail: format!("authority database refused WAL mode: {journal_mode}"),
            });
        }
        conn.execute_batch(
            "PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;",
        )
        .map_err(unavailable("set pragmas"))?;
        conn.execute_batch(SCHEMA)
            .map_err(unavailable("install schema"))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open an existing authority database read-only (reader snapshot per
    /// ADR-0002 rule 5; also models a degraded read-only volume: every
    /// governed write through this handle fails closed).
    pub fn open_read_only(path: &Path) -> Result<Self, StorePortError> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(unavailable("open read-only"))?;
        conn.execute_batch("PRAGMA query_only=ON; PRAGMA busy_timeout=5000;")
            .map_err(unavailable("set read-only pragmas"))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, StorePortError> {
        self.conn.lock().map_err(|_| StorePortError::Unavailable {
            detail: "authority connection poisoned".to_owned(),
        })
    }
}

/// F-014 sink-side fencing: verify the writer's declared epoch against the
/// fencing table INSIDE the transaction; a stale writer is a conflict and
/// the whole atomic unit rolls back.
fn verify_fencing_in_tx(tx: &Transaction<'_>, declared: Option<i64>) -> Result<(), StorePortError> {
    let Some(declared) = declared else {
        return Ok(());
    };
    let current: i64 = tx
        .query_row("SELECT epoch FROM fencing WHERE id = 1", [], |row| {
            row.get(0)
        })
        .map_err(unavailable("read fencing epoch"))?;
    if declared != current {
        return Err(StorePortError::Conflict {
            detail: format!("writer fenced: declared epoch {declared} != current {current}"),
        });
    }
    Ok(())
}

fn row_to_object(
    object_id: String,
    domain: String,
    state: String,
    version: i64,
    body_json: String,
) -> Result<StoredObject, StorePortError> {
    Ok(StoredObject {
        object_id: ObjectId::parse(&object_id).map_err(|err| corrupt("object_id", err))?,
        domain: LifecycleDomain::parse(&domain).map_err(|err| corrupt("domain", err))?,
        state: StateName::parse(&state).map_err(|err| corrupt("state", err))?,
        version: Version::new(version).map_err(|err| corrupt("version", err))?,
        body: serde_json::from_str(&body_json).map_err(|err| corrupt("body_json", err))?,
    })
}

impl AuthorityStore for SqliteAuthorityStore {
    fn load_object(
        &self,
        domain: LifecycleDomain,
        object_id: &ObjectId,
    ) -> Result<Option<StoredObject>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT object_id, domain, state, version, body_json
                 FROM governed_objects WHERE object_id = ?1 AND domain = ?2",
            )
            .map_err(unavailable("prepare load_object"))?;
        let mut rows = statement
            .query((object_id.as_str(), domain.as_str()))
            .map_err(unavailable("query load_object"))?;
        match rows.next().map_err(unavailable("read load_object"))? {
            None => Ok(None),
            Some(row) => {
                let object = row_to_object(
                    row.get(0).map_err(unavailable("column object_id"))?,
                    row.get(1).map_err(unavailable("column domain"))?,
                    row.get(2).map_err(unavailable("column state"))?,
                    row.get(3).map_err(unavailable("column version"))?,
                    row.get(4).map_err(unavailable("column body_json"))?,
                )?;
                Ok(Some(object))
            }
        }
    }

    fn admit_object(&self, admission: &ObjectAdmission) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin admission"))?;
        verify_fencing_in_tx(&tx, admission.fencing_epoch)?;
        let object = &admission.object;
        let body_json = serde_json::to_string(&object.body)
            .map_err(|err| corrupt("body serialization", err))?;
        let inserted = tx.execute(
            "INSERT INTO governed_objects
               (object_id, domain, state, version, body_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            (
                object.object_id.as_str(),
                object.domain.as_str(),
                object.state.as_str(),
                object.version.get(),
                body_json,
                admission.admitted_at.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!("object {} already exists", object.object_id),
                });
            }
            Err(err) => return Err(unavailable("insert object")(err)),
        }
        let event = &admission.event;
        tx.execute(
            "INSERT INTO events
               (event_id, object_id, domain, object_version, event_type, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                event.event_id.as_str(),
                event.object_id.as_str(),
                event.domain.as_str(),
                event.object_version.get(),
                event.event_type.as_str(),
                event.canonical_json.as_str(),
            ),
        )
        .map_err(unavailable("append admission event"))?;
        let sequence = tx.last_insert_rowid();
        for outbox in &admission.outbox {
            tx.execute(
                "INSERT INTO outbox (event_id, destination) VALUES (?1, ?2)",
                (outbox.event_id.as_str(), outbox.destination.as_str()),
            )
            .map_err(unavailable("insert outbox row"))?;
        }
        tx.commit().map_err(unavailable("commit admission"))?;
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn commit_transition(
        &self,
        commit: &TransitionCommit,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin transition"))?;
        verify_fencing_in_tx(&tx, commit.fencing_epoch)?;

        // ADR-0002 rule 3: CAS via WHERE version = expected (plus identity,
        // domain and source state); zero affected rows -> Conflict, and the
        // dropped transaction rolls back with no side effects.
        let cas = &commit.cas;
        let changed = tx
            .execute(
                "UPDATE governed_objects
                 SET state = ?1, version = ?2, updated_at = ?3
                 WHERE object_id = ?4 AND domain = ?5 AND state = ?6 AND version = ?7",
                (
                    cas.to_state.as_str(),
                    cas.next_version.get(),
                    cas.committed_at.as_str(),
                    cas.object_id.as_str(),
                    cas.domain.as_str(),
                    cas.from_state.as_str(),
                    cas.expected_version.get(),
                ),
            )
            .map_err(unavailable("object cas"))?;
        if changed == 0 {
            return Err(StorePortError::Conflict {
                detail: format!(
                    "object cas raced: {} not at {}/v{}",
                    cas.object_id, cas.from_state, cas.expected_version
                ),
            });
        }

        // Hard-budget debit joins the same transaction, directly after the
        // object CAS: a later statement failure rolls BOTH back together.
        if let Some(budget) = &commit.budget {
            let changed = tx
                .execute(
                    "UPDATE budgets SET state_json = ?1, version = ?2
                     WHERE budget_id = ?3 AND version = ?4",
                    (
                        budget.next_state_canonical_json.as_str(),
                        budget.next_version.get(),
                        budget.budget_id.as_str(),
                        budget.expected_version.get(),
                    ),
                )
                .map_err(unavailable("budget cas"))?;
            if changed == 0 {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "budget cas raced: {} not at v{}",
                        budget.budget_id, budget.expected_version
                    ),
                });
            }
        }

        let event = &commit.event;
        tx.execute(
            "INSERT INTO events
               (event_id, object_id, domain, object_version, event_type, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                event.event_id.as_str(),
                event.object_id.as_str(),
                event.domain.as_str(),
                event.object_version.get(),
                event.event_type.as_str(),
                event.canonical_json.as_str(),
            ),
        )
        .map_err(unavailable("append event"))?;
        let sequence = tx.last_insert_rowid();

        let record = &commit.record;
        tx.execute(
            "INSERT INTO transition_records
               (record_id, object_id, domain, object_version, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                record.record_id.as_str(),
                record.object_id.as_str(),
                record.domain.as_str(),
                record.object_version.get(),
                record.canonical_json.as_str(),
            ),
        )
        .map_err(unavailable("append transition record"))?;

        for outbox in &commit.outbox {
            tx.execute(
                "INSERT INTO outbox (event_id, destination) VALUES (?1, ?2)",
                (outbox.event_id.as_str(), outbox.destination.as_str()),
            )
            .map_err(unavailable("insert outbox row"))?;
        }

        tx.commit().map_err(unavailable("commit transition"))?;
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn load_budget(&self, budget_id: &BudgetId) -> Result<Option<StoredBudget>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached("SELECT state_json, version FROM budgets WHERE budget_id = ?1")
            .map_err(unavailable("prepare load_budget"))?;
        let mut rows = statement
            .query((budget_id.as_str(),))
            .map_err(unavailable("query load_budget"))?;
        match rows.next().map_err(unavailable("read load_budget"))? {
            None => Ok(None),
            Some(row) => {
                let state_json: String = row.get(0).map_err(unavailable("column state_json"))?;
                let version: i64 = row.get(1).map_err(unavailable("column version"))?;
                let state: BudgetState = serde_json::from_str(&state_json)
                    .map_err(|err| corrupt("budget state", err))?;
                Ok(Some(StoredBudget {
                    budget_id: budget_id.clone(),
                    state,
                    version: Version::new(version).map_err(|err| corrupt("budget version", err))?,
                }))
            }
        }
    }

    fn create_budget(
        &self,
        budget_id: &BudgetId,
        state_canonical_json: &str,
        created_at: &WallTimestamp,
    ) -> Result<(), StorePortError> {
        let conn = self.lock()?;
        let inserted = conn.execute(
            "INSERT INTO budgets (budget_id, state_json, version, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            (
                budget_id.as_str(),
                state_canonical_json,
                Version::INITIAL.get(),
                created_at.as_str(),
            ),
        );
        match inserted {
            Ok(_) => Ok(()),
            Err(err) if is_constraint_violation(&err) => Err(StorePortError::Conflict {
                detail: format!("budget {budget_id} already exists"),
            }),
            Err(err) => Err(unavailable("insert budget")(err)),
        }
    }

    fn read_events(
        &self,
        after_sequence: i64,
        limit: usize,
    ) -> Result<Vec<CommittedEvent>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT sequence, event_id, object_id, domain, object_version, event_type,
                        canonical_json
                 FROM events WHERE sequence > ?1 ORDER BY sequence ASC LIMIT ?2",
            )
            .map_err(unavailable("prepare read_events"))?;
        let mut rows = statement
            .query((after_sequence, limit as i64))
            .map_err(unavailable("query read_events"))?;
        let mut events = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read events row"))? {
            let event_id: String = row.get(1).map_err(unavailable("column event_id"))?;
            let object_id: String = row.get(2).map_err(unavailable("column object_id"))?;
            let domain: String = row.get(3).map_err(unavailable("column domain"))?;
            let object_version: i64 = row.get(4).map_err(unavailable("column object_version"))?;
            events.push(CommittedEvent {
                sequence: row.get(0).map_err(unavailable("column sequence"))?,
                event_id: EventId::parse(&event_id).map_err(|err| corrupt("event_id", err))?,
                object_id: ObjectId::parse(&object_id).map_err(|err| corrupt("object_id", err))?,
                domain: LifecycleDomain::parse(&domain).map_err(|err| corrupt("domain", err))?,
                object_version: Version::new(object_version)
                    .map_err(|err| corrupt("object_version", err))?,
                event_type: row.get(5).map_err(unavailable("column event_type"))?,
                canonical_json: row.get(6).map_err(unavailable("column canonical_json"))?,
            });
        }
        Ok(events)
    }

    fn pending_outbox(&self, limit: usize) -> Result<Vec<OutboxEntry>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT outbox_sequence, event_id, destination FROM outbox
                 WHERE dispatched_at IS NULL ORDER BY outbox_sequence ASC LIMIT ?1",
            )
            .map_err(unavailable("prepare pending_outbox"))?;
        let mut rows = statement
            .query((limit as i64,))
            .map_err(unavailable("query pending_outbox"))?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read outbox row"))? {
            let event_id: String = row.get(1).map_err(unavailable("column event_id"))?;
            entries.push(OutboxEntry {
                outbox_sequence: row.get(0).map_err(unavailable("column outbox_sequence"))?,
                event_id: EventId::parse(&event_id).map_err(|err| corrupt("event_id", err))?,
                destination: row.get(2).map_err(unavailable("column destination"))?,
                dispatched: false,
            });
        }
        Ok(entries)
    }

    fn mark_outbox_dispatched(
        &self,
        outbox_sequence: i64,
        dispatched_at: &WallTimestamp,
    ) -> Result<(), StorePortError> {
        let conn = self.lock()?;
        let changed = conn
            .execute(
                "UPDATE outbox SET dispatched_at = ?1
                 WHERE outbox_sequence = ?2 AND dispatched_at IS NULL",
                (dispatched_at.as_str(), outbox_sequence),
            )
            .map_err(unavailable("mark outbox dispatched"))?;
        if changed == 0 {
            return Err(StorePortError::Conflict {
                detail: format!("no pending outbox row {outbox_sequence}"),
            });
        }
        Ok(())
    }
}

fn row_to_intent(row: &rusqlite::Row<'_>) -> Result<IntentRow, rusqlite::Error> {
    let task_ref: Option<String> = row.get(9)?;
    let contract_epoch: Option<i64> = row.get(10)?;
    let task_binding = match (task_ref, contract_epoch) {
        (Some(task_ref), Some(contract_epoch)) => Some(TaskBinding {
            task_ref,
            contract_epoch,
        }),
        _ => None,
    };
    Ok(IntentRow {
        intent_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(0, "intent_id".into(), rusqlite::types::Type::Text)
        })?,
        idempotency_key: row.get(1)?,
        parameters_digest: row.get(2)?,
        action: row.get(3)?,
        target: row.get(4)?,
        effect_object_id: ObjectId::parse(&row.get::<_, String>(5)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                5,
                "effect_object_id".into(),
                rusqlite::types::Type::Text,
            )
        })?,
        expected_state_version: Version::new(row.get(6)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                6,
                "expected_state_version".into(),
                rusqlite::types::Type::Integer,
            )
        })?,
        grant_epoch: row.get(7)?,
        capability_set_version: row.get(8)?,
        task_binding,
        canonical_json: row.get(11)?,
    })
}

const INTENT_COLUMNS: &str = "intent_id, idempotency_key, parameters_digest, action, target, \
     effect_object_id, expected_state_version, grant_epoch, capability_set_version, \
     task_ref, contract_epoch, canonical_json";

impl ProtocolStore for SqliteAuthorityStore {
    fn insert_intent(
        &self,
        intent: &IntentRow,
        event: &cognitive_kernel::ports::EventDraft,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin intent"))?;
        let inserted = tx.execute(
            &format!(
                "INSERT INTO intents ({INTENT_COLUMNS}) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)"
            ),
            (
                intent.intent_id.as_str(),
                intent.idempotency_key.as_str(),
                intent.parameters_digest.as_str(),
                intent.action.as_str(),
                intent.target.as_str(),
                intent.effect_object_id.as_str(),
                intent.expected_state_version.get(),
                intent.grant_epoch,
                intent.capability_set_version,
                intent
                    .task_binding
                    .as_ref()
                    .map(|binding| binding.task_ref.as_str()),
                intent
                    .task_binding
                    .as_ref()
                    .map(|binding| binding.contract_epoch),
                intent.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "intent {} or key {} already persisted",
                        intent.intent_id, intent.idempotency_key
                    ),
                });
            }
            Err(err) => return Err(unavailable("insert intent")(err)),
        }
        tx.execute(
            "INSERT INTO events
               (event_id, object_id, domain, object_version, event_type, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                event.event_id.as_str(),
                event.object_id.as_str(),
                event.domain.as_str(),
                event.object_version.get(),
                event.event_type.as_str(),
                event.canonical_json.as_str(),
            ),
        )
        .map_err(unavailable("append intent event"))?;
        let sequence = tx.last_insert_rowid();
        tx.commit().map_err(unavailable("commit intent"))?;
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn load_intent_by_key(&self, key: &str) -> Result<Option<IntentRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {INTENT_COLUMNS} FROM intents WHERE idempotency_key = ?1"
            ))
            .map_err(unavailable("prepare load_intent_by_key"))?;
        statement
            .query_row((key,), row_to_intent)
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_intent_by_key")(other)),
            })
    }

    fn load_intent_for_effect(
        &self,
        effect_object_id: &ObjectId,
    ) -> Result<Option<IntentRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {INTENT_COLUMNS} FROM intents WHERE effect_object_id = ?1"
            ))
            .map_err(unavailable("prepare load_intent_for_effect"))?;
        statement
            .query_row((effect_object_id.as_str(),), row_to_intent)
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_intent_for_effect")(other)),
            })
    }

    fn current_fencing_epoch(&self) -> Result<i64, StorePortError> {
        let conn = self.lock()?;
        conn.query_row("SELECT epoch FROM fencing WHERE id = 1", [], |row| {
            row.get(0)
        })
        .map_err(unavailable("read fencing epoch"))
    }

    fn advance_fencing_epoch(&self) -> Result<i64, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin epoch advance"))?;
        tx.execute("UPDATE fencing SET epoch = epoch + 1 WHERE id = 1", [])
            .map_err(unavailable("advance epoch"))?;
        let epoch: i64 = tx
            .query_row("SELECT epoch FROM fencing WHERE id = 1", [], |row| {
                row.get(0)
            })
            .map_err(unavailable("read advanced epoch"))?;
        tx.commit().map_err(unavailable("commit epoch advance"))?;
        Ok(epoch)
    }

    fn list_objects_in_states(
        &self,
        domain: LifecycleDomain,
        states: &[StateName],
    ) -> Result<Vec<StoredObject>, StorePortError> {
        let conn = self.lock()?;
        let placeholders = (0..states.len())
            .map(|index| format!("?{}", index + 2))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT object_id, domain, state, version, body_json FROM governed_objects
             WHERE domain = ?1 AND state IN ({placeholders}) ORDER BY object_id"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(unavailable("prepare list_objects_in_states"))?;
        let mut rows = statement
            .query(rusqlite::params_from_iter(
                std::iter::once(domain.as_str().to_owned())
                    .chain(states.iter().map(|state| state.as_str().to_owned())),
            ))
            .map_err(unavailable("query list_objects_in_states"))?;
        let mut objects = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read objects row"))? {
            let object = row_to_object(
                row.get(0).map_err(unavailable("column object_id"))?,
                row.get(1).map_err(unavailable("column domain"))?,
                row.get(2).map_err(unavailable("column state"))?,
                row.get(3).map_err(unavailable("column version"))?,
                row.get(4).map_err(unavailable("column body_json"))?,
            )?;
            objects.push(object);
        }
        Ok(objects)
    }

    fn append_checkpoint(&self, checkpoint: &CheckpointRow) -> Result<(), StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin checkpoint"))?;
        // F-014 checkpoint sink: the declared epoch must be current.
        verify_fencing_in_tx(&tx, Some(checkpoint.fencing_epoch))?;
        tx.execute(
            "INSERT INTO checkpoints
               (checkpoint_id, loop_object_id, event_high_watermark, fencing_epoch, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                checkpoint.checkpoint_id.as_str(),
                checkpoint.loop_object_id.as_str(),
                checkpoint.event_high_watermark,
                checkpoint.fencing_epoch,
                checkpoint.canonical_json.as_str(),
            ),
        )
        .map_err(unavailable("insert checkpoint"))?;
        tx.commit().map_err(unavailable("commit checkpoint"))?;
        Ok(())
    }

    fn latest_checkpoint(
        &self,
        loop_object_id: &ObjectId,
    ) -> Result<Option<CheckpointRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT checkpoint_id, loop_object_id, event_high_watermark, fencing_epoch,
                        canonical_json
                 FROM checkpoints WHERE loop_object_id = ?1
                 ORDER BY checkpoint_seq DESC LIMIT 1",
            )
            .map_err(unavailable("prepare latest_checkpoint"))?;
        statement
            .query_row((loop_object_id.as_str(),), |row| {
                Ok(CheckpointRow {
                    checkpoint_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            0,
                            "checkpoint_id".into(),
                            rusqlite::types::Type::Text,
                        )
                    })?,
                    loop_object_id: ObjectId::parse(&row.get::<_, String>(1)?).map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            1,
                            "loop_object_id".into(),
                            rusqlite::types::Type::Text,
                        )
                    })?,
                    event_high_watermark: row.get(2)?,
                    fencing_epoch: row.get(3)?,
                    canonical_json: row.get(4)?,
                })
            })
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query latest_checkpoint")(other)),
            })
    }

    fn current_contract_epoch(&self, task_ref: &str) -> Result<i64, StorePortError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT COALESCE(MAX(contract_epoch), 0) FROM task_contracts WHERE task_ref = ?1",
            (task_ref,),
            |row| row.get(0),
        )
        .map_err(unavailable("read current contract epoch"))
    }

    fn load_event_by_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<CommittedEvent>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT sequence, event_id, object_id, domain, object_version, event_type,
                        canonical_json
                 FROM events WHERE event_id = ?1",
            )
            .map_err(unavailable("prepare load_event_by_id"))?;
        let row = statement
            .query_row((event_id.as_str(),), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_event_by_id")(other)),
            })?;
        match row {
            None => Ok(None),
            Some((sequence, event_id, object_id, domain, object_version, event_type, json)) => {
                Ok(Some(CommittedEvent {
                    sequence,
                    event_id: EventId::parse(&event_id).map_err(|err| corrupt("event_id", err))?,
                    object_id: ObjectId::parse(&object_id)
                        .map_err(|err| corrupt("object_id", err))?,
                    domain: LifecycleDomain::parse(&domain)
                        .map_err(|err| corrupt("domain", err))?,
                    object_version: Version::new(object_version)
                        .map_err(|err| corrupt("object_version", err))?,
                    event_type,
                    canonical_json: json,
                }))
            }
        }
    }
}

fn row_to_user_intent(row: &rusqlite::Row<'_>) -> Result<UserIntentRecordRow, rusqlite::Error> {
    Ok(UserIntentRecordRow {
        record_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(0, "record_id".into(), rusqlite::types::Type::Text)
        })?,
        conversation_or_scope_ref: row.get(1)?,
        actor_chain_digest: row.get(2)?,
        raw_expression: row.get(3)?,
        recorded_at: WallTimestamp::parse(&row.get::<_, String>(4)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(4, "recorded_at".into(), rusqlite::types::Type::Text)
        })?,
        intent_authority_ref: row.get(5)?,
        intent_digest: row.get(6)?,
        canonical_json: row.get(7)?,
    })
}

const USER_INTENT_COLUMNS: &str = "record_id, conversation_or_scope_ref, actor_chain_digest, \
     raw_expression, recorded_at, intent_authority_ref, intent_digest, canonical_json";

fn row_to_interpretation(row: &rusqlite::Row<'_>) -> Result<InterpretationRow, rusqlite::Error> {
    let supersedes: Option<String> = row.get(4)?;
    Ok(InterpretationRow {
        interpretation_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                0,
                "interpretation_id".into(),
                rusqlite::types::Type::Text,
            )
        })?,
        user_intent_record_id: ObjectId::parse(&row.get::<_, String>(1)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                1,
                "user_intent_record_id".into(),
                rusqlite::types::Type::Text,
            )
        })?,
        recorded_status: row.get(2)?,
        material_ambiguity_count: row.get(3)?,
        supersedes_interpretation: supersedes
            .map(|raw| {
                ObjectId::parse(&raw).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        4,
                        "supersedes_interpretation".into(),
                        rusqlite::types::Type::Text,
                    )
                })
            })
            .transpose()?,
        interpretation_digest: row.get(5)?,
        canonical_json: row.get(6)?,
    })
}

const INTERPRETATION_COLUMNS: &str = "interpretation_id, user_intent_record_id, recorded_status, \
     material_ambiguity_count, supersedes_interpretation, interpretation_digest, canonical_json";

fn row_to_task_contract(row: &rusqlite::Row<'_>) -> Result<TaskContractRow, rusqlite::Error> {
    Ok(TaskContractRow {
        contract_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(0, "contract_id".into(), rusqlite::types::Type::Text)
        })?,
        task_ref: row.get(1)?,
        contract_epoch: row.get(2)?,
        user_intent_record_id: ObjectId::parse(&row.get::<_, String>(3)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                3,
                "user_intent_record_id".into(),
                rusqlite::types::Type::Text,
            )
        })?,
        interpretation_id: ObjectId::parse(&row.get::<_, String>(4)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                4,
                "interpretation_id".into(),
                rusqlite::types::Type::Text,
            )
        })?,
        accepted_by: row.get(5)?,
        contract_digest: row.get(6)?,
        canonical_json: row.get(7)?,
    })
}

const TASK_CONTRACT_COLUMNS: &str = "contract_id, task_ref, contract_epoch, \
     user_intent_record_id, interpretation_id, accepted_by, contract_digest, canonical_json";

fn append_event_in_tx(
    tx: &Transaction<'_>,
    event: &cognitive_kernel::ports::EventDraft,
) -> Result<i64, StorePortError> {
    tx.execute(
        "INSERT INTO events
           (event_id, object_id, domain, object_version, event_type, canonical_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            event.event_id.as_str(),
            event.object_id.as_str(),
            event.domain.as_str(),
            event.object_version.get(),
            event.event_type.as_str(),
            event.canonical_json.as_str(),
        ),
    )
    .map_err(unavailable("append chain event"))?;
    Ok(tx.last_insert_rowid())
}

impl IntentChainStore for SqliteAuthorityStore {
    fn insert_user_intent(
        &self,
        record: &UserIntentRecordRow,
        event: &cognitive_kernel::ports::EventDraft,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin user intent"))?;
        let inserted = tx.execute(
            &format!(
                "INSERT INTO user_intent_records ({USER_INTENT_COLUMNS}) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"
            ),
            (
                record.record_id.as_str(),
                record.conversation_or_scope_ref.as_str(),
                record.actor_chain_digest.as_str(),
                record.raw_expression.as_str(),
                record.recorded_at.as_str(),
                record.intent_authority_ref.as_str(),
                record.intent_digest.as_str(),
                record.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!("user intent record {} already fixed", record.record_id),
                });
            }
            Err(err) => return Err(unavailable("insert user intent")(err)),
        }
        let sequence = append_event_in_tx(&tx, event)?;
        tx.commit().map_err(unavailable("commit user intent"))?;
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn load_user_intent(
        &self,
        record_id: &ObjectId,
    ) -> Result<Option<UserIntentRecordRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {USER_INTENT_COLUMNS} FROM user_intent_records WHERE record_id = ?1"
            ))
            .map_err(unavailable("prepare load_user_intent"))?;
        statement
            .query_row((record_id.as_str(),), row_to_user_intent)
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_user_intent")(other)),
            })
    }

    fn list_user_intents_for_scope(
        &self,
        conversation_or_scope_ref: &str,
    ) -> Result<Vec<UserIntentRecordRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {USER_INTENT_COLUMNS} FROM user_intent_records
                 WHERE conversation_or_scope_ref = ?1 ORDER BY record_seq ASC"
            ))
            .map_err(unavailable("prepare list_user_intents_for_scope"))?;
        let mut rows = statement
            .query((conversation_or_scope_ref,))
            .map_err(unavailable("query list_user_intents_for_scope"))?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read user intent row"))? {
            records.push(row_to_user_intent(row).map_err(|err| corrupt("user intent row", err))?);
        }
        Ok(records)
    }

    fn insert_interpretation(
        &self,
        interpretation: &InterpretationRow,
        event: &cognitive_kernel::ports::EventDraft,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin interpretation"))?;
        let inserted = tx.execute(
            &format!(
                "INSERT INTO intent_interpretations ({INTERPRETATION_COLUMNS}) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7)"
            ),
            (
                interpretation.interpretation_id.as_str(),
                interpretation.user_intent_record_id.as_str(),
                interpretation.recorded_status.as_str(),
                interpretation.material_ambiguity_count,
                interpretation
                    .supersedes_interpretation
                    .as_ref()
                    .map(|id| id.as_str()),
                interpretation.interpretation_digest.as_str(),
                interpretation.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "interpretation {} already persisted",
                        interpretation.interpretation_id
                    ),
                });
            }
            Err(err) => return Err(unavailable("insert interpretation")(err)),
        }
        let sequence = append_event_in_tx(&tx, event)?;
        tx.commit().map_err(unavailable("commit interpretation"))?;
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn load_interpretation(
        &self,
        interpretation_id: &ObjectId,
    ) -> Result<Option<InterpretationRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {INTERPRETATION_COLUMNS} FROM intent_interpretations
                 WHERE interpretation_id = ?1"
            ))
            .map_err(unavailable("prepare load_interpretation"))?;
        statement
            .query_row((interpretation_id.as_str(),), row_to_interpretation)
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_interpretation")(other)),
            })
    }

    fn insert_task_contract(
        &self,
        contract: &TaskContractRow,
        event: &cognitive_kernel::ports::EventDraft,
        expected_current_epoch: i64,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin task contract"))?;
        // Contract-epoch CAS inside the transaction: the current epoch
        // must equal the caller's expectation and the new row must be its
        // immediate successor. Any race rolls the whole unit back.
        let current: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(contract_epoch), 0) FROM task_contracts WHERE task_ref = ?1",
                (contract.task_ref.as_str(),),
                |row| row.get(0),
            )
            .map_err(unavailable("read contract epoch"))?;
        if current != expected_current_epoch {
            return Err(StorePortError::Conflict {
                detail: format!(
                    "contract epoch raced for {}: expected {expected_current_epoch}, \
                     current {current}",
                    contract.task_ref
                ),
            });
        }
        if contract.contract_epoch != expected_current_epoch + 1 {
            return Err(StorePortError::Conflict {
                detail: format!(
                    "contract epoch must advance by exactly one: current \
                     {expected_current_epoch}, proposed {}",
                    contract.contract_epoch
                ),
            });
        }
        let inserted = tx.execute(
            &format!(
                "INSERT INTO task_contracts ({TASK_CONTRACT_COLUMNS}) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"
            ),
            (
                contract.contract_id.as_str(),
                contract.task_ref.as_str(),
                contract.contract_epoch,
                contract.user_intent_record_id.as_str(),
                contract.interpretation_id.as_str(),
                contract.accepted_by.as_str(),
                contract.contract_digest.as_str(),
                contract.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "contract {} or epoch {} of {} already persisted",
                        contract.contract_id, contract.contract_epoch, contract.task_ref
                    ),
                });
            }
            Err(err) => return Err(unavailable("insert task contract")(err)),
        }
        let sequence = append_event_in_tx(&tx, event)?;
        tx.commit().map_err(unavailable("commit task contract"))?;
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn load_task_contract(
        &self,
        task_ref: &str,
        contract_epoch: i64,
    ) -> Result<Option<TaskContractRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {TASK_CONTRACT_COLUMNS} FROM task_contracts
                 WHERE task_ref = ?1 AND contract_epoch = ?2"
            ))
            .map_err(unavailable("prepare load_task_contract"))?;
        statement
            .query_row((task_ref, contract_epoch), row_to_task_contract)
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_task_contract")(other)),
            })
    }

    fn list_intents_for_task(&self, task_ref: &str) -> Result<Vec<IntentRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {INTENT_COLUMNS} FROM intents WHERE task_ref = ?1 ORDER BY intent_id"
            ))
            .map_err(unavailable("prepare list_intents_for_task"))?;
        let mut rows = statement
            .query((task_ref,))
            .map_err(unavailable("query list_intents_for_task"))?;
        let mut intents = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read intent row"))? {
            intents.push(row_to_intent(row).map_err(|err| corrupt("intent row", err))?);
        }
        Ok(intents)
    }
}

impl HarnessStore for SqliteAuthorityStore {
    fn append_progress_fact(&self, fact: &ProgressFactRow) -> Result<(), StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin progress fact"))?;
        // Same sink discipline as checkpoints (F-014 store-transaction
        // class): a stale writer cannot poison the stagnation counters.
        verify_fencing_in_tx(&tx, Some(fact.fencing_epoch))?;
        let inserted = tx.execute(
            "INSERT INTO loop_progress_facts
               (loop_object_id, iteration, status, action_fingerprint, evidence_refs_json,
                recorded_at, fencing_epoch)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                fact.loop_object_id.as_str(),
                fact.iteration,
                fact.status.as_str(),
                fact.action_fingerprint.as_str(),
                fact.evidence_refs_json.as_str(),
                fact.recorded_at.as_str(),
                fact.fencing_epoch,
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "progress fact for loop {} iteration {} already recorded",
                        fact.loop_object_id, fact.iteration
                    ),
                });
            }
            Err(err) => return Err(unavailable("insert progress fact")(err)),
        }
        tx.commit().map_err(unavailable("commit progress fact"))?;
        Ok(())
    }

    fn list_progress_facts(
        &self,
        loop_object_id: &ObjectId,
    ) -> Result<Vec<ProgressFactRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT loop_object_id, iteration, status, action_fingerprint,
                        evidence_refs_json, recorded_at, fencing_epoch
                 FROM loop_progress_facts WHERE loop_object_id = ?1 ORDER BY iteration ASC",
            )
            .map_err(unavailable("prepare list_progress_facts"))?;
        let mut rows = statement
            .query((loop_object_id.as_str(),))
            .map_err(unavailable("query list_progress_facts"))?;
        let mut facts = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read progress fact"))? {
            let loop_id: String = row.get(0).map_err(unavailable("column loop_object_id"))?;
            let recorded_at: String = row.get(5).map_err(unavailable("column recorded_at"))?;
            facts.push(ProgressFactRow {
                loop_object_id: ObjectId::parse(&loop_id)
                    .map_err(|err| corrupt("loop_object_id", err))?,
                iteration: row.get(1).map_err(unavailable("column iteration"))?,
                status: row.get(2).map_err(unavailable("column status"))?,
                action_fingerprint: row
                    .get(3)
                    .map_err(unavailable("column action_fingerprint"))?,
                evidence_refs_json: row
                    .get(4)
                    .map_err(unavailable("column evidence_refs_json"))?,
                recorded_at: WallTimestamp::parse(&recorded_at)
                    .map_err(|err| corrupt("recorded_at", err))?,
                fencing_epoch: row.get(6).map_err(unavailable("column fencing_epoch"))?,
            });
        }
        Ok(facts)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn open_asserts_wal_and_installs_append_only_triggers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("authority.db");
        drop(SqliteAuthorityStore::open(&path).unwrap());
        let conn = Connection::open(&path).unwrap();
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode.to_ascii_lowercase(), "wal");
        let triggers: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger'
                 AND name IN ('events_append_only_update','events_append_only_delete',
                              'records_append_only_update','records_append_only_delete')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(triggers, 4);
    }
}
