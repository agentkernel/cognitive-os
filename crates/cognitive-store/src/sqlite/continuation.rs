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
    UserIntentRecordRow, VerificationReportRow, VerificationRequestRow, VerificationStartCommit,
    WorkerAuthorizationStore, WorkerIterationAuthorizationConsumptionRow,
    WorkerIterationAuthorizationRow, WorkspaceContextSourceRow,
};
use cognitive_kernel::{BudgetState, EffectClass, ExecutorCapabilities, OperationDescriptor};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use super::*;

impl ContinuationAuthorityStore for SqliteAuthorityStore {
    fn begin_verification_atomically(
        &self,
        commit: &VerificationStartCommit,
    ) -> Result<CommitReceipt, StorePortError> {
        let fixed = &commit.fixed_post_state;
        let request = &commit.verification_request;
        let transition = &commit.loop_transition;
        let bindings_match = request.fixed_post_state_id == fixed.fixed_post_state_id
            && request.task_binding == fixed.task_binding
            && request.loop_object_id == fixed.loop_object_id
            && transition.cas.object_id == fixed.loop_object_id
            && transition.cas.domain == LifecycleDomain::Loop
            && transition.cas.from_state.as_str() == "ACT"
            && transition.cas.to_state.as_str() == "VERIFY"
            && request.expected_loop_version == transition.cas.next_version
            && fixed.recorded_fencing_epoch == request.issued_fencing_epoch
            && transition.fencing_epoch == Some(fixed.recorded_fencing_epoch);
        if !bindings_match {
            return Err(StorePortError::Conflict {
                detail: "verification start members do not share one authority binding".to_owned(),
            });
        }
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin verification start"))?;
        verify_fencing_in_tx(&transaction, Some(fixed.recorded_fencing_epoch))?;
        let current_contract_epoch: i64 = transaction
            .query_row(
                "SELECT COALESCE(MAX(contract_epoch), 0) FROM task_contracts WHERE task_ref=?1",
                (fixed.task_binding.task_ref.as_str(),),
                |row| row.get(0),
            )
            .map_err(unavailable("read verification contract epoch"))?;
        if current_contract_epoch != fixed.task_binding.contract_epoch {
            return Err(StorePortError::Conflict {
                detail: "verification start TaskContract epoch is stale".to_owned(),
            });
        }
        let subject_is_current_and_closed: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM governed_objects WHERE object_id=?1 AND domain=?2 AND version=?3 AND state IN ('RECONCILED','VERIFIED','VERIFY_FAILED'))",
                (
                    fixed.subject_object_id.as_str(),
                    fixed.subject_domain.as_str(),
                    fixed.subject_version.get(),
                ),
                |row| row.get(0),
            )
            .map_err(unavailable("read verification fixed subject"))?;
        if fixed.subject_domain != LifecycleDomain::Effect || !subject_is_current_and_closed {
            return Err(StorePortError::Conflict {
                detail: "verification start requires a current closed Effect post-state".to_owned(),
            });
        }
        transaction.execute(
            "INSERT INTO fixed_post_states (fixed_post_state_id, task_ref, contract_epoch, loop_object_id, subject_domain, subject_object_id, subject_version, recorded_fencing_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (fixed.fixed_post_state_id.as_str(), fixed.task_binding.task_ref.as_str(), fixed.task_binding.contract_epoch, fixed.loop_object_id.as_str(), fixed.subject_domain.as_str(), fixed.subject_object_id.as_str(), fixed.subject_version.get(), fixed.recorded_fencing_epoch, fixed.canonical_json.as_str()),
        ).map_err(unavailable("insert verification fixed post-state"))?;
        transaction.execute(
            "INSERT INTO verification_requests (verification_request_id, fixed_post_state_id, task_ref, contract_epoch, loop_object_id, expected_loop_version, verifier_ref, verifier_version, criteria_json, issued_fencing_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (request.verification_request_id.as_str(), request.fixed_post_state_id.as_str(), request.task_binding.task_ref.as_str(), request.task_binding.contract_epoch, request.loop_object_id.as_str(), request.expected_loop_version.get(), request.verifier_ref.as_str(), request.verifier_version.as_str(), request.criteria_canonical_json.as_str(), request.issued_fencing_epoch, request.canonical_json.as_str()),
        ).map_err(unavailable("insert verification request"))?;
        let receipt = commit_transition_in_transaction(&transaction, transition)?;
        transaction
            .commit()
            .map_err(unavailable("commit verification start"))?;
        Ok(receipt)
    }

    fn append_fixed_post_state(&self, row: &FixedPostStateRow) -> Result<(), StorePortError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin fixed post-state"))?;
        verify_fencing_in_tx(&transaction, Some(row.recorded_fencing_epoch))?;
        transaction.execute(
            "INSERT INTO fixed_post_states (fixed_post_state_id, task_ref, contract_epoch, loop_object_id, subject_domain, subject_object_id, subject_version, recorded_fencing_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (row.fixed_post_state_id.as_str(), row.task_binding.task_ref.as_str(), row.task_binding.contract_epoch, row.loop_object_id.as_str(), row.subject_domain.as_str(), row.subject_object_id.as_str(), row.subject_version.get(), row.recorded_fencing_epoch, row.canonical_json.as_str()),
        ).map_err(unavailable("insert fixed post-state"))?;
        transaction
            .commit()
            .map_err(unavailable("commit fixed post-state"))
    }

    fn load_fixed_post_state(
        &self,
        fixed_post_state_id: &ObjectId,
    ) -> Result<Option<FixedPostStateRow>, StorePortError> {
        let connection = self.lock()?;
        let result = connection.query_row(
            "SELECT task_ref, contract_epoch, loop_object_id, subject_domain, subject_object_id, subject_version, recorded_fencing_epoch, canonical_json FROM fixed_post_states WHERE fixed_post_state_id=?1",
            (fixed_post_state_id.as_str(),),
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, i64>(5)?, row.get::<_, i64>(6)?, row.get::<_, String>(7)?)),
        );
        match result {
            Ok((
                task_ref,
                contract_epoch,
                loop_object_id,
                subject_domain,
                subject_object_id,
                subject_version,
                recorded_fencing_epoch,
                canonical_json,
            )) => Ok(Some(FixedPostStateRow {
                fixed_post_state_id: fixed_post_state_id.clone(),
                task_binding: TaskBinding {
                    task_ref,
                    contract_epoch,
                },
                loop_object_id: ObjectId::parse(&loop_object_id)
                    .map_err(|error| corrupt("fixed post-state loop id", error))?,
                subject_domain: LifecycleDomain::parse(&subject_domain)
                    .map_err(|error| corrupt("fixed post-state subject domain", error))?,
                subject_object_id: ObjectId::parse(&subject_object_id)
                    .map_err(|error| corrupt("fixed post-state subject id", error))?,
                subject_version: Version::new(subject_version)
                    .map_err(|error| corrupt("fixed post-state subject version", error))?,
                recorded_fencing_epoch,
                canonical_json,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(unavailable("query fixed post-state")(error)),
        }
    }

    fn append_verification_request(
        &self,
        row: &VerificationRequestRow,
    ) -> Result<(), StorePortError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin verification request"))?;
        verify_fencing_in_tx(&transaction, Some(row.issued_fencing_epoch))?;
        transaction.execute(
            "INSERT INTO verification_requests (verification_request_id, fixed_post_state_id, task_ref, contract_epoch, loop_object_id, expected_loop_version, verifier_ref, verifier_version, criteria_json, issued_fencing_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (row.verification_request_id.as_str(), row.fixed_post_state_id.as_str(), row.task_binding.task_ref.as_str(), row.task_binding.contract_epoch, row.loop_object_id.as_str(), row.expected_loop_version.get(), row.verifier_ref.as_str(), row.verifier_version.as_str(), row.criteria_canonical_json.as_str(), row.issued_fencing_epoch, row.canonical_json.as_str()),
        ).map_err(unavailable("insert verification request"))?;
        transaction
            .commit()
            .map_err(unavailable("commit verification request"))
    }

    fn load_verification_request(
        &self,
        verification_request_id: &ObjectId,
    ) -> Result<Option<VerificationRequestRow>, StorePortError> {
        let connection = self.lock()?;
        let result = connection.query_row(
            "SELECT fixed_post_state_id, task_ref, contract_epoch, loop_object_id, expected_loop_version, verifier_ref, verifier_version, criteria_json, issued_fencing_epoch, canonical_json FROM verification_requests WHERE verification_request_id=?1",
            (verification_request_id.as_str(),),
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, String>(3)?, row.get::<_, i64>(4)?, row.get::<_, String>(5)?, row.get::<_, String>(6)?, row.get::<_, String>(7)?, row.get::<_, i64>(8)?, row.get::<_, String>(9)?)),
        );
        match result {
            Ok((
                fixed_post_state_id,
                task_ref,
                contract_epoch,
                loop_object_id,
                expected_loop_version,
                verifier_ref,
                verifier_version,
                criteria_canonical_json,
                issued_fencing_epoch,
                canonical_json,
            )) => Ok(Some(VerificationRequestRow {
                verification_request_id: verification_request_id.clone(),
                fixed_post_state_id: ObjectId::parse(&fixed_post_state_id)
                    .map_err(|error| corrupt("verification request fixed post-state id", error))?,
                task_binding: TaskBinding {
                    task_ref,
                    contract_epoch,
                },
                loop_object_id: ObjectId::parse(&loop_object_id)
                    .map_err(|error| corrupt("verification request loop id", error))?,
                expected_loop_version: Version::new(expected_loop_version)
                    .map_err(|error| corrupt("verification request loop version", error))?,
                verifier_ref,
                verifier_version,
                criteria_canonical_json,
                issued_fencing_epoch,
                canonical_json,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(unavailable("query verification request")(error)),
        }
    }

    fn append_verification_report(
        &self,
        row: &VerificationReportRow,
    ) -> Result<(), StorePortError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin verification report"))?;
        verify_fencing_in_tx(&transaction, Some(row.recorded_fencing_epoch))?;
        transaction.execute(
            "INSERT INTO verification_reports (verification_report_id, verification_request_id, fixed_post_state_id, verifier_ref, verifier_version, status, evidence_refs_json, completed_at, recorded_fencing_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (row.verification_report_id.as_str(), row.verification_request_id.as_str(), row.fixed_post_state_id.as_str(), row.verifier_ref.as_str(), row.verifier_version.as_str(), row.status.as_str(), row.evidence_refs_canonical_json.as_str(), row.completed_at.as_str(), row.recorded_fencing_epoch, row.canonical_json.as_str()),
        ).map_err(unavailable("insert verification report"))?;
        transaction
            .commit()
            .map_err(unavailable("commit verification report"))
    }

    fn load_verification_report(
        &self,
        verification_report_id: &ObjectId,
    ) -> Result<Option<VerificationReportRow>, StorePortError> {
        let connection = self.lock()?;
        let result = connection.query_row(
            "SELECT verification_request_id, fixed_post_state_id, verifier_ref, verifier_version, status, evidence_refs_json, completed_at, recorded_fencing_epoch, canonical_json FROM verification_reports WHERE verification_report_id=?1",
            (verification_report_id.as_str(),),
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, String>(5)?, row.get::<_, String>(6)?, row.get::<_, i64>(7)?, row.get::<_, String>(8)?)),
        );
        match result {
            Ok((
                verification_request_id,
                fixed_post_state_id,
                verifier_ref,
                verifier_version,
                status,
                evidence_refs_canonical_json,
                completed_at,
                recorded_fencing_epoch,
                canonical_json,
            )) => Ok(Some(VerificationReportRow {
                verification_report_id: verification_report_id.clone(),
                verification_request_id: ObjectId::parse(&verification_request_id)
                    .map_err(|error| corrupt("verification report request id", error))?,
                fixed_post_state_id: ObjectId::parse(&fixed_post_state_id)
                    .map_err(|error| corrupt("verification report fixed post-state id", error))?,
                verifier_ref,
                verifier_version,
                status,
                evidence_refs_canonical_json,
                completed_at: WallTimestamp::parse(&completed_at)
                    .map_err(|error| corrupt("verification report completion time", error))?,
                recorded_fencing_epoch,
                canonical_json,
            })),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(unavailable("query verification report")(error)),
        }
    }

    fn issue_continuation_authorization(
        &self,
        row: &ContinuationAuthorizationRow,
    ) -> Result<(), StorePortError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin continuation authorization"))?;
        verify_fencing_in_tx(&transaction, Some(row.issued_fencing_epoch))?;
        let current_contract_epoch: i64 = transaction
            .query_row(
                "SELECT COALESCE(MAX(contract_epoch), 0) FROM task_contracts WHERE task_ref=?1",
                (row.task_binding.task_ref.as_str(),),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("read continuation contract epoch"))?;
        if current_contract_epoch != row.task_binding.contract_epoch {
            return Err(StorePortError::Conflict {
                detail: "continuation authorization contract epoch is stale".to_owned(),
            });
        }
        let loop_state: Option<(String, i64)> = transaction
            .query_row(
                "SELECT state, version FROM governed_objects WHERE object_id=?1 AND domain='loop'",
                (row.loop_object_id.as_str(),),
                |database_row| Ok((database_row.get(0)?, database_row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("read continuation loop"))?;
        if loop_state != Some(("CONTINUE".to_owned(), row.expected_loop_version.get())) {
            return Err(StorePortError::Conflict {
                detail: "continuation authorization loop is not current CONTINUE state".to_owned(),
            });
        }
        let checkpoint_exists: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM checkpoints WHERE checkpoint_id=?1 AND loop_object_id=?2 AND fencing_epoch=?3)",
                (row.checkpoint_id.as_str(), row.loop_object_id.as_str(), row.issued_fencing_epoch),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("read continuation checkpoint"))?;
        if !checkpoint_exists {
            return Err(StorePortError::Conflict {
                detail: "continuation authorization checkpoint is unavailable or fenced".to_owned(),
            });
        }
        let report_exists: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM verification_reports AS report JOIN verification_requests AS request ON request.verification_request_id=report.verification_request_id AND request.fixed_post_state_id=report.fixed_post_state_id JOIN fixed_post_states AS fixed_state ON fixed_state.fixed_post_state_id=report.fixed_post_state_id AND fixed_state.task_ref=request.task_ref AND fixed_state.contract_epoch=request.contract_epoch AND fixed_state.loop_object_id=request.loop_object_id JOIN governed_objects AS subject ON subject.object_id=fixed_state.subject_object_id AND subject.domain=fixed_state.subject_domain WHERE report.verification_report_id=?1 AND report.status='passed' AND report.recorded_fencing_epoch=?2 AND request.issued_fencing_epoch=?2 AND fixed_state.recorded_fencing_epoch=?2 AND request.task_ref=?3 AND request.contract_epoch=?4 AND request.loop_object_id=?5 AND subject.version=fixed_state.subject_version AND NOT (subject.domain='task' AND subject.state='COMPLETED'))",
                (row.verification_report_id.as_str(), row.issued_fencing_epoch, row.task_binding.task_ref.as_str(), row.task_binding.contract_epoch, row.loop_object_id.as_str()),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("read continuation verification report"))?;
        if !report_exists {
            return Err(StorePortError::Conflict {
                detail: "continuation authorization requires a current passed verified post-state"
                    .to_owned(),
            });
        }
        transaction.execute(
            "INSERT INTO continuation_authorizations (continuation_authorization_id, task_ref, contract_epoch, loop_object_id, iteration, expected_loop_version, checkpoint_id, budget_id, budget_charge_json, verification_report_id, issued_fencing_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            (row.continuation_authorization_id.as_str(), row.task_binding.task_ref.as_str(), row.task_binding.contract_epoch, row.loop_object_id.as_str(), row.iteration, row.expected_loop_version.get(), row.checkpoint_id.as_str(), row.budget_id.as_str(), row.budget_charge_canonical_json.as_str(), row.verification_report_id.as_str(), row.issued_fencing_epoch, row.canonical_json.as_str()),
        ).map_err(unavailable("insert continuation authorization"))?;
        transaction
            .commit()
            .map_err(unavailable("commit continuation authorization"))
    }

    fn consume_continuation_authorization_bound_to_scheduler_lease(
        &self,
        request: &BoundContinuationAuthorizationConsumption,
        transition: &TransitionCommit,
    ) -> Result<CommitReceipt, StorePortError> {
        let consumption = &request.consumption;
        let scheduler_lease = &request.scheduler_lease;
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin bound continuation consumption"))?;
        verify_fencing_in_tx(&transaction, Some(consumption.consumed_fencing_epoch))?;

        let authorization_matches_lease: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM continuation_authorizations WHERE continuation_authorization_id=?1 AND task_ref=?2 AND contract_epoch=?3 AND issued_fencing_epoch=?4)",
                (consumption.continuation_authorization_id.as_str(), scheduler_lease.task_ref.as_str(), scheduler_lease.contract_epoch, consumption.consumed_fencing_epoch),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("verify bound continuation authorization"))?;
        if !authorization_matches_lease {
            return Err(StorePortError::Conflict {
                detail: "continuation authorization does not match scheduler work binding"
                    .to_owned(),
            });
        }

        let exact_lease_is_active: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM scheduler_entries WHERE task_ref=?1 AND contract_epoch=?2 AND state='leased' AND lease_owner=?3 AND lease_epoch=?4 AND cancel_requested=0)",
                (scheduler_lease.task_ref.as_str(), scheduler_lease.contract_epoch, scheduler_lease.lease_owner.as_str(), scheduler_lease.lease_epoch),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("verify exact continuation scheduler lease"))?;
        if !exact_lease_is_active {
            return Err(StorePortError::Conflict {
                detail: "scheduler lease is no longer the exact active continuation lease"
                    .to_owned(),
            });
        }

        let transition_matches_authorization: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM continuation_authorizations WHERE continuation_authorization_id=?1 AND loop_object_id=?2 AND expected_loop_version=?3 AND budget_id=?4 AND budget_charge_json=?5)",
                (consumption.continuation_authorization_id.as_str(), transition.cas.object_id.as_str(), transition.cas.expected_version.get(), transition.budget.as_ref().map(|budget| budget.budget_id.as_str()).unwrap_or(""), transition.budget.as_ref().map(|budget| budget.charge_canonical_json.as_str()).unwrap_or("")),
                |database_row| database_row.get(0),
            )
            .map_err(unavailable("verify continuation transition binding"))?;
        let is_legal_continuation_entry = transition.cas.domain == LifecycleDomain::Loop
            && transition.cas.from_state.as_str() == "CONTINUE"
            && transition.cas.to_state.as_str() == "OBSERVE"
            && transition.fencing_epoch == Some(consumption.consumed_fencing_epoch)
            && transition.budget.is_some();
        if !transition_matches_authorization || !is_legal_continuation_entry {
            return Err(StorePortError::Conflict {
                detail: "continuation authority does not match the prepared loop entry".to_owned(),
            });
        }

        let inserted_consumption = transaction.execute(
            "INSERT INTO continuation_authorization_consumptions (continuation_authorization_id, consumed_fencing_epoch, consumed_at, canonical_json) VALUES (?1, ?2, ?3, ?4)",
            (consumption.continuation_authorization_id.as_str(), consumption.consumed_fencing_epoch, consumption.consumed_at.as_str(), consumption.canonical_json.as_str()),
        );
        match inserted_consumption {
            Ok(_) => {}
            Err(error) if is_constraint_violation(&error) => {
                return Err(StorePortError::Conflict {
                    detail: "continuation authorization was already consumed".to_owned(),
                });
            }
            Err(error) => {
                return Err(unavailable("insert continuation consumption")(error));
            }
        }

        let inserted_binding = transaction.execute(
            "INSERT INTO continuation_authorization_scheduler_lease_bindings (continuation_authorization_id, task_ref, contract_epoch, lease_owner, lease_epoch) VALUES (?1, ?2, ?3, ?4, ?5)",
            (consumption.continuation_authorization_id.as_str(), scheduler_lease.task_ref.as_str(), scheduler_lease.contract_epoch, scheduler_lease.lease_owner.as_str(), scheduler_lease.lease_epoch),
        );
        match inserted_binding {
            Ok(_) => {}
            Err(error) if is_constraint_violation(&error) => {
                return Err(StorePortError::Conflict {
                    detail: "continuation authorization scheduler lease binding already exists"
                        .to_owned(),
                });
            }
            Err(error) => {
                return Err(unavailable("insert continuation scheduler lease binding")(
                    error,
                ));
            }
        }

        let receipt = commit_transition_in_transaction(&transaction, transition)?;
        transaction
            .commit()
            .map_err(unavailable("commit bound continuation entry"))?;
        Ok(receipt)
    }

    fn load_unconsumed_continuation_authorization(
        &self,
        task_binding: &TaskBinding,
    ) -> Result<Option<ContinuationAuthorizationRow>, StorePortError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare_cached("SELECT continuation_authorization_id, loop_object_id, iteration, expected_loop_version, checkpoint_id, budget_id, budget_charge_json, verification_report_id, issued_fencing_epoch, canonical_json FROM continuation_authorizations WHERE task_ref=?1 AND contract_epoch=?2 AND continuation_authorization_id NOT IN (SELECT continuation_authorization_id FROM continuation_authorization_consumptions) ORDER BY iteration LIMIT 2").map_err(unavailable("prepare continuation authorization query"))?;
        let rows = statement
            .query_map(
                (task_binding.task_ref.as_str(), task_binding.contract_epoch),
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, i64>(8)?,
                        row.get::<_, String>(9)?,
                    ))
                },
            )
            .map_err(unavailable("query continuation authorization"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read continuation authorization"))?;
        if rows.len() > 1 {
            return Err(StorePortError::Conflict {
                detail: "multiple unconsumed continuation authorizations match scheduler work"
                    .to_owned(),
            });
        }
        let Some((
            authorization_id,
            loop_object_id,
            iteration,
            expected_loop_version,
            checkpoint_id,
            budget_id,
            budget_charge_canonical_json,
            verification_report_id,
            issued_fencing_epoch,
            canonical_json,
        )) = rows.into_iter().next()
        else {
            return Ok(None);
        };
        Ok(Some(ContinuationAuthorizationRow {
            continuation_authorization_id: ObjectId::parse(&authorization_id)
                .map_err(|error| corrupt("continuation authorization id", error))?,
            task_binding: task_binding.clone(),
            loop_object_id: ObjectId::parse(&loop_object_id)
                .map_err(|error| corrupt("continuation authorization loop id", error))?,
            iteration,
            expected_loop_version: Version::new(expected_loop_version)
                .map_err(|error| corrupt("continuation authorization loop version", error))?,
            checkpoint_id: ObjectId::parse(&checkpoint_id)
                .map_err(|error| corrupt("continuation authorization checkpoint id", error))?,
            budget_id: BudgetId::parse(&budget_id)
                .map_err(|error| corrupt("continuation authorization budget id", error))?,
            budget_charge_canonical_json,
            verification_report_id: ObjectId::parse(&verification_report_id)
                .map_err(|error| corrupt("continuation authorization report id", error))?,
            issued_fencing_epoch,
            canonical_json,
        }))
    }
}
