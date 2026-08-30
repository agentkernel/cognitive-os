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

/// Hard ceiling for one archive-index page. Omitting a limit or exceeding this
/// is unbounded resume (T05-N4).
pub const CONVERSATION_RESUME_LIMIT: u32 = 32;
/// Single-record body ceiling. A dump that exceeds this is full-archive
/// injection (T05-N5), not a stored transcript.
pub const CONVERSATION_BODY_LIMIT: usize = 4096;

const ARCHIVE_APPEND_KINDS: &[&str] = &[
    "note",
    "deliverable",
    "handoff",
    "blocked",
    "decision-request",
];

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

/// Bounded authorized index/query (Clippy-safe argument bundle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchiveReadSpec<'a> {
    pub projection_id: &'a str,
    pub caller_project_id: &'a str,
    pub target_project_id: &'a str,
    pub employee_id: Option<&'a str>,
    pub limit: u32,
    pub resume_from: Option<&'a str>,
    pub include_bodies: bool,
}

/// Input for owner-management append that does not go through speech routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchiveAppendSpec<'a> {
    pub projection_id: &'a str,
    pub project_id: &'a str,
    pub employee_id: &'a str,
    pub kind: &'a str,
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

/// Index/query reference. Does not embed transcript bytes (T05-N5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationArchiveRef {
    pub record_id: String,
    pub projection_id: String,
    pub project_id: String,
    pub employee_id: String,
    pub kind: String,
    pub body_digest: String,
    pub created_at: i64,
}

/// One-record fetch. Observation-only: not Task or Project completion (T05-N6).
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

/// One page of archive references. `next_cursor` is a record_id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationIndexPage {
    pub records: Vec<ConversationArchiveRef>,
    pub truncated: bool,
    pub next_cursor: Option<String>,
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
        reject_oversize_body(spec.body)?;
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

    /// Owner-management append. Chatter is refused. Does not complete a
    /// Task or Project. Speech-routed landing stays on `land_speech`.
    pub fn append(
        &self,
        caller: crate::project_aggregate::ConfirmCaller,
        spec: &ArchiveAppendSpec<'_>,
    ) -> Result<String, ProjectAggregateError> {
        require_owner(caller)?;
        require_archive_projection(spec.projection_id)?;
        reject_secret_shape(spec.body)?;
        reject_oversize_body(spec.body)?;
        if spec.kind == "chatter" || !ARCHIVE_APPEND_KINDS.contains(&spec.kind) {
            return Err(ProjectAggregateError::Invalid {
                detail: "chatter is not an archive record",
            });
        }
        self.require_employee_in_project(spec.project_id, spec.employee_id)?;
        self.insert_archive(
            spec.project_id,
            spec.employee_id,
            None,
            spec.kind,
            spec.body,
            spec.now_ms,
        )
    }

    /// Bounded authorized index. Returns record references, never bodies.
    /// Missing/zero/oversize `limit` is unbounded resume (T05-N4).
    /// `include_bodies` is full-archive injection (T05-N5).
    pub fn read_index(
        &self,
        spec: &ArchiveReadSpec<'_>,
    ) -> Result<ConversationIndexPage, ProjectAggregateError> {
        require_archive_projection(spec.projection_id)?;
        if spec.include_bodies {
            return Err(ProjectAggregateError::Invalid {
                detail: "full-archive injection rejected",
            });
        }
        if spec.limit == 0 || spec.limit > CONVERSATION_RESUME_LIMIT {
            return Err(ProjectAggregateError::Invalid {
                detail: "unbounded conversation resume rejected",
            });
        }
        if spec.caller_project_id != spec.target_project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-scope conversation read rejected",
            });
        }
        let conn = self.lock()?;
        if let Some(eid) = spec.employee_id {
            require_employee_scope(&conn, spec.caller_project_id, eid)?;
        }
        let cursor = if let Some(record_id) = spec.resume_from {
            Some(load_cursor(&conn, spec.caller_project_id, record_id)?)
        } else {
            None
        };
        let fetch = (spec.limit as usize).saturating_add(1);
        let mut rows = query_index_refs(
            &conn,
            spec.caller_project_id,
            spec.employee_id,
            cursor.as_ref(),
            fetch,
        )?;
        let truncated = rows.len() > spec.limit as usize;
        if truncated {
            rows.truncate(spec.limit as usize);
        }
        let next_cursor = if truncated {
            rows.last().map(|row| row.record_id.clone())
        } else {
            None
        };
        Ok(ConversationIndexPage {
            records: rows,
            truncated,
            next_cursor,
        })
    }

    /// Single-record body fetch inside the caller project. Not a bulk dump.
    pub fn read_record(
        &self,
        projection_id: &str,
        caller_project_id: &str,
        record_id: &str,
    ) -> Result<ConversationArchiveRecord, ProjectAggregateError> {
        require_archive_projection(projection_id)?;
        let conn = self.lock()?;
        let row = conn
            .query_row(
                "SELECT record_id, projection_id, project_id, employee_id, speech_audit_id,
                        kind, body_digest, body_redacted, created_at
                   FROM p11_conversation_archive
                  WHERE record_id = ?1",
                [record_id],
                map_record,
            )
            .optional()
            .map_err(unavailable("read record"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "conversation record not found",
            })?;
        if row.project_id != caller_project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-scope conversation read rejected",
            });
        }
        Ok(row)
    }

    fn require_employee_in_project(
        &self,
        project_id: &str,
        employee_id: &str,
    ) -> Result<(), ProjectAggregateError> {
        let conn = self.lock()?;
        require_employee_scope(&conn, project_id, employee_id)
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

fn require_owner(
    caller: crate::project_aggregate::ConfirmCaller,
) -> Result<(), ProjectAggregateError> {
    match caller {
        crate::project_aggregate::ConfirmCaller::OwnerManagement => Ok(()),
        crate::project_aggregate::ConfirmCaller::TaskChannel
        | crate::project_aggregate::ConfirmCaller::Assistant => {
            Err(ProjectAggregateError::Forbidden {
                detail: "only owner management session may confirm or apply",
            })
        }
    }
}

fn reject_oversize_body(body: &str) -> Result<(), ProjectAggregateError> {
    if body.len() > CONVERSATION_BODY_LIMIT {
        return Err(ProjectAggregateError::Invalid {
            detail: "full-archive injection rejected",
        });
    }
    Ok(())
}

struct ArchiveCursor {
    created_at: i64,
    record_id: String,
}

fn require_employee_scope(
    conn: &Connection,
    caller_project_id: &str,
    employee_id: &str,
) -> Result<(), ProjectAggregateError> {
    let owner: Option<String> = conn
        .query_row(
            "SELECT project_id FROM p11_employee WHERE employee_id = ?1",
            [employee_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(unavailable("employee scope"))?;
    match owner {
        None => Err(ProjectAggregateError::NotFound {
            detail: "employee not found",
        }),
        Some(project) if project != caller_project_id => Err(ProjectAggregateError::Forbidden {
            detail: "cross-scope conversation read rejected",
        }),
        Some(_) => Ok(()),
    }
}

fn load_cursor(
    conn: &Connection,
    caller_project_id: &str,
    record_id: &str,
) -> Result<ArchiveCursor, ProjectAggregateError> {
    let row: Option<(i64, String, String)> = conn
        .query_row(
            "SELECT created_at, record_id, project_id
               FROM p11_conversation_archive
              WHERE record_id = ?1",
            [record_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(unavailable("resume cursor"))?;
    match row {
        None => Err(ProjectAggregateError::NotFound {
            detail: "resume cursor not found",
        }),
        Some((_, _, project)) if project != caller_project_id => {
            Err(ProjectAggregateError::Forbidden {
                detail: "cross-scope conversation read rejected",
            })
        }
        Some((created_at, id, _)) => Ok(ArchiveCursor {
            created_at,
            record_id: id,
        }),
    }
}

fn query_index_refs(
    conn: &Connection,
    project_id: &str,
    employee_id: Option<&str>,
    cursor: Option<&ArchiveCursor>,
    fetch: usize,
) -> Result<Vec<ConversationArchiveRef>, ProjectAggregateError> {
    let mapped = |row: &rusqlite::Row<'_>| -> rusqlite::Result<ConversationArchiveRef> {
        Ok(ConversationArchiveRef {
            record_id: row.get(0)?,
            projection_id: row.get(1)?,
            project_id: row.get(2)?,
            employee_id: row.get(3)?,
            kind: row.get(4)?,
            body_digest: row.get(5)?,
            created_at: row.get(6)?,
        })
    };
    let select = "SELECT record_id, projection_id, project_id, employee_id,
                         kind, body_digest, created_at
                    FROM p11_conversation_archive";
    let order = "ORDER BY created_at ASC, record_id ASC LIMIT ?1";
    match (employee_id, cursor) {
        (None, None) => {
            let mut statement = conn
                .prepare(&format!("{select} WHERE project_id = ?2 {order}"))
                .map_err(unavailable("prepare index"))?;
            statement
                .query_map(params![fetch as i64, project_id], mapped)
                .map_err(unavailable("query index"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(unavailable("collect index"))
        }
        (Some(eid), None) => {
            let mut statement = conn
                .prepare(&format!(
                    "{select} WHERE project_id = ?2 AND employee_id = ?3 {order}"
                ))
                .map_err(unavailable("prepare index"))?;
            statement
                .query_map(params![fetch as i64, project_id, eid], mapped)
                .map_err(unavailable("query index"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(unavailable("collect index"))
        }
        (None, Some(cur)) => {
            let mut statement = conn
                .prepare(&format!(
                    "{select}
                      WHERE project_id = ?2
                        AND (created_at > ?3 OR (created_at = ?3 AND record_id > ?4))
                      {order}"
                ))
                .map_err(unavailable("prepare index"))?;
            statement
                .query_map(
                    params![fetch as i64, project_id, cur.created_at, cur.record_id],
                    mapped,
                )
                .map_err(unavailable("query index"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(unavailable("collect index"))
        }
        (Some(eid), Some(cur)) => {
            let mut statement = conn
                .prepare(&format!(
                    "{select}
                      WHERE project_id = ?2 AND employee_id = ?3
                        AND (created_at > ?4 OR (created_at = ?4 AND record_id > ?5))
                      {order}"
                ))
                .map_err(unavailable("prepare index"))?;
            statement
                .query_map(
                    params![fetch as i64, project_id, eid, cur.created_at, cur.record_id],
                    mapped,
                )
                .map_err(unavailable("query index"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(unavailable("collect index"))
        }
    }
}

fn map_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConversationArchiveRecord> {
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
