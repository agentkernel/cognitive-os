//! P13-T07 Knowledge/Memory authority helpers on existing Memory tables.
//!
//! Chat auto-admission and cross-Project promote reuse `memory_candidates` /
//! `memory_admission_decisions` / `memory_objects` (no new numbered migration;
//! T06 owns `personal_db.rs` v39). Files are not Project authority.

use crate::conversation::ConversationStore;
use crate::employee::EmployeeStore;
use crate::memory_privacy::screen_memory_admission;
use crate::project_aggregate::{ConfirmCaller, ProjectAggregateError};
use crate::sqlite::SqliteAuthorityStore;
use cognitive_contracts::generated::context_view::{
    LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
};
use cognitive_domain::ObjectId;
use cognitive_kernel::authz::ObjectGovernance;
use cognitive_kernel::intent_chain::seal_governed_object_content_digest;
use cognitive_kernel::ports::{
    ContextStore, MemoryAdmissionDecisionRow, MemoryCandidateRow, MemoryObjectRow, MemoryStore,
    StorePortError, WorkspaceContextSourceRow,
};
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Pending cross-Project promote candidate purpose (existing Memory tables).
pub const PROMOTE_PURPOSE: &str = "cross-project-promote";
/// Confirmed copy purpose. Distinct so an unconfirmed preview cannot list as copied.
pub const PROMOTE_PURPOSE_COPY: &str = "cross-project-promote-copy";
const AUTO_ADMIT_PURPOSE: &str = "chat-auto-admission";
const AUTO_ADMIT_PROVENANCE: &str = "management://personal/memory/auto-admit.chat";
const PROMOTE_PROVENANCE: &str = "management://personal/memory/promote";

/// Owner-management chat admission receipt. Inspectable Memory, not Vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatAdmission {
    pub memory_id: String,
}

/// Cross-Project promote preview or confirmed copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryPromoteRow {
    pub promote_id: String,
    pub memory_id: String,
    pub from_project_id: String,
    pub to_project_id: String,
    pub preview_digest: String,
    pub status: String,
    pub promoted_memory_id: Option<String>,
}

/// Knowledge/Memory authority writer (P13-T07). Shares the daemon SQLite.
#[derive(Clone)]
pub struct KnowledgeMemoryStore {
    authority: SqliteAuthorityStore,
}

impl KnowledgeMemoryStore {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            authority: store.clone(),
        }
    }

    /// Open the authority database path (tests / CLI-free helpers).
    pub fn open_path(path: &Path) -> Result<Self, ProjectAggregateError> {
        let authority = SqliteAuthorityStore::open(path).map_err(store_err)?;
        Ok(Self { authority })
    }

    /// Owner-only: archive record → admitted Memory. Assistant cannot self-admit.
    pub fn auto_admit_chat(
        &self,
        caller: ConfirmCaller,
        projection_id: &str,
        project_id: &str,
        record_id: &str,
        now_ms: i64,
    ) -> Result<ChatAdmission, ProjectAggregateError> {
        require_owner(caller)?;
        let conversations = ConversationStore::from_authority_store(&self.authority);
        let record = conversations.read_record(projection_id, project_id, record_id)?;
        screen_memory_admission(&record.body_redacted, "{}")?;
        let employees = EmployeeStore::from_authority_store(&self.authority);
        crate::memory_privacy::require_employee_in_project(
            &employees,
            project_id,
            &record.employee_id,
        )?;
        let memory_id = self.admit_text(
            project_id,
            &record.employee_id,
            &record.body_redacted,
            AUTO_ADMIT_PURPOSE,
            AUTO_ADMIT_PROVENANCE,
            json!({
                "kind": AUTO_ADMIT_PURPOSE,
                "archive_record_id": record.record_id,
                "project_id": project_id,
            }),
            now_ms,
        )?;
        Ok(ChatAdmission { memory_id })
    }

    /// Owner preview for copying one admitted Memory into another Project.
    pub fn request_promote(
        &self,
        caller: ConfirmCaller,
        memory_id: &str,
        from_project_id: &str,
        to_project_id: &str,
        to_employee_id: &str,
        now_ms: i64,
    ) -> Result<MemoryPromoteRow, ProjectAggregateError> {
        require_owner(caller)?;
        if from_project_id == to_project_id {
            return Err(ProjectAggregateError::Invalid {
                detail: "cross-Project promote requires two Projects",
            });
        }
        self.require_live_memory(memory_id)?;
        let employees = EmployeeStore::from_authority_store(&self.authority);
        crate::memory_privacy::require_employee_in_project(
            &employees,
            to_project_id,
            to_employee_id,
        )?;
        let preview_digest =
            promote_digest(memory_id, from_project_id, to_project_id, to_employee_id);
        let payload = json!({
            "kind": PROMOTE_PURPOSE,
            "memory_id": memory_id,
            "from_project_id": from_project_id,
            "to_project_id": to_project_id,
            "to_employee_id": to_employee_id,
            "preview_digest": preview_digest,
            "status": "pending",
        });
        let promote_id = self.admit_review(
            from_project_id,
            to_employee_id,
            PROMOTE_PURPOSE,
            payload,
            now_ms,
        )?;
        Ok(MemoryPromoteRow {
            promote_id,
            memory_id: memory_id.to_owned(),
            from_project_id: from_project_id.to_owned(),
            to_project_id: to_project_id.to_owned(),
            preview_digest,
            status: "pending".to_owned(),
            promoted_memory_id: None,
        })
    }

    /// Owner confirm copies Memory into the target Project. Digest-bound.
    pub fn confirm_promote(
        &self,
        caller: ConfirmCaller,
        promote_id: &str,
        preview_digest: &str,
        now_ms: i64,
    ) -> Result<MemoryPromoteRow, ProjectAggregateError> {
        require_owner(caller)?;
        let pending = self
            .load_promote(promote_id)?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "promote preview not found",
            })?;
        if pending.status != "pending" {
            return Err(ProjectAggregateError::Conflict {
                detail: "promote preview is not pending",
            });
        }
        if pending.preview_digest != preview_digest {
            return Err(ProjectAggregateError::Conflict {
                detail: "promote preview digest does not match",
            });
        }
        if self
            .list_promotes(&pending.to_project_id)?
            .iter()
            .any(|row| row.promote_id == promote_id && row.status == "confirmed")
        {
            return Err(ProjectAggregateError::Conflict {
                detail: "promote already confirmed",
            });
        }
        self.require_live_memory(&pending.memory_id)?;
        let to_employee_id = self.promote_employee(promote_id)?;
        let source = self.load_memory_text(&pending.memory_id)?;
        screen_memory_admission(&source, "{}")?;
        let promoted_memory_id = self.admit_text(
            &pending.to_project_id,
            &to_employee_id,
            &source,
            PROMOTE_PURPOSE_COPY,
            PROMOTE_PROVENANCE,
            json!({
                "kind": PROMOTE_PURPOSE_COPY,
                "parent_promote_id": promote_id,
                "memory_id": pending.memory_id,
                "from_project_id": pending.from_project_id,
                "to_project_id": pending.to_project_id,
                "preview_digest": preview_digest,
                "status": "confirmed",
            }),
            now_ms,
        )?;
        Ok(MemoryPromoteRow {
            promote_id: pending.promote_id,
            memory_id: pending.memory_id,
            from_project_id: pending.from_project_id,
            to_project_id: pending.to_project_id,
            preview_digest: pending.preview_digest,
            status: "confirmed".to_owned(),
            promoted_memory_id: Some(promoted_memory_id),
        })
    }

    /// Promotes that mention `project_id` as source or target.
    pub fn list_promotes(
        &self,
        project_id: &str,
    ) -> Result<Vec<MemoryPromoteRow>, ProjectAggregateError> {
        let conn = self
            .authority
            .conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })?;
        let mut statement = conn
            .prepare(
                "SELECT c.candidate_id, c.purpose, c.canonical_json, d.decision, o.memory_id
                   FROM memory_candidates c
                   JOIN memory_admission_decisions d ON d.candidate_id = c.candidate_id
                   LEFT JOIN memory_objects o ON o.candidate_id = c.candidate_id
                  WHERE c.purpose IN (?1, ?2)
                  ORDER BY c.observed_at_unix_seconds, c.candidate_id",
            )
            .map_err(unavailable("prepare promotes"))?;
        let rows = statement
            .query_map(params![PROMOTE_PURPOSE, PROMOTE_PURPOSE_COPY], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            })
            .map_err(unavailable("query promotes"))?;
        let mut out = Vec::new();
        for row in rows {
            let (candidate_id, purpose, canonical, decision, memory_object) =
                row.map_err(unavailable("promote row"))?;
            let Some(parsed) = parse_promote(
                &candidate_id,
                &purpose,
                &canonical,
                &decision,
                memory_object,
            ) else {
                continue;
            };
            if parsed.from_project_id == project_id || parsed.to_project_id == project_id {
                out.push(parsed);
            }
        }
        Ok(out)
    }

    fn require_live_memory(&self, memory_id: &str) -> Result<(), ProjectAggregateError> {
        let object_id = parse_object_id(memory_id)?;
        let found = self
            .authority
            .load_memory_object(&object_id)
            .map_err(store_err)?;
        if found.is_none() {
            return Err(ProjectAggregateError::NotFound {
                detail: "Memory object not found",
            });
        }
        let conn = self
            .authority
            .conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })?;
        let tombstones: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM memory_tombstones WHERE memory_id = ?1",
                [memory_id],
                |row| row.get(0),
            )
            .map_err(unavailable("count tombstone"))?;
        if tombstones > 0 {
            return Err(ProjectAggregateError::Invalid {
                detail: "tombstoned Memory cannot be promoted or resurrected",
            });
        }
        Ok(())
    }

    fn load_promote(
        &self,
        promote_id: &str,
    ) -> Result<Option<MemoryPromoteRow>, ProjectAggregateError> {
        let conn = self
            .authority
            .conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })?;
        let row: Option<(String, String, String)> = conn
            .query_row(
                "SELECT c.purpose, c.canonical_json, d.decision
                   FROM memory_candidates c
                   JOIN memory_admission_decisions d ON d.candidate_id = c.candidate_id
                  WHERE c.candidate_id = ?1",
                [promote_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(unavailable("load promote"))?;
        Ok(row.and_then(|(purpose, canonical, decision)| {
            parse_promote(promote_id, &purpose, &canonical, &decision, None)
        }))
    }

    fn promote_employee(&self, promote_id: &str) -> Result<String, ProjectAggregateError> {
        let conn = self
            .authority
            .conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })?;
        let canonical: String = conn
            .query_row(
                "SELECT canonical_json FROM memory_candidates WHERE candidate_id = ?1",
                [promote_id],
                |row| row.get(0),
            )
            .map_err(unavailable("promote employee"))?;
        payload_string(&canonical, "to_employee_id").ok_or(ProjectAggregateError::Invalid {
            detail: "promote preview is missing to_employee_id",
        })
    }

    fn load_memory_text(&self, memory_id: &str) -> Result<String, ProjectAggregateError> {
        let object_id = parse_object_id(memory_id)?;
        let object = self
            .authority
            .load_memory_object(&object_id)
            .map_err(store_err)?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "Memory object not found",
            })?;
        let parsed: Value = serde_json::from_str(&object.canonical_json).unwrap_or(json!({}));
        if let Some(text) = parsed.get("text").and_then(Value::as_str) {
            return Ok(text.to_owned());
        }
        if let Some(text) = parsed
            .get("body")
            .and_then(|body| body.get("text"))
            .and_then(Value::as_str)
        {
            return Ok(text.to_owned());
        }
        Ok(object.canonical_json)
    }

    fn admit_review(
        &self,
        project_id: &str,
        employee_id: &str,
        purpose: &str,
        extra: Value,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        let (candidate, decision, _) =
            self.prepare_admission(project_id, employee_id, "", purpose, extra, now_ms, false)?;
        self.authority
            .append_memory_admission(&candidate, &decision, None)
            .map_err(store_err)?;
        Ok(candidate.candidate_id.to_string())
    }

    fn admit_text(
        &self,
        project_id: &str,
        employee_id: &str,
        text: &str,
        purpose: &str,
        provenance: &str,
        extra: Value,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        let mut extra = extra;
        if let Some(object) = extra.as_object_mut() {
            object.insert("text".to_owned(), json!(text));
            object.insert("provenance_ref".to_owned(), json!(provenance));
        }
        let (candidate, decision, object) =
            self.prepare_admission(project_id, employee_id, text, purpose, extra, now_ms, true)?;
        self.authority
            .append_memory_admission(&candidate, &decision, Some(&object))
            .map_err(store_err)?;
        Ok(object.memory_id.to_string())
    }

    fn prepare_admission(
        &self,
        project_id: &str,
        employee_id: &str,
        text: &str,
        purpose: &str,
        extra: Value,
        now_ms: i64,
        with_object: bool,
    ) -> Result<
        (
            MemoryCandidateRow,
            MemoryAdmissionDecisionRow,
            MemoryObjectRow,
        ),
        ProjectAggregateError,
    > {
        let scope = crate::memory_privacy::canonical_episodic_scope(project_id, employee_id);
        let source_id = next_object_id()?;
        let candidate_id = next_object_id()?;
        let decision_id = next_object_id()?;
        let memory_id = next_object_id()?;
        let source = source_row(&source_id, &scope, text, purpose)?;
        self.authority
            .append_workspace_context_source(&source)
            .map_err(store_err)?;
        let mut candidate_payload = extra.clone();
        if let Some(object) = candidate_payload.as_object_mut() {
            object.insert(
                "header".to_owned(),
                governed_header_value(&candidate_id, "MemoryCandidate", "cognitiveos.memory/0.1"),
            );
            object.insert("source_id".to_owned(), json!(source_id.to_string()));
            object.insert("source_digest".to_owned(), json!(source.source_digest));
            object.insert(
                "source_provenance_ref".to_owned(),
                json!(source.provenance_ref),
            );
            object.insert("governance_scope".to_owned(), json!(scope));
            object.insert("target_scope".to_owned(), json!(scope));
            object.insert("purpose".to_owned(), json!(purpose));
            object.insert(
                "retention_expires_at_unix_seconds".to_owned(),
                json!((now_ms / 1000).saturating_add(31_536_000)),
            );
            object.insert("observed_at_unix_seconds".to_owned(), json!(now_ms / 1000));
        }
        let (sealed_candidate, candidate_digest) =
            seal_governed_object_content_digest(candidate_payload).map_err(|error| {
                ProjectAggregateError::Unavailable {
                    detail: format!("seal Memory candidate: {error}"),
                }
            })?;
        let candidate = MemoryCandidateRow {
            candidate_id: candidate_id.clone(),
            candidate_digest: candidate_digest.clone(),
            source_id: source.source_id.clone(),
            source_digest: source.source_digest.clone(),
            source_provenance_ref: source.provenance_ref.clone(),
            governance_scope: scope.clone(),
            target_scope: scope,
            purpose: purpose.to_owned(),
            retention_expires_at_unix_seconds: (now_ms / 1000).saturating_add(31_536_000),
            observed_at_unix_seconds: now_ms / 1000,
            canonical_json: serde_json::to_string(&sealed_candidate).map_err(|error| {
                ProjectAggregateError::Unavailable {
                    detail: format!("serialize Memory candidate: {error}"),
                }
            })?,
        };
        let decision = MemoryAdmissionDecisionRow {
            decision_id: decision_id.clone(),
            candidate_id: candidate_id.clone(),
            candidate_digest,
            decision: if with_object {
                "admit".to_owned()
            } else {
                "review".to_owned()
            },
            policy_version: 1,
            reason_codes_json: if with_object {
                "[\"MEMORY_ADMISSION_ACCEPTED\"]".to_owned()
            } else {
                "[\"MEMORY_PROMOTE_PREVIEW\"]".to_owned()
            },
            canonical_json: json!({
                "decision_id": decision_id.to_string(),
                "decision": if with_object { "admit" } else { "review" },
            })
            .to_string(),
        };
        let object = MemoryObjectRow {
            memory_id: memory_id.clone(),
            candidate_id,
            decision_id,
            canonical_json: extra.to_string(),
        };
        Ok((candidate, decision, object))
    }
}

fn governed_header_value(identifier: &ObjectId, object_type: &str, schema_version: &str) -> Value {
    json!({
        "id": identifier.as_str(),
        "type": object_type,
        "schema_version": schema_version,
        "object_version": 1,
        "scope_domain": "tenant",
        "tenant_id": "00000000-0000-7000-9000-0000000000f1",
        "resource_scope_ref": {
            "kind": "strong",
            "id": "00000000-0000-7000-9000-000000000101",
            "object_version": 1,
            "content_digest": format!("sha256:{}", "a".repeat(64))
        },
        "owner_ref": {
            "kind": "strong",
            "id": "00000000-0000-7000-9000-000000000102",
            "object_version": 1,
            "content_digest": format!("sha256:{}", "a".repeat(64))
        },
        "authority_ref": {
            "kind": "strong",
            "id": "00000000-0000-7000-9000-000000000103",
            "object_version": 1,
            "content_digest": format!("sha256:{}", "a".repeat(64))
        },
        "policy_refs": [],
        "purpose_constraints": ["memory_admission"],
        "sensitivity": "internal",
        "compartments": [],
        "retention": { "policy": "standard", "expires_at": null, "legal_hold": false },
        "provenance": { "created_by": "principal://local/owner", "source_refs": [] },
        "lineage": { "parents": [], "transform": "p13-t07-knowledge-memory" },
        "content_digest": format!("sha256:{}", "0".repeat(64)),
        "created_at": "2026-09-03T00:00:00Z",
        "valid_time": { "from": "2026-09-03T00:00:00Z", "until": null }
    })
}

fn parse_promote(
    candidate_id: &str,
    purpose: &str,
    canonical: &str,
    decision: &str,
    memory_object: Option<String>,
) -> Option<MemoryPromoteRow> {
    let value: Value = serde_json::from_str(canonical).ok()?;
    let extra = value.get("extra").cloned().unwrap_or(value);
    let from = extra.get("from_project_id")?.as_str()?.to_owned();
    let to = extra.get("to_project_id")?.as_str()?.to_owned();
    let memory_id = extra.get("memory_id")?.as_str()?.to_owned();
    let preview_digest = extra
        .get("preview_digest")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    let parent = extra
        .get("parent_promote_id")
        .and_then(Value::as_str)
        .unwrap_or(candidate_id);
    if purpose == PROMOTE_PURPOSE_COPY
        || decision == "admit" && extra.get("parent_promote_id").is_some()
    {
        return Some(MemoryPromoteRow {
            promote_id: parent.to_owned(),
            memory_id,
            from_project_id: from,
            to_project_id: to,
            preview_digest,
            status: "confirmed".to_owned(),
            promoted_memory_id: memory_object.or_else(|| {
                extra
                    .get("promoted_memory_id")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            }),
        });
    }
    if purpose != PROMOTE_PURPOSE {
        return None;
    }
    Some(MemoryPromoteRow {
        promote_id: candidate_id.to_owned(),
        memory_id,
        from_project_id: from,
        to_project_id: to,
        preview_digest,
        status: "pending".to_owned(),
        promoted_memory_id: None,
    })
}

fn payload_string(canonical: &str, field: &str) -> Option<String> {
    let value: Value = serde_json::from_str(canonical).ok()?;
    let extra = value.get("extra").cloned().unwrap_or(value);
    extra.get(field)?.as_str().map(ToOwned::to_owned)
}

fn source_row(
    identifier: &ObjectId,
    scope: &str,
    text: &str,
    purpose: &str,
) -> Result<WorkspaceContextSourceRow, ProjectAggregateError> {
    let content_bytes = i64::try_from(text.len()).unwrap_or(i64::MAX);
    let content_tokens = i64::try_from(text.split_whitespace().count()).unwrap_or(i64::MAX);
    let payload = json!({
        "header": governed_header_value(identifier, "WorkspaceContextSource", "cognitiveos.context/0.1"),
        "tenant_id": "personal",
        "owner_ref": "principal://local/owner",
        "resource_scope": scope,
        "conversation_ref": null,
        "role": LoadedContextItemRole::Working,
        "trust_level": LoadedContextItemTrustLevel::Verified,
        "representation": LoadedContextItemRepresentation::Text,
        "provenance_ref": purpose,
        "content_bytes": content_bytes,
        "content_tokens": content_tokens,
        "body": { "text": text },
    });
    let (sealed, source_digest) =
        seal_governed_object_content_digest(payload).map_err(|error| {
            ProjectAggregateError::Unavailable {
                detail: format!("seal Context source: {error}"),
            }
        })?;
    Ok(WorkspaceContextSourceRow {
        source_id: identifier.clone(),
        source_digest,
        governance: ObjectGovernance {
            object_ref: identifier.as_str().to_owned(),
            tenant_id: Some("personal".to_owned()),
            owner_ref: "principal://local/owner".to_owned(),
            resource_scope: scope.to_owned(),
            conversation_ref: None,
        },
        role: LoadedContextItemRole::Working,
        trust_level: LoadedContextItemTrustLevel::Verified,
        representation: LoadedContextItemRepresentation::Text,
        provenance_ref: purpose.to_owned(),
        content_bytes,
        content_tokens: Some(content_tokens),
        canonical_json: serde_json::to_string(&sealed).map_err(|error| {
            ProjectAggregateError::Unavailable {
                detail: format!("serialize Context source: {error}"),
            }
        })?,
    })
}

fn promote_digest(memory_id: &str, from: &str, to: &str, employee: &str) -> String {
    format!(
        "{:x}",
        Sha256::digest(format!("{memory_id}\n{from}\n{to}\n{employee}").as_bytes())
    )
}

fn next_object_id() -> Result<ObjectId, ProjectAggregateError> {
    let raw = uuid::Uuid::now_v7().as_hyphenated().to_string();
    ObjectId::parse(&raw).map_err(|error| ProjectAggregateError::Unavailable {
        detail: format!("object id: {error}"),
    })
}

fn parse_object_id(value: &str) -> Result<ObjectId, ProjectAggregateError> {
    ObjectId::parse(value).map_err(|_| ProjectAggregateError::Invalid {
        detail: "memory_id is not an ObjectId",
    })
}

fn require_owner(caller: ConfirmCaller) -> Result<(), ProjectAggregateError> {
    match caller {
        ConfirmCaller::OwnerManagement => Ok(()),
        ConfirmCaller::TaskChannel | ConfirmCaller::Assistant => {
            Err(ProjectAggregateError::Forbidden {
                detail: "only owner management session may admit or promote Memory",
            })
        }
    }
}

fn store_err(error: StorePortError) -> ProjectAggregateError {
    match error {
        StorePortError::Conflict { .. } => ProjectAggregateError::Conflict {
            detail: "Memory store conflict",
        },
        StorePortError::Unavailable { detail } => ProjectAggregateError::Unavailable { detail },
    }
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}
