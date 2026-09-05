//! Personal-private Project aggregate (P11-T03, authority migration v26).
//!
//! New tables, not `family=task`. Circular FKs between `p11_project` and
//! `p11_charter_revision` are omitted (SQLite cannot insert both rows live);
//! the daemon writer maintains that integrity. `inactive` is a Project state
//! because 14 §3.3 / N15 copy lands inactive (21 §2 omitted it).

use crate::migration::MigrationPlanEntry;
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

#[path = "project_lifecycle.rs"]
mod project_lifecycle;
pub use project_lifecycle::{
    DeletePreviewView, LifecycleArchiveSpec, LifecycleCopySpec, LifecycleDeleteConfirmSpec,
    LifecycleDeletePreviewSpec, LifecycleEventView, LifecycleExportSpec, LifecycleRestoreSpec,
    ProjectExportView, ProjectLifecycleStore, ProjectLifecycleView, RestorePointView,
    project_lifecycle_migration_entry,
};

/// Authority migration v26: Personal-private Project aggregate tables.
pub const PROJECT_AGGREGATE_SCHEMA_V26: &str = "
CREATE TABLE p11_draft (
  draft_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK (kind IN ('project-create')),
  base_seq INTEGER NOT NULL CHECK (base_seq >= 0),
  payload_digest TEXT NOT NULL CHECK (length(payload_digest) = 64),
  state TEXT NOT NULL CHECK (state IN ('open','activated','abandoned')),
  activated_project_id TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
) STRICT;
CREATE TABLE p11_candidate (
  candidate_id TEXT PRIMARY KEY,
  draft_id TEXT NOT NULL REFERENCES p11_draft(draft_id),
  base_seq INTEGER NOT NULL,
  ops_digest TEXT NOT NULL CHECK (length(ops_digest) = 64),
  author TEXT NOT NULL CHECK (author IN ('owner','assistant')),
  sources_json TEXT,
  applied_at INTEGER,
  superseded_at INTEGER,
  CHECK ((author = 'assistant') = (sources_json IS NOT NULL))
) STRICT;
CREATE TABLE p11_charter_revision (
  charter_revision_id TEXT PRIMARY KEY,
  project_id TEXT,
  draft_id TEXT NOT NULL REFERENCES p11_draft(draft_id),
  seq INTEGER NOT NULL,
  content_digest TEXT NOT NULL CHECK (length(content_digest) = 64),
  status TEXT NOT NULL CHECK (status IN ('draft','confirmed','superseded')),
  confirmed_at INTEGER,
  source_intent_id TEXT,
  UNIQUE(draft_id, seq)
) STRICT;
CREATE TABLE p11_project (
  project_id TEXT PRIMARY KEY,
  state TEXT NOT NULL CHECK (state IN (
    'creating','active','attention','paused','archived','restore-ready','deletion-preview','inactive'
  )),
  current_charter_revision_id TEXT NOT NULL,
  current_plan_revision_id TEXT,
  created_at INTEGER NOT NULL,
  activated_at INTEGER NOT NULL,
  accepted_at INTEGER,
  CHECK (
    (state = 'creating' AND accepted_at IS NULL)
    OR (state = 'active' AND accepted_at IS NOT NULL)
    OR (state NOT IN ('creating','active'))
  )
) STRICT;
CREATE TABLE p11_plan_revision (
  plan_revision_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  seq INTEGER NOT NULL,
  axis_digest TEXT NOT NULL CHECK (length(axis_digest) = 64),
  created_at INTEGER NOT NULL,
  UNIQUE(project_id, seq)
) STRICT;
CREATE TABLE p11_stage (
  plan_revision_id TEXT NOT NULL REFERENCES p11_plan_revision(plan_revision_id),
  stage_id TEXT NOT NULL,
  position INTEGER NOT NULL,
  title TEXT NOT NULL,
  objective TEXT NOT NULL,
  input_seam_digest TEXT,
  output_contract_digest TEXT NOT NULL,
  acceptance_spec_ref TEXT,
  cadence_json TEXT,
  responsible_slot TEXT NOT NULL,
  confirm_status TEXT NOT NULL CHECK (confirm_status IN ('unconfirmed','confirmed')),
  stage_digest TEXT NOT NULL CHECK (length(stage_digest) = 64),
  ready INTEGER NOT NULL CHECK (ready IN (0,1)),
  PRIMARY KEY (plan_revision_id, stage_id),
  UNIQUE (plan_revision_id, position)
) STRICT;
CREATE TABLE p11_gap (
  gap_id TEXT PRIMARY KEY,
  plan_revision_id TEXT NOT NULL,
  stage_id TEXT NOT NULL,
  description TEXT NOT NULL,
  blocking INTEGER NOT NULL CHECK (blocking IN (0,1)),
  resolved_by_revision_id TEXT,
  accepted_as_limitation_at INTEGER,
  FOREIGN KEY (plan_revision_id, stage_id) REFERENCES p11_stage(plan_revision_id, stage_id)
) STRICT;
CREATE TABLE p11_stage_test_fact (
  fact_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  plan_revision_id TEXT NOT NULL,
  stage_id TEXT NOT NULL,
  task_ref TEXT NOT NULL,
  verification_report_ref TEXT NOT NULL,
  passed_at INTEGER NOT NULL,
  current INTEGER NOT NULL CHECK (current IN (0,1))
) STRICT;
CREATE UNIQUE INDEX p11_stage_test_current
  ON p11_stage_test_fact(plan_revision_id, stage_id) WHERE current = 1;
CREATE TABLE p11_acceptance_fact (
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  plan_revision_id TEXT NOT NULL,
  acceptance_decision_ref TEXT NOT NULL,
  accepted_at INTEGER NOT NULL,
  PRIMARY KEY (project_id, plan_revision_id)
) STRICT;
CREATE TABLE p11_approval_preview (
  preview_id TEXT PRIMARY KEY,
  subject_kind TEXT NOT NULL CHECK (subject_kind IN ('activation','plan-change','acceptance')),
  subject_ref TEXT NOT NULL,
  base_state_digest TEXT NOT NULL CHECK (length(base_state_digest) = 64),
  preview_bytes_ref TEXT NOT NULL,
  preview_digest TEXT NOT NULL UNIQUE CHECK (length(preview_digest) = 64),
  status TEXT NOT NULL CHECK (status IN (
    'pending','approved','rejected','stale','consumed','superseded'
  )),
  intent_id TEXT,
  receipt_ref TEXT,
  created_at INTEGER NOT NULL,
  decided_at INTEGER
) STRICT;
";

/// v26 migration entry.
pub fn project_aggregate_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(26, PROJECT_AGGREGATE_SCHEMA_V26)
}

/// Authority migration v29: durable HITL narrow/reject (P11-T09).
/// `superseded_by` records the replacement preview id when the owner narrows.
/// Status `superseded` already exists in v26; this column is the mechanical
/// `narrowed(superseded_by)` link. StandingApprovalPolicy time-box is not
/// this migration.
pub const APPROVAL_PREVIEW_NARROW_SCHEMA_V29: &str = "
ALTER TABLE p11_approval_preview ADD COLUMN superseded_by TEXT;
";

/// v29 migration entry.
pub fn approval_preview_narrow_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(29, APPROVAL_PREVIEW_NARROW_SCHEMA_V29)
}

/// Maximum StandingApprovalPolicy TTL: 7 days (product 「本周」 time-box).
pub const STANDING_POLICY_MAX_TTL_MS: i64 = 7 * 24 * 60 * 60 * 1000;

/// Authority migration v30: grant-expansion subject_kind + StandingApprovalPolicy.
/// Rebuilds `p11_approval_preview` so CHECK can name `grant-expansion`.
/// Settings list/revoke is HTTP; Control Plane chrome is T13.
pub const STANDING_APPROVAL_POLICY_SCHEMA_V30: &str = "
CREATE TABLE p11_approval_preview_v30 (
  preview_id TEXT PRIMARY KEY,
  subject_kind TEXT NOT NULL CHECK (subject_kind IN (
    'activation','plan-change','acceptance','grant-expansion'
  )),
  subject_ref TEXT NOT NULL,
  base_state_digest TEXT NOT NULL CHECK (length(base_state_digest) = 64),
  preview_bytes_ref TEXT NOT NULL,
  preview_digest TEXT NOT NULL UNIQUE CHECK (length(preview_digest) = 64),
  status TEXT NOT NULL CHECK (status IN (
    'pending','approved','rejected','stale','consumed','superseded'
  )),
  intent_id TEXT,
  receipt_ref TEXT,
  created_at INTEGER NOT NULL,
  decided_at INTEGER,
  superseded_by TEXT
) STRICT;
INSERT INTO p11_approval_preview_v30 (
  preview_id, subject_kind, subject_ref, base_state_digest, preview_bytes_ref,
  preview_digest, status, intent_id, receipt_ref, created_at, decided_at, superseded_by
) SELECT
  preview_id, subject_kind, subject_ref, base_state_digest, preview_bytes_ref,
  preview_digest, status, intent_id, receipt_ref, created_at, decided_at, superseded_by
FROM p11_approval_preview;
DROP TABLE p11_approval_preview;
ALTER TABLE p11_approval_preview_v30 RENAME TO p11_approval_preview;
CREATE TABLE p11_standing_approval_policy (
  policy_id TEXT PRIMARY KEY,
  subject_class TEXT NOT NULL,
  subject_ref TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  revoked_at INTEGER
) STRICT;
";

/// v30 migration entry.
pub fn standing_approval_policy_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(30, STANDING_APPROVAL_POLICY_SCHEMA_V30)
}

/// Failures from the Project aggregate store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectAggregateError {
    Unavailable { detail: String },
    Conflict { detail: &'static str },
    NotFound { detail: &'static str },
    Invalid { detail: &'static str },
    Forbidden { detail: &'static str },
    Stale { detail: &'static str },
    Unconfirmed { detail: &'static str },
    Rejected { detail: &'static str },
}

impl std::fmt::Display for ProjectAggregateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable { detail } => {
                write!(formatter, "project aggregate store unavailable: {detail}")
            }
            Self::Conflict { detail } => write!(formatter, "project aggregate conflict: {detail}"),
            Self::NotFound { detail } => write!(formatter, "project aggregate not found: {detail}"),
            Self::Invalid { detail } => write!(formatter, "project aggregate invalid: {detail}"),
            Self::Forbidden { detail } => {
                write!(formatter, "project aggregate forbidden: {detail}")
            }
            Self::Stale { detail } => write!(formatter, "project aggregate stale: {detail}"),
            Self::Unconfirmed { detail } => {
                write!(formatter, "project aggregate unconfirmed: {detail}")
            }
            Self::Rejected { detail } => write!(formatter, "project aggregate rejected: {detail}"),
        }
    }
}

impl std::error::Error for ProjectAggregateError {}

/// Who may confirm / apply. Only owner management is authorized (N12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmCaller {
    OwnerManagement,
    TaskChannel,
    Assistant,
}

/// T04 seating facts. Production load is always empty (N8 fail-closed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeatingFacts {
    pub seated: bool,
}

impl SeatingFacts {
    /// Production T03 load: no Employee table exists.
    pub const EMPTY: Self = Self { seated: false };
}

/// Inputs the daemon uses to *derive* StageTestPassed. There is no caller
/// `passed` string (N6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageTestOracle {
    pub project_id: String,
    pub plan_revision_id: String,
    pub stage_id: String,
    pub task_ref: String,
    pub seating: SeatingFacts,
    pub verification_current: bool,
    pub verification_report_ref: String,
    pub openable: bool,
    pub checks_passed: bool,
    pub effects_closed: bool,
    pub now_ms: i64,
}

/// One axis ring as supplied by the daemon writer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageSpec {
    pub stage_id: String,
    pub title: String,
    pub objective: String,
    pub output_contract_digest: String,
    pub acceptance_spec_ref: Option<String>,
    pub cadence_json: Option<String>,
    pub responsible_slot: String,
    pub blocking_gap: Option<String>,
}

/// Durable Project aggregate on the authority SQLite writer.
#[derive(Clone)]
pub struct ProjectAggregateStore {
    conn: Arc<Mutex<Connection>>,
}

impl ProjectAggregateStore {
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

    fn require_owner(caller: ConfirmCaller) -> Result<(), ProjectAggregateError> {
        match caller {
            ConfirmCaller::OwnerManagement => Ok(()),
            ConfirmCaller::TaskChannel | ConfirmCaller::Assistant => {
                Err(ProjectAggregateError::Forbidden {
                    detail: "only owner management session may confirm, reject, narrow, or apply",
                })
            }
        }
    }

    /// SHA-256 hex of daemon-canonical bytes.
    pub fn digest_hex(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    pub(crate) fn reject_secret_shape(bytes: &[u8]) -> Result<(), ProjectAggregateError> {
        let lowered = String::from_utf8_lossy(bytes).to_ascii_lowercase();
        if looks_like_secret(&lowered) {
            return Err(ProjectAggregateError::Invalid {
                detail: "secret-shaped material is rejected at registration",
            });
        }
        Ok(())
    }

    /// Count Task contract rows. Project creation must not change this (N1).
    pub fn count_task_contracts(&self) -> Result<i64, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row("SELECT COUNT(*) FROM task_contracts", [], |row| row.get(0))
            .map_err(unavailable("count task contracts"))
    }

    /// A Task ref is never a Project id (N1).
    pub fn get_project(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        ensure_write_facts_locked(&conn)?;
        conn.query_row(
            "SELECT p.project_id, p.state, p.current_charter_revision_id, p.current_plan_revision_id,
                    p.created_at, p.activated_at, p.accepted_at,
                    COALESCE(f.title_summary, 'unknown')
               FROM p11_project p
               LEFT JOIN p14_write_project_facts f ON f.project_id = p.project_id
              WHERE p.project_id = ?1",
            [project_id],
            map_project_row,
        )
        .optional()
        .map_err(unavailable("get project"))
    }

    pub fn create_draft(
        &self,
        payload: &[u8],
        now_ms: i64,
    ) -> Result<(String, String), ProjectAggregateError> {
        Self::reject_secret_shape(payload)?;
        let payload_utf8 = utf8_text(payload, "draft payload must be utf-8")?;
        let draft_id = next_id("draft")?;
        let payload_digest = Self::digest_hex(payload);
        let conn = self.lock()?;
        ensure_write_facts_locked(&conn)?;
        conn.execute(
            "INSERT INTO p11_draft (
                draft_id, kind, base_seq, payload_digest, state, activated_project_id, created_at, updated_at
             ) VALUES (?1,'project-create',0,?2,'open',NULL,?3,?3)",
            params![draft_id, payload_digest, now_ms],
        )
        .map_err(unavailable("insert draft"))?;
        conn.execute(
            "INSERT INTO p14_write_project_facts (
                draft_id, payload_utf8, charter_utf8, title_summary, project_id
             ) VALUES (?1,?2,'','unknown',NULL)",
            params![draft_id, payload_utf8],
        )
        .map_err(unavailable("insert write-project facts"))?;
        Ok((draft_id, payload_digest))
    }

    pub fn put_draft_charter(
        &self,
        draft_id: &str,
        content: &[u8],
        now_ms: i64,
    ) -> Result<(String, String), ProjectAggregateError> {
        let _ = now_ms;
        Self::reject_secret_shape(content)?;
        let charter_utf8 = utf8_text(content, "draft charter must be utf-8")?;
        let charter_revision_id = next_id("charter")?;
        let content_digest = Self::digest_hex(content);
        let conn = self.lock()?;
        ensure_write_facts_locked(&conn)?;
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_draft WHERE draft_id = ?1 AND state = 'open'",
                [draft_id],
                |row| row.get(0),
            )
            .map_err(unavailable("lookup draft for charter"))?;
        if exists == 0 {
            return Err(ProjectAggregateError::NotFound {
                detail: "open draft not found",
            });
        }
        conn.execute(
            "INSERT INTO p11_charter_revision (
                charter_revision_id, project_id, draft_id, seq, content_digest, status, confirmed_at, source_intent_id
             ) VALUES (?1, NULL, ?2, 1, ?3, 'draft', NULL, NULL)",
            params![charter_revision_id, draft_id, content_digest],
        )
        .map_err(unavailable("insert charter"))?;
        let updated = conn
            .execute(
                "UPDATE p14_write_project_facts SET charter_utf8 = ?1 WHERE draft_id = ?2",
                params![charter_utf8, draft_id],
            )
            .map_err(unavailable("update write-project charter"))?;
        if updated == 0 {
            conn.execute(
                "INSERT INTO p14_write_project_facts (
                    draft_id, payload_utf8, charter_utf8, title_summary, project_id
                 ) VALUES (?1,'',?2,'unknown',NULL)",
                params![draft_id, charter_utf8],
            )
            .map_err(unavailable("insert write-project charter"))?;
        }
        Ok((charter_revision_id, content_digest))
    }

    pub fn register_candidate(
        &self,
        draft_id: &str,
        base_seq: i64,
        ops: &[u8],
        author: &str,
        sources_json: Option<&str>,
    ) -> Result<(String, String), ProjectAggregateError> {
        Self::reject_secret_shape(ops)?;
        if author != "owner" && author != "assistant" {
            return Err(ProjectAggregateError::Invalid {
                detail: "candidate author must be owner or assistant",
            });
        }
        if author == "assistant" {
            validate_assistant_provenance(sources_json)?;
            reject_closed_candidate_schema(ops)?;
        }
        let candidate_id = next_id("candidate")?;
        let ops_digest = Self::digest_hex(ops);
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO p11_candidate (
                candidate_id, draft_id, base_seq, ops_digest, author, sources_json, applied_at, superseded_at
             ) VALUES (?1,?2,?3,?4,?5,?6,NULL,NULL)",
            params![
                candidate_id,
                draft_id,
                base_seq,
                ops_digest,
                author,
                sources_json
            ],
        )
        .map_err(unavailable("insert candidate"))?;
        Ok((candidate_id, ops_digest))
    }

    pub fn apply_candidate(
        &self,
        caller: ConfirmCaller,
        draft_id: &str,
        base_seq: i64,
        candidate_digest: &str,
        now_ms: i64,
    ) -> Result<(i64, String), ProjectAggregateError> {
        Self::require_owner(caller)?;
        let conn = self.lock()?;
        if is_authority_object(&conn, draft_id)? {
            return Err(ProjectAggregateError::Invalid {
                detail: "draft.apply cannot target authority objects",
            });
        }
        let (current_seq, state): (i64, String) = conn
            .query_row(
                "SELECT base_seq, state FROM p11_draft WHERE draft_id = ?1",
                [draft_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("load draft for apply"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "draft not found",
            })?;
        if state != "open" {
            return Err(ProjectAggregateError::Conflict {
                detail: "draft is not open",
            });
        }
        if current_seq != base_seq {
            return Err(ProjectAggregateError::Conflict {
                detail: "draft apply rejected: base_seq is stale",
            });
        }
        let candidate_id: String = conn
            .query_row(
                "SELECT candidate_id FROM p11_candidate
                  WHERE draft_id = ?1 AND ops_digest = ?2 AND base_seq = ?3 AND applied_at IS NULL",
                params![draft_id, candidate_digest, base_seq],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("load candidate"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "candidate not found for base_seq",
            })?;
        let new_seq = current_seq + 1;
        conn.execute(
            "UPDATE p11_draft SET base_seq = ?1, payload_digest = ?2, updated_at = ?3 WHERE draft_id = ?4",
            params![new_seq, candidate_digest, now_ms, draft_id],
        )
        .map_err(unavailable("advance draft seq"))?;
        conn.execute(
            "UPDATE p11_candidate SET applied_at = ?1 WHERE candidate_id = ?2",
            params![now_ms, candidate_id],
        )
        .map_err(unavailable("mark candidate applied"))?;
        Ok((new_seq, candidate_digest.to_owned()))
    }

    fn activation_base_digest(
        conn: &Connection,
        draft_id: &str,
    ) -> Result<String, ProjectAggregateError> {
        let payload_digest: String = conn
            .query_row(
                "SELECT payload_digest FROM p11_draft WHERE draft_id = ?1",
                [draft_id],
                |row| row.get(0),
            )
            .map_err(unavailable("draft payload for preview"))?;
        let charter_digest: Option<String> = conn
            .query_row(
                "SELECT content_digest FROM p11_charter_revision
                  WHERE draft_id = ?1 AND status = 'draft' ORDER BY seq DESC LIMIT 1",
                [draft_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("charter for preview"))?;
        let joined = format!(
            "payload={}\ncharter={}",
            payload_digest,
            charter_digest.as_deref().unwrap_or("")
        );
        Ok(Self::digest_hex(joined.as_bytes()))
    }

    pub fn request_preview(
        &self,
        subject_kind: &str,
        subject_ref: &str,
        preview_bytes: &[u8],
        now_ms: i64,
    ) -> Result<(String, String), ProjectAggregateError> {
        Self::reject_secret_shape(preview_bytes)?;
        if !matches!(
            subject_kind,
            "activation"
                | "plan-change"
                | "acceptance"
                | "grant-expansion"
                | "run-acceptance"
                | "external-send"
                | "plan-revision"
                | "task-revision"
        ) {
            return Err(ProjectAggregateError::Invalid {
                detail: "unsupported subject_kind",
            });
        }
        let conn = self.lock()?;
        let pending: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_approval_preview
                  WHERE subject_kind = ?1 AND subject_ref = ?2 AND status = 'pending'",
                params![subject_kind, subject_ref],
                |row| row.get(0),
            )
            .map_err(unavailable("pending preview count"))?;
        if pending > 0 {
            return Err(ProjectAggregateError::Conflict {
                detail: "pending preview already exists for subject",
            });
        }
        let base_state_digest =
            self.subject_base_digest_locked(&conn, subject_kind, subject_ref)?;
        Self::mint_pending_preview_locked(
            &conn,
            subject_kind,
            subject_ref,
            &base_state_digest,
            preview_bytes,
            now_ms,
        )
    }

    fn subject_base_digest_locked(
        &self,
        conn: &Connection,
        subject_kind: &str,
        subject_ref: &str,
    ) -> Result<String, ProjectAggregateError> {
        match subject_kind {
            "activation" => Self::activation_base_digest(conn, subject_ref),
            "plan-change" => self.plan_change_base_digest_locked(conn, subject_ref),
            "acceptance" => self.acceptance_base_digest_locked(conn, subject_ref),
            "grant-expansion" => Self::grant_expansion_base_digest_locked(conn, subject_ref),
            "run-acceptance" => {
                crate::attempt_artifacts::run_acceptance_base_digest_locked(conn, subject_ref)
            }
            "external-send" => {
                crate::attempt_artifacts::external_send_base_digest_locked(conn, subject_ref)
            }
            "plan-revision" | "task-revision" => {
                crate::project_chat::candidate_base_digest_locked(conn, subject_kind, subject_ref)
            }
            _ => Err(ProjectAggregateError::Invalid {
                detail: "unsupported subject_kind",
            }),
        }
    }

    fn mint_pending_preview_locked(
        conn: &Connection,
        subject_kind: &str,
        subject_ref: &str,
        base_state_digest: &str,
        preview_bytes: &[u8],
        now_ms: i64,
    ) -> Result<(String, String), ProjectAggregateError> {
        let preview_id = next_id("preview")?;
        let preview_bytes_ref = format!("cas:{}", Self::digest_hex(preview_bytes));
        let preview_digest = Self::digest_hex(
            format!("{base_state_digest}\n{preview_bytes_ref}\n{subject_kind}\n{subject_ref}")
                .as_bytes(),
        );
        conn.execute(
            "INSERT INTO p11_approval_preview (
                preview_id, subject_kind, subject_ref, base_state_digest, preview_bytes_ref,
                preview_digest, status, intent_id, receipt_ref, created_at, decided_at,
                superseded_by
             ) VALUES (?1,?2,?3,?4,?5,?6,'pending',NULL,NULL,?7,NULL,NULL)",
            params![
                preview_id,
                subject_kind,
                subject_ref,
                base_state_digest,
                preview_bytes_ref,
                preview_digest,
                now_ms
            ],
        )
        .map_err(unavailable("insert preview"))?;
        Ok((preview_id, preview_digest))
    }

    fn load_preview_locked(
        conn: &Connection,
        preview_id: &str,
    ) -> Result<PreviewLookup, ProjectAggregateError> {
        conn.query_row(
            "SELECT subject_kind, subject_ref, base_state_digest, preview_digest, status
               FROM p11_approval_preview WHERE preview_id = ?1",
            [preview_id],
            |row| {
                Ok(PreviewLookup {
                    subject_kind: row.get(0)?,
                    subject_ref: row.get(1)?,
                    base_state_digest: row.get(2)?,
                    preview_digest: row.get(3)?,
                    status: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(unavailable("load preview"))?
        .ok_or(ProjectAggregateError::NotFound {
            detail: "preview not found",
        })
    }

    fn plan_change_base_digest_locked(
        &self,
        conn: &Connection,
        subject_ref: &str,
    ) -> Result<String, ProjectAggregateError> {
        let (project_id, stage_id) = parse_stage_ref(subject_ref)?;
        let plan_id: String = conn
            .query_row(
                "SELECT current_plan_revision_id FROM p11_project WHERE project_id = ?1",
                [project_id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(unavailable("project plan for preview"))?
            .flatten()
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project has no current plan revision",
            })?;
        let digest = format!("plan={plan_id}\nstage={stage_id}");
        Ok(Self::digest_hex(digest.as_bytes()))
    }

    fn acceptance_base_digest_locked(
        &self,
        conn: &Connection,
        project_id: &str,
    ) -> Result<String, ProjectAggregateError> {
        let plan_id: Option<String> = conn
            .query_row(
                "SELECT current_plan_revision_id FROM p11_project WHERE project_id = ?1",
                [project_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("project for acceptance preview"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project not found",
            })?;
        let Some(plan_id) = plan_id else {
            return Err(ProjectAggregateError::NotFound {
                detail: "project has no current plan revision",
            });
        };
        Ok(Self::digest_hex(
            format!("accept:{project_id}:{plan_id}").as_bytes(),
        ))
    }

    fn grant_expansion_base_digest_locked(
        conn: &Connection,
        subject_ref: &str,
    ) -> Result<String, ProjectAggregateError> {
        let spec = parse_grant_expansion_ref(subject_ref)?;
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_employee WHERE employee_id = ?1 AND project_id = ?2",
                params![spec.employee_id, spec.project_id],
                |row| row.get(0),
            )
            .map_err(unavailable("employee for grant-expansion"))?;
        if exists == 0 {
            return Err(ProjectAggregateError::NotFound {
                detail: "employee not found",
            });
        }
        let mut statement = conn
            .prepare(
                "SELECT capability_ref, scope FROM p11_grant
                  WHERE project_id = ?1 AND employee_id = ?2
                  ORDER BY capability_ref, scope",
            )
            .map_err(unavailable("grant catalog for preview"))?;
        let rows = statement
            .query_map(params![spec.project_id, spec.employee_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(unavailable("grant catalog query"))?;
        let mut catalog = String::new();
        for row in rows {
            let (capability, scope) = row.map_err(unavailable("grant catalog row"))?;
            catalog.push_str(&format!("{capability}={scope}\n"));
        }
        Ok(Self::digest_hex(
            format!(
                "grant-expansion\n{}\n{}\n{}\n{}\n{catalog}",
                spec.project_id, spec.employee_id, spec.capability_ref, spec.scope
            )
            .as_bytes(),
        ))
    }

    fn grant_expansion_locked(
        conn: &Connection,
        subject_ref: &str,
        now_ms: i64,
    ) -> Result<ConfirmResult, ProjectAggregateError> {
        let spec = parse_grant_expansion_ref(subject_ref)?;
        let employee_project: String = conn
            .query_row(
                "SELECT project_id FROM p11_employee WHERE employee_id = ?1",
                [&spec.employee_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("employee for grant confirm"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "employee not found",
            })?;
        if employee_project != spec.project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-project write rejected",
            });
        }
        let installed: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_install_fact WHERE capability_ref = ?1",
                [&spec.capability_ref],
                |row| row.get(0),
            )
            .map_err(unavailable("install fact for grant-expansion"))?;
        if installed == 0 {
            return Err(ProjectAggregateError::Rejected {
                detail: "grant requires an InstallFact",
            });
        }
        let already: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_grant
                  WHERE project_id = ?1 AND employee_id = ?2
                    AND capability_ref = ?3 AND scope = ?4",
                params![
                    spec.project_id,
                    spec.employee_id,
                    spec.capability_ref,
                    spec.scope
                ],
                |row| row.get(0),
            )
            .map_err(unavailable("duplicate grant"))?;
        if already > 0 {
            return Err(ProjectAggregateError::Conflict {
                detail: "grant already exists for subject",
            });
        }
        let grant_id = next_id("grant")?;
        conn.execute(
            "INSERT INTO p11_grant (grant_id, project_id, employee_id, capability_ref, scope, created_at)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                grant_id,
                spec.project_id,
                spec.employee_id,
                spec.capability_ref,
                spec.scope,
                now_ms
            ],
        )
        .map_err(unavailable("insert grant-expansion"))?;
        let receipt_ref = format!("receipt:grant:{grant_id}");
        Ok(ConfirmResult {
            kind: "granted",
            new_ref: grant_id,
            receipt_ref,
        })
    }

    pub fn confirm_preview(
        &self,
        caller: ConfirmCaller,
        preview_id: &str,
        preview_digest: &str,
        now_ms: i64,
    ) -> Result<ConfirmResult, ProjectAggregateError> {
        Self::require_owner(caller)?;
        let conn = self.lock()?;
        let preview = Self::load_preview_locked(&conn, preview_id)?;
        if preview.status != "pending" {
            return Err(ProjectAggregateError::Invalid {
                detail: "preview is not pending",
            });
        }
        if preview.preview_digest != preview_digest {
            return Err(ProjectAggregateError::Stale {
                detail: "preview_digest does not match",
            });
        }
        let current_base =
            self.subject_base_digest_locked(&conn, &preview.subject_kind, &preview.subject_ref)?;
        if current_base != preview.base_state_digest {
            conn.execute(
                "UPDATE p11_approval_preview SET status = 'stale', decided_at = ?1 WHERE preview_id = ?2",
                params![now_ms, preview_id],
            )
            .map_err(unavailable("mark preview stale"))?;
            return Err(ProjectAggregateError::Stale {
                detail: "base_state_digest is stale",
            });
        }
        let result = match preview.subject_kind.as_str() {
            "activation" => Self::activate_locked(&conn, &preview.subject_ref, now_ms)?,
            "plan-change" => self.confirm_stage_from_preview_locked(&conn, &preview.subject_ref)?,
            "acceptance" => self.accept_locked(&conn, &preview.subject_ref, now_ms)?,
            "grant-expansion" => Self::grant_expansion_locked(&conn, &preview.subject_ref, now_ms)?,
            "run-acceptance" => {
                crate::attempt_artifacts::accept_run_locked(&conn, &preview.subject_ref, now_ms)?
            }
            "external-send" => {
                crate::attempt_artifacts::external_send_locked(&conn, &preview.subject_ref, now_ms)?
            }
            "plan-revision" | "task-revision" => {
                crate::project_chat::confirm_chat_candidate_locked(
                    &conn,
                    &preview.subject_kind,
                    &preview.subject_ref,
                    now_ms,
                )?
            }
            _ => {
                return Err(ProjectAggregateError::Invalid {
                    detail: "unsupported subject_kind",
                });
            }
        };
        conn.execute(
            "UPDATE p11_approval_preview
                SET status = 'consumed', decided_at = ?1, receipt_ref = ?2
              WHERE preview_id = ?3",
            params![now_ms, result.receipt_ref, preview_id],
        )
        .map_err(unavailable("consume preview"))?;
        Ok(result)
    }

    /// Owner reject of a pending preview. Leaves a receipt; the rejected digest
    /// is never confirmable. Stale is not a time check — reject of a pending
    /// digest succeeds even after wall-clock delay.
    pub fn reject_preview(
        &self,
        caller: ConfirmCaller,
        preview_id: &str,
        preview_digest: &str,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        Self::require_owner(caller)?;
        let conn = self.lock()?;
        let preview = Self::load_preview_locked(&conn, preview_id)?;
        if preview.status != "pending" {
            return Err(ProjectAggregateError::Invalid {
                detail: "preview is not pending",
            });
        }
        if preview.preview_digest != preview_digest {
            return Err(ProjectAggregateError::Stale {
                detail: "preview_digest does not match",
            });
        }
        let receipt_ref = format!("receipt:reject:{preview_id}");
        conn.execute(
            "UPDATE p11_approval_preview
                SET status = 'rejected', decided_at = ?1, receipt_ref = ?2
              WHERE preview_id = ?3",
            params![now_ms, receipt_ref, preview_id],
        )
        .map_err(unavailable("reject preview"))?;
        Ok(receipt_ref)
    }

    /// Owner narrow: mint a **new** pending preview and freeze the old row as
    /// `superseded` with `superseded_by`. The old digest is never confirmable.
    /// Stale is mechanical `base_state_digest` mismatch only.
    pub fn narrow_preview(
        &self,
        caller: ConfirmCaller,
        preview_id: &str,
        preview_digest: &str,
        new_preview_bytes: &[u8],
        now_ms: i64,
    ) -> Result<NarrowResult, ProjectAggregateError> {
        Self::require_owner(caller)?;
        Self::reject_secret_shape(new_preview_bytes)?;
        let conn = self.lock()?;
        let preview = Self::load_preview_locked(&conn, preview_id)?;
        if preview.status != "pending" {
            return Err(ProjectAggregateError::Invalid {
                detail: "preview is not pending",
            });
        }
        if preview.preview_digest != preview_digest {
            return Err(ProjectAggregateError::Stale {
                detail: "preview_digest does not match",
            });
        }
        let current_base =
            self.subject_base_digest_locked(&conn, &preview.subject_kind, &preview.subject_ref)?;
        if current_base != preview.base_state_digest {
            conn.execute(
                "UPDATE p11_approval_preview SET status = 'stale', decided_at = ?1 WHERE preview_id = ?2",
                params![now_ms, preview_id],
            )
            .map_err(unavailable("mark preview stale"))?;
            return Err(ProjectAggregateError::Stale {
                detail: "base_state_digest is stale",
            });
        }
        let preview_bytes_ref = format!("cas:{}", Self::digest_hex(new_preview_bytes));
        let candidate_digest = Self::digest_hex(
            format!(
                "{current_base}\n{preview_bytes_ref}\n{}\n{}",
                preview.subject_kind, preview.subject_ref
            )
            .as_bytes(),
        );
        if candidate_digest == preview.preview_digest {
            return Err(ProjectAggregateError::Invalid {
                detail: "narrow must change preview bytes",
            });
        }
        let (new_preview_id, new_digest) = Self::mint_pending_preview_locked(
            &conn,
            &preview.subject_kind,
            &preview.subject_ref,
            &current_base,
            new_preview_bytes,
            now_ms,
        )?;
        conn.execute(
            "UPDATE p11_approval_preview
                SET status = 'superseded', decided_at = ?1, superseded_by = ?2
              WHERE preview_id = ?3",
            params![now_ms, new_preview_id, preview_id],
        )
        .map_err(unavailable("supersede preview"))?;
        Ok(NarrowResult {
            preview_id: new_preview_id,
            preview_digest: new_digest,
            superseded_preview_id: preview_id.to_owned(),
        })
    }

    fn activate_locked(
        conn: &Connection,
        draft_id: &str,
        now_ms: i64,
    ) -> Result<ConfirmResult, ProjectAggregateError> {
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(unavailable("begin write-project activate"))?;
        let result = Self::activate_body_locked(conn, draft_id, now_ms);
        match result {
            Ok(outcome) => {
                conn.execute_batch("COMMIT")
                    .map_err(unavailable("commit write-project activate"))?;
                Ok(outcome)
            }
            Err(error) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(error)
            }
        }
    }

    fn activate_body_locked(
        conn: &Connection,
        draft_id: &str,
        now_ms: i64,
    ) -> Result<ConfirmResult, ProjectAggregateError> {
        ensure_write_facts_locked(conn)?;
        let charter: Option<(String, String)> = conn
            .query_row(
                "SELECT charter_revision_id, content_digest FROM p11_charter_revision
                  WHERE draft_id = ?1 AND status = 'draft' ORDER BY seq DESC LIMIT 1",
                [draft_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("load draft charter"))?;
        let Some((charter_revision_id, _)) = charter else {
            return Err(ProjectAggregateError::Unconfirmed {
                detail: "G1 rejected: no confirmed charter revision",
            });
        };
        let facts: Option<(String, String)> = conn
            .query_row(
                "SELECT payload_utf8, charter_utf8 FROM p14_write_project_facts WHERE draft_id = ?1",
                [draft_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("load write-project facts"))?;
        let (payload_utf8, charter_utf8) = facts.unwrap_or_default();
        let title = owner_title_from_payload(&payload_utf8)?.to_owned();
        let dual_track_stages = if charter_declares_process(&charter_utf8) {
            let stages = parse_dual_track_stages(&charter_utf8)?;
            if stages.is_empty() {
                return Err(ProjectAggregateError::Invalid {
                    detail: "Write Project requires a PlanRevision axis",
                });
            }
            Some(stages)
        } else {
            None
        };
        let project_id = next_id("project")?;
        if dual_track_stages.is_some() {
            conn.execute(
                "INSERT INTO p11_project (
                    project_id, state, current_charter_revision_id, current_plan_revision_id,
                    created_at, activated_at, accepted_at
                 ) VALUES (?1,'active',?2,NULL,?3,?3,?3)",
                params![project_id, charter_revision_id, now_ms],
            )
            .map_err(unavailable("insert live project"))?;
        } else {
            conn.execute(
                "INSERT INTO p11_project (
                    project_id, state, current_charter_revision_id, current_plan_revision_id,
                    created_at, activated_at, accepted_at
                 ) VALUES (?1,'creating',?2,NULL,?3,?3,NULL)",
                params![project_id, charter_revision_id, now_ms],
            )
            .map_err(unavailable("insert project"))?;
        }
        if let Some(stages) = dual_track_stages.as_ref() {
            Self::apply_plan_revision_locked(conn, &project_id, stages, now_ms)?;
            conn.execute(
                "UPDATE p14_write_project_facts
                    SET title_summary = ?1, project_id = ?2
                  WHERE draft_id = ?3",
                params![title, project_id, draft_id],
            )
            .map_err(unavailable("bind write-project title"))?;
        }
        conn.execute(
            "UPDATE p11_charter_revision
                SET project_id = ?1, status = 'confirmed', confirmed_at = ?2
              WHERE charter_revision_id = ?3",
            params![project_id, now_ms, charter_revision_id],
        )
        .map_err(unavailable("confirm charter"))?;
        conn.execute(
            "UPDATE p11_draft SET state = 'activated', activated_project_id = ?1, updated_at = ?2
              WHERE draft_id = ?3",
            params![project_id, now_ms, draft_id],
        )
        .map_err(unavailable("activate draft"))?;
        Ok(ConfirmResult {
            kind: "activated",
            new_ref: project_id,
            receipt_ref: format!("receipt:{draft_id}"),
        })
    }

    fn confirm_stage_from_preview_locked(
        &self,
        conn: &Connection,
        subject_ref: &str,
    ) -> Result<ConfirmResult, ProjectAggregateError> {
        let (project_id, stage_id) = parse_stage_ref(subject_ref)?;
        let (plan_id, expected_digest): (String, String) = conn
            .query_row(
                "SELECT s.plan_revision_id, s.stage_digest
                   FROM p11_stage s
                   JOIN p11_project p ON p.current_plan_revision_id = s.plan_revision_id
                  WHERE p.project_id = ?1 AND s.stage_id = ?2",
                params![project_id, stage_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("load stage for confirm"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "stage not on current plan revision",
            })?;
        self.confirm_stage_locked(conn, project_id, &plan_id, stage_id, &expected_digest)?;
        Ok(ConfirmResult {
            kind: "plan_revision_created",
            new_ref: plan_id,
            receipt_ref: format!("receipt:stage:{stage_id}"),
        })
    }

    pub fn confirm_stage(
        &self,
        caller: ConfirmCaller,
        project_id: &str,
        plan_revision_id: &str,
        stage_id: &str,
        stage_digest: &str,
    ) -> Result<(), ProjectAggregateError> {
        Self::require_owner(caller)?;
        let conn = self.lock()?;
        self.confirm_stage_locked(&conn, project_id, plan_revision_id, stage_id, stage_digest)
    }

    fn confirm_stage_locked(
        &self,
        conn: &Connection,
        project_id: &str,
        plan_revision_id: &str,
        stage_id: &str,
        stage_digest: &str,
    ) -> Result<(), ProjectAggregateError> {
        let current_plan: Option<String> = conn
            .query_row(
                "SELECT current_plan_revision_id FROM p11_project WHERE project_id = ?1",
                [project_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("project for stage confirm"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project not found",
            })?;
        let Some(current_plan) = current_plan else {
            return Err(ProjectAggregateError::NotFound {
                detail: "project has no plan",
            });
        };
        if current_plan != plan_revision_id {
            return Err(ProjectAggregateError::Stale {
                detail: "plan revision is superseded",
            });
        }
        let (stored_digest, blocking): (String, i64) = conn
            .query_row(
                "SELECT s.stage_digest,
                        COALESCE((
                          SELECT MAX(g.blocking) FROM p11_gap g
                           WHERE g.plan_revision_id = s.plan_revision_id
                             AND g.stage_id = s.stage_id
                             AND g.resolved_by_revision_id IS NULL
                             AND g.accepted_as_limitation_at IS NULL
                        ), 0)
                   FROM p11_stage s
                  WHERE s.plan_revision_id = ?1 AND s.stage_id = ?2",
                params![plan_revision_id, stage_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("stage digest"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "stage not found",
            })?;
        if stored_digest != stage_digest {
            return Err(ProjectAggregateError::Stale {
                detail: "stage_digest does not match current ring",
            });
        }
        if blocking != 0 {
            return Err(ProjectAggregateError::Rejected {
                detail: "blocking gap stage cannot confirm",
            });
        }
        conn.execute(
            "UPDATE p11_stage SET confirm_status = 'confirmed', ready = 1
              WHERE plan_revision_id = ?1 AND stage_id = ?2",
            params![plan_revision_id, stage_id],
        )
        .map_err(unavailable("confirm stage"))?;
        Ok(())
    }

    /// Replace the current plan with a new revision. Unchanged `stage_digest`
    /// rings keep `confirmed`; changed rings roll back (N10).
    pub fn apply_plan_revision(
        &self,
        caller_project_id: &str,
        target_project_id: &str,
        stages: &[StageSpec],
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        if caller_project_id != target_project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-project write rejected",
            });
        }
        if stages.is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "plan requires at least one stage",
            });
        }
        for stage in stages {
            Self::reject_secret_shape(stage.title.as_bytes())?;
            Self::reject_secret_shape(stage.objective.as_bytes())?;
        }
        let conn = self.lock()?;
        Self::apply_plan_revision_locked(&conn, target_project_id, stages, now_ms)
    }

    /// Materialize a new PlanRevision on an already-held writer connection.
    /// Callers must have validated caller scope and secret shape; the canvas
    /// Confirm of a chat-routed candidate (P13-T06) runs this inside the
    /// preview transaction, where the aggregate lock is already held.
    pub(crate) fn apply_plan_revision_locked(
        conn: &Connection,
        target_project_id: &str,
        stages: &[StageSpec],
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        if stages.is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "plan requires at least one stage",
            });
        }
        let (seq, previous_plan): (i64, Option<String>) = conn
            .query_row(
                "SELECT COALESCE((SELECT MAX(seq) FROM p11_plan_revision WHERE project_id = ?1), 0),
                        current_plan_revision_id
                   FROM p11_project WHERE project_id = ?1",
                [target_project_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("load project for plan"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project not found",
            })?;
        let previous_digests = if let Some(previous_plan) = previous_plan.as_ref() {
            let mut statement = conn
                .prepare(
                    "SELECT stage_id, stage_digest, confirm_status FROM p11_stage
                      WHERE plan_revision_id = ?1",
                )
                .map_err(unavailable("previous stages"))?;
            let rows = statement
                .query_map([previous_plan], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .map_err(unavailable("previous stage rows"))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(unavailable("collect previous stages"))?
        } else {
            Vec::new()
        };
        let plan_revision_id = next_id("plan")?;
        let new_seq = seq + 1;
        let mut axis = String::new();
        let mut materialized = Vec::new();
        for (index, spec) in stages.iter().enumerate() {
            let position = i64::try_from(index).unwrap_or(i64::MAX);
            let input_seam = if index == 0 {
                None
            } else {
                Some(stages[index - 1].output_contract_digest.as_str())
            };
            let stage_digest = compute_stage_digest(spec, position, input_seam);
            axis.push_str(&stage_digest);
            axis.push('\n');
            let previous = previous_digests
                .iter()
                .find(|(id, _, _)| id == &spec.stage_id);
            let confirm_status = match previous {
                Some((_, digest, status)) if digest == &stage_digest && status == "confirmed" => {
                    "confirmed"
                }
                _ => "unconfirmed",
            };
            let ready = confirm_status == "confirmed" && spec.blocking_gap.is_none();
            materialized.push((
                spec,
                position,
                input_seam.map(str::to_owned),
                stage_digest,
                confirm_status,
                i64::from(ready),
            ));
        }
        let axis_digest = Self::digest_hex(axis.as_bytes());
        conn.execute(
            "INSERT INTO p11_plan_revision (plan_revision_id, project_id, seq, axis_digest, created_at)
             VALUES (?1,?2,?3,?4,?5)",
            params![plan_revision_id, target_project_id, new_seq, axis_digest, now_ms],
        )
        .map_err(unavailable("insert plan revision"))?;
        for (spec, position, input_seam, stage_digest, confirm_status, ready) in materialized {
            conn.execute(
                "INSERT INTO p11_stage (
                    plan_revision_id, stage_id, position, title, objective, input_seam_digest,
                    output_contract_digest, acceptance_spec_ref, cadence_json, responsible_slot,
                    confirm_status, stage_digest, ready
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
                params![
                    plan_revision_id,
                    spec.stage_id,
                    position,
                    spec.title,
                    spec.objective,
                    input_seam,
                    spec.output_contract_digest,
                    spec.acceptance_spec_ref,
                    spec.cadence_json,
                    spec.responsible_slot,
                    confirm_status,
                    stage_digest,
                    ready
                ],
            )
            .map_err(unavailable("insert stage"))?;
            if let Some(description) = spec.blocking_gap.as_ref() {
                let gap_id = next_id("gap")?;
                conn.execute(
                    "INSERT INTO p11_gap (
                        gap_id, plan_revision_id, stage_id, description, blocking,
                        resolved_by_revision_id, accepted_as_limitation_at
                     ) VALUES (?1,?2,?3,?4,1,NULL,NULL)",
                    params![gap_id, plan_revision_id, spec.stage_id, description],
                )
                .map_err(unavailable("insert gap"))?;
            }
        }
        conn.execute(
            "UPDATE p11_project SET current_plan_revision_id = ?1 WHERE project_id = ?2",
            params![plan_revision_id, target_project_id],
        )
        .map_err(unavailable("point project at plan"))?;
        Ok(plan_revision_id)
    }

    pub fn get_stage(
        &self,
        plan_revision_id: &str,
        stage_id: &str,
    ) -> Result<Option<StageRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT plan_revision_id, stage_id, position, title, objective, confirm_status,
                    stage_digest, ready, acceptance_spec_ref, cadence_json, responsible_slot,
                    output_contract_digest
               FROM p11_stage WHERE plan_revision_id = ?1 AND stage_id = ?2",
            params![plan_revision_id, stage_id],
            map_stage_row,
        )
        .optional()
        .map_err(unavailable("get stage"))
    }

    pub fn gap_description(
        &self,
        plan_revision_id: &str,
        stage_id: &str,
    ) -> Result<Option<String>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT description FROM p11_gap
              WHERE plan_revision_id = ?1 AND stage_id = ?2
              ORDER BY gap_id LIMIT 1",
            params![plan_revision_id, stage_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(unavailable("get gap"))
    }

    pub fn list_gaps(
        &self,
        plan_revision_id: &str,
        stage_id: &str,
    ) -> Result<Vec<GapRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT gap_id, description, blocking, accepted_as_limitation_at
                   FROM p11_gap WHERE plan_revision_id = ?1 AND stage_id = ?2",
            )
            .map_err(unavailable("list gaps"))?;
        let rows = statement
            .query_map(params![plan_revision_id, stage_id], |row| {
                let accepted: Option<i64> = row.get(3)?;
                Ok(GapRow {
                    gap_id: row.get(0)?,
                    description: row.get(1)?,
                    blocking: row.get::<_, i64>(2)? == 1,
                    accepted_as_limitation: accepted.is_some(),
                })
            })
            .map_err(unavailable("list gap query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("list gap rows"))
    }

    /// Derive StageTestPassed. Production seating is empty (N8). Tests may
    /// pass seated facts as an argument for N6/N7 oracles.
    pub fn derive_stage_test_passed(
        &self,
        oracle: &StageTestOracle,
    ) -> Result<String, ProjectAggregateError> {
        if !oracle.seating.seated {
            return Err(ProjectAggregateError::Rejected {
                detail: "unseated stage cannot start test",
            });
        }
        let conn = self.lock()?;
        let owner_project: String = conn
            .query_row(
                "SELECT project_id FROM p11_plan_revision WHERE plan_revision_id = ?1",
                [&oracle.plan_revision_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("plan owner for test"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "plan revision not found",
            })?;
        if owner_project != oracle.project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-project write rejected",
            });
        }
        let (ready, blocking): (i64, i64) = conn
            .query_row(
                "SELECT s.ready,
                        COALESCE((
                          SELECT MAX(g.blocking) FROM p11_gap g
                           WHERE g.plan_revision_id = s.plan_revision_id
                             AND g.stage_id = s.stage_id
                             AND g.resolved_by_revision_id IS NULL
                             AND g.accepted_as_limitation_at IS NULL
                        ), 0)
                   FROM p11_stage s
                  WHERE s.plan_revision_id = ?1 AND s.stage_id = ?2",
                params![oracle.plan_revision_id, oracle.stage_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("stage ready for test"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "stage not found",
            })?;
        if blocking != 0 {
            return Err(ProjectAggregateError::Rejected {
                detail: "blocking gap stage cannot start test",
            });
        }
        if ready == 0 {
            return Err(ProjectAggregateError::Rejected {
                detail: "stage is not ready",
            });
        }
        if !oracle.verification_current {
            return Err(ProjectAggregateError::Rejected {
                detail: "completion requires current verification",
            });
        }
        let archive_as_completion: Option<String> = conn
            .query_row(
                "SELECT record_id FROM p11_conversation_archive WHERE record_id = ?1",
                [&oracle.verification_report_ref],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("archive is not completion"))?;
        if archive_as_completion.is_some() {
            return Err(ProjectAggregateError::Rejected {
                detail: "conversation archive is observation-only, not completion",
            });
        }
        if !oracle.openable {
            return Err(ProjectAggregateError::Rejected {
                detail: "missing openable artifact blocks pass",
            });
        }
        if !oracle.checks_passed {
            return Err(ProjectAggregateError::Rejected {
                detail: "registered checks have not passed",
            });
        }
        if !oracle.effects_closed {
            return Err(ProjectAggregateError::Rejected {
                detail: "open or unknown Effect blocks pass",
            });
        }
        let gap_before: Option<String> = conn
            .query_row(
                "SELECT description FROM p11_gap
                  WHERE plan_revision_id = ?1 AND stage_id = ?2 LIMIT 1",
                params![oracle.plan_revision_id, oracle.stage_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("gap before test"))?;
        conn.execute(
            "UPDATE p11_stage_test_fact SET current = 0
              WHERE plan_revision_id = ?1 AND stage_id = ?2 AND current = 1",
            params![oracle.plan_revision_id, oracle.stage_id],
        )
        .map_err(unavailable("supersede prior fact"))?;
        let fact_id = next_id("fact")?;
        conn.execute(
            "INSERT INTO p11_stage_test_fact (
                fact_id, project_id, plan_revision_id, stage_id, task_ref,
                verification_report_ref, passed_at, current
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,1)",
            params![
                fact_id,
                oracle.project_id,
                oracle.plan_revision_id,
                oracle.stage_id,
                oracle.task_ref,
                oracle.verification_report_ref,
                oracle.now_ms
            ],
        )
        .map_err(unavailable("insert stage test fact"))?;
        let gap_after: Option<String> = conn
            .query_row(
                "SELECT description FROM p11_gap
                  WHERE plan_revision_id = ?1 AND stage_id = ?2 LIMIT 1",
                params![oracle.plan_revision_id, oracle.stage_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("gap after test"))?;
        if gap_before != gap_after {
            return Err(ProjectAggregateError::Invalid {
                detail: "stage test must not mutate gap rows",
            });
        }
        Ok(fact_id)
    }

    fn accept_locked(
        &self,
        conn: &Connection,
        project_id: &str,
        now_ms: i64,
    ) -> Result<ConfirmResult, ProjectAggregateError> {
        let plan_id: Option<String> = conn
            .query_row(
                "SELECT current_plan_revision_id FROM p11_project WHERE project_id = ?1",
                [project_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("project for G2"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project not found",
            })?;
        let Some(plan_id) = plan_id else {
            return Err(ProjectAggregateError::Rejected {
                detail: "joint acceptance requires a plan",
            });
        };
        let stage_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_stage WHERE plan_revision_id = ?1",
                [&plan_id],
                |row| row.get(0),
            )
            .map_err(unavailable("count stages"))?;
        let fact_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_stage_test_fact
                  WHERE plan_revision_id = ?1 AND current = 1",
                [&plan_id],
                |row| row.get(0),
            )
            .map_err(unavailable("count stage facts"))?;
        if stage_count == 0 || fact_count != stage_count {
            return Err(ProjectAggregateError::Rejected {
                detail: "joint acceptance requires all current stage facts",
            });
        }
        let decision_bytes = format!(
            "{{\"schema_version\":1,\"decision\":\"granted\",\"project_id\":\"{project_id}\",\"plan_revision_id\":\"{plan_id}\"}}"
        );
        let acceptance_decision_ref =
            format!("cas:{}", Self::digest_hex(decision_bytes.as_bytes()));
        conn.execute(
            "INSERT INTO p11_acceptance_fact (
                project_id, plan_revision_id, acceptance_decision_ref, accepted_at
             ) VALUES (?1,?2,?3,?4)",
            params![project_id, plan_id, acceptance_decision_ref, now_ms],
        )
        .map_err(unavailable("insert acceptance fact"))?;
        conn.execute(
            "UPDATE p11_project SET state = 'active', accepted_at = ?1 WHERE project_id = ?2",
            params![now_ms, project_id],
        )
        .map_err(unavailable("activate project G2"))?;
        Ok(ConfirmResult {
            kind: "accepted",
            new_ref: project_id.to_owned(),
            receipt_ref: acceptance_decision_ref,
        })
    }

    pub fn copy_project(
        &self,
        source_project_id: &str,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        let conn = self.lock()?;
        let source = conn
            .query_row(
                "SELECT current_charter_revision_id FROM p11_project WHERE project_id = ?1",
                [source_project_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(unavailable("source project"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "source project not found",
            })?;
        let content_digest: String = conn
            .query_row(
                "SELECT content_digest FROM p11_charter_revision WHERE charter_revision_id = ?1",
                [&source],
                |row| row.get(0),
            )
            .map_err(unavailable("source charter"))?;
        if looks_like_secret(&content_digest) {
            return Err(ProjectAggregateError::Invalid {
                detail: "copy excludes secrets",
            });
        }
        let inflight: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_stage_test_fact
                  WHERE project_id = ?1 AND current = 1",
                [source_project_id],
                |row| row.get(0),
            )
            .map_err(unavailable("inflight facts"))?;
        if inflight > 0 {
            return Err(ProjectAggregateError::Rejected {
                detail: "copy excludes inflight tasks",
            });
        }
        // P13-T09: a seated or granted source may be copied. The 副本 is a
        // charter-only inactive row and never receives employees, grants,
        // plans, or armed Routines.
        let copy_id = next_id("project")?;
        let copy_charter = next_id("charter")?;
        let copy_draft = next_id("draft")?;
        conn.execute(
            "INSERT INTO p11_draft (
                draft_id, kind, base_seq, payload_digest, state, activated_project_id, created_at, updated_at
             ) VALUES (?1,'project-create',0,?2,'activated',?3,?4,?4)",
            params![copy_draft, content_digest, copy_id, now_ms],
        )
        .map_err(unavailable("copy draft"))?;
        conn.execute(
            "INSERT INTO p11_charter_revision (
                charter_revision_id, project_id, draft_id, seq, content_digest, status, confirmed_at, source_intent_id
             ) VALUES (?1,?2,?3,1,?4,'confirmed',?5,NULL)",
            params![copy_charter, copy_id, copy_draft, content_digest, now_ms],
        )
        .map_err(unavailable("copy charter"))?;
        conn.execute(
            "INSERT INTO p11_project (
                project_id, state, current_charter_revision_id, current_plan_revision_id,
                created_at, activated_at, accepted_at
             ) VALUES (?1,'inactive',?2,NULL,?3,?3,NULL)",
            params![copy_id, copy_charter, now_ms],
        )
        .map_err(unavailable("insert inactive copy"))?;
        Ok(copy_id)
    }

    pub fn list_projects(&self, limit: i64) -> Result<Vec<ProjectRow>, ProjectAggregateError> {
        let cap = limit.clamp(1, 64);
        let conn = self.lock()?;
        ensure_write_facts_locked(&conn)?;
        let mut statement = conn
            .prepare(
                "SELECT p.project_id, p.state, p.current_charter_revision_id, p.current_plan_revision_id,
                        p.created_at, p.activated_at, p.accepted_at,
                        COALESCE(f.title_summary, 'unknown')
                   FROM p11_project p
                   LEFT JOIN p14_write_project_facts f ON f.project_id = p.project_id
                  ORDER BY p.created_at LIMIT ?1",
            )
            .map_err(unavailable("list projects"))?;
        let rows = statement
            .query_map([cap], map_project_row)
            .map_err(unavailable("list project query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("list project rows"))
    }

    pub fn list_stages(
        &self,
        plan_revision_id: &str,
    ) -> Result<Vec<StageRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT plan_revision_id, stage_id, position, title, objective, confirm_status,
                        stage_digest, ready, acceptance_spec_ref, cadence_json, responsible_slot,
                        output_contract_digest
                   FROM p11_stage WHERE plan_revision_id = ?1 ORDER BY position",
            )
            .map_err(unavailable("list stages"))?;
        let rows = statement
            .query_map([plan_revision_id], map_stage_row)
            .map_err(unavailable("list stage query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("list stage rows"))
    }

    pub fn list_pending_previews(
        &self,
        subject_ref: &str,
    ) -> Result<Vec<PendingPreviewRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT preview_id, subject_kind, subject_ref, status, created_at
                   FROM p11_approval_preview
                  WHERE subject_ref = ?1 AND status = 'pending'
                  ORDER BY created_at",
            )
            .map_err(unavailable("list pending previews"))?;
        let rows = statement
            .query_map([subject_ref], |row| {
                Ok(PendingPreviewRow {
                    preview_id: row.get(0)?,
                    subject_kind: row.get(1)?,
                    subject_ref: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(unavailable("pending preview query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("pending preview rows"))
    }

    pub fn preview_detail(
        &self,
        preview_id: &str,
    ) -> Result<Option<PreviewDetailRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT preview_id, subject_kind, base_state_digest, preview_digest, preview_bytes_ref,
                    status, receipt_ref, superseded_by
               FROM p11_approval_preview WHERE preview_id = ?1",
            [preview_id],
            |row| {
                Ok(PreviewDetailRow {
                    preview_id: row.get(0)?,
                    subject_kind: row.get(1)?,
                    base_state_digest: row.get(2)?,
                    preview_digest: row.get(3)?,
                    preview_bytes_ref: row.get(4)?,
                    status: row.get(5)?,
                    receipt_ref: row.get(6)?,
                    superseded_by: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(unavailable("preview detail"))
    }

    /// Time-boxed 「本周同一类对外不再问」. `expires_at` is required and must be
    /// strictly after `now_ms` and at most 7 days later. Chat/task cannot mint.
    pub fn create_standing_policy(
        &self,
        caller: ConfirmCaller,
        subject_class: &str,
        subject_ref: &str,
        expires_at: Option<i64>,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        Self::require_owner(caller)?;
        Self::reject_secret_shape(subject_class.as_bytes())?;
        Self::reject_secret_shape(subject_ref.as_bytes())?;
        if subject_class.is_empty() || subject_ref.is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "subject_class and subject_ref required",
            });
        }
        let Some(expires_at) = expires_at else {
            return Err(ProjectAggregateError::Invalid {
                detail: "expires_at required",
            });
        };
        if expires_at <= now_ms {
            return Err(ProjectAggregateError::Invalid {
                detail: "expires_at must be in the future",
            });
        }
        if expires_at - now_ms > STANDING_POLICY_MAX_TTL_MS {
            return Err(ProjectAggregateError::Invalid {
                detail: "expires_at exceeds 7-day maximum",
            });
        }
        let conn = self.lock()?;
        let policy_id = next_id("policy")?;
        conn.execute(
            "INSERT INTO p11_standing_approval_policy (
                policy_id, subject_class, subject_ref, expires_at, created_at, revoked_at
             ) VALUES (?1,?2,?3,?4,?5,NULL)",
            params![policy_id, subject_class, subject_ref, expires_at, now_ms],
        )
        .map_err(unavailable("insert standing policy"))?;
        Ok(policy_id)
    }

    /// Settings list: non-revoked policies. `active` is `expires_at > now_ms`.
    pub fn list_standing_policies(
        &self,
        now_ms: i64,
    ) -> Result<Vec<StandingPolicyRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT policy_id, subject_class, subject_ref, expires_at, created_at, revoked_at
                   FROM p11_standing_approval_policy
                  WHERE revoked_at IS NULL
                  ORDER BY created_at",
            )
            .map_err(unavailable("list standing policies"))?;
        let rows = statement
            .query_map([], |row| {
                let expires_at: i64 = row.get(3)?;
                Ok(StandingPolicyRow {
                    policy_id: row.get(0)?,
                    subject_class: row.get(1)?,
                    subject_ref: row.get(2)?,
                    expires_at,
                    created_at: row.get(4)?,
                    revoked_at: row.get(5)?,
                    active: expires_at > now_ms,
                })
            })
            .map_err(unavailable("standing policy query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("standing policy rows"))
    }

    /// Settings revoke. Chat/task cannot revoke. Already-revoked is invalid.
    pub fn revoke_standing_policy(
        &self,
        caller: ConfirmCaller,
        policy_id: &str,
        now_ms: i64,
    ) -> Result<(), ProjectAggregateError> {
        Self::require_owner(caller)?;
        let conn = self.lock()?;
        let revoked_at: Option<i64> = conn
            .query_row(
                "SELECT revoked_at FROM p11_standing_approval_policy WHERE policy_id = ?1",
                [policy_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("load standing policy"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "standing policy not found",
            })?;
        if revoked_at.is_some() {
            return Err(ProjectAggregateError::Invalid {
                detail: "standing policy already revoked",
            });
        }
        let updated = conn
            .execute(
                "UPDATE p11_standing_approval_policy SET revoked_at = ?1 WHERE policy_id = ?2",
                params![now_ms, policy_id],
            )
            .map_err(unavailable("revoke standing policy"))?;
        if updated != 1 {
            return Err(ProjectAggregateError::Unavailable {
                detail: "standing policy revoke did not update one row".to_owned(),
            });
        }
        Ok(())
    }

    pub fn get_draft_seq(&self, draft_id: &str) -> Result<i64, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT base_seq FROM p11_draft WHERE draft_id = ?1",
            [draft_id],
            |row| row.get(0),
        )
        .map_err(|_| ProjectAggregateError::NotFound {
            detail: "draft not found",
        })
    }

    pub fn get_charter(
        &self,
        charter_revision_id: &str,
    ) -> Result<Option<CharterRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT charter_revision_id, project_id, draft_id, seq, content_digest, status, confirmed_at
               FROM p11_charter_revision WHERE charter_revision_id = ?1",
            [charter_revision_id],
            |row| {
                Ok(CharterRow {
                    charter_revision_id: row.get(0)?,
                    project_id: row.get(1)?,
                    draft_id: row.get(2)?,
                    seq: row.get(3)?,
                    content_digest: row.get(4)?,
                    status: row.get(5)?,
                    confirmed_at: row.get(6)?,
                })
            },
        )
        .optional()
        .map_err(unavailable("get charter"))
    }

    /// Honest usage hook (N14/T12): unknown cost serializes as the literal
    /// `unknown`, never as 0. Project-layer Provider binding is unbound today.
    pub fn unknown_cost_projection() -> serde_json::Value {
        crate::provider_control_plane::honest_unknown_cost("project")
    }

    pub fn leak_scan_contains(&self, needle: &str) -> Result<bool, ProjectAggregateError> {
        let conn = self.lock()?;
        ensure_write_facts_locked(&conn)?;
        let tables = [
            "SELECT payload_digest, draft_id FROM p11_draft",
            "SELECT ops_digest, sources_json FROM p11_candidate",
            "SELECT content_digest FROM p11_charter_revision",
            "SELECT title, objective, cadence_json FROM p11_stage",
            "SELECT description FROM p11_gap",
            "SELECT preview_bytes_ref FROM p11_approval_preview",
            "SELECT subject_class, subject_ref FROM p11_standing_approval_policy",
            "SELECT body_redacted FROM p11_conversation_archive",
            "SELECT body_redacted, candidate_json FROM p13_project_chat_turn",
            "SELECT payload_utf8, charter_utf8, title_summary FROM p14_write_project_facts",
        ];
        for sql in tables {
            let mut statement = conn.prepare(sql).map_err(unavailable("leak scan"))?;
            let column_count = statement.column_count();
            let mut rows = statement
                .query([])
                .map_err(unavailable("leak scan query"))?;
            while let Some(row) = rows.next().map_err(unavailable("leak scan next"))? {
                for index in 0..column_count {
                    let value: Option<String> = row.get(index).unwrap_or(None);
                    if value.is_some_and(|text| text.contains(needle)) {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
}

/// Confirm outcome for G1 / plan-change / G2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmResult {
    pub kind: &'static str,
    pub new_ref: String,
    pub receipt_ref: String,
}

/// Owner-narrow outcome: new pending preview plus frozen predecessor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrowResult {
    pub preview_id: String,
    pub preview_digest: String,
    pub superseded_preview_id: String,
}

/// Project authority row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRow {
    pub project_id: String,
    pub state: String,
    pub current_charter_revision_id: String,
    pub current_plan_revision_id: Option<String>,
    pub created_at: i64,
    pub activated_at: i64,
    pub accepted_at: Option<i64>,
    /// Owner-typed Dual Track title after Write Project; otherwise `unknown`.
    pub title_summary: String,
}

/// Stage projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageRow {
    pub plan_revision_id: String,
    pub stage_id: String,
    pub position: i64,
    pub title: String,
    pub objective: String,
    pub confirm_status: String,
    pub stage_digest: String,
    pub ready: bool,
    pub acceptance_spec_ref: Option<String>,
    pub cadence_json: Option<String>,
    pub responsible_slot: String,
    pub output_contract_digest: String,
}

/// Gap projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapRow {
    pub gap_id: String,
    pub description: String,
    pub blocking: bool,
    pub accepted_as_limitation: bool,
}

/// Charter projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharterRow {
    pub charter_revision_id: String,
    pub project_id: Option<String>,
    pub draft_id: String,
    pub seq: i64,
    pub content_digest: String,
    pub status: String,
    pub confirmed_at: Option<i64>,
}

/// Pending preview list row (no digest — 18 §4 / D3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingPreviewRow {
    pub preview_id: String,
    pub subject_kind: String,
    pub subject_ref: String,
    pub status: String,
    pub created_at: i64,
}

/// Canvas preview-detail row (digest present).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewDetailRow {
    pub preview_id: String,
    pub subject_kind: String,
    pub base_state_digest: String,
    pub preview_digest: String,
    pub preview_bytes_ref: String,
    pub status: String,
    pub receipt_ref: Option<String>,
    pub superseded_by: Option<String>,
}

/// Settings-list StandingApprovalPolicy row (time-box; no chat mint).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandingPolicyRow {
    pub policy_id: String,
    pub subject_class: String,
    pub subject_ref: String,
    pub expires_at: i64,
    pub created_at: i64,
    pub revoked_at: Option<i64>,
    pub active: bool,
}

struct PreviewLookup {
    subject_kind: String,
    subject_ref: String,
    base_state_digest: String,
    preview_digest: String,
    status: String,
}

const WRITE_PROJECT_FACTS_SQL: &str = "
CREATE TABLE IF NOT EXISTS p14_write_project_facts (
  draft_id TEXT PRIMARY KEY,
  payload_utf8 TEXT NOT NULL,
  charter_utf8 TEXT NOT NULL DEFAULT '',
  title_summary TEXT NOT NULL DEFAULT 'unknown',
  project_id TEXT
) STRICT;
";

fn ensure_write_facts_locked(conn: &Connection) -> Result<(), ProjectAggregateError> {
    conn.execute_batch(WRITE_PROJECT_FACTS_SQL)
        .map_err(unavailable("ensure write-project facts"))
}

fn utf8_text<'a>(bytes: &'a [u8], what: &'static str) -> Result<&'a str, ProjectAggregateError> {
    std::str::from_utf8(bytes).map_err(|_| ProjectAggregateError::Invalid { detail: what })
}

fn owner_title_from_payload(payload: &str) -> Result<&str, ProjectAggregateError> {
    let title = payload.trim();
    if title.is_empty() {
        return Err(ProjectAggregateError::Invalid {
            detail: "Write Project title must not be empty",
        });
    }
    if title.eq_ignore_ascii_case("unknown") {
        return Err(ProjectAggregateError::Invalid {
            detail: "Write Project title must not be unknown",
        });
    }
    Ok(title)
}

fn charter_declares_process(charter: &str) -> bool {
    charter.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "process:" || trimmed.starts_with("process:")
    })
}

fn parse_dual_track_stages(charter: &str) -> Result<Vec<StageSpec>, ProjectAggregateError> {
    let mut in_process = false;
    let mut stages = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for line in charter.lines() {
        let trimmed = line.trim();
        if !in_process {
            if trimmed == "process:" || trimmed.starts_with("process:") {
                in_process = true;
            }
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with('-') {
            break;
        }
        let item = trimmed.trim_start_matches('-').trim();
        let (name, rest) = match item.split_once(':') {
            Some((name, rest)) => (name.trim(), rest.trim()),
            None => (item, ""),
        };
        if name.is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "Write Project process ring is missing a name",
            });
        }
        let stage_id = name
            .split(|c: char| c == '(' || c.is_whitespace())
            .next()
            .unwrap_or("")
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_owned();
        if stage_id.is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "Write Project process ring is missing a stage id",
            });
        }
        if !seen.insert(stage_id.clone()) {
            return Err(ProjectAggregateError::Invalid {
                detail: "Write Project process rings must have unique ids",
            });
        }
        // Ring id is the PlanRevision seating slot (③). `rights=` is Owner
        // access, not a collapsed "owner" roster slot.
        let responsible_slot = stage_id.clone();
        stages.push(StageSpec {
            stage_id,
            title: name.to_owned(),
            objective: if rest.is_empty() {
                format!("{name} ring")
            } else {
                rest.to_owned()
            },
            output_contract_digest: ProjectAggregateStore::digest_hex(item.as_bytes()),
            acceptance_spec_ref: None,
            cadence_json: Some(r#"{"kind":"manual"}"#.to_owned()),
            responsible_slot,
            blocking_gap: None,
        });
    }
    Ok(stages)
}

fn map_project_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRow> {
    Ok(ProjectRow {
        project_id: row.get(0)?,
        state: row.get(1)?,
        current_charter_revision_id: row.get(2)?,
        current_plan_revision_id: row.get(3)?,
        created_at: row.get(4)?,
        activated_at: row.get(5)?,
        accepted_at: row.get(6)?,
        title_summary: row.get(7)?,
    })
}

fn map_stage_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StageRow> {
    let ready: i64 = row.get(7)?;
    Ok(StageRow {
        plan_revision_id: row.get(0)?,
        stage_id: row.get(1)?,
        position: row.get(2)?,
        title: row.get(3)?,
        objective: row.get(4)?,
        confirm_status: row.get(5)?,
        stage_digest: row.get(6)?,
        ready: ready == 1,
        acceptance_spec_ref: row.get(8)?,
        cadence_json: row.get(9)?,
        responsible_slot: row.get(10)?,
        output_contract_digest: row.get(11)?,
    })
}

fn compute_stage_digest(spec: &StageSpec, position: i64, input_seam: Option<&str>) -> String {
    let canonical = format!(
        "stage_id={}\nposition={}\ntitle={}\nobjective={}\ninput_seam={}\noutput={}\nspec={}\nslot={}\ncadence={}\n",
        spec.stage_id,
        position,
        spec.title,
        spec.objective,
        input_seam.unwrap_or(""),
        spec.output_contract_digest,
        spec.acceptance_spec_ref.as_deref().unwrap_or(""),
        spec.responsible_slot,
        spec.cadence_json.as_deref().unwrap_or("")
    );
    ProjectAggregateStore::digest_hex(canonical.as_bytes())
}

/// Typed assistant provenance: `sources[]` | `owner-stated` | `assistant-assumption`.
/// A non-null blob is not enough.
pub fn validate_assistant_provenance(
    sources_json: Option<&str>,
) -> Result<(), ProjectAggregateError> {
    let Some(raw) = sources_json.filter(|value| !value.trim().is_empty()) else {
        return Err(ProjectAggregateError::Invalid {
            detail: "assistant candidate requires typed provenance",
        });
    };
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|_| ProjectAggregateError::Invalid {
            detail: "assistant provenance must be typed JSON, not an unlabeled blob",
        })?;
    if value.get("confidence").is_some() {
        return Err(ProjectAggregateError::Invalid {
            detail: "forged confidence is rejected; provenance must be typed",
        });
    }
    if let Some(sources) = value.as_array() {
        return validate_sources_array(sources);
    }
    let Some(object) = value.as_object() else {
        return Err(ProjectAggregateError::Invalid {
            detail: "assistant provenance must be a typed object, not an unlabeled blob",
        });
    };
    if object.is_empty() {
        return Err(ProjectAggregateError::Invalid {
            detail: "unlabeled assistant provenance rejected",
        });
    }
    if let Some(kind) = object.get("kind").and_then(serde_json::Value::as_str) {
        return match kind {
            "sources" => object
                .get("sources")
                .and_then(serde_json::Value::as_array)
                .ok_or(ProjectAggregateError::Invalid {
                    detail: "sources provenance requires a non-empty sources array",
                })
                .and_then(|sources| validate_sources_array(sources)),
            "owner-stated" | "assistant-assumption" => {
                if object.len() != 1 {
                    return Err(ProjectAggregateError::Invalid {
                        detail: "unlabeled assistant provenance rejected",
                    });
                }
                Ok(())
            }
            _ => Err(ProjectAggregateError::Invalid {
                detail: "assistant provenance kind must be sources, owner-stated, or assistant-assumption",
            }),
        };
    }
    if let Some(sources) = object.get("sources").and_then(serde_json::Value::as_array)
        && object.len() == 1
    {
        return validate_sources_array(sources);
    }
    Err(ProjectAggregateError::Invalid {
        detail: "unlabeled assistant provenance rejected",
    })
}

fn validate_sources_array(sources: &[serde_json::Value]) -> Result<(), ProjectAggregateError> {
    if sources.is_empty() {
        return Err(ProjectAggregateError::Invalid {
            detail: "sources provenance requires a non-empty sources array",
        });
    }
    for source in sources {
        let uri = source
            .get("uri")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if uri.is_none() {
            return Err(ProjectAggregateError::Invalid {
                detail: "each source requires a non-empty uri",
            });
        }
    }
    Ok(())
}

/// Assistant candidate ops are a closed JSON object: no grant/secret/trigger-arm.
pub fn reject_closed_candidate_schema(ops: &[u8]) -> Result<(), ProjectAggregateError> {
    let value: serde_json::Value =
        serde_json::from_slice(ops).map_err(|_| ProjectAggregateError::Invalid {
            detail: "assistant candidate ops must be closed JSON, not an unlabeled blob",
        })?;
    if json_contains_forbidden_field(&value) {
        return Err(ProjectAggregateError::Invalid {
            detail: "closed candidate schema: grant/secret/trigger-arm fields rejected",
        });
    }
    Ok(())
}

fn json_contains_forbidden_field(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(map) => map.iter().any(|(key, child)| {
            matches!(
                key.as_str(),
                "grant"
                    | "grant_id"
                    | "secret"
                    | "secret_ref"
                    | "trigger-arm"
                    | "trigger_arm"
                    | "api_key"
            ) || json_contains_forbidden_field(child)
        }),
        serde_json::Value::Array(items) => items.iter().any(json_contains_forbidden_field),
        _ => false,
    }
}

fn is_authority_object(conn: &Connection, target: &str) -> Result<bool, ProjectAggregateError> {
    let queries = [
        "SELECT 1 FROM p11_project WHERE project_id = ?1",
        "SELECT 1 FROM p11_employee WHERE employee_id = ?1",
        "SELECT 1 FROM p11_grant WHERE grant_id = ?1",
        "SELECT 1 FROM p11_charter_revision WHERE charter_revision_id = ?1 AND status = 'confirmed'",
        "SELECT 1 FROM p11_acceptance_fact WHERE project_id = ?1",
    ];
    for sql in queries {
        let found: Option<i64> = conn
            .query_row(sql, [target], |row| row.get(0))
            .optional()
            .map_err(unavailable("authority object lookup"))?;
        if found.is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn parse_stage_ref(subject_ref: &str) -> Result<(&str, &str), ProjectAggregateError> {
    subject_ref
        .split_once('#')
        .ok_or(ProjectAggregateError::Invalid {
            detail: "plan-change subject_ref must be project_id#stage_id",
        })
}

struct GrantExpansionSpec {
    project_id: String,
    employee_id: String,
    capability_ref: String,
    scope: String,
}

fn parse_grant_expansion_ref(
    subject_ref: &str,
) -> Result<GrantExpansionSpec, ProjectAggregateError> {
    let value: serde_json::Value =
        serde_json::from_str(subject_ref).map_err(|_| ProjectAggregateError::Invalid {
            detail: "grant-expansion subject_ref must be JSON",
        })?;
    let required = |key: &str| -> Result<String, ProjectAggregateError> {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
            .ok_or(ProjectAggregateError::Invalid {
                detail: "grant-expansion subject_ref missing field",
            })
    };
    Ok(GrantExpansionSpec {
        project_id: required("project_id")?,
        employee_id: required("employee_id")?,
        capability_ref: required("capability_ref")?,
        scope: required("scope")?,
    })
}

fn next_id(prefix: &str) -> Result<String, ProjectAggregateError> {
    let generated = uuid::Uuid::now_v7().as_hyphenated().to_string();
    Ok(format!("{prefix}-{generated}"))
}

fn looks_like_secret(detail: &str) -> bool {
    let lowered = detail.to_ascii_lowercase();
    contains_key_prefix_token(&lowered, "sk-")
        || lowered.contains("bearer ")
        || lowered.contains("api_key")
        || lowered.contains("x-api-key")
        || lowered.contains("ssv1:")
}

/// `sk-` counts as a key prefix only at a token start. Ordinary hyphenated
/// words such as `risk-based`, `task-contract`, or `desk-side` are owner and
/// assistant prose, not Provider material; a real key never has a letter or
/// digit immediately before its prefix.
fn contains_key_prefix_token(lowered: &str, prefix: &str) -> bool {
    lowered.match_indices(prefix).any(|(index, _)| {
        lowered[..index]
            .chars()
            .next_back()
            .is_none_or(|previous| !previous.is_ascii_alphanumeric())
    })
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}
