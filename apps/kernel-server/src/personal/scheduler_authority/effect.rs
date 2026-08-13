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
use cognitive_kernel::effects::{GovernanceCurrency, WriterLease, admit_operation};
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
/// epoch. Zero matching Intents is the first pre-admission pass; exactly one
/// reconstructs the durable Effect binding. Ambiguous or internally
/// inconsistent rows fail before lease acquisition, so recovery cannot guess
/// which Effect a task should drive.
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
    let authority_binding = if intent_rows.is_empty() {
        None
    } else {
        let intent_row = select_single_effect_intent(&task_binding, &intent_rows)?;
        let action_fingerprint = format!("{}:{}", intent_row.action, intent_row.parameters_digest);
        Some(SchedulerAuthorityBinding {
            task_ref: task_ref.to_owned(),
            contract_epoch,
            action_fingerprint,
        })
    };
    Ok(ResolvedSchedulerWork {
        authority_binding,
        task_binding,
    })
}

/// Reload the exact native Tool binding selected by one consumed WIA.
///
/// The WIA, candidate, Intent, descriptor, current contract epoch, and Effect
/// object must all agree before a concrete executor can stage a request. The
/// caller supplies the composition-root family set so an unassembled family
/// fails while the Effect remains untouched.
pub(crate) fn resolve_native_worker_dispatch_with_families<S>(
    store: &S,
    authorization: &WorkerIterationAuthorizationRow,
    assembled_families: &[cognitive_kernel::tool_registry::NativeOperationFamily],
) -> Result<ResolvedNativeWorkerDispatch, SchedulerAuthorityError>
where
    S: AuthorityStore + ProtocolStore + WorkerAuthorizationStore,
{
    validate_worker_authorization_evidence(authorization)?;
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
    let task_binding = TaskBinding {
        task_ref: authorization.task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
    };
    let candidate = store
        .load_operation_candidate_proposal(&authorization.selected_candidate_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::CandidateUnavailable(
                authorization.selected_candidate_id.to_string(),
            )
        })?;
    if candidate.task_ref != authorization.task_ref
        || candidate.contract_epoch != authorization.contract_epoch
    {
        return Err(SchedulerAuthorityError::DispatchBindingMismatch(
            "WIA-selected candidate is bound to a different TaskContract epoch".to_owned(),
        ));
    }
    let intent = store
        .load_intent_for_effect(&authorization.effect_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::MissingEffectBinding {
            task_ref: authorization.task_ref.clone(),
            contract_epoch: authorization.contract_epoch,
        })?;
    let intent_matches_authorization = intent.intent_id == authorization.intent_id
        && intent.effect_object_id == authorization.effect_object_id
        && intent.task_binding.as_ref() == Some(&task_binding);
    let candidate_matches_intent = candidate.action == intent.action
        && candidate.target == intent.target
        && candidate.parameters_digest == intent.parameters_digest
        && candidate.expected_state_version == intent.expected_state_version.get();
    if !intent_matches_authorization || !candidate_matches_intent {
        return Err(SchedulerAuthorityError::DispatchBindingMismatch(
            "WIA, candidate, and durable Intent execution bindings disagree".to_owned(),
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
    {
        return Err(SchedulerAuthorityError::CandidateDescriptorUnavailable(
            "candidate and persisted descriptor disagree".to_owned(),
        ));
    }
    let native_tool =
        resolve_persisted_native_descriptor(&descriptor.descriptor).map_err(|error| {
            SchedulerAuthorityError::CandidateDescriptorUnavailable(format!(
                "persisted native descriptor was rejected: {error:?}"
            ))
        })?;
    if !assembled_families.contains(&native_tool.descriptor.family) {
        return Err(SchedulerAuthorityError::CandidateDescriptorUnavailable(
            "persisted native descriptor has no assembled executor".to_owned(),
        ));
    }
    let effect = store
        .load_object(LifecycleDomain::Effect, &authorization.effect_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::MissingEffect(authorization.effect_object_id.to_string())
        })?;
    Ok(ResolvedNativeWorkerDispatch {
        authorization: authorization.clone(),
        candidate,
        intent,
        native_tool,
        effect_version: effect.version,
        effect_state: effect.state.as_str().to_owned(),
    })
}

/// Stage and drive one freshly consumed native WIA through the existing Effect
/// protocol.
///
/// `EffectProtocol::dispatch_effect` commits `EXECUTING` before invoking the
/// router. A receipt-confirmed execution is immediately reconciled under the
/// original idempotency key; unknown and confirmed non-execution outcomes
/// remain non-success and never become Task progress or completion.
pub(crate) fn dispatch_native_worker_effect<S, C, G>(
    effect_protocol: &cognitive_kernel::effects::EffectProtocol<'_, S, C, G>,
    resolved: &ResolvedNativeWorkerDispatch,
    executor_router: &crate::personal::tool_executor::ProductionNativeToolExecutorRouter,
    grant: &cognitive_kernel::authz::AuthorizationGrant,
    governance_currency: &cognitive_kernel::effects::GovernanceCurrency,
    writer_lease: &WriterLease,
) -> Result<SchedulerEffectClosure, SchedulerAuthorityError>
where
    S: AuthorityStore + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    if resolved.effect_state != "PROPOSED" {
        return Err(SchedulerAuthorityError::UnsupportedEffectState(
            resolved.effect_state.clone(),
        ));
    }
    executor_router
        .stage_resolved(resolved)
        .map_err(|error| SchedulerAuthorityError::NativeExecution(error.to_string()))?;
    let authorized = effect_protocol.authorize_effect(
        &resolved.authorization.effect_object_id,
        resolved.effect_version,
        grant,
        governance_currency,
        writer_lease,
    )?;
    let (executing, outcome) = effect_protocol.dispatch_effect(
        &resolved.authorization.effect_object_id,
        authorized.after_version,
        grant,
        governance_currency,
        executor_router,
        writer_lease,
    )?;
    let recorded = effect_protocol.record_outcome(
        &resolved.authorization.effect_object_id,
        executing.after_version,
        &outcome,
        writer_lease,
    )?;
    match outcome {
        cognitive_kernel::executor::DispatchOutcome::Executed { .. } => {
            let (_, query) = effect_protocol.reconcile(
                &resolved.authorization.effect_object_id,
                "EXECUTED",
                recorded.after_version,
                executor_router,
                writer_lease,
            )?;
            if query != cognitive_kernel::executor::ExecutorQueryResult::ExecutedWithOriginalKey {
                return Err(SchedulerAuthorityError::NativeExecution(
                    "receipt-confirmed execution did not reconcile under its original key"
                        .to_owned(),
                ));
            }
            Ok(SchedulerEffectClosure::Closed)
        }
        cognitive_kernel::executor::DispatchOutcome::Unknown { .. } => {
            let (reconciled, query) = effect_protocol.reconcile(
                &resolved.authorization.effect_object_id,
                "OUTCOME_UNKNOWN",
                recorded.after_version,
                executor_router,
                writer_lease,
            )?;
            match query {
                cognitive_kernel::executor::ExecutorQueryResult::ExecutedWithOriginalKey => {
                    Ok(SchedulerEffectClosure::Closed)
                }
                cognitive_kernel::executor::ExecutorQueryResult::NotExecuted => {
                    effect_protocol.close_not_executed(
                        &resolved.authorization.effect_object_id,
                        reconciled.after_version,
                        writer_lease,
                    )?;
                    Ok(SchedulerEffectClosure::PendingReconciliation)
                }
                cognitive_kernel::executor::ExecutorQueryResult::Indeterminate => {
                    effect_protocol.quarantine_still_unknown(
                        &resolved.authorization.effect_object_id,
                        reconciled.after_version,
                        writer_lease,
                    )?;
                    Ok(SchedulerEffectClosure::PendingReconciliation)
                }
            }
        }
        cognitive_kernel::executor::DispatchOutcome::NotExecuted { .. } => {
            Ok(SchedulerEffectClosure::PendingReconciliation)
        }
        cognitive_kernel::executor::DispatchOutcome::FencedStaleEpoch { .. } => {
            unreachable!("record_outcome rejects a stale-sink dispatch")
        }
    }
}

/// Re-authorize one resolved native dispatch from current daemon facts.
///
/// The immutable candidate-admission snapshot is evidence of the earlier WIA
/// decision, not reusable authority for I/O. This edge reconstructs the
/// current Context authorization snapshot and evaluates the exact candidate
/// target/action/purpose again immediately before Effect authorization.
pub(crate) fn derive_current_native_execution_authorization<S>(
    store: &S,
    policy: &SchedulerExecutionPolicyRow,
    resolved: &ResolvedNativeWorkerDispatch,
    observed_at: WallTimestamp,
) -> Result<
    (
        cognitive_kernel::authz::AuthorizationGrant,
        cognitive_kernel::effects::GovernanceCurrency,
    ),
    SchedulerAuthorityError,
>
where
    S: ContextAuthorizationFactStore + IntentChainStore,
{
    let context_command = context_resolution_command_from_policy(policy, observed_at)?;
    if context_command.task_ref != resolved.authorization.task_ref {
        return Err(SchedulerAuthorityError::DispatchBindingMismatch(
            "scheduler policy and native dispatch name different Tasks".to_owned(),
        ));
    }
    let admission_command = candidate_admission_command_from_policy(policy)?;
    if admission_command.authorization_subject_ref != context_command.authorization_subject_ref {
        return Err(SchedulerAuthorityError::CandidateAuthorizationUnavailable(
            "scheduler policy authorization subjects disagree".to_owned(),
        ));
    }
    let snapshot = load_current_context_authorization_snapshot(store, &context_command)?;
    let governance = ObjectGovernance {
        object_ref: resolved.candidate.target.clone(),
        tenant_id: Some(context_command.tenant_id),
        owner_ref: admission_command.authorization_subject_ref,
        resource_scope: resolved.candidate.target.clone(),
        conversation_ref: context_command.conversation_ref,
    };
    let grant = authorize(
        &snapshot,
        &governance,
        &AccessRequest {
            action: resolved.candidate.action.clone(),
            purpose: admission_command.authorization_purpose,
        },
    )
    .map_err(|error| {
        SchedulerAuthorityError::CandidateAuthorizationUnavailable(format!(
            "current native execution authorization denied: {error:?}"
        ))
    })?;
    let currency = GovernanceCurrency {
        revocation_epoch: snapshot.revocation_epoch,
        capability_set_version: snapshot.capability_set_version,
    };
    Ok((grant, currency))
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
