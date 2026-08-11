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

impl WorkerAuthorizationStore for SqliteAuthorityStore {
    fn load_worker_iteration_authorization(
        &self,
        authorization_id: &ObjectId,
    ) -> Result<Option<WorkerIterationAuthorizationRow>, StorePortError> {
        let conn = self.lock()?;
        let row = conn.query_row(
            "SELECT authorization_id, worker_authorization_root_id, task_ref, contract_epoch,
                    loop_object_id, iteration, expected_loop_version, selected_candidate_id,
                    intent_id, effect_object_id, budget_id, budget_charge_json,
                    action_fingerprint, issued_fencing_epoch, canonical_json
             FROM worker_iteration_authorizations WHERE authorization_id=?1",
            (authorization_id.as_str(),),
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, String>(12)?,
                    row.get::<_, i64>(13)?,
                    row.get::<_, String>(14)?,
                ))
            },
        );
        let row = match row {
            Ok(row) => row,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(error) => return Err(unavailable("query worker authorization")(error)),
        };
        Ok(Some(WorkerIterationAuthorizationRow {
            authorization_id: ObjectId::parse(&row.0)
                .map_err(|error| corrupt("worker authorization id", error))?,
            worker_authorization_root_id: ObjectId::parse(&row.1)
                .map_err(|error| corrupt("worker authorization root", error))?,
            task_ref: row.2,
            contract_epoch: row.3,
            loop_object_id: ObjectId::parse(&row.4)
                .map_err(|error| corrupt("worker authorization loop", error))?,
            iteration: row.5,
            expected_loop_version: Version::new(row.6)
                .map_err(|error| corrupt("worker authorization loop version", error))?,
            selected_candidate_id: ObjectId::parse(&row.7)
                .map_err(|error| corrupt("worker authorization candidate", error))?,
            intent_id: ObjectId::parse(&row.8)
                .map_err(|error| corrupt("worker authorization intent", error))?,
            effect_object_id: ObjectId::parse(&row.9)
                .map_err(|error| corrupt("worker authorization effect", error))?,
            budget_id: BudgetId::parse(&row.10)
                .map_err(|error| corrupt("worker authorization budget", error))?,
            budget_charge_canonical_json: row.11,
            action_fingerprint: row.12,
            issued_fencing_epoch: row.13,
            canonical_json: row.14,
        }))
    }

    fn load_candidate_admission_receipt_by_selected_candidate_id(
        &self,
        selected_candidate_id: &ObjectId,
    ) -> Result<Option<CandidateAdmissionReceipt>, StorePortError> {
        let conn = self.lock()?;
        let receipt = conn.query_row(
            "SELECT authorization.authorization_id, intent_event.sequence,
                    effect_event.sequence, loop_event.sequence
             FROM worker_iteration_authorizations AS authorization
             JOIN events AS intent_event
               ON intent_event.object_id = authorization.intent_id
              AND intent_event.object_version = 1
              AND intent_event.domain = 'effect'
              AND intent_event.event_type = 'cognitiveos.intent.persisted'
             JOIN events AS effect_event
               ON effect_event.object_id = authorization.effect_object_id
              AND effect_event.object_version = 1
              AND effect_event.domain = 'effect'
              AND effect_event.event_type = 'cognitiveos.object.admitted'
             JOIN events AS loop_event
               ON loop_event.object_id = authorization.loop_object_id
              AND loop_event.object_version = authorization.expected_loop_version + 1
              AND loop_event.domain = 'loop'
              AND loop_event.event_type = 'cognitiveos.state.transition.committed'
             WHERE authorization.selected_candidate_id = ?1",
            (selected_candidate_id.as_str(),),
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        );
        let receipt = match receipt {
            Ok(receipt) => receipt,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(error) => return Err(unavailable("query candidate admission receipt")(error)),
        };
        Ok(Some(CandidateAdmissionReceipt {
            authorization_id: ObjectId::parse(&receipt.0)
                .map_err(|error| corrupt("candidate admission authorization", error))?,
            intent_event_sequence: receipt.1,
            effect_admission_event_sequence: receipt.2,
            loop_transition_event_sequence: receipt.3,
        }))
    }

    fn load_unconsumed_worker_iteration_authorization_for_task_binding(
        &self,
        task_ref: &str,
        contract_epoch: i64,
    ) -> Result<Option<WorkerIterationAuthorizationRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT authorization.authorization_id
                 FROM worker_iteration_authorizations AS authorization
                 LEFT JOIN worker_iteration_authorization_consumptions AS consumption
                   ON consumption.authorization_id = authorization.authorization_id
                 WHERE authorization.task_ref = ?1
                   AND authorization.contract_epoch = ?2
                   AND consumption.authorization_id IS NULL
                 ORDER BY authorization.iteration
                 LIMIT 2",
            )
            .map_err(unavailable("prepare unconsumed worker authorization query"))?;
        let authorization_ids = statement
            .query_map((task_ref, contract_epoch), |row| row.get::<_, String>(0))
            .map_err(unavailable("query unconsumed worker authorizations"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read unconsumed worker authorizations"))?;
        drop(statement);
        drop(conn);

        let [authorization_id] = authorization_ids.as_slice() else {
            return match authorization_ids.len() {
                0 => Ok(None),
                _ => Err(StorePortError::Conflict {
                    detail: "multiple unconsumed worker authorizations match scheduler work"
                        .to_owned(),
                }),
            };
        };
        let authorization_id = ObjectId::parse(authorization_id)
            .map_err(|error| corrupt("unconsumed worker authorization id", error))?;
        self.load_worker_iteration_authorization(&authorization_id)
    }

    fn list_consumed_worker_iteration_authorizations(
        &self,
    ) -> Result<Vec<ConsumedWorkerIterationAuthorization>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT authorization.authorization_id, authorization.worker_authorization_root_id,
                        authorization.task_ref, authorization.contract_epoch,
                        authorization.loop_object_id, authorization.iteration,
                        authorization.expected_loop_version, authorization.selected_candidate_id,
                        authorization.intent_id, authorization.effect_object_id,
                        authorization.budget_id, authorization.budget_charge_json,
                        authorization.action_fingerprint, authorization.issued_fencing_epoch,
                        authorization.canonical_json, consumption.worker_attempt_id,
                        consumption.consumed_fencing_epoch, consumption.consumed_at,
                        consumption.canonical_json, lease_binding.task_ref,
                        lease_binding.contract_epoch, lease_binding.lease_owner,
                        lease_binding.lease_epoch
                 FROM worker_iteration_authorizations AS authorization
                 INNER JOIN worker_iteration_authorization_consumptions AS consumption
                   ON consumption.authorization_id = authorization.authorization_id
                 LEFT JOIN worker_authorization_scheduler_lease_bindings AS lease_binding
                   ON lease_binding.authorization_id = consumption.authorization_id
                 ORDER BY consumption.consumed_at, authorization.authorization_id",
            )
            .map_err(unavailable(
                "prepare consumed worker authorization recovery query",
            ))?;
        let mut rows = statement
            .query(())
            .map_err(unavailable("query consumed worker authorizations"))?;
        let mut recoverable_attempts = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(unavailable("read consumed worker authorization"))?
        {
            let values: ConsumedWorkerAuthorizationDatabaseRow = (|| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                    row.get(12)?,
                    row.get(13)?,
                    row.get(14)?,
                    row.get(15)?,
                    row.get(16)?,
                    row.get(17)?,
                    row.get(18)?,
                    row.get(19)?,
                    row.get(20)?,
                    row.get(21)?,
                    row.get(22)?,
                ))
            })()
            .map_err(unavailable("decode consumed worker authorization"))?;
            let authorization_id = ObjectId::parse(&values.0)
                .map_err(|error| corrupt("worker authorization id", error))?;
            let authorization = WorkerIterationAuthorizationRow {
                authorization_id: authorization_id.clone(),
                worker_authorization_root_id: ObjectId::parse(&values.1)
                    .map_err(|error| corrupt("worker authorization root", error))?,
                task_ref: values.2,
                contract_epoch: values.3,
                loop_object_id: ObjectId::parse(&values.4)
                    .map_err(|error| corrupt("worker authorization loop", error))?,
                iteration: values.5,
                expected_loop_version: Version::new(values.6)
                    .map_err(|error| corrupt("worker authorization loop version", error))?,
                selected_candidate_id: ObjectId::parse(&values.7)
                    .map_err(|error| corrupt("worker authorization candidate", error))?,
                intent_id: ObjectId::parse(&values.8)
                    .map_err(|error| corrupt("worker authorization intent", error))?,
                effect_object_id: ObjectId::parse(&values.9)
                    .map_err(|error| corrupt("worker authorization effect", error))?,
                budget_id: BudgetId::parse(&values.10)
                    .map_err(|error| corrupt("worker authorization budget", error))?,
                budget_charge_canonical_json: values.11,
                action_fingerprint: values.12,
                issued_fencing_epoch: values.13,
                canonical_json: values.14,
            };
            let scheduler_lease = match (values.19, values.20, values.21, values.22) {
                (None, None, None, None) => None,
                (Some(task_ref), Some(contract_epoch), Some(lease_owner), Some(lease_epoch)) => {
                    Some(SchedulerLeaseBinding {
                        task_ref,
                        contract_epoch,
                        lease_owner,
                        lease_epoch,
                    })
                }
                _ => {
                    return Err(StorePortError::Unavailable {
                        detail: "stored worker authorization lease binding is partially populated"
                            .to_owned(),
                    });
                }
            };
            recoverable_attempts.push(ConsumedWorkerIterationAuthorization {
                authorization,
                consumption: WorkerIterationAuthorizationConsumptionRow {
                    authorization_id,
                    worker_attempt_id: ObjectId::parse(&values.15)
                        .map_err(|error| corrupt("worker attempt id", error))?,
                    consumed_fencing_epoch: values.16,
                    consumed_at: WallTimestamp::parse(&values.17)
                        .map_err(|error| corrupt("worker authorization consumption time", error))?,
                    canonical_json: values.18,
                },
                scheduler_lease,
            });
        }
        Ok(recoverable_attempts)
    }

    fn consume_worker_iteration_authorization(
        &self,
        consumption: &WorkerIterationAuthorizationConsumptionRow,
    ) -> Result<(), StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin worker authorization consumption"))?;
        verify_fencing_in_tx(&tx, Some(consumption.consumed_fencing_epoch))?;
        let authorization_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM worker_iteration_authorizations WHERE authorization_id=?1)",
                (consumption.authorization_id.as_str(),),
                |row| row.get(0),
            )
            .map_err(unavailable("verify worker authorization"))?;
        if !authorization_exists {
            return Err(StorePortError::Conflict {
                detail: "worker authorization is not persisted".to_owned(),
            });
        }
        let inserted = tx.execute(
            "INSERT INTO worker_iteration_authorization_consumptions
               (authorization_id, worker_attempt_id, consumed_fencing_epoch, consumed_at, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (consumption.authorization_id.as_str(), consumption.worker_attempt_id.as_str(),
             consumption.consumed_fencing_epoch, consumption.consumed_at.as_str(),
             consumption.canonical_json.as_str()),
        );
        match inserted {
            Ok(_) => tx
                .commit()
                .map_err(unavailable("commit worker authorization consumption")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: "worker authorization was already consumed".to_owned(),
            }),
            Err(error) => Err(unavailable("insert worker authorization consumption")(
                error,
            )),
        }
    }

    fn consume_worker_iteration_authorization_bound_to_scheduler_lease(
        &self,
        request: &BoundWorkerAuthorizationConsumption,
    ) -> Result<(), StorePortError> {
        let consumption = &request.consumption;
        let scheduler_lease = &request.scheduler_lease;
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin bound worker authorization consumption"))?;
        verify_fencing_in_tx(&tx, Some(consumption.consumed_fencing_epoch))?;

        let authorization_matches_lease: bool = tx
            .query_row(
                "SELECT EXISTS(
                   SELECT 1 FROM worker_iteration_authorizations
                   WHERE authorization_id=?1 AND task_ref=?2 AND contract_epoch=?3
                 )",
                (
                    consumption.authorization_id.as_str(),
                    scheduler_lease.task_ref.as_str(),
                    scheduler_lease.contract_epoch,
                ),
                |row| row.get(0),
            )
            .map_err(unavailable("verify bound worker authorization"))?;
        if !authorization_matches_lease {
            return Err(StorePortError::Conflict {
                detail: "worker authorization does not match scheduler work binding".to_owned(),
            });
        }

        let exact_lease_is_active: bool = tx
            .query_row(
                "SELECT EXISTS(
                   SELECT 1 FROM scheduler_entries
                   WHERE task_ref=?1 AND contract_epoch=?2 AND state='leased'
                     AND lease_owner=?3 AND lease_epoch=?4 AND cancel_requested=0
                 )",
                (
                    scheduler_lease.task_ref.as_str(),
                    scheduler_lease.contract_epoch,
                    scheduler_lease.lease_owner.as_str(),
                    scheduler_lease.lease_epoch,
                ),
                |row| row.get(0),
            )
            .map_err(unavailable("verify exact scheduler lease"))?;
        if !exact_lease_is_active {
            return Err(StorePortError::Conflict {
                detail: "scheduler lease is no longer the exact active handoff lease".to_owned(),
            });
        }

        let inserted_consumption = tx.execute(
            "INSERT INTO worker_iteration_authorization_consumptions
               (authorization_id, worker_attempt_id, consumed_fencing_epoch, consumed_at, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                consumption.authorization_id.as_str(),
                consumption.worker_attempt_id.as_str(),
                consumption.consumed_fencing_epoch,
                consumption.consumed_at.as_str(),
                consumption.canonical_json.as_str(),
            ),
        );
        match inserted_consumption {
            Ok(_) => {}
            Err(error) if is_constraint_violation(&error) => {
                return Err(StorePortError::Conflict {
                    detail: "worker authorization was already consumed".to_owned(),
                });
            }
            Err(error) => {
                return Err(unavailable("insert bound worker authorization consumption")(error));
            }
        }

        let inserted_binding = tx.execute(
            "INSERT INTO worker_authorization_scheduler_lease_bindings
               (authorization_id, task_ref, contract_epoch, lease_owner, lease_epoch)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                consumption.authorization_id.as_str(),
                scheduler_lease.task_ref.as_str(),
                scheduler_lease.contract_epoch,
                scheduler_lease.lease_owner.as_str(),
                scheduler_lease.lease_epoch,
            ),
        );
        match inserted_binding {
            Ok(_) => tx
                .commit()
                .map_err(unavailable("commit bound worker authorization consumption")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: "worker authorization scheduler lease binding already exists".to_owned(),
            }),
            Err(error) => Err(unavailable(
                "insert worker authorization scheduler lease binding",
            )(error)),
        }
    }

    fn commit_candidate_admission(
        &self,
        commit: &CandidateAdmissionCommit,
    ) -> Result<CandidateAdmissionReceipt, StorePortError> {
        let intent = &commit.intent;
        let effect_admission = &commit.effect_admission;
        let authorization = &commit.worker_authorization;
        let loop_transition = &commit.loop_transition;

        let consistent_bundle = commit.selected_candidate_id == authorization.selected_candidate_id
            && intent.intent_id == authorization.intent_id
            && intent.effect_object_id == authorization.effect_object_id
            && effect_admission.object.object_id == authorization.effect_object_id
            && effect_admission.object.domain == LifecycleDomain::Effect
            && effect_admission.object.version == Version::INITIAL
            && loop_transition.cas.domain == LifecycleDomain::Loop
            && loop_transition.cas.object_id == authorization.loop_object_id
            && loop_transition.cas.from_state.as_str() == "DECIDE"
            && loop_transition.cas.to_state.as_str() == "ACT"
            && loop_transition.cas.expected_version == authorization.expected_loop_version
            && matches!(
                authorization.expected_loop_version.next(),
                Ok(expected_next_version) if loop_transition.cas.next_version == expected_next_version
            )
            && effect_admission.fencing_epoch == Some(commit.fencing_epoch)
            && loop_transition.fencing_epoch == Some(commit.fencing_epoch)
            && authorization.issued_fencing_epoch == commit.fencing_epoch;
        if !consistent_bundle {
            return Err(StorePortError::Conflict {
                detail: "candidate admission bundle has inconsistent authority bindings".to_owned(),
            });
        }
        let Some(budget) = loop_transition.budget.as_ref() else {
            return Err(StorePortError::Conflict {
                detail: "candidate admission requires an exact budget debit".to_owned(),
            });
        };
        if budget.budget_id != authorization.budget_id {
            return Err(StorePortError::Conflict {
                detail: "candidate admission budget does not match worker authorization".to_owned(),
            });
        }

        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin candidate admission"))?;
        verify_fencing_in_tx(&tx, Some(commit.fencing_epoch))?;

        let candidate_binding = tx.query_row(
            "SELECT task_ref, contract_epoch, parameters_digest, action, target,
                        expected_state_version
                 FROM operation_candidate_proposals WHERE candidate_id=?1",
            (commit.selected_candidate_id.as_str(),),
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        );
        let candidate_binding = match candidate_binding {
            Ok(binding) => binding,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(StorePortError::Conflict {
                    detail: "candidate admission proposal is not persisted".to_owned(),
                });
            }
            Err(error) => return Err(unavailable("load candidate admission proposal")(error)),
        };
        let candidate_matches_authorization = candidate_binding.0 == authorization.task_ref
            && candidate_binding.1 == authorization.contract_epoch
            && candidate_binding.2 == intent.parameters_digest
            && candidate_binding.3 == intent.action
            && candidate_binding.4 == intent.target
            && candidate_binding.5 == intent.expected_state_version.get();
        if !candidate_matches_authorization {
            return Err(StorePortError::Conflict {
                detail: "candidate admission does not match persisted proposal".to_owned(),
            });
        }
        let current_contract_epoch = tx
            .query_row(
                "SELECT COALESCE(MAX(contract_epoch), 0) FROM task_contracts WHERE task_ref=?1",
                (authorization.task_ref.as_str(),),
                |row| row.get::<_, i64>(0),
            )
            .map_err(unavailable("load candidate admission contract epoch"))?;
        if current_contract_epoch != authorization.contract_epoch {
            return Err(StorePortError::Conflict {
                detail: "candidate admission TaskContract epoch was superseded".to_owned(),
            });
        }

        let insert_intent = tx.execute(
            "INSERT INTO intents
               (intent_id, idempotency_key, parameters_digest, action, target, effect_object_id,
                expected_state_version, grant_epoch, capability_set_version, task_ref,
                contract_epoch, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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
        if let Err(error) = insert_intent {
            return if is_constraint_violation(&error) {
                Err(StorePortError::Conflict {
                    detail: "candidate admission intent already exists".to_owned(),
                })
            } else {
                Err(unavailable("insert candidate admission intent")(error))
            };
        }
        let intent_event_sequence = append_event_in_tx(&tx, &commit.intent_event)?;

        let effect_body_json = serde_json::to_string(&effect_admission.object.body)
            .map_err(|error| corrupt("candidate admission effect body", error))?;
        let insert_effect = tx.execute(
            "INSERT INTO governed_objects
               (object_id, domain, state, version, body_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            (
                effect_admission.object.object_id.as_str(),
                effect_admission.object.domain.as_str(),
                effect_admission.object.state.as_str(),
                effect_admission.object.version.get(),
                effect_body_json,
                effect_admission.admitted_at.as_str(),
            ),
        );
        if let Err(error) = insert_effect {
            return if is_constraint_violation(&error) {
                Err(StorePortError::Conflict {
                    detail: "candidate admission effect already exists".to_owned(),
                })
            } else {
                Err(unavailable("insert candidate admission effect")(error))
            };
        }
        let effect_admission_event_sequence = append_event_in_tx(&tx, &effect_admission.event)?;
        for outbox in &effect_admission.outbox {
            tx.execute(
                "INSERT INTO outbox (event_id, destination) VALUES (?1, ?2)",
                (outbox.event_id.as_str(), outbox.destination.as_str()),
            )
            .map_err(unavailable("insert candidate admission effect outbox"))?;
        }

        let insert_authorization = tx.execute(
            "INSERT INTO worker_iteration_authorizations
               (authorization_id, worker_authorization_root_id, task_ref, contract_epoch,
                loop_object_id, iteration, expected_loop_version, selected_candidate_id,
                intent_id, effect_object_id, budget_id, budget_charge_json, action_fingerprint,
                issued_fencing_epoch, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            (
                authorization.authorization_id.as_str(),
                authorization.worker_authorization_root_id.as_str(),
                authorization.task_ref.as_str(),
                authorization.contract_epoch,
                authorization.loop_object_id.as_str(),
                authorization.iteration,
                authorization.expected_loop_version.get(),
                authorization.selected_candidate_id.as_str(),
                authorization.intent_id.as_str(),
                authorization.effect_object_id.as_str(),
                authorization.budget_id.as_str(),
                authorization.budget_charge_canonical_json.as_str(),
                authorization.action_fingerprint.as_str(),
                authorization.issued_fencing_epoch,
                authorization.canonical_json.as_str(),
            ),
        );
        if let Err(error) = insert_authorization {
            return if is_constraint_violation(&error) {
                Err(StorePortError::Conflict {
                    detail: "candidate admission authorization already exists".to_owned(),
                })
            } else {
                Err(unavailable("insert candidate admission authorization")(
                    error,
                ))
            };
        }

        let cas = &loop_transition.cas;
        let changed = tx
            .execute(
                "UPDATE governed_objects SET state=?1, version=?2, updated_at=?3
             WHERE object_id=?4 AND domain=?5 AND state=?6 AND version=?7",
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
            .map_err(unavailable("candidate admission loop cas"))?;
        if changed == 0 {
            return Err(StorePortError::Conflict {
                detail: "candidate admission loop cas raced".to_owned(),
            });
        }
        if let Some(budget) = &loop_transition.budget {
            let changed = tx.execute(
                "UPDATE budgets SET state_json=?1, version=?2 WHERE budget_id=?3 AND version=?4",
                (budget.next_state_canonical_json.as_str(), budget.next_version.get(),
                 budget.budget_id.as_str(), budget.expected_version.get()),
            ).map_err(unavailable("candidate admission budget cas"))?;
            if changed == 0 {
                return Err(StorePortError::Conflict {
                    detail: "candidate admission budget cas raced".to_owned(),
                });
            }
        }
        let loop_transition_event_sequence = append_event_in_tx(&tx, &loop_transition.event)?;
        tx.execute(
            "INSERT INTO transition_records (record_id, object_id, domain, object_version, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (loop_transition.record.record_id.as_str(), loop_transition.record.object_id.as_str(),
             loop_transition.record.domain.as_str(), loop_transition.record.object_version.get(),
             loop_transition.record.canonical_json.as_str()),
        ).map_err(unavailable("append candidate admission loop record"))?;
        for outbox in &loop_transition.outbox {
            tx.execute(
                "INSERT INTO outbox (event_id, destination) VALUES (?1, ?2)",
                (outbox.event_id.as_str(), outbox.destination.as_str()),
            )
            .map_err(unavailable("insert candidate admission loop outbox"))?;
        }
        tx.commit()
            .map_err(unavailable("commit candidate admission"))?;
        Ok(CandidateAdmissionReceipt {
            intent_event_sequence,
            effect_admission_event_sequence,
            loop_transition_event_sequence,
            authorization_id: authorization.authorization_id.clone(),
        })
    }

    fn append_daemon_authorization_snapshot(
        &self,
        snapshot: &DaemonAuthorizationSnapshotRow,
    ) -> Result<(), StorePortError> {
        let conn = self.lock()?;
        let inserted = conn.execute(
            "INSERT INTO daemon_authorization_snapshots
               (snapshot_id, subject_ref, target_ref, action, purpose, grant_epoch,
                capability_set_version, revocation_epoch, observed_at, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (
                snapshot.snapshot_id.as_str(),
                snapshot.subject_ref.as_str(),
                snapshot.target_ref.as_str(),
                snapshot.action.as_str(),
                snapshot.purpose.as_str(),
                snapshot.grant_epoch,
                snapshot.capability_set_version,
                snapshot.revocation_epoch,
                snapshot.observed_at.as_str(),
                snapshot.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!(
                    "daemon authorization snapshot {} already persisted",
                    snapshot.snapshot_id
                ),
            }),
            Err(error) => Err(unavailable("insert daemon authorization snapshot")(error)),
        }
    }

    fn load_latest_daemon_authorization_snapshot(
        &self,
        subject_ref: &str,
        target_ref: &str,
        action: &str,
        purpose: &str,
    ) -> Result<Option<DaemonAuthorizationSnapshotRow>, StorePortError> {
        let conn = self.lock()?;
        let result = conn.query_row(
            "SELECT snapshot_id, subject_ref, target_ref, action, purpose, grant_epoch,
                    capability_set_version, revocation_epoch, observed_at, canonical_json
             FROM daemon_authorization_snapshots
             WHERE subject_ref=?1 AND target_ref=?2 AND action=?3 AND purpose=?4
             ORDER BY snapshot_sequence DESC LIMIT 1",
            (subject_ref, target_ref, action, purpose),
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get::<_, String>(8)?,
                    row.get(9)?,
                ))
            },
        );
        match result {
            Ok((
                snapshot_id,
                subject_ref,
                target_ref,
                action,
                purpose,
                grant_epoch,
                capability_set_version,
                revocation_epoch,
                observed_at,
                canonical_json,
            )) => Ok(Some(DaemonAuthorizationSnapshotRow {
                snapshot_id: ObjectId::parse(&snapshot_id)
                    .map_err(|error| corrupt("daemon authorization snapshot id", error))?,
                subject_ref,
                target_ref,
                action,
                purpose,
                grant_epoch,
                capability_set_version,
                revocation_epoch,
                observed_at: WallTimestamp::parse(&observed_at)
                    .map_err(|error| corrupt("daemon authorization snapshot time", error))?,
                canonical_json,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(unavailable("query daemon authorization snapshot")(error)),
        }
    }

    fn append_daemon_operation_descriptor(
        &self,
        descriptor: &DaemonOperationDescriptorRow,
    ) -> Result<(), StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin daemon descriptor"))?;
        let descriptor_value = &descriptor.descriptor;
        let inserted = tx.execute(
            "INSERT INTO daemon_operation_descriptors
               (descriptor_id, operation_id, action, effect_class, executor, queryable,
                idempotent, descriptor_version, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (
                descriptor.descriptor_id.as_str(),
                descriptor_value.operation_id.as_str(),
                descriptor_value.action.as_str(),
                effect_class_name(descriptor_value.effect_class),
                descriptor_value.executor.as_str(),
                descriptor_value.capabilities.queryable,
                descriptor_value.capabilities.idempotent,
                descriptor_value.descriptor_version,
                descriptor.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(error) if is_constraint_violation(&error) => {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "daemon operation descriptor {} already persisted",
                        descriptor.descriptor_id
                    ),
                });
            }
            Err(error) => return Err(unavailable("insert daemon descriptor")(error)),
        }
        tx.commit().map_err(unavailable("commit daemon descriptor"))
    }

    fn load_daemon_operation_descriptor(
        &self,
        descriptor_id: &ObjectId,
    ) -> Result<Option<DaemonOperationDescriptorRow>, StorePortError> {
        let conn = self.lock()?;
        let result = conn.query_row(
            "SELECT descriptor_id, operation_id, action, effect_class, executor, queryable,
                    idempotent, descriptor_version, canonical_json
             FROM daemon_operation_descriptors WHERE descriptor_id = ?1",
            (descriptor_id.as_str(),),
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, bool>(5)?,
                    row.get::<_, bool>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, String>(8)?,
                ))
            },
        );
        match result {
            Ok((
                stored_id,
                operation_id,
                action,
                stored_effect_class,
                executor,
                queryable,
                idempotent,
                descriptor_version,
                canonical_json,
            )) => Ok(Some(DaemonOperationDescriptorRow {
                descriptor_id: ObjectId::parse(&stored_id)
                    .map_err(|error| corrupt("daemon descriptor id", error))?,
                descriptor: OperationDescriptor {
                    operation_id,
                    action,
                    effect_class: parse_effect_class(&stored_effect_class)?,
                    executor,
                    capabilities: ExecutorCapabilities {
                        queryable,
                        idempotent,
                    },
                    descriptor_version,
                },
                canonical_json,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(unavailable("query daemon descriptor")(error)),
        }
    }

    fn append_operation_candidate_proposal(
        &self,
        proposal: &OperationCandidateProposalRow,
    ) -> Result<(), StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin candidate proposal"))?;
        let inserted = tx.execute(
            "INSERT INTO operation_candidate_proposals
               (candidate_id, task_ref, contract_epoch, candidate_source_ref, tool_ref,
                action, target, parameters_digest, expected_state_version,
                operation_descriptor_ref, canonical_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (
                proposal.candidate_id.as_str(),
                proposal.task_ref.as_str(),
                proposal.contract_epoch,
                proposal.candidate_source_ref.as_str(),
                proposal.tool_ref.as_str(),
                proposal.action.as_str(),
                proposal.target.as_str(),
                proposal.parameters_digest.as_str(),
                proposal.expected_state_version,
                proposal.operation_descriptor_ref.as_str(),
                proposal.canonical_json.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "candidate proposal {} already persisted",
                        proposal.candidate_id
                    ),
                });
            }
            Err(err) => return Err(unavailable("insert candidate proposal")(err)),
        }
        tx.commit()
            .map_err(unavailable("commit candidate proposal"))
    }

    fn load_operation_candidate_proposal(
        &self,
        candidate_id: &ObjectId,
    ) -> Result<Option<OperationCandidateProposalRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT candidate_id, task_ref, contract_epoch, candidate_source_ref, tool_ref,
                        action, target, parameters_digest, expected_state_version,
                        operation_descriptor_ref, canonical_json
                 FROM operation_candidate_proposals WHERE candidate_id = ?1",
            )
            .map_err(unavailable("prepare load candidate proposal"))?;
        statement
            .query_row((candidate_id.as_str(),), |row| {
                let candidate_id: String = row.get(0)?;
                let operation_descriptor_ref: String = row.get(9)?;
                Ok(OperationCandidateProposalRow {
                    candidate_id: ObjectId::parse(&candidate_id).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?,
                    task_ref: row.get(1)?,
                    contract_epoch: row.get(2)?,
                    candidate_source_ref: row.get(3)?,
                    tool_ref: row.get(4)?,
                    action: row.get(5)?,
                    target: row.get(6)?,
                    parameters_digest: row.get(7)?,
                    expected_state_version: row.get(8)?,
                    operation_descriptor_ref: ObjectId::parse(&operation_descriptor_ref).map_err(
                        |error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                9,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        },
                    )?,
                    canonical_json: row.get(10)?,
                })
            })
            .map(Some)
            .or_else(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(unavailable("query load candidate proposal")(other)),
            })
    }
}
