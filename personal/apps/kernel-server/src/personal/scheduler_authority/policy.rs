#![allow(dead_code, unused_imports)]

use cognitive_contracts::{
    canonical,
    generated::governed_object_header::GovernedObjectHeaderSensitivity,
    generated::worker_iteration_authorization::WorkerIterationAuthorization,
    generated::{
        context_request::ContextRequest,
        context_view::{
            ContextView, ContextViewPinnedVersionsValue, ItemCost, LoadedContextItem,
            LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
            LossDeclaration, RejectedCandidate as PersistedRejectedCandidate, ResolutionCost,
        },
        object_reference::StrongReferenceKind,
        operation_candidate_proposal::OperationCandidateProposal,
        task_contract::TaskContract,
    },
};
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, RecordId, UriRef, Version, WallTimestamp,
};
use cognitive_kernel::authz::ObjectGovernance;
use cognitive_kernel::budget::BudgetCharge;
use cognitive_kernel::candidate_admission::{
    CandidateAdmissionFacts, CandidateAdmissionIdentities, CandidateAdmissionInputs,
    compose_candidate_admission,
};
use cognitive_kernel::context::{
    ArrivalOrderRanker, CandidateObject, ContextBudget, LossEntry, RejectedCandidate, RenderSpec,
    RequiredItem, ResolutionRequest, ResolvedContextView, resolve,
};
use cognitive_kernel::effects::{WriterLease, admit_operation};
use cognitive_kernel::engine::CommittedTransition;
use cognitive_kernel::harness::LoopDriver;
use cognitive_kernel::intent_chain::{
    GovernanceSeed, compose_governed_header, seal_governed_object_content_digest,
    strong_reference_to,
};
use cognitive_kernel::{
    ContextCacheEntry, ContextCacheKey, ContextCacheLookup, ContextSourceDigest, DerivedCacheKind,
    GovernedContextCache,
};
use cognitive_kernel::{
    authz::{AccessRequest, authorize},
    ports::{
        AuthorityStore, BoundContinuationAuthorizationConsumption,
        BoundWorkerAuthorizationConsumption, CandidateAdmissionReceipt, Clock,
        ContextAuthorizationFactStore, ContextCandidateQuery, ContextRequestRow, ContextStore,
        ContextViewRow, ContinuationAuthorityStore, ContinuationAuthorizationConsumptionRow,
        ContinuationAuthorizationRow, HarnessStore, IdGenerator, IntentChainStore, ProtocolStore,
        SchedulerExecutionPolicyRow, SchedulerExecutionPolicyStore, SchedulerLeaseBinding,
        TaskBinding, WorkerAuthorizationStore, WorkerIterationAuthorizationConsumptionRow,
        WorkerIterationAuthorizationRow,
    },
    resolve_persisted_native_descriptor,
};
use cognitive_runtime::{
    SchedulerCeilingDispatch, SchedulerCeilingDispatchError, SchedulerCeilingFacts,
    SchedulerDispatch, SchedulerService, SchedulerServiceError,
};
use cognitive_store::{
    SqliteAuthorityStore, SystemClock, UuidV7Generator,
    scheduler::{SchedulerRepository, SchedulerRepositoryError, SchedulerState, SchedulerWorkKey},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

use crate::personal::pi_runtime::{
    PrivatePiCandidateProcess, PrivatePiCandidateRequest, PrivatePiCandidateResponse,
    load_pi_config,
};

use super::*;

#[derive(Deserialize)]
struct TaskContractVersionEnvelope {
    header: TaskContractVersionHeader,
}

#[derive(Deserialize)]
struct TaskContractVersionHeader {
    schema_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SchedulerExecutionPolicyDocument {
    schema_version: i64,
    task_ref: String,
    contract_epoch: i64,
    context: SchedulerContextPolicy,
    admission: SchedulerAdmissionPolicy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SchedulerAdmissionPolicy {
    candidate_id: String,
    authorization_subject_ref: String,
    authorization_purpose: String,
    budget_charge: BTreeMap<String, i64>,
    governance: SchedulerGovernancePolicy,
    actor_ref: String,
    authority_ref: String,
    correlation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SchedulerGovernancePolicy {
    owner: cognitive_contracts::generated::object_reference::StrongReference,
    authority: cognitive_contracts::generated::object_reference::StrongReference,
    resource_scope: cognitive_contracts::generated::object_reference::StrongReference,
    tenant_id: Option<String>,
    created_by: String,
    sensitivity: GovernedObjectHeaderSensitivity,
    purpose_constraints: Vec<String>,
    retention_policy: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SchedulerContextPolicy {
    request_id: String,
    authorization_subject_ref: String,
    tenant_id: String,
    resource_scope_prefix: String,
    conversation_ref: Option<String>,
    source_limit: usize,
}

/// Parse only a current execution-bound contract before reading its bindings.
///
/// The explicit version check preserves old contract rows for audit while
/// preventing their deserialization from becoming a scheduler admission path.
pub(crate) fn parse_execution_bound_contract(
    canonical_json: &str,
) -> Result<TaskContract, SchedulerAuthorityError> {
    let version_envelope: TaskContractVersionEnvelope = serde_json::from_str(canonical_json)
        .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    let supports_execution_bindings = matches!(
        version_envelope.header.schema_version.as_str(),
        TASK_CONTRACT_EXECUTION_SCHEMA_V03 | TASK_CONTRACT_EXECUTION_SCHEMA_V04
    );
    if !supports_execution_bindings {
        return Err(SchedulerAuthorityError::LegacyContract(
            version_envelope.header.schema_version,
        ));
    }

    let contract: TaskContract = serde_json::from_str(canonical_json)
        .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    if contract.worker_authorization_root_id.is_none() {
        return Err(SchedulerAuthorityError::MalformedContract(
            "execution-bound contract has no worker authorization namespace".to_owned(),
        ));
    }
    Ok(contract)
}

/// Reload the private execution policy required by a Context-bound v0.4
/// contract. This is deliberately before WIA lookup: policy absence or a
/// request mismatch must stop the pre-admission path before Pi can be invoked
/// or existing worker authority can be consumed.
pub(crate) fn load_context_execution_policy<S>(
    store: &S,
    task_binding: &TaskBinding,
) -> Result<Option<SchedulerExecutionPolicyRow>, SchedulerAuthorityError>
where
    S: IntentChainStore + SchedulerExecutionPolicyStore,
{
    let contract_row = store
        .load_task_contract(&task_binding.task_ref, task_binding.contract_epoch)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::MissingContract(task_binding.task_ref.clone()))?;
    let version_envelope: TaskContractVersionEnvelope =
        serde_json::from_str(&contract_row.canonical_json)
            .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    let contract = parse_execution_bound_contract(&contract_row.canonical_json)?;
    if version_envelope.header.schema_version != TASK_CONTRACT_EXECUTION_SCHEMA_V04 {
        return Ok(None);
    }
    let context_request_reference = contract.context_request_ref.ok_or_else(|| {
        SchedulerAuthorityError::ContextRequestUnavailable(
            "v0.4 TaskContract has no ContextRequest binding".to_owned(),
        )
    })?;
    if context_request_reference.kind != StrongReferenceKind::Strong
        || context_request_reference.object_version != 1
    {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "v0.4 TaskContract ContextRequest reference is not strong and versioned".to_owned(),
        ));
    }
    let policy = store
        .load_scheduler_execution_policy(&task_binding.task_ref, task_binding.contract_epoch)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextRequestUnavailable(format!(
                "no scheduler execution policy exists for {} at epoch {}",
                task_binding.task_ref, task_binding.contract_epoch
            ))
        })?;
    if context_request_reference.id.0.as_str() != policy.context_request_id.as_str() {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "scheduler execution policy ContextRequest differs from TaskContract binding"
                .to_owned(),
        ));
    }
    let policy_document: SchedulerExecutionPolicyDocument =
        serde_json::from_str(&policy.canonical_json).map_err(|error| {
            SchedulerAuthorityError::ContextRequestUnavailable(format!(
                "scheduler execution policy is malformed: {error}"
            ))
        })?;
    if policy_document.schema_version != 1
        || policy_document.task_ref != task_binding.task_ref
        || policy_document.contract_epoch != task_binding.contract_epoch
        || policy_document.context.request_id != policy.context_request_id.as_str()
        || policy_document
            .context
            .authorization_subject_ref
            .trim()
            .is_empty()
        || policy_document.context.tenant_id.trim().is_empty()
        || policy_document
            .context
            .resource_scope_prefix
            .trim()
            .is_empty()
        || policy_document.context.source_limit == 0
    {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "scheduler execution policy fields do not match its durable binding".to_owned(),
        ));
    }
    Ok(Some(policy))
}

/// Convert the validated daemon policy into the deterministic Context resolver
/// command. This function only assembles authority inputs; it does not query
/// Context bodies and cannot grant Pi or worker authority.
pub(crate) fn context_resolution_command_from_policy(
    policy: &SchedulerExecutionPolicyRow,
    decided_at: WallTimestamp,
) -> Result<ContextResolutionCommand, SchedulerAuthorityError> {
    let document: SchedulerExecutionPolicyDocument = serde_json::from_str(&policy.canonical_json)
        .map_err(|error| {
        SchedulerAuthorityError::ContextRequestUnavailable(format!(
            "scheduler execution policy is malformed: {error}"
        ))
    })?;
    let request_id = ObjectId::parse(&document.context.request_id).map_err(|_| {
        SchedulerAuthorityError::ContextRequestUnavailable(
            "scheduler execution policy ContextRequest identity is malformed".to_owned(),
        )
    })?;
    if request_id != policy.context_request_id {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "scheduler execution policy command identity differs from durable policy row"
                .to_owned(),
        ));
    }
    Ok(ContextResolutionCommand {
        task_ref: document.task_ref,
        request_id,
        authorization_subject_ref: document.context.authorization_subject_ref,
        tenant_id: document.context.tenant_id,
        resource_scope_prefix: document.context.resource_scope_prefix,
        conversation_ref: document.context.conversation_ref,
        source_limit: document.context.source_limit,
        decided_at,
    })
}

/// Assemble the daemon-only admission command from the same validated policy
/// document. No candidate field is read from this document; the candidate ID
/// is supplied by the daemon after the Pi proposal has passed its own strict
/// candidate validation.
pub(crate) fn candidate_admission_command_from_policy(
    policy: &SchedulerExecutionPolicyRow,
) -> Result<DaemonCandidateAdmissionCommand, SchedulerAuthorityError> {
    let document: SchedulerExecutionPolicyDocument = serde_json::from_str(&policy.canonical_json)
        .map_err(|error| {
        SchedulerAuthorityError::CandidateAdmissionComposition(format!(
            "scheduler execution policy is malformed: {error}"
        ))
    })?;
    if document.schema_version != 1
        || document.task_ref != policy.task_ref
        || document.contract_epoch != policy.contract_epoch
        || document.context.request_id != policy.context_request_id.as_str()
        || document.context.authorization_subject_ref.trim().is_empty()
        || document.context.tenant_id.trim().is_empty()
        || document.context.resource_scope_prefix.trim().is_empty()
        || document.context.source_limit == 0
        || document
            .admission
            .authorization_subject_ref
            .trim()
            .is_empty()
        || document.admission.authorization_purpose.trim().is_empty()
    {
        return Err(SchedulerAuthorityError::CandidateAdmissionComposition(
            "scheduler execution policy fields do not match its durable binding".to_owned(),
        ));
    }
    let admission = document.admission;
    let candidate_id = ObjectId::parse(&admission.candidate_id).map_err(|_| {
        SchedulerAuthorityError::CandidateAdmissionComposition(
            "scheduler execution policy candidate identity is malformed".to_owned(),
        )
    })?;
    let budget_charge = BudgetCharge::new(admission.budget_charge).map_err(|error| {
        SchedulerAuthorityError::CandidateAdmissionComposition(format!(
            "scheduler execution policy budget charge is invalid: {error}"
        ))
    })?;
    let actor_ref = UriRef::parse(&admission.actor_ref).map_err(|error| {
        SchedulerAuthorityError::CandidateAdmissionComposition(format!(
            "scheduler execution policy actor reference is invalid: {error}"
        ))
    })?;
    let authority_ref = UriRef::parse(&admission.authority_ref).map_err(|error| {
        SchedulerAuthorityError::CandidateAdmissionComposition(format!(
            "scheduler execution policy authority reference is invalid: {error}"
        ))
    })?;
    let correlation_id = UriRef::parse(&admission.correlation_id).map_err(|error| {
        SchedulerAuthorityError::CandidateAdmissionComposition(format!(
            "scheduler execution policy correlation reference is invalid: {error}"
        ))
    })?;
    if admission.authorization_subject_ref.trim().is_empty()
        || admission.authorization_purpose.trim().is_empty()
        || admission.governance.created_by.trim().is_empty()
        || admission.governance.purpose_constraints.is_empty()
        || admission.governance.retention_policy.trim().is_empty()
    {
        return Err(SchedulerAuthorityError::CandidateAdmissionComposition(
            "scheduler execution policy admission fields are incomplete".to_owned(),
        ));
    }
    Ok(DaemonCandidateAdmissionCommand {
        candidate_id,
        authorization_subject_ref: admission.authorization_subject_ref,
        authorization_purpose: admission.authorization_purpose,
        budget_charge,
        governance: GovernanceSeed {
            owner: admission.governance.owner,
            authority: admission.governance.authority,
            resource_scope: admission.governance.resource_scope,
            tenant_id: admission.governance.tenant_id,
            created_by: admission.governance.created_by,
            sensitivity: admission.governance.sensitivity,
            purpose_constraints: admission.governance.purpose_constraints,
            retention_policy: admission.governance.retention_policy,
        },
        actor_ref,
        authority_ref,
        correlation_id,
    })
}
