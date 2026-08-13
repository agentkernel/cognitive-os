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
    GovernanceSeed, compose_governed_header, prepare_task_execution_bootstrap,
    seal_governed_object_content_digest, strong_reference_to,
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
        StorePortError, TaskBinding, WorkerAuthorizationStore,
        WorkerIterationAuthorizationConsumptionRow, WorkerIterationAuthorizationRow,
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
pub(crate) fn validate_worker_authorization_evidence(
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
    reconcile_scheduler_recovery_with_store(&authority_store, &mut scheduler_repository)
}

fn repair_admitted_task_execution_bootstraps<S>(
    authority_store: &S,
) -> Result<(), SchedulerAuthorityError>
where
    S: AuthorityStore + IntentChainStore + ProtocolStore,
{
    let current_contracts = authority_store
        .list_current_task_contracts()
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let writer_lease = WriterLease {
        epoch: authority_store
            .current_fencing_epoch()
            .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?,
    };
    let clock = SystemClock;
    let identifiers = UuidV7Generator;
    let correlation_id = UriRef::parse("correlation://personal/startup-bootstrap-repair")
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;

    for contract in current_contracts {
        let bootstrap = match prepare_task_execution_bootstrap(
            authority_store,
            &clock,
            &identifiers,
            &writer_lease,
            &contract,
            &correlation_id,
        ) {
            Ok(bootstrap) => bootstrap,
            Err(error) => {
                eprintln!(
                    "kernel-server personal scheduler recovery: skip non-execution-bound TaskContract {} at epoch {}: {error}",
                    contract.task_ref, contract.contract_epoch
                );
                continue;
            }
        };
        match authority_store.repair_task_execution_bootstrap(&contract, &bootstrap) {
            Ok(repair)
                if repair.task_created
                    || repair.loop_created
                    || repair.budget_created
                    || repair.scheduler_created =>
            {
                eprintln!(
                    "kernel-server personal scheduler recovery: repaired Task bootstrap {} at epoch {} (task={}, loop={}, budget={}, scheduler={})",
                    contract.task_ref,
                    contract.contract_epoch,
                    repair.task_created,
                    repair.loop_created,
                    repair.budget_created,
                    repair.scheduler_created
                );
            }
            Ok(_) => {}
            Err(StorePortError::Conflict { detail }) => {
                eprintln!(
                    "kernel-server personal scheduler recovery: skip conflicting Task bootstrap {} at epoch {}: {detail}",
                    contract.task_ref, contract.contract_epoch
                );
            }
            Err(error @ StorePortError::Unavailable { .. }) => {
                return Err(SchedulerAuthorityError::Store(error.to_string()));
            }
        }
    }
    Ok(())
}

/// Startup recovery against an already-open daemon-owned authority store.
///
/// Personal daemon startup must open the authority store once and reuse that
/// single-writer handle for recovery and the subsequent private tick
/// (P9-T03/D01). Callers that still pass only a path go through
/// [`reconcile_scheduler_recovery_at_startup`].
pub(crate) fn reconcile_scheduler_recovery_with_store<S>(
    authority_store: &S,
    scheduler_repository: &mut SchedulerRepository,
) -> Result<(), SchedulerAuthorityError>
where
    S: AuthorityStore
        + IntentChainStore
        + ProtocolStore
        + WorkerAuthorizationStore
        + ContinuationAuthorityStore,
{
    repair_admitted_task_execution_bootstraps(authority_store)?;
    reconcile_recovered_worker_attempts(
        authority_store,
        scheduler_repository,
        &cognitive_store::SystemClock,
    )?;
    Ok(())
}

pub(crate) fn release_closed_recovered_attempt(
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

pub(crate) fn load_current_worker_authorization<S>(
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
