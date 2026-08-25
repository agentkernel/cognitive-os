#![allow(dead_code, unused_imports)]

use crate::context_store::{
    CONTEXT_AUTHORIZATION_FACT_SCHEMA_V14, CONTEXT_STORE_SCHEMA_V12,
    SCHEDULER_EXECUTION_POLICY_SCHEMA_V15, WORKSPACE_CONTEXT_SOURCE_SCHEMA_V13,
};
use crate::memory_store::{MEMORY_ADMISSION_SCHEMA_V16, MEMORY_SEARCH_SCHEMA_V17};
use crate::scheduler::SCHEDULER_SCHEMA_CURRENT;
use crate::worker_authorization::{
    CONTINUATION_AUTHORITY_CONSUMPTION_SCHEMA_V11, CONTINUATION_AUTHORITY_SCHEMA_V10,
    DAEMON_AUTHORIZATION_SNAPSHOT_SCHEMA_V6, DAEMON_OPERATION_DESCRIPTOR_SCHEMA_V5,
    WORKER_AUTHORIZATION_LEASE_BINDING_SCHEMA_V9, WORKER_AUTHORIZATION_SCHEMA_V4,
    WORKER_ITERATION_AUTHORIZATION_CONSUMPTION_SCHEMA_V8, WORKER_ITERATION_AUTHORIZATION_SCHEMA_V7,
};
use cognitive_contracts::generated::context_request::ContextRequest;
use cognitive_contracts::generated::context_view::ContextView;
use cognitive_contracts::generated::governed_object_header::GovernedObjectHeader;
use cognitive_contracts::generated::object_reference::StrongReferenceKind;
use cognitive_contracts::projection::verify_content_digest;
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, StateName, Version, WallTimestamp,
};
use cognitive_kernel::authz::ObjectGovernance;
use cognitive_kernel::effects::GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN;
use cognitive_kernel::ports::{
    AuthorityStore, BoundContinuationAuthorizationConsumption, BoundWorkerAuthorizationConsumption,
    CandidateAdmissionCommit, CandidateAdmissionReceipt, CheckpointRow, CommitReceipt,
    CommittedEvent, ConsumedWorkerIterationAuthorization, ContextAuthorizationFactStore,
    ContextAuthorizationFactsRow, ContextCandidateMetadata, ContextCandidateQuery,
    ContextRequestRow, ContextRevocationFactRow, ContextStore, ContextViewRow,
    ContinuationAuthorityStore, ContinuationAuthorizationRow, DaemonAuthorizationSnapshotRow,
    DaemonOperationDescriptorRow, FixedPostStateRow, GovernanceObjectStore, HarnessStore,
    IntentChainStore, IntentRow, InterpretationRow, MemoryAdmissionDecisionRow, MemoryCandidateRow,
    MemoryObjectRow, MemorySearchCandidateRow, MemorySearchQuery, MemoryStore, MemoryTombstoneRow,
    MemoryUpdateRequest, ObjectAdmission, OperationCandidateProposalRow, OutboxEntry,
    ProgressFactRow, ProtocolStore, SchedulerExecutionPolicyRow, SchedulerExecutionPolicyStore,
    SchedulerLeaseBinding, SkillBindingExplanationRow, SkillBindingRevocationRow, SkillBindingRow,
    SkillPackageRow, SkillRevisionRow, SkillRevisionSupersedeRequest, SkillStore, StorePortError,
    StoredBudget, StoredObject, TaskBinding, TaskContractRow, TransitionCommit,
    UserIntentRecordRow, VerificationReportRow, VerificationRequestRow, WorkerAuthorizationStore,
    WorkerIterationAuthorizationConsumptionRow, WorkerIterationAuthorizationRow,
    WorkspaceContextSourceRow,
};
use cognitive_kernel::{BudgetState, EffectClass, ExecutorCapabilities, OperationDescriptor};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use super::*;

pub(crate) fn validate_memory_candidate(
    candidate: &MemoryCandidateRow,
) -> Result<(), StorePortError> {
    let payload: Value = serde_json::from_str(&candidate.canonical_json)
        .map_err(|error| invalid_context_payload("MemoryCandidate", error))?;
    verify_content_digest(
        &payload,
        &["/header/content_digest"],
        GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN,
        "/header/content_digest",
    )
    .map_err(|error| invalid_context_payload("MemoryCandidate", error))?;
    let header: GovernedObjectHeader =
        serde_json::from_value(payload.get("header").cloned().ok_or_else(|| {
            invalid_context_payload("MemoryCandidate", "missing governed header")
        })?)
        .map_err(|error| invalid_context_payload("MemoryCandidate", error))?;
    if header.id.0 != candidate.candidate_id.as_str()
        || header.r#type != "MemoryCandidate"
        || header.content_digest.0 != candidate.candidate_digest
        || candidate.purpose.trim().is_empty()
    {
        return Err(invalid_context_payload(
            "MemoryCandidate",
            "row identity, digest, type, or purpose differs from canonical payload",
        ));
    }
    Ok(())
}

pub(crate) fn extract_memory_source_text(
    canonical_source_json: &str,
) -> Result<String, StorePortError> {
    let source_payload: Value = serde_json::from_str(canonical_source_json).map_err(|error| {
        StorePortError::Unavailable {
            detail: format!("Memory source body cannot be indexed: {error}"),
        }
    })?;
    source_payload
        .pointer("/body/text")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| StorePortError::Unavailable {
            detail: "Memory source body is missing indexable text".to_owned(),
        })
}

pub(crate) fn validate_memory_search_query(
    query: &MemorySearchQuery,
) -> Result<(), StorePortError> {
    if query.governance_scope.trim().is_empty()
        || query.purpose.trim().is_empty()
        || query.query_text.trim().is_empty()
        || query.maximum_results == 0
    {
        return Err(StorePortError::Unavailable {
            detail: "Memory search requires non-empty metadata, query text, and result limit"
                .to_owned(),
        });
    }
    Ok(())
}

impl MemoryStore for SqliteAuthorityStore {
    fn append_memory_admission(
        &self,
        candidate: &MemoryCandidateRow,
        decision: &MemoryAdmissionDecisionRow,
        admitted_object: Option<&MemoryObjectRow>,
    ) -> Result<(), StorePortError> {
        validate_memory_candidate(candidate)?;
        if decision.candidate_id != candidate.candidate_id
            || decision.candidate_digest != candidate.candidate_digest
            || decision.policy_version < 1
            || !matches!(
                decision.decision.as_str(),
                "admit" | "reject" | "review" | "quarantine"
            )
            || serde_json::from_str::<Vec<String>>(&decision.reason_codes_json).is_err()
        {
            return Err(invalid_context_payload(
                "MemoryAdmissionDecision",
                "candidate binding, decision, policy, or reason codes are invalid",
            ));
        }
        if (decision.decision == "admit") != admitted_object.is_some() {
            return Err(invalid_context_payload(
                "MemoryAdmissionDecision",
                "only an admit decision may create a MemoryObject",
            ));
        }
        if let Some(memory_object) = admitted_object
            && (memory_object.candidate_id != candidate.candidate_id
                || memory_object.decision_id != decision.decision_id)
        {
            return Err(invalid_context_payload(
                "MemoryObject",
                "object must bind the exact candidate and decision",
            ));
        }

        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin Memory admission transaction"))?;
        let persisted_source = transaction
            .query_row(
                "SELECT source_digest, provenance_ref, resource_scope, canonical_json FROM workspace_context_sources WHERE source_id=?1",
                (candidate.source_id.as_str(),),
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)),
            )
            .optional()
            .map_err(unavailable("load Memory source binding"))?;
        let Some((source_digest, source_provenance_ref, governance_scope, source_canonical_json)) =
            persisted_source
        else {
            return Err(StorePortError::Conflict {
                detail: "Memory proposal source digest, provenance, or scope is no longer current"
                    .to_owned(),
            });
        };
        if (source_digest, source_provenance_ref, governance_scope)
            != (
                candidate.source_digest.clone(),
                candidate.source_provenance_ref.clone(),
                candidate.governance_scope.clone(),
            )
        {
            return Err(StorePortError::Conflict {
                detail: "Memory proposal source digest, provenance, or scope is no longer current"
                    .to_owned(),
            });
        }
        let source_text = admitted_object
            .map(|_| extract_memory_source_text(&source_canonical_json))
            .transpose()?;
        let insert_result = (|| -> Result<(), rusqlite::Error> {
            transaction.execute(
                "INSERT INTO memory_candidates (candidate_id, source_id, source_digest, source_provenance_ref, governance_scope, target_scope, purpose, retention_expires_at_unix_seconds, observed_at_unix_seconds, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                (candidate.candidate_id.as_str(), candidate.source_id.as_str(), candidate.source_digest.as_str(), candidate.source_provenance_ref.as_str(), candidate.governance_scope.as_str(), candidate.target_scope.as_str(), candidate.purpose.as_str(), candidate.retention_expires_at_unix_seconds, candidate.observed_at_unix_seconds, candidate.canonical_json.as_str()),
            )?;
            transaction.execute(
                "INSERT INTO memory_admission_decisions (decision_id, candidate_id, candidate_digest, decision, policy_version, reason_codes_json, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                (decision.decision_id.as_str(), decision.candidate_id.as_str(), decision.candidate_digest.as_str(), decision.decision.as_str(), decision.policy_version, decision.reason_codes_json.as_str(), decision.canonical_json.as_str()),
            )?;
            if let Some(memory_object) = admitted_object {
                transaction.execute(
                    "INSERT INTO memory_objects (memory_id, candidate_id, decision_id, canonical_json) VALUES (?1, ?2, ?3, ?4)",
                    (memory_object.memory_id.as_str(), memory_object.candidate_id.as_str(), memory_object.decision_id.as_str(), memory_object.canonical_json.as_str()),
                )?;
                transaction.execute(
                    "INSERT INTO memory_object_versions (memory_id, version) VALUES (?1, 1)",
                    (memory_object.memory_id.as_str(),),
                )?;
                transaction.execute(
                    "INSERT INTO memory_search_fts (memory_id, source_text) VALUES (?1, ?2)",
                    (
                        memory_object.memory_id.as_str(),
                        source_text.as_deref().unwrap_or_default(),
                    ),
                )?;
            }
            Ok(())
        })();
        match insert_result {
            Ok(()) => transaction
                .commit()
                .map_err(unavailable("commit Memory admission transaction")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: "Memory admission conflicts with an existing immutable record".to_owned(),
            }),
            Err(error) => Err(unavailable("insert Memory admission")(error)),
        }
    }

    fn load_memory_object(
        &self,
        memory_id: &ObjectId,
    ) -> Result<Option<MemoryObjectRow>, StorePortError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT candidate_id, decision_id, canonical_json FROM memory_objects WHERE memory_id=?1",
                (memory_id.as_str(),),
                |row| Ok(MemoryObjectRow {
                    memory_id: memory_id.clone(),
                    candidate_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|error| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error)))?,
                    decision_id: ObjectId::parse(&row.get::<_, String>(1)?).map_err(|error| rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(error)))?,
                    canonical_json: row.get(2)?,
                }),
            )
            .optional()
            .map_err(unavailable("load MemoryObject"))
    }

    fn append_memory_tombstone(
        &self,
        tombstone: &MemoryTombstoneRow,
    ) -> Result<(), StorePortError> {
        if !matches!(tombstone.action.as_str(), "forget" | "expire")
            || tombstone.reason.trim().is_empty()
            || serde_json::from_str::<serde_json::Value>(&tombstone.canonical_json).is_err()
        {
            return Err(invalid_context_payload(
                "MemoryLifecycle",
                "action, reason, or canonical audit payload is invalid",
            ));
        }

        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin Memory forget transaction"))?;
        let exists = transaction
            .query_row(
                "SELECT 1 FROM memory_objects WHERE memory_id=?1",
                (tombstone.memory_id.as_str(),),
                |_| Ok(()),
            )
            .optional()
            .map_err(unavailable("load Memory object for forget"))?
            .is_some();
        if !exists {
            return Err(StorePortError::Conflict {
                detail: format!("Memory object {} does not exist", tombstone.memory_id),
            });
        }

        let insert_result = (|| -> Result<(), rusqlite::Error> {
            transaction.execute(
                "INSERT INTO memory_tombstones (lifecycle_id, memory_id, action, occurred_at_unix_seconds, reason, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    tombstone.lifecycle_id.as_str(),
                    tombstone.memory_id.as_str(),
                    tombstone.action.as_str(),
                    tombstone.occurred_at_unix_seconds,
                    tombstone.reason.as_str(),
                    tombstone.canonical_json.as_str(),
                ),
            )?;
            transaction.execute(
                "DELETE FROM memory_search_fts WHERE memory_id=?1",
                (tombstone.memory_id.as_str(),),
            )?;
            Ok(())
        })();
        match insert_result {
            Ok(()) => transaction
                .commit()
                .map_err(unavailable("commit Memory forget transaction")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!(
                    "Memory object {} already has a lifecycle record",
                    tombstone.memory_id
                ),
            }),
            Err(error) => Err(unavailable("append Memory tombstone")(error)),
        }
    }

    fn append_memory_expiration(
        &self,
        expiration: &MemoryTombstoneRow,
    ) -> Result<(), StorePortError> {
        if expiration.action != "expire" {
            return Err(invalid_context_payload(
                "MemoryExpiration",
                "expiration lifecycle action must be expire",
            ));
        }
        let connection = self.lock()?;
        let retention_deadline = connection
            .query_row(
                "SELECT memory_candidates.retention_expires_at_unix_seconds FROM memory_objects JOIN memory_candidates ON memory_candidates.candidate_id = memory_objects.candidate_id WHERE memory_objects.memory_id=?1",
                (expiration.memory_id.as_str(),),
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(unavailable("load Memory retention deadline"))?;
        let Some(retention_deadline) = retention_deadline else {
            return Err(StorePortError::Conflict {
                detail: format!("Memory object {} does not exist", expiration.memory_id),
            });
        };
        if expiration.occurred_at_unix_seconds < retention_deadline {
            return Err(StorePortError::Conflict {
                detail: format!(
                    "Memory object {} retention has not expired",
                    expiration.memory_id
                ),
            });
        }
        drop(connection);
        self.append_memory_tombstone(expiration)
    }

    fn append_memory_update(&self, update: &MemoryUpdateRequest) -> Result<(), StorePortError> {
        if update.decision.decision != "admit"
            || update.candidate.candidate_id != update.decision.candidate_id
            || update.replacement.candidate_id != update.candidate.candidate_id
            || update.replacement.decision_id != update.decision.decision_id
            || update.supersede_tombstone.action != "supersede"
            || update.supersede_tombstone.memory_id != update.previous_memory_id
        {
            return Err(invalid_context_payload(
                "MemoryUpdate",
                "replacement bindings or supersede lifecycle fact are invalid",
            ));
        }
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin Memory version update transaction"))?;
        let current_version = transaction
            .query_row(
                "SELECT version FROM memory_object_versions WHERE memory_id=?1",
                (update.previous_memory_id.as_str(),),
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(unavailable("load Memory version for update"))?;
        let Some(current_version) = current_version else {
            return Err(StorePortError::Conflict {
                detail: "Memory update target does not exist".to_owned(),
            });
        };
        if current_version != update.expected_version {
            return Err(StorePortError::Conflict {
                detail: format!(
                    "Memory update expected version {}, found {}",
                    update.expected_version, current_version
                ),
            });
        }
        let source_canonical_json = transaction
            .query_row(
                "SELECT canonical_json FROM workspace_context_sources WHERE source_id=?1 AND source_digest=?2 AND provenance_ref=?3 AND resource_scope=?4",
                rusqlite::params![update.candidate.source_id.as_str(), update.candidate.source_digest.as_str(), update.candidate.source_provenance_ref.as_str(), update.candidate.governance_scope.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(unavailable("load Memory update source"))?
            .ok_or_else(|| StorePortError::Conflict {
                detail: "Memory update source binding is no longer current".to_owned(),
            })?;
        let source_text = extract_memory_source_text(&source_canonical_json)?;
        let insert_result = (|| -> Result<(), rusqlite::Error> {
            transaction.execute(
                "INSERT INTO memory_candidates (candidate_id, source_id, source_digest, source_provenance_ref, governance_scope, target_scope, purpose, retention_expires_at_unix_seconds, observed_at_unix_seconds, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![update.candidate.candidate_id.as_str(), update.candidate.source_id.as_str(), update.candidate.source_digest.as_str(), update.candidate.source_provenance_ref.as_str(), update.candidate.governance_scope.as_str(), update.candidate.target_scope.as_str(), update.candidate.purpose.as_str(), update.candidate.retention_expires_at_unix_seconds, update.candidate.observed_at_unix_seconds, update.candidate.canonical_json.as_str()],
            )?;
            transaction.execute(
                "INSERT INTO memory_admission_decisions (decision_id, candidate_id, candidate_digest, decision, policy_version, reason_codes_json, canonical_json) VALUES (?1, ?2, ?3, 'admit', ?4, ?5, ?6)",
                rusqlite::params![update.decision.decision_id.as_str(), update.decision.candidate_id.as_str(), update.decision.candidate_digest.as_str(), update.decision.policy_version, update.decision.reason_codes_json.as_str(), update.decision.canonical_json.as_str()],
            )?;
            transaction.execute(
                "INSERT INTO memory_objects (memory_id, candidate_id, decision_id, canonical_json) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![update.replacement.memory_id.as_str(), update.replacement.candidate_id.as_str(), update.replacement.decision_id.as_str(), update.replacement.canonical_json.as_str()],
            )?;
            transaction.execute(
                "INSERT INTO memory_object_versions (memory_id, version, supersedes_memory_id) VALUES (?1, ?2, ?3)",
                rusqlite::params![update.replacement.memory_id.as_str(), update.expected_version + 1, update.previous_memory_id.as_str()],
            )?;
            transaction.execute(
                "INSERT INTO memory_tombstones (lifecycle_id, memory_id, action, occurred_at_unix_seconds, reason, canonical_json) VALUES (?1, ?2, 'supersede', ?3, ?4, ?5)",
                rusqlite::params![update.supersede_tombstone.lifecycle_id.as_str(), update.previous_memory_id.as_str(), update.supersede_tombstone.occurred_at_unix_seconds, update.supersede_tombstone.reason.as_str(), update.supersede_tombstone.canonical_json.as_str()],
            )?;
            transaction.execute(
                "DELETE FROM memory_search_fts WHERE memory_id=?1",
                (update.previous_memory_id.as_str(),),
            )?;
            transaction.execute(
                "INSERT INTO memory_search_fts (memory_id, source_text) VALUES (?1, ?2)",
                (update.replacement.memory_id.as_str(), source_text.as_str()),
            )?;
            Ok(())
        })();
        match insert_result {
            Ok(()) => transaction
                .commit()
                .map_err(unavailable("commit Memory version update transaction")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: "Memory update conflicts with an existing immutable version".to_owned(),
            }),
            Err(error) => Err(unavailable("insert Memory version update")(error)),
        }
    }

    fn search_memory_candidates(
        &self,
        query: &MemorySearchQuery,
    ) -> Result<Vec<MemorySearchCandidateRow>, StorePortError> {
        validate_memory_search_query(query)?;
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "WITH authority_filtered_memory AS (
                    SELECT memory_objects.memory_id, memory_candidates.source_id, memory_candidates.source_digest
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
                        AND memory_candidates.governance_scope = ?1
                        AND memory_candidates.purpose = ?2
                        AND memory_candidates.retention_expires_at_unix_seconds > ?3
                )
                SELECT DISTINCT authority_filtered_memory.memory_id, authority_filtered_memory.source_id, authority_filtered_memory.source_digest
                FROM authority_filtered_memory
                JOIN memory_search_fts ON memory_search_fts.memory_id = authority_filtered_memory.memory_id
                WHERE memory_search_fts MATCH ?4
                ORDER BY bm25(memory_search_fts), authority_filtered_memory.memory_id
                LIMIT ?5",
            )
            .map_err(unavailable("prepare Memory FTS search"))?;
        let results = statement
            .query_map(
                rusqlite::params![
                    query.governance_scope,
                    query.purpose,
                    query.observed_at_unix_seconds,
                    query.query_text,
                    i64::try_from(query.maximum_results).unwrap_or(i64::MAX),
                ],
                |row| {
                    Ok(MemorySearchCandidateRow {
                        memory_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        })?,
                        source_id: ObjectId::parse(&row.get::<_, String>(1)?).map_err(|error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                1,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        })?,
                        source_digest: row.get(2)?,
                    })
                },
            )
            .map_err(unavailable("query Memory FTS search"))?;
        results
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read Memory FTS search"))
    }

    fn rebuild_memory_search_index(&self) -> Result<(), StorePortError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin Memory FTS rebuild"))?;
        transaction
            .execute("DELETE FROM memory_search_fts", [])
            .map_err(unavailable("clear Memory FTS rebuild"))?;
        transaction
            .execute(
                "INSERT INTO memory_search_fts (memory_id, source_text)
                 SELECT memory_objects.memory_id, json_extract(workspace_context_sources.canonical_json, '$.body.text')
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
                    AND json_type(workspace_context_sources.canonical_json, '$.body.text') = 'text'
                    AND length(trim(json_extract(workspace_context_sources.canonical_json, '$.body.text'))) > 0",
                [],
            )
            .map_err(unavailable("populate Memory FTS rebuild"))?;
        transaction
            .commit()
            .map_err(unavailable("commit Memory FTS rebuild"))
    }
}

impl SqliteAuthorityStore {
    /// Bounded list of admitted Memory objects that are not tombstoned.
    pub fn list_non_tombstoned_memory_objects(
        &self,
        limit: usize,
    ) -> Result<(Vec<MemoryObjectRow>, bool), StorePortError> {
        let fetch = i64::try_from(limit.saturating_add(1)).unwrap_or(i64::MAX);
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT memory_id, candidate_id, decision_id, canonical_json
                 FROM memory_objects
                 WHERE NOT EXISTS (
                     SELECT 1 FROM memory_tombstones
                     WHERE memory_tombstones.memory_id = memory_objects.memory_id
                 )
                 ORDER BY memory_id
                 LIMIT ?1",
            )
            .map_err(unavailable("prepare list Memory objects"))?;
        let rows = statement
            .query_map([fetch], |row| {
                Ok(MemoryObjectRow {
                    memory_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?,
                    candidate_id: ObjectId::parse(&row.get::<_, String>(1)?).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?,
                    decision_id: ObjectId::parse(&row.get::<_, String>(2)?).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?,
                    canonical_json: row.get(3)?,
                })
            })
            .map_err(unavailable("query list Memory objects"))?;
        let mut objects = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read Memory objects"))?;
        let truncated = objects.len() > limit;
        if truncated {
            objects.truncate(limit);
        }
        Ok((objects, truncated))
    }

    /// Load one admitted Memory object, hiding tombstoned rows.
    pub fn load_non_tombstoned_memory_object(
        &self,
        memory_id: &ObjectId,
    ) -> Result<Option<MemoryObjectRow>, StorePortError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT candidate_id, decision_id, canonical_json FROM memory_objects
                 WHERE memory_id=?1 AND NOT EXISTS (
                     SELECT 1 FROM memory_tombstones
                     WHERE memory_tombstones.memory_id = memory_objects.memory_id
                 )",
                (memory_id.as_str(),),
                |row| {
                    Ok(MemoryObjectRow {
                        memory_id: memory_id.clone(),
                        candidate_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(
                            |error| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    0,
                                    rusqlite::types::Type::Text,
                                    Box::new(error),
                                )
                            },
                        )?,
                        decision_id: ObjectId::parse(&row.get::<_, String>(1)?).map_err(
                            |error| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    1,
                                    rusqlite::types::Type::Text,
                                    Box::new(error),
                                )
                            },
                        )?,
                        canonical_json: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(unavailable("load listed Memory object"))
    }
}
