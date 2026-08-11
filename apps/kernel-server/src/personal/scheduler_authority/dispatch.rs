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
    let loop_control_decision = derive_loop_control_from_facts(
        &progress_facts,
        &binding.action_fingerprint,
        contract.max_retries,
        DEFAULT_LOOP_STAGNATION_CEILING,
    )?;
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
        loop_control_decision,
    })
}

/// Finish daemon admission after the runtime has evaluated the fresh ceiling
/// snapshot. A committed STOP is terminal for this attempt: the lease closure
/// remains uncalled, ensuring no scheduler worker is admitted after a ceiling.
pub(crate) fn complete_scheduler_admission(
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
pub(crate) fn complete_scheduler_worker_attempt(
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
pub(crate) fn release_closed_effect_dispatch(
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
pub(crate) fn complete_durable_scheduler_effect_closure<S>(
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
pub(crate) fn run_bounded_scheduler_attempt<S, C, G>(
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
    run_private_scheduler_tick_with_store(
        &authority_store,
        &mut scheduler_repository,
        provider_config_dir,
    )
}

/// Run one private scheduler pass against an already-open daemon-owned store.
///
/// Production startup opens the authority store once and reuses that
/// single-writer handle for recovery plus this tick (P9-T03/D01).
pub(crate) fn run_private_scheduler_tick_with_store(
    authority_store: &SqliteAuthorityStore,
    scheduler_repository: &mut SchedulerRepository,
    provider_config_dir: &Path,
) -> Result<(), SchedulerAuthorityError> {
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
            resolve_scheduler_work_for_task(authority_store, &scheduler_row.task_ref)?;
        if resolved_work.task_binding.contract_epoch != scheduler_row.contract_epoch {
            return Err(SchedulerAuthorityError::DispatchBindingMismatch(format!(
                "runnable scheduler work {} at epoch {} is not the current contract epoch {}",
                scheduler_row.task_ref,
                scheduler_row.contract_epoch,
                resolved_work.task_binding.contract_epoch
            )));
        }
        let context_execution_policy =
            load_context_execution_policy(authority_store, &resolved_work.task_binding)?;
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
                authority_store,
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
            authority_store,
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
            authority_store,
            scheduler_repository,
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

/// Release an already resolved closed Effect through the real scheduler
/// repository. Pending reconciliation and durable ceiling STOP attempts keep
/// their leases untouched.
pub(crate) fn complete_resolved_effect_and_release(
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
    match snapshot.loop_control_decision {
        LoopControlDecision::Continue => {}
        LoopControlDecision::Wait { reason_code } => {
            return Err(SchedulerAuthorityError::LoopUnavailable(format!(
                "loop control requires a bounded wait: {reason_code}"
            )));
        }
        LoopControlDecision::Switch {
            prior_signature_digest,
        } => {
            return Err(SchedulerAuthorityError::LoopUnavailable(format!(
                "loop control requires a daemon-owned alternate strategy after {prior_signature_digest}"
            )));
        }
        LoopControlDecision::Block { reason_code } => {
            return Err(SchedulerAuthorityError::LoopUnavailable(format!(
                "loop control blocked dispatch: {reason_code}"
            )));
        }
    }
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
