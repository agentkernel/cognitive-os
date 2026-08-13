use cognitive_contracts::generated::{
    common_defs::Digest,
    object_reference::{StrongReference, StrongReferenceKind},
    task_contract::{ContractConditionKind, TaskContract},
};
use cognitive_domain::{LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef};
use cognitive_kernel::{
    Causation, CommittedTransition, Reason, TablePin, TransitionCommand, TransitionEngine,
    effects::{WriterLease, strong_reference_for_content},
    ports::{
        AuthorityStore, Clock, ContinuationAuthorityStore, FixedPostStateRow, HarnessStore,
        IdGenerator, IntentChainStore, ProtocolStore, StorePortError, StoredObject,
        TaskAcceptanceCommit, TaskBinding, TaskCompletionClaimCommit, TaskContractRow,
        VerificationReportRow, VerificationRequestRow, WorkerIterationAuthorizationRow,
    },
};
use cognitive_store::ArtifactStore;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const ACCEPTANCE_PRINCIPAL_REF: &str = "principal://personal/acceptance-authority";
const ACCEPTANCE_AUTHORITY_REF: &str = "authority://personal/task-acceptance";
const TASK_LIFECYCLE_AUTHORITY_REF: &str = "authority://personal/task-lifecycle";

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProductionTaskCompletionOutcome {
    pub candidate: Option<CommittedTransition>,
    pub acceptance: CommittedTransition,
    pub acceptance_decision_artifact_ref: String,
}

#[derive(Debug, Error)]
pub(crate) enum TaskCompletionError {
    #[error("Task completion writer is fenced")]
    WriterFenced,
    #[error("Task completion TaskContract binding is stale or unavailable")]
    ContractUnavailable,
    #[error("governed Task lifecycle state is unavailable or ineligible")]
    TaskUnavailable,
    #[error("worker authority cannot activate this governed Task")]
    ExecutionBindingUnavailable,
    #[error("Task completion verification is unavailable or stale")]
    VerificationUnavailable,
    #[error("Task completion requires every current task-bound Effect to be closed")]
    EffectsOpen,
    #[error("Task completion evidence is unavailable or invalid: {0}")]
    EvidenceUnavailable(String),
    #[error("Task acceptance was already committed")]
    DuplicateAcceptance,
    #[error("Task completion infrastructure is unavailable: {0}")]
    Infrastructure(String),
}

struct CurrentAcceptanceFacts {
    contract_row: TaskContractRow,
    fixed_post_state: FixedPostStateRow,
    verification_request: VerificationRequestRow,
    verification_report: VerificationReportRow,
    artifact_evidence_refs: Vec<String>,
    effect_object_ids: Vec<ObjectId>,
}

/// 使用当前 daemon-issued WIA 把持久 Task 生命周期推进到 `ACTIVE`。
///
/// 该函数只读取不可变合同和原子 candidate-admission 产物；Agent、Provider、
/// Tool receipt 或任意调用方布尔值都不能建立两个 transition guards。
pub(crate) fn activate_task_for_worker_authorization<S, C, G>(
    store: &S,
    clock: &C,
    identifiers: &G,
    authorization: &WorkerIterationAuthorizationRow,
    writer_lease: &WriterLease,
) -> Result<StoredObject, TaskCompletionError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    verify_writer(store, writer_lease)?;
    let task_binding = TaskBinding {
        task_ref: authorization.task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
    };
    let contract_row = load_current_contract(store, &task_binding)?;
    let contract = parse_contract(&contract_row)?;
    if authorization.worker_authorization_root_id != contract_row.contract_id
        || authorization.issued_fencing_epoch != writer_lease.epoch
        || contract
            .worker_authorization_root_id
            .as_ref()
            .map(|id| id.0.as_str())
            != Some(contract_row.contract_id.as_str())
    {
        return Err(TaskCompletionError::ExecutionBindingUnavailable);
    }
    let effect = store
        .load_object(LifecycleDomain::Effect, &authorization.effect_object_id)
        .map_err(infrastructure)?
        .ok_or(TaskCompletionError::ExecutionBindingUnavailable)?;
    let loop_object = store
        .load_object(LifecycleDomain::Loop, &authorization.loop_object_id)
        .map_err(infrastructure)?
        .ok_or(TaskCompletionError::ExecutionBindingUnavailable)?;
    if effect.state.as_str() != "PROPOSED"
        || loop_object.state.as_str() != "ACT"
        || store
            .load_budget(&authorization.budget_id)
            .map_err(infrastructure)?
            .is_none()
    {
        return Err(TaskCompletionError::ExecutionBindingUnavailable);
    }

    let mut task = load_task(store, &contract_row.contract_id)?;
    if task.state.as_str() == "DRAFT" {
        let acceptance_is_fixed = contract
            .conditions
            .iter()
            .filter(|condition| condition.kind == ContractConditionKind::Acceptance)
            .all(|condition| {
                condition
                    .verifier_ref
                    .as_deref()
                    .is_some_and(|reference| !reference.trim().is_empty())
            })
            && contract
                .conditions
                .iter()
                .any(|condition| condition.kind == ContractConditionKind::Acceptance);
        if !acceptance_is_fixed {
            return Err(TaskCompletionError::ContractUnavailable);
        }
        let transition = task_transition_command(
            clock,
            &task_binding,
            &task,
            "READY",
            "CONTRACT_ACCEPTED",
            ["task_contract_complete", "acceptance_criteria_fixed"],
            BTreeMap::from([(
                "task_contract".to_owned(),
                strong_reference_for_content(
                    &contract_row.contract_id,
                    contract_row.contract_epoch,
                    &contract_row.canonical_json,
                )
                .map_err(|error| TaskCompletionError::EvidenceUnavailable(error.to_string()))?,
            )]),
            "principal://personal/daemon",
            TASK_LIFECYCLE_AUTHORITY_REF,
            writer_lease.epoch,
        )?;
        TransitionEngine::new(store, clock, identifiers)
            .commit_transition(&transition)
            .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?;
        task = load_task(store, &contract_row.contract_id)?;
    }
    if task.state.as_str() == "READY" {
        let transition = task_transition_command(
            clock,
            &task_binding,
            &task,
            "ACTIVE",
            "EXECUTION_STARTED",
            ["execution_admitted", "dependencies_satisfied"],
            BTreeMap::from([(
                "execution_binding".to_owned(),
                strong_reference_for_content(
                    &authorization.authorization_id,
                    1,
                    &authorization.canonical_json,
                )
                .map_err(|error| TaskCompletionError::EvidenceUnavailable(error.to_string()))?,
            )]),
            "principal://personal/daemon",
            TASK_LIFECYCLE_AUTHORITY_REF,
            writer_lease.epoch,
        )?;
        TransitionEngine::new(store, clock, identifiers)
            .commit_transition(&transition)
            .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?;
        task = load_task(store, &contract_row.contract_id)?;
    }
    if !matches!(
        task.state.as_str(),
        "ACTIVE" | "CANDIDATE_COMPLETE" | "COMPLETED"
    ) {
        return Err(TaskCompletionError::TaskUnavailable);
    }
    Ok(task)
}

/// 从持久、最新且可重新读取证据的独立报告完成 Task。
///
/// candidate claim 与 acceptance decision 是两个登记 transition；SQLite 在各自
/// 事务内重查合同、Effect 集合、fixed state、报告 currentness 与 fencing。
#[allow(clippy::too_many_arguments)]
pub(crate) fn complete_task_from_persisted_verification<S, C, G>(
    store: &S,
    artifact_store: &ArtifactStore,
    clock: &C,
    identifiers: &G,
    task_binding: &TaskBinding,
    verification_report_id: &ObjectId,
    writer_lease: &WriterLease,
) -> Result<ProductionTaskCompletionOutcome, TaskCompletionError>
where
    S: AuthorityStore
        + ContinuationAuthorityStore
        + HarnessStore
        + IntentChainStore
        + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    verify_writer(store, writer_lease)?;
    let initial_contract = load_current_contract(store, task_binding)?;
    let mut task = load_task(store, &initial_contract.contract_id)?;
    if task.state.as_str() == "COMPLETED" {
        return Err(TaskCompletionError::DuplicateAcceptance);
    }
    if !matches!(task.state.as_str(), "ACTIVE" | "CANDIDATE_COMPLETE") {
        return Err(TaskCompletionError::TaskUnavailable);
    }
    let initial_facts = load_current_acceptance_facts(
        store,
        artifact_store,
        task_binding,
        verification_report_id,
        writer_lease,
    )?;
    let candidate = if task.state.as_str() == "ACTIVE" {
        let claim_id = next_object_id(identifiers)?;
        let (claim_artifact_ref, claim_evidence) = persist_canonical_artifact(
            artifact_store,
            &claim_id,
            &json!({
                "decision": "candidate_complete",
                "effect_object_ids": initial_facts.effect_object_ids,
                "fixed_post_state_id": initial_facts.fixed_post_state.fixed_post_state_id.as_str(),
                "schema_version": 1,
                "task_ref": task_binding.task_ref,
                "contract_epoch": task_binding.contract_epoch,
                "verification_report_id": initial_facts.verification_report.verification_report_id.as_str(),
            }),
        )?;
        ensure_artifact_available(artifact_store, &claim_artifact_ref)?;
        let command = task_transition_command(
            clock,
            task_binding,
            &task,
            "CANDIDATE_COMPLETE",
            "COMPLETION_CLAIMED",
            [
                "required_effects_closed",
                "verification_requested_for_fixed_post_state",
            ],
            BTreeMap::from([
                ("completion_claim".to_owned(), claim_evidence),
                (
                    "fixed_post_state".to_owned(),
                    strong_reference_for_content(
                        &initial_facts.fixed_post_state.fixed_post_state_id,
                        1,
                        &initial_facts.fixed_post_state.canonical_json,
                    )
                    .map_err(|error| TaskCompletionError::EvidenceUnavailable(error.to_string()))?,
                ),
            ]),
            "principal://personal/daemon",
            TASK_LIFECYCLE_AUTHORITY_REF,
            writer_lease.epoch,
        )?;
        let prepared = TransitionEngine::new(store, clock, identifiers)
            .prepare_transition(&command)
            .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?;
        let receipt = store
            .claim_task_completion_atomically(&TaskCompletionClaimCommit {
                task_binding: task_binding.clone(),
                task_contract_id: initial_facts.contract_row.contract_id.clone(),
                fixed_post_state_id: initial_facts.fixed_post_state.fixed_post_state_id.clone(),
                verification_request_id: initial_facts
                    .verification_request
                    .verification_request_id
                    .clone(),
                effect_object_ids: initial_facts.effect_object_ids.clone(),
                transition: prepared.commit.clone(),
                fencing_epoch: writer_lease.epoch,
            })
            .map_err(store_error)?;
        Some(committed_transition(&prepared, receipt.event_sequence))
    } else {
        None
    };

    task = load_task(store, &initial_contract.contract_id)?;
    if task.state.as_str() != "CANDIDATE_COMPLETE" {
        return Err(TaskCompletionError::TaskUnavailable);
    }
    let current_facts = load_current_acceptance_facts(
        store,
        artifact_store,
        task_binding,
        verification_report_id,
        writer_lease,
    )?;
    let decision_id = next_object_id(identifiers)?;
    let (acceptance_decision_artifact_ref, acceptance_decision_evidence) =
        persist_canonical_artifact(
            artifact_store,
            &decision_id,
            &json!({
                "acceptance_authority": ACCEPTANCE_AUTHORITY_REF,
                "acceptance_principal": ACCEPTANCE_PRINCIPAL_REF,
                "artifact_evidence_refs": current_facts.artifact_evidence_refs,
                "decision": "granted",
                "effect_object_ids": current_facts.effect_object_ids,
                "fixed_post_state_id": current_facts.fixed_post_state.fixed_post_state_id.as_str(),
                "recorded_fencing_epoch": writer_lease.epoch,
                "schema_version": 1,
                "task_ref": task_binding.task_ref,
                "contract_epoch": task_binding.contract_epoch,
                "verification_report_id": current_facts.verification_report.verification_report_id.as_str(),
            }),
        )?;
    ensure_artifact_available(artifact_store, &acceptance_decision_artifact_ref)?;
    let command = task_transition_command(
        clock,
        task_binding,
        &task,
        "COMPLETED",
        "ACCEPTANCE_GRANTED",
        [
            "acceptance_authority_matches",
            "verification_passed_and_current",
            "fixed_post_state_unchanged",
        ],
        BTreeMap::from([
            (
                "verification_report".to_owned(),
                strong_reference_for_content(
                    &current_facts.verification_report.verification_report_id,
                    1,
                    &current_facts.verification_report.canonical_json,
                )
                .map_err(|error| TaskCompletionError::EvidenceUnavailable(error.to_string()))?,
            ),
            (
                "acceptance_decision".to_owned(),
                acceptance_decision_evidence,
            ),
        ]),
        ACCEPTANCE_PRINCIPAL_REF,
        ACCEPTANCE_AUTHORITY_REF,
        writer_lease.epoch,
    )?;
    let prepared = TransitionEngine::new(store, clock, identifiers)
        .prepare_transition(&command)
        .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?;
    let receipt = store
        .accept_task_completion_atomically(&TaskAcceptanceCommit {
            task_binding: task_binding.clone(),
            task_contract_id: current_facts.contract_row.contract_id,
            fixed_post_state_id: current_facts.fixed_post_state.fixed_post_state_id,
            verification_report_id: current_facts.verification_report.verification_report_id,
            effect_object_ids: current_facts.effect_object_ids,
            acceptance_decision_artifact_ref: acceptance_decision_artifact_ref.clone(),
            transition: prepared.commit.clone(),
            fencing_epoch: writer_lease.epoch,
        })
        .map_err(store_error)?;
    Ok(ProductionTaskCompletionOutcome {
        candidate,
        acceptance: committed_transition(&prepared, receipt.event_sequence),
        acceptance_decision_artifact_ref,
    })
}

fn load_current_acceptance_facts<S>(
    store: &S,
    artifact_store: &ArtifactStore,
    task_binding: &TaskBinding,
    verification_report_id: &ObjectId,
    writer_lease: &WriterLease,
) -> Result<CurrentAcceptanceFacts, TaskCompletionError>
where
    S: AuthorityStore + ContinuationAuthorityStore + IntentChainStore + ProtocolStore,
{
    verify_writer(store, writer_lease)?;
    let contract_row = load_current_contract(store, task_binding)?;
    let verification_report = store
        .load_verification_report(verification_report_id)
        .map_err(infrastructure)?
        .ok_or(TaskCompletionError::VerificationUnavailable)?;
    let latest_report = store
        .load_latest_verification_report_for_request(&verification_report.verification_request_id)
        .map_err(infrastructure)?
        .ok_or(TaskCompletionError::VerificationUnavailable)?;
    if latest_report.verification_report_id != *verification_report_id
        || verification_report.status != "passed"
        || verification_report.recorded_fencing_epoch != writer_lease.epoch
    {
        return Err(TaskCompletionError::VerificationUnavailable);
    }
    let verification_request = store
        .load_verification_request(&verification_report.verification_request_id)
        .map_err(infrastructure)?
        .ok_or(TaskCompletionError::VerificationUnavailable)?;
    let fixed_post_state = store
        .load_fixed_post_state(&verification_report.fixed_post_state_id)
        .map_err(infrastructure)?
        .ok_or(TaskCompletionError::VerificationUnavailable)?;
    if verification_request.task_binding != *task_binding
        || fixed_post_state.task_binding != *task_binding
        || verification_request.fixed_post_state_id != fixed_post_state.fixed_post_state_id
        || verification_report.fixed_post_state_id != fixed_post_state.fixed_post_state_id
        || verification_report.verifier_ref != verification_request.verifier_ref
        || verification_report.verifier_version != verification_request.verifier_version
        || verification_request.issued_fencing_epoch != writer_lease.epoch
        || fixed_post_state.recorded_fencing_epoch != writer_lease.epoch
        || fixed_post_state.subject_domain != LifecycleDomain::Effect
    {
        return Err(TaskCompletionError::VerificationUnavailable);
    }
    let artifact_evidence_refs =
        crate::personal::verification_executor::validate_persisted_report_artifacts(
            &verification_report,
            artifact_store,
        )
        .map_err(|error| TaskCompletionError::EvidenceUnavailable(error.to_string()))?;
    let intents = store
        .list_intents_for_task_binding(task_binding)
        .map_err(infrastructure)?;
    if intents.is_empty() {
        return Err(TaskCompletionError::EffectsOpen);
    }
    let mut effect_object_ids = Vec::with_capacity(intents.len());
    let mut fixed_subject_current = false;
    for intent in intents {
        let effect = store
            .load_object(LifecycleDomain::Effect, &intent.effect_object_id)
            .map_err(infrastructure)?
            .ok_or(TaskCompletionError::EffectsOpen)?;
        if !matches!(
            effect.state.as_str(),
            "RECONCILED" | "VERIFIED" | "VERIFY_FAILED"
        ) {
            return Err(TaskCompletionError::EffectsOpen);
        }
        if effect.object_id == fixed_post_state.subject_object_id {
            fixed_subject_current = effect.version == fixed_post_state.subject_version;
        }
        effect_object_ids.push(effect.object_id);
    }
    effect_object_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    if !fixed_subject_current {
        return Err(TaskCompletionError::VerificationUnavailable);
    }
    Ok(CurrentAcceptanceFacts {
        contract_row,
        fixed_post_state,
        verification_request,
        verification_report,
        artifact_evidence_refs,
        effect_object_ids,
    })
}

fn load_current_contract<S>(
    store: &S,
    task_binding: &TaskBinding,
) -> Result<TaskContractRow, TaskCompletionError>
where
    S: IntentChainStore + ProtocolStore,
{
    let current_epoch = store
        .current_contract_epoch(&task_binding.task_ref)
        .map_err(infrastructure)?;
    if current_epoch != task_binding.contract_epoch {
        return Err(TaskCompletionError::ContractUnavailable);
    }
    store
        .load_task_contract(&task_binding.task_ref, task_binding.contract_epoch)
        .map_err(infrastructure)?
        .ok_or(TaskCompletionError::ContractUnavailable)
}

fn parse_contract(contract_row: &TaskContractRow) -> Result<TaskContract, TaskCompletionError> {
    let contract: TaskContract = serde_json::from_str(&contract_row.canonical_json)
        .map_err(|_| TaskCompletionError::ContractUnavailable)?;
    if contract.task_ref != contract_row.task_ref
        || contract.contract_epoch != contract_row.contract_epoch
        || contract.header.id.0 != contract_row.contract_id.as_str()
    {
        return Err(TaskCompletionError::ContractUnavailable);
    }
    Ok(contract)
}

fn verify_writer<S>(store: &S, writer_lease: &WriterLease) -> Result<(), TaskCompletionError>
where
    S: ProtocolStore,
{
    let current_epoch = store.current_fencing_epoch().map_err(infrastructure)?;
    if current_epoch != writer_lease.epoch {
        return Err(TaskCompletionError::WriterFenced);
    }
    Ok(())
}

fn load_task<S>(store: &S, task_id: &ObjectId) -> Result<StoredObject, TaskCompletionError>
where
    S: AuthorityStore,
{
    store
        .load_object(LifecycleDomain::Task, task_id)
        .map_err(infrastructure)?
        .ok_or(TaskCompletionError::TaskUnavailable)
}

#[allow(clippy::too_many_arguments)]
fn task_transition_command<const N: usize>(
    clock: &impl Clock,
    task_binding: &TaskBinding,
    task: &StoredObject,
    to: &str,
    reason: &str,
    guards: [&str; N],
    evidence: BTreeMap<String, StrongReference>,
    actor_ref: &str,
    authority_ref: &str,
    fencing_epoch: i64,
) -> Result<TransitionCommand, TaskCompletionError> {
    let target_state = StateName::parse(to)
        .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?;
    let reason_code = ReasonCode::parse(reason)
        .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?;
    let correlation = UriRef::parse("correlation://personal/task-acceptance")
        .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?;
    Ok(TransitionCommand {
        request_id: UriRef::parse(&format!(
            "request://personal/task/{}/{}/{}",
            task.object_id,
            task.version.get(),
            to.to_ascii_lowercase()
        ))
        .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?,
        domain: LifecycleDomain::Task,
        object_id: task.object_id.clone(),
        subject_ref: UriRef::parse(&task_binding.task_ref)
            .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?,
        from: task.state.clone(),
        to: target_state,
        expected_version: task.version,
        reason: Reason {
            code: reason_code,
            detail: None,
        },
        causation: Causation {
            causation_id: correlation.clone(),
            correlation_id: correlation,
        },
        actor_ref: UriRef::parse(actor_ref)
            .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?,
        authority_ref: UriRef::parse(authority_ref)
            .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?,
        requested_at: clock
            .now()
            .map_err(|error| TaskCompletionError::Infrastructure(error.detail))?,
        table_pin: TablePin::current(LifecycleDomain::Task)
            .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))?,
        established_guards: guards
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>(),
        evidence,
        budget: None,
        outbox_destinations: Vec::new(),
        fencing_epoch: Some(fencing_epoch),
    })
}

fn persist_canonical_artifact(
    artifact_store: &ArtifactStore,
    object_id: &ObjectId,
    value: &serde_json::Value,
) -> Result<(String, StrongReference), TaskCompletionError> {
    let bytes = cognitive_contracts::canonical::canonical_bytes_of_value(value)
        .map_err(|error| TaskCompletionError::EvidenceUnavailable(error.to_string()))?;
    let storage_reference = artifact_store
        .put(&bytes)
        .map_err(|error| TaskCompletionError::EvidenceUnavailable(error.to_string()))?;
    let digest = storage_reference.strip_prefix("sha256:").ok_or_else(|| {
        TaskCompletionError::EvidenceUnavailable(
            "ArtifactStore returned a malformed digest".to_owned(),
        )
    })?;
    Ok((
        format!("artifact://sha256/{digest}"),
        StrongReference {
            content_digest: Digest(storage_reference),
            id: object_id.to_generated(),
            kind: StrongReferenceKind::Strong,
            object_version: 1,
        },
    ))
}

fn ensure_artifact_available(
    artifact_store: &ArtifactStore,
    artifact_ref: &str,
) -> Result<(), TaskCompletionError> {
    if !artifact_store
        .contains_artifact_uri(artifact_ref)
        .map_err(|error| TaskCompletionError::EvidenceUnavailable(error.to_string()))?
    {
        return Err(TaskCompletionError::EvidenceUnavailable(
            artifact_ref.to_owned(),
        ));
    }
    Ok(())
}

fn next_object_id(identifiers: &impl IdGenerator) -> Result<ObjectId, TaskCompletionError> {
    ObjectId::parse(
        &identifiers
            .next_uuid_v7()
            .map_err(|error| TaskCompletionError::Infrastructure(error.detail))?,
    )
    .map_err(|error| TaskCompletionError::Infrastructure(error.to_string()))
}

fn committed_transition(
    prepared: &cognitive_kernel::engine::PreparedTransition,
    event_sequence: i64,
) -> CommittedTransition {
    CommittedTransition {
        record_id: prepared.record_id.clone(),
        event_id: prepared.event_id.clone(),
        event_sequence,
        after_version: prepared.after_version,
        committed_at: prepared.committed_at.clone(),
    }
}

fn infrastructure(error: StorePortError) -> TaskCompletionError {
    TaskCompletionError::Infrastructure(error.to_string())
}

fn store_error(error: StorePortError) -> TaskCompletionError {
    match error {
        StorePortError::Conflict { detail } => TaskCompletionError::EvidenceUnavailable(detail),
        StorePortError::Unavailable { detail } => TaskCompletionError::Infrastructure(detail),
    }
}
