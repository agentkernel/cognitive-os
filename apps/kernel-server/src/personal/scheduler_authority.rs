//! Daemon-only durable scheduler authority reads (P2-T03).
//!
//! This module owns daemon-private scheduler authority reads and one bounded
//! worker-attempt composition boundary. It reloads immutable TaskContract and
//! Effect identities before every durable decision; it never accepts a Task.

#![allow(dead_code, clippy::items_after_test_module)] // Activated only after the fenced quiescence protocol exists.

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
    ArrivalOrderRanker, CandidateObject, ContextBudget, RenderSpec, RequiredItem,
    ResolutionRequest, ResolvedContextView, resolve,
};
use cognitive_kernel::effects::{WriterLease, admit_operation};
use cognitive_kernel::engine::CommittedTransition;
use cognitive_kernel::harness::LoopDriver;
use cognitive_kernel::intent_chain::{
    GovernanceSeed, compose_governed_header, seal_governed_object_content_digest,
    strong_reference_to,
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

use super::pi_runtime::{
    PrivatePiCandidateProcess, PrivatePiCandidateRequest, PrivatePiCandidateResponse,
    load_pi_config,
};

const TASK_CONTRACT_EXECUTION_SCHEMA_V03: &str = "cognitiveos.task-contract/0.3";
const TASK_CONTRACT_EXECUTION_SCHEMA_V04: &str = "cognitiveos.task-contract/0.4";
const OPERATION_CANDIDATE_SCHEMA_VERSION: &str = "cognitiveos.operation-candidate-proposal/0.1";
const DAEMON_DESCRIPTOR_REFERENCE_DIGEST_DOMAIN: &str =
    "cognitiveos.personal.daemon-descriptor-reference/0.1";

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

/// Exact identities fixed by an immutable task contract epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchedulerAuthorityBinding {
    pub task_ref: String,
    pub contract_epoch: i64,
    pub action_fingerprint: String,
}

/// Durable authority inputs required to decide one scheduler admission.
///
/// These facts are reloaded from the current immutable TaskContract and the
/// authority store for every attempt. They are never taken from the worker or
/// a prior scheduler projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchedulerAuthoritySnapshot {
    pub ceiling_facts: SchedulerCeilingFacts,
    pub loop_object_id: ObjectId,
    pub budget_id: BudgetId,
}

/// Durable facts accepted by the daemon-only candidate-admission preflight.
/// This result contains no worker output and grants no dispatch permission;
/// it is only the deterministic input set for constructing an atomic bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CandidateAdmissionPreflight {
    pub task_ref: String,
    pub contract_epoch: i64,
    pub loop_object_id: ObjectId,
    pub budget_id: BudgetId,
    pub expected_budget_version: Version,
    pub next_budget_state_canonical_json: String,
    pub expected_loop_version: Version,
    pub next_iteration: i64,
}

/// Daemon-owned bindings for one Context resolution before any candidate is
/// requested from Pi. The scheduler obtains these values from the immutable
/// TaskContract and its local owner session; they are never producer input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextResolutionCommand {
    pub task_ref: String,
    pub request_id: ObjectId,
    pub authorization_subject_ref: String,
    pub tenant_id: String,
    pub resource_scope_prefix: String,
    pub conversation_ref: Option<String>,
    pub source_limit: usize,
    pub decided_at: WallTimestamp,
}

/// The only non-authority fields a private Pi producer may propose. The
/// daemon validates them against current durable facts, supplies every
/// governed reference/header, and seals the persisted candidate itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UntrustedPiCandidate {
    pub tool_ref: String,
    pub action: String,
    pub target: String,
    pub parameters_digest: String,
    pub expected_state_version: i64,
    pub operation_descriptor_id: ObjectId,
}

/// Private Pi boundary used before candidate admission. Implementations may
/// transport one bounded request to a pinned daemon-supervised Pi child, but
/// they cannot return progress, evidence, receipts, WIA, Effect state, or
/// Task lifecycle data.
pub(crate) trait PrivatePiCandidateProposer {
    fn propose_candidate(
        &self,
        resolved_context: &ResolvedContextView,
        task_ref: &str,
        contract_epoch: i64,
    ) -> Result<UntrustedPiCandidate, String>;
}

/// Adapter from the configured daemon-supervised Pi process to the scheduler
/// proposer port. Parsing remains transport-local; the scheduler receives no
/// process output other than the bounded candidate fields.
pub(crate) struct ConfiguredPrivatePiCandidateProposer {
    process: PrivatePiCandidateProcess,
}

impl ConfiguredPrivatePiCandidateProposer {
    pub(crate) fn new(process: PrivatePiCandidateProcess) -> Self {
        Self { process }
    }
}

impl PrivatePiCandidateProposer for ConfiguredPrivatePiCandidateProposer {
    fn propose_candidate(
        &self,
        resolved_context: &ResolvedContextView,
        task_ref: &str,
        contract_epoch: i64,
    ) -> Result<UntrustedPiCandidate, String> {
        let rendered_context = String::from_utf8(resolved_context.render.bytes.clone())
            .map_err(|_| "resolved Context rendering is not UTF-8".to_owned())?;
        let response: PrivatePiCandidateResponse =
            self.process.propose(&PrivatePiCandidateRequest {
                protocol: "cognitiveos.private-candidate/1",
                task_ref: task_ref.to_owned(),
                contract_epoch,
                rendered_context,
            })?;
        let operation_descriptor_id = ObjectId::parse(&response.operation_descriptor_id)
            .map_err(|_| "private Pi descriptor reference is malformed".to_owned())?;
        Ok(UntrustedPiCandidate {
            tool_ref: response.tool_ref,
            action: response.action,
            target: response.target,
            parameters_digest: response.parameters_digest,
            expected_state_version: response.expected_state_version,
            operation_descriptor_id,
        })
    }
}

/// Daemon-owned identity and governance inputs required to create one atomic
/// candidate-admission bundle. This is deliberately not an API request type:
/// the daemon resolves all values from its own configuration and durable
/// governance state before calling [`admit_candidate_atomically`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DaemonCandidateAdmissionCommand {
    pub candidate_id: ObjectId,
    pub authorization_subject_ref: String,
    pub authorization_purpose: String,
    pub budget_charge: BudgetCharge,
    pub governance: GovernanceSeed,
    pub actor_ref: UriRef,
    pub authority_ref: UriRef,
    pub correlation_id: UriRef,
}

/// Durable facts made available to a worker after its daemon-recorded WIA
/// handoff. This is not execution success, progress, evidence, verification,
/// Task acceptance, or Task completion.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WorkerAuthorizationHandoff {
    pub authorization: WorkerIterationAuthorizationRow,
    pub worker_attempt_id: ObjectId,
    /// `None` is legacy handoff evidence and cannot release scheduler work.
    pub scheduler_lease: Option<SchedulerLeaseBinding>,
}

/// A restart-safe worker attempt reconstructed solely from daemon-recorded
/// handoff evidence and the authoritative Effect lifecycle state. It is not
/// a worker result and grants neither progress nor Task completion.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RecoveredWorkerAttempt {
    pub handoff: WorkerAuthorizationHandoff,
    pub effect_closure: SchedulerEffectClosure,
}

/// Immutable scheduler identity reconstructed from durable work during every
/// daemon tick. The scheduler table intentionally stores only task lifecycle
/// and fencing fields; binding identity remains anchored in Intent protocol
/// rows and is never copied from a worker-local queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedSchedulerWork {
    pub authority_binding: SchedulerAuthorityBinding,
    pub task_binding: TaskBinding,
}

/// One daemon-owned scheduler admission result.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SchedulerDispatchAdmission {
    Stopped(CommittedTransition),
    Leased(SchedulerDispatch),
}

/// The only scheduler-facing result a daemon worker may accept from Effect
/// processing. The worker callback must derive either state from the durable
/// Effect protocol; an external receipt is not a closed Effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchedulerEffectClosure {
    Closed,
    PendingReconciliation,
}

/// The single durable Effect resolved for one scheduler TaskContract epoch.
///
/// A later worker integration must use this object identity to derive an
/// outcome from the authority store; it must not substitute a receipt or
/// process-local result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchedulerEffectResolution {
    pub effect_object_id: ObjectId,
    pub closure: SchedulerEffectClosure,
}

/// Daemon-owned outcome after scheduler admission reaches the Effect boundary.
///
/// This remains distinct from Task acceptance. A closed Effect only permits a
/// later fenced scheduler release; independent verification still decides
/// whether a Task may complete.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SchedulerWorkerAttempt {
    Stopped(CommittedTransition),
    /// A distinct verified continuation authority atomically entered the
    /// harness. This is neither Effect completion nor Task acceptance.
    ContinuationStarted(CommittedTransition),
    EffectClosed(SchedulerDispatch),
    AwaitingReconciliation(SchedulerDispatch),
}

/// Fail-closed authority-read failures before scheduler lease acquisition.
#[derive(Debug, Error)]
pub(crate) enum SchedulerAuthorityError {
    #[error("scheduler task reference must not be empty")]
    EmptyTaskReference,
    #[error("scheduler action fingerprint must not be empty")]
    EmptyActionFingerprint,
    #[error("scheduler authority store failed: {0}")]
    Store(String),
    #[error("scheduler task has no current contract: {0}")]
    MissingContract(String),
    #[error("scheduler contract is not execution-bound: {0}")]
    LegacyContract(String),
    #[error("scheduler contract is malformed: {0}")]
    MalformedContract(String),
    #[error("scheduler bound loop is unavailable or not dispatchable: {0}")]
    LoopUnavailable(String),
    #[error("scheduler bound budget is unavailable or inconsistent: {0}")]
    BudgetUnavailable(String),
    #[error("scheduler task contract epoch must be positive: {0}")]
    InvalidContractEpoch(i64),
    #[error("scheduler candidate is unavailable or inconsistent: {0}")]
    CandidateUnavailable(String),
    #[error("scheduler candidate tool is not allowed by its TaskContract: {0}")]
    CandidateToolForbidden(String),
    #[error("scheduler candidate descriptor is unavailable or unsafe: {0}")]
    CandidateDescriptorUnavailable(String),
    #[error("scheduler candidate has no current daemon authorization: {0}")]
    CandidateAuthorizationUnavailable(String),
    #[error("scheduler candidate admission composition failed: {0}")]
    CandidateAdmissionComposition(String),
    #[error("scheduler Context request is unavailable or inconsistent: {0}")]
    ContextRequestUnavailable(String),
    #[error("scheduler Context authorization facts are unavailable: {0}")]
    ContextAuthorizationUnavailable(String),
    #[error("scheduler Context body is unavailable or inconsistent: {0}")]
    ContextBodyUnavailable(String),
    #[error("scheduler Context resolution failed: {0}")]
    ContextResolution(String),
    #[error("scheduler private Pi candidate proposal failed: {0}")]
    PrivatePiProposal(String),
    #[error("scheduler continuation authority is unavailable or inconsistent: {0}")]
    ContinuationUnavailable(String),
    #[error(
        "scheduler work binding is stale: {task_ref} requested epoch {requested_epoch}, current epoch {current_epoch}"
    )]
    StaleContractEpoch {
        task_ref: String,
        requested_epoch: i64,
        current_epoch: i64,
    },
    #[error(
        "scheduler task contract epoch has no durable Effect binding: {task_ref} at {contract_epoch}"
    )]
    MissingEffectBinding {
        task_ref: String,
        contract_epoch: i64,
    },
    #[error(
        "scheduler task contract epoch has ambiguous durable Effect bindings: {task_ref} at {contract_epoch}"
    )]
    AmbiguousEffectBindings {
        task_ref: String,
        contract_epoch: i64,
    },
    #[error("scheduler durable Intent binding is inconsistent: {0}")]
    InconsistentEffectBinding(String),
    #[error("scheduler durable Effect is unavailable: {0}")]
    MissingEffect(String),
    #[error("scheduler durable Effect state is unsupported: {0}")]
    UnsupportedEffectState(String),
    #[error("scheduler dispatch does not match the resolved TaskContract binding: {0}")]
    DispatchBindingMismatch(String),
    #[error("scheduler lease release time is invalid: {0}")]
    InvalidReleaseTime(String),
    #[error(transparent)]
    Harness(#[from] cognitive_kernel::effects::EffectError),
    #[error(transparent)]
    Repository(#[from] SchedulerRepositoryError),
    #[error(transparent)]
    Scheduler(#[from] SchedulerServiceError),
    #[error(transparent)]
    CeilingStop(#[from] SchedulerCeilingDispatchError),
}

/// Parse only a current execution-bound contract before reading its bindings.
///
/// The explicit version check preserves old contract rows for audit while
/// preventing their deserialization from becoming a scheduler admission path.
fn parse_execution_bound_contract(
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
fn load_context_execution_policy<S>(
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
fn context_resolution_command_from_policy(
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
fn candidate_admission_command_from_policy(
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

/// Reconstruct a Context authorization snapshot immediately before a body
/// access. Re-reading both fact material and revocation currency prevents an
/// earlier metadata discovery from authorizing a later body read with stale
/// policy or capability facts.
fn load_current_context_authorization_snapshot<S>(
    store: &S,
    command: &ContextResolutionCommand,
) -> Result<cognitive_kernel::authz::AuthzSnapshot, SchedulerAuthorityError>
where
    S: ContextAuthorizationFactStore,
{
    let authorization_facts = store
        .load_latest_context_authorization_facts(
            &command.authorization_subject_ref,
            &command.tenant_id,
        )
        .map_err(|error| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(error.to_string())
        })?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(
                "no durable authorization facts for Context body read".to_owned(),
            )
        })?;
    let current_revocation_epoch = store
        .load_current_context_revocation_epoch(&command.tenant_id)
        .map_err(|error| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(error.to_string())
        })?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(
                "no durable Context revocation epoch".to_owned(),
            )
        })?;
    authorization_facts
        .reconstruct_snapshot(current_revocation_epoch, command.decided_at.clone())
        .map_err(|error| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(error.to_string())
        })
}

/// Build the daemon-owned System and Task fragments that every task-bound
/// resolution needs. These fragments are derived solely from the immutable
/// ContextRequest and TaskContract; Pi and workspace sources cannot supply or
/// modify them. They deliberately use the current capability scope so the
/// normal resolver revalidates their access alongside every other body.
fn build_required_task_fragments(
    authorization_snapshot: &cognitive_kernel::authz::AuthzSnapshot,
    command: &ContextResolutionCommand,
    request_row: &ContextRequestRow,
    context_request: &ContextRequest,
    contract_row: &cognitive_kernel::ports::TaskContractRow,
    contract: &TaskContract,
) -> Result<Vec<CandidateObject>, SchedulerAuthorityError> {
    let resource_scope = authorization_snapshot
        .capability_links
        .first()
        .map(|capability| capability.resource.clone())
        .filter(|scope| !scope.trim().is_empty())
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(
                "Context Builder requires a current capability resource scope".to_owned(),
            )
        })?;
    let fragment_governance = ObjectGovernance {
        object_ref: request_row.request_id.to_string(),
        tenant_id: Some(command.tenant_id.clone()),
        owner_ref: command.authorization_subject_ref.clone(),
        resource_scope,
        conversation_ref: command.conversation_ref.clone(),
    };
    let system_body = json!({
        "fragment": "system",
        "task_ref": command.task_ref,
        "purpose": context_request.purpose,
        "context_budget": context_request.budget,
        "authority": "daemon_observational_only",
    });
    let task_body = json!({
        "fragment": "task",
        "task_ref": contract.task_ref,
        "contract_epoch": contract.contract_epoch,
        "objective": contract.objective,
        "max_iterations": contract.max_iterations,
        "max_retries": contract.max_retries,
    });
    let candidate_cost = |body: &Value| {
        canonical::canonical_bytes_of_value(body)
            .map(|bytes| (bytes.len() as i64, (bytes.len() as i64 + 3) / 4))
            .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))
    };
    let (system_bytes, system_tokens) = candidate_cost(&system_body)?;
    let (task_bytes, task_tokens) = candidate_cost(&task_body)?;
    Ok(vec![
        CandidateObject {
            object_ref: request_row.request_id.to_string(),
            object_version: 1,
            content_digest: context_request.header.content_digest.0.clone(),
            governance: fragment_governance.clone(),
            role: LoadedContextItemRole::Control,
            trust_level: LoadedContextItemTrustLevel::Verified,
            representation: LoadedContextItemRepresentation::Structured,
            body: system_body,
            cost_bytes: system_bytes,
            cost_tokens: system_tokens,
        },
        CandidateObject {
            object_ref: contract_row.contract_id.to_string(),
            object_version: contract.header.object_version,
            content_digest: contract.header.content_digest.0.clone(),
            governance: ObjectGovernance {
                object_ref: contract_row.contract_id.to_string(),
                ..fragment_governance
            },
            role: LoadedContextItemRole::AuthoritativeState,
            trust_level: LoadedContextItemTrustLevel::Verified,
            representation: LoadedContextItemRepresentation::Structured,
            body: task_body,
            cost_bytes: task_bytes,
            cost_tokens: task_tokens,
        },
    ])
}

/// Resolve daemon-admitted Context for a TaskContract before Pi can make a
/// non-authoritative candidate proposal. Metadata is queried first, each
/// source is authorized with current revocation currency, and only authorized
/// sources have their durable body materialized. This function neither calls
/// Pi nor persists a candidate, Intent, Effect, WIA, budget debit, progress,
/// evidence, verification, acceptance, or Task completion.
pub(crate) fn resolve_authorized_task_context<S>(
    store: &S,
    command: &ContextResolutionCommand,
) -> Result<ResolvedContextView, SchedulerAuthorityError>
where
    S: AuthorityStore
        + ContextStore
        + ContextAuthorizationFactStore
        + IntentChainStore
        + ProtocolStore,
{
    resolve_authorized_task_context_after_metadata(store, command, || Ok(()))
}

/// Resolve Context after the metadata-only discovery stage. Production calls
/// this through [`resolve_authorized_task_context`] with a no-op observer; the
/// private observer makes the discovery-to-body authorization boundary
/// deterministically testable without exposing a runtime control surface.
fn resolve_authorized_task_context_after_metadata<S, F>(
    store: &S,
    command: &ContextResolutionCommand,
    after_metadata: F,
) -> Result<ResolvedContextView, SchedulerAuthorityError>
where
    S: AuthorityStore
        + ContextStore
        + ContextAuthorizationFactStore
        + IntentChainStore
        + ProtocolStore,
    F: FnOnce() -> Result<(), SchedulerAuthorityError>,
{
    let current_contract_epoch = store
        .current_contract_epoch(&command.task_ref)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?;
    let contract_row = store
        .load_task_contract(&command.task_ref, current_contract_epoch)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::MissingContract(command.task_ref.clone()))?;
    let contract = parse_execution_bound_contract(&contract_row.canonical_json)?;
    let contract_request_reference = contract.context_request_ref.as_ref().ok_or_else(|| {
        SchedulerAuthorityError::ContextRequestUnavailable(
            "current TaskContract has no ContextRequest binding".to_owned(),
        )
    })?;
    let request_row = store
        .load_context_request(&command.request_id)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextRequestUnavailable(command.request_id.to_string())
        })?;
    if contract_request_reference.id.0.as_str() != command.request_id.as_str()
        || contract_request_reference.kind != StrongReferenceKind::Strong
        || contract_request_reference.object_version != 1
        || contract_request_reference.content_digest.0 != request_row.request_digest
    {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "current TaskContract ContextRequest reference differs from durable request".to_owned(),
        ));
    }
    if request_row.task_ref != command.task_ref {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "ContextRequest task binding differs from scheduler task".to_owned(),
        ));
    }
    let context_request: ContextRequest = serde_json::from_str(&request_row.canonical_json)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?;
    if context_request.perspective.task != command.task_ref {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "ContextRequest payload task differs from durable task binding".to_owned(),
        ));
    }
    if context_request.perspective.principal != command.authorization_subject_ref {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "ContextRequest principal differs from scheduler authorization subject".to_owned(),
        ));
    }
    let authorization_snapshot = load_current_context_authorization_snapshot(store, command)?;
    let mut required_fragment_candidates = build_required_task_fragments(
        &authorization_snapshot,
        command,
        &request_row,
        &context_request,
        &contract_row,
        &contract,
    )?;
    let required_fragment_refs = required_fragment_candidates
        .iter()
        .map(|candidate| candidate.object_ref.clone())
        .collect::<Vec<_>>();
    let metadata = store
        .query_context_candidate_metadata(&ContextCandidateQuery {
            tenant_id: command.tenant_id.clone(),
            resource_scope_prefix: command.resource_scope_prefix.clone(),
            conversation_ref: command.conversation_ref.clone(),
            limit: command.source_limit,
        })
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?;
    after_metadata()?;

    let mut authorized_candidates =
        Vec::with_capacity(metadata.len() + required_fragment_candidates.len());
    authorized_candidates.append(&mut required_fragment_candidates);
    let mut authorization_denied_after_discovery = false;
    for source_metadata in metadata {
        // Discovery is metadata-only. Re-read durable authorization state for
        // every body, so a revocation that lands after discovery cannot reach
        // body materialization, ranking, rendering, or the Pi boundary.
        let current_authorization_snapshot =
            load_current_context_authorization_snapshot(store, command)?;
        if authorize(
            &current_authorization_snapshot,
            &source_metadata.governance,
            &AccessRequest {
                action: "read_body".to_owned(),
                purpose: context_request.purpose.clone(),
            },
        )
        .is_err()
        {
            authorization_denied_after_discovery = true;
            continue;
        }
        let source = store
            .load_workspace_context_source_body(&source_metadata.source_id)
            .map_err(|error| SchedulerAuthorityError::ContextBodyUnavailable(error.to_string()))?
            .ok_or_else(|| {
                SchedulerAuthorityError::ContextBodyUnavailable(
                    source_metadata.source_id.to_string(),
                )
            })?;
        if source.source_digest != source_metadata.source_digest
            || source.governance != source_metadata.governance
            || source.role != source_metadata.role
            || source.trust_level != source_metadata.trust_level
            || source.content_bytes != source_metadata.content_bytes
            || source.content_tokens != source_metadata.content_tokens
        {
            return Err(SchedulerAuthorityError::ContextBodyUnavailable(
                "Context body metadata no longer matches its discovery record".to_owned(),
            ));
        }
        let source_payload: Value = serde_json::from_str(&source.canonical_json)
            .map_err(|error| SchedulerAuthorityError::ContextBodyUnavailable(error.to_string()))?;
        let body = source_payload.get("body").cloned().ok_or_else(|| {
            SchedulerAuthorityError::ContextBodyUnavailable(
                "WorkspaceContextSource payload has no body".to_owned(),
            )
        })?;
        let source_governance = source.governance;
        authorized_candidates.push(CandidateObject {
            object_ref: source_governance.object_ref.clone(),
            object_version: 1,
            content_digest: source.source_digest,
            governance: source_governance,
            role: source.role,
            trust_level: source.trust_level,
            representation: source.representation,
            body,
            cost_bytes: source.content_bytes,
            cost_tokens: source.content_tokens.unwrap_or(0),
        });
    }

    if authorization_denied_after_discovery && authorized_candidates.is_empty() {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "all discovered Context sources were denied before body materialization".to_owned(),
        ));
    }

    let resolution_request = ResolutionRequest {
        snapshot: authorization_snapshot,
        purpose: context_request.purpose,
        conversation_ref: command.conversation_ref.clone(),
        required: required_fragment_refs
            .into_iter()
            .chain(
                context_request
                    .required
                    .into_iter()
                    .map(|required| required.r#ref),
            )
            .map(|object_ref| RequiredItem { object_ref })
            .collect(),
        allow_partial: context_request.allow_partial,
        budget: ContextBudget {
            context_bytes: context_request.budget.context_bytes,
            input_tokens: context_request.budget.input_tokens,
        },
        render: RenderSpec {
            renderer_version: "personal-context-render/1".to_owned(),
            target_profile: context_request.target_profile.schema,
        },
        schema_digest: cognitive_contracts::generated::context_request::SCHEMA_DIGEST.to_owned(),
    };
    resolve(
        &resolution_request,
        &authorized_candidates,
        &ArrivalOrderRanker,
    )
    .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))
}

/// Persist the exact immutable ContextView that is about to become input to a
/// private candidate producer. The durable view intentionally contains only
/// source metadata and strong references; source bodies remain confined to the
/// already-authorized resolver and the bounded rendered transport.
fn persist_resolved_context_view<S, C, G>(
    store: &S,
    clock: &C,
    identifiers: &G,
    request_row: &ContextRequestRow,
    resolved_view: &ResolvedContextView,
    governance: &GovernanceSeed,
) -> Result<ContextViewRow, SchedulerAuthorityError>
where
    S: ContextStore,
    C: Clock,
    G: IdGenerator,
{
    let view_id = next_object_id(identifiers)?;
    let resolved_at = clock
        .now()
        .map_err(|error| SchedulerAuthorityError::ContextResolution(error.detail))?;
    let header = compose_governed_header(
        &view_id,
        "ContextView",
        "cognitiveos.context-view/0.1",
        governance,
        vec![format!(
            "activity://personal/context/{}",
            request_row.request_id
        )],
        vec![request_row.request_id.to_string()],
        "daemon-persisted-context-resolution",
        &resolved_at,
    )
    .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?;
    let loaded = resolved_view
        .loaded
        .iter()
        .map(|item| {
            let source_id = ObjectId::parse(&item.object_ref).map_err(|_| {
                SchedulerAuthorityError::ContextResolution(
                    "resolved Context item identity is not a governed object identifier".to_owned(),
                )
            })?;
            Ok(LoadedContextItem {
                item_id: item.object_ref.clone(),
                object_ref: strong_reference_to(&source_id, &item.content_digest),
                representation: item.representation,
                trust_level: item.trust_level,
                role: item.role,
                cost: ItemCost {
                    bytes: item.cost_bytes,
                    tokens: Some(item.cost_tokens),
                },
            })
        })
        .collect::<Result<Vec<_>, SchedulerAuthorityError>>()?;
    let pinned_versions = resolved_view
        .pinned_versions
        .iter()
        .map(|(object_ref, version)| {
            (
                object_ref.clone(),
                ContextViewPinnedVersionsValue::Integer(*version),
            )
        })
        .collect();
    let payload = ContextView {
        activity_bound: format!("activity://personal/context/{}", request_row.request_id),
        complete: resolved_view.complete,
        cost: ResolutionCost {
            bytes: resolved_view
                .loaded
                .iter()
                .map(|item| item.cost_bytes)
                .sum(),
            money_microunits: None,
            resolve_ms: 0,
            tokens: Some(
                resolved_view
                    .loaded
                    .iter()
                    .map(|item| item.cost_tokens)
                    .sum(),
            ),
        },
        header,
        loaded,
        loss_declaration: resolved_view
            .loss_declaration
            .iter()
            .map(|loss| LossDeclaration {
                omitted_classes: loss.omitted_classes.clone(),
                source: loss.source.clone(),
                transform: loss.transform.clone(),
                verification: resolved_view.render.digest.clone(),
            })
            .collect(),
        missing: (!resolved_view.missing.is_empty()).then(|| resolved_view.missing.clone()),
        pinned_versions,
        rejected: resolved_view
            .rejected
            .iter()
            .map(|rejected| PersistedRejectedCandidate {
                candidate_ref: rejected.candidate_ref.clone(),
                reason: rejected.reason.clone(),
            })
            .collect(),
        request_ref: strong_reference_to(&request_row.request_id, &request_row.request_digest),
    };
    let payload_value = serde_json::to_value(payload)
        .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?;
    let (sealed_payload, view_digest) = seal_governed_object_content_digest(payload_value)
        .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?;
    let canonical_json = String::from_utf8(
        canonical::canonical_bytes_of_value(&sealed_payload)
            .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?,
    )
    .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?;
    let view_row = ContextViewRow {
        view_id,
        request_id: request_row.request_id.clone(),
        view_digest,
        canonical_json,
    };
    store
        .append_context_view(&view_row)
        .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?;
    Ok(view_row)
}

/// Resolve Context, obtain exactly one untrusted Pi candidate, then have the
/// daemon seal, persist, and atomically admit it. Pi-provided fields never
/// become governed references, header facts, authority, or lifecycle state.
/// The resulting WIA remains unconsumed: executor dispatch and Task outcomes
/// belong to later scheduler and verifier paths.
pub(crate) fn propose_persist_and_admit_candidate<S, C, G, P>(
    store: &S,
    clock: &C,
    identifiers: &G,
    context_command: &ContextResolutionCommand,
    proposer: &P,
    admission_command: &DaemonCandidateAdmissionCommand,
) -> Result<CandidateAdmissionReceipt, SchedulerAuthorityError>
where
    S: AuthorityStore
        + ContextStore
        + ContextAuthorizationFactStore
        + HarnessStore
        + IntentChainStore
        + ProtocolStore
        + WorkerAuthorizationStore,
    C: Clock,
    G: IdGenerator,
    P: PrivatePiCandidateProposer,
{
    propose_persist_and_admit_candidate_after_metadata(
        store,
        clock,
        identifiers,
        context_command,
        proposer,
        admission_command,
        || Ok(()),
    )
}

/// Candidate composition with the same private metadata observer used by the
/// resolver test seam. The production wrapper above always supplies a no-op.
fn propose_persist_and_admit_candidate_after_metadata<S, C, G, P, F>(
    store: &S,
    clock: &C,
    identifiers: &G,
    context_command: &ContextResolutionCommand,
    proposer: &P,
    admission_command: &DaemonCandidateAdmissionCommand,
    after_metadata: F,
) -> Result<CandidateAdmissionReceipt, SchedulerAuthorityError>
where
    S: AuthorityStore
        + ContextStore
        + ContextAuthorizationFactStore
        + HarnessStore
        + IntentChainStore
        + ProtocolStore
        + WorkerAuthorizationStore,
    C: Clock,
    G: IdGenerator,
    P: PrivatePiCandidateProposer,
    F: FnOnce() -> Result<(), SchedulerAuthorityError>,
{
    // Candidate identity is daemon-owned and stable across a scheduler retry.
    // Never call Pi twice for that identity: a previously committed admission
    // returns its original receipt; a proposal persisted before a failed
    // admission resumes deterministic daemon-only admission.
    if let Some(existing_candidate) = store
        .load_operation_candidate_proposal(&admission_command.candidate_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
    {
        let current_contract_epoch = store
            .current_contract_epoch(&context_command.task_ref)
            .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
        if existing_candidate.task_ref != context_command.task_ref
            || existing_candidate.contract_epoch != current_contract_epoch
        {
            return Err(SchedulerAuthorityError::CandidateUnavailable(
                "candidate retry identity is bound to a different TaskContract epoch".to_owned(),
            ));
        }
        if let Some(receipt) = store
            .load_candidate_admission_receipt_by_selected_candidate_id(
                &admission_command.candidate_id,
            )
            .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        {
            return Ok(receipt);
        }
        return admit_candidate_atomically(store, clock, identifiers, admission_command);
    }
    let resolved_context =
        resolve_authorized_task_context_after_metadata(store, context_command, after_metadata)?;
    let request_row = store
        .load_context_request(&context_command.request_id)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextRequestUnavailable(
                context_command.request_id.to_string(),
            )
        })?;
    persist_resolved_context_view(
        store,
        clock,
        identifiers,
        &request_row,
        &resolved_context,
        &admission_command.governance,
    )?;
    let current_contract_epoch = store
        .current_contract_epoch(&context_command.task_ref)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let contract_row = store
        .load_task_contract(&context_command.task_ref, current_contract_epoch)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::MissingContract(context_command.task_ref.clone())
        })?;
    let task_contract = parse_execution_bound_contract(&contract_row.canonical_json)?;
    if task_contract
        .context_request_ref
        .as_ref()
        .map(|reference| reference.id.0.as_str())
        != Some(context_command.request_id.as_str())
    {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "current TaskContract ContextRequest binding changed before Pi proposal".to_owned(),
        ));
    }

    let proposed_candidate = proposer
        .propose_candidate(
            &resolved_context,
            &context_command.task_ref,
            current_contract_epoch,
        )
        .map_err(SchedulerAuthorityError::PrivatePiProposal)?;
    validate_untrusted_pi_candidate(&proposed_candidate)?;
    let descriptor = store
        .load_daemon_operation_descriptor(&proposed_candidate.operation_descriptor_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::CandidateDescriptorUnavailable(
                proposed_candidate.operation_descriptor_id.to_string(),
            )
        })?;
    if proposed_candidate.tool_ref.starts_with("native.") {
        resolve_persisted_native_descriptor(&descriptor.descriptor).map_err(|error| {
            SchedulerAuthorityError::CandidateDescriptorUnavailable(format!(
                "native Tool registry rejected {}: {error:?}",
                proposed_candidate.tool_ref
            ))
        })?;
    }
    if descriptor.descriptor.operation_id != proposed_candidate.tool_ref
        || descriptor.descriptor.action != proposed_candidate.action
        || admit_operation(&descriptor.descriptor).is_err()
    {
        return Err(SchedulerAuthorityError::CandidateDescriptorUnavailable(
            proposed_candidate.operation_descriptor_id.to_string(),
        ));
    }
    let descriptor_reference_digest =
        canonical_descriptor_reference_digest(&descriptor.canonical_json)?;
    let proposed_at = clock
        .now()
        .map_err(|error| SchedulerAuthorityError::Store(error.detail))?;
    let candidate_header = compose_governed_header(
        &admission_command.candidate_id,
        "OperationCandidateProposal",
        OPERATION_CANDIDATE_SCHEMA_VERSION,
        &admission_command.governance,
        vec!["observation://personal/private-pi-proposer".to_owned()],
        vec![task_contract.header.id.0.to_owned()],
        "daemon-sealed-private-pi-candidate",
        &proposed_at,
    )
    .map_err(|error| SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string()))?;
    let daemon_sealed_candidate = OperationCandidateProposal {
        action: proposed_candidate.action.clone(),
        candidate_source_ref: "observation://personal/private-pi-proposer".to_owned(),
        contract_epoch: current_contract_epoch,
        expected_state_version: proposed_candidate.expected_state_version,
        header: candidate_header,
        operation_descriptor_ref: strong_reference_to(
            &descriptor.descriptor_id,
            &descriptor_reference_digest,
        ),
        parameters_digest: proposed_candidate.parameters_digest.clone(),
        target: proposed_candidate.target.clone(),
        task_contract_ref: strong_reference_to(
            &contract_row.contract_id,
            &task_contract.header.content_digest.0,
        ),
        tool_ref: proposed_candidate.tool_ref.clone(),
    };
    let candidate_value = serde_json::to_value(&daemon_sealed_candidate).map_err(|error| {
        SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string())
    })?;
    let (sealed_candidate_value, _) = seal_governed_object_content_digest(candidate_value)
        .map_err(|error| {
            SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string())
        })?;
    let candidate_canonical_json = String::from_utf8(
        canonical::canonical_bytes_of_value(&sealed_candidate_value).map_err(|error| {
            SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string())
        })?,
    )
    .map_err(|error| SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string()))?;
    store
        .append_operation_candidate_proposal(
            &cognitive_kernel::ports::OperationCandidateProposalRow {
                candidate_id: admission_command.candidate_id.clone(),
                task_ref: context_command.task_ref.clone(),
                contract_epoch: current_contract_epoch,
                candidate_source_ref: "observation://personal/private-pi-proposer".to_owned(),
                tool_ref: proposed_candidate.tool_ref,
                action: proposed_candidate.action,
                target: proposed_candidate.target,
                parameters_digest: proposed_candidate.parameters_digest,
                expected_state_version: proposed_candidate.expected_state_version,
                operation_descriptor_ref: descriptor.descriptor_id,
                canonical_json: candidate_canonical_json,
            },
        )
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    admit_candidate_atomically(store, clock, identifiers, admission_command)
}

fn validate_untrusted_pi_candidate(
    candidate: &UntrustedPiCandidate,
) -> Result<(), SchedulerAuthorityError> {
    let fields_are_present = !candidate.tool_ref.is_empty()
        && !candidate.action.is_empty()
        && !candidate.target.is_empty()
        && candidate.expected_state_version >= 1;
    if !fields_are_present || !is_sha256_digest(&candidate.parameters_digest) {
        return Err(SchedulerAuthorityError::PrivatePiProposal(
            "candidate has missing fields or an invalid parameters digest".to_owned(),
        ));
    }
    Ok(())
}

fn is_sha256_digest(value: &str) -> bool {
    let Some(hexadecimal) = value.strip_prefix("sha256:") else {
        return false;
    };
    hexadecimal.len() == 64 && hexadecimal.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn canonical_descriptor_reference_digest(
    descriptor_canonical_json: &str,
) -> Result<String, SchedulerAuthorityError> {
    let descriptor_value: Value =
        serde_json::from_str(descriptor_canonical_json).map_err(|error| {
            SchedulerAuthorityError::CandidateDescriptorUnavailable(error.to_string())
        })?;
    let descriptor_bytes =
        canonical::canonical_bytes_of_value(&descriptor_value).map_err(|error| {
            SchedulerAuthorityError::CandidateDescriptorUnavailable(error.to_string())
        })?;
    canonical::digest(&descriptor_bytes, DAEMON_DESCRIPTOR_REFERENCE_DIGEST_DOMAIN)
        .map_err(|error| SchedulerAuthorityError::CandidateDescriptorUnavailable(error.to_string()))
}

/// Reject scheduler work that was bound to a superseded TaskContract epoch.
/// This fence runs before any lease mutation or harness invocation.
fn ensure_current_contract_epoch(
    binding: &SchedulerAuthorityBinding,
    current_contract_epoch: i64,
) -> Result<(), SchedulerAuthorityError> {
    if binding.contract_epoch == current_contract_epoch {
        return Ok(());
    }

    Err(SchedulerAuthorityError::StaleContractEpoch {
        task_ref: binding.task_ref.clone(),
        requested_epoch: binding.contract_epoch,
        current_epoch: current_contract_epoch,
    })
}

/// Reload and validate all daemon-owned facts required before constructing an
/// atomic candidate-admission bundle. This preflight does not mint an Intent,
/// authorize an Effect, consume WIA, dispatch work, or record progress.
pub(crate) fn preflight_candidate_admission<S>(
    store: &S,
    candidate_id: &ObjectId,
    authorization_subject_ref: &str,
    authorization_purpose: &str,
    budget_charge: &BudgetCharge,
) -> Result<CandidateAdmissionPreflight, SchedulerAuthorityError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore + WorkerAuthorizationStore,
{
    let candidate = store
        .load_operation_candidate_proposal(candidate_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::CandidateUnavailable(candidate_id.to_string()))?;
    let current_epoch = store
        .current_contract_epoch(&candidate.task_ref)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    if current_epoch != candidate.contract_epoch {
        return Err(SchedulerAuthorityError::StaleContractEpoch {
            task_ref: candidate.task_ref,
            requested_epoch: candidate.contract_epoch,
            current_epoch,
        });
    }
    let contract_row = store
        .load_task_contract(&candidate.task_ref, current_epoch)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::MissingContract(candidate.task_ref.clone()))?;
    let contract = parse_execution_bound_contract(&contract_row.canonical_json)?;
    if contract.task_ref != candidate.task_ref || contract.contract_epoch != current_epoch {
        return Err(SchedulerAuthorityError::CandidateUnavailable(
            "candidate and TaskContract binding disagree".to_owned(),
        ));
    }
    if !contract.allowed_tools.contains(&candidate.tool_ref) {
        return Err(SchedulerAuthorityError::CandidateToolForbidden(
            candidate.tool_ref,
        ));
    }
    let descriptor = store
        .load_daemon_operation_descriptor(&candidate.operation_descriptor_ref)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::CandidateDescriptorUnavailable(
                candidate.operation_descriptor_ref.to_string(),
            )
        })?;
    if descriptor.descriptor.operation_id != candidate.tool_ref
        || descriptor.descriptor.action != candidate.action
        || admit_operation(&descriptor.descriptor).is_err()
    {
        return Err(SchedulerAuthorityError::CandidateDescriptorUnavailable(
            candidate.operation_descriptor_ref.to_string(),
        ));
    }
    let authorization = store
        .load_latest_daemon_authorization_snapshot(
            authorization_subject_ref,
            &candidate.target,
            &candidate.action,
            authorization_purpose,
        )
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::CandidateAuthorizationUnavailable(candidate.action.clone())
        })?;
    if authorization.grant_epoch < 1
        || authorization.capability_set_version < 1
        || authorization.revocation_epoch < 1
    {
        return Err(SchedulerAuthorityError::CandidateAuthorizationUnavailable(
            candidate.action.clone(),
        ));
    }
    let loop_object_id = ObjectId::parse(
        &contract
            .loop_object_id
            .ok_or_else(|| {
                SchedulerAuthorityError::MalformedContract("contract has no loop".to_owned())
            })?
            .0,
    )
    .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    let budget_id = BudgetId::parse(
        &contract
            .budget_id
            .ok_or_else(|| {
                SchedulerAuthorityError::MalformedContract("contract has no budget".to_owned())
            })?
            .0,
    )
    .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    let loop_object = store
        .load_object(LifecycleDomain::Loop, &loop_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::LoopUnavailable(loop_object_id.to_string()))?;
    if loop_object.state.as_str() != "DECIDE" {
        return Err(SchedulerAuthorityError::LoopUnavailable(format!(
            "{} is {}",
            loop_object_id, loop_object.state
        )));
    }
    let stored_budget = store
        .load_budget(&budget_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::BudgetUnavailable(budget_id.to_string()))?;
    let next_budget_state = stored_budget
        .state
        .check_and_debit(budget_charge)
        .map_err(|error| SchedulerAuthorityError::BudgetUnavailable(error.to_string()))?;
    let next_budget_state_value = serde_json::to_value(&next_budget_state)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let next_budget_state_canonical_json = String::from_utf8(
        canonical::canonical_bytes_of_value(&next_budget_state_value)
            .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?,
    )
    .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let progress_facts = store
        .list_progress_facts(&loop_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    Ok(CandidateAdmissionPreflight {
        task_ref: candidate.task_ref,
        contract_epoch: current_epoch,
        loop_object_id,
        budget_id,
        expected_budget_version: stored_budget.version,
        next_budget_state_canonical_json,
        expected_loop_version: loop_object.version,
        next_iteration: progress_facts.last().map_or(1, |fact| fact.iteration + 1),
    })
}

/// Reload daemon-owned admission inputs, compose a schema-shaped bundle, and
/// persist it through the single atomic candidate-admission store sink.
///
/// This is intentionally daemon-private. It accepts neither worker output nor
/// client-supplied authority fields, and it does not consume WIA, dispatch an
/// executor, write a progress fact, release a scheduler lease, or accept a
/// Task.
pub(crate) fn admit_candidate_atomically<S, C, G>(
    store: &S,
    clock: &C,
    ids: &G,
    command: &DaemonCandidateAdmissionCommand,
) -> Result<CandidateAdmissionReceipt, SchedulerAuthorityError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore + WorkerAuthorizationStore,
    C: Clock,
    G: IdGenerator,
{
    let preflight = preflight_candidate_admission(
        store,
        &command.candidate_id,
        &command.authorization_subject_ref,
        &command.authorization_purpose,
        &command.budget_charge,
    )?;
    let candidate = store
        .load_operation_candidate_proposal(&command.candidate_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::CandidateUnavailable(command.candidate_id.to_string())
        })?;
    let task_contract = store
        .load_task_contract(&preflight.task_ref, preflight.contract_epoch)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::MissingContract(preflight.task_ref.clone()))?;
    let descriptor = store
        .load_daemon_operation_descriptor(&candidate.operation_descriptor_ref)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::CandidateDescriptorUnavailable(
                candidate.operation_descriptor_ref.to_string(),
            )
        })?;
    let authorization = store
        .load_latest_daemon_authorization_snapshot(
            &command.authorization_subject_ref,
            &candidate.target,
            &candidate.action,
            &command.authorization_purpose,
        )
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::CandidateAuthorizationUnavailable(candidate.action.clone())
        })?;
    let writer_lease = WriterLease {
        epoch: store
            .current_fencing_epoch()
            .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?,
    };
    let admitted_at = clock
        .now()
        .map_err(|error| SchedulerAuthorityError::Store(error.detail))?;
    let identities = CandidateAdmissionIdentities {
        authorization_id: next_object_id(ids)?,
        intent_id: next_object_id(ids)?,
        effect_object_id: next_object_id(ids)?,
        intent_event_id: next_event_id(ids)?,
        effect_event_id: next_event_id(ids)?,
        loop_event_id: next_event_id(ids)?,
        loop_record_id: next_record_id(ids)?,
    };
    let commit = compose_candidate_admission(&CandidateAdmissionInputs {
        candidate,
        task_contract,
        descriptor,
        authorization,
        authorization_subject_ref: command.authorization_subject_ref.clone(),
        authorization_purpose: command.authorization_purpose.clone(),
        facts: CandidateAdmissionFacts {
            loop_object_id: preflight.loop_object_id,
            budget_id: preflight.budget_id,
            expected_budget_version: preflight.expected_budget_version,
            next_budget_state_canonical_json: preflight.next_budget_state_canonical_json,
            expected_loop_version: preflight.expected_loop_version,
            iteration: preflight.next_iteration,
        },
        budget_charge: command.budget_charge.clone(),
        governance: command.governance.clone(),
        identities,
        actor_ref: command.actor_ref.clone(),
        authority_ref: command.authority_ref.clone(),
        correlation_id: command.correlation_id.clone(),
        admitted_at,
        writer_lease,
    })
    .map_err(|error| SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string()))?;
    store
        .commit_candidate_admission(&commit)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))
}

/// Reload and consume one daemon-issued WIA before a worker can use its
/// bounded input. The consumption record is an immutable handoff only.
///
/// A superseded TaskContract or missing authorization fails closed. This
/// function deliberately does not dispatch an executor or convert any worker
/// observation into progress, evidence, verification, lease release, Task
/// acceptance, or Task completion.
pub(crate) fn consume_worker_authorization_for_attempt<S, C>(
    store: &S,
    clock: &C,
    authorization_id: &ObjectId,
    worker_attempt_id: ObjectId,
    scheduler_dispatch: &SchedulerDispatch,
) -> Result<WorkerAuthorizationHandoff, SchedulerAuthorityError>
where
    S: IntentChainStore + ProtocolStore + WorkerAuthorizationStore,
    C: Clock,
{
    let authorization = store
        .load_worker_iteration_authorization(authorization_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::CandidateUnavailable(authorization_id.to_string())
        })?;
    // Consumption may be invoked independently of the bounded attempt
    // composition, so revalidate the sealed evidence at this authority edge.
    validate_worker_authorization_evidence(&authorization)?;
    let current_contract_epoch = store
        .current_contract_epoch(&authorization.task_ref)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    if current_contract_epoch != authorization.contract_epoch {
        return Err(SchedulerAuthorityError::StaleContractEpoch {
            task_ref: authorization.task_ref.clone(),
            requested_epoch: authorization.contract_epoch,
            current_epoch: current_contract_epoch,
        });
    }
    if scheduler_dispatch.task_ref != authorization.task_ref
        || scheduler_dispatch.contract_epoch != authorization.contract_epoch
    {
        return Err(SchedulerAuthorityError::DispatchBindingMismatch(
            "scheduler dispatch does not match the WorkerIterationAuthorization task binding"
                .to_owned(),
        ));
    }
    let consumed_fencing_epoch = store
        .current_fencing_epoch()
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let consumed_at = clock
        .now()
        .map_err(|error| SchedulerAuthorityError::Store(error.detail))?;
    let consumption_value = serde_json::json!({
        "authorization_id": authorization.authorization_id.as_str(),
        "consumed_at": consumed_at.as_str(),
        "consumed_fencing_epoch": consumed_fencing_epoch,
        "kind": "worker_iteration_authorization_consumed",
        "worker_attempt_id": worker_attempt_id.as_str(),
    });
    let canonical_json = String::from_utf8(
        canonical::canonical_bytes_of_value(&consumption_value)
            .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?,
    )
    .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let scheduler_lease = SchedulerLeaseBinding {
        task_ref: scheduler_dispatch.task_ref.clone(),
        contract_epoch: scheduler_dispatch.contract_epoch,
        lease_owner: scheduler_dispatch.lease_owner.clone(),
        lease_epoch: scheduler_dispatch.lease_epoch,
    };
    store
        .consume_worker_iteration_authorization_bound_to_scheduler_lease(
            &BoundWorkerAuthorizationConsumption {
                consumption: WorkerIterationAuthorizationConsumptionRow {
                    authorization_id: authorization.authorization_id.clone(),
                    worker_attempt_id: worker_attempt_id.clone(),
                    consumed_fencing_epoch,
                    consumed_at,
                    canonical_json,
                },
                scheduler_lease: scheduler_lease.clone(),
            },
        )
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    Ok(WorkerAuthorizationHandoff {
        authorization,
        worker_attempt_id,
        scheduler_lease: Some(scheduler_lease),
    })
}

/// Verify that a durable WIA row and its schema-shaped evidence describe the
/// same immutable authority. A storage row must never become worker input
/// merely because its columns parse; its sealed payload remains the binding
/// evidence for every identity and version-sensitive field.
fn validate_worker_authorization_evidence(
    authorization: &WorkerIterationAuthorizationRow,
) -> Result<(), SchedulerAuthorityError> {
    let payload: WorkerIterationAuthorization = serde_json::from_str(&authorization.canonical_json)
        .map_err(|error| {
            SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string())
        })?;
    let payload_value = serde_json::to_value(&payload).map_err(|error| {
        SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string())
    })?;
    cognitive_contracts::projection::verify_content_digest(
        &payload_value,
        &["/header/content_digest"],
        "governed-object-content/0.1",
        "/header/content_digest",
    )
    .map_err(|error| SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string()))?;
    let rows_match_payload = payload.header.id.0.as_str()
        == authorization.authorization_id.as_str()
        && payload.worker_authorization_root_id.0.as_str()
            == authorization.worker_authorization_root_id.as_str()
        && payload.contract_epoch == authorization.contract_epoch
        && payload.iteration == authorization.iteration
        && payload.expected_loop_version == authorization.expected_loop_version.get()
        && payload.selected_candidate_ref.id.0.as_str()
            == authorization.selected_candidate_id.as_str()
        && payload.intent_ref.id.0.as_str() == authorization.intent_id.as_str()
        && payload.effect_ref.id.0.as_str() == authorization.effect_object_id.as_str()
        && payload.budget_id.0.as_str() == authorization.budget_id.as_str()
        && payload.action_fingerprint == authorization.action_fingerprint
        && payload.issued_fencing_epoch == authorization.issued_fencing_epoch;
    if !rows_match_payload {
        return Err(SchedulerAuthorityError::CandidateAdmissionComposition(
            "worker authorization row and sealed evidence disagree".to_owned(),
        ));
    }
    let payload_budget_charge = serde_json::to_value(&payload.budget_charge).map_err(|error| {
        SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string())
    })?;
    let canonical_budget_charge = String::from_utf8(
        canonical::canonical_bytes_of_value(&payload_budget_charge).map_err(|error| {
            SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string())
        })?,
    )
    .map_err(|error| SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string()))?;
    if canonical_budget_charge != authorization.budget_charge_canonical_json {
        return Err(SchedulerAuthorityError::CandidateAdmissionComposition(
            "worker authorization budget charge evidence disagrees with its row".to_owned(),
        ));
    }
    Ok(())
}

/// Discover in-flight worker attempts after daemon restart.
///
/// Only consumed WIA records appear here: issued-but-unconsumed authority is
/// not evidence that a worker started. Every discovered row is revalidated
/// against the current TaskContract epoch and its sealed WIA evidence before
/// reading the durable Effect state. No process-local callback, receipt, or
/// scheduler queue value participates in recovery.
pub(crate) fn recover_consumed_worker_attempts<S>(
    store: &S,
) -> Result<Vec<RecoveredWorkerAttempt>, SchedulerAuthorityError>
where
    S: AuthorityStore + IntentChainStore + ProtocolStore + WorkerAuthorizationStore,
{
    let consumed_authorizations = store
        .list_consumed_worker_iteration_authorizations()
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let mut recovered_attempts = Vec::with_capacity(consumed_authorizations.len());
    for consumed_authorization in consumed_authorizations {
        let authorization = consumed_authorization.authorization;
        validate_worker_authorization_evidence(&authorization)?;
        let current_contract_epoch = store
            .current_contract_epoch(&authorization.task_ref)
            .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
        ensure_current_contract_epoch(
            &SchedulerAuthorityBinding {
                task_ref: authorization.task_ref.clone(),
                contract_epoch: authorization.contract_epoch,
                action_fingerprint: authorization.action_fingerprint.clone(),
            },
            current_contract_epoch,
        )?;
        let effect = store
            .load_object(LifecycleDomain::Effect, &authorization.effect_object_id)
            .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
            .ok_or_else(|| {
                SchedulerAuthorityError::MissingEffect(
                    authorization.effect_object_id.as_str().to_owned(),
                )
            })?;
        recovered_attempts.push(RecoveredWorkerAttempt {
            handoff: WorkerAuthorizationHandoff {
                authorization,
                worker_attempt_id: consumed_authorization.consumption.worker_attempt_id,
                scheduler_lease: consumed_authorization.scheduler_lease,
            },
            effect_closure: classify_scheduler_effect_closure(effect.state.as_str())?,
        });
    }
    Ok(recovered_attempts)
}

/// Reconcile recovered handoffs whose authoritative Effects are closed.
///
/// A release is possible only when the durable handoff includes the exact
/// lease owner and epoch captured at consumption. Legacy unbound handoffs and
/// pending Effects deliberately retain their scheduler work. This does not
/// interpret worker output or complete a Task.
pub(crate) fn reconcile_recovered_worker_attempts<S, C>(
    store: &S,
    scheduler_repository: &mut SchedulerRepository,
    clock: &C,
) -> Result<Vec<RecoveredWorkerAttempt>, SchedulerAuthorityError>
where
    S: AuthorityStore + IntentChainStore + ProtocolStore + WorkerAuthorizationStore,
    C: Clock,
{
    let recovered_attempts = recover_consumed_worker_attempts(store)?;
    let released_at = clock
        .now()
        .map_err(|error| SchedulerAuthorityError::InvalidReleaseTime(error.detail))?;

    for recovered_attempt in &recovered_attempts {
        release_closed_recovered_attempt(
            recovered_attempt,
            scheduler_repository,
            released_at.as_str(),
        )?;
    }

    Ok(recovered_attempts)
}

/// Run daemon startup recovery against the single Personal authority database
/// before the HTTP endpoint is published. This intentionally reconciles only
/// already-consumed, exact-lease-bound handoffs; it never claims runnable work
/// or dispatches an executor during startup.
pub(crate) fn reconcile_scheduler_recovery_at_startup(
    authority_database_path: &Path,
) -> Result<(), SchedulerAuthorityError> {
    let authority_store = cognitive_store::SqliteAuthorityStore::open(authority_database_path)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let mut scheduler_repository = SchedulerRepository::open(authority_database_path)?;
    reconcile_recovered_worker_attempts(
        &authority_store,
        &mut scheduler_repository,
        &cognitive_store::SystemClock,
    )?;
    Ok(())
}

fn release_closed_recovered_attempt(
    recovered_attempt: &RecoveredWorkerAttempt,
    scheduler_repository: &mut SchedulerRepository,
    released_at: &str,
) -> Result<(), SchedulerAuthorityError> {
    if recovered_attempt.effect_closure != SchedulerEffectClosure::Closed {
        return Ok(());
    }
    let Some(scheduler_lease) = &recovered_attempt.handoff.scheduler_lease else {
        return Ok(());
    };
    if scheduler_lease.task_ref != recovered_attempt.handoff.authorization.task_ref
        || scheduler_lease.contract_epoch != recovered_attempt.handoff.authorization.contract_epoch
    {
        return Err(SchedulerAuthorityError::DispatchBindingMismatch(
            "recovered scheduler lease binding does not match its WorkerIterationAuthorization"
                .to_owned(),
        ));
    }
    scheduler_repository.release_lease(
        &SchedulerWorkKey {
            task_ref: scheduler_lease.task_ref.clone(),
            contract_epoch: scheduler_lease.contract_epoch,
        },
        &scheduler_lease.lease_owner,
        scheduler_lease.lease_epoch,
        SchedulerState::Succeeded,
        released_at,
    )?;
    Ok(())
}

fn next_object_id<G: IdGenerator>(ids: &G) -> Result<ObjectId, SchedulerAuthorityError> {
    let raw_id = ids
        .next_uuid_v7()
        .map_err(|error| SchedulerAuthorityError::Store(error.detail))?;
    ObjectId::parse(&raw_id)
        .map_err(|error| SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string()))
}

fn next_event_id<G: IdGenerator>(ids: &G) -> Result<EventId, SchedulerAuthorityError> {
    let raw_id = ids
        .next_uuid_v7()
        .map_err(|error| SchedulerAuthorityError::Store(error.detail))?;
    EventId::parse(&raw_id)
        .map_err(|error| SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string()))
}

fn next_record_id<G: IdGenerator>(ids: &G) -> Result<RecordId, SchedulerAuthorityError> {
    let raw_id = ids
        .next_uuid_v7()
        .map_err(|error| SchedulerAuthorityError::Store(error.detail))?;
    RecordId::parse(&raw_id)
        .map_err(|error| SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{
        ContextResolutionCommand, RecoveredWorkerAttempt, SchedulerAuthorityBinding,
        SchedulerAuthorityError, SchedulerDispatchAdmission, SchedulerEffectClosure,
        SchedulerWorkerAttempt, UntrustedPiCandidate, WorkerAuthorizationHandoff,
        candidate_admission_command_from_policy, classify_scheduler_effect_closure,
        complete_resolved_effect_and_release, complete_scheduler_admission,
        complete_scheduler_worker_attempt, ensure_current_contract_epoch,
        parse_execution_bound_contract, propose_persist_and_admit_candidate_after_metadata,
        release_closed_effect_dispatch, release_closed_recovered_attempt,
        select_single_effect_intent, validate_untrusted_pi_candidate,
        validate_worker_authorization_evidence,
    };
    use cognitive_contracts::{
        canonical,
        generated::governed_object_header::GovernedObjectHeaderSensitivity,
        generated::{
            common_defs::Budget,
            context_view::{
                LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
            },
            task_contract::{ContractCondition, ContractConditionKind, TaskContract, TaskScope},
            worker_iteration_authorization::WorkerIterationAuthorization,
        },
    };
    use cognitive_domain::{
        BudgetId, EventId, LifecycleDomain, ObjectId, RecordId, StateName, UriRef, Version,
        WallTimestamp,
        capability::{CapabilityConstraints, LeaseWindow},
    };
    use cognitive_kernel::authz::{
        AccessRequest, ActorChainFacts, AuthzSnapshot, MembershipFacts, ObjectGovernance,
        PrincipalFacts, authorize,
    };
    use cognitive_kernel::budget::BudgetCharge;
    use cognitive_kernel::budget::BudgetState;
    use cognitive_kernel::effects::{EffectProtocol, GovernanceCurrency, WriterLease};
    use cognitive_kernel::engine::CommittedTransition;
    use cognitive_kernel::intent_chain::{
        GovernanceSeed, compose_governed_header, seal_governed_object_content_digest,
        strong_reference_to,
    };
    use cognitive_kernel::ports::{
        AuthorityStore, BudgetCas, CandidateAdmissionCommit, ContextAuthorizationFactStore,
        ContextAuthorizationFactsRow, ContextRequestRow, ContextRevocationFactRow, ContextStore,
        EventDraft, IntentChainStore, IntentRow, ObjectAdmission, ObjectCas,
        OperationCandidateProposalRow, RecordDraft, SchedulerExecutionPolicyRow,
        SchedulerLeaseBinding, StoredObject, TaskBinding, TaskContractRow, TransitionCommit,
        WorkerAuthorizationStore, WorkerIterationAuthorizationRow, WorkspaceContextSourceRow,
    };
    use cognitive_runtime::{SchedulerCeilingDispatch, SchedulerDispatch};
    use cognitive_store::{
        PersonalDataLayout, ScriptedExecutor, SqliteAuthorityStore, UuidV7Generator,
        prepare_personal_databases,
        scheduler::{SchedulerRepository, SchedulerRow, SchedulerState, SchedulerWorkKey},
    };
    use serde_json::json;
    use std::cell::Cell;
    use std::collections::BTreeMap;
    use std::io::Write;
    use std::net::TcpStream;
    use std::sync::mpsc;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scheduler_row(task_ref: &str) -> SchedulerRow {
        SchedulerRow {
            task_ref: task_ref.to_owned(),
            contract_epoch: 1,
            state: SchedulerState::Runnable.as_str().to_owned(),
            lease_owner: None,
            lease_epoch: 0,
            lease_expires: None,
            next_eligible: "2026-08-03T00:00:00Z".to_owned(),
            attempt_count: 0,
            cancel_requested: false,
        }
    }

    fn scheduler_work_key(task_ref: &str) -> SchedulerWorkKey {
        SchedulerWorkKey {
            task_ref: task_ref.to_owned(),
            contract_epoch: 1,
        }
    }

    fn object_id(sequence: u64) -> ObjectId {
        ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}")).unwrap()
    }

    fn context_governance() -> GovernanceSeed {
        GovernanceSeed {
            owner: strong_reference_to(&object_id(910), &format!("sha256:{}", "a".repeat(64))),
            authority: strong_reference_to(&object_id(911), &format!("sha256:{}", "b".repeat(64))),
            resource_scope: strong_reference_to(
                &object_id(912),
                &format!("sha256:{}", "c".repeat(64)),
            ),
            tenant_id: Some("tenant-a".to_owned()),
            created_by: "principal://tenant-a/daemon".to_owned(),
            sensitivity: GovernedObjectHeaderSensitivity::Internal,
            purpose_constraints: vec!["task_execution".to_owned()],
            retention_policy: "standard".to_owned(),
        }
    }

    fn seal_payload(payload: serde_json::Value) -> (String, String) {
        let (sealed_payload, digest) = seal_governed_object_content_digest(payload).unwrap();
        (serde_json::to_string(&sealed_payload).unwrap(), digest)
    }

    fn append_context_race_fixture(
        store: &SqliteAuthorityStore,
        task_ref: &str,
        required_context_ref: Option<&str>,
    ) -> (ContextResolutionCommand, ContextRevocationFactRow) {
        let governance = context_governance();
        let issued_at = WallTimestamp::parse("2026-08-07T00:00:00Z").unwrap();
        let request_id = object_id(920);
        let request_header = compose_governed_header(
            &request_id,
            "ContextRequest",
            "cognitiveos.context-request/0.1",
            &governance,
            Vec::new(),
            Vec::new(),
            "p2-t04-race-test-request",
            &issued_at,
        )
        .unwrap();
        let (request_json, request_digest) = seal_payload(json!({
            "header": request_header,
            "purpose": "task_execution",
            "perspective": {
                "principal": "principal://tenant-a/daemon",
                "task": task_ref,
                "episode": "episode://tenant-a/p2-t04-race",
            },
            "budget": {},
            "priority": ["task"],
            "required": required_context_ref.map(|object_ref| vec![json!({"ref": object_ref})]).unwrap_or_default(),
            "forbidden": [],
            "freshness": {"world_max_age_ms": 0},
            "sensitivity": {"max_input": "internal", "egress": "none"},
            "target_profile": {"kind": "structured", "schema": "p2-t04-race/v1"},
            "allow_partial": false,
        }));
        let request = ContextRequestRow {
            request_id: request_id.clone(),
            task_ref: task_ref.to_owned(),
            request_digest: request_digest.clone(),
            canonical_json: request_json,
        };
        store.append_context_request(&request).unwrap();

        let source_id = object_id(921);
        let source_header = compose_governed_header(
            &source_id,
            "WorkspaceContextSource",
            "cognitiveos.workspace-context-source/0.1",
            &governance,
            Vec::new(),
            Vec::new(),
            "p2-t04-race-test-source",
            &issued_at,
        )
        .unwrap();
        let (source_json, source_digest) = seal_payload(json!({
            "header": source_header,
            "tenant_id": "tenant-a",
            "owner_ref": "principal://tenant-a/daemon",
            "resource_scope": "workspace://tenant-a/project/alpha",
            "conversation_ref": "conversation://tenant-a/one",
            "role": "working",
            "trust_level": "verified",
            "representation": "text",
            "provenance_ref": "admission://tenant-a/daemon/race-test",
            "content_bytes": 20,
            "content_tokens": 5,
            "body": {"text": "must-not-reach-pi"},
        }));
        store
            .append_workspace_context_source(&WorkspaceContextSourceRow {
                source_id: source_id.clone(),
                source_digest,
                governance: ObjectGovernance {
                    object_ref: source_id.to_string(),
                    tenant_id: Some("tenant-a".to_owned()),
                    owner_ref: "principal://tenant-a/daemon".to_owned(),
                    resource_scope: "workspace://tenant-a/project/alpha".to_owned(),
                    conversation_ref: Some("conversation://tenant-a/one".to_owned()),
                },
                role: LoadedContextItemRole::Working,
                trust_level: LoadedContextItemTrustLevel::Verified,
                representation: LoadedContextItemRepresentation::Text,
                provenance_ref: "admission://tenant-a/daemon/race-test".to_owned(),
                content_bytes: 20,
                content_tokens: Some(5),
                canonical_json: source_json,
            })
            .unwrap();

        let principal = PrincipalFacts {
            principal_ref: UriRef::parse("principal://tenant-a/daemon").unwrap(),
            authenticated: true,
            active: true,
            tenant_id: Some("tenant-a".to_owned()),
        };
        let facts_id = object_id(922);
        let capability = CapabilityConstraints {
            subject: principal.principal_ref.to_string(),
            audience: "daemon://tenant-a/context".to_owned(),
            resource: "workspace://tenant-a/project".to_owned(),
            purpose: "task_execution".to_owned(),
            actions: ["read_body".to_owned()].into(),
            parameter_bounds: BTreeMap::new(),
            lease: LeaseWindow {
                not_before: WallTimestamp::parse("2026-08-06T00:00:00Z").unwrap(),
                expires: WallTimestamp::parse("2026-08-08T00:00:00Z").unwrap(),
            },
            depth_remaining: 1,
            issued_epoch: 1,
        };
        let actor_chain = ActorChainFacts {
            chain_digest: format!("sha256:{}", "d".repeat(64)),
            resolved: true,
        };
        let membership = Some(MembershipFacts {
            valid: true,
            roles: ["owner".to_owned()].into(),
        });
        let facts_header = compose_governed_header(
            &facts_id,
            "ContextAuthorizationFacts",
            "cognitiveos.context-authorization-facts/0.1",
            &governance,
            Vec::new(),
            Vec::new(),
            "p2-t04-race-test-facts",
            &issued_at,
        )
        .unwrap();
        let (facts_json, _) = seal_payload(json!({
            "header": facts_header,
            "fact_set_id": facts_id,
            "subject_ref": principal.principal_ref,
            "tenant_id": "tenant-a",
            "principal": principal,
            "actor_chain": actor_chain,
            "membership": membership,
            "capability_links": [capability],
            "explicit_denies": [],
            "capability_set_version": 1,
            "issued_revocation_epoch": 1,
        }));
        store
            .append_context_authorization_facts(&ContextAuthorizationFactsRow {
                fact_set_id: facts_id,
                subject_ref: "principal://tenant-a/daemon".to_owned(),
                tenant_id: "tenant-a".to_owned(),
                principal,
                actor_chain,
                membership,
                capability_links: vec![capability],
                explicit_denies: Vec::new(),
                capability_set_version: 1,
                issued_revocation_epoch: 1,
                canonical_json: facts_json,
            })
            .unwrap();

        let initial_revocation =
            context_revocation_fact(&governance, object_id(923), 1, &issued_at);
        store
            .append_context_revocation_fact(&initial_revocation)
            .unwrap();
        let later_revocation = context_revocation_fact(&governance, object_id(924), 2, &issued_at);

        let contract_id = object_id(925);
        let contract_header = compose_governed_header(
            &contract_id,
            "TaskContract",
            "cognitiveos.task-contract/0.4",
            &governance,
            Vec::new(),
            Vec::new(),
            "p2-t04-race-test-contract",
            &issued_at,
        )
        .unwrap();
        let contract = TaskContract {
            allowed_state_domains: vec!["task".to_owned(), "effect".to_owned()],
            allowed_tools: Vec::new(),
            budget: Budget {
                attention_slots: None,
                context_bytes: None,
                egress_bytes: None,
                input_tokens: None,
                money_microunits: None,
                output_tokens: None,
                semantic_calls: None,
                tool_calls: Some(1),
                wall_time_ms: None,
            },
            budget_id: Some(
                BudgetId::parse("00000000-0000-7000-b000-000000000926")
                    .unwrap()
                    .to_generated(),
            ),
            conditions: vec![ContractCondition {
                description: "test acceptance".to_owned(),
                id: "accept".to_owned(),
                kind: ContractConditionKind::Acceptance,
                machine_expression: None,
                verifier_ref: None,
            }],
            context_request_ref: Some(strong_reference_to(&request_id, &request_digest)),
            contract_epoch: 1,
            deadline: None,
            header: contract_header,
            human_gates: None,
            intent_acceptance_ref: strong_reference_to(
                &object_id(927),
                &format!("sha256:{}", "e".repeat(64)),
            ),
            intent_interpretation_ref: strong_reference_to(
                &object_id(928),
                &format!("sha256:{}", "f".repeat(64)),
            ),
            loop_object_id: Some(object_id(929).to_generated()),
            max_iterations: 1,
            max_retries: 0,
            objective: "race regression".to_owned(),
            scope: TaskScope {
                in_scope: vec!["test".to_owned()],
                out_of_scope: Vec::new(),
            },
            task_ref: task_ref.to_owned(),
            user_intent_ref: strong_reference_to(
                &object_id(930),
                &format!("sha256:{}", "1".repeat(64)),
            ),
            worker_authorization_root_id: Some(contract_id.to_generated()),
        };
        let (contract_json, contract_digest) =
            seal_payload(serde_json::to_value(contract).unwrap());
        store
            .insert_task_contract(
                &TaskContractRow {
                    contract_id: contract_id.clone(),
                    task_ref: task_ref.to_owned(),
                    contract_epoch: 1,
                    user_intent_record_id: object_id(930),
                    interpretation_id: object_id(928),
                    accepted_by: "principal://tenant-a/daemon".to_owned(),
                    contract_digest,
                    canonical_json: contract_json,
                },
                &EventDraft {
                    event_id: EventId::parse("00000000-0000-7000-a000-000000000925").unwrap(),
                    object_id: contract_id,
                    domain: LifecycleDomain::Task,
                    object_version: Version::INITIAL,
                    event_type: "task-contract.minted".to_owned(),
                    canonical_json: "{\"event\":\"p2-t04-race\"}".to_owned(),
                },
                0,
            )
            .unwrap();

        (
            ContextResolutionCommand {
                task_ref: task_ref.to_owned(),
                request_id,
                authorization_subject_ref: "principal://tenant-a/daemon".to_owned(),
                tenant_id: "tenant-a".to_owned(),
                resource_scope_prefix: "workspace://tenant-a/project".to_owned(),
                conversation_ref: Some("conversation://tenant-a/one".to_owned()),
                source_limit: 1,
                decided_at: issued_at,
            },
            later_revocation,
        )
    }

    fn context_revocation_fact(
        governance: &GovernanceSeed,
        fact_id: ObjectId,
        epoch: i64,
        issued_at: &WallTimestamp,
    ) -> ContextRevocationFactRow {
        let header = compose_governed_header(
            &fact_id,
            "ContextRevocationFact",
            "cognitiveos.context-revocation-fact/0.1",
            governance,
            Vec::new(),
            Vec::new(),
            "p2-t04-race-test-revocation",
            issued_at,
        )
        .unwrap();
        let (canonical_json, _) = seal_payload(
            json!({"header": header, "revocation_fact_id": fact_id, "tenant_id": "tenant-a", "revocation_epoch": epoch, "revoked_subject_ref": null, "revoked_capability_ref": null}),
        );
        ContextRevocationFactRow {
            revocation_fact_id: fact_id,
            tenant_id: "tenant-a".to_owned(),
            revocation_epoch: epoch,
            revoked_subject_ref: None,
            revoked_capability_ref: None,
            canonical_json,
        }
    }

    #[derive(Default)]
    struct CountingPiProposer {
        calls: Cell<usize>,
    }

    impl super::PrivatePiCandidateProposer for CountingPiProposer {
        fn propose_candidate(
            &self,
            _resolved_context: &super::ResolvedContextView,
            _task_ref: &str,
            _contract_epoch: i64,
        ) -> Result<UntrustedPiCandidate, String> {
            self.calls.set(self.calls.get() + 1);
            Err("Pi must not receive revoked Context".to_owned())
        }
    }

    #[test]
    fn revocation_after_metadata_discovery_blocks_body_ranking_and_private_pi() {
        let layout = temporary_personal_layout();
        layout.ensure_directories().unwrap();
        prepare_personal_databases(&layout).unwrap();
        let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
        let task_ref = "task://tenant-a/p2-t04-revocation-race";
        let (context_command, later_revocation) =
            append_context_race_fixture(&store, task_ref, None);
        let proposer = CountingPiProposer::default();
        let candidate_id = object_id(931);
        let admission_command = super::DaemonCandidateAdmissionCommand {
            candidate_id: candidate_id.clone(),
            authorization_subject_ref: "principal://tenant-a/daemon".to_owned(),
            authorization_purpose: "task_execution".to_owned(),
            budget_charge: BudgetCharge::new(BTreeMap::from([("tool_calls".to_owned(), 1)]))
                .unwrap(),
            governance: context_governance(),
            actor_ref: UriRef::parse("principal://tenant-a/daemon").unwrap(),
            authority_ref: UriRef::parse("authority://tenant-a/daemon").unwrap(),
            correlation_id: UriRef::parse("correlation://tenant-a/p2-t04-race").unwrap(),
        };

        let result = propose_persist_and_admit_candidate_after_metadata(
            &store,
            &super::FixedSchedulerClock::parse("2026-08-07T00:00:00Z").unwrap(),
            &UuidV7Generator,
            &context_command,
            &proposer,
            &admission_command,
            || {
                store
                    .append_context_revocation_fact(&later_revocation)
                    .map_err(|error| {
                        SchedulerAuthorityError::ContextAuthorizationUnavailable(error.to_string())
                    })
            },
        );

        assert!(matches!(
            result,
            Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(detail))
                if detail.contains("denied before body materialization")
        ));
        assert_eq!(proposer.calls.get(), 0, "revoked Context must not reach Pi");
        assert_eq!(
            store
                .load_current_context_revocation_epoch("tenant-a")
                .unwrap(),
            Some(2)
        );
        assert!(
            store
                .load_operation_candidate_proposal(&candidate_id)
                .unwrap()
                .is_none(),
            "a rejected Context must not persist a candidate"
        );

        drop(store);
        std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn missing_required_context_blocks_private_pi_and_candidate_admission() {
        let layout = temporary_personal_layout();
        layout.ensure_directories().unwrap();
        prepare_personal_databases(&layout).unwrap();
        let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
        let task_ref = "task://tenant-a/p2-t04-required-context";
        let (context_command, _) = append_context_race_fixture(
            &store,
            task_ref,
            Some("workspace://tenant-a/project/required-but-missing"),
        );
        let proposer = CountingPiProposer::default();
        let candidate_id = object_id(932);
        let admission_command = super::DaemonCandidateAdmissionCommand {
            candidate_id: candidate_id.clone(),
            authorization_subject_ref: "principal://tenant-a/daemon".to_owned(),
            authorization_purpose: "task_execution".to_owned(),
            budget_charge: BudgetCharge::new(BTreeMap::from([("tool_calls".to_owned(), 1)]))
                .unwrap(),
            governance: context_governance(),
            actor_ref: UriRef::parse("principal://tenant-a/daemon").unwrap(),
            authority_ref: UriRef::parse("authority://tenant-a/daemon").unwrap(),
            correlation_id: UriRef::parse("correlation://tenant-a/p2-t04-required").unwrap(),
        };

        let result = super::propose_persist_and_admit_candidate(
            &store,
            &super::FixedSchedulerClock::parse("2026-08-07T00:00:00Z").unwrap(),
            &UuidV7Generator,
            &context_command,
            &proposer,
            &admission_command,
        );

        assert!(matches!(
            result,
            Err(SchedulerAuthorityError::ContextResolution(detail))
                if detail.contains("CONTEXT_INCOMPLETE")
        ));
        assert_eq!(
            proposer.calls.get(),
            0,
            "incomplete Context must not reach Pi"
        );
        assert!(
            store
                .load_operation_candidate_proposal(&candidate_id)
                .unwrap()
                .is_none()
        );

        drop(store);
        std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn duplicate_candidate_retry_does_not_reinvoke_private_pi() {
        let layout = temporary_personal_layout();
        layout.ensure_directories().unwrap();
        prepare_personal_databases(&layout).unwrap();
        let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
        let task_ref = "task://tenant-a/p2-t04-duplicate-candidate";
        let (context_command, _) = append_context_race_fixture(&store, task_ref, None);
        let candidate_id = object_id(933);
        store
            .append_operation_candidate_proposal(&OperationCandidateProposalRow {
                candidate_id: candidate_id.clone(),
                task_ref: task_ref.to_owned(),
                contract_epoch: 1,
                candidate_source_ref: "observation://tenant-a/pi/previous-attempt".to_owned(),
                tool_ref: "operation://tenant-a/observe".to_owned(),
                action: "observe".to_owned(),
                target: "workspace://tenant-a/project/alpha".to_owned(),
                parameters_digest: format!("sha256:{}", "2".repeat(64)),
                expected_state_version: 1,
                operation_descriptor_ref: object_id(934),
                canonical_json: "{\"candidate\":\"previous-attempt\"}".to_owned(),
            })
            .unwrap();
        let proposer = CountingPiProposer::default();
        let admission_command = super::DaemonCandidateAdmissionCommand {
            candidate_id: candidate_id.clone(),
            authorization_subject_ref: "principal://tenant-a/daemon".to_owned(),
            authorization_purpose: "task_execution".to_owned(),
            budget_charge: BudgetCharge::new(BTreeMap::from([("tool_calls".to_owned(), 1)]))
                .unwrap(),
            governance: context_governance(),
            actor_ref: UriRef::parse("principal://tenant-a/daemon").unwrap(),
            authority_ref: UriRef::parse("authority://tenant-a/daemon").unwrap(),
            correlation_id: UriRef::parse("correlation://tenant-a/p2-t04-duplicate").unwrap(),
        };

        let result = super::propose_persist_and_admit_candidate(
            &store,
            &super::FixedSchedulerClock::parse("2026-08-07T00:00:00Z").unwrap(),
            &UuidV7Generator,
            &context_command,
            &proposer,
            &admission_command,
        );

        assert!(
            result.is_err(),
            "the deliberately incomplete daemon-only admission fixture must not succeed"
        );
        assert_eq!(
            proposer.calls.get(),
            0,
            "a duplicate candidate identity must resume daemon admission without another Pi proposal"
        );
        assert_eq!(
            store
                .load_operation_candidate_proposal(&candidate_id)
                .unwrap()
                .unwrap()
                .canonical_json,
            "{\"candidate\":\"previous-attempt\"}"
        );

        drop(store);
        std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn candidate_admission_rejects_policy_with_mismatched_durable_task_binding() {
        let context_request_id = object_id(901);
        let policy = SchedulerExecutionPolicyRow {
            task_ref: "task://personal/expected".to_owned(),
            contract_epoch: 1,
            context_request_id: context_request_id.clone(),
            canonical_json: json!({
                "schema_version": 1,
                "task_ref": "task://personal/substituted",
                "contract_epoch": 1,
                "context": {
                    "request_id": context_request_id.as_str(),
                    "authorization_subject_ref": "principal://personal/owner",
                    "tenant_id": "personal",
                    "resource_scope_prefix": "workspace://personal/",
                    "conversation_ref": null,
                    "source_limit": 1,
                },
                "admission": {
                    "candidate_id": object_id(902).as_str(),
                    "authorization_subject_ref": "principal://personal/owner",
                    "authorization_purpose": "task_execution",
                    "budget_charge": {"semantic_calls": 1},
                    "governance": {
                        "owner": strong_reference_to(&object_id(903), &format!("sha256:{}", "a".repeat(64))),
                        "authority": strong_reference_to(&object_id(904), &format!("sha256:{}", "b".repeat(64))),
                        "resource_scope": strong_reference_to(&object_id(905), &format!("sha256:{}", "c".repeat(64))),
                        "tenant_id": null,
                        "created_by": "principal://personal/daemon",
                        "sensitivity": "internal",
                        "purpose_constraints": ["task_execution"],
                        "retention_policy": "standard",
                    },
                    "actor_ref": "principal://personal/daemon",
                    "authority_ref": "authority://personal/daemon",
                    "correlation_id": "correlation://personal/scheduler",
                },
            })
            .to_string(),
        };

        let error = candidate_admission_command_from_policy(&policy).unwrap_err();

        assert!(matches!(
            error,
            SchedulerAuthorityError::CandidateAdmissionComposition(detail)
                if detail.contains("durable binding")
        ));
    }

    fn sealed_worker_authorization_row() -> WorkerIterationAuthorizationRow {
        let authorization_id = object_id(810);
        let worker_authorization_root_id = object_id(811);
        let selected_candidate_id = object_id(812);
        let intent_id = object_id(813);
        let effect_object_id = object_id(814);
        let task_contract_id = object_id(815);
        let budget_id = BudgetId::parse("00000000-0000-7000-b000-000000000816").unwrap();
        let budget_charge = Budget {
            attention_slots: None,
            context_bytes: None,
            egress_bytes: None,
            input_tokens: None,
            money_microunits: None,
            output_tokens: None,
            semantic_calls: None,
            tool_calls: Some(1),
            wall_time_ms: None,
        };
        let governance = GovernanceSeed {
            owner: strong_reference_to(&object_id(817), &format!("sha256:{}", "a".repeat(64))),
            authority: strong_reference_to(
                &object_id(818),
                &format!("sha256:{}", "b".repeat(64)),
            ),
            resource_scope: strong_reference_to(
                &object_id(819),
                &format!("sha256:{}", "c".repeat(64)),
            ),
            tenant_id: Some("00000000-0000-7000-9000-000000000820".to_owned()),
            created_by: "principal://personal/daemon".to_owned(),
            sensitivity: cognitive_contracts::generated::governed_object_header::GovernedObjectHeaderSensitivity::Internal,
            purpose_constraints: vec!["task_execution".to_owned()],
            retention_policy: "standard".to_owned(),
        };
        let issued_at = WallTimestamp::parse("2026-08-04T12:00:00Z").unwrap();
        let header = compose_governed_header(
            &authorization_id,
            "WorkerIterationAuthorization",
            "cognitiveos.worker-iteration-authorization/0.1",
            &governance,
            Vec::new(),
            Vec::new(),
            "scheduler-authority-evidence-test",
            &issued_at,
        )
        .unwrap();
        let payload = WorkerIterationAuthorization {
            action_fingerprint: format!("sha256:{}", "d".repeat(64)),
            budget_charge: budget_charge.clone(),
            budget_id: budget_id.to_generated(),
            contract_epoch: 1,
            effect_ref: strong_reference_to(
                &effect_object_id,
                &format!("sha256:{}", "e".repeat(64)),
            ),
            expected_loop_version: 1,
            header,
            intent_ref: strong_reference_to(&intent_id, &format!("sha256:{}", "f".repeat(64))),
            issued_fencing_epoch: 1,
            iteration: 1,
            selected_candidate_ref: strong_reference_to(
                &selected_candidate_id,
                &format!("sha256:{}", "1".repeat(64)),
            ),
            task_contract_ref: strong_reference_to(
                &task_contract_id,
                &format!("sha256:{}", "2".repeat(64)),
            ),
            worker_authorization_root_id: worker_authorization_root_id.to_generated(),
        };
        let payload_value = serde_json::to_value(&payload).unwrap();
        let (sealed_payload, _) = seal_governed_object_content_digest(payload_value).unwrap();
        let budget_charge_canonical_json = String::from_utf8(
            canonical::canonical_bytes_of_value(&serde_json::to_value(budget_charge).unwrap())
                .unwrap(),
        )
        .unwrap();

        WorkerIterationAuthorizationRow {
            authorization_id,
            worker_authorization_root_id,
            task_ref: "task://personal/sealed-worker-authorization".to_owned(),
            contract_epoch: 1,
            loop_object_id: object_id(821),
            iteration: 1,
            expected_loop_version: Version::INITIAL,
            selected_candidate_id,
            intent_id,
            effect_object_id,
            budget_id,
            budget_charge_canonical_json,
            action_fingerprint: payload.action_fingerprint,
            issued_fencing_epoch: 1,
            canonical_json: serde_json::to_string(&sealed_payload).unwrap(),
        }
    }

    fn recovered_closed_attempt(task_ref: &str, lease_epoch: i64) -> RecoveredWorkerAttempt {
        RecoveredWorkerAttempt {
            handoff: WorkerAuthorizationHandoff {
                authorization: WorkerIterationAuthorizationRow {
                    authorization_id: object_id(800),
                    worker_authorization_root_id: object_id(801),
                    task_ref: task_ref.to_owned(),
                    contract_epoch: 1,
                    loop_object_id: object_id(802),
                    iteration: 1,
                    expected_loop_version: Version::INITIAL,
                    selected_candidate_id: object_id(803),
                    intent_id: object_id(804),
                    effect_object_id: object_id(805),
                    budget_id: BudgetId::parse("00000000-0000-7000-b000-000000000806").unwrap(),
                    budget_charge_canonical_json: "{\"tool_calls\":1}".to_owned(),
                    action_fingerprint: "recovered-lease-release".to_owned(),
                    issued_fencing_epoch: 1,
                    canonical_json: "{\"worker_authorization\":1}".to_owned(),
                },
                worker_attempt_id: object_id(807),
                scheduler_lease: Some(SchedulerLeaseBinding {
                    task_ref: task_ref.to_owned(),
                    contract_epoch: 1,
                    lease_owner: "scheduler-worker".to_owned(),
                    lease_epoch,
                }),
            },
            effect_closure: SchedulerEffectClosure::Closed,
        }
    }

    fn temporary_scheduler_database_path() -> std::path::PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "cognitiveos-scheduler-authority-{}-{unique_suffix}.db",
            std::process::id()
        ))
    }

    fn temporary_personal_layout() -> PersonalDataLayout {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "cognitiveos-server-recovery-{}-{unique_suffix}",
            std::process::id()
        ));
        PersonalDataLayout::from_xdg_roots(
            root.join("config"),
            root.join("data"),
            root.join("state"),
            root.join("cache"),
            root.join("runtime"),
        )
    }

    fn endpoint_document_path(layout: &PersonalDataLayout) -> std::path::PathBuf {
        layout.state_dir().join("daemon-endpoint.json")
    }

    fn wait_for_published_endpoint(layout: &PersonalDataLayout) -> Option<String> {
        let endpoint_path = endpoint_document_path(layout);
        // Recovery tests perform SQLite replay before the server publishes the
        // endpoint. Windows CI can take longer than the original two-second
        // polling window under concurrent workspace test load.
        for _ in 0..300 {
            if let Ok(document) = std::fs::read_to_string(&endpoint_path) {
                let endpoint =
                    serde_json::from_str::<serde_json::Value>(&document).unwrap()["endpoint"]
                        .as_str()
                        .unwrap()
                        .to_owned();
                return Some(endpoint);
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        None
    }

    fn send_health_request_to_once_server(endpoint: &str) {
        let mut stream = TcpStream::connect(endpoint).unwrap();
        stream
            .write_all(
                b"GET /personal/health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
    }

    fn state(value: &str) -> StateName {
        StateName::parse(value).unwrap()
    }

    fn recovery_effect_grant() -> cognitive_kernel::authz::AuthorizationGrant {
        let authorization_time = WallTimestamp::parse("2026-08-04T12:02:00Z").unwrap();
        authorize(
            &AuthzSnapshot {
                tenant_id: "personal-test".to_owned(),
                principal: PrincipalFacts {
                    principal_ref: UriRef::parse("principal://personal/daemon").unwrap(),
                    authenticated: true,
                    active: true,
                    tenant_id: Some("personal-test".to_owned()),
                },
                actor_chain: ActorChainFacts {
                    chain_digest: format!("sha256:{}", "c".repeat(64)),
                    resolved: true,
                },
                membership: Some(MembershipFacts {
                    valid: true,
                    roles: ["daemon".to_owned()].into(),
                }),
                capability_links: vec![CapabilityConstraints {
                    subject: "principal://personal/daemon".to_owned(),
                    audience: "authority://personal/effect-authority".to_owned(),
                    resource: "scope://personal/restart-recovery".to_owned(),
                    purpose: "task_execution".to_owned(),
                    actions: ["filesystem.read".to_owned()].into(),
                    parameter_bounds: BTreeMap::new(),
                    lease: LeaseWindow {
                        not_before: WallTimestamp::parse("2026-08-04T12:00:00Z").unwrap(),
                        expires: WallTimestamp::parse("2026-08-04T12:05:00Z").unwrap(),
                    },
                    depth_remaining: 1,
                    issued_epoch: 1,
                }],
                capability_set_version: 1,
                explicit_denies: Vec::new(),
                revocation_epoch: 1,
                decided_at: authorization_time,
            },
            &ObjectGovernance {
                object_ref: "effect://personal/restart-recovery".to_owned(),
                tenant_id: Some("personal-test".to_owned()),
                owner_ref: "principal://personal/daemon".to_owned(),
                resource_scope: "scope://personal/restart-recovery/effect".to_owned(),
                conversation_ref: None,
            },
            &AccessRequest {
                action: "filesystem.read".to_owned(),
                purpose: "task_execution".to_owned(),
            },
        )
        .unwrap()
    }

    fn reconcile_effect_for_restart_recovery(
        store: &SqliteAuthorityStore,
        effect_object_id: &ObjectId,
    ) {
        let clock = super::FixedSchedulerClock::parse("2026-08-04T12:02:00Z").unwrap();
        let identifiers = UuidV7Generator;
        let effect_protocol = EffectProtocol::new(
            store,
            &clock,
            &identifiers,
            UriRef::parse("actor://personal/daemon").unwrap(),
            UriRef::parse("authority://personal/effect-authority").unwrap(),
            UriRef::parse("correlation://personal/restart-recovery").unwrap(),
        );
        let grant = recovery_effect_grant();
        let currency = GovernanceCurrency {
            revocation_epoch: 1,
            capability_set_version: 1,
        };
        let writer_lease = WriterLease { epoch: 1 };
        let executor = ScriptedExecutor::queryable(1);

        let authorized = effect_protocol
            .authorize_effect(
                effect_object_id,
                Version::INITIAL,
                &grant,
                &currency,
                &writer_lease,
            )
            .unwrap();
        let (dispatched, outcome) = effect_protocol
            .dispatch_effect(
                effect_object_id,
                authorized.after_version,
                &grant,
                &currency,
                &executor,
                &writer_lease,
            )
            .unwrap();
        let executed = effect_protocol
            .record_outcome(
                effect_object_id,
                dispatched.after_version,
                &outcome,
                &writer_lease,
            )
            .unwrap();
        effect_protocol
            .reconcile(
                effect_object_id,
                "EXECUTED",
                executed.after_version,
                &executor,
                &writer_lease,
            )
            .unwrap();
    }

    /// Persist the minimum complete D05 handoff evidence through the normal
    /// store APIs. The Effect intentionally remains PROPOSED: this fixture
    /// exercises restart recovery's retain path without executing a tool or
    /// manufacturing a terminal Effect transition.
    fn persist_pending_bound_handoff(
        database_path: &std::path::Path,
        consume_authorization: bool,
    ) -> (ObjectId, SchedulerWorkKey) {
        let store = SqliteAuthorityStore::open(database_path).unwrap();
        let authorization = sealed_worker_authorization_row();
        let task_ref = authorization.task_ref.clone();
        let scheduler_work_key = SchedulerWorkKey {
            task_ref: task_ref.clone(),
            contract_epoch: authorization.contract_epoch,
        };
        let admitted_at = WallTimestamp::parse("2026-08-04T12:00:00Z").unwrap();

        store
            .insert_task_contract(
                &TaskContractRow {
                    contract_id: object_id(830),
                    task_ref: task_ref.clone(),
                    contract_epoch: authorization.contract_epoch,
                    user_intent_record_id: object_id(831),
                    interpretation_id: object_id(832),
                    accepted_by: "principal://personal/daemon".to_owned(),
                    contract_digest: format!("sha256:{}", "a".repeat(64)),
                    canonical_json: "{\"task_contract\":\"recovery-fixture\"}".to_owned(),
                },
                &EventDraft {
                    event_id: EventId::parse("00000000-0000-7000-a000-000000000830").unwrap(),
                    object_id: object_id(830),
                    domain: LifecycleDomain::Task,
                    object_version: Version::INITIAL,
                    event_type: "task-contract.minted".to_owned(),
                    canonical_json: "{\"event\":\"task-contract\"}".to_owned(),
                },
                0,
            )
            .unwrap();
        store
            .append_operation_candidate_proposal(&OperationCandidateProposalRow {
                candidate_id: authorization.selected_candidate_id.clone(),
                task_ref: task_ref.clone(),
                contract_epoch: authorization.contract_epoch,
                candidate_source_ref: "observation://personal/restart-recovery".to_owned(),
                tool_ref: "operation://personal/filesystem/read".to_owned(),
                action: "filesystem.read".to_owned(),
                target: "file:///workspace/input.txt".to_owned(),
                parameters_digest: format!("sha256:{}", "b".repeat(64)),
                expected_state_version: Version::INITIAL.get(),
                operation_descriptor_ref: object_id(833),
                canonical_json: "{\"candidate\":\"recovery-fixture\"}".to_owned(),
            })
            .unwrap();
        store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: authorization.loop_object_id.clone(),
                    domain: LifecycleDomain::Loop,
                    state: state("DECIDE"),
                    version: authorization.expected_loop_version,
                    body: json!({"fixture": "restart-recovery"}),
                },
                admitted_at: admitted_at.clone(),
                event: EventDraft {
                    event_id: EventId::parse("00000000-0000-7000-a000-000000000821").unwrap(),
                    object_id: authorization.loop_object_id.clone(),
                    domain: LifecycleDomain::Loop,
                    object_version: authorization.expected_loop_version,
                    event_type: "loop.fixture-admitted".to_owned(),
                    canonical_json: "{\"event\":\"loop\"}".to_owned(),
                },
                outbox: Vec::new(),
                fencing_epoch: Some(authorization.issued_fencing_epoch),
            })
            .unwrap();
        let budget_state =
            BudgetState::new(BTreeMap::from([("tool_calls".to_owned(), 2)])).unwrap();
        let budget_state_json = serde_json::to_string(&budget_state).unwrap();
        store
            .create_budget(&authorization.budget_id, &budget_state_json, &admitted_at)
            .unwrap();

        let candidate_admission = CandidateAdmissionCommit {
            selected_candidate_id: authorization.selected_candidate_id.clone(),
            intent: IntentRow {
                intent_id: authorization.intent_id.clone(),
                idempotency_key: "restart-recovery-pending".to_owned(),
                parameters_digest: format!("sha256:{}", "b".repeat(64)),
                action: "filesystem.read".to_owned(),
                target: "file:///workspace/input.txt".to_owned(),
                effect_object_id: authorization.effect_object_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: Some(TaskBinding {
                    task_ref: task_ref.clone(),
                    contract_epoch: authorization.contract_epoch,
                }),
                canonical_json: "{\"intent\":\"restart-recovery\"}".to_owned(),
            },
            intent_event: EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000813").unwrap(),
                object_id: authorization.intent_id.clone(),
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"event\":\"intent\"}".to_owned(),
            },
            effect_admission: ObjectAdmission {
                object: StoredObject {
                    object_id: authorization.effect_object_id.clone(),
                    domain: LifecycleDomain::Effect,
                    state: state("PROPOSED"),
                    version: Version::INITIAL,
                    body: json!({"effect": "restart-recovery"}),
                },
                admitted_at: admitted_at.clone(),
                event: EventDraft {
                    event_id: EventId::parse("00000000-0000-7000-a000-000000000814").unwrap(),
                    object_id: authorization.effect_object_id.clone(),
                    domain: LifecycleDomain::Effect,
                    object_version: Version::INITIAL,
                    event_type: "effect.admitted".to_owned(),
                    canonical_json: "{\"event\":\"effect\"}".to_owned(),
                },
                outbox: Vec::new(),
                fencing_epoch: Some(authorization.issued_fencing_epoch),
            },
            worker_authorization: authorization.clone(),
            loop_transition: TransitionCommit {
                cas: ObjectCas {
                    object_id: authorization.loop_object_id.clone(),
                    domain: LifecycleDomain::Loop,
                    from_state: state("DECIDE"),
                    to_state: state("ACT"),
                    expected_version: authorization.expected_loop_version,
                    next_version: authorization.expected_loop_version.next().unwrap(),
                    committed_at: admitted_at.clone(),
                },
                event: EventDraft {
                    event_id: EventId::parse("00000000-0000-7000-a000-000000000822").unwrap(),
                    object_id: authorization.loop_object_id.clone(),
                    domain: LifecycleDomain::Loop,
                    object_version: authorization.expected_loop_version.next().unwrap(),
                    event_type: "loop.operation-admitted".to_owned(),
                    canonical_json: "{\"event\":\"loop\"}".to_owned(),
                },
                record: RecordDraft {
                    record_id: RecordId::parse("00000000-0000-7000-8000-000000000821").unwrap(),
                    object_id: authorization.loop_object_id.clone(),
                    domain: LifecycleDomain::Loop,
                    object_version: authorization.expected_loop_version.next().unwrap(),
                    canonical_json: "{\"record\":\"loop\"}".to_owned(),
                },
                budget: Some(BudgetCas {
                    budget_id: authorization.budget_id.clone(),
                    expected_version: Version::INITIAL,
                    next_version: Version::INITIAL.next().unwrap(),
                    charge_canonical_json: "{\"tool_calls\":1}".to_owned(),
                    next_state_canonical_json: serde_json::to_string(
                        &BudgetState::new(BTreeMap::from([("tool_calls".to_owned(), 1)])).unwrap(),
                    )
                    .unwrap(),
                }),
                outbox: Vec::new(),
                fencing_epoch: Some(authorization.issued_fencing_epoch),
            },
            fencing_epoch: authorization.issued_fencing_epoch,
        };
        store
            .commit_candidate_admission(&candidate_admission)
            .unwrap();

        let mut scheduler_repository = SchedulerRepository::open(database_path).unwrap();
        scheduler_repository
            .upsert(&scheduler_row(&task_ref))
            .unwrap();
        if consume_authorization {
            let leased_row = scheduler_repository
                .acquire_lease(
                    &scheduler_work_key,
                    "restart-recovery-worker",
                    41,
                    "2026-08-04T12:05:00Z",
                )
                .unwrap();
            let dispatch = SchedulerDispatch {
                task_ref,
                contract_epoch: authorization.contract_epoch,
                lease_owner: leased_row.lease_owner.unwrap(),
                lease_epoch: leased_row.lease_epoch,
                lease_expires: leased_row.lease_expires.unwrap(),
                attempt_count: leased_row.attempt_count,
            };
            super::consume_worker_authorization_for_attempt(
                &store,
                &super::FixedSchedulerClock::parse("2026-08-04T12:01:00Z").unwrap(),
                &authorization.authorization_id,
                object_id(834),
                &dispatch,
            )
            .unwrap();
        }
        drop(scheduler_repository);
        drop(store);
        (authorization.effect_object_id, scheduler_work_key)
    }

    fn committed_ceiling_stop() -> CommittedTransition {
        CommittedTransition {
            record_id: RecordId::parse("00000000-0000-7000-8000-000000000001").unwrap(),
            event_id: EventId::parse("00000000-0000-7000-8000-000000000002").unwrap(),
            event_sequence: 1,
            after_version: Version::new(2).unwrap(),
            committed_at: WallTimestamp::parse("2026-08-02T00:00:00Z").unwrap(),
        }
    }

    fn task_binding() -> TaskBinding {
        TaskBinding {
            task_ref: "task://personal/durable-effect-resolution".to_owned(),
            contract_epoch: 4,
        }
    }

    fn effect_intent(intent_suffix: u64, binding: Option<TaskBinding>) -> IntentRow {
        IntentRow {
            intent_id: ObjectId::parse(&format!("00000000-0000-7000-8000-{intent_suffix:012x}"))
                .unwrap(),
            idempotency_key: format!("scheduler-effect-{intent_suffix}"),
            parameters_digest: format!("sha256:{}", "ab".repeat(32)),
            action: "scheduler.effect".to_owned(),
            target: "effect://personal/scheduler".to_owned(),
            effect_object_id: ObjectId::parse(&format!(
                "00000000-0000-7000-9000-{intent_suffix:012x}"
            ))
            .unwrap(),
            expected_state_version: Version::INITIAL,
            grant_epoch: 1,
            capability_set_version: 1,
            task_binding: binding,
            canonical_json: "{}".to_owned(),
        }
    }

    #[test]
    fn restarted_recovery_retains_a_pending_effects_exact_bound_lease() {
        let database_path = temporary_scheduler_database_path();
        let (effect_object_id, scheduler_work_key) =
            persist_pending_bound_handoff(&database_path, true);

        let reopened_store = SqliteAuthorityStore::open(&database_path).unwrap();
        let mut reopened_scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let recovered_attempts = super::reconcile_recovered_worker_attempts(
            &reopened_store,
            &mut reopened_scheduler_repository,
            &super::FixedSchedulerClock::parse("2026-08-04T12:02:00Z").unwrap(),
        )
        .unwrap();

        assert_eq!(recovered_attempts.len(), 1);
        assert_eq!(
            recovered_attempts[0].effect_closure,
            SchedulerEffectClosure::PendingReconciliation
        );
        assert_eq!(
            recovered_attempts[0]
                .handoff
                .scheduler_lease
                .as_ref()
                .unwrap()
                .lease_epoch,
            41
        );
        assert_eq!(
            reopened_store
                .load_object(LifecycleDomain::Effect, &effect_object_id)
                .unwrap()
                .unwrap()
                .state
                .as_str(),
            "PROPOSED"
        );

        let scheduler_row = reopened_scheduler_repository
            .load(&scheduler_work_key)
            .unwrap()
            .unwrap();
        assert_eq!(scheduler_row.state, SchedulerState::Leased.as_str());
        assert_eq!(
            scheduler_row.lease_owner.as_deref(),
            Some("restart-recovery-worker")
        );
        assert_eq!(scheduler_row.lease_epoch, 41);
        assert_eq!(scheduler_row.attempt_count, 1);

        drop(reopened_scheduler_repository);
        drop(reopened_store);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn restarted_recovery_releases_only_a_reconciled_effects_exact_bound_lease() {
        let database_path = temporary_scheduler_database_path();
        let (effect_object_id, scheduler_work_key) =
            persist_pending_bound_handoff(&database_path, true);

        let closing_store = SqliteAuthorityStore::open(&database_path).unwrap();
        reconcile_effect_for_restart_recovery(&closing_store, &effect_object_id);
        let reconciled_effect = closing_store
            .load_object(LifecycleDomain::Effect, &effect_object_id)
            .unwrap()
            .unwrap();
        assert_eq!(reconciled_effect.state.as_str(), "RECONCILED");
        assert_eq!(reconciled_effect.version, Version::new(5).unwrap());
        drop(closing_store);

        let reopened_store = SqliteAuthorityStore::open(&database_path).unwrap();
        let mut reopened_scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let recovered_attempts = super::reconcile_recovered_worker_attempts(
            &reopened_store,
            &mut reopened_scheduler_repository,
            &super::FixedSchedulerClock::parse("2026-08-04T12:03:00Z").unwrap(),
        )
        .unwrap();

        assert_eq!(recovered_attempts.len(), 1);
        assert_eq!(
            recovered_attempts[0].effect_closure,
            SchedulerEffectClosure::Closed
        );
        let recovered_lease = recovered_attempts[0]
            .handoff
            .scheduler_lease
            .as_ref()
            .unwrap();
        assert_eq!(recovered_lease.lease_owner, "restart-recovery-worker");
        assert_eq!(recovered_lease.lease_epoch, 41);

        let scheduler_row = reopened_scheduler_repository
            .load(&scheduler_work_key)
            .unwrap()
            .unwrap();
        assert_eq!(scheduler_row.state, SchedulerState::Succeeded.as_str());
        assert_eq!(scheduler_row.lease_owner, None);
        assert_eq!(scheduler_row.lease_expires, None);
        assert_eq!(scheduler_row.lease_epoch, 41);
        assert_eq!(scheduler_row.attempt_count, 1);

        drop(reopened_scheduler_repository);
        drop(reopened_store);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn server_startup_recovers_closed_effect_before_publishing_endpoint() {
        let layout = temporary_personal_layout();
        layout.ensure_directories().unwrap();
        let authority_database_path = layout.authority_database_path();
        let (effect_object_id, scheduler_work_key) =
            persist_pending_bound_handoff(&authority_database_path, true);
        let closing_store = SqliteAuthorityStore::open(&authority_database_path).unwrap();
        reconcile_effect_for_restart_recovery(&closing_store, &effect_object_id);
        drop(closing_store);

        let (result_sender, result_receiver) = mpsc::channel();
        let server_layout = layout.clone();
        let server_thread = std::thread::spawn(move || {
            let result = super::super::server::serve_personal_loopback(
                super::super::server::PersonalDaemonConfig {
                    bind_address: "127.0.0.1:0".to_owned(),
                    layout: server_layout,
                    bounds: super::super::bounds::PersonalResourceBounds::personal_v1_baseline(),
                    once: true,
                },
            );
            result_sender.send(result).unwrap();
        });

        let endpoint = wait_for_published_endpoint(&layout);
        assert!(
            endpoint.is_some(),
            "server did not publish its endpoint document"
        );
        let endpoint = endpoint.unwrap();
        send_health_request_to_once_server(&endpoint);
        assert!(result_receiver.recv().unwrap().is_ok());
        server_thread.join().unwrap();

        let mut scheduler_repository = SchedulerRepository::open(&authority_database_path).unwrap();
        let scheduler_row = scheduler_repository
            .load(&scheduler_work_key)
            .unwrap()
            .unwrap();
        assert_eq!(scheduler_row.state, SchedulerState::Succeeded.as_str());
        assert_eq!(scheduler_row.lease_owner, None);
        assert_eq!(scheduler_row.lease_expires, None);
        assert_eq!(scheduler_row.lease_epoch, 41);
        assert_eq!(scheduler_row.attempt_count, 1);
        assert!(!endpoint_document_path(&layout).exists());

        drop(scheduler_repository);
        std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn server_startup_recovery_stale_contract_does_not_publish_endpoint() {
        let layout = temporary_personal_layout();
        layout.ensure_directories().unwrap();
        let authority_database_path = layout.authority_database_path();
        let (_, scheduler_work_key) = persist_pending_bound_handoff(&authority_database_path, true);
        let store = SqliteAuthorityStore::open(&authority_database_path).unwrap();
        store
            .insert_task_contract(
                &TaskContractRow {
                    contract_id: object_id(840),
                    task_ref: scheduler_work_key.task_ref,
                    contract_epoch: 2,
                    user_intent_record_id: object_id(841),
                    interpretation_id: object_id(842),
                    accepted_by: "principal://personal/daemon".to_owned(),
                    contract_digest: format!("sha256:{}", "d".repeat(64)),
                    canonical_json: "{\"task_contract\":\"superseding-fixture\"}".to_owned(),
                },
                &EventDraft {
                    event_id: EventId::parse("00000000-0000-7000-a000-000000000840").unwrap(),
                    object_id: object_id(840),
                    domain: LifecycleDomain::Task,
                    object_version: Version::INITIAL,
                    event_type: "task-contract.superseded".to_owned(),
                    canonical_json: "{\"event\":\"task-contract\"}".to_owned(),
                },
                1,
            )
            .unwrap();
        drop(store);

        let result = super::super::server::serve_personal_loopback(
            super::super::server::PersonalDaemonConfig {
                bind_address: "127.0.0.1:0".to_owned(),
                layout: layout.clone(),
                bounds: super::super::bounds::PersonalResourceBounds::personal_v1_baseline(),
                once: true,
            },
        );

        assert!(matches!(
            result,
            Err(super::super::server::PersonalDaemonError::Io { detail })
                if detail.contains("reconcile durable scheduler recovery before startup")
        ));
        assert!(!endpoint_document_path(&layout).exists());
        assert!(!layout.daemon_lock_path().exists());

        std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn private_scheduler_tick_rejects_unreadable_current_contract_before_wia_handoff() {
        let database_path = temporary_scheduler_database_path();
        let (_, scheduler_work_key) = persist_pending_bound_handoff(&database_path, false);
        let store = SqliteAuthorityStore::open(&database_path).unwrap();
        store
            .insert_task_contract(
                &TaskContractRow {
                    contract_id: object_id(850),
                    task_ref: scheduler_work_key.task_ref.clone(),
                    contract_epoch: 2,
                    user_intent_record_id: object_id(851),
                    interpretation_id: object_id(852),
                    accepted_by: "principal://personal/daemon".to_owned(),
                    contract_digest: format!("sha256:{}", "e".repeat(64)),
                    canonical_json: "{\"task_contract\":\"superseding-tick-fixture\"}".to_owned(),
                },
                &EventDraft {
                    event_id: EventId::parse("00000000-0000-7000-a000-000000000850").unwrap(),
                    object_id: object_id(850),
                    domain: LifecycleDomain::Task,
                    object_version: Version::INITIAL,
                    event_type: "task-contract.superseded".to_owned(),
                    canonical_json: "{\"event\":\"task-contract\"}".to_owned(),
                },
                1,
            )
            .unwrap();
        drop(store);

        // The scheduler fails closed before the handoff. The exact rejected
        // authority read is deliberately not part of this safety boundary.
        assert!(super::run_private_scheduler_tick(&database_path).is_err());

        let reopened_store = SqliteAuthorityStore::open(&database_path).unwrap();
        assert!(
            reopened_store
                .list_consumed_worker_iteration_authorizations()
                .unwrap()
                .is_empty()
        );
        let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let scheduler_row = scheduler_repository
            .load(&scheduler_work_key)
            .unwrap()
            .unwrap();
        assert_eq!(scheduler_row.state, SchedulerState::Runnable.as_str());
        assert_eq!(scheduler_row.attempt_count, 0);
        assert_eq!(scheduler_row.lease_owner, None);

        drop(scheduler_repository);
        drop(reopened_store);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn legacy_contract_is_rejected_before_execution_binding_deserialization() {
        let legacy_contract = r#"{
            "header": {
                "schema_version": "cognitiveos.task-contract/0.1"
            }
        }"#;

        assert!(matches!(parse_execution_bound_contract(legacy_contract),
            Err(SchedulerAuthorityError::LegacyContract(version))
            if version == "cognitiveos.task-contract/0.1"));
    }

    #[test]
    fn execution_schema_without_required_bindings_is_rejected_as_malformed() {
        let incomplete_execution_contract = r#"{
            "header": {
                "schema_version": "cognitiveos.task-contract/0.3"
            }
        }"#;

        assert!(matches!(
            parse_execution_bound_contract(incomplete_execution_contract),
            Err(SchedulerAuthorityError::MalformedContract(_))
        ));
    }

    #[test]
    fn context_bound_execution_schema_is_not_rejected_as_legacy() {
        let incomplete_context_bound_contract = r#"{
            "header": {
                "schema_version": "cognitiveos.task-contract/0.4"
            }
        }"#;

        assert!(matches!(
            parse_execution_bound_contract(incomplete_context_bound_contract),
            Err(SchedulerAuthorityError::MalformedContract(_))
        ));
    }

    #[test]
    fn private_pi_candidate_rejects_invalid_non_authority_fields() {
        let invalid_digest_candidate = UntrustedPiCandidate {
            tool_ref: "operation://personal/filesystem/read".to_owned(),
            action: "filesystem.read".to_owned(),
            target: "file:///workspace/input.txt".to_owned(),
            parameters_digest: "not-a-digest".to_owned(),
            expected_state_version: 1,
            operation_descriptor_id: object_id(990),
        };
        assert!(matches!(
            validate_untrusted_pi_candidate(&invalid_digest_candidate),
            Err(SchedulerAuthorityError::PrivatePiProposal(_))
        ));

        let invalid_version_candidate = UntrustedPiCandidate {
            parameters_digest: format!("sha256:{}", "a".repeat(64)),
            expected_state_version: 0,
            ..invalid_digest_candidate
        };
        assert!(matches!(
            validate_untrusted_pi_candidate(&invalid_version_candidate),
            Err(SchedulerAuthorityError::PrivatePiProposal(_))
        ));
    }

    #[test]
    fn stale_contract_epoch_is_rejected_before_scheduler_admission() {
        let binding = SchedulerAuthorityBinding {
            task_ref: "task://personal/superseded-contract".to_owned(),
            contract_epoch: 4,
            action_fingerprint: "scheduler.effect:sha256:test".to_owned(),
        };

        assert!(matches!(
            ensure_current_contract_epoch(&binding, 5),
            Err(SchedulerAuthorityError::StaleContractEpoch {
                task_ref,
                requested_epoch: 4,
                current_epoch: 5,
            }) if task_ref == binding.task_ref
        ));
    }

    #[test]
    fn sealed_wia_evidence_rejects_budget_charge_and_loop_version_row_mismatches() {
        let matching_row = sealed_worker_authorization_row();
        assert!(
            validate_worker_authorization_evidence(&matching_row).is_ok(),
            "a row derived from its sealed WIA payload must validate"
        );

        let mut charge_mismatch = matching_row.clone();
        charge_mismatch.budget_charge_canonical_json = "{\"tool_calls\":2}".to_owned();
        assert!(matches!(
            validate_worker_authorization_evidence(&charge_mismatch),
            Err(SchedulerAuthorityError::CandidateAdmissionComposition(_))
        ));

        let mut loop_version_mismatch = matching_row;
        loop_version_mismatch.expected_loop_version = Version::new(2).unwrap();
        assert!(matches!(
            validate_worker_authorization_evidence(&loop_version_mismatch),
            Err(SchedulerAuthorityError::CandidateAdmissionComposition(_))
        ));
    }

    #[test]
    fn effect_resolution_rejects_missing_ambiguous_and_inconsistent_bindings() {
        let binding = task_binding();

        assert!(matches!(
            select_single_effect_intent(&binding, &[]),
            Err(SchedulerAuthorityError::MissingEffectBinding {
                task_ref,
                contract_epoch: 4,
            }) if task_ref == binding.task_ref
        ));

        let first_intent = effect_intent(11, Some(binding.clone()));
        let second_intent = effect_intent(12, Some(binding.clone()));
        assert!(matches!(
            select_single_effect_intent(&binding, &[first_intent, second_intent]),
            Err(SchedulerAuthorityError::AmbiguousEffectBindings {
                task_ref,
                contract_epoch: 4,
            }) if task_ref == binding.task_ref
        ));

        let inconsistent_intent = effect_intent(
            13,
            Some(TaskBinding {
                task_ref: binding.task_ref.clone(),
                contract_epoch: 5,
            }),
        );
        assert!(matches!(
            select_single_effect_intent(&binding, &[inconsistent_intent]),
            Err(SchedulerAuthorityError::InconsistentEffectBinding(_))
        ));
    }

    #[test]
    fn reached_ceiling_returns_durable_stop_without_attempting_a_scheduler_lease() {
        let lease_acquisition_attempted = Cell::new(false);

        let admission = complete_scheduler_admission(
            SchedulerCeilingDispatch::Stopped(committed_ceiling_stop()),
            || {
                lease_acquisition_attempted.set(true);
                unreachable!("a reached ceiling must not acquire a scheduler lease")
            },
        )
        .unwrap();

        assert!(matches!(admission, SchedulerDispatchAdmission::Stopped(_)));
        assert!(
            !lease_acquisition_attempted.get(),
            "a terminal ceiling STOP must precede every lease attempt"
        );
    }

    #[test]
    fn clear_ceiling_acquires_exactly_one_scheduler_lease() {
        let lease_acquisition_count = Cell::new(0);

        let admission = complete_scheduler_admission(SchedulerCeilingDispatch::Proceed, || {
            lease_acquisition_count.set(lease_acquisition_count.get() + 1);
            Ok(SchedulerDispatch {
                task_ref: "task://personal/admission-order".to_owned(),
                contract_epoch: 1,
                lease_owner: "scheduler-worker".to_owned(),
                lease_epoch: 3,
                lease_expires: "2026-08-02T00:01:00Z".to_owned(),
                attempt_count: 1,
            })
        })
        .unwrap();

        assert!(matches!(admission, SchedulerDispatchAdmission::Leased(_)));
        assert_eq!(lease_acquisition_count.get(), 1);
    }

    #[test]
    fn ceiling_stop_skips_the_effect_closure_callback() {
        let effect_closure_attempted = Cell::new(false);

        let attempt = complete_scheduler_worker_attempt(
            SchedulerDispatchAdmission::Stopped(committed_ceiling_stop()),
            |_| {
                effect_closure_attempted.set(true);
                unreachable!("a stopped scheduler attempt must not process an Effect")
            },
        )
        .unwrap();

        assert!(matches!(attempt, SchedulerWorkerAttempt::Stopped(_)));
        assert!(
            !effect_closure_attempted.get(),
            "a durable ceiling STOP must precede every Effect-closure callback"
        );
    }

    #[test]
    fn unresolved_effect_keeps_the_fenced_dispatch_for_reconciliation() {
        let dispatch = SchedulerDispatch {
            task_ref: "task://personal/effect-reconciliation".to_owned(),
            contract_epoch: 1,
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 7,
            lease_expires: "2026-08-02T00:01:00Z".to_owned(),
            attempt_count: 1,
        };

        let attempt = complete_scheduler_worker_attempt(
            SchedulerDispatchAdmission::Leased(dispatch.clone()),
            |received_dispatch| {
                assert_eq!(received_dispatch, dispatch);
                Ok(SchedulerEffectClosure::PendingReconciliation)
            },
        )
        .unwrap();

        assert_eq!(
            attempt,
            SchedulerWorkerAttempt::AwaitingReconciliation(dispatch),
            "an unresolved Effect must not be converted into a scheduler success"
        );
    }

    #[test]
    fn only_a_closed_effect_can_release_the_exact_fenced_scheduler_dispatch() {
        let dispatch = SchedulerDispatch {
            task_ref: "task://personal/closed-effect".to_owned(),
            contract_epoch: 1,
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 8,
            lease_expires: "2026-08-02T00:01:00Z".to_owned(),
            attempt_count: 1,
        };
        let release_count = Cell::new(0);

        let released = release_closed_effect_dispatch(
            SchedulerWorkerAttempt::EffectClosed(dispatch.clone()),
            |received_dispatch| {
                release_count.set(release_count.get() + 1);
                assert_eq!(received_dispatch, dispatch);
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(released, SchedulerWorkerAttempt::EffectClosed(dispatch));
        assert_eq!(release_count.get(), 1);
    }

    #[test]
    fn pending_effect_reconciliation_does_not_release_its_scheduler_lease() {
        let dispatch = SchedulerDispatch {
            task_ref: "task://personal/pending-effect".to_owned(),
            contract_epoch: 1,
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 9,
            lease_expires: "2026-08-02T00:01:00Z".to_owned(),
            attempt_count: 1,
        };
        let release_attempted = Cell::new(false);

        let retained = release_closed_effect_dispatch(
            SchedulerWorkerAttempt::AwaitingReconciliation(dispatch.clone()),
            |_| {
                release_attempted.set(true);
                unreachable!("a pending Effect must retain its fenced scheduler lease")
            },
        )
        .unwrap();

        assert_eq!(
            retained,
            SchedulerWorkerAttempt::AwaitingReconciliation(dispatch)
        );
        assert!(
            !release_attempted.get(),
            "a pending Effect must not release its scheduler lease"
        );
    }

    #[test]
    fn closed_effect_releases_the_matching_durable_lease_without_completing_the_task() {
        let database_path = temporary_scheduler_database_path();
        let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let task_ref = "task://personal/durable-effect-closure";
        scheduler_repository
            .upsert(&scheduler_row(task_ref))
            .unwrap();
        scheduler_repository
            .acquire_lease(
                &scheduler_work_key(task_ref),
                "scheduler-worker",
                10,
                "2026-08-03T00:00:00Z",
            )
            .unwrap();
        let dispatch = SchedulerDispatch {
            task_ref: task_ref.to_owned(),
            contract_epoch: 1,
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 10,
            lease_expires: "2026-08-03T00:01:00Z".to_owned(),
            attempt_count: 1,
        };

        let completed_attempt = complete_resolved_effect_and_release(
            SchedulerWorkerAttempt::EffectClosed(dispatch.clone()),
            &mut scheduler_repository,
            "2026-08-03T00:00:30Z",
        )
        .unwrap();

        assert_eq!(
            completed_attempt,
            SchedulerWorkerAttempt::EffectClosed(dispatch),
            "a closed Effect ends this scheduler attempt, not Task acceptance"
        );
        let durable_row = scheduler_repository
            .load(&scheduler_work_key(task_ref))
            .unwrap()
            .unwrap();
        assert_eq!(durable_row.state, SchedulerState::Succeeded.as_str());
        assert_eq!(durable_row.lease_owner, None);
        drop(scheduler_repository);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn recovered_closed_effect_releases_only_its_persisted_owner_and_epoch_lease() {
        let database_path = temporary_scheduler_database_path();
        let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let task_ref = "task://personal/recovered-exact-lease";
        scheduler_repository
            .upsert(&scheduler_row(task_ref))
            .unwrap();
        scheduler_repository
            .acquire_lease(
                &scheduler_work_key(task_ref),
                "scheduler-worker",
                21,
                "2026-08-04T12:05:00Z",
            )
            .unwrap();

        release_closed_recovered_attempt(
            &recovered_closed_attempt(task_ref, 21),
            &mut scheduler_repository,
            "2026-08-04T12:01:00Z",
        )
        .unwrap();

        let row = scheduler_repository
            .load(&scheduler_work_key(task_ref))
            .unwrap()
            .unwrap();
        assert_eq!(row.state, SchedulerState::Succeeded.as_str());
        assert_eq!(row.lease_owner, None);
        drop(scheduler_repository);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn recovered_legacy_unbound_handoff_retains_its_scheduler_lease() {
        let database_path = temporary_scheduler_database_path();
        let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let task_ref = "task://personal/recovered-legacy-unbound";
        scheduler_repository
            .upsert(&scheduler_row(task_ref))
            .unwrap();
        scheduler_repository
            .acquire_lease(
                &scheduler_work_key(task_ref),
                "scheduler-worker",
                23,
                "2026-08-04T12:05:00Z",
            )
            .unwrap();

        let mut recovered_attempt = recovered_closed_attempt(task_ref, 23);
        recovered_attempt.handoff.scheduler_lease = None;
        release_closed_recovered_attempt(
            &recovered_attempt,
            &mut scheduler_repository,
            "2026-08-04T12:01:00Z",
        )
        .unwrap();

        let row = scheduler_repository
            .load(&scheduler_work_key(task_ref))
            .unwrap()
            .unwrap();
        assert_eq!(row.state, SchedulerState::Leased.as_str());
        assert_eq!(row.lease_epoch, 23);
        assert_eq!(row.lease_owner.as_deref(), Some("scheduler-worker"));
        drop(scheduler_repository);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn recovered_closed_effect_cannot_release_a_successor_lease_epoch() {
        let database_path = temporary_scheduler_database_path();
        let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let task_ref = "task://personal/recovered-stale-lease";
        scheduler_repository
            .upsert(&scheduler_row(task_ref))
            .unwrap();
        scheduler_repository
            .acquire_lease(
                &scheduler_work_key(task_ref),
                "scheduler-worker",
                22,
                "2026-08-04T12:05:00Z",
            )
            .unwrap();

        let result = release_closed_recovered_attempt(
            &recovered_closed_attempt(task_ref, 21),
            &mut scheduler_repository,
            "2026-08-04T12:01:00Z",
        );
        assert!(
            result.is_err(),
            "a recovered handoff cannot release a successor lease epoch"
        );
        let Err(error) = result else {
            return;
        };
        assert!(matches!(error, SchedulerAuthorityError::Repository(_)));
        let row = scheduler_repository
            .load(&scheduler_work_key(task_ref))
            .unwrap()
            .unwrap();
        assert_eq!(row.state, SchedulerState::Leased.as_str());
        assert_eq!(row.lease_epoch, 22);
        assert_eq!(row.lease_owner.as_deref(), Some("scheduler-worker"));
        drop(scheduler_repository);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn malformed_release_time_preserves_the_closed_effects_fenced_lease() {
        let database_path = temporary_scheduler_database_path();
        let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let task_ref = "task://personal/malformed-release-time";
        scheduler_repository
            .upsert(&scheduler_row(task_ref))
            .unwrap();
        scheduler_repository
            .acquire_lease(
                &scheduler_work_key(task_ref),
                "scheduler-worker",
                11,
                "2026-08-03T00:00:00Z",
            )
            .unwrap();
        let dispatch = SchedulerDispatch {
            task_ref: task_ref.to_owned(),
            contract_epoch: 1,
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 11,
            lease_expires: "2026-08-03T00:01:00Z".to_owned(),
            attempt_count: 1,
        };

        assert!(matches!(
            complete_resolved_effect_and_release(
                SchedulerWorkerAttempt::EffectClosed(dispatch),
                &mut scheduler_repository,
                "not-a-timestamp",
            ),
            Err(SchedulerAuthorityError::InvalidReleaseTime(value)) if value == "not-a-timestamp"
        ));

        let durable_row = scheduler_repository
            .load(&scheduler_work_key(task_ref))
            .unwrap()
            .unwrap();
        assert_eq!(durable_row.state, SchedulerState::Leased.as_str());
        assert_eq!(durable_row.lease_owner.as_deref(), Some("scheduler-worker"));
        assert_eq!(durable_row.lease_epoch, 11);
        drop(scheduler_repository);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn stale_closed_effect_release_preserves_a_successor_fenced_lease() {
        let database_path = temporary_scheduler_database_path();
        let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let task_ref = "task://personal/stale-closed-effect";
        scheduler_repository
            .upsert(&scheduler_row(task_ref))
            .unwrap();
        scheduler_repository
            .acquire_eligible_lease(
                &scheduler_work_key(task_ref),
                "scheduler-worker",
                12,
                "2026-08-03T00:00:00Z",
                "2026-08-03T00:00:30Z",
            )
            .unwrap();
        scheduler_repository
            .acquire_eligible_lease(
                &scheduler_work_key(task_ref),
                "scheduler-worker",
                13,
                "2026-08-03T00:00:30Z",
                "2026-08-03T00:01:30Z",
            )
            .unwrap();
        let stale_dispatch = SchedulerDispatch {
            task_ref: task_ref.to_owned(),
            contract_epoch: 1,
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 12,
            lease_expires: "2026-08-03T00:01:00Z".to_owned(),
            attempt_count: 1,
        };

        assert!(matches!(
            complete_resolved_effect_and_release(
                SchedulerWorkerAttempt::EffectClosed(stale_dispatch),
                &mut scheduler_repository,
                "2026-08-03T00:01:30Z",
            ),
            Err(SchedulerAuthorityError::Repository(_))
        ));

        let durable_row = scheduler_repository
            .load(&scheduler_work_key(task_ref))
            .unwrap()
            .unwrap();
        assert_eq!(durable_row.state, SchedulerState::Leased.as_str());
        assert_eq!(durable_row.lease_owner.as_deref(), Some("scheduler-worker"));
        assert_eq!(durable_row.lease_epoch, 13);
        drop(scheduler_repository);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn only_durable_terminal_effect_states_close_a_scheduler_attempt() {
        assert_eq!(
            classify_scheduler_effect_closure("RECONCILED").unwrap(),
            SchedulerEffectClosure::Closed
        );
        assert_eq!(
            classify_scheduler_effect_closure("EXECUTING").unwrap(),
            SchedulerEffectClosure::PendingReconciliation
        );
        assert!(matches!(
            classify_scheduler_effect_closure("UNRECOGNIZED"),
            Err(SchedulerAuthorityError::UnsupportedEffectState(state)) if state == "UNRECOGNIZED"
        ));
    }
}

/// Resolve the exact durable Effect bound to one scheduler TaskContract epoch.
///
/// This is deliberately a read-only authority boundary. It rejects zero or
/// multiple bindings, missing objects, adapter-inconsistent rows, and unknown
/// states before any worker can turn a process result into a scheduler outcome.
pub(crate) fn resolve_scheduler_effect_for_task_binding<S>(
    store: &S,
    task_binding: &TaskBinding,
) -> Result<SchedulerEffectResolution, SchedulerAuthorityError>
where
    S: AuthorityStore + ProtocolStore,
{
    if task_binding.task_ref.is_empty() {
        return Err(SchedulerAuthorityError::EmptyTaskReference);
    }
    if task_binding.contract_epoch <= 0 {
        return Err(SchedulerAuthorityError::InvalidContractEpoch(
            task_binding.contract_epoch,
        ));
    }

    let intent_rows = store
        .list_intents_for_task_binding(task_binding)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let intent_row = select_single_effect_intent(task_binding, &intent_rows)?;

    let effect_object = store
        .load_object(LifecycleDomain::Effect, &intent_row.effect_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::MissingEffect(intent_row.effect_object_id.as_str().to_owned())
        })?;
    let closure = classify_scheduler_effect_closure(effect_object.state.as_str())?;

    Ok(SchedulerEffectResolution {
        effect_object_id: intent_row.effect_object_id.clone(),
        closure,
    })
}

/// Reconstruct the sole dispatchable TaskBinding from durable task work.
///
/// A scheduler row does not carry a mutable copy of contract or action
/// identity. Each worker tick instead reads the current immutable contract
/// epoch and requires exactly one matching persisted Intent. Missing,
/// ambiguous, or internally inconsistent binding rows fail before lease
/// acquisition, so recovery cannot guess which Effect a task should drive.
pub(crate) fn resolve_scheduler_work_for_task<S>(
    store: &S,
    task_ref: &str,
) -> Result<ResolvedSchedulerWork, SchedulerAuthorityError>
where
    S: IntentChainStore + ProtocolStore,
{
    if task_ref.is_empty() {
        return Err(SchedulerAuthorityError::EmptyTaskReference);
    }
    let contract_epoch = store
        .current_contract_epoch(task_ref)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    if contract_epoch <= 0 {
        return Err(SchedulerAuthorityError::MissingContract(
            task_ref.to_owned(),
        ));
    }
    let task_binding = TaskBinding {
        task_ref: task_ref.to_owned(),
        contract_epoch,
    };
    let intent_rows = store
        .list_intents_for_task_binding(&task_binding)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let intent_row = select_single_effect_intent(&task_binding, &intent_rows)?;
    let action_fingerprint = format!("{}:{}", intent_row.action, intent_row.parameters_digest);
    Ok(ResolvedSchedulerWork {
        authority_binding: SchedulerAuthorityBinding {
            task_ref: task_ref.to_owned(),
            contract_epoch,
            action_fingerprint,
        },
        task_binding,
    })
}

/// Select exactly one immutable Intent and verify that its stored binding
/// agrees with the reverse-index query used to find it.
fn select_single_effect_intent<'intent>(
    task_binding: &TaskBinding,
    intent_rows: &'intent [cognitive_kernel::ports::IntentRow],
) -> Result<&'intent cognitive_kernel::ports::IntentRow, SchedulerAuthorityError> {
    let intent_row = match intent_rows {
        [] => {
            return Err(SchedulerAuthorityError::MissingEffectBinding {
                task_ref: task_binding.task_ref.clone(),
                contract_epoch: task_binding.contract_epoch,
            });
        }
        [intent_row] => intent_row,
        _ => {
            return Err(SchedulerAuthorityError::AmbiguousEffectBindings {
                task_ref: task_binding.task_ref.clone(),
                contract_epoch: task_binding.contract_epoch,
            });
        }
    };
    if intent_row.task_binding.as_ref() != Some(task_binding) {
        return Err(SchedulerAuthorityError::InconsistentEffectBinding(
            intent_row.intent_id.as_str().to_owned(),
        ));
    }

    Ok(intent_row)
}

/// Classify an Effect only from its durable lifecycle state.
///
/// The states accepted as closed mirror the fail-closed checkpoint inventory:
/// reconciliation or verification has reached a terminal disposition. Every
/// in-flight state retains the fenced dispatch for reconciliation; unknown
/// values are rejected rather than treated as a successful closure.
fn classify_scheduler_effect_closure(
    state: &str,
) -> Result<SchedulerEffectClosure, SchedulerAuthorityError> {
    match state {
        "RECONCILED" | "VERIFIED" | "VERIFY_FAILED" => Ok(SchedulerEffectClosure::Closed),
        "PROPOSED" | "AUTHORIZED" | "EXECUTING" | "OUTCOME_UNKNOWN" | "EXECUTED"
        | "COMPENSATING" | "QUARANTINED" => Ok(SchedulerEffectClosure::PendingReconciliation),
        _ => Err(SchedulerAuthorityError::UnsupportedEffectState(
            state.to_owned(),
        )),
    }
}

/// Reload durable facts that must precede a scheduler dispatch decision.
pub(crate) fn load_scheduler_ceiling_facts<S>(
    store: &S,
    binding: &SchedulerAuthorityBinding,
) -> Result<SchedulerCeilingFacts, SchedulerAuthorityError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore,
{
    Ok(load_scheduler_authority_snapshot(store, binding)?.ceiling_facts)
}

/// Reload the full durable input set required before scheduler admission.
pub(crate) fn load_scheduler_authority_snapshot<S>(
    store: &S,
    binding: &SchedulerAuthorityBinding,
) -> Result<SchedulerAuthoritySnapshot, SchedulerAuthorityError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore,
{
    if binding.task_ref.is_empty() {
        return Err(SchedulerAuthorityError::EmptyTaskReference);
    }
    if binding.action_fingerprint.is_empty() {
        return Err(SchedulerAuthorityError::EmptyActionFingerprint);
    }

    let contract_epoch = store
        .current_contract_epoch(&binding.task_ref)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    if contract_epoch == 0 {
        return Err(SchedulerAuthorityError::MissingContract(
            binding.task_ref.clone(),
        ));
    }
    ensure_current_contract_epoch(binding, contract_epoch)?;
    let contract_row = store
        .load_task_contract(&binding.task_ref, contract_epoch)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::MissingContract(binding.task_ref.clone()))?;
    let contract = parse_execution_bound_contract(&contract_row.canonical_json)?;

    let deadline = contract.deadline.ok_or_else(|| {
        SchedulerAuthorityError::MalformedContract("v0.2 contract has no deadline".to_owned())
    })?;
    let loop_object_id = ObjectId::parse(
        &contract
            .loop_object_id
            .ok_or_else(|| {
                SchedulerAuthorityError::MalformedContract(
                    "v0.2 contract has no loop object identity".to_owned(),
                )
            })?
            .0,
    )
    .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    let budget_id = BudgetId::parse(
        &contract
            .budget_id
            .ok_or_else(|| {
                SchedulerAuthorityError::MalformedContract(
                    "v0.2 contract has no budget identity".to_owned(),
                )
            })?
            .0,
    )
    .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    let loop_object = store
        .load_object(LifecycleDomain::Loop, &loop_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::LoopUnavailable(loop_object_id.as_str().to_owned())
        })?;
    if !matches!(loop_object.state.as_str(), "START" | "CONTINUE") {
        return Err(SchedulerAuthorityError::LoopUnavailable(format!(
            "{} is {}",
            loop_object_id.as_str(),
            loop_object.state.as_str()
        )));
    }

    let progress_facts = store
        .list_progress_facts(&loop_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let retry_count = progress_facts
        .iter()
        .filter(|fact| {
            fact.action_fingerprint == binding.action_fingerprint && fact.status != "advanced"
        })
        .count() as i64;
    let completed_steps = progress_facts.len() as i64;
    let stored_budget = store
        .load_budget(&budget_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::BudgetUnavailable(budget_id.as_str().to_owned()))?;
    let configured_cost = contract.budget.money_microunits;
    let (spent_cost_microunits, cost_ceiling_microunits) = match configured_cost {
        Some(cost_ceiling) => {
            let remaining_cost = stored_budget
                .state
                .remaining()
                .get("money_microunits")
                .copied()
                .ok_or_else(|| {
                    SchedulerAuthorityError::BudgetUnavailable(budget_id.as_str().to_owned())
                })?;
            if remaining_cost > cost_ceiling {
                return Err(SchedulerAuthorityError::BudgetUnavailable(
                    budget_id.as_str().to_owned(),
                ));
            }
            (cost_ceiling - remaining_cost, cost_ceiling)
        }
        None => (0, i64::MAX),
    };

    Ok(SchedulerAuthoritySnapshot {
        ceiling_facts: SchedulerCeilingFacts {
            deadline: Some(deadline),
            retry_count,
            retry_ceiling: contract.max_retries,
            completed_steps,
            step_ceiling: contract.max_iterations,
            spent_cost_microunits,
            cost_ceiling_microunits,
        },
        loop_object_id,
        budget_id,
    })
}

/// Finish daemon admission after the runtime has evaluated the fresh ceiling
/// snapshot. A committed STOP is terminal for this attempt: the lease closure
/// remains uncalled, ensuring no scheduler worker is admitted after a ceiling.
fn complete_scheduler_admission(
    ceiling_dispatch: SchedulerCeilingDispatch,
    acquire_lease: impl FnOnce() -> Result<SchedulerDispatch, SchedulerServiceError>,
) -> Result<SchedulerDispatchAdmission, SchedulerAuthorityError> {
    match ceiling_dispatch {
        SchedulerCeilingDispatch::Stopped(transition) => {
            Ok(SchedulerDispatchAdmission::Stopped(transition))
        }
        SchedulerCeilingDispatch::Proceed => {
            Ok(SchedulerDispatchAdmission::Leased(acquire_lease()?))
        }
    }
}

/// Invoke the daemon-owned Effect-closure boundary only after a fenced lease.
///
/// A persisted ceiling STOP never reaches the callback. A callback error or a
/// pending reconciliation leaves the durable lease untouched so a stale or
/// uncertain Effect cannot be reported as a scheduler or Task success.
fn complete_scheduler_worker_attempt(
    admission: SchedulerDispatchAdmission,
    complete_effect: impl FnOnce(
        SchedulerDispatch,
    ) -> Result<SchedulerEffectClosure, SchedulerAuthorityError>,
) -> Result<SchedulerWorkerAttempt, SchedulerAuthorityError> {
    match admission {
        SchedulerDispatchAdmission::Stopped(transition) => {
            Ok(SchedulerWorkerAttempt::Stopped(transition))
        }
        SchedulerDispatchAdmission::Leased(dispatch) => match complete_effect(dispatch.clone())? {
            SchedulerEffectClosure::Closed => Ok(SchedulerWorkerAttempt::EffectClosed(dispatch)),
            SchedulerEffectClosure::PendingReconciliation => {
                Ok(SchedulerWorkerAttempt::AwaitingReconciliation(dispatch))
            }
        },
    }
}

/// Release a fenced scheduler lease only after durable Effect closure.
///
/// The supplied release operation must call `SchedulerRepository::release_lease`
/// with the exact dispatch task reference, owner, and epoch. This boundary
/// intentionally retains stopped and reconciliation-pending attempts: neither
/// state proves that an Effect is closed, and neither may become scheduler or
/// Task success through lease release.
fn release_closed_effect_dispatch(
    worker_attempt: SchedulerWorkerAttempt,
    release_lease: impl FnOnce(SchedulerDispatch) -> Result<(), SchedulerAuthorityError>,
) -> Result<SchedulerWorkerAttempt, SchedulerAuthorityError> {
    match worker_attempt {
        SchedulerWorkerAttempt::EffectClosed(dispatch) => {
            release_lease(dispatch.clone())?;
            Ok(SchedulerWorkerAttempt::EffectClosed(dispatch))
        }
        SchedulerWorkerAttempt::Stopped(transition) => {
            Ok(SchedulerWorkerAttempt::Stopped(transition))
        }
        SchedulerWorkerAttempt::ContinuationStarted(transition) => {
            Ok(SchedulerWorkerAttempt::ContinuationStarted(transition))
        }
        SchedulerWorkerAttempt::AwaitingReconciliation(dispatch) => {
            Ok(SchedulerWorkerAttempt::AwaitingReconciliation(dispatch))
        }
    }
}

/// Resolve a dispatch's durable Effect and close only the matching scheduler
/// lease. This is the concrete worker closure boundary: it accepts neither a
/// process receipt nor a caller-provided Effect state.
///
/// `task_binding` is fixed before worker entry and must match the leased task.
/// The repository release retains the dispatch owner and epoch, while the
/// durable Effect resolver supplies the only closure disposition. A scheduler
/// `Succeeded` row means this dispatch's Effect reached a terminal durable
/// state; it does not accept or complete the Task.
fn complete_durable_scheduler_effect_closure<S>(
    admission: SchedulerDispatchAdmission,
    store: &S,
    task_binding: &TaskBinding,
    scheduler_repository: &mut SchedulerRepository,
    released_at: &str,
) -> Result<SchedulerWorkerAttempt, SchedulerAuthorityError>
where
    S: AuthorityStore + ProtocolStore,
{
    let worker_attempt = complete_scheduler_worker_attempt(admission, |dispatch| {
        if dispatch.task_ref != task_binding.task_ref
            || dispatch.contract_epoch != task_binding.contract_epoch
        {
            return Err(SchedulerAuthorityError::DispatchBindingMismatch(format!(
                "leased task {} at epoch {} does not match bound task {} at epoch {}",
                dispatch.task_ref,
                dispatch.contract_epoch,
                task_binding.task_ref,
                task_binding.contract_epoch,
            )));
        }
        Ok(resolve_scheduler_effect_for_task_binding(store, task_binding)?.closure)
    })?;
    complete_resolved_effect_and_release(worker_attempt, scheduler_repository, released_at)
}

/// Consume one candidate-admission WIA after fresh scheduler admission.
///
/// A candidate WIA authorizes the prior atomic `DECIDE -> ACT` admission. It
/// is not a continuation token for `CONTINUE -> OBSERVE`: that later
/// transition needs a distinct authority, checkpoint, current loop version,
/// and fresh budget admission. This boundary therefore records only the
/// recoverable handoff and resolves its durable Effect state.
#[allow(clippy::too_many_arguments)]
fn run_bounded_scheduler_attempt<S, C, G>(
    authority_store: &S,
    scheduler_repository: &mut SchedulerRepository,
    scheduler_service: &mut SchedulerService,
    driver: &LoopDriver<'_, S, C, G>,
    binding: &SchedulerAuthorityBinding,
    task_binding: &TaskBinding,
    scheduler_lease_epoch: i64,
    observed_wall_time: &str,
    candidate_authorization_id: Option<&ObjectId>,
    continuation_authorization: Option<&ContinuationAuthorizationRow>,
    worker_attempt_id: ObjectId,
    released_at: &str,
) -> Result<SchedulerWorkerAttempt, SchedulerAuthorityError>
where
    S: AuthorityStore
        + ContinuationAuthorityStore
        + HarnessStore
        + IntentChainStore
        + ProtocolStore
        + WorkerAuthorizationStore,
    C: Clock,
    G: IdGenerator,
{
    let candidate_authorization = match (continuation_authorization, candidate_authorization_id) {
        (Some(_), _) => None,
        (None, Some(authorization_id)) => Some(load_current_worker_authorization(
            authority_store,
            authorization_id,
            binding,
        )?),
        (None, None) => {
            return Err(SchedulerAuthorityError::CandidateUnavailable(
                "scheduler attempt has no candidate or continuation authority".to_owned(),
            ));
        }
    };
    let expected_loop_version = continuation_authorization
        .map(|authorization| authorization.expected_loop_version)
        .or_else(|| {
            candidate_authorization
                .as_ref()
                .map(|authorization| authorization.expected_loop_version)
        })
        .ok_or_else(|| {
            SchedulerAuthorityError::CandidateUnavailable(
                "scheduler attempt has no candidate or continuation authority".to_owned(),
            )
        })?;
    let writer_lease = WriterLease {
        epoch: authority_store
            .current_fencing_epoch()
            .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?,
    };
    let admission = admit_scheduler_dispatch(
        authority_store,
        scheduler_repository,
        scheduler_service,
        driver,
        binding,
        scheduler_lease_epoch,
        observed_wall_time,
        expected_loop_version,
        &writer_lease,
    )?;
    let SchedulerDispatchAdmission::Leased(dispatch) = admission else {
        let SchedulerDispatchAdmission::Stopped(transition) = admission else {
            unreachable!("scheduler admission has only stopped or leased outcomes");
        };
        return Ok(SchedulerWorkerAttempt::Stopped(transition));
    };

    if let Some(authorization) = continuation_authorization {
        let budget_charge: BudgetCharge =
            serde_json::from_str(&authorization.budget_charge_canonical_json).map_err(|error| {
                SchedulerAuthorityError::ContinuationUnavailable(format!(
                    "continuation {} has an invalid budget charge: {error}",
                    authorization.continuation_authorization_id
                ))
            })?;
        let consumed_at = FixedSchedulerClock::parse(observed_wall_time)?
            .now()
            .map_err(|error| SchedulerAuthorityError::Store(error.detail))?;
        let canonical_json = String::from_utf8(
            canonical::canonical_bytes_of_value(&json!({
                "continuation_authorization_id": authorization.continuation_authorization_id,
                "consumed_at": consumed_at.as_str(),
                "consumed_fencing_epoch": writer_lease.epoch,
                "worker_attempt_id": worker_attempt_id,
            }))
            .map_err(|error| SchedulerAuthorityError::ContinuationUnavailable(error.to_string()))?,
        )
        .map_err(|error| SchedulerAuthorityError::ContinuationUnavailable(error.to_string()))?;
        let consumption = BoundContinuationAuthorizationConsumption {
            consumption: ContinuationAuthorizationConsumptionRow {
                continuation_authorization_id: authorization.continuation_authorization_id.clone(),
                worker_attempt_id,
                consumed_fencing_epoch: writer_lease.epoch,
                consumed_at,
                canonical_json,
            },
            scheduler_lease: SchedulerLeaseBinding {
                task_ref: dispatch.task_ref.clone(),
                contract_epoch: dispatch.contract_epoch,
                lease_owner: dispatch.lease_owner.clone(),
                lease_epoch: dispatch.lease_epoch,
            },
        };
        let transition = driver.begin_verified_continuation_atomically(
            &consumption,
            &authorization.loop_object_id,
            authorization.expected_loop_version,
            &authorization.task_binding.task_ref,
            authorization.iteration,
            &authorization.budget_id,
            &budget_charge,
            &writer_lease,
        )?;
        return Ok(SchedulerWorkerAttempt::ContinuationStarted(transition));
    }

    consume_worker_authorization_for_attempt(
        authority_store,
        // The LoopDriver clock is unavailable here; use the scheduler's
        // trusted observation time only after parsing it as a wall timestamp.
        &FixedSchedulerClock::parse(observed_wall_time)?,
        candidate_authorization_id.ok_or_else(|| {
            SchedulerAuthorityError::CandidateUnavailable(
                "candidate WIA was absent after continuation selection".to_owned(),
            )
        })?,
        worker_attempt_id,
        &dispatch,
    )?;
    let worker_attempt = complete_durable_scheduler_effect_closure(
        SchedulerDispatchAdmission::Leased(dispatch),
        authority_store,
        task_binding,
        scheduler_repository,
        released_at,
    )?;

    Ok(worker_attempt)
}

/// Run one daemon-private scheduler pass over durable runnable work.
///
/// This private tick chooses either candidate-WIA reconciliation or a
/// distinct persisted verified continuation entry. It never accepts client
/// input, dispatches an executor, records worker progress, or changes Task
/// lifecycle state. Candidate WIA cannot enter the bounded harness.
pub(crate) fn run_private_scheduler_tick(
    authority_database_path: &Path,
) -> Result<(), SchedulerAuthorityError> {
    let provider_config_dir = authority_database_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    run_private_scheduler_tick_with_provider_config(authority_database_path, provider_config_dir)
}

/// Run one scheduler pass with the daemon-owned configuration directory used
/// for the selected model, Pi configuration, and private completion socket.
/// The separate parameter keeps scheduler fixtures hermetic while production
/// startup supplies the layout's actual configuration directory.
pub(crate) fn run_private_scheduler_tick_with_provider_config(
    authority_database_path: &Path,
    provider_config_dir: &Path,
) -> Result<(), SchedulerAuthorityError> {
    let authority_store = SqliteAuthorityStore::open(authority_database_path)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let mut scheduler_repository = SchedulerRepository::open(authority_database_path)?;
    let clock = SystemClock;
    let identifiers = UuidV7Generator;
    let mut scheduler_service = SchedulerService::new("personal-daemon-scheduler", 60)?;
    let scheduler_rows = scheduler_repository.list_recoverable()?;

    for scheduler_row in scheduler_rows {
        if scheduler_row.state != SchedulerState::Runnable.as_str()
            || scheduler_row.cancel_requested
        {
            continue;
        }
        let resolved_work =
            resolve_scheduler_work_for_task(&authority_store, &scheduler_row.task_ref)?;
        if resolved_work.task_binding.contract_epoch != scheduler_row.contract_epoch {
            return Err(SchedulerAuthorityError::DispatchBindingMismatch(format!(
                "runnable scheduler work {} at epoch {} is not the current contract epoch {}",
                scheduler_row.task_ref,
                scheduler_row.contract_epoch,
                resolved_work.task_binding.contract_epoch
            )));
        }
        let context_execution_policy =
            load_context_execution_policy(&authority_store, &resolved_work.task_binding)?;
        let observed_wall_time = clock
            .now()
            .map_err(|error| SchedulerAuthorityError::Store(error.detail))?;
        let continuation_authorization = authority_store
            .load_unconsumed_continuation_authorization(&resolved_work.task_binding)
            .map_err(|error| SchedulerAuthorityError::ContinuationUnavailable(error.to_string()))?;
        let candidate_authorization = if continuation_authorization.is_none() {
            authority_store
                .load_unconsumed_worker_iteration_authorization_for_task_binding(
                    &resolved_work.task_binding.task_ref,
                    resolved_work.task_binding.contract_epoch,
                )
                .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        } else {
            None
        };
        if continuation_authorization.is_none() && candidate_authorization.is_none() {
            let policy = context_execution_policy.as_ref().ok_or_else(|| {
                SchedulerAuthorityError::CandidateUnavailable(
                    "runnable work has no daemon-private execution policy".to_owned(),
                )
            })?;
            let context_command =
                context_resolution_command_from_policy(policy, observed_wall_time.clone())?;
            let admission_command = candidate_admission_command_from_policy(policy)?;
            let pi_config = load_pi_config(provider_config_dir).map_err(|_| {
                SchedulerAuthorityError::CandidateUnavailable(
                    "daemon-private Pi candidate transport is not configured".to_owned(),
                )
            })?;
            let proposer = ConfiguredPrivatePiCandidateProposer::new(
                PrivatePiCandidateProcess::from_config(&pi_config, provider_config_dir),
            );
            propose_persist_and_admit_candidate(
                &authority_store,
                &clock,
                &identifiers,
                &context_command,
                &proposer,
                &admission_command,
            )?;
            // Admission is the only outcome of the Pi proposal pass. Leave
            // the newly issued WIA durable and unconsumed for a later normal
            // scheduler recovery/dispatch pass; Pi invocation must never
            // borrow or immediately consume worker authority.
            continue;
        }
        let scheduler_lease_epoch = scheduler_row.lease_epoch.checked_add(1).ok_or_else(|| {
            SchedulerAuthorityError::Store("scheduler lease epoch overflow".to_owned())
        })?;
        let worker_attempt_id = next_object_id(&identifiers)?;
        let driver = LoopDriver::new(
            &authority_store,
            &clock,
            &identifiers,
            UriRef::parse("principal://personal/daemon").map_err(|error| {
                SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string())
            })?,
            UriRef::parse("authority://personal/loop").map_err(|error| {
                SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string())
            })?,
            UriRef::parse("correlation://personal/private-scheduler-tick").map_err(|error| {
                SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string())
            })?,
        );
        run_bounded_scheduler_attempt(
            &authority_store,
            &mut scheduler_repository,
            &mut scheduler_service,
            &driver,
            &resolved_work.authority_binding,
            &resolved_work.task_binding,
            scheduler_lease_epoch,
            observed_wall_time.as_str(),
            candidate_authorization
                .as_ref()
                .map(|authorization| &authorization.authorization_id),
            continuation_authorization.as_ref(),
            worker_attempt_id,
            observed_wall_time.as_str(),
        )?;
    }

    Ok(())
}

struct FixedSchedulerClock(WallTimestamp);

impl FixedSchedulerClock {
    fn parse(value: &str) -> Result<Self, SchedulerAuthorityError> {
        WallTimestamp::parse(value)
            .map(Self)
            .map_err(|_| SchedulerAuthorityError::InvalidReleaseTime(value.to_owned()))
    }
}

impl Clock for FixedSchedulerClock {
    fn now(&self) -> Result<WallTimestamp, cognitive_kernel::ports::PortFailure> {
        Ok(self.0.clone())
    }
}

fn load_current_worker_authorization<S>(
    store: &S,
    authorization_id: &ObjectId,
    binding: &SchedulerAuthorityBinding,
) -> Result<WorkerIterationAuthorizationRow, SchedulerAuthorityError>
where
    S: IntentChainStore + ProtocolStore + WorkerAuthorizationStore,
{
    let authorization = store
        .load_worker_iteration_authorization(authorization_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::CandidateUnavailable(authorization_id.to_string())
        })?;
    validate_worker_authorization_evidence(&authorization)?;
    let current_contract_epoch = store
        .current_contract_epoch(&authorization.task_ref)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    ensure_current_contract_epoch(binding, current_contract_epoch)?;
    if authorization.task_ref != binding.task_ref
        || authorization.contract_epoch != binding.contract_epoch
    {
        return Err(SchedulerAuthorityError::DispatchBindingMismatch(
            "WorkerIterationAuthorization does not match scheduler authority binding".to_owned(),
        ));
    }
    Ok(authorization)
}

/// Release an already resolved closed Effect through the real scheduler
/// repository. Pending reconciliation and durable ceiling STOP attempts keep
/// their leases untouched.
fn complete_resolved_effect_and_release(
    worker_attempt: SchedulerWorkerAttempt,
    scheduler_repository: &mut SchedulerRepository,
    released_at: &str,
) -> Result<SchedulerWorkerAttempt, SchedulerAuthorityError> {
    WallTimestamp::parse(released_at)
        .map_err(|_| SchedulerAuthorityError::InvalidReleaseTime(released_at.to_owned()))?;
    release_closed_effect_dispatch(worker_attempt, |dispatch| {
        scheduler_repository.release_lease(
            &SchedulerWorkKey {
                task_ref: dispatch.task_ref.clone(),
                contract_epoch: dispatch.contract_epoch,
            },
            &dispatch.lease_owner,
            dispatch.lease_epoch,
            SchedulerState::Succeeded,
            released_at,
        )?;
        Ok(())
    })
}

/// Commit a reached ceiling STOP before a worker can obtain a scheduler lease.
///
/// This is the daemon composition boundary: it reloads the current authority
/// snapshot, delegates the fenced STOP commit to the kernel, and only calls
/// the scheduler repository when no hard ceiling was reached. It deliberately
/// stops before external worker or Effect dispatch.
#[allow(clippy::too_many_arguments)]
pub(crate) fn admit_scheduler_dispatch<S, C, G>(
    authority_store: &S,
    scheduler_repository: &mut SchedulerRepository,
    scheduler_service: &mut SchedulerService,
    driver: &LoopDriver<'_, S, C, G>,
    binding: &SchedulerAuthorityBinding,
    lease_epoch: i64,
    observed_wall_time: &str,
    expected_loop_version: Version,
    writer_lease: &WriterLease,
) -> Result<SchedulerDispatchAdmission, SchedulerAuthorityError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    let snapshot = load_scheduler_authority_snapshot(authority_store, binding)?;
    let ceiling_dispatch = scheduler_service.stop_before_dispatch_when_ceiling_reached(
        &snapshot.ceiling_facts,
        observed_wall_time,
        driver,
        &snapshot.loop_object_id,
        expected_loop_version,
        &binding.task_ref,
        &snapshot.budget_id,
        writer_lease,
    )?;
    complete_scheduler_admission(ceiling_dispatch, || {
        scheduler_service.claim_eligible(
            scheduler_repository,
            &SchedulerWorkKey {
                task_ref: binding.task_ref.clone(),
                contract_epoch: binding.contract_epoch,
            },
            lease_epoch,
            observed_wall_time,
        )
    })
}
