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

impl ProtocolStore for SqliteAuthorityStore {
    fn insert_intent(
        &self,
        intent: &IntentRow,
        event: &cognitive_kernel::ports::EventDraft,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin intent"))?;
        let inserted = tx.execute(
            &format!(
                "INSERT INTO intents ({INTENT_COLUMNS}) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)"
            ),
            (
                intent.intent_id.as_str(),
                intent.idempotency_key.as_str(),
                intent.parameters_digest.as_str(),
                intent.action.as_str(),
                intent.target.as_str(),
                intent.effect_object_id.as_str(),
                intent.expected_state_version.get(),
                intent.grant_epoch,
                intent.capability_set_version,
                intent
                    .task_binding
                    .as_ref()
                    .map(|binding| binding.task_ref.as_str()),
                intent
                    .task_binding
                    .as_ref()
                    .map(|binding| binding.contract_epoch),
                intent.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "intent {} or key {} already persisted",
                        intent.intent_id, intent.idempotency_key
                    ),
                });
            }
            Err(err) => return Err(unavailable("insert intent")(err)),
        }
        if let Some(task_binding) = intent.task_binding.as_ref() {
            let eligible_at = scheduler_eligible_at(event)?;
            tx.execute(
                "INSERT INTO scheduler_entries \
                 (task_ref, contract_epoch, state, lease_owner, lease_epoch, lease_expires, next_eligible, attempt_count, cancel_requested) \
                 VALUES (?1, ?2, 'runnable', NULL, 0, NULL, ?3, 0, 0) \
                 ON CONFLICT(task_ref, contract_epoch) DO NOTHING",
                (
                    task_binding.task_ref.as_str(),
                    task_binding.contract_epoch,
                    eligible_at.as_str(),
                ),
            )
            .map_err(unavailable("register scheduler work"))?;
        }
        tx.execute(
            "INSERT INTO events
               (event_id, object_id, domain, object_version, event_type, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                event.event_id.as_str(),
                event.object_id.as_str(),
                event.domain.as_str(),
                event.object_version.get(),
                event.event_type.as_str(),
                event.canonical_json.as_str(),
            ),
        )
        .map_err(unavailable("append intent event"))?;
        let sequence = tx.last_insert_rowid();
        tx.commit().map_err(unavailable("commit intent"))?;
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn load_intent_by_key(&self, key: &str) -> Result<Option<IntentRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {INTENT_COLUMNS} FROM intents WHERE idempotency_key = ?1"
            ))
            .map_err(unavailable("prepare load_intent_by_key"))?;
        statement
            .query_row((key,), row_to_intent)
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_intent_by_key")(other)),
            })
    }

    fn load_intent_for_effect(
        &self,
        effect_object_id: &ObjectId,
    ) -> Result<Option<IntentRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {INTENT_COLUMNS} FROM intents WHERE effect_object_id = ?1"
            ))
            .map_err(unavailable("prepare load_intent_for_effect"))?;
        statement
            .query_row((effect_object_id.as_str(),), row_to_intent)
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_intent_for_effect")(other)),
            })
    }

    fn list_intents_for_task_binding(
        &self,
        task_binding: &TaskBinding,
    ) -> Result<Vec<IntentRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {INTENT_COLUMNS} FROM intents
                 WHERE task_ref = ?1 AND contract_epoch = ?2
                 ORDER BY intent_id"
            ))
            .map_err(unavailable("prepare list_intents_for_task_binding"))?;
        let rows = statement
            .query_map(
                (task_binding.task_ref.as_str(), task_binding.contract_epoch),
                row_to_intent,
            )
            .map_err(unavailable("query list_intents_for_task_binding"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read list_intents_for_task_binding"))
    }

    fn current_fencing_epoch(&self) -> Result<i64, StorePortError> {
        let conn = self.lock()?;
        conn.query_row("SELECT epoch FROM fencing WHERE id = 1", [], |row| {
            row.get(0)
        })
        .map_err(unavailable("read fencing epoch"))
    }

    fn advance_fencing_epoch(&self) -> Result<i64, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin epoch advance"))?;
        tx.execute("UPDATE fencing SET epoch = epoch + 1 WHERE id = 1", [])
            .map_err(unavailable("advance epoch"))?;
        let epoch: i64 = tx
            .query_row("SELECT epoch FROM fencing WHERE id = 1", [], |row| {
                row.get(0)
            })
            .map_err(unavailable("read advanced epoch"))?;
        tx.commit().map_err(unavailable("commit epoch advance"))?;
        Ok(epoch)
    }

    fn list_objects_in_states(
        &self,
        domain: LifecycleDomain,
        states: &[StateName],
    ) -> Result<Vec<StoredObject>, StorePortError> {
        let conn = self.lock()?;
        let placeholders = (0..states.len())
            .map(|index| format!("?{}", index + 2))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT object_id, domain, state, version, body_json FROM governed_objects
             WHERE domain = ?1 AND state IN ({placeholders}) ORDER BY object_id"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(unavailable("prepare list_objects_in_states"))?;
        let mut rows = statement
            .query(rusqlite::params_from_iter(
                std::iter::once(domain.as_str().to_owned())
                    .chain(states.iter().map(|state| state.as_str().to_owned())),
            ))
            .map_err(unavailable("query list_objects_in_states"))?;
        let mut objects = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read objects row"))? {
            let object = row_to_object(
                row.get(0).map_err(unavailable("column object_id"))?,
                row.get(1).map_err(unavailable("column domain"))?,
                row.get(2).map_err(unavailable("column state"))?,
                row.get(3).map_err(unavailable("column version"))?,
                row.get(4).map_err(unavailable("column body_json"))?,
            )?;
            objects.push(object);
        }
        Ok(objects)
    }

    fn append_checkpoint(&self, checkpoint: &CheckpointRow) -> Result<(), StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin checkpoint"))?;
        // F-014 checkpoint sink: the declared epoch must be current.
        verify_fencing_in_tx(&tx, Some(checkpoint.fencing_epoch))?;
        tx.execute(
            "INSERT INTO checkpoints
               (checkpoint_id, loop_object_id, event_high_watermark, fencing_epoch, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                checkpoint.checkpoint_id.as_str(),
                checkpoint.loop_object_id.as_str(),
                checkpoint.event_high_watermark,
                checkpoint.fencing_epoch,
                checkpoint.canonical_json.as_str(),
            ),
        )
        .map_err(unavailable("insert checkpoint"))?;
        tx.commit().map_err(unavailable("commit checkpoint"))?;
        Ok(())
    }

    fn latest_checkpoint(
        &self,
        loop_object_id: &ObjectId,
    ) -> Result<Option<CheckpointRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT checkpoint_id, loop_object_id, event_high_watermark, fencing_epoch,
                        canonical_json
                 FROM checkpoints WHERE loop_object_id = ?1
                 ORDER BY checkpoint_seq DESC LIMIT 1",
            )
            .map_err(unavailable("prepare latest_checkpoint"))?;
        statement
            .query_row((loop_object_id.as_str(),), |row| {
                Ok(CheckpointRow {
                    checkpoint_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            0,
                            "checkpoint_id".into(),
                            rusqlite::types::Type::Text,
                        )
                    })?,
                    loop_object_id: ObjectId::parse(&row.get::<_, String>(1)?).map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            1,
                            "loop_object_id".into(),
                            rusqlite::types::Type::Text,
                        )
                    })?,
                    event_high_watermark: row.get(2)?,
                    fencing_epoch: row.get(3)?,
                    canonical_json: row.get(4)?,
                })
            })
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query latest_checkpoint")(other)),
            })
    }

    fn current_contract_epoch(&self, task_ref: &str) -> Result<i64, StorePortError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT COALESCE(MAX(contract_epoch), 0) FROM task_contracts WHERE task_ref = ?1",
            (task_ref,),
            |row| row.get(0),
        )
        .map_err(unavailable("read current contract epoch"))
    }

    fn load_event_by_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<CommittedEvent>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT sequence, event_id, object_id, domain, object_version, event_type,
                        canonical_json
                 FROM events WHERE event_id = ?1",
            )
            .map_err(unavailable("prepare load_event_by_id"))?;
        let row = statement
            .query_row((event_id.as_str(),), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_event_by_id")(other)),
            })?;
        match row {
            None => Ok(None),
            Some((sequence, event_id, object_id, domain, object_version, event_type, json)) => {
                Ok(Some(CommittedEvent {
                    sequence,
                    event_id: EventId::parse(&event_id).map_err(|err| corrupt("event_id", err))?,
                    object_id: ObjectId::parse(&object_id)
                        .map_err(|err| corrupt("object_id", err))?,
                    domain: LifecycleDomain::parse(&domain)
                        .map_err(|err| corrupt("domain", err))?,
                    object_version: Version::new(object_version)
                        .map_err(|err| corrupt("object_version", err))?,
                    event_type,
                    canonical_json: json,
                }))
            }
        }
    }
}
