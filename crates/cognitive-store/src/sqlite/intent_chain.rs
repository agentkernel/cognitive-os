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
use cognitive_contracts::generated::task_contract::TaskContract;
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
    StoredBudget, StoredObject, TaskBinding, TaskContractRow, TaskExecutionBootstrap,
    TaskExecutionBootstrapRepair, TransitionCommit, UserIntentRecordRow, VerificationReportRow,
    VerificationRequestRow, WorkerAuthorizationStore, WorkerIterationAuthorizationConsumptionRow,
    WorkerIterationAuthorizationRow, WorkspaceContextSourceRow,
};
use cognitive_kernel::{BudgetState, EffectClass, ExecutorCapabilities, OperationDescriptor};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use super::*;

impl GovernanceObjectStore for SqliteAuthorityStore {
    fn load_governed_object_header(
        &self,
        object_id: &ObjectId,
    ) -> Result<Option<GovernedObjectHeader>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT canonical_json FROM user_intent_records WHERE record_id = ?1 \
                 UNION ALL SELECT canonical_json FROM intent_interpretations WHERE interpretation_id = ?1 \
                 UNION ALL SELECT canonical_json FROM task_contracts WHERE contract_id = ?1",
            )
            .map_err(unavailable("prepare governed-header lookup"))?;
        let rows = statement
            .query_map([object_id.as_str()], |row| row.get::<_, String>(0))
            .map_err(unavailable("query governed-header lookup"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read governed-header lookup"))?;
        let [canonical_json] = rows.as_slice() else {
            return if rows.is_empty() {
                Ok(None)
            } else {
                Err(StorePortError::Unavailable {
                    detail: "ambiguous governed object identity".to_owned(),
                })
            };
        };
        let value: serde_json::Value = serde_json::from_str(canonical_json)
            .map_err(|err| corrupt("governed canonical json", err))?;
        let header: GovernedObjectHeader =
            serde_json::from_value(value.get("header").cloned().ok_or_else(|| {
                StorePortError::Unavailable {
                    detail: "governed object has no header".to_owned(),
                }
            })?)
            .map_err(|err| corrupt("governed header", err))?;
        if header.id.0 != object_id.as_str() {
            return Err(StorePortError::Unavailable {
                detail: "governed header identity mismatch".to_owned(),
            });
        }
        Ok(Some(header))
    }
}

fn insert_task_contract_row_in_transaction(
    transaction: &Transaction<'_>,
    contract: &TaskContractRow,
    expected_current_epoch: i64,
) -> Result<(), StorePortError> {
    let current: i64 = transaction
        .query_row(
            "SELECT COALESCE(MAX(contract_epoch), 0) FROM task_contracts WHERE task_ref = ?1",
            (contract.task_ref.as_str(),),
            |row| row.get(0),
        )
        .map_err(unavailable("read contract epoch"))?;
    if current != expected_current_epoch {
        return Err(StorePortError::Conflict {
            detail: format!(
                "contract epoch raced for {}: expected {expected_current_epoch}, current {current}",
                contract.task_ref
            ),
        });
    }
    if contract.contract_epoch != expected_current_epoch + 1 {
        return Err(StorePortError::Conflict {
            detail: format!(
                "contract epoch must advance by exactly one: current {expected_current_epoch}, proposed {}",
                contract.contract_epoch
            ),
        });
    }
    let inserted = transaction.execute(
        &format!(
            "INSERT INTO task_contracts ({TASK_CONTRACT_COLUMNS}) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"
        ),
        (
            contract.contract_id.as_str(),
            contract.task_ref.as_str(),
            contract.contract_epoch,
            contract.user_intent_record_id.as_str(),
            contract.interpretation_id.as_str(),
            contract.accepted_by.as_str(),
            contract.contract_digest.as_str(),
            contract.canonical_json.as_str(),
        ),
    );
    match inserted {
        Ok(_) => Ok(()),
        Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
            detail: format!(
                "contract {} or epoch {} of {} already persisted",
                contract.contract_id, contract.contract_epoch, contract.task_ref
            ),
        }),
        Err(error) => Err(unavailable("insert task contract")(error)),
    }
}

fn validate_execution_bootstrap(
    contract: &TaskContractRow,
    bootstrap: &TaskExecutionBootstrap,
) -> Result<(), StorePortError> {
    let conflict = |detail: String| StorePortError::Conflict { detail };
    let contract_payload: TaskContract = serde_json::from_str(&contract.canonical_json)
        .map_err(|error| corrupt("schedulable TaskContract", error))?;
    if contract_payload.task_ref != contract.task_ref
        || contract_payload.contract_epoch != contract.contract_epoch
    {
        return Err(conflict(
            "TaskContract row and canonical execution binding differ".to_owned(),
        ));
    }
    let contract_loop_id = contract_payload
        .loop_object_id
        .as_ref()
        .ok_or_else(|| conflict("TaskContract has no Loop identity".to_owned()))?;
    let contract_budget_id = contract_payload
        .budget_id
        .as_ref()
        .ok_or_else(|| conflict("TaskContract has no Budget identity".to_owned()))?;
    let loop_admission = &bootstrap.loop_admission;
    if loop_admission.object.object_id.as_str() != contract_loop_id.0.as_str()
        || loop_admission.object.domain != LifecycleDomain::Loop
        || loop_admission.object.state.as_str() != "START"
        || loop_admission.object.version != Version::INITIAL
        || loop_admission.event.object_id != loop_admission.object.object_id
        || loop_admission.event.domain != LifecycleDomain::Loop
        || loop_admission.event.object_version != Version::INITIAL
        || loop_admission.fencing_epoch.is_none()
    {
        return Err(conflict(
            "Task execution bootstrap Loop is not the contract-named fenced START admission"
                .to_owned(),
        ));
    }
    if bootstrap.budget_id.as_str() != contract_budget_id.0.as_str() {
        return Err(conflict(
            "Task execution bootstrap Budget differs from the TaskContract".to_owned(),
        ));
    }
    let budget_state: BudgetState = serde_json::from_str(&bootstrap.budget_state_canonical_json)
        .map_err(|error| corrupt("Task execution bootstrap Budget", error))?;
    BudgetState::new(budget_state.remaining().clone())
        .map_err(|error| corrupt("Task execution bootstrap Budget", error))?;
    let body = &loop_admission.object.body;
    if body.get("task_ref").and_then(Value::as_str) != Some(contract.task_ref.as_str())
        || body.get("contract_epoch").and_then(Value::as_i64) != Some(contract.contract_epoch)
        || body.get("task_contract_id").and_then(Value::as_str)
            != Some(contract.contract_id.as_str())
        || body.get("budget_id").and_then(Value::as_str) != Some(bootstrap.budget_id.as_str())
    {
        return Err(conflict(
            "Task execution bootstrap Loop body differs from the TaskContract binding".to_owned(),
        ));
    }
    Ok(())
}

impl IntentChainStore for SqliteAuthorityStore {
    fn insert_user_intent(
        &self,
        record: &UserIntentRecordRow,
        event: &cognitive_kernel::ports::EventDraft,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin user intent"))?;
        let inserted = tx.execute(
            &format!(
                "INSERT INTO user_intent_records ({USER_INTENT_COLUMNS}) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"
            ),
            (
                record.record_id.as_str(),
                record.conversation_or_scope_ref.as_str(),
                record.actor_chain_digest.as_str(),
                record.raw_expression.as_str(),
                record.recorded_at.as_str(),
                record.intent_authority_ref.as_str(),
                record.intent_digest.as_str(),
                record.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!("user intent record {} already fixed", record.record_id),
                });
            }
            Err(err) => return Err(unavailable("insert user intent")(err)),
        }
        let sequence = append_event_in_tx(&tx, event)?;
        tx.commit().map_err(unavailable("commit user intent"))?;
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn load_user_intent(
        &self,
        record_id: &ObjectId,
    ) -> Result<Option<UserIntentRecordRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {USER_INTENT_COLUMNS} FROM user_intent_records WHERE record_id = ?1"
            ))
            .map_err(unavailable("prepare load_user_intent"))?;
        statement
            .query_row((record_id.as_str(),), row_to_user_intent)
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_user_intent")(other)),
            })
    }

    fn list_user_intents_for_scope(
        &self,
        conversation_or_scope_ref: &str,
    ) -> Result<Vec<UserIntentRecordRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {USER_INTENT_COLUMNS} FROM user_intent_records
                 WHERE conversation_or_scope_ref = ?1 ORDER BY record_seq ASC"
            ))
            .map_err(unavailable("prepare list_user_intents_for_scope"))?;
        let mut rows = statement
            .query((conversation_or_scope_ref,))
            .map_err(unavailable("query list_user_intents_for_scope"))?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read user intent row"))? {
            records.push(row_to_user_intent(row).map_err(|err| corrupt("user intent row", err))?);
        }
        Ok(records)
    }

    fn insert_interpretation(
        &self,
        interpretation: &InterpretationRow,
        event: &cognitive_kernel::ports::EventDraft,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin interpretation"))?;
        let inserted = tx.execute(
            &format!(
                "INSERT INTO intent_interpretations ({INTERPRETATION_COLUMNS}) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7)"
            ),
            (
                interpretation.interpretation_id.as_str(),
                interpretation.user_intent_record_id.as_str(),
                interpretation.recorded_status.as_str(),
                interpretation.material_ambiguity_count,
                interpretation
                    .supersedes_interpretation
                    .as_ref()
                    .map(|id| id.as_str()),
                interpretation.interpretation_digest.as_str(),
                interpretation.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "interpretation {} already persisted",
                        interpretation.interpretation_id
                    ),
                });
            }
            Err(err) => return Err(unavailable("insert interpretation")(err)),
        }
        let sequence = append_event_in_tx(&tx, event)?;
        tx.commit().map_err(unavailable("commit interpretation"))?;
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn load_interpretation(
        &self,
        interpretation_id: &ObjectId,
    ) -> Result<Option<InterpretationRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {INTERPRETATION_COLUMNS} FROM intent_interpretations
                 WHERE interpretation_id = ?1"
            ))
            .map_err(unavailable("prepare load_interpretation"))?;
        statement
            .query_row((interpretation_id.as_str(),), row_to_interpretation)
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_interpretation")(other)),
            })
    }

    fn insert_task_contract(
        &self,
        contract: &TaskContractRow,
        event: &cognitive_kernel::ports::EventDraft,
        expected_current_epoch: i64,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin task contract"))?;
        // Contract-epoch CAS inside the transaction: the current epoch
        // must equal the caller's expectation and the new row must be its
        // immediate successor. Any race rolls the whole unit back.
        insert_task_contract_row_in_transaction(&tx, contract, expected_current_epoch)?;
        let sequence = append_event_in_tx(&tx, event)?;
        tx.commit().map_err(unavailable("commit task contract"))?;
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn insert_task_contract_with_execution_bootstrap(
        &self,
        contract: &TaskContractRow,
        event: &cognitive_kernel::ports::EventDraft,
        expected_current_epoch: i64,
        bootstrap: &TaskExecutionBootstrap,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin schedulable task admission"))?;
        verify_fencing_in_tx(&transaction, bootstrap.loop_admission.fencing_epoch)?;
        if event.object_id != contract.contract_id
            || event.domain != LifecycleDomain::Task
            || event.object_version != Version::INITIAL
        {
            return Err(StorePortError::Conflict {
                detail: "TaskContract event does not match the admitted contract".to_owned(),
            });
        }
        validate_execution_bootstrap(contract, bootstrap)?;
        insert_task_contract_row_in_transaction(&transaction, contract, expected_current_epoch)?;
        let contract_event_sequence = append_event_in_tx(&transaction, event)?;

        let loop_object = &bootstrap.loop_admission.object;
        let loop_body_json = serde_json::to_string(&loop_object.body)
            .map_err(|error| corrupt("Task admission Loop body", error))?;
        transaction
            .execute(
                "INSERT INTO governed_objects
                   (object_id, domain, state, version, body_json, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                (
                    loop_object.object_id.as_str(),
                    loop_object.domain.as_str(),
                    loop_object.state.as_str(),
                    loop_object.version.get(),
                    loop_body_json.as_str(),
                    bootstrap.loop_admission.admitted_at.as_str(),
                ),
            )
            .map_err(|error| {
                if is_constraint_violation(&error) {
                    StorePortError::Conflict {
                        detail: format!(
                            "Task admission Loop {} already exists",
                            loop_object.object_id
                        ),
                    }
                } else {
                    unavailable("insert Task admission Loop")(error)
                }
            })?;
        append_event_in_tx(&transaction, &bootstrap.loop_admission.event)?;
        for outbox in &bootstrap.loop_admission.outbox {
            transaction
                .execute(
                    "INSERT INTO outbox (event_id, destination) VALUES (?1, ?2)",
                    (outbox.event_id.as_str(), outbox.destination.as_str()),
                )
                .map_err(unavailable("insert Task admission Loop outbox"))?;
        }

        transaction
            .execute(
                "INSERT INTO budgets (budget_id, state_json, version, created_at)
                 VALUES (?1, ?2, 1, ?3)",
                (
                    bootstrap.budget_id.as_str(),
                    bootstrap.budget_state_canonical_json.as_str(),
                    bootstrap.budget_created_at.as_str(),
                ),
            )
            .map_err(|error| {
                if is_constraint_violation(&error) {
                    StorePortError::Conflict {
                        detail: format!(
                            "Task admission Budget {} already exists",
                            bootstrap.budget_id
                        ),
                    }
                } else {
                    unavailable("insert Task admission Budget")(error)
                }
            })?;

        let eligible_at = scheduler_eligible_at(event)?;
        transaction
            .execute(
                "INSERT INTO scheduler_entries
                   (task_ref, contract_epoch, state, lease_owner, lease_epoch, lease_expires,
                    next_eligible, attempt_count, cancel_requested)
                 VALUES (?1, ?2, 'runnable', NULL, 0, NULL, ?3, 0, 0)",
                (
                    contract.task_ref.as_str(),
                    contract.contract_epoch,
                    eligible_at.as_str(),
                ),
            )
            .map_err(|error| {
                if is_constraint_violation(&error) {
                    StorePortError::Conflict {
                        detail: format!(
                            "scheduler work for {} at epoch {} already exists",
                            contract.task_ref, contract.contract_epoch
                        ),
                    }
                } else {
                    unavailable("insert Task admission scheduler work")(error)
                }
            })?;

        transaction
            .commit()
            .map_err(unavailable("commit schedulable task admission"))?;
        Ok(CommitReceipt {
            event_sequence: contract_event_sequence,
        })
    }

    fn repair_task_execution_bootstrap(
        &self,
        contract: &TaskContractRow,
        bootstrap: &TaskExecutionBootstrap,
    ) -> Result<TaskExecutionBootstrapRepair, StorePortError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin Task execution bootstrap repair"))?;
        verify_fencing_in_tx(&transaction, bootstrap.loop_admission.fencing_epoch)?;
        validate_execution_bootstrap(contract, bootstrap)?;

        let current_epoch: i64 = transaction
            .query_row(
                "SELECT COALESCE(MAX(contract_epoch), 0) FROM task_contracts WHERE task_ref=?1",
                (contract.task_ref.as_str(),),
                |row| row.get(0),
            )
            .map_err(unavailable("read bootstrap repair contract epoch"))?;
        if current_epoch != contract.contract_epoch {
            return Err(StorePortError::Conflict {
                detail: "bootstrap repair TaskContract is not the current epoch".to_owned(),
            });
        }
        let persisted_contract: Option<(String, String, String)> = transaction
            .query_row(
                "SELECT contract_id, contract_digest, canonical_json
                 FROM task_contracts WHERE task_ref=?1 AND contract_epoch=?2",
                (contract.task_ref.as_str(), contract.contract_epoch),
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(unavailable("read bootstrap repair TaskContract"))?;
        if persisted_contract
            != Some((
                contract.contract_id.as_str().to_owned(),
                contract.contract_digest.clone(),
                contract.canonical_json.clone(),
            ))
        {
            return Err(StorePortError::Conflict {
                detail: "bootstrap repair TaskContract differs from durable authority".to_owned(),
            });
        }

        let loop_object = &bootstrap.loop_admission.object;
        let existing_loop: Option<(String, String, i64, String)> = transaction
            .query_row(
                "SELECT domain, state, version, body_json
                 FROM governed_objects WHERE object_id=?1",
                (loop_object.object_id.as_str(),),
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(unavailable("read Task bootstrap Loop"))?;
        let loop_created = if let Some((domain, _state, version, body_json)) = existing_loop {
            let body: Value = serde_json::from_str(&body_json)
                .map_err(|error| corrupt("existing Task bootstrap Loop", error))?;
            if domain != LifecycleDomain::Loop.as_str()
                || version < Version::INITIAL.get()
                || body.get("task_ref").and_then(Value::as_str) != Some(contract.task_ref.as_str())
                || body.get("contract_epoch").and_then(Value::as_i64)
                    != Some(contract.contract_epoch)
                || body.get("task_contract_id").and_then(Value::as_str)
                    != Some(contract.contract_id.as_str())
                || body.get("budget_id").and_then(Value::as_str)
                    != Some(bootstrap.budget_id.as_str())
            {
                return Err(StorePortError::Conflict {
                    detail: "existing Task bootstrap Loop has a different authority binding"
                        .to_owned(),
                });
            }
            false
        } else {
            let loop_body_json = serde_json::to_string(&loop_object.body)
                .map_err(|error| corrupt("Task bootstrap repair Loop body", error))?;
            transaction
                .execute(
                    "INSERT INTO governed_objects
                       (object_id, domain, state, version, body_json, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                    (
                        loop_object.object_id.as_str(),
                        loop_object.domain.as_str(),
                        loop_object.state.as_str(),
                        loop_object.version.get(),
                        loop_body_json.as_str(),
                        bootstrap.loop_admission.admitted_at.as_str(),
                    ),
                )
                .map_err(unavailable("insert repaired Task bootstrap Loop"))?;
            append_event_in_tx(&transaction, &bootstrap.loop_admission.event)?;
            for outbox in &bootstrap.loop_admission.outbox {
                transaction
                    .execute(
                        "INSERT INTO outbox (event_id, destination) VALUES (?1, ?2)",
                        (outbox.event_id.as_str(), outbox.destination.as_str()),
                    )
                    .map_err(unavailable("insert repaired Task bootstrap Loop outbox"))?;
            }
            true
        };

        let existing_budget: Option<String> = transaction
            .query_row(
                "SELECT state_json FROM budgets WHERE budget_id=?1",
                (bootstrap.budget_id.as_str(),),
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("read Task bootstrap Budget"))?;
        let budget_created = if let Some(state_json) = existing_budget {
            let state: BudgetState = serde_json::from_str(&state_json)
                .map_err(|error| corrupt("existing Task bootstrap Budget", error))?;
            BudgetState::new(state.remaining().clone())
                .map_err(|error| corrupt("existing Task bootstrap Budget", error))?;
            false
        } else {
            transaction
                .execute(
                    "INSERT INTO budgets (budget_id, state_json, version, created_at)
                     VALUES (?1, ?2, 1, ?3)",
                    (
                        bootstrap.budget_id.as_str(),
                        bootstrap.budget_state_canonical_json.as_str(),
                        bootstrap.budget_created_at.as_str(),
                    ),
                )
                .map_err(unavailable("insert repaired Task bootstrap Budget"))?;
            true
        };

        let scheduler_exists: bool = transaction
            .query_row(
                "SELECT EXISTS(
                   SELECT 1 FROM scheduler_entries WHERE task_ref=?1 AND contract_epoch=?2
                 )",
                (contract.task_ref.as_str(), contract.contract_epoch),
                |row| row.get(0),
            )
            .map_err(unavailable("read Task bootstrap scheduler work"))?;
        let scheduler_created = if scheduler_exists {
            false
        } else {
            transaction
                .execute(
                    "INSERT INTO scheduler_entries
                       (task_ref, contract_epoch, state, lease_owner, lease_epoch, lease_expires,
                        next_eligible, attempt_count, cancel_requested)
                     VALUES (?1, ?2, 'runnable', NULL, 0, NULL, ?3, 0, 0)",
                    (
                        contract.task_ref.as_str(),
                        contract.contract_epoch,
                        bootstrap.budget_created_at.as_str(),
                    ),
                )
                .map_err(unavailable("insert repaired Task bootstrap scheduler work"))?;
            true
        };

        transaction
            .commit()
            .map_err(unavailable("commit Task execution bootstrap repair"))?;
        Ok(TaskExecutionBootstrapRepair {
            loop_created,
            budget_created,
            scheduler_created,
        })
    }

    fn load_task_contract(
        &self,
        task_ref: &str,
        contract_epoch: i64,
    ) -> Result<Option<TaskContractRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {TASK_CONTRACT_COLUMNS} FROM task_contracts
                 WHERE task_ref = ?1 AND contract_epoch = ?2"
            ))
            .map_err(unavailable("prepare load_task_contract"))?;
        statement
            .query_row((task_ref, contract_epoch), row_to_task_contract)
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load_task_contract")(other)),
            })
    }

    fn list_current_task_contracts(&self) -> Result<Vec<TaskContractRow>, StorePortError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare_cached(&format!(
                "SELECT {TASK_CONTRACT_COLUMNS}
                 FROM task_contracts AS contract
                 WHERE contract.contract_epoch = (
                   SELECT MAX(current.contract_epoch)
                   FROM task_contracts AS current
                   WHERE current.task_ref = contract.task_ref
                 )
                 ORDER BY contract.task_ref"
            ))
            .map_err(unavailable("prepare list current TaskContracts"))?;
        let rows = statement
            .query_map([], row_to_task_contract)
            .map_err(unavailable("query list current TaskContracts"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read current TaskContract"))
    }

    fn list_intents_for_task(&self, task_ref: &str) -> Result<Vec<IntentRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(&format!(
                "SELECT {INTENT_COLUMNS} FROM intents WHERE task_ref = ?1 ORDER BY intent_id"
            ))
            .map_err(unavailable("prepare list_intents_for_task"))?;
        let mut rows = statement
            .query((task_ref,))
            .map_err(unavailable("query list_intents_for_task"))?;
        let mut intents = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read intent row"))? {
            intents.push(row_to_intent(row).map_err(|err| corrupt("intent row", err))?);
        }
        Ok(intents)
    }
}
