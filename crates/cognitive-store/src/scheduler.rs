//! Durable scheduler repository (P2-T03).
//!
//! Persists runnable task state, lease ownership with epoch fencing, next
//! eligible time, attempt accounting and cancel requests. All mutable
//! operations run inside the real SQLite WAL transaction; lease takeover
//! is a CAS so two workers can never hold the same lease.
//!
//! Data layout (migration v2, authority database):
//! - `scheduler_entries`: one row per task_ref with state, lease owner/
//!   epoch/expiry, next_eligible, attempt count, cancel flag.
//!
//! No scheduler table existed before this slice; this is the first schema
//! addition to the authority database after the P1-T01 v1 full schema.

use crate::migration::MigrationPlanEntry;
use cognitive_domain::WallTimestamp;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use std::path::Path;
use thiserror::Error;

/// Migration v2: durable scheduler persistence tables.
pub const SCHEDULER_SCHEMA_V2: &str = "
CREATE TABLE IF NOT EXISTS scheduler_entries (
  task_ref        TEXT PRIMARY KEY,
  state           TEXT NOT NULL,
  lease_owner     TEXT,
  lease_epoch     INTEGER NOT NULL DEFAULT 0,
  lease_expires   TEXT,
  next_eligible   TEXT NOT NULL,
  attempt_count   INTEGER NOT NULL DEFAULT 0,
  cancel_requested INTEGER NOT NULL DEFAULT 0
) STRICT;
";

/// The version-2 migration entry (appended after authority v1).
pub fn scheduler_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(2, SCHEDULER_SCHEMA_V2)
}

/// Scheduler row state machine (deterministic, product-owned).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerState {
    Pending,
    Runnable,
    Leased,
    Succeeded,
    Failed,
    Cancelled,
}

impl SchedulerState {
    /// Wire text of this state (persisted form).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Runnable => "runnable",
            Self::Leased => "leased",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// One durable scheduler row.
#[derive(Debug, Clone, PartialEq)]
pub struct SchedulerRow {
    pub task_ref: String,
    pub state: String,
    pub lease_owner: Option<String>,
    pub lease_epoch: i64,
    pub lease_expires: Option<String>,
    pub next_eligible: String,
    pub attempt_count: i64,
    pub cancel_requested: bool,
}

/// Fail-closed scheduler repository error.
#[derive(Debug, Error)]
pub enum SchedulerRepositoryError {
    #[error("scheduler store unavailable: {0}")]
    Unavailable(String),
    #[error("scheduler lease conflict: {0}")]
    LeaseConflict(String),
    #[error("scheduler row not found: {0}")]
    NotFound(String),
    #[error("scheduler timestamp is invalid: {0}")]
    InvalidTimestamp(String),
}

/// Durable scheduler repository over one authority database connection.
pub struct SchedulerRepository {
    conn: Connection,
}

impl SchedulerRepository {
    /// Open the scheduler repository on a database file (creates schema).
    pub fn open(path: &Path) -> Result<Self, SchedulerRepositoryError> {
        let conn = Connection::open(path)
            .map_err(|e| SchedulerRepositoryError::Unavailable(format!("open: {e}")))?;
        conn.execute_batch(SCHEDULER_SCHEMA_V2)
            .map_err(|e| SchedulerRepositoryError::Unavailable(format!("schema: {e}")))?;
        Ok(Self { conn })
    }

    /// Insert or replace one scheduler row (upsert by task_ref).
    pub fn upsert(&mut self, row: &SchedulerRow) -> Result<(), SchedulerRepositoryError> {
        self.conn
            .execute(
                "INSERT INTO scheduler_entries \
                 (task_ref, state, lease_owner, lease_epoch, lease_expires, next_eligible, attempt_count, cancel_requested) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8) \
                 ON CONFLICT(task_ref) DO UPDATE SET \
                   state=excluded.state, \
                   lease_owner=excluded.lease_owner, \
                   lease_epoch=excluded.lease_epoch, \
                   lease_expires=excluded.lease_expires, \
                   next_eligible=excluded.next_eligible, \
                   attempt_count=excluded.attempt_count, \
                   cancel_requested=excluded.cancel_requested",
                rusqlite::params![
                    row.task_ref,
                    row.state,
                    row.lease_owner,
                    row.lease_epoch,
                    row.lease_expires,
                    row.next_eligible,
                    row.attempt_count,
                    row.cancel_requested as i64,
                ],
            )
            .map_err(|e| SchedulerRepositoryError::Unavailable(format!("upsert: {e}")))?;
        Ok(())
    }

    /// Atomically acquire a lease with CAS: the row must currently be
    /// `runnable` (or unleased) and unexpired-leased. A stale/duplicate
    /// owner is refused. Returns the updated row.
    pub fn acquire_lease(
        &mut self,
        task_ref: &str,
        owner: &str,
        lease_epoch: i64,
        expires: &str,
    ) -> Result<SchedulerRow, SchedulerRepositoryError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|e| SchedulerRepositoryError::Unavailable(format!("tx: {e}")))?;
        let existing: Option<SchedulerRow> = tx
            .query_row(
                "SELECT task_ref, state, lease_owner, lease_epoch, lease_expires, \
                 next_eligible, attempt_count, cancel_requested \
                 FROM scheduler_entries WHERE task_ref = ?1",
                [task_ref],
                |row| {
                    Ok(SchedulerRow {
                        task_ref: row.get(0)?,
                        state: row.get(1)?,
                        lease_owner: row.get(2)?,
                        lease_epoch: row.get(3)?,
                        lease_expires: row.get(4)?,
                        next_eligible: row.get(5)?,
                        attempt_count: row.get(6)?,
                        cancel_requested: row.get::<_, i64>(7)? != 0,
                    })
                },
            )
            .optional()
            .map_err(|e| SchedulerRepositoryError::Unavailable(format!("read: {e}")))?;

        match existing {
            None => Err(SchedulerRepositoryError::NotFound(task_ref.to_owned())),
            Some(row) if row.cancel_requested => Err(SchedulerRepositoryError::LeaseConflict(
                "task is cancelled".to_owned(),
            )),
            Some(row) if row.lease_owner.is_some() && row.state == "leased" => {
                Err(SchedulerRepositoryError::LeaseConflict(format!(
                    "task {} already leased by {}",
                    task_ref,
                    row.lease_owner.unwrap_or_default()
                )))
            }
            Some(_) => {
                tx.execute(
                    "UPDATE scheduler_entries \
                     SET state='leased', lease_owner=?2, lease_epoch=?3, lease_expires=?4, \
                         attempt_count = attempt_count + 1 \
                     WHERE task_ref=?1",
                    rusqlite::params![task_ref, owner, lease_epoch, expires],
                )
                .map_err(|e| SchedulerRepositoryError::Unavailable(format!("lease: {e}")))?;
                tx.commit()
                    .map_err(|e| SchedulerRepositoryError::Unavailable(format!("commit: {e}")))?;
                self.load(task_ref)?
                    .ok_or_else(|| SchedulerRepositoryError::NotFound(task_ref.to_owned()))
            }
        }
    }

    /// Atomically claim work that is eligible at `now`. A currently leased
    /// entry can be reclaimed only after its persisted expiry; the successor
    /// must use a strictly greater epoch to fence the former worker.
    pub fn acquire_eligible_lease(
        &mut self,
        task_ref: &str,
        owner: &str,
        lease_epoch: i64,
        now: &str,
        expires: &str,
    ) -> Result<SchedulerRow, SchedulerRepositoryError> {
        let now_timestamp = parse_scheduler_timestamp(now)?;
        let expires_timestamp = parse_scheduler_timestamp(expires)?;
        if expires_timestamp.instant_key() <= now_timestamp.instant_key() {
            return Err(SchedulerRepositoryError::LeaseConflict(
                "lease expiry must be after the trusted scheduler time".to_owned(),
            ));
        }

        let existing = self
            .load(task_ref)?
            .ok_or_else(|| SchedulerRepositoryError::NotFound(task_ref.to_owned()))?;
        let next_eligible = parse_scheduler_timestamp(&existing.next_eligible)?;
        let lease_is_expired = existing
            .lease_expires
            .as_deref()
            .map(parse_scheduler_timestamp)
            .transpose()?
            .is_some_and(|lease_expiry| lease_expiry.instant_key() <= now_timestamp.instant_key());
        let is_runnable_now = existing.state == SchedulerState::Runnable.as_str()
            && next_eligible.instant_key() <= now_timestamp.instant_key();
        let is_expired_lease =
            existing.state == SchedulerState::Leased.as_str() && lease_is_expired;

        if existing.cancel_requested || (!is_runnable_now && !is_expired_lease) {
            return Err(SchedulerRepositoryError::LeaseConflict(
                "task is not eligible for dispatch".to_owned(),
            ));
        }
        if lease_epoch <= existing.lease_epoch {
            return Err(SchedulerRepositoryError::LeaseConflict(
                "lease epoch must advance the existing fence".to_owned(),
            ));
        }

        let updated = self
            .conn
            .execute(
                "UPDATE scheduler_entries \
                 SET state='leased', lease_owner=?2, lease_epoch=?3, lease_expires=?4, \
                     attempt_count=attempt_count + 1 \
                 WHERE task_ref=?1 AND cancel_requested=0 AND lease_epoch < ?3 \
                   AND ((state='runnable' AND julianday(next_eligible) <= julianday(?5)) \
                     OR (state='leased' AND lease_expires IS NOT NULL \
                         AND julianday(lease_expires) <= julianday(?5)))",
                rusqlite::params![task_ref, owner, lease_epoch, expires, now],
            )
            .map_err(|error| {
                SchedulerRepositoryError::Unavailable(format!("eligible lease: {error}"))
            })?;
        if updated != 1 {
            return Err(SchedulerRepositoryError::LeaseConflict(
                "concurrent scheduler update refused lease acquisition".to_owned(),
            ));
        }
        self.load(task_ref)?
            .ok_or_else(|| SchedulerRepositoryError::NotFound(task_ref.to_owned()))
    }

    /// Release the exact fenced lease held by a worker.
    ///
    /// Both worker identity and lease epoch must match the persisted lease.
    /// This prevents a timed-out attempt from releasing a successor lease when
    /// a restarted worker reuses the same logical owner identity.
    pub fn release_lease(
        &mut self,
        task_ref: &str,
        owner: &str,
        lease_epoch: i64,
        next_state: SchedulerState,
        next_eligible: &str,
    ) -> Result<SchedulerRow, SchedulerRepositoryError> {
        let updated = self
            .conn
            .execute(
                "UPDATE scheduler_entries \
             SET state=?2, lease_owner=NULL, lease_epoch=lease_epoch, \
                 lease_expires=NULL, next_eligible=?3 \
             WHERE task_ref=?1 AND lease_owner=?4 AND lease_epoch=?5",
                rusqlite::params![
                    task_ref,
                    next_state.as_str(),
                    next_eligible,
                    owner,
                    lease_epoch
                ],
            )
            .map_err(|e| SchedulerRepositoryError::Unavailable(format!("release: {e}")))?;
        if updated == 0 {
            return Err(SchedulerRepositoryError::LeaseConflict(
                "lease owner or epoch mismatch on release".to_owned(),
            ));
        }
        self.load(task_ref)?
            .ok_or_else(|| SchedulerRepositoryError::NotFound(task_ref.to_owned()))
    }

    /// Load one row.
    pub fn load(
        &mut self,
        task_ref: &str,
    ) -> Result<Option<SchedulerRow>, SchedulerRepositoryError> {
        self.conn
            .query_row(
                "SELECT task_ref, state, lease_owner, lease_epoch, lease_expires, \
                 next_eligible, attempt_count, cancel_requested \
                 FROM scheduler_entries WHERE task_ref = ?1",
                [task_ref],
                |row| {
                    Ok(SchedulerRow {
                        task_ref: row.get(0)?,
                        state: row.get(1)?,
                        lease_owner: row.get(2)?,
                        lease_epoch: row.get(3)?,
                        lease_expires: row.get(4)?,
                        next_eligible: row.get(5)?,
                        attempt_count: row.get(6)?,
                        cancel_requested: row.get::<_, i64>(7)? != 0,
                    })
                },
            )
            .optional()
            .map_err(|e| SchedulerRepositoryError::Unavailable(format!("load: {e}")))
    }

    /// List work that a daemon recovery loop must inspect from durable state.
    ///
    /// This deliberately returns every non-terminal row, including active
    /// leases. The daemon must reload each row's expiry and authority facts
    /// before attempting a claim; this method must never itself imply that a
    /// row is eligible or that a worker may dispatch it.
    pub fn list_recoverable(&mut self) -> Result<Vec<SchedulerRow>, SchedulerRepositoryError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT task_ref, state, lease_owner, lease_epoch, lease_expires, \
                 next_eligible, attempt_count, cancel_requested \
                 FROM scheduler_entries \
                 WHERE state IN ('pending', 'runnable', 'leased') \
                 ORDER BY task_ref",
            )
            .map_err(|error| {
                SchedulerRepositoryError::Unavailable(format!("list work: {error}"))
            })?;
        let rows = statement
            .query_map([], |row| {
                Ok(SchedulerRow {
                    task_ref: row.get(0)?,
                    state: row.get(1)?,
                    lease_owner: row.get(2)?,
                    lease_epoch: row.get(3)?,
                    lease_expires: row.get(4)?,
                    next_eligible: row.get(5)?,
                    attempt_count: row.get(6)?,
                    cancel_requested: row.get::<_, i64>(7)? != 0,
                })
            })
            .map_err(|error| {
                SchedulerRepositoryError::Unavailable(format!("query work: {error}"))
            })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| SchedulerRepositoryError::Unavailable(format!("read work: {error}")))
    }

    /// Mark a task cancelled (cancel request propagated durably).
    pub fn request_cancel(&mut self, task_ref: &str) -> Result<(), SchedulerRepositoryError> {
        self.conn
            .execute(
                "UPDATE scheduler_entries SET cancel_requested=1 WHERE task_ref=?1",
                [task_ref],
            )
            .map_err(|e| SchedulerRepositoryError::Unavailable(format!("cancel: {e}")))?;
        Ok(())
    }
}

fn parse_scheduler_timestamp(value: &str) -> Result<WallTimestamp, SchedulerRepositoryError> {
    WallTimestamp::parse(value)
        .map_err(|_| SchedulerRepositoryError::InvalidTimestamp(value.to_owned()))
}
