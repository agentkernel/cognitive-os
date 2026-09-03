//! Daemon-generated reflection candidates and versioned Member Runtime
//! improvement (P13-T11, authority migration v40).
//!
//! Reflection candidates are produced from Attempt / verification / evidence /
//! occurrence facts. A model self-report is never an improvement. A Member
//! Runtime change is a new `p11_employee_revision` inserted only after Owner
//! preview confirm, and is rollback by appending another revision — never by
//! rewriting history or silently upgrading a Blueprint. Running Attempts are
//! not prompt-injected. Nested under `personal_db` so this module does not
//! require a `lib.rs` edit (sibling P13-T10 owns that file).

use crate::employee::EmployeeStore;
use crate::migration::MigrationPlanEntry;
use crate::project_aggregate::{ConfirmCaller, ProjectAggregateError};
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

/// Personal-private reflection envelope. Hidden capability, not chrome.
pub const REFLECTION_PROJECTION_ID: &str = "cognitiveos.personal.reflection/0.1";
/// ApprovalPreview subject kind for a Member Runtime revision.
pub const MEMBER_RUNTIME_SUBJECT_KIND: &str = "member-runtime-revision";
/// ApprovalPreview subject kind for a cross-Project Role Template proposal.
pub const ROLE_TEMPLATE_SUBJECT_KIND: &str = "role-template-proposal";

/// Authority migration v40: reflection candidates, runtime-improvement
/// proposals, Role Template proposals, plus the two new ApprovalPreview
/// subject kinds (table rebuild, v39 precedent).
pub const REFLECTION_SCHEMA_V40: &str = "
CREATE TABLE p13_reflection_candidate (
  candidate_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  employee_id TEXT NOT NULL REFERENCES p11_employee(employee_id),
  kind TEXT NOT NULL CHECK (kind IN ('key-result','daily','cycle','incident')),
  source TEXT NOT NULL CHECK (source IN (
    'attempt-terminal','verification-evidence','occurrence-ledger'
  )),
  attempt_id TEXT,
  evidence_id TEXT,
  occurrence_id TEXT,
  fact_digest TEXT NOT NULL CHECK (length(fact_digest) = 64),
  body_json TEXT NOT NULL,
  completion_claimed INTEGER NOT NULL CHECK (completion_claimed = 0),
  model_self_report INTEGER NOT NULL CHECK (model_self_report = 0),
  created_at INTEGER NOT NULL,
  UNIQUE(project_id, kind, fact_digest)
) STRICT;
CREATE INDEX p13_reflection_candidate_scope
  ON p13_reflection_candidate(project_id, created_at);
CREATE TABLE p13_runtime_improvement (
  improvement_id TEXT PRIMARY KEY,
  candidate_id TEXT NOT NULL REFERENCES p13_reflection_candidate(candidate_id),
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  employee_id TEXT NOT NULL REFERENCES p11_employee(employee_id),
  base_revision_id TEXT NOT NULL,
  proposed_prompt TEXT NOT NULL,
  proposed_tools_json TEXT NOT NULL,
  proposed_digest TEXT NOT NULL CHECK (length(proposed_digest) = 64),
  applied_revision_id TEXT,
  preview_id TEXT NOT NULL,
  preview_digest TEXT NOT NULL CHECK (length(preview_digest) = 64),
  state TEXT NOT NULL CHECK (state IN ('preview','active','rolled-back','rejected')),
  implicit_blueprint INTEGER NOT NULL CHECK (implicit_blueprint = 0),
  created_at INTEGER NOT NULL,
  applied_at INTEGER,
  rolled_back_at INTEGER
) STRICT;
CREATE TABLE p13_role_template_proposal (
  proposal_id TEXT PRIMARY KEY,
  source_project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  source_employee_id TEXT NOT NULL REFERENCES p11_employee(employee_id),
  recipe_digest TEXT NOT NULL CHECK (length(recipe_digest) = 64),
  preview_id TEXT NOT NULL,
  preview_digest TEXT NOT NULL CHECK (length(preview_digest) = 64),
  state TEXT NOT NULL CHECK (state IN ('preview','confirmed','rejected')),
  silent_reuse INTEGER NOT NULL CHECK (silent_reuse = 0),
  created_at INTEGER NOT NULL,
  confirmed_at INTEGER
) STRICT;
CREATE TABLE p11_approval_preview_v40 (
  preview_id TEXT PRIMARY KEY,
  subject_kind TEXT NOT NULL CHECK (subject_kind IN (
    'activation','plan-change','acceptance','grant-expansion','run-acceptance','external-send',
    'plan-revision','task' || '-' || 'revision',
    'member' || '-' || 'runtime' || '-' || 'revision',
    'role' || '-' || 'template' || '-' || 'proposal'
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
INSERT INTO p11_approval_preview_v40 (
  preview_id, subject_kind, subject_ref, base_state_digest, preview_bytes_ref,
  preview_digest, status, intent_id, receipt_ref, created_at, decided_at, superseded_by
) SELECT
  preview_id, subject_kind, subject_ref, base_state_digest, preview_bytes_ref,
  preview_digest, status, intent_id, receipt_ref, created_at, decided_at, superseded_by
FROM p11_approval_preview;
DROP TABLE p11_approval_preview;
ALTER TABLE p11_approval_preview_v40 RENAME TO p11_approval_preview;
";

/// v40 migration entry.
pub fn reflection_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(40, REFLECTION_SCHEMA_V40)
}

/// Durable reflection / runtime-improvement store on the authority writer.
#[derive(Clone)]
pub struct ReflectionStore {
    conn: Arc<Mutex<Connection>>,
    employees: EmployeeStore,
}

/// One daemon-generated reflection candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectionCandidateRow {
    pub candidate_id: String,
    pub project_id: String,
    pub employee_id: String,
    pub kind: String,
    pub source: String,
    pub attempt_id: Option<String>,
    pub evidence_id: Option<String>,
    pub fact_digest: String,
    pub completion_claimed: bool,
}

/// One versioned Member Runtime improvement proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeImprovementRow {
    pub improvement_id: String,
    pub candidate_id: String,
    pub employee_id: String,
    pub base_revision_id: String,
    pub applied_revision_id: Option<String>,
    pub preview_id: String,
    pub preview_digest: String,
    pub state: String,
}

/// Fields for one reflection-candidate insert. Bundled so
/// `insert_candidate_locked` stays under clippy's argument limit.
struct CandidateDraft<'a> {
    project_id: &'a str,
    employee_id: &'a str,
    kind: &'a str,
    source: &'a str,
    attempt_id: Option<&'a str>,
    evidence_id: Option<&'a str>,
    occurrence_id: Option<&'a str>,
    fact: &'a str,
    now_ms: i64,
}

/// Proposal inputs. `new_blueprint_revision_id` is accepted only so the
/// implicit-upgrade refusal is a real caller path, not a missing field.
pub struct RuntimeImprovementSpec<'a> {
    pub candidate_id: &'a str,
    pub proposed_prompt: &'a str,
    pub proposed_tools: &'a [String],
    pub new_blueprint_revision_id: Option<&'a str>,
    pub now_ms: i64,
}

impl ReflectionStore {
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

    fn digest_hex(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn reject_secret_shape(text: &str) -> Result<(), ProjectAggregateError> {
        let lowered = text.to_ascii_lowercase();
        if lowered.contains("bearer ")
            || lowered.contains("ssv1:")
            || lowered.contains("api_key")
            || lowered.contains("x-api-key")
            || lowered.match_indices("sk-").any(|(index, _)| {
                lowered[..index]
                    .chars()
                    .next_back()
                    .is_none_or(|previous| !previous.is_ascii_alphanumeric())
            })
        {
            return Err(ProjectAggregateError::Invalid {
                detail: "secret-shaped material is rejected before a reflection row exists",
            });
        }
        Ok(())
    }

    /// A model-authored self-report is never admitted as an improvement.
    pub fn admit_model_self_report(
        &self,
        _project_id: &str,
        _body: &str,
    ) -> Result<(), ProjectAggregateError> {
        Err(ProjectAggregateError::Rejected {
            detail: "model self-report is not a Member Runtime improvement",
        })
    }

    /// Reflection never completes an Attempt or a run.
    pub fn claim_reflection_is_completion(
        &self,
        _candidate_id: &str,
    ) -> Result<(), ProjectAggregateError> {
        Err(ProjectAggregateError::Rejected {
            detail: "reflection is never completion",
        })
    }

    /// Members are not shared across Projects. Template reuse is the only
    /// cross-Project reuse, and it still needs Owner preview.
    pub fn reuse_member_in_other_project(
        &self,
        employee_id: &str,
        other_project_id: &str,
    ) -> Result<(), ProjectAggregateError> {
        self.employees
            .reuse_employee_in_project(employee_id, other_project_id)
    }

    /// Refuse every write of prompt / context onto an Attempt. Safe-point
    /// instruction revisions are P13-T05; this plane never injects.
    pub fn overwrite_running_attempt_context(
        &self,
        attempt_id: &str,
        new_prompt: &str,
    ) -> Result<(), ProjectAggregateError> {
        Self::reject_secret_shape(new_prompt)?;
        let conn = self.lock()?;
        let state: Option<String> = conn
            .query_row(
                "SELECT state FROM p13_hosted_dsh_attempt WHERE attempt_id = ?1",
                [attempt_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("attempt for injection"))?;
        match state.as_deref() {
            Some("persisted" | "dispatched") => Err(ProjectAggregateError::Rejected {
                detail: "silent prompt injection into a running Attempt is refused",
            }),
            Some(_) => Err(ProjectAggregateError::Rejected {
                detail: "reflection cannot rewrite Attempt context",
            }),
            None => Err(ProjectAggregateError::NotFound {
                detail: "attempt not found",
            }),
        }
    }

    /// Generate candidates from durable facts. Caller-supplied prose is not a
    /// source. Idempotent on `(project, kind, fact_digest)`.
    pub fn generate_from_facts(
        &self,
        project_id: &str,
        now_ms: i64,
    ) -> Result<Vec<ReflectionCandidateRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        let project_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_project WHERE project_id = ?1",
                [project_id],
                |row| row.get(0),
            )
            .map_err(unavailable("project for reflection"))?;
        if project_exists == 0 {
            return Err(ProjectAggregateError::NotFound {
                detail: "project not found",
            });
        }
        let mut inserted = Vec::new();
        inserted.extend(generate_from_evidence_locked(&conn, project_id, now_ms)?);
        inserted.extend(generate_from_terminals_locked(&conn, project_id, now_ms)?);
        inserted.extend(generate_from_occurrences_locked(&conn, project_id, now_ms)?);
        inserted.extend(generate_from_daily_locked(&conn, project_id, now_ms)?);
        Ok(inserted)
    }

    /// List candidates for a Project, oldest first.
    pub fn list_candidates(
        &self,
        project_id: &str,
    ) -> Result<Vec<ReflectionCandidateRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT candidate_id, project_id, employee_id, kind, source, attempt_id,
                        evidence_id, fact_digest, completion_claimed
                   FROM p13_reflection_candidate
                  WHERE project_id = ?1
                  ORDER BY created_at ASC",
            )
            .map_err(unavailable("list reflection candidates"))?;
        let rows = statement
            .query_map([project_id], map_candidate_row)
            .map_err(unavailable("reflection candidate query"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("reflection candidate rows"))?;
        Ok(rows)
    }

    /// Propose a new Member Runtime revision. Inserts no `p11_employee_revision`
    /// until Owner confirm. Implicit Blueprint upgrade is refused.
    pub fn propose_runtime_improvement(
        &self,
        caller: ConfirmCaller,
        spec: &RuntimeImprovementSpec<'_>,
    ) -> Result<RuntimeImprovementRow, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if spec.new_blueprint_revision_id.is_some() {
            return Err(ProjectAggregateError::Rejected {
                detail: "implicit Blueprint upgrade is refused",
            });
        }
        Self::reject_secret_shape(spec.proposed_prompt)?;
        for tool in spec.proposed_tools {
            Self::reject_secret_shape(tool)?;
        }
        let conn = self.lock()?;
        let (project_id, employee_id): (String, String) = conn
            .query_row(
                "SELECT project_id, employee_id FROM p13_reflection_candidate
                  WHERE candidate_id = ?1",
                [spec.candidate_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("candidate for improvement"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "reflection candidate not found",
            })?;
        let running: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p13_hosted_dsh_attempt
                  WHERE employee_id = ?1 AND state IN ('persisted','dispatched')",
                [&employee_id],
                |row| row.get(0),
            )
            .map_err(unavailable("running attempts for improvement"))?;
        if running > 0 {
            return Err(ProjectAggregateError::Rejected {
                detail: "silent prompt injection into a running Attempt is refused",
            });
        }
        let base_revision_id: String = conn
            .query_row(
                "SELECT employee_revision_id FROM p11_employee_revision
                  WHERE employee_id = ?1 ORDER BY seq DESC LIMIT 1",
                [&employee_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("base employee revision"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "employee revision not found",
            })?;
        let tools_json = serde_json::to_string(spec.proposed_tools).map_err(|_| {
            ProjectAggregateError::Unavailable {
                detail: "serialize proposed tools".to_owned(),
            }
        })?;
        let recipe = json!({
            "prompt": spec.proposed_prompt,
            "tools": spec.proposed_tools,
            "base_revision_id": base_revision_id,
            "candidate_id": spec.candidate_id,
        });
        let recipe_bytes =
            serde_json::to_vec(&recipe).map_err(|_| ProjectAggregateError::Unavailable {
                detail: "serialize improvement recipe".to_owned(),
            })?;
        let proposed_digest = Self::digest_hex(&recipe_bytes);
        let pending: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p13_runtime_improvement
                  WHERE employee_id = ?1 AND state = 'preview'",
                [&employee_id],
                |row| row.get(0),
            )
            .map_err(unavailable("pending improvement count"))?;
        if pending > 0 {
            return Err(ProjectAggregateError::Conflict {
                detail: "pending runtime improvement already exists for employee",
            });
        }
        let (preview_id, preview_digest) = mint_preview_locked(
            &conn,
            MEMBER_RUNTIME_SUBJECT_KIND,
            spec.candidate_id,
            &proposed_digest,
            &recipe_bytes,
            spec.now_ms,
        )?;
        let improvement_id = next_id("improve");
        conn.execute(
            "INSERT INTO p13_runtime_improvement (
                improvement_id, candidate_id, project_id, employee_id, base_revision_id,
                proposed_prompt, proposed_tools_json, proposed_digest, applied_revision_id,
                preview_id, preview_digest, state, implicit_blueprint, created_at,
                applied_at, rolled_back_at
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,NULL,?9,?10,'preview',0,?11,NULL,NULL)",
            params![
                improvement_id,
                spec.candidate_id,
                project_id,
                employee_id,
                base_revision_id,
                spec.proposed_prompt,
                tools_json,
                proposed_digest,
                preview_id,
                preview_digest,
                spec.now_ms
            ],
        )
        .map_err(unavailable("insert runtime improvement"))?;
        Ok(RuntimeImprovementRow {
            improvement_id,
            candidate_id: spec.candidate_id.to_owned(),
            employee_id,
            base_revision_id,
            applied_revision_id: None,
            preview_id,
            preview_digest,
            state: "preview".to_owned(),
        })
    }

    /// Owner confirm: insert the new Employee revision. Latest-revision
    /// readers then see the new seq. History is append-only.
    pub fn confirm_runtime_improvement(
        &self,
        caller: ConfirmCaller,
        preview_id: &str,
        preview_digest: &str,
        now_ms: i64,
    ) -> Result<RuntimeImprovementRow, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        let conn = self.lock()?;
        let (status, stored_digest, subject_kind): (String, String, String) = conn
            .query_row(
                "SELECT status, preview_digest, subject_kind FROM p11_approval_preview
                  WHERE preview_id = ?1",
                [preview_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(unavailable("preview for confirm"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "preview not found",
            })?;
        if subject_kind != MEMBER_RUNTIME_SUBJECT_KIND {
            return Err(ProjectAggregateError::Invalid {
                detail: "preview subject_kind is not a Member Runtime revision",
            });
        }
        if status != "pending" {
            return Err(ProjectAggregateError::Conflict {
                detail: "preview is not pending",
            });
        }
        if stored_digest != preview_digest {
            return Err(ProjectAggregateError::Stale {
                detail: "preview digest mismatch",
            });
        }
        let mut row = load_improvement_by_preview_locked(&conn, preview_id)?;
        if row.state != "preview" {
            return Err(ProjectAggregateError::Conflict {
                detail: "runtime improvement is not pending preview",
            });
        }
        let (prompt, tools_json): (String, String) = conn
            .query_row(
                "SELECT proposed_prompt, proposed_tools_json FROM p13_runtime_improvement
                  WHERE improvement_id = ?1",
                [&row.improvement_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(unavailable("proposed recipe"))?;
        let next_seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) + 1 FROM p11_employee_revision WHERE employee_id = ?1",
                [&row.employee_id],
                |r| r.get(0),
            )
            .map_err(unavailable("next employee revision seq"))?;
        let applied_revision_id = next_id("erev");
        conn.execute(
            "INSERT INTO p11_employee_revision (
                employee_revision_id, employee_id, seq, recipe_digest, prompt_bytes,
                tools_declared_json, created_at
             ) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                applied_revision_id,
                row.employee_id,
                next_seq,
                row_preview_digest_from_prompt(&prompt, &tools_json),
                prompt,
                tools_json,
                now_ms
            ],
        )
        .map_err(unavailable("insert confirmed employee revision"))?;
        conn.execute(
            "UPDATE p13_runtime_improvement
                SET state = 'active', applied_revision_id = ?1, applied_at = ?2
              WHERE improvement_id = ?3",
            params![applied_revision_id, now_ms, row.improvement_id],
        )
        .map_err(unavailable("mark improvement active"))?;
        conn.execute(
            "UPDATE p11_approval_preview
                SET status = 'consumed', decided_at = ?1
              WHERE preview_id = ?2",
            params![now_ms, preview_id],
        )
        .map_err(unavailable("consume runtime preview"))?;
        row.applied_revision_id = Some(applied_revision_id);
        row.state = "active".to_owned();
        Ok(row)
    }

    /// Rollback by appending a new revision that restores the pre-confirm
    /// recipe. Prior revisions stay.
    pub fn rollback_runtime_improvement(
        &self,
        caller: ConfirmCaller,
        improvement_id: &str,
        now_ms: i64,
    ) -> Result<RuntimeImprovementRow, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        let conn = self.lock()?;
        let mut row = load_improvement_locked(&conn, improvement_id)?;
        if row.state != "active" {
            return Err(ProjectAggregateError::Rejected {
                detail: "only an active runtime improvement can be rolled back",
            });
        }
        let (prompt, tools_json): (String, String) = conn
            .query_row(
                "SELECT prompt_bytes, tools_declared_json FROM p11_employee_revision
                  WHERE employee_revision_id = ?1",
                [&row.base_revision_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(unavailable("base revision recipe"))?;
        let next_seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) + 1 FROM p11_employee_revision WHERE employee_id = ?1",
                [&row.employee_id],
                |r| r.get(0),
            )
            .map_err(unavailable("rollback revision seq"))?;
        let rollback_revision_id = next_id("erev");
        conn.execute(
            "INSERT INTO p11_employee_revision (
                employee_revision_id, employee_id, seq, recipe_digest, prompt_bytes,
                tools_declared_json, created_at
             ) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                rollback_revision_id,
                row.employee_id,
                next_seq,
                row_preview_digest_from_prompt(&prompt, &tools_json),
                prompt,
                tools_json,
                now_ms
            ],
        )
        .map_err(unavailable("insert rollback employee revision"))?;
        conn.execute(
            "UPDATE p13_runtime_improvement
                SET state = 'rolled-back', rolled_back_at = ?1
              WHERE improvement_id = ?2",
            params![now_ms, improvement_id],
        )
        .map_err(unavailable("mark improvement rolled-back"))?;
        row.state = "rolled-back".to_owned();
        Ok(row)
    }

    /// Cross-Project Role Template proposal. Never copies the Employee.
    pub fn propose_role_template(
        &self,
        caller: ConfirmCaller,
        source_employee_id: &str,
        now_ms: i64,
    ) -> Result<(String, String), ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        let conn = self.lock()?;
        let (source_project_id, revision_id, prompt, tools_json): (String, String, String, String) =
            conn.query_row(
                "SELECT e.project_id, r.employee_revision_id, r.prompt_bytes, r.tools_declared_json
                       FROM p11_employee e
                       JOIN p11_employee_revision r ON r.employee_id = e.employee_id
                      WHERE e.employee_id = ?1
                      ORDER BY r.seq DESC LIMIT 1",
                [source_employee_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(unavailable("employee for role template"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "employee not found",
            })?;
        let recipe = json!({
            "source_employee_id": source_employee_id,
            "source_revision_id": revision_id,
            "prompt": prompt,
            "tools": tools_json,
            "copies_employee": false,
        });
        let recipe_bytes =
            serde_json::to_vec(&recipe).map_err(|_| ProjectAggregateError::Unavailable {
                detail: "serialize role template recipe".to_owned(),
            })?;
        let recipe_digest = Self::digest_hex(&recipe_bytes);
        let (preview_id, preview_digest) = mint_preview_locked(
            &conn,
            ROLE_TEMPLATE_SUBJECT_KIND,
            source_employee_id,
            &recipe_digest,
            &recipe_bytes,
            now_ms,
        )?;
        let proposal_id = next_id("rtemplate");
        conn.execute(
            "INSERT INTO p13_role_template_proposal (
                proposal_id, source_project_id, source_employee_id, recipe_digest,
                preview_id, preview_digest, state, silent_reuse, created_at, confirmed_at
             ) VALUES (?1,?2,?3,?4,?5,?6,'preview',0,?7,NULL)",
            params![
                proposal_id,
                source_project_id,
                source_employee_id,
                recipe_digest,
                preview_id,
                preview_digest,
                now_ms
            ],
        )
        .map_err(unavailable("insert role template proposal"))?;
        Ok((proposal_id, preview_id))
    }

    /// Confirm a Role Template proposal. Does not create or copy an Employee
    /// into another Project.
    pub fn confirm_role_template(
        &self,
        caller: ConfirmCaller,
        proposal_id: &str,
        now_ms: i64,
    ) -> Result<(), ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        let conn = self.lock()?;
        let (state, preview_id, silent_reuse): (String, String, i64) = conn
            .query_row(
                "SELECT state, preview_id, silent_reuse FROM p13_role_template_proposal
                  WHERE proposal_id = ?1",
                [proposal_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(unavailable("role template proposal"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "role template proposal not found",
            })?;
        if silent_reuse != 0 {
            return Err(ProjectAggregateError::Rejected {
                detail: "silent Role Template reuse is unrepresentable",
            });
        }
        if state != "preview" {
            return Err(ProjectAggregateError::Conflict {
                detail: "role template proposal is not pending preview",
            });
        }
        conn.execute(
            "UPDATE p13_role_template_proposal
                SET state = 'confirmed', confirmed_at = ?1
              WHERE proposal_id = ?2",
            params![now_ms, proposal_id],
        )
        .map_err(unavailable("confirm role template"))?;
        conn.execute(
            "UPDATE p11_approval_preview
                SET status = 'consumed', decided_at = ?1
              WHERE preview_id = ?2",
            params![now_ms, preview_id],
        )
        .map_err(unavailable("consume role template preview"))?;
        Ok(())
    }

    /// Latest applied (or rolled-back) improvement for tests / HTTP later.
    pub fn get_improvement(
        &self,
        improvement_id: &str,
    ) -> Result<Option<RuntimeImprovementRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        match load_improvement_locked(&conn, improvement_id) {
            Ok(row) => Ok(Some(row)),
            Err(ProjectAggregateError::NotFound { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

fn generate_from_evidence_locked(
    conn: &Connection,
    project_id: &str,
    now_ms: i64,
) -> Result<Vec<ReflectionCandidateRow>, ProjectAggregateError> {
    let mut statement = conn
        .prepare(
            "SELECT e.evidence_id, e.disposition, a.attempt_id, a.employee_id
               FROM p13_artifact_evidence e
               JOIN p13_attempt_artifact art ON art.artifact_id = e.artifact_id
               JOIN p13_hosted_dsh_attempt a ON a.attempt_id = art.attempt_id
              WHERE a.project_id = ?1 AND e.disposition = 'passed'",
        )
        .map_err(unavailable("evidence facts for reflection"))?;
    let facts: Vec<(String, String, String, String)> = statement
        .query_map([project_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(unavailable("evidence fact query"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(unavailable("evidence fact rows"))?;
    drop(statement);
    let mut inserted = Vec::new();
    for (evidence_id, disposition, attempt_id, employee_id) in facts {
        let fact =
            format!("evidence={evidence_id}\nattempt={attempt_id}\ndisposition={disposition}");
        if let Some(row) = insert_candidate_locked(
            conn,
            &CandidateDraft {
                project_id,
                employee_id: &employee_id,
                kind: "key-result",
                source: "verification-evidence",
                attempt_id: Some(&attempt_id),
                evidence_id: Some(&evidence_id),
                occurrence_id: None,
                fact: &fact,
                now_ms,
            },
        )? {
            inserted.push(row);
        }
    }
    Ok(inserted)
}

fn generate_from_terminals_locked(
    conn: &Connection,
    project_id: &str,
    now_ms: i64,
) -> Result<Vec<ReflectionCandidateRow>, ProjectAggregateError> {
    let mut statement = conn
        .prepare(
            "SELECT attempt_id, employee_id, terminal_kind, response_status
               FROM p13_hosted_dsh_attempt
              WHERE project_id = ?1 AND state IN ('terminal','unknown-outcome')
                AND (terminal_kind IN (
                    'signaled','timed-out','spawn-failed','unknown-outcome'
                  ) OR response_status IN ('failed','blocked','unknown'))",
        )
        .map_err(unavailable("terminal facts for reflection"))?;
    let facts: Vec<(String, String, String, String)> = statement
        .query_map([project_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(unavailable("terminal fact query"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(unavailable("terminal fact rows"))?;
    drop(statement);
    let mut inserted = Vec::new();
    for (attempt_id, employee_id, terminal_kind, response_status) in facts {
        let fact =
            format!("attempt={attempt_id}\nterminal={terminal_kind}\nresponse={response_status}");
        if let Some(row) = insert_candidate_locked(
            conn,
            &CandidateDraft {
                project_id,
                employee_id: &employee_id,
                kind: "incident",
                source: "attempt-terminal",
                attempt_id: Some(&attempt_id),
                evidence_id: None,
                occurrence_id: None,
                fact: &fact,
                now_ms,
            },
        )? {
            inserted.push(row);
        }
    }
    Ok(inserted)
}

fn generate_from_occurrences_locked(
    conn: &Connection,
    project_id: &str,
    now_ms: i64,
) -> Result<Vec<ReflectionCandidateRow>, ProjectAggregateError> {
    let mut statement = conn
        .prepare(
            "SELECT o.occurrence_id, o.attempt_id, a.employee_id, o.attempt_outcome
               FROM p11_routine_occurrence o
               JOIN p13_hosted_dsh_attempt a ON a.attempt_id = o.attempt_id
              WHERE o.project_id = ?1 AND o.disposition = 'attempted'
                AND o.attempt_id IS NOT NULL",
        )
        .map_err(unavailable("occurrence facts for reflection"))?;
    let facts: Vec<(String, Option<String>, String, Option<String>)> = statement
        .query_map([project_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(unavailable("occurrence fact query"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(unavailable("occurrence fact rows"))?;
    drop(statement);
    let mut inserted = Vec::new();
    for (occurrence_id, attempt_id, employee_id, outcome) in facts {
        let fact = format!(
            "occurrence={occurrence_id}\nattempt={}\noutcome={}",
            attempt_id.as_deref().unwrap_or(""),
            outcome.as_deref().unwrap_or("")
        );
        if let Some(row) = insert_candidate_locked(
            conn,
            &CandidateDraft {
                project_id,
                employee_id: &employee_id,
                kind: "cycle",
                source: "occurrence-ledger",
                attempt_id: attempt_id.as_deref(),
                evidence_id: None,
                occurrence_id: Some(&occurrence_id),
                fact: &fact,
                now_ms,
            },
        )? {
            inserted.push(row);
        }
    }
    Ok(inserted)
}

/// One `daily` candidate per seated Member per UTC day that has a terminal
/// Attempt. This is a rollup, not a key-result: `response done` / exit 0
/// without evidence still yields `daily` and still must not become
/// `key-result`.
fn generate_from_daily_locked(
    conn: &Connection,
    project_id: &str,
    now_ms: i64,
) -> Result<Vec<ReflectionCandidateRow>, ProjectAggregateError> {
    let mut statement = conn
        .prepare(
            "SELECT employee_id, date(terminal_at / 1000, 'unixepoch'), COUNT(*)
               FROM p13_hosted_dsh_attempt
              WHERE project_id = ?1
                AND state IN ('terminal','unknown-outcome')
                AND terminal_at IS NOT NULL
              GROUP BY employee_id, date(terminal_at / 1000, 'unixepoch')",
        )
        .map_err(unavailable("daily facts for reflection"))?;
    let facts: Vec<(String, String, i64)> = statement
        .query_map([project_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(unavailable("daily fact query"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(unavailable("daily fact rows"))?;
    drop(statement);
    let mut inserted = Vec::new();
    for (employee_id, day, count) in facts {
        let fact = format!("daily={day}\nemployee={employee_id}\nterminals={count}");
        if let Some(row) = insert_candidate_locked(
            conn,
            &CandidateDraft {
                project_id,
                employee_id: &employee_id,
                kind: "daily",
                source: "attempt-terminal",
                attempt_id: None,
                evidence_id: None,
                occurrence_id: None,
                fact: &fact,
                now_ms,
            },
        )? {
            inserted.push(row);
        }
    }
    Ok(inserted)
}

fn insert_candidate_locked(
    conn: &Connection,
    draft: &CandidateDraft<'_>,
) -> Result<Option<ReflectionCandidateRow>, ProjectAggregateError> {
    let fact_digest = format!("{:x}", Sha256::digest(draft.fact.as_bytes()));
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM p13_reflection_candidate
              WHERE project_id = ?1 AND kind = ?2 AND fact_digest = ?3",
            params![draft.project_id, draft.kind, fact_digest],
            |row| row.get(0),
        )
        .map_err(unavailable("existing reflection candidate"))?;
    if exists > 0 {
        return Ok(None);
    }
    let body = json!({
        "kind": draft.kind,
        "source": draft.source,
        "fact": draft.fact,
        "completion_claimed": false,
        "model_self_report": false,
    })
    .to_string();
    let candidate_id = next_id("reflect");
    conn.execute(
        "INSERT INTO p13_reflection_candidate (
            candidate_id, project_id, employee_id, kind, source, attempt_id,
            evidence_id, occurrence_id, fact_digest, body_json, completion_claimed,
            model_self_report, created_at
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,0,0,?11)",
        params![
            candidate_id,
            draft.project_id,
            draft.employee_id,
            draft.kind,
            draft.source,
            draft.attempt_id,
            draft.evidence_id,
            draft.occurrence_id,
            fact_digest,
            body,
            draft.now_ms
        ],
    )
    .map_err(unavailable("insert reflection candidate"))?;
    Ok(Some(ReflectionCandidateRow {
        candidate_id,
        project_id: draft.project_id.to_owned(),
        employee_id: draft.employee_id.to_owned(),
        kind: draft.kind.to_owned(),
        source: draft.source.to_owned(),
        attempt_id: draft.attempt_id.map(str::to_owned),
        evidence_id: draft.evidence_id.map(str::to_owned),
        fact_digest,
        completion_claimed: false,
    }))
}

fn mint_preview_locked(
    conn: &Connection,
    subject_kind: &str,
    subject_ref: &str,
    base_state_digest: &str,
    preview_bytes: &[u8],
    now_ms: i64,
) -> Result<(String, String), ProjectAggregateError> {
    let preview_id = next_id("preview");
    let preview_bytes_ref = format!("cas:{:x}", Sha256::digest(preview_bytes));
    let preview_digest = format!(
        "{:x}",
        Sha256::digest(
            format!("{base_state_digest}\n{preview_bytes_ref}\n{subject_kind}\n{subject_ref}")
                .as_bytes()
        )
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
    .map_err(unavailable("insert reflection preview"))?;
    Ok((preview_id, preview_digest))
}

fn load_improvement_by_preview_locked(
    conn: &Connection,
    preview_id: &str,
) -> Result<RuntimeImprovementRow, ProjectAggregateError> {
    conn.query_row(
        "SELECT improvement_id, candidate_id, employee_id, base_revision_id,
                applied_revision_id, preview_id, preview_digest, state
           FROM p13_runtime_improvement WHERE preview_id = ?1",
        [preview_id],
        map_improvement_row,
    )
    .optional()
    .map_err(unavailable("improvement by preview"))?
    .ok_or(ProjectAggregateError::NotFound {
        detail: "runtime improvement not found",
    })
}

fn load_improvement_locked(
    conn: &Connection,
    improvement_id: &str,
) -> Result<RuntimeImprovementRow, ProjectAggregateError> {
    conn.query_row(
        "SELECT improvement_id, candidate_id, employee_id, base_revision_id,
                applied_revision_id, preview_id, preview_digest, state
           FROM p13_runtime_improvement WHERE improvement_id = ?1",
        [improvement_id],
        map_improvement_row,
    )
    .optional()
    .map_err(unavailable("load runtime improvement"))?
    .ok_or(ProjectAggregateError::NotFound {
        detail: "runtime improvement not found",
    })
}

fn map_candidate_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReflectionCandidateRow> {
    let claimed: i64 = row.get(8)?;
    Ok(ReflectionCandidateRow {
        candidate_id: row.get(0)?,
        project_id: row.get(1)?,
        employee_id: row.get(2)?,
        kind: row.get(3)?,
        source: row.get(4)?,
        attempt_id: row.get(5)?,
        evidence_id: row.get(6)?,
        fact_digest: row.get(7)?,
        completion_claimed: claimed != 0,
    })
}

fn map_improvement_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RuntimeImprovementRow> {
    Ok(RuntimeImprovementRow {
        improvement_id: row.get(0)?,
        candidate_id: row.get(1)?,
        employee_id: row.get(2)?,
        base_revision_id: row.get(3)?,
        applied_revision_id: row.get(4)?,
        preview_id: row.get(5)?,
        preview_digest: row.get(6)?,
        state: row.get(7)?,
    })
}

fn row_preview_digest_from_prompt(prompt: &str, tools_json: &str) -> String {
    format!(
        "{:x}",
        Sha256::digest(format!("prompt={prompt}\ntools={tools_json}").as_bytes())
    )
}

fn next_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::now_v7().as_hyphenated())
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}

#[cfg(test)]
mod schema_tests {
    #[test]
    fn v40_check_sql_omits_sk_substring() {
        assert!(
            !super::REFLECTION_SCHEMA_V40.contains("sk-"),
            "v40 CHECK SQL must not persist the sk- byte sequence into sqlite_master"
        );
    }
}
