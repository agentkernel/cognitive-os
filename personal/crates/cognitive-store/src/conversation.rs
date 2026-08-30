//! Personal-private conversation archive (P11-T05, authority migration v28).
//!
//! New identifier `cognitiveos.personal.conversation-archive/0.1`. ADR-0058
//! `cognitiveos.personal.conversation-projection/0.1` is retained and is
//! never coerced onto this archive. Observation-only: an archive row is not
//! Task or Project completion.

use crate::employee::EmployeeStore;
use crate::project_aggregate::ProjectAggregateError;
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

/// New Personal-private OPC conversation archive envelope (P11-T05).
pub const CONVERSATION_ARCHIVE_PROJECTION_ID: &str =
    "cognitiveos.personal.conversation-archive/0.1";
/// ADR-0058 identifier. Clients presenting this id are refused, not remapped.
pub const LEGACY_CONVERSATION_PROJECTION_ID: &str =
    "cognitiveos.personal.conversation-projection/0.1";

/// Authority migration v28: scoped conversation archive rows.
pub const CONVERSATION_ARCHIVE_SCHEMA_V28: &str = "
CREATE TABLE p11_conversation_archive (
  record_id TEXT PRIMARY KEY,
  projection_id TEXT NOT NULL CHECK (projection_id = 'cognitiveos.personal.conversation-archive/0.1'),
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  employee_id TEXT NOT NULL REFERENCES p11_employee(employee_id),
  speech_audit_id TEXT,
  kind TEXT NOT NULL,
  body_digest TEXT NOT NULL CHECK (length(body_digest) = 64),
  body_redacted TEXT NOT NULL,
  created_at INTEGER NOT NULL
) STRICT;
CREATE INDEX p11_conversation_archive_scope
  ON p11_conversation_archive(project_id, employee_id, created_at);
";

/// v28 migration entry.
pub fn conversation_migration_entry() -> crate::migration::MigrationPlanEntry {
    crate::migration::MigrationPlanEntry::new(28, CONVERSATION_ARCHIVE_SCHEMA_V28)
}

/// Input for speech → archive landing (Clippy-safe argument bundle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeechArchiveSpec<'a> {
    pub projection_id: &'a str,
    pub project_id: &'a str,
    pub employee_id: &'a str,
    pub kind: &'a str,
    pub mentioned: bool,
    pub body: &'a str,
    pub now_ms: i64,
}

/// Outcome of `land_speech`: audit always; archive row only when delivered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeechArchiveOutcome {
    pub audit_id: String,
    pub delivered: bool,
    pub reason: String,
    pub record_id: Option<String>,
}

/// Durable archive row. Not a completion fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationArchiveRecord {
    pub record_id: String,
    pub projection_id: String,
    pub project_id: String,
    pub employee_id: String,
    pub speech_audit_id: Option<String>,
    pub kind: String,
    pub body_digest: String,
    pub body_redacted: String,
    pub created_at: i64,
}

/// Personal-private conversation archive on the authority writer.
#[derive(Clone)]
pub struct ConversationStore {
    conn: Arc<Mutex<Connection>>,
}

impl ConversationStore {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            conn: Arc::clone(&store.conn),
        }
    }

    /// Open the authority database path (tests / CLI-free helpers).
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

    /// Route member speech, then persist an archive row only when delivered.
    /// Chatter stays audit-only. Secret-shaped bodies and legacy projection
    /// identifiers fail closed before any archive insert.
    pub fn land_speech(
        &self,
        employees: &EmployeeStore,
        spec: &SpeechArchiveSpec<'_>,
    ) -> Result<SpeechArchiveOutcome, ProjectAggregateError> {
        require_archive_projection(spec.projection_id)?;
        reject_secret_shape(spec.body)?;
        let decision = employees.route_speech(
            spec.project_id,
            spec.employee_id,
            spec.kind,
            spec.mentioned,
            spec.now_ms,
        )?;
        if !decision.delivered {
            return Ok(SpeechArchiveOutcome {
                audit_id: decision.audit_id,
                delivered: false,
                reason: decision.reason,
                record_id: None,
            });
        }
        let record_id = self.insert_archive(
            spec.project_id,
            spec.employee_id,
            Some(&decision.audit_id),
            spec.kind,
            spec.body,
            spec.now_ms,
        )?;
        Ok(SpeechArchiveOutcome {
            audit_id: decision.audit_id,
            delivered: true,
            reason: decision.reason,
            record_id: Some(record_id),
        })
    }

    /// Authorized scoped read. `caller_project_id` must equal `target_project_id`.
    /// Legacy projection ids fail closed. Cross-project employee ids are forbidden.
    pub fn read_scoped(
        &self,
        projection_id: &str,
        caller_project_id: &str,
        target_project_id: &str,
        employee_id: Option<&str>,
    ) -> Result<Vec<ConversationArchiveRecord>, ProjectAggregateError> {
        require_archive_projection(projection_id)?;
        if caller_project_id != target_project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-scope conversation read rejected",
            });
        }
        let conn = self.lock()?;
        if let Some(eid) = employee_id {
            let owner: Option<String> = conn
                .query_row(
                    "SELECT project_id FROM p11_employee WHERE employee_id = ?1",
                    [eid],
                    |row| row.get(0),
                )
                .optional()
                .map_err(unavailable("employee scope"))?;
            match owner {
                None => {
                    return Err(ProjectAggregateError::NotFound {
                        detail: "employee not found",
                    });
                }
                Some(project) if project != caller_project_id => {
                    return Err(ProjectAggregateError::Forbidden {
                        detail: "cross-scope conversation read rejected",
                    });
                }
                Some(_) => {}
            }
        }
        let sql = if employee_id.is_some() {
            "SELECT record_id, projection_id, project_id, employee_id, speech_audit_id,
                    kind, body_digest, body_redacted, created_at
               FROM p11_conversation_archive
              WHERE project_id = ?1 AND employee_id = ?2
              ORDER BY created_at ASC, record_id ASC"
        } else {
            "SELECT record_id, projection_id, project_id, employee_id, speech_audit_id,
                    kind, body_digest, body_redacted, created_at
               FROM p11_conversation_archive
              WHERE project_id = ?1
              ORDER BY created_at ASC, record_id ASC"
        };
        let mut statement = conn.prepare(sql).map_err(unavailable("prepare archive"))?;
        let mapped = |row: &rusqlite::Row<'_>| -> rusqlite::Result<ConversationArchiveRecord> {
            Ok(ConversationArchiveRecord {
                record_id: row.get(0)?,
                projection_id: row.get(1)?,
                project_id: row.get(2)?,
                employee_id: row.get(3)?,
                speech_audit_id: row.get(4)?,
                kind: row.get(5)?,
                body_digest: row.get(6)?,
                body_redacted: row.get(7)?,
                created_at: row.get(8)?,
            })
        };
        let rows = if let Some(eid) = employee_id {
            statement
                .query_map(params![caller_project_id, eid], mapped)
                .map_err(unavailable("query archive"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(unavailable("collect archive"))?
        } else {
            statement
                .query_map(params![caller_project_id], mapped)
                .map_err(unavailable("query archive"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(unavailable("collect archive"))?
        };
        Ok(rows)
    }

    fn insert_archive(
        &self,
        project_id: &str,
        employee_id: &str,
        speech_audit_id: Option<&str>,
        kind: &str,
        body: &str,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        let record_id = next_id("conv")?;
        let body_digest = digest_hex(body.as_bytes());
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO p11_conversation_archive (
                record_id, projection_id, project_id, employee_id, speech_audit_id,
                kind, body_digest, body_redacted, created_at
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                record_id,
                CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id,
                employee_id,
                speech_audit_id,
                kind,
                body_digest,
                body,
                now_ms
            ],
        )
        .map_err(unavailable("insert archive"))?;
        Ok(record_id)
    }
}

fn require_archive_projection(projection_id: &str) -> Result<(), ProjectAggregateError> {
    if projection_id == CONVERSATION_ARCHIVE_PROJECTION_ID {
        return Ok(());
    }
    Err(ProjectAggregateError::Invalid {
        detail: "legacy conversation-projection identifier is not coerced",
    })
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
