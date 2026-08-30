//! Scoped episodic Memory privacy, admission screening, and forget/rebuild.
//!
//! Reuses the existing Memory authority tables, `admit_memory_candidate`, and
//! FTS search. Does not add a second Memory store, a Letta/Mem0 write path, or
//! Agent self-admission.

use crate::employee::EmployeeStore;
use crate::project_aggregate::ProjectAggregateError;
use cognitive_domain::ObjectId;
use cognitive_kernel::ports::{
    MemorySearchCandidateRow, MemorySearchQuery, MemoryStore, MemoryTombstoneRow, StorePortError,
};
use rusqlite::OptionalExtension;
use serde_json::Value;
use std::sync::MutexGuard;

use crate::sqlite::SqliteAuthorityStore;

/// Canonical Personal 2.0.0 episodic Memory scope (project × employee).
pub fn canonical_episodic_scope(project_id: &str, employee_id: &str) -> String {
    format!("opc://project/{project_id}/employee/{employee_id}")
}

/// Caller/target pair for fail-closed episodic recall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpisodicRecallSpec<'a> {
    pub caller_project_id: &'a str,
    pub target_project_id: &'a str,
    pub caller_employee_id: &'a str,
    pub target_employee_id: &'a str,
    pub query_text: &'a str,
    pub purpose: &'a str,
    pub observed_at_unix_seconds: i64,
    pub maximum_results: usize,
}

/// Reject secret/PII-shaped text and Letta/Mem0/Agent-self direct writes.
pub fn screen_memory_admission(
    text: &str,
    envelope_json: &str,
) -> Result<(), ProjectAggregateError> {
    reject_secret_or_pii_shape(text)?;
    reject_secret_or_pii_shape(envelope_json)?;
    reject_direct_or_self_admission(envelope_json)?;
    Ok(())
}

/// Employee must already exist in the named Project.
pub fn require_employee_in_project(
    employees: &EmployeeStore,
    project_id: &str,
    employee_id: &str,
) -> Result<(), ProjectAggregateError> {
    if project_id.trim().is_empty() || employee_id.trim().is_empty() {
        return Err(ProjectAggregateError::Invalid {
            detail: "Memory project_id and employee_id are required together",
        });
    }
    match employees.get_employee(employee_id)? {
        Some(row) if row.project_id == project_id => Ok(()),
        Some(_) | None => Err(ProjectAggregateError::Forbidden {
            detail: "cross-scope Memory access is forbidden",
        }),
    }
}

/// Caller and target project/employee must be identical; employee must sit in
/// that project. Returns the canonical governance_scope used for FTS.
pub fn require_episodic_recall_scope(
    employees: &EmployeeStore,
    spec: &EpisodicRecallSpec<'_>,
) -> Result<String, ProjectAggregateError> {
    if spec.caller_project_id.trim().is_empty()
        || spec.target_project_id.trim().is_empty()
        || spec.caller_employee_id.trim().is_empty()
        || spec.target_employee_id.trim().is_empty()
        || spec.query_text.trim().is_empty()
        || spec.purpose.trim().is_empty()
        || spec.maximum_results == 0
    {
        return Err(ProjectAggregateError::Invalid {
            detail: "episodic Memory recall requires project, employee, query, and purpose",
        });
    }
    if spec.caller_project_id != spec.target_project_id
        || spec.caller_employee_id != spec.target_employee_id
    {
        return Err(ProjectAggregateError::Forbidden {
            detail: "cross-scope Memory access is forbidden",
        });
    }
    require_employee_in_project(employees, spec.target_project_id, spec.target_employee_id)?;
    Ok(canonical_episodic_scope(
        spec.target_project_id,
        spec.target_employee_id,
    ))
}

/// Scoped FTS recall. Cross-project or cross-employee callers fail closed
/// before the index is consulted.
pub fn recall_episodic_memory(
    store: &SqliteAuthorityStore,
    employees: &EmployeeStore,
    spec: &EpisodicRecallSpec<'_>,
) -> Result<Vec<MemorySearchCandidateRow>, ProjectAggregateError> {
    let governance_scope = require_episodic_recall_scope(employees, spec)?;
    store
        .search_memory_candidates(&MemorySearchQuery {
            governance_scope,
            purpose: spec.purpose.to_owned(),
            observed_at_unix_seconds: spec.observed_at_unix_seconds,
            query_text: spec.query_text.to_owned(),
            maximum_results: spec.maximum_results,
        })
        .map_err(map_store)
}

/// Forget only when the Memory object's governance_scope matches the caller.
pub fn forget_episodic_memory(
    store: &SqliteAuthorityStore,
    employees: &EmployeeStore,
    project_id: &str,
    employee_id: &str,
    tombstone: &MemoryTombstoneRow,
) -> Result<(), ProjectAggregateError> {
    require_employee_in_project(employees, project_id, employee_id)?;
    let expected = canonical_episodic_scope(project_id, employee_id);
    let actual = load_memory_governance_scope(store, tombstone.memory_id.as_str())?;
    if actual != expected {
        return Err(ProjectAggregateError::Forbidden {
            detail: "cross-scope Memory access is forbidden",
        });
    }
    store.append_memory_tombstone(tombstone).map_err(map_store)
}

/// Rebuild FTS from admitted, non-tombstoned Memory. Tombstones cannot return.
pub fn rebuild_episodic_memory_index(
    store: &SqliteAuthorityStore,
) -> Result<(), ProjectAggregateError> {
    store.rebuild_memory_search_index().map_err(map_store)
}

/// Current governance_scope for an admitted Memory object.
pub fn load_memory_governance_scope(
    store: &SqliteAuthorityStore,
    memory_id: &str,
) -> Result<String, ProjectAggregateError> {
    let parsed = ObjectId::parse(memory_id).map_err(|_| ProjectAggregateError::Invalid {
        detail: "Memory object id is invalid",
    })?;
    let connection = lock_store(store)?;
    connection
        .query_row(
            "SELECT memory_candidates.governance_scope
             FROM memory_objects
             JOIN memory_candidates
               ON memory_candidates.candidate_id = memory_objects.candidate_id
             WHERE memory_objects.memory_id = ?1",
            [parsed.as_str()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|source| ProjectAggregateError::Unavailable {
            detail: format!("load Memory governance_scope: {source}"),
        })?
        .ok_or(ProjectAggregateError::NotFound {
            detail: "memory object not found",
        })
}

fn lock_store(
    store: &SqliteAuthorityStore,
) -> Result<MutexGuard<'_, rusqlite::Connection>, ProjectAggregateError> {
    store
        .conn
        .lock()
        .map_err(|_| ProjectAggregateError::Unavailable {
            detail: "authority writer lock poisoned".to_owned(),
        })
}

fn map_store(error: StorePortError) -> ProjectAggregateError {
    match error {
        StorePortError::Conflict { .. } => ProjectAggregateError::Conflict {
            detail: "Memory authority conflict",
        },
        StorePortError::Unavailable { detail } => ProjectAggregateError::Unavailable { detail },
    }
}

fn reject_secret_or_pii_shape(body: &str) -> Result<(), ProjectAggregateError> {
    let lowered = body.to_ascii_lowercase();
    if lowered.contains("sk-")
        || lowered.contains("bearer ")
        || lowered.contains("api_key")
        || lowered.contains("x-api-key")
        || lowered.contains("ssv1:")
        || lowered.contains("ssn")
        || contains_email_shape(body)
        || contains_ssn_digits(body)
    {
        return Err(ProjectAggregateError::Invalid {
            detail: "secret-shaped or PII-shaped Memory candidate is rejected",
        });
    }
    Ok(())
}

fn contains_email_shape(body: &str) -> bool {
    for token in body.split_whitespace() {
        let trimmed = token.trim_matches(|c: char| {
            !c.is_ascii_alphanumeric() && c != '@' && c != '.' && c != '_' && c != '-' && c != '+'
        });
        if let Some((local, domain)) = trimmed.split_once('@')
            && !local.is_empty()
            && domain.contains('.')
            && domain.split('.').all(|label| !label.is_empty())
        {
            return true;
        }
    }
    false
}

fn contains_ssn_digits(body: &str) -> bool {
    let chars: Vec<char> = body.chars().collect();
    let length = chars.len();
    let mut index = 0;
    while index + 11 <= length {
        if chars[index].is_ascii_digit()
            && chars[index + 1].is_ascii_digit()
            && chars[index + 2].is_ascii_digit()
            && chars[index + 3] == '-'
            && chars[index + 4].is_ascii_digit()
            && chars[index + 5].is_ascii_digit()
            && chars[index + 6] == '-'
            && chars[index + 7].is_ascii_digit()
            && chars[index + 8].is_ascii_digit()
            && chars[index + 9].is_ascii_digit()
            && chars[index + 10].is_ascii_digit()
        {
            return true;
        }
        index += 1;
    }
    false
}

fn reject_direct_or_self_admission(envelope_json: &str) -> Result<(), ProjectAggregateError> {
    let lowered = envelope_json.to_ascii_lowercase();
    if lowered.contains("letta")
        || lowered.contains("mem0")
        || lowered.contains("self_admit")
        || lowered.contains("agent_self")
        || lowered.contains("\"admitted_by\":\"agent\"")
        || lowered.contains("\"admitted_by\": \"agent\"")
        || lowered.contains("\"admitted_by\":\"self\"")
        || lowered.contains("\"write_path\":\"direct\"")
        || lowered.contains("\"write_mode\":\"direct\"")
    {
        return Err(ProjectAggregateError::Invalid {
            detail: "direct Agent, Letta, or Mem0 Memory write is rejected",
        });
    }
    if let Ok(document) = serde_json::from_str::<Value>(envelope_json) {
        let admitted_by = document.get("admitted_by").and_then(Value::as_str);
        let source = document.get("source").and_then(Value::as_str);
        let engine = document.get("engine").and_then(Value::as_str);
        let write_path = document
            .get("write_path")
            .and_then(Value::as_str)
            .or_else(|| document.get("write_mode").and_then(Value::as_str));
        if matches!(admitted_by, Some("agent" | "self" | "assistant"))
            || matches!(source, Some("letta" | "mem0" | "agent-self" | "letta-code"))
            || matches!(engine, Some("letta" | "mem0"))
            || matches!(write_path, Some("direct"))
        {
            return Err(ProjectAggregateError::Invalid {
                detail: "direct Agent, Letta, or Mem0 Memory write is rejected",
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn canonical_scope_encodes_project_and_employee() {
        assert_eq!(
            canonical_episodic_scope("proj-a", "emp-1"),
            "opc://project/proj-a/employee/emp-1"
        );
    }

    #[test]
    fn screen_rejects_secret_and_email_and_letta() {
        assert!(screen_memory_admission("api_key=sk-p11t11-fixture", "{}").is_err());
        assert!(screen_memory_admission("contact owner@example.com later", "{}").is_err());
        assert!(screen_memory_admission("ssn 123-45-6789", "{}").is_err());
        assert!(screen_memory_admission("lantern note", r#"{"source":"letta"}"#).is_err());
        assert!(screen_memory_admission("lantern note", r#"{"admitted_by":"agent"}"#).is_err());
        assert!(screen_memory_admission("lantern hangs east", "{}").is_ok());
    }
}
