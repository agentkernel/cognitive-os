//! Deterministic construction of one daemon-only candidate-admission bundle.
//!
//! The composer deliberately has no store dependency and does not persist,
//! dispatch, consume a worker authorization, record progress, or complete a
//! Task. The daemon reloads and validates facts first, passes only those facts
//! here, then commits the returned bundle through exactly one store method.

use crate::budget::BudgetCharge;
use crate::effects::{
    GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN, OperationDescriptor, WriterLease, canonical_text,
    strong_reference_for_content,
};
use crate::engine::{
    EVENT_TYPE_OBJECT_ADMITTED, EVENT_TYPE_TRANSITION_COMMITTED, validate_registered_transition,
};
use crate::intent_chain::{
    GovernanceSeed, compose_governed_header, seal_governed_object_content_digest,
    strong_reference_to,
};
use crate::ports::{
    BudgetCas, CandidateAdmissionCommit, DaemonAuthorizationSnapshotRow,
    DaemonOperationDescriptorRow, EventDraft, IntentRow, ObjectAdmission, ObjectCas,
    OperationCandidateProposalRow, RecordDraft, StoredObject, TaskBinding, TaskContractRow,
    TransitionCommit, WorkerIterationAuthorizationRow,
};
use cognitive_contracts::canonical;
use cognitive_contracts::generated::common_defs::{Budget, Digest};
use cognitive_contracts::generated::effect::{
    Effect, EffectDecision, EffectObservedOutcome, EffectState, EffectVerificationStanding,
    EffectVerificationStandingStatus,
};
use cognitive_contracts::generated::operation_candidate_proposal::OperationCandidateProposal;
use cognitive_contracts::generated::state_transition_record::CommittedStateTransitionRecord;
use cognitive_contracts::generated::state_transition_request::{Causation, Reason};
use cognitive_contracts::generated::task_contract::TaskContract;
use cognitive_contracts::generated::worker_iteration_authorization::WorkerIterationAuthorization;
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, RecordId, StateName, UriRef, Version,
    WallTimestamp,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

const EFFECT_SCHEMA_VERSION: &str = "cognitiveos.effect/0.2";
const WORKER_AUTHORIZATION_SCHEMA_VERSION: &str = "cognitiveos.worker-iteration-authorization/0.1";
const LOOP_OPERATION_ADMITTED_REASON: &str = "OPERATION_ADMITTED";

/// Daemon-generated identities for one candidate-admission bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateAdmissionIdentities {
    pub authorization_id: ObjectId,
    pub intent_id: ObjectId,
    pub effect_object_id: ObjectId,
    pub intent_event_id: EventId,
    pub effect_event_id: EventId,
    pub loop_event_id: EventId,
    pub loop_record_id: RecordId,
}

/// Immutable facts returned by the daemon's authority-read preflight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateAdmissionFacts {
    pub loop_object_id: ObjectId,
    pub budget_id: BudgetId,
    pub expected_budget_version: Version,
    pub next_budget_state_canonical_json: String,
    pub expected_loop_version: Version,
    pub iteration: i64,
}

/// Fully daemon-owned input to the non-persisting candidate-admission
/// composer. Callers must reload the candidate, contract, descriptor and
/// authorization snapshot from durable daemon registries before construction.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateAdmissionInputs {
    pub candidate: OperationCandidateProposalRow,
    pub task_contract: TaskContractRow,
    pub descriptor: DaemonOperationDescriptorRow,
    pub authorization: DaemonAuthorizationSnapshotRow,
    pub authorization_subject_ref: String,
    pub authorization_purpose: String,
    pub facts: CandidateAdmissionFacts,
    pub budget_charge: BudgetCharge,
    pub governance: GovernanceSeed,
    pub identities: CandidateAdmissionIdentities,
    pub actor_ref: UriRef,
    pub authority_ref: UriRef,
    pub correlation_id: UriRef,
    pub admitted_at: WallTimestamp,
    pub writer_lease: WriterLease,
}

/// Fail-closed composition error. No authority has been persisted when this
/// is returned; callers must not reconstruct partial bundle members.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CandidateAdmissionCompositionError {
    #[error("candidate admission inputs are inconsistent: {0}")]
    Inconsistent(String),
    #[error("candidate admission payload is malformed: {0}")]
    Malformed(String),
}

fn inconsistent(detail: impl Into<String>) -> CandidateAdmissionCompositionError {
    CandidateAdmissionCompositionError::Inconsistent(detail.into())
}

fn malformed(detail: impl Into<String>) -> CandidateAdmissionCompositionError {
    CandidateAdmissionCompositionError::Malformed(detail.into())
}

fn canonical_digest(value: &Value) -> Result<String, CandidateAdmissionCompositionError> {
    let bytes =
        canonical::canonical_bytes_of_value(value).map_err(|error| malformed(error.to_string()))?;
    canonical::digest(&bytes, GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN)
        .map_err(|error| malformed(error.to_string()))
}

fn canonical_string(value: &Value) -> Result<String, CandidateAdmissionCompositionError> {
    canonical_text(value).map_err(|error| malformed(error.to_string()))
}

fn canonical_budget(charge: &BudgetCharge) -> Budget {
    let amounts = charge.amounts();
    Budget {
        attention_slots: amounts.get("attention_slots").copied(),
        context_bytes: amounts.get("context_bytes").copied(),
        egress_bytes: amounts.get("egress_bytes").copied(),
        input_tokens: amounts.get("input_tokens").copied(),
        money_microunits: amounts.get("money_microunits").copied(),
        output_tokens: amounts.get("output_tokens").copied(),
        semantic_calls: amounts.get("semantic_calls").copied(),
        tool_calls: amounts.get("tool_calls").copied(),
        wall_time_ms: amounts.get("wall_time_ms").copied(),
    }
}

fn parse_candidate(
    candidate: &OperationCandidateProposalRow,
) -> Result<OperationCandidateProposal, CandidateAdmissionCompositionError> {
    serde_json::from_str(&candidate.canonical_json).map_err(|error| {
        malformed(format!(
            "candidate {} is not schema-shaped: {error}",
            candidate.candidate_id
        ))
    })
}

fn parse_contract(
    contract: &TaskContractRow,
) -> Result<TaskContract, CandidateAdmissionCompositionError> {
    serde_json::from_str(&contract.canonical_json).map_err(|error| {
        malformed(format!(
            "TaskContract {} is not schema-shaped: {error}",
            contract.contract_id
        ))
    })
}

fn verify_sealed_governed_payload(
    payload: &Value,
    payload_name: &str,
) -> Result<(), CandidateAdmissionCompositionError> {
    cognitive_contracts::projection::verify_content_digest(
        payload,
        &["/header/content_digest"],
        GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN,
        "/header/content_digest",
    )
    .map_err(|error| malformed(format!("{payload_name} content digest is invalid: {error}")))
}

fn action_fingerprint(
    candidate: &OperationCandidateProposalRow,
    descriptor: &OperationDescriptor,
) -> Result<String, CandidateAdmissionCompositionError> {
    canonical_digest(&json!({
        "action": candidate.action,
        "descriptor_version": descriptor.descriptor_version,
        "operation_id": descriptor.operation_id,
        "parameters_digest": candidate.parameters_digest,
        "target": candidate.target,
        "tool_ref": candidate.tool_ref,
    }))
}

/// Compose every member of one candidate-admission transaction.
///
/// The returned value must be handed unchanged to
/// [`crate::ports::WorkerAuthorizationStore::commit_candidate_admission`].
/// This function intentionally cannot write to the store, invoke an executor,
/// consume the resulting WIA, or claim progress or Task completion.
pub fn compose_candidate_admission(
    inputs: &CandidateAdmissionInputs,
) -> Result<CandidateAdmissionCommit, CandidateAdmissionCompositionError> {
    let candidate_payload = parse_candidate(&inputs.candidate)?;
    let contract_payload = parse_contract(&inputs.task_contract)?;
    let candidate_value: Value = serde_json::from_str(&inputs.candidate.canonical_json)
        .map_err(|error| malformed(error.to_string()))?;
    let contract_value: Value = serde_json::from_str(&inputs.task_contract.canonical_json)
        .map_err(|error| malformed(error.to_string()))?;
    verify_sealed_governed_payload(&candidate_value, "candidate")?;
    verify_sealed_governed_payload(&contract_value, "TaskContract")?;
    if candidate_payload.header.id.0.as_str() != inputs.candidate.candidate_id.as_str() {
        return Err(inconsistent(
            "candidate header identity differs from candidate row",
        ));
    }
    if contract_payload.header.id.0.as_str() != inputs.task_contract.contract_id.as_str() {
        return Err(inconsistent(
            "TaskContract header identity differs from contract row",
        ));
    }
    if inputs.candidate.task_ref != inputs.task_contract.task_ref
        || inputs.candidate.contract_epoch != inputs.task_contract.contract_epoch
        || inputs.candidate.contract_epoch != contract_payload.contract_epoch
        || inputs.facts.iteration < 1
    {
        return Err(inconsistent(
            "candidate, TaskContract, or iteration binding is invalid",
        ));
    }
    let worker_authorization_root_id = contract_payload
        .worker_authorization_root_id
        .as_ref()
        .ok_or_else(|| inconsistent("TaskContract has no worker authorization root"))?;
    let contract_loop_object_id = contract_payload
        .loop_object_id
        .as_ref()
        .ok_or_else(|| inconsistent("TaskContract has no loop object identity"))?;
    let contract_budget_id = contract_payload
        .budget_id
        .as_ref()
        .ok_or_else(|| inconsistent("TaskContract has no budget identity"))?;
    if !contract_payload
        .allowed_tools
        .contains(&inputs.candidate.tool_ref)
        || contract_loop_object_id.0.as_str() != inputs.facts.loop_object_id.as_str()
        || contract_budget_id.0.as_str() != inputs.facts.budget_id.as_str()
    {
        return Err(inconsistent(
            "TaskContract does not bind the candidate tool, Loop, and budget facts",
        ));
    }
    if inputs.descriptor.descriptor.operation_id != inputs.candidate.tool_ref
        || inputs.descriptor.descriptor.action != inputs.candidate.action
    {
        return Err(inconsistent(
            "descriptor does not bind the selected candidate",
        ));
    }
    if inputs.authorization.subject_ref != inputs.authorization_subject_ref
        || inputs.authorization.purpose != inputs.authorization_purpose
        || inputs.authorization.target_ref != inputs.candidate.target
        || inputs.authorization.action != inputs.candidate.action
        || inputs.authorization.grant_epoch < 1
        || inputs.authorization.capability_set_version < 1
    {
        return Err(inconsistent(
            "authorization snapshot does not cover the selected candidate",
        ));
    }

    let candidate_reference = strong_reference_to(
        &inputs.candidate.candidate_id,
        &candidate_payload.header.content_digest.0,
    );
    let contract_reference = strong_reference_to(
        &inputs.task_contract.contract_id,
        &contract_payload.header.content_digest.0,
    );
    let parameters_digest = Digest(inputs.candidate.parameters_digest.clone());
    let verified_parameters = candidate_payload
        .parameters
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|error| malformed(error.to_string()))?;
    let action_fingerprint = action_fingerprint(&inputs.candidate, &inputs.descriptor.descriptor)?;
    let budget_charge = canonical_budget(&inputs.budget_charge);
    let budget_charge_value =
        serde_json::to_value(&budget_charge).map_err(|error| malformed(error.to_string()))?;
    let budget_charge_canonical_json = canonical_string(&budget_charge_value)?;

    let mut intent_value = json!({
        "action": inputs.candidate.action,
        "capability_set_version": inputs.authorization.capability_set_version,
        "contract_epoch": inputs.task_contract.contract_epoch,
        "effect_object_id": inputs.identities.effect_object_id.as_str(),
        "executor": inputs.descriptor.descriptor.executor,
        "expected_state_version": inputs.candidate.expected_state_version,
        "grant_epoch": inputs.authorization.grant_epoch,
        "idempotency_key": format!(
            "candidate-admission:{}:{}",
            inputs.candidate.candidate_id, inputs.task_contract.contract_epoch
        ),
        "intent_id": inputs.identities.intent_id.as_str(),
        "minted_at": inputs.admitted_at.as_str(),
        "operation_id": inputs.descriptor.descriptor.operation_id,
        "parameters_digest": inputs.candidate.parameters_digest,
        "target": inputs.candidate.target,
        "task_ref": inputs.task_contract.task_ref,
    });
    if let Some(parameters) = verified_parameters {
        intent_value["parameters"] = parameters;
    }
    let intent_canonical_json = canonical_string(&intent_value)?;
    let intent_reference = strong_reference_for_content(
        &inputs.identities.intent_id,
        Version::INITIAL.get(),
        &intent_canonical_json,
    )
    .map_err(|error| malformed(error.to_string()))?;
    let intent = IntentRow {
        intent_id: inputs.identities.intent_id.clone(),
        idempotency_key: format!(
            "candidate-admission:{}:{}",
            inputs.candidate.candidate_id, inputs.task_contract.contract_epoch
        ),
        parameters_digest: inputs.candidate.parameters_digest.clone(),
        action: inputs.candidate.action.clone(),
        target: inputs.candidate.target.clone(),
        effect_object_id: inputs.identities.effect_object_id.clone(),
        expected_state_version: Version::new(inputs.candidate.expected_state_version)
            .map_err(|error| malformed(error.to_string()))?,
        grant_epoch: inputs.authorization.grant_epoch,
        capability_set_version: inputs.authorization.capability_set_version,
        task_binding: Some(TaskBinding {
            task_ref: inputs.task_contract.task_ref.clone(),
            contract_epoch: inputs.task_contract.contract_epoch,
        }),
        canonical_json: intent_canonical_json,
    };
    let intent_event = EventDraft {
        event_id: inputs.identities.intent_event_id.clone(),
        object_id: inputs.identities.intent_id.clone(),
        domain: LifecycleDomain::Effect,
        object_version: Version::INITIAL,
        event_type: crate::replay::EVENT_TYPE_INTENT_PERSISTED.to_owned(),
        canonical_json: canonical_string(&json!({
            "actor_ref": inputs.actor_ref.as_str(),
            "authority_ref": inputs.authority_ref.as_str(),
            "causation": {
                "causation_id": inputs.correlation_id.as_str(),
                "correlation_id": inputs.correlation_id.as_str(),
            },
            "effect_object_id": inputs.identities.effect_object_id.as_str(),
            "event_id": inputs.identities.intent_event_id.as_str(),
            "event_time": inputs.admitted_at.as_str(),
            "event_type": crate::replay::EVENT_TYPE_INTENT_PERSISTED,
            "fencing_epoch": inputs.writer_lease.epoch,
            "idempotency_key": intent.idempotency_key,
            "object_id": inputs.identities.intent_id.as_str(),
            "parameters_digest": intent.parameters_digest,
        }))?,
    };

    let authorization_digest = canonical::digest(
        inputs.authorization.canonical_json.as_bytes(),
        GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN,
    )
    .map_err(|error| malformed(error.to_string()))?;
    let effect_header = compose_governed_header(
        &inputs.identities.effect_object_id,
        "Effect",
        EFFECT_SCHEMA_VERSION,
        &inputs.governance,
        vec![inputs.candidate.candidate_source_ref.clone()],
        vec![
            inputs.candidate.candidate_id.to_string(),
            inputs.identities.intent_id.to_string(),
        ],
        "candidate-admission-effect",
        &inputs.admitted_at,
    )
    .map_err(|error| malformed(error.to_string()))?;
    let effect = Effect {
        attempt: inputs.facts.iteration,
        authorization_digest: Digest(authorization_digest),
        decision: EffectDecision::Pending,
        event_refs: vec![inputs.identities.intent_event_id.to_string()],
        executor: inputs.descriptor.descriptor.executor.clone(),
        fencing_token: Some(inputs.writer_lease.epoch),
        header: effect_header,
        idempotency_key: intent.idempotency_key.clone(),
        intent_ref: intent_reference.clone(),
        observed_outcome: EffectObservedOutcome::NotObserved,
        parameters_digest: parameters_digest.clone(),
        receipt_ref: None,
        reconciliation_report_ref: None,
        reconciliation_result: None,
        state: EffectState::Proposed,
        verification: EffectVerificationStanding {
            report_ref: None,
            status: EffectVerificationStandingStatus::Pending,
        },
    };
    let effect_value =
        serde_json::to_value(&effect).map_err(|error| malformed(error.to_string()))?;
    let (sealed_effect, effect_digest) = seal_governed_object_content_digest(effect_value)
        .map_err(|error| malformed(error.to_string()))?;
    let effect_reference = strong_reference_to(&inputs.identities.effect_object_id, &effect_digest);
    let effect_admission = ObjectAdmission {
        object: StoredObject {
            object_id: inputs.identities.effect_object_id.clone(),
            domain: LifecycleDomain::Effect,
            state: StateName::parse("PROPOSED").map_err(|error| malformed(error.to_string()))?,
            version: Version::INITIAL,
            body: sealed_effect,
        },
        admitted_at: inputs.admitted_at.clone(),
        event: EventDraft {
            event_id: inputs.identities.effect_event_id.clone(),
            object_id: inputs.identities.effect_object_id.clone(),
            domain: LifecycleDomain::Effect,
            object_version: Version::INITIAL,
            event_type: EVENT_TYPE_OBJECT_ADMITTED.to_owned(),
            canonical_json: canonical_string(&json!({
                "actor_ref": inputs.actor_ref.as_str(),
                "authority_ref": inputs.authority_ref.as_str(),
                "correlation_id": inputs.correlation_id.as_str(),
                "domain": "effect",
                "event_id": inputs.identities.effect_event_id.as_str(),
                "event_time": inputs.admitted_at.as_str(),
                "event_type": EVENT_TYPE_OBJECT_ADMITTED,
                "initial_state": "PROPOSED",
                "object_id": inputs.identities.effect_object_id.as_str(),
            }))?,
        },
        outbox: Vec::new(),
        fencing_epoch: Some(inputs.writer_lease.epoch),
    };

    let authorization_header = compose_governed_header(
        &inputs.identities.authorization_id,
        "WorkerIterationAuthorization",
        WORKER_AUTHORIZATION_SCHEMA_VERSION,
        &inputs.governance,
        vec![inputs.candidate.candidate_source_ref.clone()],
        vec![
            inputs.task_contract.contract_id.to_string(),
            inputs.candidate.candidate_id.to_string(),
            inputs.identities.intent_id.to_string(),
            inputs.identities.effect_object_id.to_string(),
        ],
        "candidate-admission-worker-authorization",
        &inputs.admitted_at,
    )
    .map_err(|error| malformed(error.to_string()))?;
    let authorization_payload = WorkerIterationAuthorization {
        action_fingerprint: action_fingerprint.clone(),
        budget_charge: budget_charge.clone(),
        budget_id: inputs.facts.budget_id.to_generated(),
        contract_epoch: inputs.task_contract.contract_epoch,
        effect_ref: effect_reference,
        expected_loop_version: inputs.facts.expected_loop_version.get(),
        header: authorization_header,
        intent_ref: intent_reference,
        issued_fencing_epoch: inputs.writer_lease.epoch,
        iteration: inputs.facts.iteration,
        selected_candidate_ref: candidate_reference.clone(),
        task_contract_ref: contract_reference,
        worker_authorization_root_id: worker_authorization_root_id.clone(),
    };
    let authorization_value = serde_json::to_value(&authorization_payload)
        .map_err(|error| malformed(error.to_string()))?;
    let (sealed_authorization, _) = seal_governed_object_content_digest(authorization_value)
        .map_err(|error| malformed(error.to_string()))?;
    let worker_authorization = WorkerIterationAuthorizationRow {
        authorization_id: inputs.identities.authorization_id.clone(),
        worker_authorization_root_id: ObjectId::parse(&worker_authorization_root_id.0)
            .map_err(|error| malformed(error.to_string()))?,
        task_ref: inputs.task_contract.task_ref.clone(),
        contract_epoch: inputs.task_contract.contract_epoch,
        loop_object_id: inputs.facts.loop_object_id.clone(),
        iteration: inputs.facts.iteration,
        expected_loop_version: inputs.facts.expected_loop_version,
        selected_candidate_id: inputs.candidate.candidate_id.clone(),
        intent_id: inputs.identities.intent_id.clone(),
        effect_object_id: inputs.identities.effect_object_id.clone(),
        budget_id: inputs.facts.budget_id.clone(),
        budget_charge_canonical_json: budget_charge_canonical_json.clone(),
        action_fingerprint,
        issued_fencing_epoch: inputs.writer_lease.epoch,
        canonical_json: canonical_string(&sealed_authorization)?,
    };

    let authorization_snapshot_reference = strong_reference_for_content(
        &inputs.authorization.snapshot_id,
        Version::INITIAL.get(),
        &inputs.authorization.canonical_json,
    )
    .map_err(|error| malformed(error.to_string()))?;
    let loop_from_state =
        StateName::parse("DECIDE").map_err(|error| malformed(error.to_string()))?;
    let loop_to_state = StateName::parse("ACT").map_err(|error| malformed(error.to_string()))?;
    let table_pin = validate_registered_transition(
        LifecycleDomain::Loop,
        &loop_from_state,
        &loop_to_state,
        LOOP_OPERATION_ADMITTED_REASON,
        &BTreeSet::from([
            "proposal_bounded".to_owned(),
            "authorization_granted".to_owned(),
            "hard_budget_available".to_owned(),
        ]),
        &BTreeSet::from([
            "operation_proposal".to_owned(),
            "authorization_decision".to_owned(),
        ]),
    )
    .map_err(|error| malformed(error.to_string()))?;
    let next_loop_version = Version::new(inputs.facts.expected_loop_version.get() + 1)
        .map_err(|error| malformed(error.to_string()))?;
    let record = CommittedStateTransitionRecord {
        actor_ref: inputs.actor_ref.as_str().to_owned(),
        after_state: "ACT".to_owned(),
        after_version: next_loop_version.get(),
        authority_ref: inputs.authority_ref.as_str().to_owned(),
        before_state: "DECIDE".to_owned(),
        before_version: inputs.facts.expected_loop_version.get(),
        causation: Causation {
            causation_id: inputs.correlation_id.as_str().to_owned(),
            correlation_id: inputs.correlation_id.as_str().to_owned(),
            request_ref: Some(inputs.identities.authorization_id.to_string()),
        },
        committed_at: inputs.admitted_at.as_str().to_owned(),
        domain: "loop".to_owned(),
        event_ref: None,
        evidence_refs: vec![candidate_reference, authorization_snapshot_reference],
        expected_version: inputs.facts.expected_loop_version.get(),
        metadata: None,
        reason: Reason {
            code: LOOP_OPERATION_ADMITTED_REASON.to_owned(),
            detail: Some("daemon admitted a bounded operation candidate".to_owned()),
        },
        record_id: inputs.identities.loop_record_id.to_string(),
        request_ref: inputs.identities.authorization_id.to_string(),
        requested_at: inputs.admitted_at.as_str().to_owned(),
        subject_ref: inputs.facts.loop_object_id.to_string(),
        table_digest: table_pin.digest.clone(),
        table_version: table_pin.version.clone(),
    };
    let loop_transition = TransitionCommit {
        cas: ObjectCas {
            object_id: inputs.facts.loop_object_id.clone(),
            domain: LifecycleDomain::Loop,
            from_state: loop_from_state,
            to_state: loop_to_state,
            expected_version: inputs.facts.expected_loop_version,
            next_version: next_loop_version,
            committed_at: inputs.admitted_at.clone(),
        },
        event: EventDraft {
            event_id: inputs.identities.loop_event_id.clone(),
            object_id: inputs.facts.loop_object_id.clone(),
            domain: LifecycleDomain::Loop,
            object_version: next_loop_version,
            event_type: EVENT_TYPE_TRANSITION_COMMITTED.to_owned(),
            canonical_json: canonical_string(&json!({
                "after_state": "ACT",
                "after_version": next_loop_version.get(),
                "before_state": "DECIDE",
                "before_version": inputs.facts.expected_loop_version.get(),
                "causation": {
                    "causation_id": inputs.correlation_id.as_str(),
                    "correlation_id": inputs.correlation_id.as_str(),
                    "request_ref": inputs.identities.authorization_id.to_string(),
                },
                "domain": "loop",
                "event_id": inputs.identities.loop_event_id.as_str(),
                "event_time": inputs.admitted_at.as_str(),
                "event_type": EVENT_TYPE_TRANSITION_COMMITTED,
                "object_id": inputs.facts.loop_object_id.to_string(),
                "reason": record.reason.clone(),
                "record_ref": inputs.identities.loop_record_id.to_string(),
                "subject_ref": inputs.facts.loop_object_id.to_string(),
                "table_digest": table_pin.digest,
                "table_version": table_pin.version,
            }))?,
        },
        record: RecordDraft {
            record_id: inputs.identities.loop_record_id.clone(),
            object_id: inputs.facts.loop_object_id.clone(),
            domain: LifecycleDomain::Loop,
            object_version: next_loop_version,
            canonical_json: canonical_string(
                &serde_json::to_value(&record).map_err(|error| malformed(error.to_string()))?,
            )?,
        },
        budget: Some(BudgetCas {
            budget_id: inputs.facts.budget_id.clone(),
            expected_version: inputs.facts.expected_budget_version,
            next_version: Version::new(inputs.facts.expected_budget_version.get() + 1)
                .map_err(|error| malformed(error.to_string()))?,
            charge_canonical_json: budget_charge_canonical_json.clone(),
            next_state_canonical_json: inputs.facts.next_budget_state_canonical_json.clone(),
        }),
        outbox: Vec::new(),
        fencing_epoch: Some(inputs.writer_lease.epoch),
    };

    Ok(CandidateAdmissionCommit {
        selected_candidate_id: inputs.candidate.candidate_id.clone(),
        intent,
        intent_event,
        effect_admission,
        worker_authorization,
        loop_transition,
        fencing_epoch: inputs.writer_lease.epoch,
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::effects::EffectClass;
    use crate::executor::ExecutorCapabilities;
    use cognitive_contracts::generated::governed_object_header::GovernedObjectHeaderSensitivity;
    use cognitive_contracts::generated::object_reference::UuidV7;
    use cognitive_contracts::generated::task_contract::{
        ContractCondition, ContractConditionKind, TaskScope,
    };
    use std::collections::BTreeMap;

    fn object_id(sequence: u64) -> ObjectId {
        ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}")).unwrap()
    }

    fn event_id(sequence: u64) -> EventId {
        EventId::parse(&format!("00000000-0000-7000-a000-{sequence:012x}")).unwrap()
    }

    fn record_id(sequence: u64) -> RecordId {
        RecordId::parse(&format!("00000000-0000-7000-8000-{sequence:012x}")).unwrap()
    }

    fn budget_id(sequence: u64) -> BudgetId {
        BudgetId::parse(&format!("00000000-0000-7000-b000-{sequence:012x}")).unwrap()
    }

    fn timestamp() -> WallTimestamp {
        WallTimestamp::parse("2026-08-04T12:00:00Z").unwrap()
    }

    fn uri(value: &str) -> UriRef {
        UriRef::parse(value).unwrap()
    }

    fn governance() -> GovernanceSeed {
        GovernanceSeed {
            owner: strong_reference_to(&object_id(901), &format!("sha256:{}", "a".repeat(64))),
            authority: strong_reference_to(&object_id(902), &format!("sha256:{}", "b".repeat(64))),
            resource_scope: strong_reference_to(
                &object_id(903),
                &format!("sha256:{}", "c".repeat(64)),
            ),
            tenant_id: Some("00000000-0000-7000-9000-0000000000f1".to_owned()),
            created_by: "principal://personal/daemon".to_owned(),
            sensitivity: GovernedObjectHeaderSensitivity::Internal,
            purpose_constraints: vec!["task_execution".to_owned()],
            retention_policy: "standard".to_owned(),
        }
    }

    fn sealed_canonical(value: Value) -> (String, String) {
        let (sealed, digest) = seal_governed_object_content_digest(value).unwrap();
        (canonical_string(&sealed).unwrap(), digest)
    }

    fn valid_inputs() -> CandidateAdmissionInputs {
        let contract_id = object_id(100);
        let candidate_id = object_id(101);
        let descriptor_id = object_id(102);
        let loop_id = object_id(103);
        let budget = budget_id(104);
        let contract_header = compose_governed_header(
            &contract_id,
            "TaskContract",
            "cognitiveos.task-contract/0.3",
            &governance(),
            Vec::new(),
            Vec::new(),
            "test-contract",
            &timestamp(),
        )
        .unwrap();
        let contract = TaskContract {
            allowed_state_domains: vec!["effect".to_owned(), "loop".to_owned()],
            allowed_tools: vec!["operation://personal/filesystem/read".to_owned()],
            budget: Budget {
                attention_slots: None,
                context_bytes: None,
                egress_bytes: None,
                input_tokens: None,
                money_microunits: None,
                output_tokens: None,
                semantic_calls: None,
                tool_calls: Some(2),
                wall_time_ms: None,
            },
            budget_id: Some(budget.to_generated()),
            conditions: vec![ContractCondition {
                description: "verified completion".to_owned(),
                id: "acceptance".to_owned(),
                kind: ContractConditionKind::Acceptance,
                machine_expression: None,
                verifier_ref: Some("verifier://personal/test".to_owned()),
            }],
            context_request_ref: None,
            contract_epoch: 1,
            deadline: None,
            header: contract_header,
            human_gates: None,
            intent_acceptance_ref: strong_reference_to(
                &object_id(110),
                &format!("sha256:{}", "d".repeat(64)),
            ),
            intent_interpretation_ref: strong_reference_to(
                &object_id(111),
                &format!("sha256:{}", "e".repeat(64)),
            ),
            loop_object_id: Some(loop_id.to_generated()),
            max_iterations: 2,
            max_retries: 1,
            objective: "read bounded input".to_owned(),
            scope: TaskScope {
                in_scope: vec!["input".to_owned()],
                out_of_scope: vec!["writes".to_owned()],
            },
            task_ref: "task://personal/candidate-admission".to_owned(),
            user_intent_ref: strong_reference_to(
                &object_id(112),
                &format!("sha256:{}", "f".repeat(64)),
            ),
            worker_authorization_root_id: Some(UuidV7(contract_id.to_string())),
        };
        let (contract_canonical_json, contract_digest) =
            sealed_canonical(serde_json::to_value(&contract).unwrap());
        let task_contract = TaskContractRow {
            contract_id: contract_id.clone(),
            task_ref: "task://personal/candidate-admission".to_owned(),
            contract_epoch: 1,
            user_intent_record_id: object_id(110),
            interpretation_id: object_id(111),
            accepted_by: "principal://personal/daemon".to_owned(),
            contract_digest,
            canonical_json: contract_canonical_json.clone(),
        };
        let sealed_contract: TaskContract = serde_json::from_str(&contract_canonical_json).unwrap();
        let candidate_header = compose_governed_header(
            &candidate_id,
            "OperationCandidateProposal",
            "cognitiveos.operation-candidate-proposal/0.1",
            &governance(),
            vec!["observation://personal/test".to_owned()],
            vec![contract_id.to_string()],
            "test-candidate",
            &timestamp(),
        )
        .unwrap();
        let candidate = OperationCandidateProposal {
            action: "filesystem.read".to_owned(),
            candidate_source_ref: "observation://personal/test".to_owned(),
            contract_epoch: 1,
            expected_state_version: 1,
            header: candidate_header,
            operation_descriptor_ref: strong_reference_to(
                &descriptor_id,
                &format!("sha256:{}", "1".repeat(64)),
            ),
            parameters: None,
            parameters_digest: format!("sha256:{}", "2".repeat(64)),
            target: "file:///workspace/input.txt".to_owned(),
            task_contract_ref: strong_reference_to(
                &contract_id,
                &sealed_contract.header.content_digest.0,
            ),
            tool_ref: "operation://personal/filesystem/read".to_owned(),
        };
        let (candidate_canonical_json, _) =
            sealed_canonical(serde_json::to_value(&candidate).unwrap());

        CandidateAdmissionInputs {
            candidate: OperationCandidateProposalRow {
                candidate_id: candidate_id.clone(),
                task_ref: "task://personal/candidate-admission".to_owned(),
                contract_epoch: 1,
                candidate_source_ref: "observation://personal/test".to_owned(),
                tool_ref: "operation://personal/filesystem/read".to_owned(),
                action: "filesystem.read".to_owned(),
                target: "file:///workspace/input.txt".to_owned(),
                parameters_digest: format!("sha256:{}", "2".repeat(64)),
                expected_state_version: 1,
                operation_descriptor_ref: descriptor_id.clone(),
                canonical_json: candidate_canonical_json,
            },
            task_contract,
            descriptor: DaemonOperationDescriptorRow {
                descriptor_id,
                descriptor: OperationDescriptor {
                    operation_id: "operation://personal/filesystem/read".to_owned(),
                    action: "filesystem.read".to_owned(),
                    effect_class: EffectClass::GovernedExternal,
                    executor: "executor://personal/filesystem".to_owned(),
                    capabilities: ExecutorCapabilities {
                        queryable: true,
                        idempotent: false,
                    },
                    descriptor_version: 1,
                },
                canonical_json: "{\"descriptor\":1}".to_owned(),
            },
            authorization: DaemonAuthorizationSnapshotRow {
                snapshot_id: object_id(105),
                subject_ref: "principal://personal/daemon".to_owned(),
                target_ref: "file:///workspace/input.txt".to_owned(),
                action: "filesystem.read".to_owned(),
                purpose: "task_execution".to_owned(),
                grant_epoch: 1,
                capability_set_version: 1,
                revocation_epoch: 1,
                observed_at: timestamp(),
                canonical_json: "{\"authorization\":1}".to_owned(),
            },
            authorization_subject_ref: "principal://personal/daemon".to_owned(),
            authorization_purpose: "task_execution".to_owned(),
            facts: CandidateAdmissionFacts {
                loop_object_id: loop_id,
                budget_id: budget,
                expected_budget_version: Version::INITIAL,
                next_budget_state_canonical_json: "{\"tool_calls\":1}".to_owned(),
                expected_loop_version: Version::INITIAL,
                iteration: 1,
            },
            budget_charge: BudgetCharge::new(BTreeMap::from([("tool_calls".to_owned(), 1)]))
                .unwrap(),
            governance: governance(),
            identities: CandidateAdmissionIdentities {
                authorization_id: object_id(120),
                intent_id: object_id(121),
                effect_object_id: object_id(122),
                intent_event_id: event_id(123),
                effect_event_id: event_id(124),
                loop_event_id: event_id(125),
                loop_record_id: record_id(126),
            },
            actor_ref: uri("principal://personal/daemon"),
            authority_ref: uri("authority://personal/daemon"),
            correlation_id: uri("correlation://personal/candidate-admission"),
            admitted_at: timestamp(),
            writer_lease: WriterLease { epoch: 1 },
        }
    }

    #[test]
    fn composer_emits_schema_shaped_effect_and_worker_authorization() {
        let commit = compose_candidate_admission(&valid_inputs()).unwrap();

        let effect: Effect =
            serde_json::from_value(commit.effect_admission.object.body.clone()).unwrap();
        let authorization: WorkerIterationAuthorization =
            serde_json::from_str(&commit.worker_authorization.canonical_json).unwrap();
        assert_eq!(effect.state, EffectState::Proposed);
        assert_eq!(authorization.iteration, 1);
        assert_eq!(authorization.expected_loop_version, 1);
        assert!(authorization.action_fingerprint.starts_with("sha256:"));
        assert_eq!(commit.loop_transition.cas.from_state.as_str(), "DECIDE");
        assert_eq!(commit.loop_transition.cas.to_state.as_str(), "ACT");
        assert!(commit.loop_transition.budget.is_some());
        cognitive_contracts::projection::verify_content_digest(
            &commit.effect_admission.object.body,
            &["/header/content_digest"],
            GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN,
            "/header/content_digest",
        )
        .expect("Effect content digest must bind the complete payload");
        let authorization_value: Value =
            serde_json::from_str(&commit.worker_authorization.canonical_json).unwrap();
        cognitive_contracts::projection::verify_content_digest(
            &authorization_value,
            &["/header/content_digest"],
            GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN,
            "/header/content_digest",
        )
        .expect("WIA content digest must bind the complete payload");
    }

    #[test]
    fn composer_rejects_candidate_with_a_replaced_header_identity() {
        let mut inputs = valid_inputs();
        let mut candidate_payload: Value =
            serde_json::from_str(&inputs.candidate.canonical_json).unwrap();
        candidate_payload["header"]["id"] = json!(object_id(999).to_string());
        inputs.candidate.canonical_json = sealed_canonical(candidate_payload).0;

        assert!(matches!(
            compose_candidate_admission(&inputs),
            Err(CandidateAdmissionCompositionError::Inconsistent(detail))
                if detail.contains("candidate header identity")
        ));
    }

    #[test]
    fn composer_rejects_authorization_for_a_different_purpose() {
        let mut inputs = valid_inputs();
        inputs.authorization_purpose = "unrelated_purpose".to_owned();

        assert!(matches!(
            compose_candidate_admission(&inputs),
            Err(CandidateAdmissionCompositionError::Inconsistent(detail))
                if detail.contains("authorization snapshot")
        ));
    }

    #[test]
    fn composer_rejects_candidate_with_a_tampered_content_digest() {
        let mut inputs = valid_inputs();
        let mut candidate_payload: Value =
            serde_json::from_str(&inputs.candidate.canonical_json).unwrap();
        candidate_payload["header"]["content_digest"] = json!(format!("sha256:{}", "0".repeat(64)));
        inputs.candidate.canonical_json = canonical_string(&candidate_payload).unwrap();

        assert!(matches!(
            compose_candidate_admission(&inputs),
            Err(CandidateAdmissionCompositionError::Malformed(detail))
                if detail.contains("candidate content digest is invalid")
        ));
    }
}
