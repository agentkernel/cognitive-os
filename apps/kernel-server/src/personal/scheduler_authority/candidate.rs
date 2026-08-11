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

use super::pi_runtime::{
    PrivatePiCandidateProcess, PrivatePiCandidateRequest, PrivatePiCandidateResponse,
    load_pi_config,
};

use super::*;

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
pub(crate) fn propose_persist_and_admit_candidate_after_metadata<S, C, G, P, F>(
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

pub(crate) fn validate_untrusted_pi_candidate(
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

pub(crate) fn is_sha256_digest(value: &str) -> bool {
    let Some(hexadecimal) = value.strip_prefix("sha256:") else {
        return false;
    };
    hexadecimal.len() == 64 && hexadecimal.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn canonical_descriptor_reference_digest(
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
