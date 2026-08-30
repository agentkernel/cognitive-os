//! Personal-private Employee / Role Blueprint / Assignment (P11-T04, v27).
//!
//! Authoritative id = Employee. Chrome may say Member Runtime. This is not a
//! Role=Agent merge, not a process id, and not a third identity besides the
//! existing Agent/adapter string. Blueprint rows have no Provider binding.

use crate::project_aggregate::{ConfirmCaller, ProjectAggregateError};
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

/// Built-in Project Manager blueprint (14 §4.2).
pub const PROJECT_MANAGER_BLUEPRINT_ID: &str = "role-blueprint:project-manager";
/// Built-in member blueprint.
pub const MEMBER_BLUEPRINT_ID: &str = "role-blueprint:member";

/// Authority migration v27: Employee / Blueprint / Assignment / Grant.
pub const EMPLOYEE_SCHEMA_V27: &str = "
CREATE TABLE p11_role_blueprint (
  blueprint_id TEXT PRIMARY KEY,
  specialization TEXT NOT NULL CHECK (specialization IN ('project-manager','member')),
  created_at INTEGER NOT NULL
) STRICT;
CREATE TABLE p11_role_blueprint_revision (
  blueprint_revision_id TEXT PRIMARY KEY,
  blueprint_id TEXT NOT NULL REFERENCES p11_role_blueprint(blueprint_id),
  seq INTEGER NOT NULL CHECK (seq >= 1),
  recipe_digest TEXT NOT NULL CHECK (length(recipe_digest) = 64),
  recipe_bytes_ref TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  UNIQUE(blueprint_id, seq)
) STRICT;
CREATE TABLE p11_employee (
  employee_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  blueprint_revision_id TEXT NOT NULL REFERENCES p11_role_blueprint_revision(blueprint_revision_id),
  responsible_stage_ids_json TEXT NOT NULL,
  provider_model_binding TEXT,
  runtime_binding_ref TEXT,
  state TEXT NOT NULL CHECK (state IN (
    'proposed','seating','seated','pending','refused','suspended','removed'
  )),
  is_current_manager INTEGER NOT NULL CHECK (is_current_manager IN (0,1)),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
) STRICT;
CREATE UNIQUE INDEX p11_one_current_manager
  ON p11_employee(project_id) WHERE is_current_manager = 1;
CREATE UNIQUE INDEX p11_one_seating
  ON p11_employee(project_id) WHERE state = 'seating';
CREATE TABLE p11_employee_revision (
  employee_revision_id TEXT PRIMARY KEY,
  employee_id TEXT NOT NULL REFERENCES p11_employee(employee_id),
  seq INTEGER NOT NULL CHECK (seq >= 1),
  recipe_digest TEXT NOT NULL CHECK (length(recipe_digest) = 64),
  prompt_bytes TEXT NOT NULL,
  tools_declared_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  UNIQUE(employee_id, seq)
) STRICT;
CREATE TABLE p11_assignment (
  assignment_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  plan_revision_id TEXT NOT NULL,
  slot TEXT NOT NULL,
  employee_id TEXT NOT NULL REFERENCES p11_employee(employee_id),
  UNIQUE(project_id, plan_revision_id, slot)
) STRICT;
CREATE TABLE p11_install_fact (
  install_id TEXT PRIMARY KEY,
  capability_ref TEXT NOT NULL,
  version_pin TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  UNIQUE(capability_ref, version_pin)
) STRICT;
CREATE TABLE p11_grant (
  grant_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  employee_id TEXT NOT NULL REFERENCES p11_employee(employee_id),
  capability_ref TEXT NOT NULL,
  scope TEXT NOT NULL,
  created_at INTEGER NOT NULL
) STRICT;
CREATE TABLE p11_speech_audit (
  audit_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  employee_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  mentioned INTEGER NOT NULL CHECK (mentioned IN (0,1)),
  delivered INTEGER NOT NULL CHECK (delivered IN (0,1)),
  reason TEXT NOT NULL,
  created_at INTEGER NOT NULL
) STRICT;
CREATE TABLE p11_handoff (
  handoff_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  source_employee_id TEXT NOT NULL,
  target_employee_id TEXT NOT NULL,
  bounded_work_digest TEXT NOT NULL,
  blocked_or_ready TEXT NOT NULL CHECK (blocked_or_ready IN ('blocked','ready')),
  authority_stays INTEGER NOT NULL CHECK (authority_stays = 1),
  created_at INTEGER NOT NULL
) STRICT;
";

/// v27 migration entry.
pub fn employee_migration_entry() -> crate::migration::MigrationPlanEntry {
    crate::migration::MigrationPlanEntry::new(27, EMPLOYEE_SCHEMA_V27)
}

/// One slot → Employee proposal for roster registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RosterProposal {
    pub slot: String,
    pub specialization: String,
    pub prompt: String,
    pub tools_declared: Vec<String>,
}

/// Durable Employee row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmployeeRow {
    pub employee_id: String,
    pub project_id: String,
    pub blueprint_revision_id: String,
    pub responsible_stage_ids_json: String,
    pub provider_model_binding: Option<String>,
    pub runtime_binding_ref: Option<String>,
    pub state: String,
    pub is_current_manager: bool,
}

/// Seating progress: numerator is committed seated facts only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeatingProgress {
    pub seated: i64,
    pub roster: i64,
}

/// Speech router decision. Durable archive landing is ConversationStore (P11-T05).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeechDecision {
    pub audit_id: String,
    pub delivered: bool,
    pub reason: String,
}

/// Bounded handoff row. Schema forces `authority_stays = 1`; chat cannot transfer authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffSpec<'a> {
    pub project_id: &'a str,
    pub source_employee_id: &'a str,
    pub target_employee_id: &'a str,
    pub bounded_work_digest: &'a str,
    pub blocked_or_ready: &'a str,
    pub now_ms: i64,
}

/// Durable Employee / Blueprint / Grant store on the authority writer.
#[derive(Clone)]
pub struct EmployeeStore {
    conn: Arc<Mutex<Connection>>,
}

impl EmployeeStore {
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

    pub(crate) fn conn_arc(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    pub(crate) fn require_owner(caller: ConfirmCaller) -> Result<(), ProjectAggregateError> {
        match caller {
            ConfirmCaller::OwnerManagement => Ok(()),
            ConfirmCaller::TaskChannel | ConfirmCaller::Assistant => {
                Err(ProjectAggregateError::Forbidden {
                    detail: "only owner management session may confirm or apply",
                })
            }
        }
    }

    fn digest_hex(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn reject_secret_shape(bytes: &[u8]) -> Result<(), ProjectAggregateError> {
        let lowered = String::from_utf8_lossy(bytes).to_ascii_lowercase();
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

    fn reject_blueprint_recipe(bytes: &[u8]) -> Result<Value, ProjectAggregateError> {
        Self::reject_secret_shape(bytes)?;
        let parsed: Value =
            serde_json::from_slice(bytes).map_err(|_| ProjectAggregateError::Invalid {
                detail: "blueprint recipe must be JSON",
            })?;
        let forbidden = [
            "provider",
            "provider_binding",
            "provider_model",
            "grant",
            "secret",
        ];
        if let Some(object) = parsed.as_object() {
            for key in forbidden {
                if object.contains_key(key) {
                    return Err(ProjectAggregateError::Invalid {
                        detail: "blueprint must not carry Provider binding, grant, or secret",
                    });
                }
            }
        }
        Ok(parsed)
    }

    /// Seed built-in manager/member blueprints (idempotent).
    pub fn ensure_builtins(&self, now_ms: i64) -> Result<(), ProjectAggregateError> {
        self.ensure_blueprint(
            PROJECT_MANAGER_BLUEPRINT_ID,
            "project-manager",
            br#"{"duty":"Project Manager","prompt":"coordinate the project","tools":[]}"#,
            now_ms,
        )?;
        self.ensure_blueprint(
            MEMBER_BLUEPRINT_ID,
            "member",
            br#"{"duty":"Member","prompt":"deliver assigned stages","tools":[]}"#,
            now_ms,
        )?;
        Ok(())
    }

    fn ensure_blueprint(
        &self,
        blueprint_id: &str,
        specialization: &str,
        recipe: &[u8],
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        Self::reject_blueprint_recipe(recipe)?;
        let conn = self.lock()?;
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_role_blueprint WHERE blueprint_id = ?1",
                [blueprint_id],
                |row| row.get(0),
            )
            .map_err(unavailable("blueprint exists"))?;
        if exists == 0 {
            conn.execute(
                "INSERT INTO p11_role_blueprint (blueprint_id, specialization, created_at)
                 VALUES (?1,?2,?3)",
                params![blueprint_id, specialization, now_ms],
            )
            .map_err(unavailable("insert blueprint"))?;
            let revision_id = next_id("bprev")?;
            let digest = Self::digest_hex(recipe);
            conn.execute(
                "INSERT INTO p11_role_blueprint_revision (
                    blueprint_revision_id, blueprint_id, seq, recipe_digest, recipe_bytes_ref, created_at
                 ) VALUES (?1,?2,1,?3,?4,?5)",
                params![
                    revision_id,
                    blueprint_id,
                    digest,
                    format!("cas:{digest}"),
                    now_ms
                ],
            )
            .map_err(unavailable("insert blueprint revision"))?;
            return Ok(revision_id);
        }
        conn.query_row(
            "SELECT blueprint_revision_id FROM p11_role_blueprint_revision
              WHERE blueprint_id = ?1 ORDER BY seq DESC LIMIT 1",
            [blueprint_id],
            |row| row.get(0),
        )
        .map_err(unavailable("current blueprint revision"))
    }

    /// Publish a new global Blueprint revision. Existing Employees are not updated.
    pub fn publish_blueprint_revision(
        &self,
        caller: ConfirmCaller,
        blueprint_id: &str,
        recipe: &[u8],
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        Self::require_owner(caller)?;
        Self::reject_blueprint_recipe(recipe)?;
        let conn = self.lock()?;
        let next_seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) + 1 FROM p11_role_blueprint_revision WHERE blueprint_id = ?1",
                [blueprint_id],
                |row| row.get(0),
            )
            .map_err(unavailable("next blueprint seq"))?;
        if next_seq == 1 {
            return Err(ProjectAggregateError::NotFound {
                detail: "blueprint not found",
            });
        }
        let revision_id = next_id("bprev")?;
        let digest = Self::digest_hex(recipe);
        conn.execute(
            "INSERT INTO p11_role_blueprint_revision (
                blueprint_revision_id, blueprint_id, seq, recipe_digest, recipe_bytes_ref, created_at
             ) VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                revision_id,
                blueprint_id,
                next_seq,
                digest,
                format!("cas:{digest}"),
                now_ms
            ],
        )
        .map_err(unavailable("insert blueprint revision"))?;
        Ok(revision_id)
    }

    /// Per-Project opt-in to a newer Blueprint revision.
    pub fn upgrade_employee_blueprint(
        &self,
        caller: ConfirmCaller,
        employee_id: &str,
        new_revision_id: &str,
        opt_in: bool,
        now_ms: i64,
    ) -> Result<(), ProjectAggregateError> {
        Self::require_owner(caller)?;
        if !opt_in {
            return Err(ProjectAggregateError::Rejected {
                detail: "blueprint upgrade requires per-Project opt-in preview",
            });
        }
        let conn = self.lock()?;
        let current: String = conn
            .query_row(
                "SELECT blueprint_revision_id FROM p11_employee WHERE employee_id = ?1",
                [employee_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("employee for upgrade"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "employee not found",
            })?;
        let new_blueprint: String = conn
            .query_row(
                "SELECT blueprint_id FROM p11_role_blueprint_revision WHERE blueprint_revision_id = ?1",
                [new_revision_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("new blueprint revision"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "blueprint revision not found",
            })?;
        let current_blueprint: String = conn
            .query_row(
                "SELECT blueprint_id FROM p11_role_blueprint_revision WHERE blueprint_revision_id = ?1",
                [&current],
                |row| row.get(0),
            )
            .map_err(unavailable("current blueprint id"))?;
        if new_blueprint != current_blueprint {
            return Err(ProjectAggregateError::Invalid {
                detail: "upgrade revision must belong to the same blueprint",
            });
        }
        conn.execute(
            "UPDATE p11_employee SET blueprint_revision_id = ?1, updated_at = ?2 WHERE employee_id = ?3",
            params![new_revision_id, now_ms, employee_id],
        )
        .map_err(unavailable("opt-in upgrade"))?;
        Ok(())
    }

    /// Register a full-coverage roster. Missing or surplus slots fail closed.
    pub fn register_roster(
        &self,
        caller: ConfirmCaller,
        project_id: &str,
        plan_revision_id: &str,
        proposals: &[RosterProposal],
        now_ms: i64,
    ) -> Result<Vec<String>, ProjectAggregateError> {
        Self::require_owner(caller)?;
        self.ensure_builtins(now_ms)?;
        for proposal in proposals {
            if proposal.specialization == "agent" || proposal.slot == "agent" {
                return Err(ProjectAggregateError::Rejected {
                    detail: "Role must not be merged with Agent",
                });
            }
            if proposal.specialization != "project-manager" && proposal.specialization != "member" {
                return Err(ProjectAggregateError::Invalid {
                    detail: "specialization must be project-manager or member",
                });
            }
            Self::reject_secret_shape(proposal.prompt.as_bytes())?;
        }
        let conn = self.lock()?;
        let owner: String = conn
            .query_row(
                "SELECT project_id FROM p11_plan_revision WHERE plan_revision_id = ?1",
                [plan_revision_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("plan owner"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "plan revision not found",
            })?;
        if owner != project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-project write rejected",
            });
        }
        let mut slot_statement = conn
            .prepare("SELECT DISTINCT responsible_slot FROM p11_stage WHERE plan_revision_id = ?1")
            .map_err(unavailable("list slots"))?;
        let axis_slots: Vec<String> = slot_statement
            .query_map([plan_revision_id], |row| row.get(0))
            .map_err(unavailable("slot query"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("slot rows"))?;
        drop(slot_statement);
        if axis_slots.is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "plan has no slots",
            });
        }
        let mut proposal_slots: Vec<String> = proposals.iter().map(|p| p.slot.clone()).collect();
        proposal_slots.sort();
        let mut expected = axis_slots.clone();
        expected.sort();
        for slot in &expected {
            if !proposal_slots.iter().any(|p| p == slot) {
                return Err(ProjectAggregateError::Rejected {
                    detail: "roster missing slot coverage",
                });
            }
        }
        for slot in &proposal_slots {
            if !expected.iter().any(|e| e == slot) {
                return Err(ProjectAggregateError::Rejected {
                    detail: "roster has surplus member without a slot",
                });
            }
        }
        if proposal_slots.len() != expected.len() {
            return Err(ProjectAggregateError::Rejected {
                detail: "roster must map each slot to exactly one proposal",
            });
        }
        let mut created = Vec::new();
        for proposal in proposals {
            let mut stage_statement = conn
                .prepare(
                    "SELECT stage_id FROM p11_stage
                      WHERE plan_revision_id = ?1 AND responsible_slot = ?2",
                )
                .map_err(unavailable("stages for slot"))?;
            let stage_ids: Vec<String> = stage_statement
                .query_map(params![plan_revision_id, proposal.slot], |row| row.get(0))
                .map_err(unavailable("stage id query"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(unavailable("stage id rows"))?;
            drop(stage_statement);
            let blueprint_id = if proposal.specialization == "project-manager" {
                PROJECT_MANAGER_BLUEPRINT_ID
            } else {
                MEMBER_BLUEPRINT_ID
            };
            let blueprint_revision_id: String = conn
                .query_row(
                    "SELECT blueprint_revision_id FROM p11_role_blueprint_revision
                      WHERE blueprint_id = ?1 ORDER BY seq DESC LIMIT 1",
                    [blueprint_id],
                    |row| row.get(0),
                )
                .map_err(unavailable("blueprint revision for proposal"))?;
            let employee_id = next_id("employee")?;
            let stages_json = serde_json::to_string(&stage_ids).map_err(|_| {
                ProjectAggregateError::Unavailable {
                    detail: "serialize stage ids".to_owned(),
                }
            })?;
            conn.execute(
                "INSERT INTO p11_employee (
                    employee_id, project_id, blueprint_revision_id, responsible_stage_ids_json,
                    provider_model_binding, runtime_binding_ref, state, is_current_manager,
                    created_at, updated_at
                 ) VALUES (?1,?2,?3,?4,NULL,NULL,'proposed',0,?5,?5)",
                params![
                    employee_id,
                    project_id,
                    blueprint_revision_id,
                    stages_json,
                    now_ms
                ],
            )
            .map_err(unavailable("insert employee"))?;
            let assignment_id = next_id("assign")?;
            conn.execute(
                "INSERT INTO p11_assignment (
                    assignment_id, project_id, plan_revision_id, slot, employee_id
                 ) VALUES (?1,?2,?3,?4,?5)",
                params![
                    assignment_id,
                    project_id,
                    plan_revision_id,
                    proposal.slot,
                    employee_id
                ],
            )
            .map_err(unavailable("insert assignment"))?;
            let tools_json = serde_json::to_string(&proposal.tools_declared).map_err(|_| {
                ProjectAggregateError::Unavailable {
                    detail: "serialize tools".to_owned(),
                }
            })?;
            let recipe = json!({
                "prompt": proposal.prompt,
                "tools": proposal.tools_declared,
            });
            let recipe_bytes =
                serde_json::to_vec(&recipe).map_err(|_| ProjectAggregateError::Unavailable {
                    detail: "serialize recipe".to_owned(),
                })?;
            let revision_id = next_id("erev")?;
            conn.execute(
                "INSERT INTO p11_employee_revision (
                    employee_revision_id, employee_id, seq, recipe_digest, prompt_bytes,
                    tools_declared_json, created_at
                 ) VALUES (?1,?2,1,?3,?4,?5,?6)",
                params![
                    revision_id,
                    employee_id,
                    Self::digest_hex(&recipe_bytes),
                    proposal.prompt,
                    tools_json,
                    now_ms
                ],
            )
            .map_err(unavailable("insert employee revision"))?;
            created.push(employee_id);
        }
        Ok(created)
    }

    /// Begin sequential seating. At most one `seating` Employee per Project.
    pub fn request_seating(
        &self,
        caller: ConfirmCaller,
        employee_id: &str,
        now_ms: i64,
    ) -> Result<(), ProjectAggregateError> {
        Self::require_owner(caller)?;
        let conn = self.lock()?;
        let (project_id, state): (String, String) = conn
            .query_row(
                "SELECT project_id, state FROM p11_employee WHERE employee_id = ?1",
                [employee_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("employee for seating"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "employee not found",
            })?;
        if state != "proposed" && state != "pending" {
            return Err(ProjectAggregateError::Rejected {
                detail: "employee is not eligible for seating",
            });
        }
        let seating: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_employee WHERE project_id = ?1 AND state = 'seating'",
                [&project_id],
                |row| row.get(0),
            )
            .map_err(unavailable("count seating"))?;
        if seating > 0 {
            return Err(ProjectAggregateError::Conflict {
                detail: "sequential seating: another employee is already seating",
            });
        }
        conn.execute(
            "UPDATE p11_employee SET state = 'seating', updated_at = ?1 WHERE employee_id = ?2",
            params![now_ms, employee_id],
        )
        .map_err(unavailable("mark seating"))?;
        Ok(())
    }

    /// Confirm seating. Missing model → `pending`. Reject → `refused`.
    pub fn confirm_seating(
        &self,
        caller: ConfirmCaller,
        employee_id: &str,
        model_binding: Option<&str>,
        accept: bool,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        Self::require_owner(caller)?;
        if let Some(model) = model_binding {
            Self::reject_secret_shape(model.as_bytes())?;
        }
        let conn = self.lock()?;
        let (project_id, state, blueprint_revision_id): (String, String, String) = conn
            .query_row(
                "SELECT project_id, state, blueprint_revision_id FROM p11_employee WHERE employee_id = ?1",
                [employee_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(unavailable("employee for confirm"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "employee not found",
            })?;
        if state != "seating" {
            return Err(ProjectAggregateError::Rejected {
                detail: "employee is not seating",
            });
        }
        if !accept {
            conn.execute(
                "UPDATE p11_employee SET state = 'refused', is_current_manager = 0, updated_at = ?1
                  WHERE employee_id = ?2",
                params![now_ms, employee_id],
            )
            .map_err(unavailable("refuse seating"))?;
            return Ok("refused".to_owned());
        }
        if model_binding.is_none() {
            conn.execute(
                "UPDATE p11_employee SET state = 'pending', provider_model_binding = NULL, updated_at = ?1
                  WHERE employee_id = ?2",
                params![now_ms, employee_id],
            )
            .map_err(unavailable("pending seating"))?;
            return Ok("pending".to_owned());
        }
        let specialization: String = conn
            .query_row(
                "SELECT b.specialization FROM p11_role_blueprint b
                   JOIN p11_role_blueprint_revision r ON r.blueprint_id = b.blueprint_id
                  WHERE r.blueprint_revision_id = ?1",
                [&blueprint_revision_id],
                |row| row.get(0),
            )
            .map_err(unavailable("specialization"))?;
        let want_manager = i64::from(specialization == "project-manager");
        if want_manager == 1 {
            let existing: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM p11_employee
                      WHERE project_id = ?1 AND is_current_manager = 1 AND employee_id != ?2",
                    params![project_id, employee_id],
                    |row| row.get(0),
                )
                .map_err(unavailable("current manager count"))?;
            if existing > 0 {
                return Err(ProjectAggregateError::Rejected {
                    detail: "active Project already has one current manager",
                });
            }
        }
        conn.execute(
            "UPDATE p11_employee
                SET state = 'seated',
                    provider_model_binding = ?1,
                    is_current_manager = ?2,
                    updated_at = ?3
              WHERE employee_id = ?4",
            params![model_binding, want_manager, now_ms, employee_id],
        )
        .map_err(unavailable("seat employee"))?;
        Ok("seated".to_owned())
    }

    /// Replaceable runtime/adapter binding. Does not change employee_id.
    pub fn bind_runtime(
        &self,
        caller: ConfirmCaller,
        employee_id: &str,
        runtime_binding_ref: &str,
        now_ms: i64,
    ) -> Result<(), ProjectAggregateError> {
        Self::require_owner(caller)?;
        if runtime_binding_ref.starts_with("agent:") {
            return Err(ProjectAggregateError::Rejected {
                detail: "Role must not be merged with Agent",
            });
        }
        reject_pi_member_engine(runtime_binding_ref)?;
        reject_installed_agent_chrome(runtime_binding_ref)?;
        let conn = self.lock()?;
        let updated = conn
            .execute(
                "UPDATE p11_employee SET runtime_binding_ref = ?1, updated_at = ?2 WHERE employee_id = ?3",
                params![runtime_binding_ref, now_ms, employee_id],
            )
            .map_err(unavailable("bind runtime"))?;
        if updated == 0 {
            return Err(ProjectAggregateError::NotFound {
                detail: "employee not found",
            });
        }
        Ok(())
    }

    /// Attempt/process death observer: Employee authority is not a process.
    /// Records hosted DSH child exit. Does not delete Employee, conversation, or Memory.
    pub fn observe_attempt_process_exit(
        &self,
        employee_id: &str,
    ) -> Result<(), ProjectAggregateError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE p11_hosted_dsh_child
                SET pid = NULL, state = 'exited', terminal_kind = 'exited'
              WHERE employee_id = ?1 AND state = 'bound'",
            [employee_id],
        )
        .map_err(unavailable("observe hosted dsh exit"))?;
        Ok(())
    }

    /// Latest EmployeeRevision id for Attempt-runner start.
    pub fn latest_revision_id(
        &self,
        employee_id: &str,
    ) -> Result<Option<String>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT employee_revision_id FROM p11_employee_revision
              WHERE employee_id = ?1 ORDER BY seq DESC LIMIT 1",
            [employee_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(unavailable("latest employee revision"))
    }

    /// Refuse to reuse an Employee id in another Project. Only Blueprint is reusable.
    pub fn reuse_employee_in_project(
        &self,
        employee_id: &str,
        other_project_id: &str,
    ) -> Result<(), ProjectAggregateError> {
        let row = self
            .get_employee(employee_id)?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "employee not found",
            })?;
        if row.project_id != other_project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "employee is not shared across projects",
            });
        }
        Err(ProjectAggregateError::Forbidden {
            detail: "employee is not shared across projects",
        })
    }

    /// Remove a manager/member. History rows stay (`removed`).
    pub fn remove_employee(
        &self,
        caller: ConfirmCaller,
        employee_id: &str,
        now_ms: i64,
    ) -> Result<(), ProjectAggregateError> {
        Self::require_owner(caller)?;
        let conn = self.lock()?;
        let updated = conn
            .execute(
                "UPDATE p11_employee
                    SET state = 'removed', is_current_manager = 0, updated_at = ?1
                  WHERE employee_id = ?2",
                params![now_ms, employee_id],
            )
            .map_err(unavailable("remove employee"))?;
        if updated == 0 {
            return Err(ProjectAggregateError::NotFound {
                detail: "employee not found",
            });
        }
        Ok(())
    }

    pub fn get_employee(
        &self,
        employee_id: &str,
    ) -> Result<Option<EmployeeRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT employee_id, project_id, blueprint_revision_id, responsible_stage_ids_json,
                    provider_model_binding, runtime_binding_ref, state, is_current_manager
               FROM p11_employee WHERE employee_id = ?1",
            [employee_id],
            map_employee_row,
        )
        .optional()
        .map_err(unavailable("get employee"))
    }

    pub fn list_roster(&self, project_id: &str) -> Result<Vec<EmployeeRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT employee_id, project_id, blueprint_revision_id, responsible_stage_ids_json,
                        provider_model_binding, runtime_binding_ref, state, is_current_manager
                   FROM p11_employee WHERE project_id = ?1 ORDER BY created_at",
            )
            .map_err(unavailable("list roster"))?;
        let rows = statement
            .query_map([project_id], map_employee_row)
            .map_err(unavailable("roster query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("roster rows"))
    }

    pub fn seating_progress(
        &self,
        project_id: &str,
    ) -> Result<SeatingProgress, ProjectAggregateError> {
        let conn = self.lock()?;
        let roster: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_employee
                  WHERE project_id = ?1 AND state != 'refused'",
                [project_id],
                |row| row.get(0),
            )
            .map_err(unavailable("roster count"))?;
        let seated: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_employee
                  WHERE project_id = ?1 AND state = 'seated'",
                [project_id],
                |row| row.get(0),
            )
            .map_err(unavailable("seated count"))?;
        Ok(SeatingProgress { seated, roster })
    }

    /// Production seating predicate for T03 ④. Empty / pending / unseated = false.
    pub fn stage_is_seated(
        &self,
        project_id: &str,
        plan_revision_id: &str,
        stage_id: &str,
    ) -> Result<bool, ProjectAggregateError> {
        let conn = self.lock()?;
        let slot: Option<String> = conn
            .query_row(
                "SELECT responsible_slot FROM p11_stage
                  WHERE plan_revision_id = ?1 AND stage_id = ?2",
                params![plan_revision_id, stage_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("stage slot"))?;
        let Some(slot) = slot else {
            return Ok(false);
        };
        let seated: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_assignment a
                   JOIN p11_employee e ON e.employee_id = a.employee_id
                  WHERE a.project_id = ?1
                    AND a.plan_revision_id = ?2
                    AND a.slot = ?3
                    AND e.state = 'seated'
                    AND e.provider_model_binding IS NOT NULL",
                params![project_id, plan_revision_id, slot],
                |row| row.get(0),
            )
            .map_err(unavailable("seated assignment"))?;
        Ok(seated > 0)
    }

    pub fn record_install_fact(
        &self,
        capability_ref: &str,
        version_pin: &str,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        Self::reject_secret_shape(capability_ref.as_bytes())?;
        let conn = self.lock()?;
        let install_id = next_id("install")?;
        conn.execute(
            "INSERT OR IGNORE INTO p11_install_fact (install_id, capability_ref, version_pin, created_at)
             VALUES (?1,?2,?3,?4)",
            params![install_id, capability_ref, version_pin, now_ms],
        )
        .map_err(unavailable("insert install fact"))?;
        let id: String = conn
            .query_row(
                "SELECT install_id FROM p11_install_fact
                  WHERE capability_ref = ?1 AND version_pin = ?2",
                params![capability_ref, version_pin],
                |row| row.get(0),
            )
            .map_err(unavailable("load install fact"))?;
        Ok(id)
    }

    pub fn grant_capability(
        &self,
        caller: ConfirmCaller,
        project_id: &str,
        employee_id: &str,
        capability_ref: &str,
        scope: &str,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        Self::require_owner(caller)?;
        let employee = self
            .get_employee(employee_id)?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "employee not found",
            })?;
        if employee.project_id != project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-project write rejected",
            });
        }
        let conn = self.lock()?;
        let installed: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_install_fact WHERE capability_ref = ?1",
                [capability_ref],
                |row| row.get(0),
            )
            .map_err(unavailable("install fact for grant"))?;
        if installed == 0 {
            return Err(ProjectAggregateError::Rejected {
                detail: "grant requires an InstallFact",
            });
        }
        let grant_id = next_id("grant")?;
        conn.execute(
            "INSERT INTO p11_grant (grant_id, project_id, employee_id, capability_ref, scope, created_at)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![grant_id, project_id, employee_id, capability_ref, scope, now_ms],
        )
        .map_err(unavailable("insert grant"))?;
        Ok(grant_id)
    }

    /// L2 catalog = grants only. Recipe tool mentions grant nothing.
    pub fn tool_catalog(
        &self,
        project_id: &str,
        employee_id: &str,
    ) -> Result<Vec<String>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT capability_ref FROM p11_grant
                  WHERE project_id = ?1 AND employee_id = ?2 ORDER BY capability_ref",
            )
            .map_err(unavailable("catalog"))?;
        let rows = statement
            .query_map(params![project_id, employee_id], |row| row.get(0))
            .map_err(unavailable("catalog query"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("catalog rows"))
    }

    pub fn invoke_tool(
        &self,
        project_id: &str,
        employee_id: &str,
        tool_name: &str,
    ) -> Result<(), ProjectAggregateError> {
        let catalog = self.tool_catalog(project_id, employee_id)?;
        if catalog.iter().any(|item| item == tool_name) {
            Ok(())
        } else {
            Err(ProjectAggregateError::Forbidden {
                detail: "tool is not in the grant catalog",
            })
        }
    }

    pub fn recipe_declared_tools(
        &self,
        employee_id: &str,
    ) -> Result<Vec<String>, ProjectAggregateError> {
        let conn = self.lock()?;
        let json_text: String = conn
            .query_row(
                "SELECT tools_declared_json FROM p11_employee_revision
                  WHERE employee_id = ?1 ORDER BY seq DESC LIMIT 1",
                [employee_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("recipe tools"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "employee revision not found",
            })?;
        serde_json::from_str(&json_text).map_err(|_| ProjectAggregateError::Unavailable {
            detail: "parse tools_declared_json".to_owned(),
        })
    }

    /// Member proactive speech is whitelist-only. Writes `p11_speech_audit` only.
    pub fn route_speech(
        &self,
        project_id: &str,
        employee_id: &str,
        kind: &str,
        mentioned: bool,
        now_ms: i64,
    ) -> Result<SpeechDecision, ProjectAggregateError> {
        let employee = self
            .get_employee(employee_id)?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "employee not found",
            })?;
        if employee.project_id != project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-project write rejected",
            });
        }
        let whitelist = ["deliverable", "handoff", "blocked", "decision-request"];
        let is_manager = employee.is_current_manager;
        let delivered = is_manager || mentioned || whitelist.contains(&kind);
        let reason = if delivered {
            if is_manager {
                "manager-default"
            } else if mentioned {
                "mentioned"
            } else {
                "whitelist"
            }
        } else {
            "speech-filtered"
        };
        let conn = self.lock()?;
        let audit_id = next_id("speech")?;
        conn.execute(
            "INSERT INTO p11_speech_audit (
                audit_id, project_id, employee_id, kind, mentioned, delivered, reason, created_at
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                audit_id,
                project_id,
                employee_id,
                kind,
                i64::from(mentioned),
                i64::from(delivered),
                reason,
                now_ms
            ],
        )
        .map_err(unavailable("speech audit"))?;
        Ok(SpeechDecision {
            audit_id,
            delivered,
            reason: reason.to_owned(),
        })
    }

    pub fn record_handoff(
        &self,
        caller: ConfirmCaller,
        spec: &HandoffSpec<'_>,
    ) -> Result<String, ProjectAggregateError> {
        Self::require_owner(caller)?;
        if spec.bounded_work_digest.len() != 64 {
            return Err(ProjectAggregateError::Invalid {
                detail: "bounded_work_digest must be 64 hex chars",
            });
        }
        let conn = self.lock()?;
        let handoff_id = next_id("handoff")?;
        conn.execute(
            "INSERT INTO p11_handoff (
                handoff_id, project_id, source_employee_id, target_employee_id,
                bounded_work_digest, blocked_or_ready, authority_stays, created_at
             ) VALUES (?1,?2,?3,?4,?5,?6,1,?7)",
            params![
                handoff_id,
                spec.project_id,
                spec.source_employee_id,
                spec.target_employee_id,
                spec.bounded_work_digest,
                spec.blocked_or_ready,
                spec.now_ms
            ],
        )
        .map_err(unavailable("insert handoff"))?;
        Ok(handoff_id)
    }

    /// Chat/handoff text cannot transfer authority. Grants stay unchanged.
    pub fn apply_chat_authority_transfer(&self, _claim: &str) -> Result<(), ProjectAggregateError> {
        Err(ProjectAggregateError::Forbidden {
            detail: "chat cannot transfer authority",
        })
    }

    pub fn grant_count(&self, employee_id: &str) -> Result<i64, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT COUNT(*) FROM p11_grant WHERE employee_id = ?1",
            [employee_id],
            |row| row.get(0),
        )
        .map_err(unavailable("grant count"))
    }

    pub fn current_manager_count(&self, project_id: &str) -> Result<i64, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT COUNT(*) FROM p11_employee WHERE project_id = ?1 AND is_current_manager = 1",
            [project_id],
            |row| row.get(0),
        )
        .map_err(unavailable("manager count"))
    }

    /// Schema assertion: Blueprint table has no Provider binding column.
    pub fn blueprint_column_names(&self) -> Result<Vec<String>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare("PRAGMA table_info(p11_role_blueprint)")
            .map_err(unavailable("pragma blueprint"))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(unavailable("blueprint columns"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("blueprint column rows"))
    }

    /// Schema assertion: handoff has no authority-transfer field.
    pub fn handoff_column_names(&self) -> Result<Vec<String>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare("PRAGMA table_info(p11_handoff)")
            .map_err(unavailable("pragma handoff"))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(unavailable("handoff columns"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("handoff column rows"))
    }

    pub fn employee_column_names(&self) -> Result<Vec<String>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare("PRAGMA table_info(p11_employee)")
            .map_err(unavailable("pragma employee"))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(unavailable("employee columns"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("employee column rows"))
    }
}

fn map_employee_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EmployeeRow> {
    let manager: i64 = row.get(7)?;
    Ok(EmployeeRow {
        employee_id: row.get(0)?,
        project_id: row.get(1)?,
        blueprint_revision_id: row.get(2)?,
        responsible_stage_ids_json: row.get(3)?,
        provider_model_binding: row.get(4)?,
        runtime_binding_ref: row.get(5)?,
        state: row.get(6)?,
        is_current_manager: manager == 1,
    })
}

pub(crate) fn reject_pi_member_engine(value: &str) -> Result<(), ProjectAggregateError> {
    let lowered = value.to_ascii_lowercase();
    if lowered.starts_with("pi:")
        || lowered.contains("earendil.pi")
        || lowered.contains("hidden-pi-assistant")
        || lowered.contains("agent://personal/pi")
        || lowered.contains("cognitiveos.private-candidate/1")
    {
        return Err(ProjectAggregateError::Rejected {
            detail: "Pi is not the Member execution engine",
        });
    }
    Ok(())
}

pub(crate) fn reject_installed_agent_chrome(value: &str) -> Result<(), ProjectAggregateError> {
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("installed-agent")
        || lowered.contains("/ui/")
        || lowered.contains("apps/web")
        || lowered.contains("plugin-store")
        || lowered.contains("dsh-web")
        || lowered.contains("engine-store")
    {
        return Err(ProjectAggregateError::Rejected {
            detail: "Installed Agent chrome is not the hosted DSH engine",
        });
    }
    Ok(())
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
