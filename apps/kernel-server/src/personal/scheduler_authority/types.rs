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

pub(crate) const TASK_CONTRACT_EXECUTION_SCHEMA_V03: &str = "cognitiveos.task-contract/0.3";
pub(crate) const TASK_CONTRACT_EXECUTION_SCHEMA_V04: &str = "cognitiveos.task-contract/0.4";
pub(crate) const OPERATION_CANDIDATE_SCHEMA_VERSION: &str =
    "cognitiveos.operation-candidate-proposal/0.1";
pub(crate) const DAEMON_DESCRIPTOR_REFERENCE_DIGEST_DOMAIN: &str =
    "cognitiveos.personal.daemon-descriptor-reference/0.1";
pub(crate) const DEFAULT_LOOP_STAGNATION_CEILING: usize = 3;

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
    pub loop_control_decision: LoopControlDecision,
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

/// Bounded, body-free observation from one governed Context cache consultation.
/// It records only cache outcome and digest counts; it is not Task progress,
/// evidence, verification, or a continuation decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextCacheTelemetry {
    pub cache_hit: bool,
    pub stable_prefix_segment_count: usize,
    pub delta_segment_count: usize,
}

/// A freshly authorized Context resolution accompanied by a digest-only cache
/// observation. The resolved view is always built through the same authority
/// checks as an uncached request.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GovernedContextResolution {
    pub resolved_view: ResolvedContextView,
    pub cache_telemetry: ContextCacheTelemetry,
}

/// A daemon-only loop-control observation. These outcomes control whether a
/// later scheduler attempt may be considered; none of them accepts a Task or
/// advances verification, Effect, or Gate state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LoopControlDecision {
    Continue,
    Wait { reason_code: &'static str },
    Switch { prior_signature_digest: String },
    Block { reason_code: &'static str },
}

/// Fold append-only progress facts into a bounded repeat/no-progress decision.
/// The daemon-issued action fingerprint already binds action, target, tool,
/// descriptor, and parameters; the durable status is the bounded error class.
/// Evidence references are canonicalized and hashed without retaining bodies.
pub(crate) fn derive_loop_control_from_facts(
    progress_facts: &[cognitive_kernel::ports::ProgressFactRow],
    action_fingerprint: &str,
    maximum_retries: i64,
    stagnation_ceiling: usize,
) -> Result<LoopControlDecision, SchedulerAuthorityError> {
    if action_fingerprint.is_empty() || maximum_retries < 0 || stagnation_ceiling == 0 {
        return Err(SchedulerAuthorityError::LoopControlUnavailable(
            "loop-control bounds or action identity are invalid".to_owned(),
        ));
    }

    let mut trailing_non_progress_count = 0usize;
    let mut latest_signature = None;
    let mut repeated_signature_count = 0usize;
    for progress_fact in progress_facts.iter().rev() {
        if !matches!(
            progress_fact.status.as_str(),
            "advanced" | "none" | "uncertain" | "blocked"
        ) {
            return Err(SchedulerAuthorityError::LoopControlUnavailable(
                "progress fact status is outside the registered progress set".to_owned(),
            ));
        }
        let evidence_digest = digest_evidence_references(&progress_fact.evidence_refs_json)?;
        let signature_digest = canonical::digest(
            &canonical::canonical_bytes_of_value(&json!({
                "action_fingerprint": progress_fact.action_fingerprint,
                "error_class": progress_fact.status,
                "evidence_digest": evidence_digest,
            }))
            .map_err(|error| SchedulerAuthorityError::LoopControlUnavailable(error.to_string()))?,
            "cognitiveos.personal.loop-signature/0.1",
        )
        .map_err(|error| SchedulerAuthorityError::LoopControlUnavailable(error.to_string()))?;

        if progress_fact.status != "advanced" {
            trailing_non_progress_count += 1;
        } else {
            break;
        }
        if latest_signature.is_none() {
            latest_signature = Some(signature_digest.clone());
        }
        if latest_signature.as_deref() == Some(signature_digest.as_str()) {
            repeated_signature_count += 1;
        } else {
            break;
        }
    }

    if trailing_non_progress_count >= stagnation_ceiling {
        return Ok(LoopControlDecision::Block {
            reason_code: "no_progress_ceiling_reached",
        });
    }
    if repeated_signature_count > maximum_retries as usize {
        return Ok(LoopControlDecision::Block {
            reason_code: "repeat_retry_ceiling_reached",
        });
    }
    if repeated_signature_count > 1 {
        return Ok(LoopControlDecision::Switch {
            prior_signature_digest: latest_signature.unwrap_or_default(),
        });
    }
    if progress_facts
        .last()
        .is_some_and(|fact| fact.status == "blocked" || fact.status == "uncertain")
    {
        return Ok(LoopControlDecision::Wait {
            reason_code: "durable_progress_uncertain_or_blocked",
        });
    }
    Ok(LoopControlDecision::Continue)
}

pub(crate) fn digest_evidence_references(
    evidence_refs_json: &str,
) -> Result<String, SchedulerAuthorityError> {
    let parsed_value: Value = serde_json::from_str(evidence_refs_json).map_err(|error| {
        SchedulerAuthorityError::LoopControlUnavailable(format!(
            "evidence references are not valid JSON: {error}"
        ))
    })?;
    let Value::Array(evidence_values) = parsed_value else {
        return Err(SchedulerAuthorityError::LoopControlUnavailable(
            "evidence references must be a JSON array".to_owned(),
        ));
    };
    let mut evidence_references = evidence_values
        .into_iter()
        .map(|value| match value {
            Value::String(reference) if !reference.is_empty() => Ok(reference),
            _ => Err(SchedulerAuthorityError::LoopControlUnavailable(
                "evidence references must contain non-empty strings".to_owned(),
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    evidence_references.sort();
    let canonical_evidence = canonical::canonical_bytes_of_value(&json!(evidence_references))
        .map_err(|error| SchedulerAuthorityError::LoopControlUnavailable(error.to_string()))?;
    canonical::digest(
        &canonical_evidence,
        "cognitiveos.personal.loop-evidence/0.1",
    )
    .map_err(|error| SchedulerAuthorityError::LoopControlUnavailable(error.to_string()))
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
    /// Absent only for the first pre-admission pass, before any candidate
    /// Intent/Effect binding exists. Every worker-authority path requires it.
    pub authority_binding: Option<SchedulerAuthorityBinding>,
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

pub(crate) struct FixedSchedulerClock(WallTimestamp);

impl FixedSchedulerClock {
    pub(crate) fn parse(value: &str) -> Result<Self, SchedulerAuthorityError> {
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

/// Reject scheduler work that was bound to a superseded TaskContract epoch.
/// This fence runs before any lease mutation or harness invocation.
pub(crate) fn ensure_current_contract_epoch(
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

pub(crate) fn next_object_id<G: IdGenerator>(ids: &G) -> Result<ObjectId, SchedulerAuthorityError> {
    let raw_id = ids
        .next_uuid_v7()
        .map_err(|error| SchedulerAuthorityError::Store(error.detail))?;
    ObjectId::parse(&raw_id)
        .map_err(|error| SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string()))
}

pub(crate) fn next_event_id<G: IdGenerator>(ids: &G) -> Result<EventId, SchedulerAuthorityError> {
    let raw_id = ids
        .next_uuid_v7()
        .map_err(|error| SchedulerAuthorityError::Store(error.detail))?;
    EventId::parse(&raw_id)
        .map_err(|error| SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string()))
}

pub(crate) fn next_record_id<G: IdGenerator>(ids: &G) -> Result<RecordId, SchedulerAuthorityError> {
    let raw_id = ids
        .next_uuid_v7()
        .map_err(|error| SchedulerAuthorityError::Store(error.detail))?;
    RecordId::parse(&raw_id)
        .map_err(|error| SchedulerAuthorityError::CandidateAdmissionComposition(error.to_string()))
}
