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
pub(crate) fn select_single_effect_intent<'intent>(
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
pub(crate) fn classify_scheduler_effect_closure(
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
