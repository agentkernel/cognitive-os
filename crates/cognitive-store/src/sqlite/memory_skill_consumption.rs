//! Memory/Skill 受治理消费记录的 SQLite 适配。

use super::{is_constraint_violation, unavailable};
use cognitive_domain::ObjectId;
use cognitive_kernel::memory_skill_consumption::{
    EligibleMemoryConsumption, MemoryConsumptionPin, MemorySkillConsumptionRecord,
    MemorySkillConsumptionStore, SkillConsumptionPin,
};
use cognitive_kernel::ports::StorePortError;
use rusqlite::OptionalExtension;
use serde_json::{Value, json};

struct StoredConsumptionRow {
    consumption_id: ObjectId,
    task_ref: String,
    contract_epoch: i64,
    context_request_id: String,
    context_request_digest: String,
    session_ref: String,
    reuse_of: Option<String>,
    canonical_json: String,
}

impl MemorySkillConsumptionStore for super::SqliteAuthorityStore {
    fn list_eligible_memory_pins(
        &self,
        governance_scope: &str,
        task_ref: &str,
        purpose: &str,
        observed_at_unix_seconds: i64,
    ) -> Result<Vec<EligibleMemoryConsumption>, StorePortError> {
        if governance_scope.trim().is_empty()
            || task_ref.trim().is_empty()
            || purpose.trim().is_empty()
        {
            return Err(StorePortError::Unavailable {
                detail: "Memory consumption scope, Task, or purpose is missing".to_owned(),
            });
        }
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT memory_objects.memory_id, memory_candidates.source_id,
                        memory_candidates.source_digest, workspace_context_sources.tenant_id,
                        workspace_context_sources.owner_ref,
                        workspace_context_sources.resource_scope,
                        memory_candidates.target_scope, memory_candidates.purpose,
                        memory_candidates.source_provenance_ref
                 FROM memory_objects
                 JOIN memory_candidates ON memory_candidates.candidate_id = memory_objects.candidate_id
                 JOIN memory_admission_decisions ON memory_admission_decisions.decision_id = memory_objects.decision_id
                 JOIN workspace_context_sources ON workspace_context_sources.source_id = memory_candidates.source_id
                     AND workspace_context_sources.source_digest = memory_candidates.source_digest
                     AND workspace_context_sources.provenance_ref = memory_candidates.source_provenance_ref
                     AND workspace_context_sources.resource_scope = memory_candidates.governance_scope
                 WHERE memory_admission_decisions.decision = 'admit'
                     AND NOT EXISTS (
                         SELECT 1 FROM memory_tombstones
                         WHERE memory_tombstones.memory_id = memory_objects.memory_id
                     )
                     AND (
                         memory_candidates.governance_scope = ?1
                         OR memory_candidates.governance_scope LIKE ?1 || '/%'
                     )
                     AND (
                         memory_candidates.target_scope = ?2
                         OR memory_candidates.target_scope = memory_candidates.governance_scope
                     )
                     AND memory_candidates.purpose = ?3
                     AND memory_candidates.retention_expires_at_unix_seconds > ?4
                 ORDER BY memory_objects.memory_id",
            )
            .map_err(unavailable("prepare eligible Memory pins"))?;
        let rows = statement
            .query_map(
                rusqlite::params![
                    governance_scope,
                    task_ref,
                    purpose,
                    observed_at_unix_seconds
                ],
                |row| {
                    Ok(EligibleMemoryConsumption {
                        pin: MemoryConsumptionPin {
                            memory_id: parse_object_id(row.get::<_, String>(0)?, 0)?,
                            source_id: parse_object_id(row.get::<_, String>(1)?, 1)?,
                            source_digest: row.get(2)?,
                        },
                        tenant_id: row.get(3)?,
                        owner_ref: row.get(4)?,
                        resource_scope: row.get(5)?,
                        target_scope: row.get(6)?,
                        purpose: row.get(7)?,
                        source_provenance_ref: row.get(8)?,
                    })
                },
            )
            .map_err(unavailable("query eligible Memory pins"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read eligible Memory pins"))
    }

    fn list_eligible_skill_pins(
        &self,
        workspace_scope: &str,
        task_ref: &str,
    ) -> Result<Vec<SkillConsumptionPin>, StorePortError> {
        if workspace_scope.trim().is_empty() || task_ref.trim().is_empty() {
            return Err(StorePortError::Unavailable {
                detail: "Skill consumption workspace or task is missing".to_owned(),
            });
        }
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT skill_bindings.binding_id, skill_bindings.revision_id, skill_packages.package_id, skill_revisions.content_digest
                 FROM skill_bindings
                 JOIN skill_revisions ON skill_revisions.revision_id = skill_bindings.revision_id
                 JOIN skill_packages ON skill_packages.package_id = skill_revisions.package_id
                 WHERE skill_bindings.status = 'active'
                     AND NOT EXISTS (
                         SELECT 1 FROM skill_binding_revocations
                         WHERE skill_binding_revocations.binding_id = skill_bindings.binding_id
                     )
                     AND skill_bindings.workspace_scope = ?1
                     AND (
                         (skill_bindings.target_kind = 'task' AND skill_bindings.target_ref = ?2)
                         OR (skill_bindings.target_kind = 'workspace' AND skill_bindings.target_ref = ?1)
                     )
                 ORDER BY skill_bindings.binding_id",
            )
            .map_err(unavailable("prepare eligible Skill pins"))?;
        let rows = statement
            .query_map(rusqlite::params![workspace_scope, task_ref], |row| {
                Ok(SkillConsumptionPin {
                    binding_id: parse_object_id(row.get::<_, String>(0)?, 0)?,
                    revision_id: parse_object_id(row.get::<_, String>(1)?, 1)?,
                    package_id: parse_object_id(row.get::<_, String>(2)?, 2)?,
                    content_digest: row.get(3)?,
                })
            })
            .map_err(unavailable("query eligible Skill pins"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read eligible Skill pins"))
    }

    fn append_memory_skill_consumption(
        &self,
        record: &MemorySkillConsumptionRecord,
    ) -> Result<(), StorePortError> {
        validate_consumption_record(record)?;
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(unavailable("begin Memory/Skill consumption"))?;
        let insert_result = transaction.execute(
            "INSERT INTO memory_skill_consumption_records (
                consumption_id, task_ref, contract_epoch, context_request_id,
                context_request_digest, session_ref, reuse_of, canonical_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                record.consumption_id.as_str(),
                record.task_ref,
                record.contract_epoch,
                record.context_request_id.as_str(),
                record.context_request_digest,
                record.session_ref,
                record.reuse_of.as_ref().map(ObjectId::as_str),
                record.canonical_json,
            ],
        );
        match insert_result {
            Ok(_) => transaction
                .commit()
                .map_err(unavailable("commit Memory/Skill consumption")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: "Memory/Skill consumption record conflicts with an existing binding"
                    .to_owned(),
            }),
            Err(error) => Err(unavailable("insert Memory/Skill consumption")(error)),
        }
    }

    fn load_memory_skill_consumption(
        &self,
        consumption_id: &ObjectId,
    ) -> Result<Option<MemorySkillConsumptionRecord>, StorePortError> {
        let connection = self.lock()?;
        let loaded = connection
            .query_row(
                "SELECT task_ref, contract_epoch, context_request_id, context_request_digest, session_ref, reuse_of, canonical_json
                 FROM memory_skill_consumption_records WHERE consumption_id=?1",
                (consumption_id.as_str(),),
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                },
            )
            .optional()
            .map_err(unavailable("load Memory/Skill consumption"))?;
        loaded
            .map(
                |(
                    task_ref,
                    contract_epoch,
                    request_id,
                    request_digest,
                    session_ref,
                    reuse_of,
                    canonical_json,
                )| {
                    row_to_record(StoredConsumptionRow {
                        consumption_id: consumption_id.clone(),
                        task_ref,
                        contract_epoch,
                        context_request_id: request_id,
                        context_request_digest: request_digest,
                        session_ref,
                        reuse_of,
                        canonical_json,
                    })
                },
            )
            .transpose()
    }

    fn load_latest_memory_skill_consumption(
        &self,
        task_ref: &str,
        contract_epoch: i64,
        context_request_id: &ObjectId,
    ) -> Result<Option<MemorySkillConsumptionRecord>, StorePortError> {
        let connection = self.lock()?;
        let loaded = connection
            .query_row(
                "SELECT consumption_id, task_ref, contract_epoch, context_request_id, context_request_digest, session_ref, reuse_of, canonical_json
                 FROM memory_skill_consumption_records
                 WHERE task_ref=?1 AND contract_epoch=?2 AND context_request_id=?3
                 ORDER BY rowid DESC LIMIT 1",
                rusqlite::params![task_ref, contract_epoch, context_request_id.as_str()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, String>(7)?,
                    ))
                },
            )
            .optional()
            .map_err(unavailable("load latest Memory/Skill consumption"))?;
        loaded
            .map(
                |(
                    id,
                    task_ref,
                    contract_epoch,
                    request_id,
                    request_digest,
                    session_ref,
                    reuse_of,
                    canonical_json,
                )| {
                    row_to_record(StoredConsumptionRow {
                        consumption_id: ObjectId::parse(&id).map_err(|error| {
                            StorePortError::Unavailable {
                                detail: format!("Memory/Skill consumption id is invalid: {error}"),
                            }
                        })?,
                        task_ref,
                        contract_epoch,
                        context_request_id: request_id,
                        context_request_digest: request_digest,
                        session_ref,
                        reuse_of,
                        canonical_json,
                    })
                },
            )
            .transpose()
    }

    fn load_skill_revision_payload(
        &self,
        revision_id: &ObjectId,
    ) -> Result<Option<(String, String)>, StorePortError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT content_digest, canonical_json FROM skill_revisions WHERE revision_id=?1",
                (revision_id.as_str(),),
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("load Skill revision payload"))
    }
}

fn parse_object_id(value: String, column: usize) -> Result<ObjectId, rusqlite::Error> {
    ObjectId::parse(&value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

fn row_to_record(
    row: StoredConsumptionRow,
) -> Result<MemorySkillConsumptionRecord, StorePortError> {
    let StoredConsumptionRow {
        consumption_id,
        task_ref,
        contract_epoch,
        context_request_id,
        context_request_digest,
        session_ref,
        reuse_of,
        canonical_json,
    } = row;
    let document: Value =
        serde_json::from_str(&canonical_json).map_err(|error| StorePortError::Unavailable {
            detail: format!("Memory/Skill consumption payload is malformed: {error}"),
        })?;
    let memory = document
        .get("memory")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|item| {
            Ok(MemoryConsumptionPin {
                memory_id: parse_json_object_id(&item, "memory_id")?,
                source_id: parse_json_object_id(&item, "source_id")?,
                source_digest: string_field(&item, "source_digest")?,
            })
        })
        .collect::<Result<Vec<_>, StorePortError>>()?;
    let skill = document
        .get("skill")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|item| {
            Ok(SkillConsumptionPin {
                binding_id: parse_json_object_id(&item, "binding_id")?,
                revision_id: parse_json_object_id(&item, "revision_id")?,
                package_id: parse_json_object_id(&item, "package_id")?,
                content_digest: string_field(&item, "content_digest")?,
            })
        })
        .collect::<Result<Vec<_>, StorePortError>>()?;
    Ok(MemorySkillConsumptionRecord {
        consumption_id,
        task_ref,
        contract_epoch,
        context_request_id: ObjectId::parse(&context_request_id).map_err(|error| {
            StorePortError::Unavailable {
                detail: format!("Memory/Skill consumption request id is invalid: {error}"),
            }
        })?,
        context_request_digest,
        session_ref,
        reuse_of: reuse_of
            .map(|value| {
                ObjectId::parse(&value).map_err(|error| StorePortError::Unavailable {
                    detail: format!("Memory/Skill consumption reuse id is invalid: {error}"),
                })
            })
            .transpose()?,
        memory,
        skill,
        canonical_json,
    })
}

fn parse_json_object_id(value: &Value, field: &str) -> Result<ObjectId, StorePortError> {
    ObjectId::parse(&string_field(value, field)?).map_err(|error| StorePortError::Unavailable {
        detail: format!("Memory/Skill consumption {field} is invalid: {error}"),
    })
}

fn string_field(value: &Value, field: &str) -> Result<String, StorePortError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| StorePortError::Unavailable {
            detail: format!("Memory/Skill consumption {field} is missing"),
        })
}

fn validate_consumption_record(
    record: &MemorySkillConsumptionRecord,
) -> Result<(), StorePortError> {
    if record.task_ref.trim().is_empty()
        || record.contract_epoch < 1
        || record.context_request_digest.trim().is_empty()
        || record.session_ref.trim().is_empty()
        || serde_json::from_str::<Value>(&record.canonical_json).is_err()
    {
        return Err(StorePortError::Unavailable {
            detail: "Memory/Skill consumption record is incomplete".to_owned(),
        });
    }
    let expected = json!({
        "memory": record.memory.iter().map(|pin| json!({
            "memory_id": pin.memory_id.to_string(),
            "source_id": pin.source_id.to_string(),
            "source_digest": pin.source_digest,
        })).collect::<Vec<_>>(),
        "skill": record.skill.iter().map(|pin| json!({
            "binding_id": pin.binding_id.to_string(),
            "revision_id": pin.revision_id.to_string(),
            "package_id": pin.package_id.to_string(),
            "content_digest": pin.content_digest,
        })).collect::<Vec<_>>(),
    });
    let actual: Value = serde_json::from_str(&record.canonical_json).map_err(|error| {
        StorePortError::Unavailable {
            detail: format!("Memory/Skill consumption payload is malformed: {error}"),
        }
    })?;
    if actual.get("memory") != expected.get("memory")
        || actual.get("skill") != expected.get("skill")
    {
        return Err(StorePortError::Unavailable {
            detail: "Memory/Skill consumption pins differ from the canonical payload".to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{PersonalDataLayout, SqliteAuthorityStore, prepare_personal_databases};

    #[test]
    fn consumption_chain_survives_store_reopen_and_replays_latest_append() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let layout = PersonalDataLayout::from_xdg_roots(
            root.join("config"),
            root.join("data"),
            root.join("state"),
            root.join("cache"),
            root.join("runtime"),
        );
        prepare_personal_databases(&layout).unwrap();
        let database_path = layout.authority_database_path();
        let first = record(1, "conversation://tenant-a/one", None);

        let store = SqliteAuthorityStore::open(&database_path).unwrap();
        store.append_memory_skill_consumption(&first).unwrap();
        drop(store);

        let reopened = SqliteAuthorityStore::open(&database_path).unwrap();
        assert_eq!(
            reopened
                .load_memory_skill_consumption(&first.consumption_id)
                .unwrap(),
            Some(first.clone())
        );
        let second = record(
            2,
            "conversation://tenant-a/two",
            Some(first.consumption_id.clone()),
        );
        reopened.append_memory_skill_consumption(&second).unwrap();
        drop(reopened);

        let replayed = SqliteAuthorityStore::open(&database_path).unwrap();
        assert_eq!(
            replayed
                .load_latest_memory_skill_consumption(
                    &second.task_ref,
                    second.contract_epoch,
                    &second.context_request_id,
                )
                .unwrap(),
            Some(second)
        );
    }

    fn record(
        serial: u64,
        session_ref: &str,
        reuse_of: Option<ObjectId>,
    ) -> MemorySkillConsumptionRecord {
        MemorySkillConsumptionRecord {
            consumption_id: object_id(serial),
            task_ref: "task://tenant-a/restart-replay".to_owned(),
            contract_epoch: 1,
            context_request_id: object_id(10),
            context_request_digest: format!("sha256:{}", "c".repeat(64)),
            session_ref: session_ref.to_owned(),
            reuse_of,
            memory: Vec::new(),
            skill: Vec::new(),
            canonical_json: json!({
                "principal_ref": "principal://tenant-a/owner",
                "tenant_id": "tenant-a",
                "resource_scope": "workspace://tenant-a/project",
                "purpose": "task_execution",
                "memory": [],
                "skill": [],
            })
            .to_string(),
        }
    }

    fn object_id(serial: u64) -> ObjectId {
        ObjectId::parse(&format!("00000000-0000-7000-9000-{serial:012x}")).unwrap()
    }
}
