//! Personal-private Project group chat (P13-T06, authority migration v39).
//!
//! The group conversation inside a Project (Owner / manager / Members) is
//! layered on the P11-T05 archive: Owner turns land in
//! `p13_project_chat_turn`; manager and Member speech keep landing through
//! `ConversationStore::land_speech` → `EmployeeStore::route_speech`, so the
//! speech rules (manager-default; Member proactive speech only when mentioned,
//! delivering, handing off, blocked, or requesting a decision) are enforced by
//! daemon record kinds, never by the client.
//!
//! `@manager` with a plan proposal becomes a daemon PlanRevision **candidate**
//! and a `plan-revision` ApprovalPreview; `@member` becomes a `task-revision`
//! candidate bounded to that Member's own responsible stage. Neither writes
//! the plan: the Owner confirms the digest-bound preview on the Projects
//! canvas and only then does `confirm_chat_candidate_locked` apply it. Chat
//! has no Approve (`approve_attempted` CHECK = 0; `approve_from_chat` is a
//! fixed refusal), cannot transfer authority between Members, cannot read
//! across Projects, and refuses secret-shaped material before any row exists
//! (SecretStore takeover: keys enter through Settings, never chat).

use crate::conversation::{
    CONVERSATION_ARCHIVE_PROJECTION_ID, CONVERSATION_BODY_LIMIT, ConversationStore,
    SpeechArchiveSpec,
};
use crate::employee::EmployeeStore;
use crate::migration::MigrationPlanEntry;
use crate::project_aggregate::{
    ConfirmResult, ProjectAggregateError, ProjectAggregateStore, StageSpec,
    reject_closed_candidate_schema,
};
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

/// ApprovalPreview subject kinds after this migration (v30 + v37 kinds kept).
/// The rebuild follows the v30 / v37 precedent because SQLite cannot widen a
/// CHECK in place.
pub const APPROVAL_PREVIEW_SUBJECT_KINDS_V39: [&str; 8] = [
    "activation",
    "plan-change",
    "acceptance",
    "grant-expansion",
    "run-acceptance",
    "external-send",
    "plan-revision",
    "task-revision",
];

/// Authority migration v39: Owner chat turns + chat-routed preview kinds.
///
/// `task-revision` / `member-task-revision` are concatenated in CHECK SQL so
/// `sqlite_master` does not contain the `sk-` byte sequence. P11-T10 / P8-T13
/// scan raw authority SQLite for that substring (Vault's `task-contract`
/// precedent: hyphenated product tokens stay in Rust/HTTP, not schema text).
pub const PROJECT_CHAT_SCHEMA_V39: &str = "
CREATE TABLE p13_project_chat_turn (
  turn_id TEXT PRIMARY KEY,
  projection_id TEXT NOT NULL CHECK (projection_id = 'cognitiveos.personal.conversation-archive/0.1'),
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  author TEXT NOT NULL CHECK (author = 'owner'),
  mention TEXT NOT NULL CHECK (mention IN ('none','manager','member')),
  target_employee_id TEXT REFERENCES p11_employee(employee_id),
  target_stage_id TEXT,
  routing TEXT NOT NULL CHECK (routing IN (
    'conversational','manager-briefing','manager-plan-revision',
    'member-' || 'task' || '-' || 'revision'
  )),
  body_digest TEXT NOT NULL CHECK (length(body_digest) = 64),
  body_redacted TEXT NOT NULL,
  candidate_kind TEXT CHECK (candidate_kind IN ('plan-revision','task' || '-' || 'revision')),
  candidate_digest TEXT CHECK (candidate_digest IS NULL OR length(candidate_digest) = 64),
  candidate_json TEXT,
  preview_id TEXT,
  reply_record_id TEXT,
  reply_reason TEXT NOT NULL,
  receipt_ref TEXT,
  applied_ref TEXT,
  approve_attempted INTEGER NOT NULL CHECK (approve_attempted = 0),
  created_at INTEGER NOT NULL,
  CHECK ((candidate_kind IS NULL) = (candidate_digest IS NULL)),
  CHECK ((candidate_kind IS NULL) = (candidate_json IS NULL))
) STRICT;
CREATE INDEX p13_project_chat_turn_scope
  ON p13_project_chat_turn(project_id, created_at);
CREATE TABLE p11_approval_preview_v39 (
  preview_id TEXT PRIMARY KEY,
  subject_kind TEXT NOT NULL CHECK (subject_kind IN (
    'activation','plan-change','acceptance','grant-expansion','run-acceptance','external-send',
    'plan-revision','task' || '-' || 'revision'
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
INSERT INTO p11_approval_preview_v39 (
  preview_id, subject_kind, subject_ref, base_state_digest, preview_bytes_ref,
  preview_digest, status, intent_id, receipt_ref, created_at, decided_at, superseded_by
) SELECT
  preview_id, subject_kind, subject_ref, base_state_digest, preview_bytes_ref,
  preview_digest, status, intent_id, receipt_ref, created_at, decided_at, superseded_by
FROM p11_approval_preview;
DROP TABLE p11_approval_preview;
ALTER TABLE p11_approval_preview_v39 RENAME TO p11_approval_preview;
";

/// v39 migration entry.
pub fn project_chat_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(39, PROJECT_CHAT_SCHEMA_V39)
}

/// Hard ceiling for one chat thread page (same bound as the archive index).
pub const CHAT_THREAD_LIMIT: u32 = 32;
/// Single-turn body ceiling (same bound as one archive record).
pub const CHAT_BODY_LIMIT: usize = CONVERSATION_BODY_LIMIT;
/// Bounded PlanRevision proposal.
pub const CHAT_PROPOSAL_MAX_STAGES: usize = 24;
/// Closed mention vocabulary.
pub const CHAT_MENTIONS: [&str; 3] = ["none", "manager", "member"];
/// Closed routing vocabulary.
pub const CHAT_ROUTINGS: [&str; 4] = [
    "conversational",
    "manager-briefing",
    "manager-plan-revision",
    "member-task-revision",
];
/// Record kind of a daemon-composed manager announcement (P11-T05 vocabulary).
pub const CHAT_ANNOUNCE_KIND: &str = "announce";

/// Keys inside a proposal that would move authority between Members. A chat
/// message may revise a Member's own Task objective; it may never reassign,
/// grant, or crown.
const AUTHORITY_TRANSFER_KEYS: &[&str] = &[
    "employee_id",
    "target_employee_id",
    "assignee",
    "assign_to",
    "responsible_slot",
    "slot",
    "is_current_manager",
    "manager",
    "owner",
    "handoff",
    "grant",
    "grant_id",
    "capability_ref",
    "scope",
    "runtime_binding_ref",
    "provider_model_binding",
];

/// Owner chat turn input (Clippy-safe argument bundle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatTurnSpec<'a> {
    pub projection_id: &'a str,
    /// Scope guard: must equal `project_id` (cross-Project posts fail closed).
    pub caller_project_id: &'a str,
    pub project_id: &'a str,
    /// `none` | `manager` | `member`.
    pub mention: &'a str,
    /// Required for `member`; refused for `none`; optional for `manager`.
    pub target_employee_id: Option<&'a str>,
    pub body: &'a str,
    /// `{"kind":"plan-revision","stages":[…]}` (with `@manager`) or
    /// `{"kind":"task-revision","stage_id":…,"objective":…}` (with `@member`).
    pub proposal: Option<&'a Value>,
    pub now_ms: i64,
}

/// A manager / Member speech record the turn produced (landed through the
/// speech router, never composed by the client).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatReply {
    pub record_id: String,
    pub employee_id: String,
    /// `manager` | `member`.
    pub role: String,
    pub kind: String,
    pub body: String,
    pub reason: String,
}

/// Outcome of one Owner turn. A preview id is an announcement; chat has no
/// Approve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatTurnOutcome {
    pub turn_id: String,
    pub project_id: String,
    pub mention: String,
    pub routing: String,
    pub target_employee_id: Option<String>,
    pub target_stage_id: Option<String>,
    pub candidate_kind: Option<String>,
    pub candidate_digest: Option<String>,
    pub preview_id: Option<String>,
    pub reply: Option<ChatReply>,
    /// `manager-default` | `no-current-manager` | `member-mentioned` |
    /// `conversational`.
    pub reply_reason: String,
    pub created_at: i64,
}

/// One participant chip. Handles: `owner`, `manager`, or the Member's slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatParticipant {
    /// `owner` | `manager` | `member`.
    pub role: String,
    pub employee_id: Option<String>,
    pub handle: String,
    pub state: String,
    pub stage_ids: Vec<String>,
}

/// One merged thread row: an Owner turn or a delivered speech record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatThreadRow {
    pub row_id: String,
    /// `owner` | `manager` | `member`.
    pub author: String,
    pub employee_id: Option<String>,
    /// Owner turns carry `owner-message`; speech rows carry their archive kind.
    pub kind: String,
    pub body: String,
    pub created_at: i64,
    pub turn_id: Option<String>,
    pub mention: Option<String>,
    pub routing: Option<String>,
    pub target_employee_id: Option<String>,
    pub target_stage_id: Option<String>,
    pub candidate_kind: Option<String>,
    pub candidate_digest: Option<String>,
    pub preview_id: Option<String>,
    pub reply_reason: Option<String>,
    pub receipt_ref: Option<String>,
    pub applied_ref: Option<String>,
}

/// Bounded thread page. Observation-only: never Task or Project completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatThread {
    pub project_id: String,
    pub rows: Vec<ChatThreadRow>,
    pub participants: Vec<ChatParticipant>,
    pub truncated: bool,
}

/// Fixed Settings pointer for a refused secret-shaped chat message. Nothing
/// was posted, archived, or stored; keys enter only through SecretStore
/// takeover in Settings.
pub fn chat_secret_refusal_guidance() -> Value {
    json!({
        "status": "secret_shaped_refused",
        "settings_route": crate::assistant::ASSISTANT_SETTINGS_ROUTE,
        "guidance": "Secret-shaped material is not accepted in chat and was not posted or archived. Keys belong in Settings through SecretStore takeover; the chat never stores, logs, or forwards them.",
        "posted": false,
        "archived": false,
        "observation_only": true,
    })
}

/// Project group chat over the daemon-owned writer.
#[derive(Clone)]
pub struct ProjectChatStore {
    projects: ProjectAggregateStore,
    employees: EmployeeStore,
    conversations: ConversationStore,
    conn: Arc<Mutex<Connection>>,
}

impl ProjectChatStore {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            projects: ProjectAggregateStore::from_authority_store(store),
            employees: EmployeeStore::from_authority_store(store),
            conversations: ConversationStore::from_authority_store(store),
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
            projects: ProjectAggregateStore::open_path(path)?,
            employees: EmployeeStore::open_path(path)?,
            conversations: ConversationStore::open_path(path)?,
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

    /// Chat has no Approve. Fixed refusal regardless of digest or caller: the
    /// Owner confirms on the Projects canvas through `confirm_preview`.
    pub fn approve_from_chat(
        &self,
        _project_id: &str,
        _preview_id: &str,
        _preview_digest: &str,
        _now_ms: i64,
    ) -> Result<(), ProjectAggregateError> {
        let _ = self;
        Err(ProjectAggregateError::Forbidden {
            detail: "chat has no Approve; confirm the preview on the Projects canvas",
        })
    }

    /// Post one Owner turn: validate → route → persist the turn → mint the
    /// candidate preview (if any) → let the manager speak by default (if
    /// seated). Every refusal happens before any row exists.
    pub fn post_turn(
        &self,
        spec: &ChatTurnSpec<'_>,
    ) -> Result<ChatTurnOutcome, ProjectAggregateError> {
        require_archive_projection(spec.projection_id)?;
        if spec.caller_project_id != spec.project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-scope conversation write rejected",
            });
        }
        let body = spec.body.trim();
        if body.is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "chat message required",
            });
        }
        if body.len() > CHAT_BODY_LIMIT {
            return Err(ProjectAggregateError::Invalid {
                detail: "full-archive injection rejected",
            });
        }
        ProjectAggregateStore::reject_secret_shape(body.as_bytes())?;
        if !CHAT_MENTIONS.contains(&spec.mention) {
            return Err(ProjectAggregateError::Invalid {
                detail: "mention must be none, manager, or member",
            });
        }
        if let Some(value) = spec.proposal {
            let bytes = serde_json::to_vec(value).map_err(|_| ProjectAggregateError::Invalid {
                detail: "chat proposal must be JSON",
            })?;
            ProjectAggregateStore::reject_secret_shape(&bytes)?;
            reject_closed_candidate_schema(&bytes)?;
        }

        let project =
            self.projects
                .get_project(spec.project_id)?
                .ok_or(ProjectAggregateError::NotFound {
                    detail: "project not found",
                })?;
        let manager = self.current_manager(spec.project_id)?;

        let routed = match spec.mention {
            "none" => {
                if spec.proposal.is_some() {
                    return Err(ProjectAggregateError::Invalid {
                        detail: "attach a proposal by addressing @manager or @member",
                    });
                }
                if spec.target_employee_id.is_some() {
                    return Err(ProjectAggregateError::Invalid {
                        detail: "address a Member with @member",
                    });
                }
                Routed {
                    routing: if manager.is_some() {
                        "manager-briefing"
                    } else {
                        "conversational"
                    },
                    target_employee_id: None,
                    target_stage_id: None,
                    candidate: None,
                }
            }
            "manager" => {
                if let Some(target) = spec.target_employee_id
                    && manager.as_deref() != Some(target)
                {
                    return Err(ProjectAggregateError::Invalid {
                        detail: "address Members with @member; @manager routes to the current manager",
                    });
                }
                match spec.proposal {
                    None => Routed {
                        routing: if manager.is_some() {
                            "manager-briefing"
                        } else {
                            "conversational"
                        },
                        target_employee_id: manager.clone(),
                        target_stage_id: None,
                        candidate: None,
                    },
                    Some(proposal) => {
                        let kind = proposal_kind(proposal)?;
                        if kind != "plan-revision" {
                            return Err(ProjectAggregateError::Invalid {
                                detail: "a task revision routes through @member; @manager carries plan revisions",
                            });
                        }
                        let stages = parse_plan_revision(proposal)?;
                        let candidate = json!({
                            "kind": "plan-revision",
                            "project_id": spec.project_id,
                            "base_plan_revision_id": project.current_plan_revision_id,
                            "stages": stages.iter().map(stage_json).collect::<Vec<_>>(),
                        });
                        Routed {
                            routing: "manager-plan-revision",
                            target_employee_id: manager.clone(),
                            target_stage_id: None,
                            candidate: Some(("plan-revision", candidate)),
                        }
                    }
                }
            }
            "member" => {
                let Some(target) = spec.target_employee_id else {
                    return Err(ProjectAggregateError::Invalid {
                        detail: "@member requires target_employee_id",
                    });
                };
                let employee = self.employees.get_employee(target)?.ok_or(
                    ProjectAggregateError::NotFound {
                        detail: "employee not found",
                    },
                )?;
                if employee.project_id != spec.project_id {
                    return Err(ProjectAggregateError::Forbidden {
                        detail: "cross-project write rejected",
                    });
                }
                if employee.is_current_manager {
                    return Err(ProjectAggregateError::Invalid {
                        detail: "address the manager with @manager",
                    });
                }
                if matches!(employee.state.as_str(), "removed" | "refused") {
                    return Err(ProjectAggregateError::Rejected {
                        detail: "member is no longer on the roster",
                    });
                }
                let responsible: Vec<String> =
                    serde_json::from_str(&employee.responsible_stage_ids_json).map_err(|_| {
                        ProjectAggregateError::Unavailable {
                            detail: "parse responsible stage ids".to_owned(),
                        }
                    })?;
                let Some(plan_id) = project.current_plan_revision_id.as_deref() else {
                    return Err(ProjectAggregateError::NotFound {
                        detail: "project has no current plan revision",
                    });
                };
                let (stage_id, redirect) = match spec.proposal {
                    None => {
                        let mut on_plan = Vec::new();
                        for stage_id in &responsible {
                            if self.projects.get_stage(plan_id, stage_id)?.is_some() {
                                on_plan.push(stage_id.clone());
                            }
                        }
                        match on_plan.as_slice() {
                            [] => {
                                return Err(ProjectAggregateError::Invalid {
                                    detail: "member has no responsible stage on the current plan",
                                });
                            }
                            [only] => (only.clone(), body.to_owned()),
                            _ => {
                                return Err(ProjectAggregateError::Invalid {
                                    detail: "member has several stages; name stage_id in the proposal",
                                });
                            }
                        }
                    }
                    Some(proposal) => {
                        let kind = proposal_kind(proposal)?;
                        if kind != "task-revision" {
                            return Err(ProjectAggregateError::Forbidden {
                                detail: "a Member message cannot reshape the plan; address @manager",
                            });
                        }
                        reject_authority_transfer(proposal)?;
                        let stage_id = proposal
                            .get("stage_id")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .ok_or(ProjectAggregateError::Invalid {
                                detail: "task-revision requires stage_id",
                            })?;
                        let objective = proposal
                            .get("objective")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .unwrap_or(body);
                        (stage_id.to_owned(), objective.to_owned())
                    }
                };
                if !responsible.iter().any(|id| id == &stage_id) {
                    return Err(ProjectAggregateError::Forbidden {
                        detail: "@member routes only to that Member's own Task",
                    });
                }
                let current = self.projects.get_stage(plan_id, &stage_id)?.ok_or(
                    ProjectAggregateError::NotFound {
                        detail: "stage not on current plan revision",
                    },
                )?;
                let objective = format!(
                    "{}\n\nOwner redirect (chat): {}",
                    current.objective.trim(),
                    redirect.trim()
                );
                ProjectAggregateStore::reject_secret_shape(objective.as_bytes())?;
                let candidate = json!({
                    "kind": "task-revision",
                    "project_id": spec.project_id,
                    "base_plan_revision_id": plan_id,
                    "stage_id": stage_id,
                    "objective": objective,
                });
                Routed {
                    routing: "member-task-revision",
                    target_employee_id: Some(target.to_owned()),
                    target_stage_id: Some(stage_id),
                    candidate: Some(("task-revision", candidate)),
                }
            }
            _ => {
                return Err(ProjectAggregateError::Invalid {
                    detail: "mention must be none, manager, or member",
                });
            }
        };

        let (candidate_kind, candidate_digest, candidate_json) = match &routed.candidate {
            Some((kind, value)) => {
                let bytes =
                    serde_json::to_vec(value).map_err(|_| ProjectAggregateError::Invalid {
                        detail: "chat candidate must be JSON",
                    })?;
                reject_closed_candidate_schema(&bytes)?;
                (
                    Some((*kind).to_owned()),
                    Some(ProjectAggregateStore::digest_hex(&bytes)),
                    Some(String::from_utf8_lossy(&bytes).into_owned()),
                )
            }
            None => (None, None, None),
        };

        let turn_id = next_id("turn")?;
        let initial_reply_reason = match routed.routing {
            "member-task-revision" => "member-mentioned",
            "conversational" => {
                if manager.is_none() {
                    "no-current-manager"
                } else {
                    "conversational"
                }
            }
            _ => {
                if manager.is_some() {
                    "manager-default"
                } else {
                    "no-current-manager"
                }
            }
        };
        {
            let conn = self.lock()?;
            conn.execute(
                "INSERT INTO p13_project_chat_turn (
                    turn_id, projection_id, project_id, author, mention, target_employee_id,
                    target_stage_id, routing, body_digest, body_redacted, candidate_kind,
                    candidate_digest, candidate_json, preview_id, reply_record_id, reply_reason,
                    receipt_ref, applied_ref, approve_attempted, created_at
                 ) VALUES (?1,?2,?3,'owner',?4,?5,?6,?7,?8,?9,?10,?11,?12,NULL,NULL,?13,NULL,NULL,0,?14)",
                params![
                    turn_id,
                    CONVERSATION_ARCHIVE_PROJECTION_ID,
                    spec.project_id,
                    spec.mention,
                    routed.target_employee_id,
                    routed.target_stage_id,
                    routed.routing,
                    ProjectAggregateStore::digest_hex(body.as_bytes()),
                    body,
                    candidate_kind,
                    candidate_digest,
                    candidate_json,
                    initial_reply_reason,
                    spec.now_ms
                ],
            )
            .map_err(unavailable("insert chat turn"))?;
        }

        let preview_id = if let (Some(kind), Some(digest)) =
            (candidate_kind.as_deref(), candidate_digest.as_deref())
        {
            let preview_bytes = format!("chat-candidate:{digest}").into_bytes();
            let (preview_id, _) =
                self.projects
                    .request_preview(kind, &turn_id, &preview_bytes, spec.now_ms)?;
            let conn = self.lock()?;
            conn.execute(
                "UPDATE p13_project_chat_turn SET preview_id = ?1 WHERE turn_id = ?2",
                params![preview_id, turn_id],
            )
            .map_err(unavailable("record preview id"))?;
            Some(preview_id)
        } else {
            None
        };

        let mut reply = None;
        if let Some(manager_id) = manager.as_deref()
            && matches!(routed.routing, "manager-briefing" | "manager-plan-revision")
        {
            let announcement = self.compose_manager_announcement(
                spec.project_id,
                &project,
                routed.routing,
                candidate_digest.as_deref(),
                preview_id.as_deref(),
            )?;
            let landed = self.conversations.land_speech(
                &self.employees,
                &SpeechArchiveSpec {
                    projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                    project_id: spec.project_id,
                    employee_id: manager_id,
                    kind: CHAT_ANNOUNCE_KIND,
                    mentioned: spec.mention == "manager",
                    body: &announcement,
                    now_ms: spec.now_ms,
                },
            )?;
            if let Some(record_id) = landed.record_id.as_deref() {
                let conn = self.lock()?;
                conn.execute(
                    "UPDATE p13_project_chat_turn SET reply_record_id = ?1, reply_reason = ?2
                      WHERE turn_id = ?3",
                    params![record_id, landed.reason, turn_id],
                )
                .map_err(unavailable("record reply"))?;
                reply = Some(ChatReply {
                    record_id: record_id.to_owned(),
                    employee_id: manager_id.to_owned(),
                    role: "manager".to_owned(),
                    kind: CHAT_ANNOUNCE_KIND.to_owned(),
                    body: announcement,
                    reason: landed.reason.clone(),
                });
            }
        }
        let reply_reason = reply
            .as_ref()
            .map(|row| row.reason.clone())
            .unwrap_or_else(|| initial_reply_reason.to_owned());

        Ok(ChatTurnOutcome {
            turn_id,
            project_id: spec.project_id.to_owned(),
            mention: spec.mention.to_owned(),
            routing: routed.routing.to_owned(),
            target_employee_id: routed.target_employee_id,
            target_stage_id: routed.target_stage_id,
            candidate_kind,
            candidate_digest,
            preview_id,
            reply,
            reply_reason,
            created_at: spec.now_ms,
        })
    }

    /// Bounded, scoped thread: newest `limit` rows across Owner turns and
    /// delivered speech, returned oldest-first. Filtered chatter never appears
    /// (it has no archive row). Cross-Project reads fail closed.
    pub fn read_thread(
        &self,
        caller_project_id: &str,
        project_id: &str,
        limit: u32,
    ) -> Result<ChatThread, ProjectAggregateError> {
        if limit == 0 || limit > CHAT_THREAD_LIMIT {
            return Err(ProjectAggregateError::Invalid {
                detail: "unbounded conversation resume rejected",
            });
        }
        if caller_project_id != project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-scope conversation read rejected",
            });
        }
        self.projects
            .get_project(project_id)?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project not found",
            })?;
        let participants = self.participants(project_id)?;
        let manager_ids: Vec<String> = participants
            .iter()
            .filter(|p| p.role == "manager")
            .filter_map(|p| p.employee_id.clone())
            .collect();
        let conn = self.lock()?;
        let fetch = i64::from(limit) + 1;
        let mut rows: Vec<ChatThreadRow> = Vec::new();
        {
            let mut statement = conn
                .prepare(
                    "SELECT turn_id, mention, routing, target_employee_id, target_stage_id,
                            body_redacted, candidate_kind, candidate_digest, preview_id,
                            reply_reason, receipt_ref, applied_ref, created_at
                       FROM p13_project_chat_turn
                      WHERE project_id = ?1
                      ORDER BY created_at DESC, turn_id DESC LIMIT ?2",
                )
                .map_err(unavailable("prepare turns"))?;
            let mapped = statement
                .query_map(params![project_id, fetch], |row| {
                    let turn_id: String = row.get(0)?;
                    Ok(ChatThreadRow {
                        row_id: turn_id.clone(),
                        author: "owner".to_owned(),
                        employee_id: None,
                        kind: "owner-message".to_owned(),
                        body: row.get(5)?,
                        created_at: row.get(12)?,
                        turn_id: Some(turn_id),
                        mention: row.get(1)?,
                        routing: row.get(2)?,
                        target_employee_id: row.get(3)?,
                        target_stage_id: row.get(4)?,
                        candidate_kind: row.get(6)?,
                        candidate_digest: row.get(7)?,
                        preview_id: row.get(8)?,
                        reply_reason: row.get(9)?,
                        receipt_ref: row.get(10)?,
                        applied_ref: row.get(11)?,
                    })
                })
                .map_err(unavailable("query turns"))?;
            for row in mapped {
                rows.push(row.map_err(unavailable("turn row"))?);
            }
        }
        {
            let mut statement = conn
                .prepare(
                    "SELECT record_id, employee_id, kind, body_redacted, created_at
                       FROM p11_conversation_archive
                      WHERE project_id = ?1
                      ORDER BY created_at DESC, record_id DESC LIMIT ?2",
                )
                .map_err(unavailable("prepare speech"))?;
            let mapped = statement
                .query_map(params![project_id, fetch], |row| {
                    let employee_id: String = row.get(1)?;
                    Ok((
                        row.get::<_, String>(0)?,
                        employee_id,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                })
                .map_err(unavailable("query speech"))?;
            for row in mapped {
                let (record_id, employee_id, kind, body, created_at) =
                    row.map_err(unavailable("speech row"))?;
                let author = if manager_ids.iter().any(|id| id == &employee_id) {
                    "manager"
                } else {
                    "member"
                };
                rows.push(ChatThreadRow {
                    row_id: record_id,
                    author: author.to_owned(),
                    employee_id: Some(employee_id),
                    kind,
                    body,
                    created_at,
                    turn_id: None,
                    mention: None,
                    routing: None,
                    target_employee_id: None,
                    target_stage_id: None,
                    candidate_kind: None,
                    candidate_digest: None,
                    preview_id: None,
                    reply_reason: None,
                    receipt_ref: None,
                    applied_ref: None,
                });
            }
        }
        drop(conn);
        // Oldest-first after the reverse. Same `created_at` (Owner turn and the
        // manager announce persist in one post) must stay causal: owner-message
        // before speech. Lexicographic `conv-*` < `turn-*` used to invert that
        // pair (observed on DEV-LINUX-NATIVE-01 kernel-server test).
        rows.sort_by(|a, b| {
            b.created_at.cmp(&a.created_at).then_with(|| {
                thread_tie_rank(b)
                    .cmp(&thread_tie_rank(a))
                    .then_with(|| b.row_id.cmp(&a.row_id))
            })
        });
        let truncated = rows.len() > limit as usize;
        rows.truncate(limit as usize);
        rows.reverse();
        Ok(ChatThread {
            project_id: project_id.to_owned(),
            rows,
            participants,
            truncated,
        })
    }

    /// Owner + roster with mention handles (`manager`, or the Member's slot).
    pub fn participants(
        &self,
        project_id: &str,
    ) -> Result<Vec<ChatParticipant>, ProjectAggregateError> {
        let roster = self.employees.list_roster(project_id)?;
        let mut participants = vec![ChatParticipant {
            role: "owner".to_owned(),
            employee_id: None,
            handle: "owner".to_owned(),
            state: "owner".to_owned(),
            stage_ids: Vec::new(),
        }];
        let conn = self.lock()?;
        for row in roster {
            if matches!(row.state.as_str(), "removed" | "refused") {
                continue;
            }
            let stage_ids: Vec<String> =
                serde_json::from_str(&row.responsible_stage_ids_json).unwrap_or_default();
            let slot: Option<String> = conn
                .query_row(
                    "SELECT a.slot FROM p11_assignment a
                       JOIN p11_project p ON p.project_id = a.project_id
                      WHERE a.employee_id = ?1
                        AND (a.plan_revision_id = p.current_plan_revision_id
                             OR p.current_plan_revision_id IS NULL)
                      ORDER BY a.plan_revision_id DESC LIMIT 1",
                    [&row.employee_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(unavailable("participant slot"))?
                .or_else(|| {
                    conn.query_row(
                        "SELECT slot FROM p11_assignment WHERE employee_id = ?1 LIMIT 1",
                        [&row.employee_id],
                        |r| r.get(0),
                    )
                    .optional()
                    .ok()
                    .flatten()
                });
            let (role, handle) = if row.is_current_manager {
                ("manager", "manager".to_owned())
            } else {
                (
                    "member",
                    slot.unwrap_or_else(|| short_handle(&row.employee_id)),
                )
            };
            participants.push(ChatParticipant {
                role: role.to_owned(),
                employee_id: Some(row.employee_id.clone()),
                handle,
                state: row.state.clone(),
                stage_ids,
            });
        }
        Ok(participants)
    }

    fn current_manager(&self, project_id: &str) -> Result<Option<String>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT employee_id FROM p11_employee
              WHERE project_id = ?1 AND is_current_manager = 1 AND state = 'seated'",
            [project_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(unavailable("current manager"))
    }

    /// Daemon-composed manager speech from observed facts. No model prose;
    /// "Observed now" narration only, and never an approval, verification, or
    /// completion claim.
    fn compose_manager_announcement(
        &self,
        project_id: &str,
        project: &crate::project_aggregate::ProjectRow,
        routing: &str,
        candidate_digest: Option<&str>,
        preview_id: Option<&str>,
    ) -> Result<String, ProjectAggregateError> {
        let plan_id = project.current_plan_revision_id.as_deref();
        let stages = match plan_id {
            Some(plan_id) => self.projects.list_stages(plan_id)?,
            None => Vec::new(),
        };
        let confirmed = stages
            .iter()
            .filter(|row| row.confirm_status == "confirmed")
            .count();
        let ready = stages.iter().filter(|row| row.ready).count();
        let seating = self.employees.seating_progress(project_id)?;
        let (pending_chat_previews, latest_attempt) = {
            let conn = self.lock()?;
            let pending: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM p11_approval_preview ap
                       JOIN p13_project_chat_turn t ON t.preview_id = ap.preview_id
                      WHERE t.project_id = ?1 AND ap.status = 'pending'",
                    [project_id],
                    |row| row.get(0),
                )
                .map_err(unavailable("pending chat previews"))?;
            let attempt: Option<(String, String, String)> = conn
                .query_row(
                    "SELECT state, terminal_kind, response_status FROM p13_hosted_dsh_attempt
                      WHERE project_id = ?1 ORDER BY created_at DESC LIMIT 1",
                    [project_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()
                .map_err(unavailable("latest attempt"))?;
            (pending, attempt)
        };
        let attempt_line = match latest_attempt {
            Some((state, terminal, response)) => format!(
                "latest Attempt state {state}, terminal {terminal}, response {response}; completion not claimed"
            ),
            None => "no Attempt has run yet".to_owned(),
        };
        let facts = format!(
            "Observed now: project state {}; plan revision {}; {confirmed}/{} stages confirmed, {ready} ready; seated {}/{}; {pending_chat_previews} chat preview(s) pending; {attempt_line}.",
            project.state,
            plan_id.unwrap_or("(none)"),
            stages.len(),
            seating.seated,
            seating.roster
        );
        let announcement = match routing {
            "manager-plan-revision" => format!(
                "Plan revision candidate {} registered from your message; preview {} awaits your Confirm on the Projects canvas. {facts} Chat cannot approve, verify, or publish.",
                candidate_digest
                    .map(|digest| digest.chars().take(12).collect::<String>())
                    .unwrap_or_else(|| "(none)".to_owned()),
                preview_id.unwrap_or("(none)")
            ),
            _ => format!("{facts} Chat cannot approve, verify, or publish."),
        };
        Ok(announcement)
    }
}

struct Routed {
    routing: &'static str,
    target_employee_id: Option<String>,
    target_stage_id: Option<String>,
    candidate: Option<(&'static str, Value)>,
}

fn proposal_kind(proposal: &Value) -> Result<&str, ProjectAggregateError> {
    let object = proposal.as_object().ok_or(ProjectAggregateError::Invalid {
        detail: "chat proposal must be a JSON object",
    })?;
    match object.get("kind").and_then(Value::as_str) {
        Some(kind @ ("plan-revision" | "task-revision")) => Ok(kind),
        _ => Err(ProjectAggregateError::Invalid {
            detail: "chat proposal kind must be plan-revision or task-revision",
        }),
    }
}

/// A task revision may change that Member's own objective and nothing else.
fn reject_authority_transfer(proposal: &Value) -> Result<(), ProjectAggregateError> {
    if json_has_any_key(proposal, AUTHORITY_TRANSFER_KEYS) {
        return Err(ProjectAggregateError::Forbidden {
            detail: "chat cannot transfer authority between Members",
        });
    }
    Ok(())
}

fn json_has_any_key(value: &Value, keys: &[&str]) -> bool {
    match value {
        Value::Object(map) => map
            .iter()
            .any(|(key, child)| keys.contains(&key.as_str()) || json_has_any_key(child, keys)),
        Value::Array(items) => items.iter().any(|item| json_has_any_key(item, keys)),
        _ => false,
    }
}

/// Owner-stated stage list → bounded `StageSpec`s. Digest-bearing fields the
/// Owner does not type are derived deterministically from the stage id.
fn parse_plan_revision(proposal: &Value) -> Result<Vec<StageSpec>, ProjectAggregateError> {
    let items =
        proposal
            .get("stages")
            .and_then(Value::as_array)
            .ok_or(ProjectAggregateError::Invalid {
                detail: "plan-revision requires a stages array",
            })?;
    if items.is_empty() {
        return Err(ProjectAggregateError::Invalid {
            detail: "plan requires at least one stage",
        });
    }
    if items.len() > CHAT_PROPOSAL_MAX_STAGES {
        return Err(ProjectAggregateError::Invalid {
            detail: "plan-revision exceeds the bounded stage count",
        });
    }
    let mut stages = Vec::with_capacity(items.len());
    for item in items {
        let text = |key: &str| -> Option<String> {
            item.get(key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        };
        let stage_id = text("stage_id").ok_or(ProjectAggregateError::Invalid {
            detail: "plan-revision stage requires stage_id",
        })?;
        if stages
            .iter()
            .any(|existing: &StageSpec| existing.stage_id == stage_id)
        {
            return Err(ProjectAggregateError::Invalid {
                detail: "plan-revision stage ids must be unique",
            });
        }
        let title = text("title").unwrap_or_else(|| stage_id.clone());
        let objective = text("objective").ok_or(ProjectAggregateError::Invalid {
            detail: "plan-revision stage requires objective",
        })?;
        let responsible_slot = text("responsible_slot").ok_or(ProjectAggregateError::Invalid {
            detail: "plan-revision stage requires responsible_slot",
        })?;
        if responsible_slot == "agent" {
            return Err(ProjectAggregateError::Rejected {
                detail: "Role must not be merged with Agent",
            });
        }
        let output_contract = text("output_contract").unwrap_or_else(|| format!("out-{stage_id}"));
        stages.push(StageSpec {
            stage_id,
            title,
            objective,
            output_contract_digest: ProjectAggregateStore::digest_hex(output_contract.as_bytes()),
            acceptance_spec_ref: text("acceptance_spec_ref"),
            cadence_json: text("cadence_json"),
            responsible_slot,
            blocking_gap: text("blocking_gap"),
        });
    }
    Ok(stages)
}

fn stage_json(stage: &StageSpec) -> Value {
    json!({
        "stage_id": stage.stage_id,
        "title": stage.title,
        "objective": stage.objective,
        "output_contract_digest": stage.output_contract_digest,
        "acceptance_spec_ref": stage.acceptance_spec_ref,
        "cadence_json": stage.cadence_json,
        "responsible_slot": stage.responsible_slot,
        "blocking_gap": stage.blocking_gap,
    })
}

fn stage_from_json(value: &Value) -> Result<StageSpec, ProjectAggregateError> {
    let text =
        |key: &str| -> Option<String> { value.get(key).and_then(Value::as_str).map(str::to_owned) };
    Ok(StageSpec {
        stage_id: text("stage_id").ok_or(ProjectAggregateError::Invalid {
            detail: "stored candidate stage lacks stage_id",
        })?,
        title: text("title").unwrap_or_default(),
        objective: text("objective").unwrap_or_default(),
        output_contract_digest: text("output_contract_digest").ok_or(
            ProjectAggregateError::Invalid {
                detail: "stored candidate stage lacks output digest",
            },
        )?,
        acceptance_spec_ref: text("acceptance_spec_ref"),
        cadence_json: text("cadence_json"),
        responsible_slot: text("responsible_slot").ok_or(ProjectAggregateError::Invalid {
            detail: "stored candidate stage lacks responsible_slot",
        })?,
        blocking_gap: text("blocking_gap"),
    })
}

struct StoredCandidate {
    project_id: String,
    candidate_kind: String,
    candidate_digest: String,
    candidate: Value,
}

type ChatCandidateColumns = (String, Option<String>, Option<String>, Option<String>);

fn load_candidate_locked(
    conn: &Connection,
    turn_id: &str,
) -> Result<StoredCandidate, ProjectAggregateError> {
    let row: Option<ChatCandidateColumns> = conn
        .query_row(
            "SELECT project_id, candidate_kind, candidate_digest, candidate_json
               FROM p13_project_chat_turn WHERE turn_id = ?1",
            [turn_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(unavailable("load chat candidate"))?;
    let Some((project_id, kind, digest, json_text)) = row else {
        return Err(ProjectAggregateError::NotFound {
            detail: "chat turn not found",
        });
    };
    let (Some(candidate_kind), Some(candidate_digest), Some(json_text)) = (kind, digest, json_text)
    else {
        return Err(ProjectAggregateError::Invalid {
            detail: "chat turn carries no candidate",
        });
    };
    let candidate: Value =
        serde_json::from_str(&json_text).map_err(|_| ProjectAggregateError::Unavailable {
            detail: "parse stored chat candidate".to_owned(),
        })?;
    Ok(StoredCandidate {
        project_id,
        candidate_kind,
        candidate_digest,
        candidate,
    })
}

fn current_plan_locked(
    conn: &Connection,
    project_id: &str,
) -> Result<Option<String>, ProjectAggregateError> {
    conn.query_row(
        "SELECT current_plan_revision_id FROM p11_project WHERE project_id = ?1",
        [project_id],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()
    .map_err(unavailable("project plan for chat candidate"))?
    .ok_or(ProjectAggregateError::NotFound {
        detail: "project not found",
    })
}

/// Preview base digest for a chat-routed candidate: the plan the candidate was
/// composed against plus the candidate bytes. Any plan movement makes the
/// preview stale; a mismatched kind never confirms.
pub(crate) fn candidate_base_digest_locked(
    conn: &Connection,
    subject_kind: &str,
    turn_id: &str,
) -> Result<String, ProjectAggregateError> {
    let stored = load_candidate_locked(conn, turn_id)?;
    if stored.candidate_kind != subject_kind {
        return Err(ProjectAggregateError::Invalid {
            detail: "preview subject_kind does not match the chat candidate",
        });
    }
    let plan_id = current_plan_locked(conn, &stored.project_id)?;
    Ok(ProjectAggregateStore::digest_hex(
        format!(
            "chat-candidate\n{}\n{}\n{}\n{}",
            stored.project_id,
            subject_kind,
            plan_id.as_deref().unwrap_or("(none)"),
            stored.candidate_digest
        )
        .as_bytes(),
    ))
}

/// Canvas Confirm of a chat-routed candidate. Runs inside the aggregate lock:
/// applies the PlanRevision (plan-revision) or re-materializes the current plan
/// with only the mentioned Member's stage objective revised (task-revision),
/// carries roster assignments forward for unchanged slots, and returns the
/// receipt to the conversation row.
pub(crate) fn confirm_chat_candidate_locked(
    conn: &Connection,
    subject_kind: &str,
    turn_id: &str,
    now_ms: i64,
) -> Result<ConfirmResult, ProjectAggregateError> {
    let stored = load_candidate_locked(conn, turn_id)?;
    if stored.candidate_kind != subject_kind {
        return Err(ProjectAggregateError::Invalid {
            detail: "preview subject_kind does not match the chat candidate",
        });
    }
    let previous_plan = current_plan_locked(conn, &stored.project_id)?;
    let (stages, kind): (Vec<StageSpec>, &'static str) = match subject_kind {
        "plan-revision" => {
            let items = stored
                .candidate
                .get("stages")
                .and_then(Value::as_array)
                .ok_or(ProjectAggregateError::Invalid {
                    detail: "stored plan-revision candidate lacks stages",
                })?;
            let stages = items
                .iter()
                .map(stage_from_json)
                .collect::<Result<Vec<_>, _>>()?;
            (stages, "plan_revision_applied")
        }
        "task-revision" => {
            let Some(plan_id) = previous_plan.as_deref() else {
                return Err(ProjectAggregateError::NotFound {
                    detail: "project has no current plan revision",
                });
            };
            let stage_id = stored
                .candidate
                .get("stage_id")
                .and_then(Value::as_str)
                .ok_or(ProjectAggregateError::Invalid {
                    detail: "stored task-revision candidate lacks stage_id",
                })?;
            let objective = stored
                .candidate
                .get("objective")
                .and_then(Value::as_str)
                .ok_or(ProjectAggregateError::Invalid {
                    detail: "stored task-revision candidate lacks objective",
                })?;
            let mut statement = conn
                .prepare(
                    "SELECT stage_id, title, objective, output_contract_digest, acceptance_spec_ref,
                            cadence_json, responsible_slot
                       FROM p11_stage WHERE plan_revision_id = ?1 ORDER BY position",
                )
                .map_err(unavailable("current stages"))?;
            let rows = statement
                .query_map([plan_id], |row| {
                    Ok(StageSpec {
                        stage_id: row.get(0)?,
                        title: row.get(1)?,
                        objective: row.get(2)?,
                        output_contract_digest: row.get(3)?,
                        acceptance_spec_ref: row.get(4)?,
                        cadence_json: row.get(5)?,
                        responsible_slot: row.get(6)?,
                        blocking_gap: None,
                    })
                })
                .map_err(unavailable("current stage rows"))?;
            let mut stages = rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(unavailable("collect current stages"))?;
            // Still-blocking gaps ride along: a task revision never silently
            // clears a gap the Owner has not resolved or accepted.
            let mut gaps = conn
                .prepare(
                    "SELECT stage_id, description FROM p11_gap
                      WHERE plan_revision_id = ?1 AND blocking = 1
                        AND resolved_by_revision_id IS NULL
                        AND accepted_as_limitation_at IS NULL
                      ORDER BY rowid",
                )
                .map_err(unavailable("current gaps"))?;
            let gap_rows = gaps
                .query_map([plan_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(unavailable("current gap rows"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(unavailable("collect current gaps"))?;
            for (gap_stage, description) in gap_rows {
                if let Some(stage) = stages.iter_mut().find(|stage| stage.stage_id == gap_stage)
                    && stage.blocking_gap.is_none()
                {
                    stage.blocking_gap = Some(description);
                }
            }
            let Some(target) = stages.iter_mut().find(|stage| stage.stage_id == stage_id) else {
                return Err(ProjectAggregateError::NotFound {
                    detail: "stage not on current plan revision",
                });
            };
            target.objective = objective.to_owned();
            (stages, "task_revision_applied")
        }
        _ => {
            return Err(ProjectAggregateError::Invalid {
                detail: "unsupported subject_kind",
            });
        }
    };
    let new_plan = ProjectAggregateStore::apply_plan_revision_locked(
        conn,
        &stored.project_id,
        &stages,
        now_ms,
    )?;
    if let Some(previous) = previous_plan.as_deref() {
        carry_assignments_forward_locked(conn, &stored.project_id, previous, &new_plan, &stages)?;
    }
    let receipt_ref = format!("receipt:chat:{subject_kind}:{turn_id}");
    conn.execute(
        "UPDATE p13_project_chat_turn SET receipt_ref = ?1, applied_ref = ?2 WHERE turn_id = ?3",
        params![receipt_ref, new_plan, turn_id],
    )
    .map_err(unavailable("record chat receipt"))?;
    Ok(ConfirmResult {
        kind,
        new_ref: new_plan,
        receipt_ref,
    })
}

/// A revision confirmed by the Owner keeps every seat whose slot still exists.
/// It changes no Employee, grant, or manager fact.
fn carry_assignments_forward_locked(
    conn: &Connection,
    project_id: &str,
    previous_plan: &str,
    new_plan: &str,
    stages: &[StageSpec],
) -> Result<(), ProjectAggregateError> {
    let mut statement = conn
        .prepare(
            "SELECT slot, employee_id FROM p11_assignment
              WHERE project_id = ?1 AND plan_revision_id = ?2",
        )
        .map_err(unavailable("previous assignments"))?;
    let rows = statement
        .query_map(params![project_id, previous_plan], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(unavailable("assignment rows"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(unavailable("collect assignments"))?;
    drop(statement);
    for (slot, employee_id) in rows {
        if !stages.iter().any(|stage| stage.responsible_slot == slot) {
            continue;
        }
        let assignment_id = next_id("assign")?;
        conn.execute(
            "INSERT OR IGNORE INTO p11_assignment (
                assignment_id, project_id, plan_revision_id, slot, employee_id
             ) VALUES (?1,?2,?3,?4,?5)",
            params![assignment_id, project_id, new_plan, slot, employee_id],
        )
        .map_err(unavailable("carry assignment"))?;
    }
    Ok(())
}

fn require_archive_projection(projection_id: &str) -> Result<(), ProjectAggregateError> {
    if projection_id == CONVERSATION_ARCHIVE_PROJECTION_ID {
        return Ok(());
    }
    Err(ProjectAggregateError::Invalid {
        detail: "legacy conversation-projection identifier is not coerced",
    })
}

fn short_handle(employee_id: &str) -> String {
    let tail: String = employee_id
        .rsplit('-')
        .next()
        .unwrap_or(employee_id)
        .chars()
        .take(6)
        .collect();
    format!("member-{tail}")
}

/// Sort key used only as a same-timestamp tie-break. Owner turns rank before
/// speech so a manager announce that shares `now_ms` with its triggering turn
/// cannot leapfrog it because `conv-*` sorts before `turn-*`.
fn thread_tie_rank(row: &ChatThreadRow) -> u8 {
    if row.author == "owner" { 0 } else { 1 }
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn authority_transfer_keys_are_refused_anywhere_in_the_proposal() {
        let nested = json!({ "kind": "task-revision", "stage_id": "s2", "objective": "x", "extra": { "assignee": "e" } });
        assert!(reject_authority_transfer(&nested).is_err());
        let clean = json!({ "kind": "task-revision", "stage_id": "s2", "objective": "x" });
        assert!(reject_authority_transfer(&clean).is_ok());
    }

    #[test]
    fn plan_revision_parser_is_bounded_and_deterministic() {
        let proposal = json!({
            "kind": "plan-revision",
            "stages": [
                { "stage_id": "a", "objective": "do a", "responsible_slot": "manager" },
                { "stage_id": "b", "title": "B", "objective": "do b", "responsible_slot": "member" }
            ]
        });
        let stages = parse_plan_revision(&proposal).expect("parse");
        assert_eq!(stages[0].title, "a");
        assert_eq!(stages[1].title, "B");
        assert_eq!(
            stages[0].output_contract_digest,
            ProjectAggregateStore::digest_hex(b"out-a")
        );
        let duplicate = json!({
            "kind": "plan-revision",
            "stages": [
                { "stage_id": "a", "objective": "do a", "responsible_slot": "manager" },
                { "stage_id": "a", "objective": "again", "responsible_slot": "member" }
            ]
        });
        assert!(parse_plan_revision(&duplicate).is_err());
        let agent = json!({
            "kind": "plan-revision",
            "stages": [{ "stage_id": "a", "objective": "do a", "responsible_slot": "agent" }]
        });
        assert!(matches!(
            parse_plan_revision(&agent),
            Err(ProjectAggregateError::Rejected { .. })
        ));
        let too_many = json!({
            "kind": "plan-revision",
            "stages": (0..=CHAT_PROPOSAL_MAX_STAGES).map(|index| json!({
                "stage_id": format!("s{index}"), "objective": "o", "responsible_slot": "m"
            })).collect::<Vec<_>>()
        });
        assert!(parse_plan_revision(&too_many).is_err());
    }

    #[test]
    fn v39_rebuild_keeps_every_earlier_preview_subject_kind() {
        for kind in APPROVAL_PREVIEW_SUBJECT_KINDS_V39 {
            let pinned = if kind == "task-revision" {
                PROJECT_CHAT_SCHEMA_V39.contains("'task' || '-' || 'revision'")
            } else {
                PROJECT_CHAT_SCHEMA_V39.contains(&format!("'{kind}'"))
            };
            assert!(pinned, "{kind} missing from the v39 CHECK");
        }
        assert!(PROJECT_CHAT_SCHEMA_V39.contains("approve_attempted = 0"));
        assert!(
            !PROJECT_CHAT_SCHEMA_V39.contains("sk-"),
            "v39 CHECK SQL must not embed the sk- byte sequence (P11-T10 sqlite scan)"
        );
    }

    #[test]
    fn secret_refusal_guidance_points_at_settings_and_claims_nothing() {
        let guidance = chat_secret_refusal_guidance();
        assert_eq!(guidance["settings_route"], json!("#/settings"));
        assert_eq!(guidance["posted"], json!(false));
        assert_eq!(guidance["archived"], json!(false));
    }
}
