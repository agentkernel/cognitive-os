//! Hosted DSH Attempt ledger (P13-T02, authority migration v36).
//!
//! One seated Member's Attempt on one Task through the hidden hosted DSH child
//! is persisted **before** the daemon spawns anything (Intent), marked
//! `dispatched` once the OS process exists (Effect dispatch marker), and
//! closed by a daemon-observed terminal row. Every frame the child emitted is
//! an observation (`authority_written = 0`); no frame, heartbeat, exit code,
//! or `response` status can advance Task, Effect, or Verification state.
//! `completion_claimed` is CHECK-pinned to `0` and `verification_status` to
//! `not-run`: completion belongs to the independent verifier (`P13-T04`).
//! Rows left `persisted`/`dispatched` by a daemon crash reconcile to
//! `unknown-outcome`, never to success. Artifact health / update / rollback
//! facts are an append-only ledger the spawn gate reads.

use crate::employee::{EmployeeStore, reject_installed_agent_chrome, reject_pi_member_engine};
use crate::hosted_dsh::{
    HOSTED_DSH_ARTIFACT_DIGEST, HOSTED_DSH_WIN_GNU_FENCE, HostedDshPlane, secret_shaped_value,
};
use crate::migration::MigrationPlanEntry;
use crate::project_aggregate::{ConfirmCaller, ProjectAggregateError};
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

/// Personal-private hosted Attempt envelope. Hidden capability, not chrome.
pub const HOSTED_ATTEMPT_PROJECTION_ID: &str = "cognitiveos.personal.hosted-dsh-attempt/0.1";
/// Bounded Context payload ceiling handed to one child (bytes). The runtime
/// broker imports this constant; the ledger CHECK pins the same number.
pub const HOSTED_ATTEMPT_CONTEXT_MAX_BYTES: usize = 64 * 1024;
/// Frames retained per Attempt in the durable ledger.
pub const HOSTED_ATTEMPT_MAX_FRAMES: usize = 512;
/// Redacted text retained per durable frame.
pub const HOSTED_ATTEMPT_FRAME_TEXT_MAX_CHARS: usize = 1024;

/// Authority migration v36: hosted DSH artifact facts + Attempt + frame ledger.
pub const HOSTED_DSH_ATTEMPT_SCHEMA_V36: &str = "
CREATE TABLE p13_hosted_dsh_artifact_fact (
  fact_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK (kind IN ('health-check','update','rollback')),
  expected_revision TEXT NOT NULL,
  configured_revision TEXT,
  pin_file_revision TEXT,
  health TEXT NOT NULL CHECK (health IN ('pinned','absent','corrupt','mismatch','script-missing')),
  child_script_digest TEXT,
  previous_fact_id TEXT,
  detail_redacted TEXT NOT NULL,
  created_at INTEGER NOT NULL
) STRICT;
CREATE INDEX p13_hosted_dsh_artifact_fact_created
  ON p13_hosted_dsh_artifact_fact(created_at);
CREATE TRIGGER p13_hosted_dsh_artifact_fact_append_only_update
BEFORE UPDATE ON p13_hosted_dsh_artifact_fact
BEGIN SELECT RAISE(ABORT, 'append-only: hosted DSH artifact facts are immutable'); END;
CREATE TRIGGER p13_hosted_dsh_artifact_fact_append_only_delete
BEFORE DELETE ON p13_hosted_dsh_artifact_fact
BEGIN SELECT RAISE(ABORT, 'append-only: hosted DSH artifact facts are immutable'); END;
CREATE TABLE p13_hosted_dsh_attempt (
  attempt_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  employee_id TEXT NOT NULL REFERENCES p11_employee(employee_id),
  employee_revision_id TEXT NOT NULL,
  task_ref TEXT NOT NULL,
  child_id TEXT,
  artifact_digest TEXT NOT NULL CHECK (length(artifact_digest) = 40 OR length(artifact_digest) = 64),
  artifact_fact_id TEXT NOT NULL REFERENCES p13_hosted_dsh_artifact_fact(fact_id),
  context_digest TEXT NOT NULL CHECK (length(context_digest) = 64),
  context_bytes INTEGER NOT NULL CHECK (context_bytes > 0 AND context_bytes <= 65536),
  intent_persisted INTEGER NOT NULL CHECK (intent_persisted = 1),
  state TEXT NOT NULL CHECK (state IN ('persisted','dispatched','terminal','unknown-outcome')),
  pid INTEGER,
  terminal_kind TEXT NOT NULL CHECK (terminal_kind IN (
    'pending','exited','signaled','timed-out','spawn-failed','unknown-outcome'
  )),
  exit_code INTEGER,
  response_status TEXT NOT NULL CHECK (response_status IN ('pending','done','failed','blocked','unknown')),
  completion_claimed INTEGER NOT NULL CHECK (completion_claimed = 0),
  verification_status TEXT NOT NULL CHECK (verification_status = 'not-run'),
  candidate_count INTEGER NOT NULL DEFAULT 0,
  observation_count INTEGER NOT NULL DEFAULT 0,
  rejected_frame_count INTEGER NOT NULL DEFAULT 0,
  unknown_line_count INTEGER NOT NULL DEFAULT 0,
  stdout_bytes INTEGER NOT NULL DEFAULT 0,
  stdout_truncated INTEGER NOT NULL DEFAULT 0 CHECK (stdout_truncated IN (0,1)),
  stderr_tail_redacted TEXT NOT NULL DEFAULT '',
  elapsed_ms INTEGER,
  created_at INTEGER NOT NULL,
  dispatched_at INTEGER,
  terminal_at INTEGER
) STRICT;
CREATE INDEX p13_hosted_dsh_attempt_project
  ON p13_hosted_dsh_attempt(project_id, created_at);
CREATE INDEX p13_hosted_dsh_attempt_employee
  ON p13_hosted_dsh_attempt(employee_id, created_at);
CREATE TRIGGER p13_hosted_dsh_attempt_no_delete
BEFORE DELETE ON p13_hosted_dsh_attempt
BEGIN SELECT RAISE(ABORT, 'append-only: hosted DSH attempts are never deleted'); END;
CREATE TABLE p13_hosted_dsh_attempt_frame (
  frame_id TEXT PRIMARY KEY,
  attempt_id TEXT NOT NULL REFERENCES p13_hosted_dsh_attempt(attempt_id),
  seq INTEGER NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('observation','candidate','heartbeat','response','rejected')),
  operation TEXT,
  payload_digest TEXT,
  reject_reason TEXT,
  text_redacted TEXT NOT NULL,
  authority_written INTEGER NOT NULL CHECK (authority_written = 0),
  created_at INTEGER NOT NULL,
  UNIQUE(attempt_id, seq)
) STRICT;
CREATE INDEX p13_hosted_dsh_attempt_frame_attempt
  ON p13_hosted_dsh_attempt_frame(attempt_id, seq);
CREATE TRIGGER p13_hosted_dsh_attempt_frame_append_only_update
BEFORE UPDATE ON p13_hosted_dsh_attempt_frame
BEGIN SELECT RAISE(ABORT, 'append-only: hosted DSH attempt frames are immutable'); END;
CREATE TRIGGER p13_hosted_dsh_attempt_frame_append_only_delete
BEFORE DELETE ON p13_hosted_dsh_attempt_frame
BEGIN SELECT RAISE(ABORT, 'append-only: hosted DSH attempt frames are immutable'); END;
";

/// v36 migration entry.
pub fn hosted_dsh_attempt_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(36, HOSTED_DSH_ATTEMPT_SCHEMA_V36)
}

/// What the daemon observed about the configured artifact. Never spawns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedArtifactObservation {
    /// `revision` field of the daemon-owned `dsh.json`, if readable.
    pub configured_revision: Option<String>,
    /// Content of `<dsh_root>/.cognitiveos-dsh-revision`, if readable.
    pub pin_file_revision: Option<String>,
    /// `pinned` | `absent` | `corrupt` | `mismatch` | `script-missing`.
    pub health: String,
    /// SHA-256 of the hosted attempt child script, when present.
    pub child_script_digest: Option<String>,
    /// Redacted, bounded explanation.
    pub detail: String,
}

/// Durable artifact fact row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedArtifactFact {
    pub fact_id: String,
    pub kind: String,
    pub expected_revision: String,
    pub configured_revision: Option<String>,
    pub pin_file_revision: Option<String>,
    pub health: String,
    pub child_script_digest: Option<String>,
    pub previous_fact_id: Option<String>,
    pub detail_redacted: String,
    pub created_at: i64,
}

impl HostedArtifactFact {
    /// Only a `pinned` fact admits a spawn.
    pub fn admits_spawn(&self) -> bool {
        self.health == "pinned"
    }
}

/// Persist-before-dispatch input for one Attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostedAttemptIntentSpec<'a> {
    pub employee_id: &'a str,
    pub employee_revision_id: &'a str,
    pub task_ref: &'a str,
    pub bounded_context: &'a str,
    pub artifact_digest: &'a str,
    pub now_ms: i64,
}

/// Durable Attempt row. There is no `success` terminal kind by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedAttemptRow {
    pub attempt_id: String,
    pub project_id: String,
    pub employee_id: String,
    pub employee_revision_id: String,
    pub task_ref: String,
    pub child_id: Option<String>,
    pub artifact_digest: String,
    pub artifact_fact_id: String,
    pub context_digest: String,
    pub context_bytes: i64,
    pub intent_persisted: bool,
    pub state: String,
    pub pid: Option<u32>,
    pub terminal_kind: String,
    pub exit_code: Option<i64>,
    pub response_status: String,
    pub completion_claimed: bool,
    pub verification_status: String,
    pub candidate_count: i64,
    pub observation_count: i64,
    pub rejected_frame_count: i64,
    pub unknown_line_count: i64,
    pub stdout_bytes: i64,
    pub stdout_truncated: bool,
    pub stderr_tail_redacted: String,
    pub elapsed_ms: Option<i64>,
    pub created_at: i64,
    pub dispatched_at: Option<i64>,
    pub terminal_at: Option<i64>,
}

/// One frame the broker accepted or refused. Written as an observation only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedAttemptFrameSpec {
    pub seq: u64,
    /// `observation` | `candidate` | `heartbeat` | `response` | `rejected`.
    pub kind: String,
    pub operation: Option<String>,
    pub payload_digest: Option<String>,
    pub reject_reason: Option<String>,
    pub text_redacted: String,
}

/// Durable frame row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedAttemptFrameRow {
    pub frame_id: String,
    pub attempt_id: String,
    pub seq: i64,
    pub kind: String,
    pub operation: Option<String>,
    pub payload_digest: Option<String>,
    pub reject_reason: Option<String>,
    pub text_redacted: String,
    pub authority_written: bool,
}

/// Daemon-observed terminal facts for one child run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedAttemptTerminalSpec<'a> {
    /// `exited` | `signaled` | `timed-out` | `spawn-failed`.
    pub terminal_kind: &'a str,
    pub exit_code: Option<i32>,
    /// `done` | `failed` | `blocked` | `unknown` (absent response → `unknown`).
    pub response_status: Option<&'a str>,
    pub candidate_count: usize,
    pub observation_count: usize,
    pub rejected_frame_count: usize,
    pub unknown_line_count: usize,
    pub stdout_bytes: usize,
    pub stdout_truncated: bool,
    pub stderr_tail_redacted: &'a str,
    pub elapsed_ms: u64,
    pub now_ms: i64,
}

/// Hosted Attempt ledger over the daemon-owned writer.
#[derive(Clone)]
pub struct HostedDshAttemptStore {
    conn: Arc<Mutex<Connection>>,
    employees: EmployeeStore,
}

impl HostedDshAttemptStore {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            conn: Arc::clone(&store.conn),
            employees: EmployeeStore::from_authority_store(store),
        }
    }

    /// Open the authority database path (tests / CLI-free helpers).
    pub fn open_path(path: &std::path::Path) -> Result<Self, ProjectAggregateError> {
        let employees = EmployeeStore::open_path(path)?;
        Ok(Self {
            conn: employees.conn_arc(),
            employees,
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ProjectAggregateError> {
        self.conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })
    }

    pub(crate) fn conn_arc(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    /// SHA-256 hex of the bounded Context bytes (same function the broker uses).
    pub fn context_digest(bounded_context: &str) -> String {
        format!("{:x}", Sha256::digest(bounded_context.as_bytes()))
    }

    // ------------------------------------------------------------------
    // Artifact health / update / rollback facts
    // ------------------------------------------------------------------

    /// Record one daemon observation of the configured artifact.
    ///
    /// `kind` is derived, never caller-supplied: the first fact and any
    /// observation with the same configured revision as the latest fact is a
    /// `health-check`; a changed configured revision is an `update`; a change
    /// back to the revision recorded by the fact **before** the latest one is
    /// a `rollback`. Health is whatever the filesystem showed.
    pub fn record_artifact_observation(
        &self,
        caller: ConfirmCaller,
        observation: &HostedArtifactObservation,
        now_ms: i64,
    ) -> Result<HostedArtifactFact, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if !matches!(
            observation.health.as_str(),
            "pinned" | "absent" | "corrupt" | "mismatch" | "script-missing"
        ) {
            return Err(ProjectAggregateError::Invalid {
                detail: "artifact health class is unknown",
            });
        }
        if observation.health == "pinned"
            && (observation.configured_revision.as_deref() != Some(HOSTED_DSH_ARTIFACT_DIGEST)
                || observation.pin_file_revision.as_deref() != Some(HOSTED_DSH_ARTIFACT_DIGEST)
                || observation.child_script_digest.is_none())
        {
            return Err(ProjectAggregateError::Rejected {
                detail: "artifact cannot be pinned unless config, pin file and child script agree with the product pin",
            });
        }
        let detail = redact_bounded(&observation.detail, 512);
        let latest = self.latest_artifact_fact()?;
        let previous = match &latest {
            Some(latest) => self.artifact_fact_before(latest.created_at, &latest.fact_id)?,
            None => None,
        };
        let kind = match &latest {
            None => "health-check",
            Some(latest) if latest.configured_revision == observation.configured_revision => {
                "health-check"
            }
            Some(_) => match &previous {
                Some(previous)
                    if previous.configured_revision == observation.configured_revision =>
                {
                    "rollback"
                }
                _ => "update",
            },
        };
        let fact_id = format!("dshfact-{}", uuid::Uuid::now_v7().as_hyphenated());
        let previous_fact_id = latest.as_ref().map(|fact| fact.fact_id.clone());
        {
            let conn = self.lock()?;
            conn.execute(
                "INSERT INTO p13_hosted_dsh_artifact_fact (
                    fact_id, kind, expected_revision, configured_revision, pin_file_revision,
                    health, child_script_digest, previous_fact_id, detail_redacted, created_at
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![
                    fact_id,
                    kind,
                    HOSTED_DSH_ARTIFACT_DIGEST,
                    observation.configured_revision,
                    observation.pin_file_revision,
                    observation.health,
                    observation.child_script_digest,
                    previous_fact_id,
                    detail,
                    now_ms
                ],
            )
            .map_err(unavailable("insert artifact fact"))?;
        }
        self.get_artifact_fact(&fact_id)?
            .ok_or(ProjectAggregateError::Unavailable {
                detail: "artifact fact vanished after insert".to_owned(),
            })
    }

    /// Latest artifact fact, if any observation was ever recorded.
    pub fn latest_artifact_fact(
        &self,
    ) -> Result<Option<HostedArtifactFact>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT fact_id, kind, expected_revision, configured_revision, pin_file_revision,
                    health, child_script_digest, previous_fact_id, detail_redacted, created_at
               FROM p13_hosted_dsh_artifact_fact
              ORDER BY created_at DESC, rowid DESC LIMIT 1",
            [],
            map_fact_row,
        )
        .optional()
        .map_err(unavailable("latest artifact fact"))
    }

    fn artifact_fact_before(
        &self,
        created_at: i64,
        fact_id: &str,
    ) -> Result<Option<HostedArtifactFact>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT fact_id, kind, expected_revision, configured_revision, pin_file_revision,
                    health, child_script_digest, previous_fact_id, detail_redacted, created_at
               FROM p13_hosted_dsh_artifact_fact
              WHERE (created_at < ?1 OR (created_at = ?1 AND fact_id <> ?2))
                AND fact_id <> ?2
              ORDER BY created_at DESC, rowid DESC LIMIT 1",
            params![created_at, fact_id],
            map_fact_row,
        )
        .optional()
        .map_err(unavailable("previous artifact fact"))
    }

    /// One fact by id.
    pub fn get_artifact_fact(
        &self,
        fact_id: &str,
    ) -> Result<Option<HostedArtifactFact>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT fact_id, kind, expected_revision, configured_revision, pin_file_revision,
                    health, child_script_digest, previous_fact_id, detail_redacted, created_at
               FROM p13_hosted_dsh_artifact_fact WHERE fact_id = ?1",
            [fact_id],
            map_fact_row,
        )
        .optional()
        .map_err(unavailable("get artifact fact"))
    }

    /// Newest-first fact history, bounded.
    pub fn list_artifact_facts(
        &self,
        limit: i64,
    ) -> Result<Vec<HostedArtifactFact>, ProjectAggregateError> {
        let limit = limit.clamp(1, 64);
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT fact_id, kind, expected_revision, configured_revision, pin_file_revision,
                        health, child_script_digest, previous_fact_id, detail_redacted, created_at
                   FROM p13_hosted_dsh_artifact_fact
                  ORDER BY created_at DESC, rowid DESC LIMIT ?1",
            )
            .map_err(unavailable("prepare artifact facts"))?;
        let rows = statement
            .query_map([limit], map_fact_row)
            .map_err(unavailable("query artifact facts"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect artifact facts"))
    }

    // ------------------------------------------------------------------
    // Attempt: persist (Intent) → dispatched (Effect marker) → terminal
    // ------------------------------------------------------------------

    /// Persist the Attempt Intent **before** any spawn. Fails closed on an
    /// unseated Member, revision drift, an artifact that is not `pinned`,
    /// an empty / oversize / secret-shaped Context, or the GNU fence.
    pub fn persist_intent(
        &self,
        caller: ConfirmCaller,
        spec: &HostedAttemptIntentSpec<'_>,
    ) -> Result<HostedAttemptRow, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if HostedDshPlane::isolated_spawn_is_fenced() {
            return Err(ProjectAggregateError::Rejected {
                detail: HOSTED_DSH_WIN_GNU_FENCE,
            });
        }
        reject_pi_member_engine(spec.task_ref)?;
        reject_installed_agent_chrome(spec.task_ref)?;
        if spec.artifact_digest != HOSTED_DSH_ARTIFACT_DIGEST {
            return Err(ProjectAggregateError::Rejected {
                detail: "hosted DSH artifact digest mismatch",
            });
        }
        if !spec.task_ref.starts_with("task://") || spec.task_ref.len() < 8 {
            return Err(ProjectAggregateError::Invalid {
                detail: "task_ref must be a task:// reference",
            });
        }
        let context = spec.bounded_context;
        if context.trim().is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "bounded_context required",
            });
        }
        if context.len() > HOSTED_ATTEMPT_CONTEXT_MAX_BYTES {
            return Err(ProjectAggregateError::Invalid {
                detail: "bounded_context exceeds the 64 KiB ceiling",
            });
        }
        if secret_shaped_value(context) {
            return Err(ProjectAggregateError::Invalid {
                detail: "secret-shaped material must not enter the hosted child context",
            });
        }
        let artifact = self
            .latest_artifact_fact()?
            .ok_or(ProjectAggregateError::Rejected {
                detail: "hosted DSH artifact health is unknown; run artifact.check first",
            })?;
        if !artifact.admits_spawn() {
            return Err(ProjectAggregateError::Rejected {
                detail: "hosted DSH artifact is not pinned; spawn refused",
            });
        }
        let employee = self.employees.get_employee(spec.employee_id)?.ok_or(
            ProjectAggregateError::NotFound {
                detail: "employee not found",
            },
        )?;
        let Some(revision_id) = self.employees.latest_revision_id(spec.employee_id)? else {
            return Err(ProjectAggregateError::NotFound {
                detail: "employee revision not found",
            });
        };
        if revision_id != spec.employee_revision_id {
            return Err(ProjectAggregateError::Rejected {
                detail: "employee_revision_id mismatch",
            });
        }
        if employee.state != "seated" {
            return Err(ProjectAggregateError::Rejected {
                detail: "employee must be seated before a hosted Attempt",
            });
        }
        let attempt_id = format!("dshattempt-{}", uuid::Uuid::now_v7().as_hyphenated());
        let context_digest = Self::context_digest(context);
        let context_bytes = i64::try_from(context.len()).unwrap_or(i64::MAX);
        {
            let conn = self.lock()?;
            conn.execute(
                "INSERT INTO p13_hosted_dsh_attempt (
                    attempt_id, project_id, employee_id, employee_revision_id, task_ref,
                    child_id, artifact_digest, artifact_fact_id, context_digest, context_bytes,
                    intent_persisted, state, pid, terminal_kind, exit_code, response_status,
                    completion_claimed, verification_status, created_at
                 ) VALUES (?1,?2,?3,?4,?5,NULL,?6,?7,?8,?9,1,'persisted',NULL,'pending',NULL,'pending',0,'not-run',?10)",
                params![
                    attempt_id,
                    employee.project_id,
                    spec.employee_id,
                    spec.employee_revision_id,
                    spec.task_ref,
                    spec.artifact_digest,
                    artifact.fact_id,
                    context_digest,
                    context_bytes,
                    spec.now_ms
                ],
            )
            .map_err(unavailable("insert hosted attempt"))?;
        }
        self.require_attempt(&attempt_id)
    }

    /// Effect dispatch marker: the OS process exists. Requires `persisted`.
    pub fn mark_dispatched(
        &self,
        attempt_id: &str,
        child_id: Option<&str>,
        pid: u32,
        now_ms: i64,
    ) -> Result<HostedAttemptRow, ProjectAggregateError> {
        let updated = {
            let conn = self.lock()?;
            conn.execute(
                "UPDATE p13_hosted_dsh_attempt
                    SET state = 'dispatched', pid = ?1, child_id = COALESCE(?2, child_id),
                        dispatched_at = ?3
                  WHERE attempt_id = ?4 AND state = 'persisted'",
                params![i64::from(pid), child_id, now_ms, attempt_id],
            )
            .map_err(unavailable("mark attempt dispatched"))?
        };
        if updated == 0 {
            return Err(ProjectAggregateError::Conflict {
                detail: "attempt is not in the persisted state",
            });
        }
        self.require_attempt(attempt_id)
    }

    /// Bind the managed child identity (v31 row) to an Attempt before spawn.
    pub fn bind_child_identity(
        &self,
        attempt_id: &str,
        child_id: &str,
    ) -> Result<(), ProjectAggregateError> {
        let conn = self.lock()?;
        let updated = conn
            .execute(
                "UPDATE p13_hosted_dsh_attempt SET child_id = ?1
                  WHERE attempt_id = ?2 AND state IN ('persisted','dispatched')",
                params![child_id, attempt_id],
            )
            .map_err(unavailable("bind child identity"))?;
        if updated == 0 {
            return Err(ProjectAggregateError::Conflict {
                detail: "attempt cannot bind a child identity in its current state",
            });
        }
        Ok(())
    }

    /// Append broker frames as observations. Never touches Employee, Task,
    /// Effect, or Verification rows; bounded to the ledger frame ceiling.
    pub fn record_frames(
        &self,
        attempt_id: &str,
        frames: &[HostedAttemptFrameSpec],
        now_ms: i64,
    ) -> Result<usize, ProjectAggregateError> {
        let attempt = self.require_attempt(attempt_id)?;
        if attempt.state == "terminal" || attempt.state == "unknown-outcome" {
            return Err(ProjectAggregateError::Conflict {
                detail: "attempt already reached a terminal observation",
            });
        }
        let mut written = 0usize;
        let conn = self.lock()?;
        for frame in frames.iter().take(HOSTED_ATTEMPT_MAX_FRAMES) {
            if !matches!(
                frame.kind.as_str(),
                "observation" | "candidate" | "heartbeat" | "response" | "rejected"
            ) {
                return Err(ProjectAggregateError::Invalid {
                    detail: "frame kind is not an observation class",
                });
            }
            let frame_id = format!("dshframe-{}", uuid::Uuid::now_v7().as_hyphenated());
            let seq = i64::try_from(frame.seq).unwrap_or(i64::MAX);
            conn.execute(
                "INSERT INTO p13_hosted_dsh_attempt_frame (
                    frame_id, attempt_id, seq, kind, operation, payload_digest, reject_reason,
                    text_redacted, authority_written, created_at
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,0,?9)",
                params![
                    frame_id,
                    attempt_id,
                    seq,
                    frame.kind,
                    frame.operation,
                    frame.payload_digest,
                    frame.reject_reason,
                    redact_bounded(&frame.text_redacted, HOSTED_ATTEMPT_FRAME_TEXT_MAX_CHARS),
                    now_ms
                ],
            )
            .map_err(unavailable("insert attempt frame"))?;
            written += 1;
        }
        Ok(written)
    }

    /// Daemon-observed terminal write. `terminal_kind` is never `success`;
    /// `completion_claimed` stays `0`; `verification_status` stays `not-run`.
    pub fn record_terminal(
        &self,
        attempt_id: &str,
        terminal: &HostedAttemptTerminalSpec<'_>,
    ) -> Result<HostedAttemptRow, ProjectAggregateError> {
        if !matches!(
            terminal.terminal_kind,
            "exited" | "signaled" | "timed-out" | "spawn-failed"
        ) {
            return Err(ProjectAggregateError::Invalid {
                detail: "terminal kind must be exited, signaled, timed-out, or spawn-failed",
            });
        }
        let response_status = match terminal.response_status {
            Some("done") => "done",
            Some("failed") => "failed",
            Some("blocked") => "blocked",
            _ => "unknown",
        };
        let updated = {
            let conn = self.lock()?;
            conn.execute(
                "UPDATE p13_hosted_dsh_attempt
                    SET state = 'terminal', terminal_kind = ?1, exit_code = ?2,
                        response_status = ?3, candidate_count = ?4, observation_count = ?5,
                        rejected_frame_count = ?6, unknown_line_count = ?7, stdout_bytes = ?8,
                        stdout_truncated = ?9, stderr_tail_redacted = ?10, elapsed_ms = ?11,
                        terminal_at = ?12, pid = NULL
                  WHERE attempt_id = ?13 AND state IN ('persisted','dispatched')",
                params![
                    terminal.terminal_kind,
                    terminal.exit_code.map(i64::from),
                    response_status,
                    i64::try_from(terminal.candidate_count).unwrap_or(i64::MAX),
                    i64::try_from(terminal.observation_count).unwrap_or(i64::MAX),
                    i64::try_from(terminal.rejected_frame_count).unwrap_or(i64::MAX),
                    i64::try_from(terminal.unknown_line_count).unwrap_or(i64::MAX),
                    i64::try_from(terminal.stdout_bytes).unwrap_or(i64::MAX),
                    i64::from(terminal.stdout_truncated),
                    redact_bounded(terminal.stderr_tail_redacted, 2048),
                    i64::try_from(terminal.elapsed_ms).unwrap_or(i64::MAX),
                    terminal.now_ms,
                    attempt_id
                ],
            )
            .map_err(unavailable("record attempt terminal"))?
        };
        if updated == 0 {
            return Err(ProjectAggregateError::Conflict {
                detail: "attempt already has a terminal observation",
            });
        }
        self.require_attempt(attempt_id)
    }

    /// Startup / crash reconcile: rows the daemon left non-terminal become
    /// `unknown-outcome` (A3). Returns the reconciled attempt ids.
    pub fn reconcile_unknown_outcomes(
        &self,
        now_ms: i64,
    ) -> Result<Vec<String>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT attempt_id FROM p13_hosted_dsh_attempt
                  WHERE state IN ('persisted','dispatched') ORDER BY created_at",
            )
            .map_err(unavailable("prepare unknown outcomes"))?;
        let ids = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(unavailable("query unknown outcomes"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect unknown outcomes"))?;
        drop(statement);
        conn.execute(
            "UPDATE p13_hosted_dsh_attempt
                SET state = 'unknown-outcome', terminal_kind = 'unknown-outcome',
                    response_status = 'unknown', pid = NULL, terminal_at = ?1
              WHERE state IN ('persisted','dispatched')",
            [now_ms],
        )
        .map_err(unavailable("reconcile unknown outcomes"))?;
        Ok(ids)
    }

    /// One Attempt by id.
    pub fn get_attempt(
        &self,
        attempt_id: &str,
    ) -> Result<Option<HostedAttemptRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            &format!("{ATTEMPT_SELECT} WHERE attempt_id = ?1"),
            [attempt_id],
            map_attempt_row,
        )
        .optional()
        .map_err(unavailable("get attempt"))
    }

    fn require_attempt(&self, attempt_id: &str) -> Result<HostedAttemptRow, ProjectAggregateError> {
        self.get_attempt(attempt_id)?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "attempt not found",
            })
    }

    /// Newest-first Attempt history for one Project (the `runs` data source).
    pub fn list_attempts(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<HostedAttemptRow>, ProjectAggregateError> {
        let limit = limit.clamp(1, 64);
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(&format!(
                "{ATTEMPT_SELECT} WHERE project_id = ?1 ORDER BY created_at DESC, rowid DESC LIMIT ?2"
            ))
            .map_err(unavailable("prepare attempts"))?;
        let rows = statement
            .query_map(params![project_id, limit], map_attempt_row)
            .map_err(unavailable("query attempts"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect attempts"))
    }

    /// Frames of one Attempt in sequence order, bounded.
    pub fn list_frames(
        &self,
        attempt_id: &str,
        limit: i64,
    ) -> Result<Vec<HostedAttemptFrameRow>, ProjectAggregateError> {
        let limit = limit.clamp(1, 512);
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT frame_id, attempt_id, seq, kind, operation, payload_digest, reject_reason,
                        text_redacted, authority_written
                   FROM p13_hosted_dsh_attempt_frame
                  WHERE attempt_id = ?1 ORDER BY seq ASC LIMIT ?2",
            )
            .map_err(unavailable("prepare frames"))?;
        let rows = statement
            .query_map(params![attempt_id, limit], |row| {
                let authority_written: i64 = row.get(8)?;
                Ok(HostedAttemptFrameRow {
                    frame_id: row.get(0)?,
                    attempt_id: row.get(1)?,
                    seq: row.get(2)?,
                    kind: row.get(3)?,
                    operation: row.get(4)?,
                    payload_digest: row.get(5)?,
                    reject_reason: row.get(6)?,
                    text_redacted: row.get(7)?,
                    authority_written: authority_written != 0,
                })
            })
            .map_err(unavailable("query frames"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect frames"))
    }
}

const ATTEMPT_SELECT: &str =
    "SELECT attempt_id, project_id, employee_id, employee_revision_id, task_ref, child_id,
        artifact_digest, artifact_fact_id, context_digest, context_bytes, intent_persisted, state,
        pid, terminal_kind, exit_code, response_status, completion_claimed, verification_status,
        candidate_count, observation_count, rejected_frame_count, unknown_line_count,
        stdout_bytes, stdout_truncated, stderr_tail_redacted, elapsed_ms, created_at,
        dispatched_at, terminal_at
   FROM p13_hosted_dsh_attempt";

fn map_attempt_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<HostedAttemptRow> {
    let pid: Option<i64> = row.get(12)?;
    let intent_persisted: i64 = row.get(10)?;
    let completion_claimed: i64 = row.get(16)?;
    let stdout_truncated: i64 = row.get(23)?;
    Ok(HostedAttemptRow {
        attempt_id: row.get(0)?,
        project_id: row.get(1)?,
        employee_id: row.get(2)?,
        employee_revision_id: row.get(3)?,
        task_ref: row.get(4)?,
        child_id: row.get(5)?,
        artifact_digest: row.get(6)?,
        artifact_fact_id: row.get(7)?,
        context_digest: row.get(8)?,
        context_bytes: row.get(9)?,
        intent_persisted: intent_persisted != 0,
        state: row.get(11)?,
        pid: pid.and_then(|value| u32::try_from(value).ok()),
        terminal_kind: row.get(13)?,
        exit_code: row.get(14)?,
        response_status: row.get(15)?,
        completion_claimed: completion_claimed != 0,
        verification_status: row.get(17)?,
        candidate_count: row.get(18)?,
        observation_count: row.get(19)?,
        rejected_frame_count: row.get(20)?,
        unknown_line_count: row.get(21)?,
        stdout_bytes: row.get(22)?,
        stdout_truncated: stdout_truncated != 0,
        stderr_tail_redacted: row.get(24)?,
        elapsed_ms: row.get(25)?,
        created_at: row.get(26)?,
        dispatched_at: row.get(27)?,
        terminal_at: row.get(28)?,
    })
}

fn map_fact_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<HostedArtifactFact> {
    Ok(HostedArtifactFact {
        fact_id: row.get(0)?,
        kind: row.get(1)?,
        expected_revision: row.get(2)?,
        configured_revision: row.get(3)?,
        pin_file_revision: row.get(4)?,
        health: row.get(5)?,
        child_script_digest: row.get(6)?,
        previous_fact_id: row.get(7)?,
        detail_redacted: row.get(8)?,
        created_at: row.get(9)?,
    })
}

/// Redact secret-shaped tokens and bound the text before it becomes durable.
pub fn redact_bounded(text: &str, max_chars: usize) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while !rest.is_empty() {
        let lowered = rest.to_ascii_lowercase();
        let next = ["sk-", "bearer ", "ssv1:", "secretref:"]
            .iter()
            .filter_map(|marker| lowered.find(marker).map(|index| (index, marker.len())))
            .min_by_key(|(index, _)| *index);
        let Some((index, marker_len)) = next else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..index]);
        output.push_str(&rest[index..index + marker_len]);
        output.push_str("[redacted]");
        let after = &rest[index + marker_len..];
        let token_end = after
            .find(|character: char| {
                character.is_whitespace() || character == '"' || character == '\''
            })
            .unwrap_or(after.len());
        rest = &after[token_end..];
    }
    output.chars().take(max_chars).collect()
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}
