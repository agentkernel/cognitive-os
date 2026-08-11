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

pub struct SqliteAuthorityStore {
    pub(crate) conn: Mutex<Connection>,
}

impl SqliteAuthorityStore {
    /// Open (creating if needed) an authority database in WAL mode with
    /// `synchronous=FULL`, and install the schema.
    pub fn open(path: &Path) -> Result<Self, StorePortError> {
        let conn = Connection::open(path).map_err(unavailable("open"))?;
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .map_err(unavailable("set journal_mode"))?;
        if !journal_mode.eq_ignore_ascii_case("wal") {
            return Err(StorePortError::Unavailable {
                detail: format!("authority database refused WAL mode: {journal_mode}"),
            });
        }
        conn.execute_batch(
            "PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;",
        )
        .map_err(unavailable("set pragmas"))?;
        let schema = [
            AUTHORITY_SCHEMA_V1,
            SCHEDULER_SCHEMA_CURRENT,
            WORKER_AUTHORIZATION_SCHEMA_V4,
            DAEMON_OPERATION_DESCRIPTOR_SCHEMA_V5,
            DAEMON_AUTHORIZATION_SNAPSHOT_SCHEMA_V6,
            WORKER_ITERATION_AUTHORIZATION_SCHEMA_V7,
            WORKER_ITERATION_AUTHORIZATION_CONSUMPTION_SCHEMA_V8,
            WORKER_AUTHORIZATION_LEASE_BINDING_SCHEMA_V9,
            CONTINUATION_AUTHORITY_SCHEMA_V10,
            CONTINUATION_AUTHORITY_CONSUMPTION_SCHEMA_V11,
            CONTEXT_STORE_SCHEMA_V12,
            WORKSPACE_CONTEXT_SOURCE_SCHEMA_V13,
            CONTEXT_AUTHORIZATION_FACT_SCHEMA_V14,
            SCHEDULER_EXECUTION_POLICY_SCHEMA_V15,
            MEMORY_ADMISSION_SCHEMA_V16,
            MEMORY_SEARCH_SCHEMA_V17,
        ]
        .join("\n");
        conn.execute_batch(&schema)
            .map_err(unavailable("install schema"))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open an existing authority database read-only (reader snapshot per
    /// ADR-0002 rule 5; also models a degraded read-only volume: every
    /// governed write through this handle fails closed).
    pub fn open_read_only(path: &Path) -> Result<Self, StorePortError> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(unavailable("open read-only"))?;
        conn.execute_batch("PRAGMA query_only=ON; PRAGMA busy_timeout=5000;")
            .map_err(unavailable("set read-only pragmas"))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Connection>, StorePortError> {
        self.conn.lock().map_err(|_| StorePortError::Unavailable {
            detail: "authority connection poisoned".to_owned(),
        })
    }
}

impl AuthorityStore for SqliteAuthorityStore {
    fn load_object(
        &self,
        domain: LifecycleDomain,
        object_id: &ObjectId,
    ) -> Result<Option<StoredObject>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT object_id, domain, state, version, body_json
                 FROM governed_objects WHERE object_id = ?1 AND domain = ?2",
            )
            .map_err(unavailable("prepare load_object"))?;
        let mut rows = statement
            .query((object_id.as_str(), domain.as_str()))
            .map_err(unavailable("query load_object"))?;
        match rows.next().map_err(unavailable("read load_object"))? {
            None => Ok(None),
            Some(row) => {
                let object = row_to_object(
                    row.get(0).map_err(unavailable("column object_id"))?,
                    row.get(1).map_err(unavailable("column domain"))?,
                    row.get(2).map_err(unavailable("column state"))?,
                    row.get(3).map_err(unavailable("column version"))?,
                    row.get(4).map_err(unavailable("column body_json"))?,
                )?;
                Ok(Some(object))
            }
        }
    }

    fn admit_object(&self, admission: &ObjectAdmission) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin admission"))?;
        verify_fencing_in_tx(&tx, admission.fencing_epoch)?;
        let object = &admission.object;
        let body_json = serde_json::to_string(&object.body)
            .map_err(|err| corrupt("body serialization", err))?;
        let inserted = tx.execute(
            "INSERT INTO governed_objects
               (object_id, domain, state, version, body_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            (
                object.object_id.as_str(),
                object.domain.as_str(),
                object.state.as_str(),
                object.version.get(),
                body_json,
                admission.admitted_at.as_str(),
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!("object {} already exists", object.object_id),
                });
            }
            Err(err) => return Err(unavailable("insert object")(err)),
        }
        let event = &admission.event;
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
        .map_err(unavailable("append admission event"))?;
        let sequence = tx.last_insert_rowid();
        for outbox in &admission.outbox {
            tx.execute(
                "INSERT INTO outbox (event_id, destination) VALUES (?1, ?2)",
                (outbox.event_id.as_str(), outbox.destination.as_str()),
            )
            .map_err(unavailable("insert outbox row"))?;
        }
        tx.commit().map_err(unavailable("commit admission"))?;
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn commit_transition(
        &self,
        commit: &TransitionCommit,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin transition"))?;
        verify_fencing_in_tx(&tx, commit.fencing_epoch)?;
        let receipt = commit_transition_in_transaction(&tx, commit)?;
        tx.commit().map_err(unavailable("commit transition"))?;
        Ok(receipt)
    }

    fn load_budget(&self, budget_id: &BudgetId) -> Result<Option<StoredBudget>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached("SELECT state_json, version FROM budgets WHERE budget_id = ?1")
            .map_err(unavailable("prepare load_budget"))?;
        let mut rows = statement
            .query((budget_id.as_str(),))
            .map_err(unavailable("query load_budget"))?;
        match rows.next().map_err(unavailable("read load_budget"))? {
            None => Ok(None),
            Some(row) => {
                let state_json: String = row.get(0).map_err(unavailable("column state_json"))?;
                let version: i64 = row.get(1).map_err(unavailable("column version"))?;
                let state: BudgetState = serde_json::from_str(&state_json)
                    .map_err(|err| corrupt("budget state", err))?;
                Ok(Some(StoredBudget {
                    budget_id: budget_id.clone(),
                    state,
                    version: Version::new(version).map_err(|err| corrupt("budget version", err))?,
                }))
            }
        }
    }

    fn create_budget(
        &self,
        budget_id: &BudgetId,
        state_canonical_json: &str,
        created_at: &WallTimestamp,
    ) -> Result<(), StorePortError> {
        let conn = self.lock()?;
        let inserted = conn.execute(
            "INSERT INTO budgets (budget_id, state_json, version, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            (
                budget_id.as_str(),
                state_canonical_json,
                Version::INITIAL.get(),
                created_at.as_str(),
            ),
        );
        match inserted {
            Ok(_) => Ok(()),
            Err(err) if is_constraint_violation(&err) => Err(StorePortError::Conflict {
                detail: format!("budget {budget_id} already exists"),
            }),
            Err(err) => Err(unavailable("insert budget")(err)),
        }
    }

    fn read_events(
        &self,
        after_sequence: i64,
        limit: usize,
    ) -> Result<Vec<CommittedEvent>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT sequence, event_id, object_id, domain, object_version, event_type,
                        canonical_json
                 FROM events WHERE sequence > ?1 ORDER BY sequence ASC LIMIT ?2",
            )
            .map_err(unavailable("prepare read_events"))?;
        let mut rows = statement
            .query((after_sequence, limit as i64))
            .map_err(unavailable("query read_events"))?;
        let mut events = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read events row"))? {
            let event_id: String = row.get(1).map_err(unavailable("column event_id"))?;
            let object_id: String = row.get(2).map_err(unavailable("column object_id"))?;
            let domain: String = row.get(3).map_err(unavailable("column domain"))?;
            let object_version: i64 = row.get(4).map_err(unavailable("column object_version"))?;
            events.push(CommittedEvent {
                sequence: row.get(0).map_err(unavailable("column sequence"))?,
                event_id: EventId::parse(&event_id).map_err(|err| corrupt("event_id", err))?,
                object_id: ObjectId::parse(&object_id).map_err(|err| corrupt("object_id", err))?,
                domain: LifecycleDomain::parse(&domain).map_err(|err| corrupt("domain", err))?,
                object_version: Version::new(object_version)
                    .map_err(|err| corrupt("object_version", err))?,
                event_type: row.get(5).map_err(unavailable("column event_type"))?,
                canonical_json: row.get(6).map_err(unavailable("column canonical_json"))?,
            });
        }
        Ok(events)
    }

    fn pending_outbox(&self, limit: usize) -> Result<Vec<OutboxEntry>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT outbox_sequence, event_id, destination FROM outbox
                 WHERE dispatched_at IS NULL ORDER BY outbox_sequence ASC LIMIT ?1",
            )
            .map_err(unavailable("prepare pending_outbox"))?;
        let mut rows = statement
            .query((limit as i64,))
            .map_err(unavailable("query pending_outbox"))?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read outbox row"))? {
            let event_id: String = row.get(1).map_err(unavailable("column event_id"))?;
            entries.push(OutboxEntry {
                outbox_sequence: row.get(0).map_err(unavailable("column outbox_sequence"))?,
                event_id: EventId::parse(&event_id).map_err(|err| corrupt("event_id", err))?,
                destination: row.get(2).map_err(unavailable("column destination"))?,
                dispatched: false,
            });
        }
        Ok(entries)
    }

    fn mark_outbox_dispatched(
        &self,
        outbox_sequence: i64,
        dispatched_at: &WallTimestamp,
    ) -> Result<(), StorePortError> {
        let conn = self.lock()?;
        let changed = conn
            .execute(
                "UPDATE outbox SET dispatched_at = ?1
                 WHERE outbox_sequence = ?2 AND dispatched_at IS NULL",
                (dispatched_at.as_str(), outbox_sequence),
            )
            .map_err(unavailable("mark outbox dispatched"))?;
        if changed == 0 {
            return Err(StorePortError::Conflict {
                detail: format!("no pending outbox row {outbox_sequence}"),
            });
        }
        Ok(())
    }
}
