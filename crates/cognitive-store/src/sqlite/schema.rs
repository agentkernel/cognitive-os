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

/// Schema of the authority database. Two structural guarantees matter to
/// the contract: the event log and transition records are append-only
/// (triggers), and versions are positive integers (CHECK).
/// Immutable authority schema body for Personal migration plan version 1.
///
/// Shared with `personal_db` so the production open path and the versioned
/// migration plan cannot drift. This is not a machine-contract surface.
pub(crate) const AUTHORITY_SCHEMA_V1: &str = "
CREATE TABLE IF NOT EXISTS governed_objects (
  object_id  TEXT PRIMARY KEY,
  domain     TEXT NOT NULL,
  state      TEXT NOT NULL,
  version    INTEGER NOT NULL CHECK (version >= 1),
  body_json  TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS events (
  sequence       INTEGER PRIMARY KEY AUTOINCREMENT,
  event_id       TEXT NOT NULL UNIQUE,
  object_id      TEXT NOT NULL,
  domain         TEXT NOT NULL,
  object_version INTEGER NOT NULL CHECK (object_version >= 1),
  event_type     TEXT NOT NULL,
  canonical_json TEXT NOT NULL,
  UNIQUE (object_id, object_version)
) STRICT;

CREATE TRIGGER IF NOT EXISTS events_append_only_update
BEFORE UPDATE ON events
BEGIN SELECT RAISE(ABORT, 'append-only: committed events are immutable'); END;

CREATE TRIGGER IF NOT EXISTS events_append_only_delete
BEFORE DELETE ON events
BEGIN SELECT RAISE(ABORT, 'append-only: committed events are immutable'); END;

CREATE TABLE IF NOT EXISTS transition_records (
  record_seq     INTEGER PRIMARY KEY AUTOINCREMENT,
  record_id      TEXT NOT NULL UNIQUE,
  object_id      TEXT NOT NULL,
  domain         TEXT NOT NULL,
  object_version INTEGER NOT NULL CHECK (object_version >= 1),
  canonical_json TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS records_append_only_update
BEFORE UPDATE ON transition_records
BEGIN SELECT RAISE(ABORT, 'append-only: committed records are immutable'); END;

CREATE TRIGGER IF NOT EXISTS records_append_only_delete
BEFORE DELETE ON transition_records
BEGIN SELECT RAISE(ABORT, 'append-only: committed records are immutable'); END;

CREATE TABLE IF NOT EXISTS budgets (
  budget_id  TEXT PRIMARY KEY,
  state_json TEXT NOT NULL,
  version    INTEGER NOT NULL CHECK (version >= 1),
  created_at TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS outbox (
  outbox_sequence INTEGER PRIMARY KEY AUTOINCREMENT,
  event_id        TEXT NOT NULL REFERENCES events(event_id),
  destination     TEXT NOT NULL,
  dispatched_at   TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS intents (
  intent_id              TEXT PRIMARY KEY,
  idempotency_key        TEXT NOT NULL UNIQUE,
  parameters_digest      TEXT NOT NULL,
  action                 TEXT NOT NULL,
  target                 TEXT NOT NULL,
  effect_object_id       TEXT NOT NULL UNIQUE,
  expected_state_version INTEGER NOT NULL,
  grant_epoch            INTEGER NOT NULL,
  capability_set_version INTEGER NOT NULL,
  task_ref               TEXT,
  contract_epoch         INTEGER,
  canonical_json         TEXT NOT NULL,
  CHECK ((task_ref IS NULL) = (contract_epoch IS NULL))
) STRICT;

CREATE TRIGGER IF NOT EXISTS intents_append_only_update
BEFORE UPDATE ON intents
BEGIN SELECT RAISE(ABORT, 'append-only: persisted intents are immutable'); END;

CREATE TRIGGER IF NOT EXISTS intents_append_only_delete
BEFORE DELETE ON intents
BEGIN SELECT RAISE(ABORT, 'append-only: persisted intents are immutable'); END;

CREATE TABLE IF NOT EXISTS fencing (
  id    INTEGER PRIMARY KEY CHECK (id = 1),
  epoch INTEGER NOT NULL CHECK (epoch >= 1)
) STRICT;

INSERT OR IGNORE INTO fencing (id, epoch) VALUES (1, 1);

CREATE TABLE IF NOT EXISTS checkpoints (
  checkpoint_seq       INTEGER PRIMARY KEY AUTOINCREMENT,
  checkpoint_id        TEXT NOT NULL UNIQUE,
  loop_object_id       TEXT NOT NULL,
  event_high_watermark INTEGER NOT NULL,
  fencing_epoch        INTEGER NOT NULL,
  canonical_json       TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS checkpoints_append_only_update
BEFORE UPDATE ON checkpoints
BEGIN SELECT RAISE(ABORT, 'append-only: checkpoints are immutable'); END;

CREATE TRIGGER IF NOT EXISTS checkpoints_append_only_delete
BEFORE DELETE ON checkpoints
BEGIN SELECT RAISE(ABORT, 'append-only: checkpoints are immutable'); END;

CREATE TABLE IF NOT EXISTS user_intent_records (
  record_seq                 INTEGER PRIMARY KEY AUTOINCREMENT,
  record_id                  TEXT NOT NULL UNIQUE,
  conversation_or_scope_ref  TEXT NOT NULL,
  actor_chain_digest         TEXT NOT NULL,
  raw_expression             TEXT NOT NULL,
  recorded_at                TEXT NOT NULL,
  intent_authority_ref       TEXT NOT NULL,
  intent_digest              TEXT NOT NULL,
  canonical_json             TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS user_intents_append_only_update
BEFORE UPDATE ON user_intent_records
BEGIN SELECT RAISE(ABORT, 'append-only: user intent records are immutable'); END;

CREATE TRIGGER IF NOT EXISTS user_intents_append_only_delete
BEFORE DELETE ON user_intent_records
BEGIN SELECT RAISE(ABORT, 'append-only: user intent records are immutable'); END;

CREATE TABLE IF NOT EXISTS intent_interpretations (
  interpretation_seq          INTEGER PRIMARY KEY AUTOINCREMENT,
  interpretation_id           TEXT NOT NULL UNIQUE,
  user_intent_record_id       TEXT NOT NULL,
  recorded_status             TEXT NOT NULL CHECK (recorded_status IN ('candidate','clarification_required')),
  material_ambiguity_count    INTEGER NOT NULL CHECK (material_ambiguity_count >= 0),
  supersedes_interpretation   TEXT,
  interpretation_digest       TEXT NOT NULL,
  canonical_json              TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS interpretations_append_only_update
BEFORE UPDATE ON intent_interpretations
BEGIN SELECT RAISE(ABORT, 'append-only: interpretation candidates are immutable'); END;

CREATE TRIGGER IF NOT EXISTS interpretations_append_only_delete
BEFORE DELETE ON intent_interpretations
BEGIN SELECT RAISE(ABORT, 'append-only: interpretation candidates are immutable'); END;

CREATE TABLE IF NOT EXISTS task_contracts (
  contract_seq           INTEGER PRIMARY KEY AUTOINCREMENT,
  contract_id            TEXT NOT NULL UNIQUE,
  task_ref               TEXT NOT NULL,
  contract_epoch         INTEGER NOT NULL CHECK (contract_epoch >= 1),
  user_intent_record_id  TEXT NOT NULL,
  interpretation_id      TEXT NOT NULL,
  accepted_by            TEXT NOT NULL,
  contract_digest        TEXT NOT NULL,
  canonical_json         TEXT NOT NULL,
  UNIQUE (task_ref, contract_epoch)
) STRICT;

CREATE TRIGGER IF NOT EXISTS task_contracts_append_only_update
BEFORE UPDATE ON task_contracts
BEGIN SELECT RAISE(ABORT, 'append-only: task contracts are immutable'); END;

CREATE TRIGGER IF NOT EXISTS task_contracts_append_only_delete
BEFORE DELETE ON task_contracts
BEGIN SELECT RAISE(ABORT, 'append-only: task contracts are immutable'); END;

CREATE TABLE IF NOT EXISTS loop_progress_facts (
  progress_seq        INTEGER PRIMARY KEY AUTOINCREMENT,
  loop_object_id      TEXT NOT NULL,
  iteration           INTEGER NOT NULL CHECK (iteration >= 1),
  status              TEXT NOT NULL CHECK (status IN ('advanced','none','uncertain','blocked')),
  action_fingerprint  TEXT NOT NULL,
  evidence_refs_json  TEXT NOT NULL,
  recorded_at         TEXT NOT NULL,
  fencing_epoch       INTEGER NOT NULL,
  UNIQUE (loop_object_id, iteration)
) STRICT;

CREATE TRIGGER IF NOT EXISTS progress_facts_append_only_update
BEFORE UPDATE ON loop_progress_facts
BEGIN SELECT RAISE(ABORT, 'append-only: progress facts are immutable'); END;

CREATE TRIGGER IF NOT EXISTS progress_facts_append_only_delete
BEFORE DELETE ON loop_progress_facts
BEGIN SELECT RAISE(ABORT, 'append-only: progress facts are immutable'); END;
";
