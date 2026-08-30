//! Personal-private Routine / Trigger / missed ledger (P11-T08, authority
//! migration v33).
//!
//! Daemon-authored Routine revisions plus Trigger admission reuse the existing
//! `scheduler_entries` table. There is no second scheduler, Temporal service,
//! or engine store. Checkpoint bytes are recovery input, not completion.

use crate::migration::MigrationPlanEntry;
use crate::project_aggregate::{ConfirmCaller, ProjectAggregateError};
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

/// Personal-private Routine envelope (P11-T08).
pub const ROUTINE_PROJECTION_ID: &str = "cognitiveos.personal.routine/0.1";

/// Authority migration v33: Routine revision, occurrence ledger, missed/coalesced.
pub const ROUTINE_SCHEMA_V33: &str = "
CREATE TABLE p11_routine (
  routine_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  current_revision_id TEXT NOT NULL,
  created_at INTEGER NOT NULL
) STRICT;
CREATE TABLE p11_routine_revision (
  revision_id TEXT PRIMARY KEY,
  routine_id TEXT NOT NULL,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  seq INTEGER NOT NULL CHECK (seq >= 1),
  policy_digest TEXT NOT NULL CHECK (length(policy_digest) = 64),
  overlap_policy TEXT NOT NULL CHECK (overlap_policy = 'no-overlap-queue-latest'),
  catch_up_policy TEXT NOT NULL CHECK (catch_up_policy IN ('missed-visible','coalesce')),
  risk_class TEXT NOT NULL CHECK (risk_class IN ('internal','consequential')),
  body_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  UNIQUE (routine_id, seq)
) STRICT;
CREATE INDEX p11_routine_revision_current
  ON p11_routine_revision(routine_id, seq);
CREATE TABLE p11_routine_occurrence (
  occurrence_id TEXT PRIMARY KEY,
  routine_id TEXT NOT NULL,
  revision_id TEXT NOT NULL,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  trigger_kind TEXT NOT NULL CHECK (trigger_kind IN (
    'manual','schedule','qualified-event'
  )),
  trigger_source TEXT NOT NULL,
  requested_at INTEGER NOT NULL,
  disposition TEXT NOT NULL CHECK (disposition IN (
    'active','queued','coalesced','missed','cancelled'
  )),
  coalesced_by TEXT,
  miss_reason TEXT,
  policy_digest TEXT NOT NULL CHECK (length(policy_digest) = 64),
  scheduler_task_ref TEXT,
  checkpoint_json TEXT,
  recorded_at INTEGER NOT NULL
) STRICT;
CREATE INDEX p11_routine_occurrence_scope
  ON p11_routine_occurrence(routine_id, disposition, recorded_at);
";

/// v33 migration entry.
pub fn routine_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(33, ROUTINE_SCHEMA_V33)
}

/// Daemon scheduler work key for one Routine occurrence. Not a second scheduler.
pub fn routine_scheduler_task_ref(occurrence_id: &str) -> String {
    format!("task://personal/routine/{occurrence_id}")
}

/// New or successor Routine revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutineRevisionSpec<'a> {
    pub project_id: &'a str,
    pub routine_id: Option<&'a str>,
    pub body_json: &'a str,
    pub risk_class: &'a str,
    pub now_ms: i64,
}

/// Trigger admission input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutineTriggerSpec<'a> {
    pub routine_id: &'a str,
    pub revision_id: &'a str,
    pub trigger_kind: &'a str,
    pub trigger_source: &'a str,
    pub force_parallel: bool,
    pub host_unavailable: bool,
    pub now_ms: i64,
}

/// One ledger row (active / queued / missed / coalesced).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineOccurrence {
    pub occurrence_id: String,
    pub routine_id: String,
    pub revision_id: String,
    pub project_id: String,
    pub trigger_kind: String,
    pub trigger_source: String,
    pub requested_at: i64,
    pub disposition: String,
    pub coalesced_by: Option<String>,
    pub miss_reason: Option<String>,
    pub policy_digest: String,
    pub scheduler_task_ref: Option<String>,
    pub checkpoint_json: Option<String>,
    pub recorded_at: i64,
}

/// Published Routine identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineRevision {
    pub routine_id: String,
    pub revision_id: String,
    pub seq: i64,
    pub policy_digest: String,
    pub risk_class: String,
}

/// Personal-private Routine store on the authority writer.
#[derive(Clone)]
pub struct RoutineStore {
    conn: Arc<Mutex<Connection>>,
}

impl RoutineStore {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            conn: Arc::clone(&store.conn),
        }
    }

    /// Open the authority database path (tests).
    pub fn open_path(path: &std::path::Path) -> Result<Self, ProjectAggregateError> {
        let conn = Connection::open(path).map_err(unavailable("open"))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;",
        )
        .map_err(unavailable("pragma"))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ProjectAggregateError> {
        self.conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })
    }

    /// Daemon-authored Routine revision. First call creates the Routine.
    pub fn publish_revision(
        &self,
        caller: ConfirmCaller,
        spec: &RoutineRevisionSpec<'_>,
    ) -> Result<RoutineRevision, ProjectAggregateError> {
        require_owner(caller)?;
        reject_secret_shape(spec.body_json)?;
        require_json_object(spec.body_json)?;
        require_risk_class(spec.risk_class)?;
        let policy_digest = digest_hex(spec.body_json.as_bytes());
        let conn = self.lock()?;
        let project_found: Option<String> = conn
            .query_row(
                "SELECT project_id FROM p11_project WHERE project_id = ?1",
                params![spec.project_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("lookup project"))?;
        if project_found.is_none() {
            return Err(ProjectAggregateError::NotFound {
                detail: "project not found",
            });
        }
        let (routine_id, seq) = match spec.routine_id {
            Some(existing) => {
                let row: Option<(String, i64)> = conn
                    .query_row(
                        "SELECT routine_id, COALESCE((
                           SELECT MAX(seq) FROM p11_routine_revision WHERE routine_id = ?1
                         ), 0)
                         FROM p11_routine WHERE routine_id = ?1 AND project_id = ?2",
                        params![existing, spec.project_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()
                    .map_err(unavailable("lookup routine"))?;
                let Some((routine_id, last_seq)) = row else {
                    return Err(ProjectAggregateError::NotFound {
                        detail: "routine not found",
                    });
                };
                (routine_id, last_seq + 1)
            }
            None => (next_id("routine")?, 1),
        };
        let revision_id = next_id("rrev")?;
        if seq == 1 {
            conn.execute(
                "INSERT INTO p11_routine (
                   routine_id, project_id, current_revision_id, created_at
                 ) VALUES (?1, ?2, ?3, ?4)",
                params![routine_id, spec.project_id, revision_id, spec.now_ms],
            )
            .map_err(unavailable("insert routine"))?;
        }
        conn.execute(
            "INSERT INTO p11_routine_revision (
               revision_id, routine_id, project_id, seq, policy_digest,
               overlap_policy, catch_up_policy, risk_class, body_json, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, 'no-overlap-queue-latest', 'missed-visible', ?6, ?7, ?8)",
            params![
                revision_id,
                routine_id,
                spec.project_id,
                seq,
                policy_digest,
                spec.risk_class,
                spec.body_json,
                spec.now_ms
            ],
        )
        .map_err(unavailable("insert revision"))?;
        conn.execute(
            "UPDATE p11_routine SET current_revision_id = ?1 WHERE routine_id = ?2",
            params![revision_id, routine_id],
        )
        .map_err(unavailable("point current revision"))?;
        Ok(RoutineRevision {
            routine_id,
            revision_id,
            seq,
            policy_digest,
            risk_class: spec.risk_class.to_owned(),
        })
    }

    /// Admit a Trigger. No-overlap / queue-latest; missed is visible.
    pub fn admit_trigger(
        &self,
        caller: ConfirmCaller,
        spec: &RoutineTriggerSpec<'_>,
    ) -> Result<RoutineOccurrence, ProjectAggregateError> {
        require_owner(caller)?;
        require_trigger_kind(spec.trigger_kind)?;
        reject_secret_shape(spec.trigger_source)?;
        let conn = self.lock()?;
        let current: Option<(String, String, String, String)> = conn
            .query_row(
                "SELECT r.project_id, r.current_revision_id, v.policy_digest, v.risk_class
                 FROM p11_routine r
                 JOIN p11_routine_revision v ON v.revision_id = r.current_revision_id
                 WHERE r.routine_id = ?1",
                params![spec.routine_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(unavailable("load current revision"))?;
        let Some((project_id, current_revision_id, policy_digest, _risk_class)) = current else {
            return Err(ProjectAggregateError::NotFound {
                detail: "routine not found",
            });
        };
        if spec.revision_id != current_revision_id {
            return Err(ProjectAggregateError::Stale {
                detail: "stale Routine policy/revision is rejected",
            });
        }
        let active_id = load_one_disposition(&conn, spec.routine_id, "active")?;
        if spec.force_parallel && active_id.is_some() {
            return Err(ProjectAggregateError::Conflict {
                detail: "overlap rejected: one Routine cannot run two active occurrences",
            });
        }
        let occurrence_id = next_id("occ")?;
        if spec.host_unavailable {
            insert_occurrence(
                &conn,
                OccurrenceWrite {
                    occurrence_id: &occurrence_id,
                    routine_id: spec.routine_id,
                    revision_id: spec.revision_id,
                    project_id: &project_id,
                    trigger_kind: spec.trigger_kind,
                    trigger_source: spec.trigger_source,
                    requested_at: spec.now_ms,
                    disposition: "missed",
                    coalesced_by: None,
                    miss_reason: Some("host-unavailable"),
                    policy_digest: &policy_digest,
                    scheduler_task_ref: None,
                    recorded_at: spec.now_ms,
                },
            )?;
            return load_occurrence(&conn, &occurrence_id);
        }
        if active_id.is_some() {
            if let Some(queued_id) = load_one_disposition(&conn, spec.routine_id, "queued")? {
                conn.execute(
                    "UPDATE p11_routine_occurrence
                     SET disposition = 'coalesced', coalesced_by = ?1
                     WHERE occurrence_id = ?2",
                    params![occurrence_id, queued_id],
                )
                .map_err(unavailable("coalesce queued"))?;
            }
            insert_occurrence(
                &conn,
                OccurrenceWrite {
                    occurrence_id: &occurrence_id,
                    routine_id: spec.routine_id,
                    revision_id: spec.revision_id,
                    project_id: &project_id,
                    trigger_kind: spec.trigger_kind,
                    trigger_source: spec.trigger_source,
                    requested_at: spec.now_ms,
                    disposition: "queued",
                    coalesced_by: None,
                    miss_reason: None,
                    policy_digest: &policy_digest,
                    scheduler_task_ref: None,
                    recorded_at: spec.now_ms,
                },
            )?;
            return load_occurrence(&conn, &occurrence_id);
        }
        let task_ref = routine_scheduler_task_ref(&occurrence_id);
        insert_occurrence(
            &conn,
            OccurrenceWrite {
                occurrence_id: &occurrence_id,
                routine_id: spec.routine_id,
                revision_id: spec.revision_id,
                project_id: &project_id,
                trigger_kind: spec.trigger_kind,
                trigger_source: spec.trigger_source,
                requested_at: spec.now_ms,
                disposition: "active",
                coalesced_by: None,
                miss_reason: None,
                policy_digest: &policy_digest,
                scheduler_task_ref: Some(&task_ref),
                recorded_at: spec.now_ms,
            },
        )?;
        upsert_scheduler_row(&conn, &task_ref)?;
        load_occurrence(&conn, &occurrence_id)
    }

    /// Visible missed / coalesced / queued / active ledger. Not Inbox L1.
    pub fn list_ledger(
        &self,
        project_id: &str,
        routine_id: &str,
    ) -> Result<Vec<RoutineOccurrence>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT occurrence_id, routine_id, revision_id, project_id, trigger_kind,
                        trigger_source, requested_at, disposition, coalesced_by, miss_reason,
                        policy_digest, scheduler_task_ref, checkpoint_json, recorded_at
                 FROM p11_routine_occurrence
                 WHERE project_id = ?1 AND routine_id = ?2
                 ORDER BY recorded_at, occurrence_id",
            )
            .map_err(unavailable("prepare ledger"))?;
        let rows = statement
            .query_map(params![project_id, routine_id], map_occurrence)
            .map_err(unavailable("query ledger"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect ledger"))
    }

    /// Persist a checkpoint. Completing from a checkpoint is forbidden.
    pub fn record_checkpoint(
        &self,
        caller: ConfirmCaller,
        occurrence_id: &str,
        checkpoint_json: &str,
        complete: bool,
    ) -> Result<RoutineOccurrence, ProjectAggregateError> {
        require_owner(caller)?;
        reject_secret_shape(checkpoint_json)?;
        if complete {
            return Err(ProjectAggregateError::Invalid {
                detail: "checkpoint is not completion",
            });
        }
        let conn = self.lock()?;
        let updated = conn
            .execute(
                "UPDATE p11_routine_occurrence
                 SET checkpoint_json = ?1
                 WHERE occurrence_id = ?2",
                params![checkpoint_json, occurrence_id],
            )
            .map_err(unavailable("record checkpoint"))?;
        if updated == 0 {
            return Err(ProjectAggregateError::NotFound {
                detail: "occurrence not found",
            });
        }
        load_occurrence(&conn, occurrence_id)
    }

    /// Resume a missed occurrence. Consequential auto-resume fails closed.
    pub fn resume_missed(
        &self,
        caller: ConfirmCaller,
        occurrence_id: &str,
        now_ms: i64,
    ) -> Result<RoutineOccurrence, ProjectAggregateError> {
        require_owner(caller)?;
        let conn = self.lock()?;
        let row: Option<(String, String, String)> = conn
            .query_row(
                "SELECT o.routine_id, o.disposition, v.risk_class
                 FROM p11_routine_occurrence o
                 JOIN p11_routine_revision v ON v.revision_id = o.revision_id
                 WHERE o.occurrence_id = ?1",
                params![occurrence_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(unavailable("load missed"))?;
        let Some((routine_id, disposition, risk_class)) = row else {
            return Err(ProjectAggregateError::NotFound {
                detail: "occurrence not found",
            });
        };
        if disposition != "missed" {
            return Err(ProjectAggregateError::Invalid {
                detail: "only missed occurrences may resume",
            });
        }
        if risk_class == "consequential" {
            return Err(ProjectAggregateError::Forbidden {
                detail: "consequential auto-resume is forbidden",
            });
        }
        if load_one_disposition(&conn, &routine_id, "active")?.is_some() {
            return Err(ProjectAggregateError::Conflict {
                detail: "overlap rejected: one Routine cannot run two active occurrences",
            });
        }
        let task_ref = routine_scheduler_task_ref(occurrence_id);
        conn.execute(
            "UPDATE p11_routine_occurrence
             SET disposition = 'active', miss_reason = NULL, scheduler_task_ref = ?1, recorded_at = ?2
             WHERE occurrence_id = ?3",
            params![task_ref, now_ms, occurrence_id],
        )
        .map_err(unavailable("resume missed"))?;
        upsert_scheduler_row(&conn, &task_ref)?;
        load_occurrence(&conn, occurrence_id)
    }
}

struct OccurrenceWrite<'a> {
    occurrence_id: &'a str,
    routine_id: &'a str,
    revision_id: &'a str,
    project_id: &'a str,
    trigger_kind: &'a str,
    trigger_source: &'a str,
    requested_at: i64,
    disposition: &'a str,
    coalesced_by: Option<&'a str>,
    miss_reason: Option<&'a str>,
    policy_digest: &'a str,
    scheduler_task_ref: Option<&'a str>,
    recorded_at: i64,
}

fn insert_occurrence(
    conn: &Connection,
    write: OccurrenceWrite<'_>,
) -> Result<(), ProjectAggregateError> {
    conn.execute(
        "INSERT INTO p11_routine_occurrence (
           occurrence_id, routine_id, revision_id, project_id, trigger_kind,
           trigger_source, requested_at, disposition, coalesced_by, miss_reason,
           policy_digest, scheduler_task_ref, checkpoint_json, recorded_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, NULL, ?13)",
        params![
            write.occurrence_id,
            write.routine_id,
            write.revision_id,
            write.project_id,
            write.trigger_kind,
            write.trigger_source,
            write.requested_at,
            write.disposition,
            write.coalesced_by,
            write.miss_reason,
            write.policy_digest,
            write.scheduler_task_ref,
            write.recorded_at,
        ],
    )
    .map_err(unavailable("insert occurrence"))?;
    Ok(())
}

fn upsert_scheduler_row(conn: &Connection, task_ref: &str) -> Result<(), ProjectAggregateError> {
    conn.execute(
        "INSERT INTO scheduler_entries (
           task_ref, contract_epoch, state, lease_owner, lease_epoch, lease_expires,
           next_eligible, attempt_count, cancel_requested
         ) VALUES (?1, 1, 'runnable', NULL, 0, NULL, '2026-01-01T00:00:00Z', 0, 0)
         ON CONFLICT(task_ref, contract_epoch) DO UPDATE SET
           state = excluded.state,
           next_eligible = excluded.next_eligible",
        params![task_ref],
    )
    .map_err(unavailable("upsert scheduler row"))?;
    Ok(())
}

fn load_one_disposition(
    conn: &Connection,
    routine_id: &str,
    disposition: &str,
) -> Result<Option<String>, ProjectAggregateError> {
    conn.query_row(
        "SELECT occurrence_id FROM p11_routine_occurrence
         WHERE routine_id = ?1 AND disposition = ?2
         ORDER BY recorded_at DESC LIMIT 1",
        params![routine_id, disposition],
        |row| row.get(0),
    )
    .optional()
    .map_err(unavailable("load disposition"))
}

fn load_occurrence(
    conn: &Connection,
    occurrence_id: &str,
) -> Result<RoutineOccurrence, ProjectAggregateError> {
    conn.query_row(
        "SELECT occurrence_id, routine_id, revision_id, project_id, trigger_kind,
                trigger_source, requested_at, disposition, coalesced_by, miss_reason,
                policy_digest, scheduler_task_ref, checkpoint_json, recorded_at
         FROM p11_routine_occurrence WHERE occurrence_id = ?1",
        params![occurrence_id],
        map_occurrence,
    )
    .map_err(unavailable("load occurrence"))
}

fn map_occurrence(row: &rusqlite::Row<'_>) -> rusqlite::Result<RoutineOccurrence> {
    Ok(RoutineOccurrence {
        occurrence_id: row.get(0)?,
        routine_id: row.get(1)?,
        revision_id: row.get(2)?,
        project_id: row.get(3)?,
        trigger_kind: row.get(4)?,
        trigger_source: row.get(5)?,
        requested_at: row.get(6)?,
        disposition: row.get(7)?,
        coalesced_by: row.get(8)?,
        miss_reason: row.get(9)?,
        policy_digest: row.get(10)?,
        scheduler_task_ref: row.get(11)?,
        checkpoint_json: row.get(12)?,
        recorded_at: row.get(13)?,
    })
}

fn require_owner(caller: ConfirmCaller) -> Result<(), ProjectAggregateError> {
    match caller {
        ConfirmCaller::OwnerManagement => Ok(()),
        ConfirmCaller::TaskChannel | ConfirmCaller::Assistant => {
            Err(ProjectAggregateError::Forbidden {
                detail: "only owner management session may confirm or apply",
            })
        }
    }
}

fn require_risk_class(value: &str) -> Result<(), ProjectAggregateError> {
    if value == "internal" || value == "consequential" {
        return Ok(());
    }
    Err(ProjectAggregateError::Invalid {
        detail: "risk_class must be internal or consequential",
    })
}

fn require_trigger_kind(value: &str) -> Result<(), ProjectAggregateError> {
    if value == "manual" || value == "schedule" || value == "qualified-event" {
        return Ok(());
    }
    Err(ProjectAggregateError::Invalid {
        detail: "trigger_kind must be manual, schedule, or qualified-event",
    })
}

fn require_json_object(body: &str) -> Result<(), ProjectAggregateError> {
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(serde_json::Value::Object(_)) => Ok(()),
        _ => Err(ProjectAggregateError::Invalid {
            detail: "Routine body must be a JSON object",
        }),
    }
}

fn reject_secret_shape(body: &str) -> Result<(), ProjectAggregateError> {
    let lowered = body.to_ascii_lowercase();
    if lowered.contains("sk-")
        || lowered.contains("bearer ")
        || lowered.contains("api_key")
        || lowered.contains("x-api-key")
        || lowered.contains("ssv1:")
    {
        return Err(ProjectAggregateError::Invalid {
            detail: "secret-shaped material is rejected at registration",
        });
    }
    Ok(())
}

fn digest_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn next_id(prefix: &str) -> Result<String, ProjectAggregateError> {
    let generated = uuid::Uuid::now_v7().as_hyphenated().to_string();
    Ok(format!("{prefix}-{generated}"))
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}
