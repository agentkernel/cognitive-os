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

pub(crate) fn invalid_context_payload(
    kind: &str,
    detail: impl std::fmt::Display,
) -> StorePortError {
    StorePortError::Unavailable {
        detail: format!("invalid {kind} append payload: {detail}"),
    }
}

pub(crate) fn parse_and_verify_context_payload(
    canonical_json: &str,
    kind: &str,
) -> Result<Value, StorePortError> {
    let payload: Value = serde_json::from_str(canonical_json)
        .map_err(|error| invalid_context_payload(kind, error))?;
    verify_content_digest(
        &payload,
        &["/header/content_digest"],
        GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN,
        "/header/content_digest",
    )
    .map_err(|error| invalid_context_payload(kind, error))?;
    Ok(payload)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ContextAuthorizationFactsPayload {
    header: GovernedObjectHeader,
    fact_set_id: String,
    subject_ref: String,
    tenant_id: String,
    principal: cognitive_kernel::authz::PrincipalFacts,
    actor_chain: cognitive_kernel::authz::ActorChainFacts,
    membership: Option<cognitive_kernel::authz::MembershipFacts>,
    capability_links: Vec<cognitive_domain::capability::CapabilityConstraints>,
    explicit_denies: Vec<cognitive_kernel::authz::DenyRule>,
    capability_set_version: i64,
    issued_revocation_epoch: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ContextRevocationFactPayload {
    header: GovernedObjectHeader,
    revocation_fact_id: String,
    tenant_id: String,
    revocation_epoch: i64,
    revoked_subject_ref: Option<String>,
    revoked_capability_ref: Option<String>,
}

pub(crate) fn parse_context_authorization_facts(
    canonical_json: &str,
) -> Result<ContextAuthorizationFactsPayload, StorePortError> {
    let payload = parse_and_verify_context_payload(canonical_json, "ContextAuthorizationFacts")?;
    serde_json::from_value(payload)
        .map_err(|error| invalid_context_payload("ContextAuthorizationFacts", error))
}

pub(crate) fn validate_context_authorization_facts_row(
    facts: &ContextAuthorizationFactsRow,
) -> Result<(), StorePortError> {
    let payload = parse_context_authorization_facts(&facts.canonical_json)?;
    if payload.header.id.0 != facts.fact_set_id.as_str()
        || payload.header.r#type != "ContextAuthorizationFacts"
        || payload.fact_set_id != facts.fact_set_id.as_str()
        || payload.subject_ref != facts.subject_ref
        || payload.tenant_id != facts.tenant_id
        || payload.principal != facts.principal
        || payload.actor_chain != facts.actor_chain
        || payload.membership != facts.membership
        || payload.capability_links != facts.capability_links
        || payload.explicit_denies != facts.explicit_denies
        || payload.capability_set_version != facts.capability_set_version
        || payload.issued_revocation_epoch != facts.issued_revocation_epoch
    {
        return Err(invalid_context_payload(
            "ContextAuthorizationFacts",
            "row metadata differs from canonical authorization facts",
        ));
    }
    facts.reconstruct_snapshot(
        facts.issued_revocation_epoch,
        WallTimestamp::parse("2026-01-01T00:00:00Z")
            .map_err(|error| invalid_context_payload("ContextAuthorizationFacts", error))?,
    )?;
    Ok(())
}

pub(crate) fn validate_context_revocation_fact_row(
    fact: &ContextRevocationFactRow,
) -> Result<(), StorePortError> {
    let payload = parse_and_verify_context_payload(&fact.canonical_json, "ContextRevocationFact")?;
    let payload: ContextRevocationFactPayload = serde_json::from_value(payload)
        .map_err(|error| invalid_context_payload("ContextRevocationFact", error))?;
    if payload.header.id.0 != fact.revocation_fact_id.as_str()
        || payload.header.r#type != "ContextRevocationFact"
        || payload.revocation_fact_id != fact.revocation_fact_id.as_str()
        || payload.tenant_id != fact.tenant_id
        || payload.revocation_epoch != fact.revocation_epoch
        || payload.revoked_subject_ref != fact.revoked_subject_ref
        || payload.revoked_capability_ref != fact.revoked_capability_ref
        || fact.revocation_epoch < 1
    {
        return Err(invalid_context_payload(
            "ContextRevocationFact",
            "row metadata differs from canonical revocation fact",
        ));
    }
    Ok(())
}

pub(crate) fn validate_context_request_row(
    request: &ContextRequestRow,
) -> Result<(), StorePortError> {
    let payload = parse_and_verify_context_payload(&request.canonical_json, "ContextRequest")?;
    let context_request: ContextRequest = serde_json::from_value(payload)
        .map_err(|error| invalid_context_payload("ContextRequest", error))?;
    let header = &context_request.header;
    if header.id.0 != request.request_id.as_str()
        || header.r#type != "ContextRequest"
        || header.content_digest.0 != request.request_digest
        || context_request.perspective.task != request.task_ref
    {
        return Err(invalid_context_payload(
            "ContextRequest",
            "row identity, type, digest, or task reference differs from canonical payload",
        ));
    }
    Ok(())
}

pub(crate) fn validate_context_view_row(
    connection: &Connection,
    view: &ContextViewRow,
) -> Result<(), StorePortError> {
    let payload = parse_and_verify_context_payload(&view.canonical_json, "ContextView")?;
    let context_view: ContextView = serde_json::from_value(payload)
        .map_err(|error| invalid_context_payload("ContextView", error))?;
    let header = &context_view.header;
    let request_reference = &context_view.request_ref;
    if header.id.0 != view.view_id.as_str()
        || header.r#type != "ContextView"
        || header.content_digest.0 != view.view_digest
        || request_reference.id.0 != view.request_id.as_str()
        || request_reference.kind != StrongReferenceKind::Strong
        || request_reference.object_version != 1
    {
        return Err(invalid_context_payload(
            "ContextView",
            "row identity, type, digest, or request strong reference differs from canonical payload",
        ));
    }
    let persisted_request_digest = connection
        .query_row(
            "SELECT request_digest FROM context_requests WHERE request_id=?1",
            (view.request_id.as_str(),),
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(unavailable("load ContextRequest for ContextView binding"))?;
    let persisted_request_digest =
        persisted_request_digest.ok_or_else(|| StorePortError::Conflict {
            detail: format!("ContextView {} names an unknown request", view.view_id),
        })?;
    if request_reference.content_digest.0 != persisted_request_digest {
        return Err(invalid_context_payload(
            "ContextView",
            "request strong-reference digest differs from the persisted ContextRequest",
        ));
    }
    Ok(())
}

pub(crate) fn validate_workspace_context_source_row(
    source: &WorkspaceContextSourceRow,
) -> Result<(), StorePortError> {
    let payload =
        parse_and_verify_context_payload(&source.canonical_json, "WorkspaceContextSource")?;
    let header: GovernedObjectHeader =
        serde_json::from_value(payload.get("header").cloned().ok_or_else(|| {
            invalid_context_payload("WorkspaceContextSource", "missing governed header")
        })?)
        .map_err(|error| invalid_context_payload("WorkspaceContextSource", error))?;
    if header.id.0 != source.source_id.as_str()
        || header.r#type != "WorkspaceContextSource"
        || header.content_digest.0 != source.source_digest
    {
        return Err(invalid_context_payload(
            "WorkspaceContextSource",
            "row identity, type, or digest differs from canonical payload",
        ));
    }
    let expected_metadata = [
        ("tenant_id", serde_json::json!(source.governance.tenant_id)),
        ("owner_ref", serde_json::json!(source.governance.owner_ref)),
        (
            "resource_scope",
            serde_json::json!(source.governance.resource_scope),
        ),
        (
            "conversation_ref",
            serde_json::json!(source.governance.conversation_ref),
        ),
        ("role", serde_json::json!(source.role)),
        ("trust_level", serde_json::json!(source.trust_level)),
        ("representation", serde_json::json!(source.representation)),
        ("provenance_ref", serde_json::json!(source.provenance_ref)),
        ("content_bytes", serde_json::json!(source.content_bytes)),
        ("content_tokens", serde_json::json!(source.content_tokens)),
    ];
    for (field, expected_value) in expected_metadata {
        if payload.get(field) != Some(&expected_value) {
            return Err(invalid_context_payload(
                "WorkspaceContextSource",
                format!("row {field} differs from canonical payload"),
            ));
        }
    }
    if source.governance.tenant_id.is_none()
        || source.governance.object_ref != source.source_id.as_str()
    {
        return Err(invalid_context_payload(
            "WorkspaceContextSource",
            "workspace source requires tenant governance and matching object reference",
        ));
    }
    Ok(())
}

pub(crate) struct WorkspaceContextSourceDatabaseRow {
    source_id: String,
    source_digest: String,
    tenant_id: String,
    owner_ref: String,
    resource_scope: String,
    conversation_ref: Option<String>,
    role: String,
    trust_level: String,
    representation: String,
    provenance_ref: String,
    content_bytes: i64,
    content_tokens: Option<i64>,
    canonical_json: String,
}

pub(crate) fn parse_workspace_context_source_row(
    database_row: WorkspaceContextSourceDatabaseRow,
) -> Result<WorkspaceContextSourceRow, rusqlite::Error> {
    let parse_enum_error = |error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
    };
    Ok(WorkspaceContextSourceRow {
        source_id: ObjectId::parse(&database_row.source_id).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        source_digest: database_row.source_digest,
        governance: ObjectGovernance {
            object_ref: database_row.source_id,
            tenant_id: Some(database_row.tenant_id),
            owner_ref: database_row.owner_ref,
            resource_scope: database_row.resource_scope,
            conversation_ref: database_row.conversation_ref,
        },
        role: serde_json::from_value(serde_json::Value::String(database_row.role))
            .map_err(parse_enum_error)?,
        trust_level: serde_json::from_value(serde_json::Value::String(database_row.trust_level))
            .map_err(parse_enum_error)?,
        representation: serde_json::from_value(serde_json::Value::String(
            database_row.representation,
        ))
        .map_err(parse_enum_error)?,
        provenance_ref: database_row.provenance_ref,
        content_bytes: database_row.content_bytes,
        content_tokens: database_row.content_tokens,
        canonical_json: database_row.canonical_json,
    })
}

impl ContextStore for SqliteAuthorityStore {
    fn append_context_request(&self, request: &ContextRequestRow) -> Result<(), StorePortError> {
        validate_context_request_row(request)?;
        let connection = self.lock()?;
        let result = connection.execute(
            "INSERT INTO context_requests (request_id, task_ref, request_digest, canonical_json) \
             VALUES (?1, ?2, ?3, ?4)",
            (
                request.request_id.as_str(),
                request.task_ref.as_str(),
                request.request_digest.as_str(),
                request.canonical_json.as_str(),
            ),
        );
        match result {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!("ContextRequest {} already persisted", request.request_id),
            }),
            Err(error) => Err(unavailable("insert ContextRequest")(error)),
        }
    }

    fn load_context_request(
        &self,
        request_id: &ObjectId,
    ) -> Result<Option<ContextRequestRow>, StorePortError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT task_ref, request_digest, canonical_json FROM context_requests WHERE request_id=?1",
                (request_id.as_str(),),
                |row| {
                    Ok(ContextRequestRow {
                        request_id: request_id.clone(),
                        task_ref: row.get(0)?,
                        request_digest: row.get(1)?,
                        canonical_json: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(unavailable("load ContextRequest"))
    }

    fn append_context_view(&self, view: &ContextViewRow) -> Result<(), StorePortError> {
        let connection = self.lock()?;
        validate_context_view_row(&connection, view)?;
        let result = connection.execute(
            "INSERT INTO context_views (view_id, request_id, view_digest, canonical_json) \
             VALUES (?1, ?2, ?3, ?4)",
            (
                view.view_id.as_str(),
                view.request_id.as_str(),
                view.view_digest.as_str(),
                view.canonical_json.as_str(),
            ),
        );
        match result {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!(
                    "ContextView {} is duplicate or names an unknown request",
                    view.view_id
                ),
            }),
            Err(error) => Err(unavailable("insert ContextView")(error)),
        }
    }

    fn load_context_view(
        &self,
        view_id: &ObjectId,
    ) -> Result<Option<ContextViewRow>, StorePortError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT request_id, view_digest, canonical_json FROM context_views WHERE view_id=?1",
                (view_id.as_str(),),
                |row| {
                    let request_id: String = row.get(0)?;
                    Ok(ContextViewRow {
                        view_id: view_id.clone(),
                        request_id: ObjectId::parse(&request_id).map_err(|error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        })?,
                        view_digest: row.get(1)?,
                        canonical_json: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(unavailable("load ContextView"))
    }

    fn append_workspace_context_source(
        &self,
        source: &WorkspaceContextSourceRow,
    ) -> Result<(), StorePortError> {
        validate_workspace_context_source_row(source)?;
        let connection = self.lock()?;
        let role = match source.role {
            cognitive_contracts::generated::context_view::LoadedContextItemRole::Control => "control",
            cognitive_contracts::generated::context_view::LoadedContextItemRole::AuthoritativeState => "authoritative_state",
            cognitive_contracts::generated::context_view::LoadedContextItemRole::Evidence => "evidence",
            cognitive_contracts::generated::context_view::LoadedContextItemRole::Working => "working",
            cognitive_contracts::generated::context_view::LoadedContextItemRole::UntrustedInput => "untrusted_input",
        };
        let trust_level = match source.trust_level {
            cognitive_contracts::generated::context_view::LoadedContextItemTrustLevel::Control => "control",
            cognitive_contracts::generated::context_view::LoadedContextItemTrustLevel::Authoritative => "authoritative",
            cognitive_contracts::generated::context_view::LoadedContextItemTrustLevel::Verified => "verified",
            cognitive_contracts::generated::context_view::LoadedContextItemTrustLevel::Untrusted => "untrusted",
        };
        let representation = match source.representation {
            cognitive_contracts::generated::context_view::LoadedContextItemRepresentation::Structured => "structured",
            cognitive_contracts::generated::context_view::LoadedContextItemRepresentation::Text => "text",
            cognitive_contracts::generated::context_view::LoadedContextItemRepresentation::BinaryRef => "binary_ref",
        };
        let result = connection.execute(
            "INSERT INTO workspace_context_sources (source_id, source_digest, tenant_id, owner_ref, resource_scope, conversation_ref, role, trust_level, representation, provenance_ref, content_bytes, content_tokens, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            (
                source.source_id.as_str(),
                source.source_digest.as_str(),
                source.governance.tenant_id.as_deref(),
                source.governance.owner_ref.as_str(),
                source.governance.resource_scope.as_str(),
                source.governance.conversation_ref.as_deref(),
                role,
                trust_level,
                representation,
                source.provenance_ref.as_str(),
                source.content_bytes,
                source.content_tokens,
                source.canonical_json.as_str(),
            ),
        );
        match result {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!(
                    "WorkspaceContextSource {} already persisted or violates metadata invariants",
                    source.source_id
                ),
            }),
            Err(error) => Err(unavailable("insert WorkspaceContextSource")(error)),
        }
    }

    fn query_context_candidate_metadata(
        &self,
        query: &ContextCandidateQuery,
    ) -> Result<Vec<ContextCandidateMetadata>, StorePortError> {
        let connection = self.lock()?;
        let escaped_prefix = query
            .resource_scope_prefix
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let mut statement = connection.prepare_cached("SELECT source_id, source_digest, tenant_id, owner_ref, resource_scope, conversation_ref, role, trust_level, representation, provenance_ref, content_bytes, content_tokens, canonical_json FROM workspace_context_sources WHERE tenant_id=?1 AND resource_scope LIKE ?2 ESCAPE '\\' AND ((?3 IS NULL AND conversation_ref IS NULL) OR conversation_ref=?3) ORDER BY source_id LIMIT ?4").map_err(unavailable("prepare Context metadata query"))?;
        let rows = statement
            .query_map(
                (
                    query.tenant_id.as_str(),
                    format!("{escaped_prefix}%"),
                    query.conversation_ref.as_deref(),
                    query.limit as i64,
                ),
                |row| {
                    let source =
                        parse_workspace_context_source_row(WorkspaceContextSourceDatabaseRow {
                            source_id: row.get(0)?,
                            source_digest: row.get(1)?,
                            tenant_id: row.get(2)?,
                            owner_ref: row.get(3)?,
                            resource_scope: row.get(4)?,
                            conversation_ref: row.get(5)?,
                            role: row.get(6)?,
                            trust_level: row.get(7)?,
                            representation: row.get(8)?,
                            provenance_ref: row.get(9)?,
                            content_bytes: row.get(10)?,
                            content_tokens: row.get(11)?,
                            canonical_json: row.get(12)?,
                        })?;
                    let payload: Value =
                        serde_json::from_str(&source.canonical_json).map_err(|error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                12,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        })?;
                    let header: GovernedObjectHeader = serde_json::from_value(
                        payload.get("header").cloned().ok_or_else(|| {
                            rusqlite::Error::InvalidColumnType(
                                12,
                                "canonical_json".to_owned(),
                                rusqlite::types::Type::Null,
                            )
                        })?,
                    )
                    .map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            12,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?;
                    let created_at = WallTimestamp::parse(&header.created_at).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            12,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?;
                    Ok(ContextCandidateMetadata {
                        source_id: source.source_id,
                        source_digest: source.source_digest,
                        created_at,
                        governance: source.governance,
                        role: source.role,
                        trust_level: source.trust_level,
                        representation: source.representation,
                        provenance_ref: source.provenance_ref,
                        content_bytes: source.content_bytes,
                        content_tokens: source.content_tokens,
                    })
                },
            )
            .map_err(unavailable("query Context metadata"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read Context metadata"))
    }

    fn load_workspace_context_source_body(
        &self,
        source_id: &ObjectId,
    ) -> Result<Option<WorkspaceContextSourceRow>, StorePortError> {
        let connection = self.lock()?;
        connection.query_row("SELECT source_id, source_digest, tenant_id, owner_ref, resource_scope, conversation_ref, role, trust_level, representation, provenance_ref, content_bytes, content_tokens, canonical_json FROM workspace_context_sources WHERE source_id=?1", [source_id.as_str()], |row| parse_workspace_context_source_row(WorkspaceContextSourceDatabaseRow {
            source_id: row.get(0)?,
            source_digest: row.get(1)?,
            tenant_id: row.get(2)?,
            owner_ref: row.get(3)?,
            resource_scope: row.get(4)?,
            conversation_ref: row.get(5)?,
            role: row.get(6)?,
            trust_level: row.get(7)?,
            representation: row.get(8)?,
            provenance_ref: row.get(9)?,
            content_bytes: row.get(10)?,
            content_tokens: row.get(11)?,
            canonical_json: row.get(12)?,
        })).optional().map_err(unavailable("load WorkspaceContextSource body"))
    }
}

impl SchedulerExecutionPolicyStore for SqliteAuthorityStore {
    fn append_scheduler_execution_policy(
        &self,
        policy: &SchedulerExecutionPolicyRow,
    ) -> Result<(), StorePortError> {
        if policy.task_ref.trim().is_empty() || policy.contract_epoch < 1 {
            return Err(StorePortError::Conflict {
                detail: "scheduler execution policy binding is invalid".to_owned(),
            });
        }
        let canonical_value: Value =
            serde_json::from_str(&policy.canonical_json).map_err(|_| StorePortError::Conflict {
                detail: "scheduler execution policy is not valid JSON".to_owned(),
            })?;
        if !canonical_value.is_object() {
            return Err(StorePortError::Conflict {
                detail: "scheduler execution policy must be a JSON object".to_owned(),
            });
        }
        let connection = self.lock()?;
        let result = connection.execute(
            "INSERT INTO scheduler_execution_policies \
             (task_ref, contract_epoch, context_request_id, canonical_json) \
             VALUES (?1, ?2, ?3, ?4)",
            (
                policy.task_ref.as_str(),
                policy.contract_epoch,
                policy.context_request_id.as_str(),
                policy.canonical_json.as_str(),
            ),
        );
        match result {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => {
                let existing = connection
                    .query_row(
                        "SELECT context_request_id, canonical_json \
                         FROM scheduler_execution_policies \
                         WHERE task_ref=?1 AND contract_epoch=?2",
                        (policy.task_ref.as_str(), policy.contract_epoch),
                        |row| {
                            let context_request_id: String = row.get(0)?;
                            let canonical_json: String = row.get(1)?;
                            Ok((context_request_id, canonical_json))
                        },
                    )
                    .optional()
                    .map_err(unavailable("load duplicate scheduler execution policy"))?;
                if existing
                    .as_ref()
                    .is_some_and(|(context_request_id, canonical_json)| {
                        context_request_id == policy.context_request_id.as_str()
                            && canonical_json == policy.canonical_json.as_str()
                    })
                {
                    Ok(())
                } else {
                    Err(StorePortError::Conflict {
                        detail: format!(
                            "scheduler execution policy already exists with different content for {} at epoch {}",
                            policy.task_ref, policy.contract_epoch
                        ),
                    })
                }
            }
            Err(error) => Err(unavailable("insert scheduler execution policy")(error)),
        }
    }

    fn load_scheduler_execution_policy(
        &self,
        task_ref: &str,
        contract_epoch: i64,
    ) -> Result<Option<SchedulerExecutionPolicyRow>, StorePortError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT context_request_id, canonical_json \
                 FROM scheduler_execution_policies \
                 WHERE task_ref=?1 AND contract_epoch=?2",
                (task_ref, contract_epoch),
                |row| {
                    let context_request_id_text: String = row.get(0)?;
                    let context_request_id =
                        ObjectId::parse(&context_request_id_text).map_err(|error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        })?;
                    Ok(SchedulerExecutionPolicyRow {
                        task_ref: task_ref.to_owned(),
                        contract_epoch,
                        context_request_id,
                        canonical_json: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(unavailable("load scheduler execution policy"))
    }
}

impl ContextAuthorizationFactStore for SqliteAuthorityStore {
    fn append_context_authorization_facts(
        &self,
        facts: &ContextAuthorizationFactsRow,
    ) -> Result<(), StorePortError> {
        validate_context_authorization_facts_row(facts)?;
        let connection = self.lock()?;
        let result = connection.execute(
            "INSERT INTO context_authorization_fact_sets (fact_set_id, subject_ref, tenant_id, capability_set_version, issued_revocation_epoch, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                facts.fact_set_id.as_str(),
                facts.subject_ref.as_str(),
                facts.tenant_id.as_str(),
                facts.capability_set_version,
                facts.issued_revocation_epoch,
                facts.canonical_json.as_str(),
            ),
        );
        match result {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!(
                    "Context authorization facts {} already persisted",
                    facts.fact_set_id
                ),
            }),
            Err(error) => Err(unavailable("insert Context authorization facts")(error)),
        }
    }

    fn append_context_revocation_fact(
        &self,
        fact: &ContextRevocationFactRow,
    ) -> Result<(), StorePortError> {
        validate_context_revocation_fact_row(fact)?;
        let connection = self.lock()?;
        let result = connection.execute(
            "INSERT INTO context_revocation_facts (revocation_fact_id, tenant_id, revocation_epoch, revoked_subject_ref, revoked_capability_ref, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                fact.revocation_fact_id.as_str(),
                fact.tenant_id.as_str(),
                fact.revocation_epoch,
                fact.revoked_subject_ref.as_deref(),
                fact.revoked_capability_ref.as_deref(),
                fact.canonical_json.as_str(),
            ),
        );
        match result {
            Ok(_) => Ok(()),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: format!(
                    "Context revocation fact {} is duplicate or conflicts with tenant epoch",
                    fact.revocation_fact_id
                ),
            }),
            Err(error) => Err(unavailable("insert Context revocation fact")(error)),
        }
    }

    fn load_latest_context_authorization_facts(
        &self,
        subject_ref: &str,
        tenant_id: &str,
    ) -> Result<Option<ContextAuthorizationFactsRow>, StorePortError> {
        let connection = self.lock()?;
        let canonical_json = connection.query_row(
            "SELECT canonical_json FROM context_authorization_fact_sets WHERE subject_ref=?1 AND tenant_id=?2 ORDER BY fact_sequence DESC LIMIT 1",
            (subject_ref, tenant_id),
            |row| row.get::<_, String>(0),
        ).optional().map_err(unavailable("load latest Context authorization facts"))?;
        canonical_json
            .map(|canonical_json| {
                let payload = parse_context_authorization_facts(&canonical_json)?;
                let fact_set_id = ObjectId::parse(&payload.fact_set_id)
                    .map_err(|error| invalid_context_payload("ContextAuthorizationFacts", error))?;
                let row = ContextAuthorizationFactsRow {
                    fact_set_id,
                    subject_ref: payload.subject_ref,
                    tenant_id: payload.tenant_id,
                    principal: payload.principal,
                    actor_chain: payload.actor_chain,
                    membership: payload.membership,
                    capability_links: payload.capability_links,
                    explicit_denies: payload.explicit_denies,
                    capability_set_version: payload.capability_set_version,
                    issued_revocation_epoch: payload.issued_revocation_epoch,
                    canonical_json,
                };
                validate_context_authorization_facts_row(&row)?;
                Ok(row)
            })
            .transpose()
    }

    fn load_current_context_revocation_epoch(
        &self,
        tenant_id: &str,
    ) -> Result<Option<i64>, StorePortError> {
        let connection = self.lock()?;
        connection.query_row(
            "SELECT revocation_epoch FROM context_revocation_facts WHERE tenant_id=?1 ORDER BY revocation_epoch DESC LIMIT 1",
            [tenant_id],
            |row| row.get(0),
        ).optional().map_err(unavailable("load current Context revocation epoch"))
    }
}
