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

use crate::scheduler::SCHEDULER_SCHEMA_CURRENT;
use crate::worker_authorization::{
    CONTINUATION_AUTHORITY_CONSUMPTION_SCHEMA_V11, CONTINUATION_AUTHORITY_SCHEMA_V10,
    DAEMON_AUTHORIZATION_SNAPSHOT_SCHEMA_V6, DAEMON_OPERATION_DESCRIPTOR_SCHEMA_V5,
    WORKER_AUTHORIZATION_LEASE_BINDING_SCHEMA_V9, WORKER_AUTHORIZATION_SCHEMA_V4,
    WORKER_ITERATION_AUTHORIZATION_CONSUMPTION_SCHEMA_V8, WORKER_ITERATION_AUTHORIZATION_SCHEMA_V7,
};
use cognitive_contracts::generated::context_request::ContextRequest;
use cognitive_contracts::generated::context_view::ContextView;
use cognitive_contracts::generated::governed_object_header::GovernedObjectHeader;
use cognitive_contracts::generated::object_reference::StrongReferenceKind;
use cognitive_contracts::projection::verify_content_digest;
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, StateName, Version, WallTimestamp,
};
use cognitive_kernel::authz::ObjectGovernance;
use cognitive_kernel::effects::GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN;
use cognitive_kernel::ports::{
    AuthorityStore, BoundContinuationAuthorizationConsumption, BoundWorkerAuthorizationConsumption,
    CandidateAdmissionCommit, CandidateAdmissionReceipt, CheckpointRow, CommitReceipt,
    CommittedEvent, ConsumedWorkerIterationAuthorization, ContextAuthorizationFactStore,
    ContextAuthorizationFactsRow, ContextCandidateMetadata, ContextCandidateQuery,
    ContextRequestRow, ContextRevocationFactRow, ContextStore, ContextViewRow,
    ContinuationAuthorityStore, ContinuationAuthorizationRow, DaemonAuthorizationSnapshotRow,
    DaemonOperationDescriptorRow, FixedPostStateRow, GovernanceObjectStore, HarnessStore,
    IntentChainStore, IntentRow, InterpretationRow, ObjectAdmission, OperationCandidateProposalRow,
    OutboxEntry, ProgressFactRow, ProtocolStore, SchedulerLeaseBinding, StorePortError,
    StoredBudget, StoredObject, TaskBinding, TaskContractRow, TransitionCommit,
    UserIntentRecordRow, VerificationReportRow, VerificationRequestRow, WorkerAuthorizationStore,
    WorkerIterationAuthorizationConsumptionRow, WorkerIterationAuthorizationRow,
    WorkspaceContextSourceRow,
};
use cognitive_kernel::{BudgetState, EffectClass, ExecutorCapabilities, OperationDescriptor};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

type ConsumedWorkerAuthorizationDatabaseRow = (
    String,
    String,
    String,
    i64,
    String,
    i64,
    i64,
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    String,
    String,
    i64,
    String,
    String,
    Option<String>,
    Option<i64>,
    Option<String>,
    Option<i64>,
);

/// Schema of the authority database. Two structural guarantees matter to
/// the contract: the event log and transition records are append-only
/// (triggers), and versions are positive integers (CHECK).
/// Immutable authority schema body for Personal migration plan version 1.
///
/// Shared with `personal_db` so the production open path and the versioned
/// migration plan cannot drift. This is not a machine-contract surface.
pub(crate) const AUTHORITY_SCHEMA_V1: &str = "
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

fn effect_class_name(effect_class: EffectClass) -> &'static str {
    match effect_class {
        EffectClass::Pure => "pure",
        EffectClass::LocalEphemeral => "local_ephemeral",
        EffectClass::GovernedExternal => "governed_external",
        EffectClass::EmergencySafety => "emergency_safety",
    }
}

fn parse_effect_class(value: &str) -> Result<EffectClass, StorePortError> {
    match value {
        "pure" => Ok(EffectClass::Pure),
        "local_ephemeral" => Ok(EffectClass::LocalEphemeral),
        "governed_external" => Ok(EffectClass::GovernedExternal),
        "emergency_safety" => Ok(EffectClass::EmergencySafety),
        _ => Err(StorePortError::Unavailable {
            detail: format!("stored daemon descriptor has unknown effect class {value}"),
        }),
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
        conn.execute_batch(&format!(
            "{AUTHORITY_SCHEMA_V1}\n{SCHEDULER_SCHEMA_CURRENT}\n{WORKER_AUTHORIZATION_SCHEMA_V4}\n{DAEMON_OPERATION_DESCRIPTOR_SCHEMA_V5}\n{DAEMON_AUTHORIZATION_SNAPSHOT_SCHEMA_V6}\n{WORKER_ITERATION_AUTHORIZATION_SCHEMA_V7}\n{WORKER_ITERATION_AUTHORIZATION_CONSUMPTION_SCHEMA_V8}\n{WORKER_AUTHORIZATION_LEASE_BINDING_SCHEMA_V9}\n{CONTINUATION_AUTHORITY_SCHEMA_V10}\n{CONTINUATION_AUTHORITY_CONSUMPTION_SCHEMA_V11}"
        ))
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

/// Apply the governed transition portion of an authority commit inside an
/// already-open SQLite transaction. Callers retain ownership of the
/// transaction so compound authority boundaries can commit or roll back all
/// of their evidence together.
fn commit_transition_in_transaction(
    transaction: &Transaction<'_>,
    commit: &TransitionCommit,
) -> Result<CommitReceipt, StorePortError> {
    let cas = &commit.cas;
    let changed = transaction
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

    if let Some(budget) = &commit.budget {
        let changed = transaction
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
    transaction
        .execute(
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
    let event_sequence = transaction.last_insert_rowid();

    let record = &commit.record;
    transaction
        .execute(
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
        transaction
            .execute(
                "INSERT INTO outbox (event_id, destination) VALUES (?1, ?2)",
                (outbox.event_id.as_str(), outbox.destination.as_str()),
            )
            .map_err(unavailable("insert outbox row"))?;
    }

    Ok(CommitReceipt { event_sequence })
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
        let receipt = commit_transition_in_transaction(&tx, commit)?;
        tx.commit().map_err(unavailable("commit transition"))?;
        Ok(receipt)
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
        if let Some(task_binding) = intent.task_binding.as_ref() {
            let eligible_at = scheduler_eligible_at(event)?;
            tx.execute(
                "INSERT INTO scheduler_entries \
                 (task_ref, contract_epoch, state, lease_owner, lease_epoch, lease_expires, next_eligible, attempt_count, cancel_requested) \
                 VALUES (?1, ?2, 'runnable', NULL, 0, NULL, ?3, 0, 0) \
                 ON CONFLICT(task_ref, contract_epoch) DO NOTHING",
                (
                    task_binding.task_ref.as_str(),
                    task_binding.contract_epoch,
                    eligible_at.as_str(),
                ),
            )
            .map_err(unavailable("register scheduler work"))?;
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

    fn list_intents_for_task_binding(
        &self,
        task_binding: &TaskBinding,
    ) -> Result<Vec<IntentRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {INTENT_COLUMNS} FROM intents
                 WHERE task_ref = ?1 AND contract_epoch = ?2
                 ORDER BY intent_id"
            ))
            .map_err(unavailable("prepare list_intents_for_task_binding"))?;
        let rows = statement
            .query_map(
                (task_binding.task_ref.as_str(), task_binding.contract_epoch),
                row_to_intent,
            )
            .map_err(unavailable("query list_intents_for_task_binding"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read list_intents_for_task_binding"))
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

/// Derive scheduler eligibility from the immutable Intent event that is being
/// committed in the same transaction. A binding with no canonical event time
/// must fail closed instead of creating a work row with an invented clock.
fn scheduler_eligible_at(
    event: &cognitive_kernel::ports::EventDraft,
) -> Result<WallTimestamp, StorePortError> {
    let event_value: serde_json::Value =
        serde_json::from_str(&event.canonical_json).map_err(|error| {
            StorePortError::Unavailable {
                detail: format!("scheduler registration event is not canonical JSON: {error}"),
            }
        })?;
    let event_time = event_value
        .get("event_time")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| StorePortError::Unavailable {
            detail: "scheduler registration event has no event_time".to_owned(),
        })?;
    WallTimestamp::parse(event_time).map_err(|error| StorePortError::Unavailable {
        detail: format!("scheduler registration event_time is invalid: {error}"),
    })
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

fn invalid_context_payload(kind: &str, detail: impl std::fmt::Display) -> StorePortError {
    StorePortError::Unavailable {
        detail: format!("invalid {kind} append payload: {detail}"),
    }
}

fn parse_and_verify_context_payload(
    canonical_json: &str,
    kind: &str,
) -> Result<Value, StorePortError> {
    let payload: Value = serde_json::from_str(canonical_json)
        .map_err(|error| invalid_context_payload(kind, error))?;
    verify_content_digest(
        &payload,
        &["/header/content_digest"],
        GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN,
        "/header/content_digest",
    )
    .map_err(|error| invalid_context_payload(kind, error))?;
    Ok(payload)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ContextAuthorizationFactsPayload {
    fact_set_id: String,
    subject_ref: String,
    tenant_id: String,
    principal: cognitive_kernel::authz::PrincipalFacts,
    actor_chain: cognitive_kernel::authz::ActorChainFacts,
    membership: Option<cognitive_kernel::authz::MembershipFacts>,
    capability_links: Vec<cognitive_domain::capability::CapabilityConstraints>,
    explicit_denies: Vec<cognitive_kernel::authz::DenyRule>,
    capability_set_version: i64,
    issued_revocation_epoch: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ContextRevocationFactPayload {
    revocation_fact_id: String,
    tenant_id: String,
    revocation_epoch: i64,
    revoked_subject_ref: Option<String>,
    revoked_capability_ref: Option<String>,
}

fn parse_context_authorization_facts(
    canonical_json: &str,
) -> Result<ContextAuthorizationFactsPayload, StorePortError> {
    serde_json::from_str(canonical_json)
        .map_err(|error| invalid_context_payload("ContextAuthorizationFacts", error))
}

fn validate_context_authorization_facts_row(
    facts: &ContextAuthorizationFactsRow,
) -> Result<(), StorePortError> {
    let payload = parse_context_authorization_facts(&facts.canonical_json)?;
    if payload.fact_set_id != facts.fact_set_id.as_str()
        || payload.subject_ref != facts.subject_ref
        || payload.tenant_id != facts.tenant_id
        || payload.principal != facts.principal
        || payload.actor_chain != facts.actor_chain
        || payload.membership != facts.membership
        || payload.capability_links != facts.capability_links
        || payload.explicit_denies != facts.explicit_denies
        || payload.capability_set_version != facts.capability_set_version
        || payload.issued_revocation_epoch != facts.issued_revocation_epoch
    {
        return Err(invalid_context_payload(
            "ContextAuthorizationFacts",
            "row metadata differs from canonical authorization facts",
        ));
    }
    facts.reconstruct_snapshot(
        facts.issued_revocation_epoch,
        WallTimestamp::parse("2026-01-01T00:00:00Z")
            .map_err(|error| invalid_context_payload("ContextAuthorizationFacts", error))?,
    )?;
    Ok(())
}

fn validate_context_revocation_fact_row(
    fact: &ContextRevocationFactRow,
) -> Result<(), StorePortError> {
    let payload: ContextRevocationFactPayload = serde_json::from_str(&fact.canonical_json)
        .map_err(|error| invalid_context_payload("ContextRevocationFact", error))?;
    if payload.revocation_fact_id != fact.revocation_fact_id.as_str()
        || payload.tenant_id != fact.tenant_id
        || payload.revocation_epoch != fact.revocation_epoch
        || payload.revoked_subject_ref != fact.revoked_subject_ref
        || payload.revoked_capability_ref != fact.revoked_capability_ref
        || fact.revocation_epoch < 1
    {
        return Err(invalid_context_payload(
            "ContextRevocationFact",
            "row metadata differs from canonical revocation fact",
        ));
    }
    Ok(())
}

fn validate_context_request_row(request: &ContextRequestRow) -> Result<(), StorePortError> {
    let payload = parse_and_verify_context_payload(&request.canonical_json, "ContextRequest")?;
    let context_request: ContextRequest = serde_json::from_value(payload)
        .map_err(|error| invalid_context_payload("ContextRequest", error))?;
    let header = &context_request.header;
    if header.id.0 != request.request_id.as_str()
        || header.r#type != "ContextRequest"
        || header.content_digest.0 != request.request_digest
        || context_request.perspective.task != request.task_ref
    {
        return Err(invalid_context_payload(
            "ContextRequest",
            "row identity, type, digest, or task reference differs from canonical payload",
        ));
    }
    Ok(())
}

fn validate_context_view_row(
    connection: &Connection,
    view: &ContextViewRow,
) -> Result<(), StorePortError> {
    let payload = parse_and_verify_context_payload(&view.canonical_json, "ContextView")?;
    let context_view: ContextView = serde_json::from_value(payload)
        .map_err(|error| invalid_context_payload("ContextView", error))?;
    let header = &context_view.header;
    let request_reference = &context_view.request_ref;
    if header.id.0 != view.view_id.as_str()
        || header.r#type != "ContextView"
        || header.content_digest.0 != view.view_digest
        || request_reference.id.0 != view.request_id.as_str()
        || request_reference.kind != StrongReferenceKind::Strong
        || request_reference.object_version != 1
    {
        return Err(invalid_context_payload(
            "ContextView",
            "row identity, type, digest, or request strong reference differs from canonical payload",
        ));
    }
    let persisted_request_digest = connection
        .query_row(
            "SELECT request_digest FROM context_requests WHERE request_id=?1",
            (view.request_id.as_str(),),
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(unavailable("load ContextRequest for ContextView binding"))?;
    let persisted_request_digest =
        persisted_request_digest.ok_or_else(|| StorePortError::Conflict {
            detail: format!("ContextView {} names an unknown request", view.view_id),
        })?;
    if request_reference.content_digest.0 != persisted_request_digest {
        return Err(invalid_context_payload(
            "ContextView",
            "request strong-reference digest differs from the persisted ContextRequest",
        ));
    }
    Ok(())
}

fn validate_workspace_context_source_row(
    source: &WorkspaceContextSourceRow,
) -> Result<(), StorePortError> {
    let payload =
        parse_and_verify_context_payload(&source.canonical_json, "WorkspaceContextSource")?;
    let header: GovernedObjectHeader =
        serde_json::from_value(payload.get("header").cloned().ok_or_else(|| {
            invalid_context_payload("WorkspaceContextSource", "missing governed header")
        })?)
        .map_err(|error| invalid_context_payload("WorkspaceContextSource", error))?;
    if header.id.0 != source.source_id.as_str()
        || header.r#type != "WorkspaceContextSource"
        || header.content_digest.0 != source.source_digest
    {
        return Err(invalid_context_payload(
            "WorkspaceContextSource",
            "row identity, type, or digest differs from canonical payload",
        ));
    }
    let expected_metadata = [
        ("tenant_id", serde_json::json!(source.governance.tenant_id)),
        ("owner_ref", serde_json::json!(source.governance.owner_ref)),
        (
            "resource_scope",
            serde_json::json!(source.governance.resource_scope),
        ),
        (
            "conversation_ref",
            serde_json::json!(source.governance.conversation_ref),
        ),
        ("role", serde_json::json!(source.role)),
        ("trust_level", serde_json::json!(source.trust_level)),
        ("representation", serde_json::json!(source.representation)),
        ("provenance_ref", serde_json::json!(source.provenance_ref)),
        ("content_bytes", serde_json::json!(source.content_bytes)),
        ("content_tokens", serde_json::json!(source.content_tokens)),
    ];
    for (field, expected_value) in expected_metadata {
        if payload.get(field) != Some(&expected_value) {
            return Err(invalid_context_payload(
                "WorkspaceContextSource",
                format!("row {field} differs from canonical payload"),
            ));
        }
    }
    if source.governance.tenant_id.is_none()
        || source.governance.object_ref != source.source_id.as_str()
    {
        return Err(invalid_context_payload(
            "WorkspaceContextSource",
            "workspace source requires tenant governance and matching object reference",
        ));
    }
    Ok(())
}

struct WorkspaceContextSourceDatabaseRow {
    source_id: String,
    source_digest: String,
    tenant_id: String,
    owner_ref: String,
    resource_scope: String,
    conversation_ref: Option<String>,
    role: String,
    trust_level: String,
    representation: String,
    provenance_ref: String,
    content_bytes: i64,
    content_tokens: Option<i64>,
    canonical_json: String,
}

fn parse_workspace_context_source_row(
    database_row: WorkspaceContextSourceDatabaseRow,
) -> Result<WorkspaceContextSourceRow, rusqlite::Error> {
    let parse_enum_error = |error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
    };
    Ok(WorkspaceContextSourceRow {
        source_id: ObjectId::parse(&database_row.source_id).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        source_digest: database_row.source_digest,
        governance: ObjectGovernance {
            object_ref: database_row.source_id,
            tenant_id: Some(database_row.tenant_id),
            owner_ref: database_row.owner_ref,
            resource_scope: database_row.resource_scope,
            conversation_ref: database_row.conversation_ref,
        },
        role: serde_json::from_value(serde_json::Value::String(database_row.role))
            .map_err(parse_enum_error)?,
        trust_level: serde_json::from_value(serde_json::Value::String(database_row.trust_level))
            .map_err(parse_enum_error)?,
        representation: serde_json::from_value(serde_json::Value::String(
            database_row.representation,
        ))
        .map_err(parse_enum_error)?,
        provenance_ref: database_row.provenance_ref,
        content_bytes: database_row.content_bytes,
        content_tokens: database_row.content_tokens,
        canonical_json: database_row.canonical_json,
    })
}

impl ContextStore for SqliteAuthorityStore {
    fn append_context_request(&self, request: &ContextRequestRow) -> Result<(), StorePortError> {
        validate_context_request_row(request)?;
        let connection = self.lock()?;
        let result = connection.execute(
            "INSERT INTO context_requests (request_id, task_ref, request_digest, canonical_json) \
             VALUES (?1, ?2, ?3, ?4)",
            (
                request.request_id.as_str(),
                request.task_ref.as_str(),
                request.request_digest.as_str(),
                request.canonical_json.as_str(),
            ),
        );
        match result {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!("ContextRequest {} already persisted", request.request_id),
            }),
            Err(error) => Err(unavailable("insert ContextRequest")(error)),
        }
    }

    fn load_context_request(
        &self,
        request_id: &ObjectId,
    ) -> Result<Option<ContextRequestRow>, StorePortError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT task_ref, request_digest, canonical_json FROM context_requests WHERE request_id=?1",
                (request_id.as_str(),),
                |row| {
                    Ok(ContextRequestRow {
                        request_id: request_id.clone(),
                        task_ref: row.get(0)?,
                        request_digest: row.get(1)?,
                        canonical_json: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(unavailable("load ContextRequest"))
    }

    fn append_context_view(&self, view: &ContextViewRow) -> Result<(), StorePortError> {
        let connection = self.lock()?;
        validate_context_view_row(&connection, view)?;
        let result = connection.execute(
            "INSERT INTO context_views (view_id, request_id, view_digest, canonical_json) \
             VALUES (?1, ?2, ?3, ?4)",
            (
                view.view_id.as_str(),
                view.request_id.as_str(),
                view.view_digest.as_str(),
                view.canonical_json.as_str(),
            ),
        );
        match result {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!(
                    "ContextView {} is duplicate or names an unknown request",
                    view.view_id
                ),
            }),
            Err(error) => Err(unavailable("insert ContextView")(error)),
        }
    }

    fn load_context_view(
        &self,
        view_id: &ObjectId,
    ) -> Result<Option<ContextViewRow>, StorePortError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT request_id, view_digest, canonical_json FROM context_views WHERE view_id=?1",
                (view_id.as_str(),),
                |row| {
                    let request_id: String = row.get(0)?;
                    Ok(ContextViewRow {
                        view_id: view_id.clone(),
                        request_id: ObjectId::parse(&request_id).map_err(|error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        })?,
                        view_digest: row.get(1)?,
                        canonical_json: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(unavailable("load ContextView"))
    }

    fn append_workspace_context_source(
        &self,
        source: &WorkspaceContextSourceRow,
    ) -> Result<(), StorePortError> {
        validate_workspace_context_source_row(source)?;
        let connection = self.lock()?;
        let role = match source.role {
            cognitive_contracts::generated::context_view::LoadedContextItemRole::Control => "control",
            cognitive_contracts::generated::context_view::LoadedContextItemRole::AuthoritativeState => "authoritative_state",
            cognitive_contracts::generated::context_view::LoadedContextItemRole::Evidence => "evidence",
            cognitive_contracts::generated::context_view::LoadedContextItemRole::Working => "working",
            cognitive_contracts::generated::context_view::LoadedContextItemRole::UntrustedInput => "untrusted_input",
        };
        let trust_level = match source.trust_level {
            cognitive_contracts::generated::context_view::LoadedContextItemTrustLevel::Control => "control",
            cognitive_contracts::generated::context_view::LoadedContextItemTrustLevel::Authoritative => "authoritative",
            cognitive_contracts::generated::context_view::LoadedContextItemTrustLevel::Verified => "verified",
            cognitive_contracts::generated::context_view::LoadedContextItemTrustLevel::Untrusted => "untrusted",
        };
        let representation = match source.representation {
            cognitive_contracts::generated::context_view::LoadedContextItemRepresentation::Structured => "structured",
            cognitive_contracts::generated::context_view::LoadedContextItemRepresentation::Text => "text",
            cognitive_contracts::generated::context_view::LoadedContextItemRepresentation::BinaryRef => "binary_ref",
        };
        let result = connection.execute(
            "INSERT INTO workspace_context_sources (source_id, source_digest, tenant_id, owner_ref, resource_scope, conversation_ref, role, trust_level, representation, provenance_ref, content_bytes, content_tokens, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            (
                source.source_id.as_str(),
                source.source_digest.as_str(),
                source.governance.tenant_id.as_deref(),
                source.governance.owner_ref.as_str(),
                source.governance.resource_scope.as_str(),
                source.governance.conversation_ref.as_deref(),
                role,
                trust_level,
                representation,
                source.provenance_ref.as_str(),
                source.content_bytes,
                source.content_tokens,
                source.canonical_json.as_str(),
            ),
        );
        match result {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!(
                    "WorkspaceContextSource {} already persisted or violates metadata invariants",
                    source.source_id
                ),
            }),
            Err(error) => Err(unavailable("insert WorkspaceContextSource")(error)),
        }
    }

    fn query_context_candidate_metadata(
        &self,
        query: &ContextCandidateQuery,
    ) -> Result<Vec<ContextCandidateMetadata>, StorePortError> {
        let connection = self.lock()?;
        let escaped_prefix = query
            .resource_scope_prefix
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let mut statement = connection.prepare_cached("SELECT source_id, source_digest, tenant_id, owner_ref, resource_scope, conversation_ref, role, trust_level, representation, provenance_ref, content_bytes, content_tokens FROM workspace_context_sources WHERE tenant_id=?1 AND resource_scope LIKE ?2 ESCAPE '\\' AND ((?3 IS NULL AND conversation_ref IS NULL) OR conversation_ref=?3) ORDER BY source_id LIMIT ?4").map_err(unavailable("prepare Context metadata query"))?;
        let rows = statement
            .query_map(
                (
                    query.tenant_id.as_str(),
                    format!("{escaped_prefix}%"),
                    query.conversation_ref.as_deref(),
                    query.limit as i64,
                ),
                |row| {
                    let source =
                        parse_workspace_context_source_row(WorkspaceContextSourceDatabaseRow {
                            source_id: row.get(0)?,
                            source_digest: row.get(1)?,
                            tenant_id: row.get(2)?,
                            owner_ref: row.get(3)?,
                            resource_scope: row.get(4)?,
                            conversation_ref: row.get(5)?,
                            role: row.get(6)?,
                            trust_level: row.get(7)?,
                            representation: row.get(8)?,
                            provenance_ref: row.get(9)?,
                            content_bytes: row.get(10)?,
                            content_tokens: row.get(11)?,
                            canonical_json: String::new(),
                        })?;
                    Ok(ContextCandidateMetadata {
                        source_id: source.source_id,
                        source_digest: source.source_digest,
                        governance: source.governance,
                        role: source.role,
                        trust_level: source.trust_level,
                        representation: source.representation,
                        provenance_ref: source.provenance_ref,
                        content_bytes: source.content_bytes,
                        content_tokens: source.content_tokens,
                    })
                },
            )
            .map_err(unavailable("query Context metadata"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read Context metadata"))
    }

    fn load_workspace_context_source_body(
        &self,
        source_id: &ObjectId,
    ) -> Result<Option<WorkspaceContextSourceRow>, StorePortError> {
        let connection = self.lock()?;
        connection.query_row("SELECT source_id, source_digest, tenant_id, owner_ref, resource_scope, conversation_ref, role, trust_level, representation, provenance_ref, content_bytes, content_tokens, canonical_json FROM workspace_context_sources WHERE source_id=?1", [source_id.as_str()], |row| parse_workspace_context_source_row(WorkspaceContextSourceDatabaseRow {
            source_id: row.get(0)?,
            source_digest: row.get(1)?,
            tenant_id: row.get(2)?,
            owner_ref: row.get(3)?,
            resource_scope: row.get(4)?,
            conversation_ref: row.get(5)?,
            role: row.get(6)?,
            trust_level: row.get(7)?,
            representation: row.get(8)?,
            provenance_ref: row.get(9)?,
            content_bytes: row.get(10)?,
            content_tokens: row.get(11)?,
            canonical_json: row.get(12)?,
        })).optional().map_err(unavailable("load WorkspaceContextSource body"))
    }
}

impl ContextAuthorizationFactStore for SqliteAuthorityStore {
    fn append_context_authorization_facts(
        &self,
        facts: &ContextAuthorizationFactsRow,
    ) -> Result<(), StorePortError> {
        validate_context_authorization_facts_row(facts)?;
        let connection = self.lock()?;
        let result = connection.execute(
            "INSERT INTO context_authorization_fact_sets (fact_set_id, subject_ref, tenant_id, capability_set_version, issued_revocation_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                facts.fact_set_id.as_str(),
                facts.subject_ref.as_str(),
                facts.tenant_id.as_str(),
                facts.capability_set_version,
                facts.issued_revocation_epoch,
                facts.canonical_json.as_str(),
            ),
        );
        match result {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!(
                    "Context authorization facts {} already persisted",
                    facts.fact_set_id
                ),
            }),
            Err(error) => Err(unavailable("insert Context authorization facts")(error)),
        }
    }

    fn append_context_revocation_fact(
        &self,
        fact: &ContextRevocationFactRow,
    ) -> Result<(), StorePortError> {
        validate_context_revocation_fact_row(fact)?;
        let connection = self.lock()?;
        let result = connection.execute(
            "INSERT INTO context_revocation_facts (revocation_fact_id, tenant_id, revocation_epoch, revoked_subject_ref, revoked_capability_ref, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                fact.revocation_fact_id.as_str(),
                fact.tenant_id.as_str(),
                fact.revocation_epoch,
                fact.revoked_subject_ref.as_deref(),
                fact.revoked_capability_ref.as_deref(),
                fact.canonical_json.as_str(),
            ),
        );
        match result {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!(
                    "Context revocation fact {} is duplicate or conflicts with tenant epoch",
                    fact.revocation_fact_id
                ),
            }),
            Err(error) => Err(unavailable("insert Context revocation fact")(error)),
        }
    }

    fn load_latest_context_authorization_facts(
        &self,
        subject_ref: &str,
        tenant_id: &str,
    ) -> Result<Option<ContextAuthorizationFactsRow>, StorePortError> {
        let connection = self.lock()?;
        let canonical_json = connection.query_row(
            "SELECT canonical_json FROM context_authorization_fact_sets WHERE subject_ref=?1 AND tenant_id=?2 ORDER BY fact_sequence DESC LIMIT 1",
            (subject_ref, tenant_id),
            |row| row.get::<_, String>(0),
        ).optional().map_err(unavailable("load latest Context authorization facts"))?;
        canonical_json
            .map(|canonical_json| {
                let payload = parse_context_authorization_facts(&canonical_json)?;
                let fact_set_id = ObjectId::parse(&payload.fact_set_id)
                    .map_err(|error| invalid_context_payload("ContextAuthorizationFacts", error))?;
                let row = ContextAuthorizationFactsRow {
                    fact_set_id,
                    subject_ref: payload.subject_ref,
                    tenant_id: payload.tenant_id,
                    principal: payload.principal,
                    actor_chain: payload.actor_chain,
                    membership: payload.membership,
                    capability_links: payload.capability_links,
                    explicit_denies: payload.explicit_denies,
                    capability_set_version: payload.capability_set_version,
                    issued_revocation_epoch: payload.issued_revocation_epoch,
                    canonical_json,
                };
                validate_context_authorization_facts_row(&row)?;
                Ok(row)
            })
            .transpose()
    }

    fn load_current_context_revocation_epoch(
        &self,
        tenant_id: &str,
    ) -> Result<Option<i64>, StorePortError> {
        let connection = self.lock()?;
        connection.query_row(
            "SELECT revocation_epoch FROM context_revocation_facts WHERE tenant_id=?1 ORDER BY revocation_epoch DESC LIMIT 1",
            [tenant_id],
            |row| row.get(0),
        ).optional().map_err(unavailable("load current Context revocation epoch"))
    }
}

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

impl GovernanceObjectStore for SqliteAuthorityStore {
    fn load_governed_object_header(
        &self,
        object_id: &ObjectId,
    ) -> Result<Option<GovernedObjectHeader>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT canonical_json FROM user_intent_records WHERE record_id = ?1 \
                 UNION ALL SELECT canonical_json FROM intent_interpretations WHERE interpretation_id = ?1 \
                 UNION ALL SELECT canonical_json FROM task_contracts WHERE contract_id = ?1",
            )
            .map_err(unavailable("prepare governed-header lookup"))?;
        let rows = statement
            .query_map([object_id.as_str()], |row| row.get::<_, String>(0))
            .map_err(unavailable("query governed-header lookup"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read governed-header lookup"))?;
        let [canonical_json] = rows.as_slice() else {
            return if rows.is_empty() {
                Ok(None)
            } else {
                Err(StorePortError::Unavailable {
                    detail: "ambiguous governed object identity".to_owned(),
                })
            };
        };
        let value: serde_json::Value = serde_json::from_str(canonical_json)
            .map_err(|err| corrupt("governed canonical json", err))?;
        let header: GovernedObjectHeader =
            serde_json::from_value(value.get("header").cloned().ok_or_else(|| {
                StorePortError::Unavailable {
                    detail: "governed object has no header".to_owned(),
                }
            })?)
            .map_err(|err| corrupt("governed header", err))?;
        if header.id.0 != object_id.as_str() {
            return Err(StorePortError::Unavailable {
                detail: "governed header identity mismatch".to_owned(),
            });
        }
        Ok(Some(header))
    }
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

impl WorkerAuthorizationStore for SqliteAuthorityStore {
    fn load_worker_iteration_authorization(
        &self,
        authorization_id: &ObjectId,
    ) -> Result<Option<WorkerIterationAuthorizationRow>, StorePortError> {
        let conn = self.lock()?;
        let row = conn.query_row(
            "SELECT authorization_id, worker_authorization_root_id, task_ref, contract_epoch,
                    loop_object_id, iteration, expected_loop_version, selected_candidate_id,
                    intent_id, effect_object_id, budget_id, budget_charge_json,
                    action_fingerprint, issued_fencing_epoch, canonical_json
             FROM worker_iteration_authorizations WHERE authorization_id=?1",
            (authorization_id.as_str(),),
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, String>(12)?,
                    row.get::<_, i64>(13)?,
                    row.get::<_, String>(14)?,
                ))
            },
        );
        let row = match row {
            Ok(row) => row,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(error) => return Err(unavailable("query worker authorization")(error)),
        };
        Ok(Some(WorkerIterationAuthorizationRow {
            authorization_id: ObjectId::parse(&row.0)
                .map_err(|error| corrupt("worker authorization id", error))?,
            worker_authorization_root_id: ObjectId::parse(&row.1)
                .map_err(|error| corrupt("worker authorization root", error))?,
            task_ref: row.2,
            contract_epoch: row.3,
            loop_object_id: ObjectId::parse(&row.4)
                .map_err(|error| corrupt("worker authorization loop", error))?,
            iteration: row.5,
            expected_loop_version: Version::new(row.6)
                .map_err(|error| corrupt("worker authorization loop version", error))?,
            selected_candidate_id: ObjectId::parse(&row.7)
                .map_err(|error| corrupt("worker authorization candidate", error))?,
            intent_id: ObjectId::parse(&row.8)
                .map_err(|error| corrupt("worker authorization intent", error))?,
            effect_object_id: ObjectId::parse(&row.9)
                .map_err(|error| corrupt("worker authorization effect", error))?,
            budget_id: BudgetId::parse(&row.10)
                .map_err(|error| corrupt("worker authorization budget", error))?,
            budget_charge_canonical_json: row.11,
            action_fingerprint: row.12,
            issued_fencing_epoch: row.13,
            canonical_json: row.14,
        }))
    }

    fn load_unconsumed_worker_iteration_authorization_for_task_binding(
        &self,
        task_ref: &str,
        contract_epoch: i64,
    ) -> Result<Option<WorkerIterationAuthorizationRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT authorization.authorization_id
                 FROM worker_iteration_authorizations AS authorization
                 LEFT JOIN worker_iteration_authorization_consumptions AS consumption
                   ON consumption.authorization_id = authorization.authorization_id
                 WHERE authorization.task_ref = ?1
                   AND authorization.contract_epoch = ?2
                   AND consumption.authorization_id IS NULL
                 ORDER BY authorization.iteration
                 LIMIT 2",
            )
            .map_err(unavailable("prepare unconsumed worker authorization query"))?;
        let authorization_ids = statement
            .query_map((task_ref, contract_epoch), |row| row.get::<_, String>(0))
            .map_err(unavailable("query unconsumed worker authorizations"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read unconsumed worker authorizations"))?;
        drop(statement);
        drop(conn);

        let [authorization_id] = authorization_ids.as_slice() else {
            return match authorization_ids.len() {
                0 => Ok(None),
                _ => Err(StorePortError::Conflict {
                    detail: "multiple unconsumed worker authorizations match scheduler work"
                        .to_owned(),
                }),
            };
        };
        let authorization_id = ObjectId::parse(authorization_id)
            .map_err(|error| corrupt("unconsumed worker authorization id", error))?;
        self.load_worker_iteration_authorization(&authorization_id)
    }

    fn list_consumed_worker_iteration_authorizations(
        &self,
    ) -> Result<Vec<ConsumedWorkerIterationAuthorization>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT authorization.authorization_id, authorization.worker_authorization_root_id,
                        authorization.task_ref, authorization.contract_epoch,
                        authorization.loop_object_id, authorization.iteration,
                        authorization.expected_loop_version, authorization.selected_candidate_id,
                        authorization.intent_id, authorization.effect_object_id,
                        authorization.budget_id, authorization.budget_charge_json,
                        authorization.action_fingerprint, authorization.issued_fencing_epoch,
                        authorization.canonical_json, consumption.worker_attempt_id,
                        consumption.consumed_fencing_epoch, consumption.consumed_at,
                        consumption.canonical_json, lease_binding.task_ref,
                        lease_binding.contract_epoch, lease_binding.lease_owner,
                        lease_binding.lease_epoch
                 FROM worker_iteration_authorizations AS authorization
                 INNER JOIN worker_iteration_authorization_consumptions AS consumption
                   ON consumption.authorization_id = authorization.authorization_id
                 LEFT JOIN worker_authorization_scheduler_lease_bindings AS lease_binding
                   ON lease_binding.authorization_id = consumption.authorization_id
                 ORDER BY consumption.consumed_at, authorization.authorization_id",
            )
            .map_err(unavailable(
                "prepare consumed worker authorization recovery query",
            ))?;
        let mut rows = statement
            .query(())
            .map_err(unavailable("query consumed worker authorizations"))?;
        let mut recoverable_attempts = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(unavailable("read consumed worker authorization"))?
        {
            let values: ConsumedWorkerAuthorizationDatabaseRow = (|| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                    row.get(12)?,
                    row.get(13)?,
                    row.get(14)?,
                    row.get(15)?,
                    row.get(16)?,
                    row.get(17)?,
                    row.get(18)?,
                    row.get(19)?,
                    row.get(20)?,
                    row.get(21)?,
                    row.get(22)?,
                ))
            })()
            .map_err(unavailable("decode consumed worker authorization"))?;
            let authorization_id = ObjectId::parse(&values.0)
                .map_err(|error| corrupt("worker authorization id", error))?;
            let authorization = WorkerIterationAuthorizationRow {
                authorization_id: authorization_id.clone(),
                worker_authorization_root_id: ObjectId::parse(&values.1)
                    .map_err(|error| corrupt("worker authorization root", error))?,
                task_ref: values.2,
                contract_epoch: values.3,
                loop_object_id: ObjectId::parse(&values.4)
                    .map_err(|error| corrupt("worker authorization loop", error))?,
                iteration: values.5,
                expected_loop_version: Version::new(values.6)
                    .map_err(|error| corrupt("worker authorization loop version", error))?,
                selected_candidate_id: ObjectId::parse(&values.7)
                    .map_err(|error| corrupt("worker authorization candidate", error))?,
                intent_id: ObjectId::parse(&values.8)
                    .map_err(|error| corrupt("worker authorization intent", error))?,
                effect_object_id: ObjectId::parse(&values.9)
                    .map_err(|error| corrupt("worker authorization effect", error))?,
                budget_id: BudgetId::parse(&values.10)
                    .map_err(|error| corrupt("worker authorization budget", error))?,
                budget_charge_canonical_json: values.11,
                action_fingerprint: values.12,
                issued_fencing_epoch: values.13,
                canonical_json: values.14,
            };
            let scheduler_lease = match (values.19, values.20, values.21, values.22) {
                (None, None, None, None) => None,
                (Some(task_ref), Some(contract_epoch), Some(lease_owner), Some(lease_epoch)) => {
                    Some(SchedulerLeaseBinding {
                        task_ref,
                        contract_epoch,
                        lease_owner,
                        lease_epoch,
                    })
                }
                _ => {
                    return Err(StorePortError::Unavailable {
                        detail: "stored worker authorization lease binding is partially populated"
                            .to_owned(),
                    });
                }
            };
            recoverable_attempts.push(ConsumedWorkerIterationAuthorization {
                authorization,
                consumption: WorkerIterationAuthorizationConsumptionRow {
                    authorization_id,
                    worker_attempt_id: ObjectId::parse(&values.15)
                        .map_err(|error| corrupt("worker attempt id", error))?,
                    consumed_fencing_epoch: values.16,
                    consumed_at: WallTimestamp::parse(&values.17)
                        .map_err(|error| corrupt("worker authorization consumption time", error))?,
                    canonical_json: values.18,
                },
                scheduler_lease,
            });
        }
        Ok(recoverable_attempts)
    }

    fn consume_worker_iteration_authorization(
        &self,
        consumption: &WorkerIterationAuthorizationConsumptionRow,
    ) -> Result<(), StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin worker authorization consumption"))?;
        verify_fencing_in_tx(&tx, Some(consumption.consumed_fencing_epoch))?;
        let authorization_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM worker_iteration_authorizations WHERE authorization_id=?1)",
                (consumption.authorization_id.as_str(),),
                |row| row.get(0),
            )
            .map_err(unavailable("verify worker authorization"))?;
        if !authorization_exists {
            return Err(StorePortError::Conflict {
                detail: "worker authorization is not persisted".to_owned(),
            });
        }
        let inserted = tx.execute(
            "INSERT INTO worker_iteration_authorization_consumptions
               (authorization_id, worker_attempt_id, consumed_fencing_epoch, consumed_at, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (consumption.authorization_id.as_str(), consumption.worker_attempt_id.as_str(),
             consumption.consumed_fencing_epoch, consumption.consumed_at.as_str(),
             consumption.canonical_json.as_str()),
        );
        match inserted {
            Ok(_) => tx
                .commit()
                .map_err(unavailable("commit worker authorization consumption")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: "worker authorization was already consumed".to_owned(),
            }),
            Err(error) => Err(unavailable("insert worker authorization consumption")(
                error,
            )),
        }
    }

    fn consume_worker_iteration_authorization_bound_to_scheduler_lease(
        &self,
        request: &BoundWorkerAuthorizationConsumption,
    ) -> Result<(), StorePortError> {
        let consumption = &request.consumption;
        let scheduler_lease = &request.scheduler_lease;
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin bound worker authorization consumption"))?;
        verify_fencing_in_tx(&tx, Some(consumption.consumed_fencing_epoch))?;

        let authorization_matches_lease: bool = tx
            .query_row(
                "SELECT EXISTS(
                   SELECT 1 FROM worker_iteration_authorizations
                   WHERE authorization_id=?1 AND task_ref=?2 AND contract_epoch=?3
                 )",
                (
                    consumption.authorization_id.as_str(),
                    scheduler_lease.task_ref.as_str(),
                    scheduler_lease.contract_epoch,
                ),
                |row| row.get(0),
            )
            .map_err(unavailable("verify bound worker authorization"))?;
        if !authorization_matches_lease {
            return Err(StorePortError::Conflict {
                detail: "worker authorization does not match scheduler work binding".to_owned(),
            });
        }

        let exact_lease_is_active: bool = tx
            .query_row(
                "SELECT EXISTS(
                   SELECT 1 FROM scheduler_entries
                   WHERE task_ref=?1 AND contract_epoch=?2 AND state='leased'
                     AND lease_owner=?3 AND lease_epoch=?4 AND cancel_requested=0
                 )",
                (
                    scheduler_lease.task_ref.as_str(),
                    scheduler_lease.contract_epoch,
                    scheduler_lease.lease_owner.as_str(),
                    scheduler_lease.lease_epoch,
                ),
                |row| row.get(0),
            )
            .map_err(unavailable("verify exact scheduler lease"))?;
        if !exact_lease_is_active {
            return Err(StorePortError::Conflict {
                detail: "scheduler lease is no longer the exact active handoff lease".to_owned(),
            });
        }

        let inserted_consumption = tx.execute(
            "INSERT INTO worker_iteration_authorization_consumptions
               (authorization_id, worker_attempt_id, consumed_fencing_epoch, consumed_at, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                consumption.authorization_id.as_str(),
                consumption.worker_attempt_id.as_str(),
                consumption.consumed_fencing_epoch,
                consumption.consumed_at.as_str(),
                consumption.canonical_json.as_str(),
            ),
        );
        match inserted_consumption {
            Ok(_) => {}
            Err(error) if is_constraint_violation(&error) => {
                return Err(StorePortError::Conflict {
                    detail: "worker authorization was already consumed".to_owned(),
                });
            }
            Err(error) => {
                return Err(unavailable("insert bound worker authorization consumption")(error));
            }
        }

        let inserted_binding = tx.execute(
            "INSERT INTO worker_authorization_scheduler_lease_bindings
               (authorization_id, task_ref, contract_epoch, lease_owner, lease_epoch)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                consumption.authorization_id.as_str(),
                scheduler_lease.task_ref.as_str(),
                scheduler_lease.contract_epoch,
                scheduler_lease.lease_owner.as_str(),
                scheduler_lease.lease_epoch,
            ),
        );
        match inserted_binding {
            Ok(_) => tx
                .commit()
                .map_err(unavailable("commit bound worker authorization consumption")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: "worker authorization scheduler lease binding already exists".to_owned(),
            }),
            Err(error) => Err(unavailable(
                "insert worker authorization scheduler lease binding",
            )(error)),
        }
    }

    fn commit_candidate_admission(
        &self,
        commit: &CandidateAdmissionCommit,
    ) -> Result<CandidateAdmissionReceipt, StorePortError> {
        let intent = &commit.intent;
        let effect_admission = &commit.effect_admission;
        let authorization = &commit.worker_authorization;
        let loop_transition = &commit.loop_transition;

        let consistent_bundle = commit.selected_candidate_id == authorization.selected_candidate_id
            && intent.intent_id == authorization.intent_id
            && intent.effect_object_id == authorization.effect_object_id
            && effect_admission.object.object_id == authorization.effect_object_id
            && effect_admission.object.domain == LifecycleDomain::Effect
            && effect_admission.object.version == Version::INITIAL
            && loop_transition.cas.domain == LifecycleDomain::Loop
            && loop_transition.cas.object_id == authorization.loop_object_id
            && loop_transition.cas.from_state.as_str() == "DECIDE"
            && loop_transition.cas.to_state.as_str() == "ACT"
            && loop_transition.cas.expected_version == authorization.expected_loop_version
            && matches!(
                authorization.expected_loop_version.next(),
                Ok(expected_next_version) if loop_transition.cas.next_version == expected_next_version
            )
            && effect_admission.fencing_epoch == Some(commit.fencing_epoch)
            && loop_transition.fencing_epoch == Some(commit.fencing_epoch)
            && authorization.issued_fencing_epoch == commit.fencing_epoch;
        if !consistent_bundle {
            return Err(StorePortError::Conflict {
                detail: "candidate admission bundle has inconsistent authority bindings".to_owned(),
            });
        }
        let Some(budget) = loop_transition.budget.as_ref() else {
            return Err(StorePortError::Conflict {
                detail: "candidate admission requires an exact budget debit".to_owned(),
            });
        };
        if budget.budget_id != authorization.budget_id {
            return Err(StorePortError::Conflict {
                detail: "candidate admission budget does not match worker authorization".to_owned(),
            });
        }

        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin candidate admission"))?;
        verify_fencing_in_tx(&tx, Some(commit.fencing_epoch))?;

        let candidate_binding = tx.query_row(
            "SELECT task_ref, contract_epoch, parameters_digest, action, target,
                        expected_state_version
                 FROM operation_candidate_proposals WHERE candidate_id=?1",
            (commit.selected_candidate_id.as_str(),),
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        );
        let candidate_binding = match candidate_binding {
            Ok(binding) => binding,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(StorePortError::Conflict {
                    detail: "candidate admission proposal is not persisted".to_owned(),
                });
            }
            Err(error) => return Err(unavailable("load candidate admission proposal")(error)),
        };
        let candidate_matches_authorization = candidate_binding.0 == authorization.task_ref
            && candidate_binding.1 == authorization.contract_epoch
            && candidate_binding.2 == intent.parameters_digest
            && candidate_binding.3 == intent.action
            && candidate_binding.4 == intent.target
            && candidate_binding.5 == intent.expected_state_version.get();
        if !candidate_matches_authorization {
            return Err(StorePortError::Conflict {
                detail: "candidate admission does not match persisted proposal".to_owned(),
            });
        }
        let current_contract_epoch = tx
            .query_row(
                "SELECT COALESCE(MAX(contract_epoch), 0) FROM task_contracts WHERE task_ref=?1",
                (authorization.task_ref.as_str(),),
                |row| row.get::<_, i64>(0),
            )
            .map_err(unavailable("load candidate admission contract epoch"))?;
        if current_contract_epoch != authorization.contract_epoch {
            return Err(StorePortError::Conflict {
                detail: "candidate admission TaskContract epoch was superseded".to_owned(),
            });
        }

        let insert_intent = tx.execute(
            "INSERT INTO intents
               (intent_id, idempotency_key, parameters_digest, action, target, effect_object_id,
                expected_state_version, grant_epoch, capability_set_version, task_ref,
                contract_epoch, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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
        if let Err(error) = insert_intent {
            return if is_constraint_violation(&error) {
                Err(StorePortError::Conflict {
                    detail: "candidate admission intent already exists".to_owned(),
                })
            } else {
                Err(unavailable("insert candidate admission intent")(error))
            };
        }
        let intent_event_sequence = append_event_in_tx(&tx, &commit.intent_event)?;

        let effect_body_json = serde_json::to_string(&effect_admission.object.body)
            .map_err(|error| corrupt("candidate admission effect body", error))?;
        let insert_effect = tx.execute(
            "INSERT INTO governed_objects
               (object_id, domain, state, version, body_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            (
                effect_admission.object.object_id.as_str(),
                effect_admission.object.domain.as_str(),
                effect_admission.object.state.as_str(),
                effect_admission.object.version.get(),
                effect_body_json,
                effect_admission.admitted_at.as_str(),
            ),
        );
        if let Err(error) = insert_effect {
            return if is_constraint_violation(&error) {
                Err(StorePortError::Conflict {
                    detail: "candidate admission effect already exists".to_owned(),
                })
            } else {
                Err(unavailable("insert candidate admission effect")(error))
            };
        }
        let effect_admission_event_sequence = append_event_in_tx(&tx, &effect_admission.event)?;
        for outbox in &effect_admission.outbox {
            tx.execute(
                "INSERT INTO outbox (event_id, destination) VALUES (?1, ?2)",
                (outbox.event_id.as_str(), outbox.destination.as_str()),
            )
            .map_err(unavailable("insert candidate admission effect outbox"))?;
        }

        let insert_authorization = tx.execute(
            "INSERT INTO worker_iteration_authorizations
               (authorization_id, worker_authorization_root_id, task_ref, contract_epoch,
                loop_object_id, iteration, expected_loop_version, selected_candidate_id,
                intent_id, effect_object_id, budget_id, budget_charge_json, action_fingerprint,
                issued_fencing_epoch, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            (
                authorization.authorization_id.as_str(),
                authorization.worker_authorization_root_id.as_str(),
                authorization.task_ref.as_str(),
                authorization.contract_epoch,
                authorization.loop_object_id.as_str(),
                authorization.iteration,
                authorization.expected_loop_version.get(),
                authorization.selected_candidate_id.as_str(),
                authorization.intent_id.as_str(),
                authorization.effect_object_id.as_str(),
                authorization.budget_id.as_str(),
                authorization.budget_charge_canonical_json.as_str(),
                authorization.action_fingerprint.as_str(),
                authorization.issued_fencing_epoch,
                authorization.canonical_json.as_str(),
            ),
        );
        if let Err(error) = insert_authorization {
            return if is_constraint_violation(&error) {
                Err(StorePortError::Conflict {
                    detail: "candidate admission authorization already exists".to_owned(),
                })
            } else {
                Err(unavailable("insert candidate admission authorization")(
                    error,
                ))
            };
        }

        let cas = &loop_transition.cas;
        let changed = tx
            .execute(
                "UPDATE governed_objects SET state=?1, version=?2, updated_at=?3
             WHERE object_id=?4 AND domain=?5 AND state=?6 AND version=?7",
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
            .map_err(unavailable("candidate admission loop cas"))?;
        if changed == 0 {
            return Err(StorePortError::Conflict {
                detail: "candidate admission loop cas raced".to_owned(),
            });
        }
        if let Some(budget) = &loop_transition.budget {
            let changed = tx.execute(
                "UPDATE budgets SET state_json=?1, version=?2 WHERE budget_id=?3 AND version=?4",
                (budget.next_state_canonical_json.as_str(), budget.next_version.get(),
                 budget.budget_id.as_str(), budget.expected_version.get()),
            ).map_err(unavailable("candidate admission budget cas"))?;
            if changed == 0 {
                return Err(StorePortError::Conflict {
                    detail: "candidate admission budget cas raced".to_owned(),
                });
            }
        }
        let loop_transition_event_sequence = append_event_in_tx(&tx, &loop_transition.event)?;
        tx.execute(
            "INSERT INTO transition_records (record_id, object_id, domain, object_version, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (loop_transition.record.record_id.as_str(), loop_transition.record.object_id.as_str(),
             loop_transition.record.domain.as_str(), loop_transition.record.object_version.get(),
             loop_transition.record.canonical_json.as_str()),
        ).map_err(unavailable("append candidate admission loop record"))?;
        for outbox in &loop_transition.outbox {
            tx.execute(
                "INSERT INTO outbox (event_id, destination) VALUES (?1, ?2)",
                (outbox.event_id.as_str(), outbox.destination.as_str()),
            )
            .map_err(unavailable("insert candidate admission loop outbox"))?;
        }
        tx.commit()
            .map_err(unavailable("commit candidate admission"))?;
        Ok(CandidateAdmissionReceipt {
            intent_event_sequence,
            effect_admission_event_sequence,
            loop_transition_event_sequence,
            authorization_id: authorization.authorization_id.clone(),
        })
    }

    fn append_daemon_authorization_snapshot(
        &self,
        snapshot: &DaemonAuthorizationSnapshotRow,
    ) -> Result<(), StorePortError> {
        let conn = self.lock()?;
        let inserted = conn.execute(
            "INSERT INTO daemon_authorization_snapshots
               (snapshot_id, subject_ref, target_ref, action, purpose, grant_epoch,
                capability_set_version, revocation_epoch, observed_at, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (
                snapshot.snapshot_id.as_str(),
                snapshot.subject_ref.as_str(),
                snapshot.target_ref.as_str(),
                snapshot.action.as_str(),
                snapshot.purpose.as_str(),
                snapshot.grant_epoch,
                snapshot.capability_set_version,
                snapshot.revocation_epoch,
                snapshot.observed_at.as_str(),
                snapshot.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!(
                    "daemon authorization snapshot {} already persisted",
                    snapshot.snapshot_id
                ),
            }),
            Err(error) => Err(unavailable("insert daemon authorization snapshot")(error)),
        }
    }

    fn load_latest_daemon_authorization_snapshot(
        &self,
        subject_ref: &str,
        target_ref: &str,
        action: &str,
        purpose: &str,
    ) -> Result<Option<DaemonAuthorizationSnapshotRow>, StorePortError> {
        let conn = self.lock()?;
        let result = conn.query_row(
            "SELECT snapshot_id, subject_ref, target_ref, action, purpose, grant_epoch,
                    capability_set_version, revocation_epoch, observed_at, canonical_json
             FROM daemon_authorization_snapshots
             WHERE subject_ref=?1 AND target_ref=?2 AND action=?3 AND purpose=?4
             ORDER BY snapshot_sequence DESC LIMIT 1",
            (subject_ref, target_ref, action, purpose),
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get::<_, String>(8)?,
                    row.get(9)?,
                ))
            },
        );
        match result {
            Ok((
                snapshot_id,
                subject_ref,
                target_ref,
                action,
                purpose,
                grant_epoch,
                capability_set_version,
                revocation_epoch,
                observed_at,
                canonical_json,
            )) => Ok(Some(DaemonAuthorizationSnapshotRow {
                snapshot_id: ObjectId::parse(&snapshot_id)
                    .map_err(|error| corrupt("daemon authorization snapshot id", error))?,
                subject_ref,
                target_ref,
                action,
                purpose,
                grant_epoch,
                capability_set_version,
                revocation_epoch,
                observed_at: WallTimestamp::parse(&observed_at)
                    .map_err(|error| corrupt("daemon authorization snapshot time", error))?,
                canonical_json,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(unavailable("query daemon authorization snapshot")(error)),
        }
    }

    fn append_daemon_operation_descriptor(
        &self,
        descriptor: &DaemonOperationDescriptorRow,
    ) -> Result<(), StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin daemon descriptor"))?;
        let descriptor_value = &descriptor.descriptor;
        let inserted = tx.execute(
            "INSERT INTO daemon_operation_descriptors
               (descriptor_id, operation_id, action, effect_class, executor, queryable,
                idempotent, descriptor_version, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (
                descriptor.descriptor_id.as_str(),
                descriptor_value.operation_id.as_str(),
                descriptor_value.action.as_str(),
                effect_class_name(descriptor_value.effect_class),
                descriptor_value.executor.as_str(),
                descriptor_value.capabilities.queryable,
                descriptor_value.capabilities.idempotent,
                descriptor_value.descriptor_version,
                descriptor.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(error) if is_constraint_violation(&error) => {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "daemon operation descriptor {} already persisted",
                        descriptor.descriptor_id
                    ),
                });
            }
            Err(error) => return Err(unavailable("insert daemon descriptor")(error)),
        }
        tx.commit().map_err(unavailable("commit daemon descriptor"))
    }

    fn load_daemon_operation_descriptor(
        &self,
        descriptor_id: &ObjectId,
    ) -> Result<Option<DaemonOperationDescriptorRow>, StorePortError> {
        let conn = self.lock()?;
        let result = conn.query_row(
            "SELECT descriptor_id, operation_id, action, effect_class, executor, queryable,
                    idempotent, descriptor_version, canonical_json
             FROM daemon_operation_descriptors WHERE descriptor_id = ?1",
            (descriptor_id.as_str(),),
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, bool>(5)?,
                    row.get::<_, bool>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, String>(8)?,
                ))
            },
        );
        match result {
            Ok((
                stored_id,
                operation_id,
                action,
                stored_effect_class,
                executor,
                queryable,
                idempotent,
                descriptor_version,
                canonical_json,
            )) => Ok(Some(DaemonOperationDescriptorRow {
                descriptor_id: ObjectId::parse(&stored_id)
                    .map_err(|error| corrupt("daemon descriptor id", error))?,
                descriptor: OperationDescriptor {
                    operation_id,
                    action,
                    effect_class: parse_effect_class(&stored_effect_class)?,
                    executor,
                    capabilities: ExecutorCapabilities {
                        queryable,
                        idempotent,
                    },
                    descriptor_version,
                },
                canonical_json,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(unavailable("query daemon descriptor")(error)),
        }
    }

    fn append_operation_candidate_proposal(
        &self,
        proposal: &OperationCandidateProposalRow,
    ) -> Result<(), StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin candidate proposal"))?;
        let inserted = tx.execute(
            "INSERT INTO operation_candidate_proposals
               (candidate_id, task_ref, contract_epoch, candidate_source_ref, tool_ref,
                action, target, parameters_digest, expected_state_version,
                operation_descriptor_ref, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (
                proposal.candidate_id.as_str(),
                proposal.task_ref.as_str(),
                proposal.contract_epoch,
                proposal.candidate_source_ref.as_str(),
                proposal.tool_ref.as_str(),
                proposal.action.as_str(),
                proposal.target.as_str(),
                proposal.parameters_digest.as_str(),
                proposal.expected_state_version,
                proposal.operation_descriptor_ref.as_str(),
                proposal.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "candidate proposal {} already persisted",
                        proposal.candidate_id
                    ),
                });
            }
            Err(err) => return Err(unavailable("insert candidate proposal")(err)),
        }
        tx.commit()
            .map_err(unavailable("commit candidate proposal"))
    }

    fn load_operation_candidate_proposal(
        &self,
        candidate_id: &ObjectId,
    ) -> Result<Option<OperationCandidateProposalRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT candidate_id, task_ref, contract_epoch, candidate_source_ref, tool_ref,
                        action, target, parameters_digest, expected_state_version,
                        operation_descriptor_ref, canonical_json
                 FROM operation_candidate_proposals WHERE candidate_id = ?1",
            )
            .map_err(unavailable("prepare load candidate proposal"))?;
        statement
            .query_row((candidate_id.as_str(),), |row| {
                let candidate_id: String = row.get(0)?;
                let operation_descriptor_ref: String = row.get(9)?;
                Ok(OperationCandidateProposalRow {
                    candidate_id: ObjectId::parse(&candidate_id).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?,
                    task_ref: row.get(1)?,
                    contract_epoch: row.get(2)?,
                    candidate_source_ref: row.get(3)?,
                    tool_ref: row.get(4)?,
                    action: row.get(5)?,
                    target: row.get(6)?,
                    parameters_digest: row.get(7)?,
                    expected_state_version: row.get(8)?,
                    operation_descriptor_ref: ObjectId::parse(&operation_descriptor_ref).map_err(
                        |error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                9,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        },
                    )?,
                    canonical_json: row.get(10)?,
                })
            })
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load candidate proposal")(other)),
            })
    }
}

impl ContinuationAuthorityStore for SqliteAuthorityStore {
    fn append_fixed_post_state(&self, row: &FixedPostStateRow) -> Result<(), StorePortError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin fixed post-state"))?;
        verify_fencing_in_tx(&transaction, Some(row.recorded_fencing_epoch))?;
        transaction.execute(
            "INSERT INTO fixed_post_states (fixed_post_state_id, task_ref, contract_epoch, loop_object_id, subject_domain, subject_object_id, subject_version, recorded_fencing_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (row.fixed_post_state_id.as_str(), row.task_binding.task_ref.as_str(), row.task_binding.contract_epoch, row.loop_object_id.as_str(), row.subject_domain.as_str(), row.subject_object_id.as_str(), row.subject_version.get(), row.recorded_fencing_epoch, row.canonical_json.as_str()),
        ).map_err(unavailable("insert fixed post-state"))?;
        transaction
            .commit()
            .map_err(unavailable("commit fixed post-state"))
    }

    fn load_fixed_post_state(
        &self,
        fixed_post_state_id: &ObjectId,
    ) -> Result<Option<FixedPostStateRow>, StorePortError> {
        let connection = self.lock()?;
        let result = connection.query_row(
            "SELECT task_ref, contract_epoch, loop_object_id, subject_domain, subject_object_id, subject_version, recorded_fencing_epoch, canonical_json FROM fixed_post_states WHERE fixed_post_state_id=?1",
            (fixed_post_state_id.as_str(),),
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, i64>(5)?, row.get::<_, i64>(6)?, row.get::<_, String>(7)?)),
        );
        match result {
            Ok((
                task_ref,
                contract_epoch,
                loop_object_id,
                subject_domain,
                subject_object_id,
                subject_version,
                recorded_fencing_epoch,
                canonical_json,
            )) => Ok(Some(FixedPostStateRow {
                fixed_post_state_id: fixed_post_state_id.clone(),
                task_binding: TaskBinding {
                    task_ref,
                    contract_epoch,
                },
                loop_object_id: ObjectId::parse(&loop_object_id)
                    .map_err(|error| corrupt("fixed post-state loop id", error))?,
                subject_domain: LifecycleDomain::parse(&subject_domain)
                    .map_err(|error| corrupt("fixed post-state subject domain", error))?,
                subject_object_id: ObjectId::parse(&subject_object_id)
                    .map_err(|error| corrupt("fixed post-state subject id", error))?,
                subject_version: Version::new(subject_version)
                    .map_err(|error| corrupt("fixed post-state subject version", error))?,
                recorded_fencing_epoch,
                canonical_json,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(unavailable("query fixed post-state")(error)),
        }
    }

    fn append_verification_request(
        &self,
        row: &VerificationRequestRow,
    ) -> Result<(), StorePortError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin verification request"))?;
        verify_fencing_in_tx(&transaction, Some(row.issued_fencing_epoch))?;
        transaction.execute(
            "INSERT INTO verification_requests (verification_request_id, fixed_post_state_id, task_ref, contract_epoch, loop_object_id, expected_loop_version, verifier_ref, verifier_version, criteria_json, issued_fencing_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (row.verification_request_id.as_str(), row.fixed_post_state_id.as_str(), row.task_binding.task_ref.as_str(), row.task_binding.contract_epoch, row.loop_object_id.as_str(), row.expected_loop_version.get(), row.verifier_ref.as_str(), row.verifier_version.as_str(), row.criteria_canonical_json.as_str(), row.issued_fencing_epoch, row.canonical_json.as_str()),
        ).map_err(unavailable("insert verification request"))?;
        transaction
            .commit()
            .map_err(unavailable("commit verification request"))
    }

    fn load_verification_request(
        &self,
        verification_request_id: &ObjectId,
    ) -> Result<Option<VerificationRequestRow>, StorePortError> {
        let connection = self.lock()?;
        let result = connection.query_row(
            "SELECT fixed_post_state_id, task_ref, contract_epoch, loop_object_id, expected_loop_version, verifier_ref, verifier_version, criteria_json, issued_fencing_epoch, canonical_json FROM verification_requests WHERE verification_request_id=?1",
            (verification_request_id.as_str(),),
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, String>(3)?, row.get::<_, i64>(4)?, row.get::<_, String>(5)?, row.get::<_, String>(6)?, row.get::<_, String>(7)?, row.get::<_, i64>(8)?, row.get::<_, String>(9)?)),
        );
        match result {
            Ok((
                fixed_post_state_id,
                task_ref,
                contract_epoch,
                loop_object_id,
                expected_loop_version,
                verifier_ref,
                verifier_version,
                criteria_canonical_json,
                issued_fencing_epoch,
                canonical_json,
            )) => Ok(Some(VerificationRequestRow {
                verification_request_id: verification_request_id.clone(),
                fixed_post_state_id: ObjectId::parse(&fixed_post_state_id)
                    .map_err(|error| corrupt("verification request fixed post-state id", error))?,
                task_binding: TaskBinding {
                    task_ref,
                    contract_epoch,
                },
                loop_object_id: ObjectId::parse(&loop_object_id)
                    .map_err(|error| corrupt("verification request loop id", error))?,
                expected_loop_version: Version::new(expected_loop_version)
                    .map_err(|error| corrupt("verification request loop version", error))?,
                verifier_ref,
                verifier_version,
                criteria_canonical_json,
                issued_fencing_epoch,
                canonical_json,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(unavailable("query verification request")(error)),
        }
    }

    fn append_verification_report(
        &self,
        row: &VerificationReportRow,
    ) -> Result<(), StorePortError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin verification report"))?;
        verify_fencing_in_tx(&transaction, Some(row.recorded_fencing_epoch))?;
        transaction.execute(
            "INSERT INTO verification_reports (verification_report_id, verification_request_id, fixed_post_state_id, verifier_ref, verifier_version, status, evidence_refs_json, completed_at, recorded_fencing_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (row.verification_report_id.as_str(), row.verification_request_id.as_str(), row.fixed_post_state_id.as_str(), row.verifier_ref.as_str(), row.verifier_version.as_str(), row.status.as_str(), row.evidence_refs_canonical_json.as_str(), row.completed_at.as_str(), row.recorded_fencing_epoch, row.canonical_json.as_str()),
        ).map_err(unavailable("insert verification report"))?;
        transaction
            .commit()
            .map_err(unavailable("commit verification report"))
    }

    fn load_verification_report(
        &self,
        verification_report_id: &ObjectId,
    ) -> Result<Option<VerificationReportRow>, StorePortError> {
        let connection = self.lock()?;
        let result = connection.query_row(
            "SELECT verification_request_id, fixed_post_state_id, verifier_ref, verifier_version, status, evidence_refs_json, completed_at, recorded_fencing_epoch, canonical_json FROM verification_reports WHERE verification_report_id=?1",
            (verification_report_id.as_str(),),
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, String>(5)?, row.get::<_, String>(6)?, row.get::<_, i64>(7)?, row.get::<_, String>(8)?)),
        );
        match result {
            Ok((
                verification_request_id,
                fixed_post_state_id,
                verifier_ref,
                verifier_version,
                status,
                evidence_refs_canonical_json,
                completed_at,
                recorded_fencing_epoch,
                canonical_json,
            )) => Ok(Some(VerificationReportRow {
                verification_report_id: verification_report_id.clone(),
                verification_request_id: ObjectId::parse(&verification_request_id)
                    .map_err(|error| corrupt("verification report request id", error))?,
                fixed_post_state_id: ObjectId::parse(&fixed_post_state_id)
                    .map_err(|error| corrupt("verification report fixed post-state id", error))?,
                verifier_ref,
                verifier_version,
                status,
                evidence_refs_canonical_json,
                completed_at: WallTimestamp::parse(&completed_at)
                    .map_err(|error| corrupt("verification report completion time", error))?,
                recorded_fencing_epoch,
                canonical_json,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(unavailable("query verification report")(error)),
        }
    }

    fn issue_continuation_authorization(
        &self,
        row: &ContinuationAuthorizationRow,
    ) -> Result<(), StorePortError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin continuation authorization"))?;
        verify_fencing_in_tx(&transaction, Some(row.issued_fencing_epoch))?;
        let current_contract_epoch: i64 = transaction
            .query_row(
                "SELECT COALESCE(MAX(contract_epoch), 0) FROM task_contracts WHERE task_ref=?1",
                (row.task_binding.task_ref.as_str(),),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("read continuation contract epoch"))?;
        if current_contract_epoch != row.task_binding.contract_epoch {
            return Err(StorePortError::Conflict {
                detail: "continuation authorization contract epoch is stale".to_owned(),
            });
        }
        let loop_state: Option<(String, i64)> = transaction
            .query_row(
                "SELECT state, version FROM governed_objects WHERE object_id=?1 AND domain='loop'",
                (row.loop_object_id.as_str(),),
                |database_row| Ok((database_row.get(0)?, database_row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("read continuation loop"))?;
        if loop_state != Some(("CONTINUE".to_owned(), row.expected_loop_version.get())) {
            return Err(StorePortError::Conflict {
                detail: "continuation authorization loop is not current CONTINUE state".to_owned(),
            });
        }
        let checkpoint_exists: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM checkpoints WHERE checkpoint_id=?1 AND loop_object_id=?2 AND fencing_epoch=?3)",
                (row.checkpoint_id.as_str(), row.loop_object_id.as_str(), row.issued_fencing_epoch),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("read continuation checkpoint"))?;
        if !checkpoint_exists {
            return Err(StorePortError::Conflict {
                detail: "continuation authorization checkpoint is unavailable or fenced".to_owned(),
            });
        }
        let report_exists: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM verification_reports AS report JOIN verification_requests AS request ON request.verification_request_id=report.verification_request_id AND request.fixed_post_state_id=report.fixed_post_state_id JOIN fixed_post_states AS fixed_state ON fixed_state.fixed_post_state_id=report.fixed_post_state_id AND fixed_state.task_ref=request.task_ref AND fixed_state.contract_epoch=request.contract_epoch AND fixed_state.loop_object_id=request.loop_object_id JOIN governed_objects AS subject ON subject.object_id=fixed_state.subject_object_id AND subject.domain=fixed_state.subject_domain WHERE report.verification_report_id=?1 AND report.status='passed' AND report.recorded_fencing_epoch=?2 AND request.issued_fencing_epoch=?2 AND fixed_state.recorded_fencing_epoch=?2 AND request.task_ref=?3 AND request.contract_epoch=?4 AND request.loop_object_id=?5 AND subject.version=fixed_state.subject_version AND NOT (subject.domain='task' AND subject.state='COMPLETED'))",
                (row.verification_report_id.as_str(), row.issued_fencing_epoch, row.task_binding.task_ref.as_str(), row.task_binding.contract_epoch, row.loop_object_id.as_str()),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("read continuation verification report"))?;
        if !report_exists {
            return Err(StorePortError::Conflict {
                detail: "continuation authorization requires a current passed verified post-state"
                    .to_owned(),
            });
        }
        transaction.execute(
            "INSERT INTO continuation_authorizations (continuation_authorization_id, task_ref, contract_epoch, loop_object_id, iteration, expected_loop_version, checkpoint_id, budget_id, budget_charge_json, verification_report_id, issued_fencing_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            (row.continuation_authorization_id.as_str(), row.task_binding.task_ref.as_str(), row.task_binding.contract_epoch, row.loop_object_id.as_str(), row.iteration, row.expected_loop_version.get(), row.checkpoint_id.as_str(), row.budget_id.as_str(), row.budget_charge_canonical_json.as_str(), row.verification_report_id.as_str(), row.issued_fencing_epoch, row.canonical_json.as_str()),
        ).map_err(unavailable("insert continuation authorization"))?;
        transaction
            .commit()
            .map_err(unavailable("commit continuation authorization"))
    }

    fn consume_continuation_authorization_bound_to_scheduler_lease(
        &self,
        request: &BoundContinuationAuthorizationConsumption,
        transition: &TransitionCommit,
    ) -> Result<CommitReceipt, StorePortError> {
        let consumption = &request.consumption;
        let scheduler_lease = &request.scheduler_lease;
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin bound continuation consumption"))?;
        verify_fencing_in_tx(&transaction, Some(consumption.consumed_fencing_epoch))?;

        let authorization_matches_lease: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM continuation_authorizations WHERE continuation_authorization_id=?1 AND task_ref=?2 AND contract_epoch=?3 AND issued_fencing_epoch=?4)",
                (consumption.continuation_authorization_id.as_str(), scheduler_lease.task_ref.as_str(), scheduler_lease.contract_epoch, consumption.consumed_fencing_epoch),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("verify bound continuation authorization"))?;
        if !authorization_matches_lease {
            return Err(StorePortError::Conflict {
                detail: "continuation authorization does not match scheduler work binding"
                    .to_owned(),
            });
        }

        let exact_lease_is_active: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM scheduler_entries WHERE task_ref=?1 AND contract_epoch=?2 AND state='leased' AND lease_owner=?3 AND lease_epoch=?4 AND cancel_requested=0)",
                (scheduler_lease.task_ref.as_str(), scheduler_lease.contract_epoch, scheduler_lease.lease_owner.as_str(), scheduler_lease.lease_epoch),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("verify exact continuation scheduler lease"))?;
        if !exact_lease_is_active {
            return Err(StorePortError::Conflict {
                detail: "scheduler lease is no longer the exact active continuation lease"
                    .to_owned(),
            });
        }

        let transition_matches_authorization: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM continuation_authorizations WHERE continuation_authorization_id=?1 AND loop_object_id=?2 AND expected_loop_version=?3 AND budget_id=?4 AND budget_charge_json=?5)",
                (consumption.continuation_authorization_id.as_str(), transition.cas.object_id.as_str(), transition.cas.expected_version.get(), transition.budget.as_ref().map(|budget| budget.budget_id.as_str()).unwrap_or(""), transition.budget.as_ref().map(|budget| budget.charge_canonical_json.as_str()).unwrap_or("")),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("verify continuation transition binding"))?;
        let is_legal_continuation_entry = transition.cas.domain == LifecycleDomain::Loop
            && transition.cas.from_state.as_str() == "CONTINUE"
            && transition.cas.to_state.as_str() == "OBSERVE"
            && transition.fencing_epoch == Some(consumption.consumed_fencing_epoch)
            && transition.budget.is_some();
        if !transition_matches_authorization || !is_legal_continuation_entry {
            return Err(StorePortError::Conflict {
                detail: "continuation authority does not match the prepared loop entry".to_owned(),
            });
        }

        let inserted_consumption = transaction.execute(
            "INSERT INTO continuation_authorization_consumptions (continuation_authorization_id, consumed_fencing_epoch, consumed_at, canonical_json) VALUES (?1, ?2, ?3, ?4)",
            (consumption.continuation_authorization_id.as_str(), consumption.consumed_fencing_epoch, consumption.consumed_at.as_str(), consumption.canonical_json.as_str()),
        );
        match inserted_consumption {
            Ok(_) => {}
            Err(error) if is_constraint_violation(&error) => {
                return Err(StorePortError::Conflict {
                    detail: "continuation authorization was already consumed".to_owned(),
                });
            }
            Err(error) => {
                return Err(unavailable("insert continuation consumption")(error));
            }
        }

        let inserted_binding = transaction.execute(
            "INSERT INTO continuation_authorization_scheduler_lease_bindings (continuation_authorization_id, task_ref, contract_epoch, lease_owner, lease_epoch) VALUES (?1, ?2, ?3, ?4, ?5)",
            (consumption.continuation_authorization_id.as_str(), scheduler_lease.task_ref.as_str(), scheduler_lease.contract_epoch, scheduler_lease.lease_owner.as_str(), scheduler_lease.lease_epoch),
        );
        match inserted_binding {
            Ok(_) => {}
            Err(error) if is_constraint_violation(&error) => {
                return Err(StorePortError::Conflict {
                    detail: "continuation authorization scheduler lease binding already exists"
                        .to_owned(),
                });
            }
            Err(error) => {
                return Err(unavailable("insert continuation scheduler lease binding")(
                    error,
                ));
            }
        }

        let receipt = commit_transition_in_transaction(&transaction, transition)?;
        transaction
            .commit()
            .map_err(unavailable("commit bound continuation entry"))?;
        Ok(receipt)
    }

    fn load_unconsumed_continuation_authorization(
        &self,
        task_binding: &TaskBinding,
    ) -> Result<Option<ContinuationAuthorizationRow>, StorePortError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare_cached("SELECT continuation_authorization_id, loop_object_id, iteration, expected_loop_version, checkpoint_id, budget_id, budget_charge_json, verification_report_id, issued_fencing_epoch, canonical_json FROM continuation_authorizations WHERE task_ref=?1 AND contract_epoch=?2 AND continuation_authorization_id NOT IN (SELECT continuation_authorization_id FROM continuation_authorization_consumptions) ORDER BY iteration LIMIT 2").map_err(unavailable("prepare continuation authorization query"))?;
        let rows = statement
            .query_map(
                (task_binding.task_ref.as_str(), task_binding.contract_epoch),
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, i64>(8)?,
                        row.get::<_, String>(9)?,
                    ))
                },
            )
            .map_err(unavailable("query continuation authorization"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read continuation authorization"))?;
        if rows.len() > 1 {
            return Err(StorePortError::Conflict {
                detail: "multiple unconsumed continuation authorizations match scheduler work"
                    .to_owned(),
            });
        }
        let Some((
            authorization_id,
            loop_object_id,
            iteration,
            expected_loop_version,
            checkpoint_id,
            budget_id,
            budget_charge_canonical_json,
            verification_report_id,
            issued_fencing_epoch,
            canonical_json,
        )) = rows.into_iter().next()
        else {
            return Ok(None);
        };
        Ok(Some(ContinuationAuthorizationRow {
            continuation_authorization_id: ObjectId::parse(&authorization_id)
                .map_err(|error| corrupt("continuation authorization id", error))?,
            task_binding: task_binding.clone(),
            loop_object_id: ObjectId::parse(&loop_object_id)
                .map_err(|error| corrupt("continuation authorization loop id", error))?,
            iteration,
            expected_loop_version: Version::new(expected_loop_version)
                .map_err(|error| corrupt("continuation authorization loop version", error))?,
            checkpoint_id: ObjectId::parse(&checkpoint_id)
                .map_err(|error| corrupt("continuation authorization checkpoint id", error))?,
            budget_id: BudgetId::parse(&budget_id)
                .map_err(|error| corrupt("continuation authorization budget id", error))?,
            budget_charge_canonical_json,
            verification_report_id: ObjectId::parse(&verification_report_id)
                .map_err(|error| corrupt("continuation authorization report id", error))?,
            issued_fencing_epoch,
            canonical_json,
        }))
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
