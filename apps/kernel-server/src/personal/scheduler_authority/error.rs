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
    #[error("scheduler loop-control facts are malformed or unavailable: {0}")]
    LoopControlUnavailable(String),
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
    #[error("scheduler native Tool execution failed closed: {0}")]
    NativeExecution(String),
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
