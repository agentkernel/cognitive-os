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

pub(crate) type ConsumedWorkerAuthorizationDatabaseRow = (
    String,
    String,
    String,
    i64,
    String,
    i64,
    i64,
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    String,
    String,
    i64,
    String,
    String,
    Option<String>,
    Option<i64>,
    Option<String>,
    Option<i64>,
);

pub(crate) fn unavailable(what: &str) -> impl FnOnce(rusqlite::Error) -> StorePortError + '_ {
    move |err| StorePortError::Unavailable {
        detail: format!("{what}: {err}"),
    }
}

pub(crate) fn is_constraint_violation(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(failure, _)
            if failure.code == rusqlite::ErrorCode::ConstraintViolation
    )
}

/// Every persisted Skill digest must also occur in its immutable canonical
/// payload. This prevents callers from recording an authority digest that is
/// detached from the reviewed package or revision representation.
pub(crate) fn canonical_json_digest_matches(
    canonical_json: &str,
    digest_field: &str,
    expected_digest: &str,
) -> bool {
    serde_json::from_str::<Value>(canonical_json)
        .ok()
        .and_then(|value| value.get(digest_field)?.as_str().map(str::to_owned))
        .as_deref()
        == Some(expected_digest)
}

pub(crate) fn corrupt(what: &str, err: impl std::fmt::Display) -> StorePortError {
    StorePortError::Unavailable {
        detail: format!("stored value unusable ({what}): {err}"),
    }
}

pub(crate) fn effect_class_name(effect_class: EffectClass) -> &'static str {
    match effect_class {
        EffectClass::Pure => "pure",
        EffectClass::LocalEphemeral => "local_ephemeral",
        EffectClass::GovernedExternal => "governed_external",
        EffectClass::EmergencySafety => "emergency_safety",
    }
}

pub(crate) fn parse_effect_class(value: &str) -> Result<EffectClass, StorePortError> {
    match value {
        "pure" => Ok(EffectClass::Pure),
        "local_ephemeral" => Ok(EffectClass::LocalEphemeral),
        "governed_external" => Ok(EffectClass::GovernedExternal),
        "emergency_safety" => Ok(EffectClass::EmergencySafety),
        _ => Err(StorePortError::Unavailable {
            detail: format!("stored daemon descriptor has unknown effect class {value}"),
        }),
    }
}

pub(crate) fn verify_fencing_in_tx(
    tx: &Transaction<'_>,
    declared: Option<i64>,
) -> Result<(), StorePortError> {
    let Some(declared) = declared else {
        return Ok(());
    };
    let current: i64 = tx
        .query_row("SELECT epoch FROM fencing WHERE id = 1", [], |row| {
            row.get(0)
        })
        .map_err(unavailable("read fencing epoch"))?;
    if declared != current {
        return Err(StorePortError::Conflict {
            detail: format!("writer fenced: declared epoch {declared} != current {current}"),
        });
    }
    Ok(())
}

pub(crate) fn row_to_object(
    object_id: String,
    domain: String,
    state: String,
    version: i64,
    body_json: String,
) -> Result<StoredObject, StorePortError> {
    Ok(StoredObject {
        object_id: ObjectId::parse(&object_id).map_err(|err| corrupt("object_id", err))?,
        domain: LifecycleDomain::parse(&domain).map_err(|err| corrupt("domain", err))?,
        state: StateName::parse(&state).map_err(|err| corrupt("state", err))?,
        version: Version::new(version).map_err(|err| corrupt("version", err))?,
        body: serde_json::from_str(&body_json).map_err(|err| corrupt("body_json", err))?,
    })
}

/// Apply the governed transition portion of an authority commit inside an
/// already-open SQLite transaction. Callers retain ownership of the
/// transaction so compound authority boundaries can commit or roll back all
/// of their evidence together.
pub(crate) fn commit_transition_in_transaction(
    transaction: &Transaction<'_>,
    commit: &TransitionCommit,
) -> Result<CommitReceipt, StorePortError> {
    let cas = &commit.cas;
    let changed = transaction
        .execute(
            "UPDATE governed_objects
             SET state = ?1, version = ?2, updated_at = ?3
             WHERE object_id = ?4 AND domain = ?5 AND state = ?6 AND version = ?7",
            (
                cas.to_state.as_str(),
                cas.next_version.get(),
                cas.committed_at.as_str(),
                cas.object_id.as_str(),
                cas.domain.as_str(),
                cas.from_state.as_str(),
                cas.expected_version.get(),
            ),
        )
        .map_err(unavailable("object cas"))?;
    if changed == 0 {
        return Err(StorePortError::Conflict {
            detail: format!(
                "object cas raced: {} not at {}/v{}",
                cas.object_id, cas.from_state, cas.expected_version
            ),
        });
    }

    if let Some(budget) = &commit.budget {
        let changed = transaction
            .execute(
                "UPDATE budgets SET state_json = ?1, version = ?2
                 WHERE budget_id = ?3 AND version = ?4",
                (
                    budget.next_state_canonical_json.as_str(),
                    budget.next_version.get(),
                    budget.budget_id.as_str(),
                    budget.expected_version.get(),
                ),
            )
            .map_err(unavailable("budget cas"))?;
        if changed == 0 {
            return Err(StorePortError::Conflict {
                detail: format!(
                    "budget cas raced: {} not at v{}",
                    budget.budget_id, budget.expected_version
                ),
            });
        }
    }

    let event = &commit.event;
    transaction
        .execute(
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
        .map_err(unavailable("append event"))?;
    let event_sequence = transaction.last_insert_rowid();

    let record = &commit.record;
    transaction
        .execute(
            "INSERT INTO transition_records
             (record_id, object_id, domain, object_version, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                record.record_id.as_str(),
                record.object_id.as_str(),
                record.domain.as_str(),
                record.object_version.get(),
                record.canonical_json.as_str(),
            ),
        )
        .map_err(unavailable("append transition record"))?;

    for outbox in &commit.outbox {
        transaction
            .execute(
                "INSERT INTO outbox (event_id, destination) VALUES (?1, ?2)",
                (outbox.event_id.as_str(), outbox.destination.as_str()),
            )
            .map_err(unavailable("insert outbox row"))?;
    }

    Ok(CommitReceipt { event_sequence })
}

pub(crate) fn row_to_intent(row: &rusqlite::Row<'_>) -> Result<IntentRow, rusqlite::Error> {
    let task_ref: Option<String> = row.get(9)?;
    let contract_epoch: Option<i64> = row.get(10)?;
    let task_binding = match (task_ref, contract_epoch) {
        (Some(task_ref), Some(contract_epoch)) => Some(TaskBinding {
            task_ref,
            contract_epoch,
        }),
        _ => None,
    };
    Ok(IntentRow {
        intent_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(0, "intent_id".into(), rusqlite::types::Type::Text)
        })?,
        idempotency_key: row.get(1)?,
        parameters_digest: row.get(2)?,
        action: row.get(3)?,
        target: row.get(4)?,
        effect_object_id: ObjectId::parse(&row.get::<_, String>(5)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                5,
                "effect_object_id".into(),
                rusqlite::types::Type::Text,
            )
        })?,
        expected_state_version: Version::new(row.get(6)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                6,
                "expected_state_version".into(),
                rusqlite::types::Type::Integer,
            )
        })?,
        grant_epoch: row.get(7)?,
        capability_set_version: row.get(8)?,
        task_binding,
        canonical_json: row.get(11)?,
    })
}

pub(crate) const INTENT_COLUMNS: &str = "intent_id, idempotency_key, parameters_digest, action, target, \
     effect_object_id, expected_state_version, grant_epoch, capability_set_version, \
     task_ref, contract_epoch, canonical_json";

pub(crate) fn scheduler_eligible_at(
    event: &cognitive_kernel::ports::EventDraft,
) -> Result<WallTimestamp, StorePortError> {
    let event_value: serde_json::Value =
        serde_json::from_str(&event.canonical_json).map_err(|error| {
            StorePortError::Unavailable {
                detail: format!("scheduler registration event is not canonical JSON: {error}"),
            }
        })?;
    let event_time = event_value
        .get("event_time")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| StorePortError::Unavailable {
            detail: "scheduler registration event has no event_time".to_owned(),
        })?;
    WallTimestamp::parse(event_time).map_err(|error| StorePortError::Unavailable {
        detail: format!("scheduler registration event_time is invalid: {error}"),
    })
}

pub(crate) fn row_to_user_intent(
    row: &rusqlite::Row<'_>,
) -> Result<UserIntentRecordRow, rusqlite::Error> {
    Ok(UserIntentRecordRow {
        record_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(0, "record_id".into(), rusqlite::types::Type::Text)
        })?,
        conversation_or_scope_ref: row.get(1)?,
        actor_chain_digest: row.get(2)?,
        raw_expression: row.get(3)?,
        recorded_at: WallTimestamp::parse(&row.get::<_, String>(4)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(4, "recorded_at".into(), rusqlite::types::Type::Text)
        })?,
        intent_authority_ref: row.get(5)?,
        intent_digest: row.get(6)?,
        canonical_json: row.get(7)?,
    })
}

pub(crate) const USER_INTENT_COLUMNS: &str = "record_id, conversation_or_scope_ref, actor_chain_digest, \
     raw_expression, recorded_at, intent_authority_ref, intent_digest, canonical_json";

pub(crate) fn row_to_interpretation(
    row: &rusqlite::Row<'_>,
) -> Result<InterpretationRow, rusqlite::Error> {
    let supersedes: Option<String> = row.get(4)?;
    Ok(InterpretationRow {
        interpretation_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                0,
                "interpretation_id".into(),
                rusqlite::types::Type::Text,
            )
        })?,
        user_intent_record_id: ObjectId::parse(&row.get::<_, String>(1)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                1,
                "user_intent_record_id".into(),
                rusqlite::types::Type::Text,
            )
        })?,
        recorded_status: row.get(2)?,
        material_ambiguity_count: row.get(3)?,
        supersedes_interpretation: supersedes
            .map(|raw| {
                ObjectId::parse(&raw).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        4,
                        "supersedes_interpretation".into(),
                        rusqlite::types::Type::Text,
                    )
                })
            })
            .transpose()?,
        interpretation_digest: row.get(5)?,
        canonical_json: row.get(6)?,
    })
}

pub(crate) const INTERPRETATION_COLUMNS: &str = "interpretation_id, user_intent_record_id, recorded_status, \
     material_ambiguity_count, supersedes_interpretation, interpretation_digest, canonical_json";

pub(crate) fn row_to_task_contract(
    row: &rusqlite::Row<'_>,
) -> Result<TaskContractRow, rusqlite::Error> {
    Ok(TaskContractRow {
        contract_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(0, "contract_id".into(), rusqlite::types::Type::Text)
        })?,
        task_ref: row.get(1)?,
        contract_epoch: row.get(2)?,
        user_intent_record_id: ObjectId::parse(&row.get::<_, String>(3)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                3,
                "user_intent_record_id".into(),
                rusqlite::types::Type::Text,
            )
        })?,
        interpretation_id: ObjectId::parse(&row.get::<_, String>(4)?).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                4,
                "interpretation_id".into(),
                rusqlite::types::Type::Text,
            )
        })?,
        accepted_by: row.get(5)?,
        contract_digest: row.get(6)?,
        canonical_json: row.get(7)?,
    })
}

pub(crate) const TASK_CONTRACT_COLUMNS: &str = "contract_id, task_ref, contract_epoch, \
     user_intent_record_id, interpretation_id, accepted_by, contract_digest, canonical_json";

pub(crate) fn append_event_in_tx(
    tx: &Transaction<'_>,
    event: &cognitive_kernel::ports::EventDraft,
) -> Result<i64, StorePortError> {
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
    .map_err(unavailable("append chain event"))?;
    Ok(tx.last_insert_rowid())
}
