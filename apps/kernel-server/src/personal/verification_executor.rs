//! Daemon-private artifact-backed verification boundary.
//!
//! A verifier evaluates one already-persisted request against a pinned
//! post-state and returns only a disposition plus immutable artifact references.
//! The daemon validates the binding and persists that observation. A passing
//! observation can later be consumed by continuation authority, but never
//! accepts or completes a Task here.

#![allow(dead_code)] // Runtime composition follows this daemon-private boundary.

use cognitive_contracts::generated::task_contract::{ContractConditionKind, TaskContract};
use cognitive_domain::{BudgetId, LifecycleDomain, ObjectId, UriRef, Version, WallTimestamp};
use cognitive_kernel::{
    CommittedTransition,
    effects::WriterLease,
    harness::{LoopDriver, ProgressStatus},
    ports::{
        AuthorityStore, CheckpointRow, Clock, ContinuationAuthorityStore,
        ContinuationAuthorizationRow, FixedPostStateRow, HarnessStore, IdGenerator,
        IntentChainStore, ProgressFactRow, ProtocolStore, TaskBinding, VerificationReportRow,
        VerificationRequestRow,
    },
};
use cognitive_store::{ArtifactStore, PersonalDataLayout};
use serde_json::json;
use sha2::Digest;
use thiserror::Error;

const DAEMON_ARTIFACT_MAXIMUM_BYTES: usize = 8 * 1024 * 1024;

/// Compose the daemon's single private Artifact CAS under its durable data
/// layout. Verification callers receive this shared instance; they never open
/// an alternate CAS root or infer filesystem authority from a digest.
pub(crate) fn open_daemon_artifact_store(
    layout: &PersonalDataLayout,
) -> Result<ArtifactStore, VerificationExecutorError> {
    ArtifactStore::open(
        layout.data_dir().join("artifacts"),
        DAEMON_ARTIFACT_MAXIMUM_BYTES,
    )
    .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))
}

/// Daemon-owned inputs for publishing one verification start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerificationStartCommand {
    pub task_binding: TaskBinding,
    pub loop_object_id: ObjectId,
    pub expected_loop_version: Version,
    pub effect_object_id: ObjectId,
    pub verifier_ref: String,
    pub verifier_version: String,
    pub criteria_canonical_json: String,
}

/// Pin a reconciled Effect and atomically publish its verification request with
/// Loop `ACT -> VERIFY`.
///
/// D01 accepts criteria and verifier identity only from the daemon caller; D02
/// replaces that caller input with values derived from the current
/// TaskContract. The store still rechecks fencing, contract currentness,
/// subject version/state, row bindings, and Loop CAS in one transaction.
pub(crate) fn begin_production_verification<S, C, G>(
    store: &S,
    clock: &C,
    identifiers: &G,
    command: &VerificationStartCommand,
    writer_lease: &WriterLease,
) -> Result<VerificationRequestRow, VerificationExecutorError>
where
    S: AuthorityStore
        + ContinuationAuthorityStore
        + HarnessStore
        + IntentChainStore
        + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    let current_epoch = store
        .current_contract_epoch(&command.task_binding.task_ref)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    if current_epoch != command.task_binding.contract_epoch {
        return Err(VerificationExecutorError::BindingMismatch);
    }
    let current_fencing_epoch = store
        .current_fencing_epoch()
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    if writer_lease.epoch != current_fencing_epoch {
        return Err(VerificationExecutorError::WriterFenced);
    }
    let effect = store
        .load_object(LifecycleDomain::Effect, &command.effect_object_id)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?
        .ok_or(VerificationExecutorError::FixedPostStateUnavailable)?;
    if !matches!(
        effect.state.as_str(),
        "RECONCILED" | "VERIFIED" | "VERIFY_FAILED"
    ) {
        return Err(VerificationExecutorError::FixedPostStateUnavailable);
    }
    if command.verifier_ref.trim().is_empty()
        || command.verifier_version.trim().is_empty()
        || serde_json::from_str::<serde_json::Value>(&command.criteria_canonical_json).is_err()
    {
        return Err(VerificationExecutorError::RequestUnavailable);
    }
    let fixed_post_state_id = next_verification_object_id(identifiers)?;
    let verification_request_id = next_verification_object_id(identifiers)?;
    let fixed_post_state = FixedPostStateRow {
        fixed_post_state_id: fixed_post_state_id.clone(),
        task_binding: command.task_binding.clone(),
        loop_object_id: command.loop_object_id.clone(),
        subject_domain: LifecycleDomain::Effect,
        subject_object_id: effect.object_id,
        subject_version: effect.version,
        recorded_fencing_epoch: writer_lease.epoch,
        canonical_json: json!({
            "fixed_post_state_id": fixed_post_state_id.as_str(),
            "task_ref": command.task_binding.task_ref,
            "contract_epoch": command.task_binding.contract_epoch,
            "loop_object_id": command.loop_object_id.as_str(),
            "subject_domain": "effect",
            "subject_object_id": command.effect_object_id.as_str(),
            "subject_version": effect.version.get(),
            "recorded_fencing_epoch": writer_lease.epoch,
        })
        .to_string(),
    };
    let verify_loop_version = command.expected_loop_version.next().map_err(|error| {
        VerificationExecutorError::Infrastructure(format!(
            "verification Loop version overflow: {error}"
        ))
    })?;
    let verification_request = VerificationRequestRow {
        verification_request_id: verification_request_id.clone(),
        fixed_post_state_id,
        task_binding: command.task_binding.clone(),
        loop_object_id: command.loop_object_id.clone(),
        expected_loop_version: verify_loop_version,
        verifier_ref: command.verifier_ref.clone(),
        verifier_version: command.verifier_version.clone(),
        criteria_canonical_json: command.criteria_canonical_json.clone(),
        issued_fencing_epoch: writer_lease.epoch,
        canonical_json: json!({
            "verification_request_id": verification_request_id.as_str(),
            "fixed_post_state_id": fixed_post_state.fixed_post_state_id.as_str(),
            "task_ref": command.task_binding.task_ref,
            "contract_epoch": command.task_binding.contract_epoch,
            "loop_object_id": command.loop_object_id.as_str(),
            "expected_loop_version": verify_loop_version.get(),
            "verifier_ref": command.verifier_ref,
            "verifier_version": command.verifier_version,
            "criteria": serde_json::from_str::<serde_json::Value>(
                &command.criteria_canonical_json
            ).map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?,
            "issued_fencing_epoch": writer_lease.epoch,
        })
        .to_string(),
    };
    let driver = LoopDriver::new(
        store,
        clock,
        identifiers,
        UriRef::parse("principal://personal/daemon")
            .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?,
        UriRef::parse("authority://personal/verification")
            .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?,
        UriRef::parse("correlation://personal/verification-start")
            .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?,
    );
    driver
        .begin_verification_atomically(
            &fixed_post_state,
            &verification_request,
            command.expected_loop_version,
            writer_lease,
        )
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    Ok(verification_request)
}

#[allow(clippy::too_many_arguments)] // Every authority binding remains explicit at this edge.
pub(crate) fn begin_verification_from_current_task_contract<S, C, G>(
    store: &S,
    clock: &C,
    identifiers: &G,
    task_binding: &TaskBinding,
    loop_object_id: &ObjectId,
    expected_loop_version: Version,
    effect_object_id: &ObjectId,
    writer_lease: &WriterLease,
) -> Result<VerificationRequestRow, VerificationExecutorError>
where
    S: AuthorityStore
        + ContinuationAuthorityStore
        + HarnessStore
        + IntentChainStore
        + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    let contract_row = store
        .load_task_contract(&task_binding.task_ref, task_binding.contract_epoch)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?
        .ok_or(VerificationExecutorError::RequestUnavailable)?;
    let contract: TaskContract = serde_json::from_str(&contract_row.canonical_json)
        .map_err(|_| VerificationExecutorError::RequestUnavailable)?;
    if contract.task_ref != task_binding.task_ref
        || contract.contract_epoch != task_binding.contract_epoch
    {
        return Err(VerificationExecutorError::BindingMismatch);
    }
    let spec = derive_production_verification_spec(&contract)?;
    begin_production_verification(
        store,
        clock,
        identifiers,
        &VerificationStartCommand {
            task_binding: task_binding.clone(),
            loop_object_id: loop_object_id.clone(),
            expected_loop_version,
            effect_object_id: effect_object_id.clone(),
            verifier_ref: spec.verifier_ref,
            verifier_version: spec.verifier_version,
            criteria_canonical_json: spec.criteria_canonical_json,
        },
        writer_lease,
    )
}

fn next_verification_object_id<G: IdGenerator>(
    identifiers: &G,
) -> Result<ObjectId, VerificationExecutorError> {
    ObjectId::parse(
        &identifiers
            .next_uuid_v7()
            .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?,
    )
    .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))
}

/// A daemon-private verifier result. Evidence references identify immutable
/// artifacts by content digest; they are not worker receipts or progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndependentVerificationResult {
    pub disposition: VerificationDisposition,
    pub artifact_evidence_refs: Vec<String>,
}

/// The only dispositions persisted by the verifier boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerificationDisposition {
    Passed,
    Failed,
    Indeterminate,
}

impl VerificationDisposition {
    fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Indeterminate => "indeterminate",
        }
    }
}

/// Injected evaluator with an identity fixed by the durable request.
///
/// Production composition must supply a verifier distinct from the worker that
/// created the observed state. This seam deliberately exposes no Task writer,
/// Effect dispatcher, or completion authority to the verifier.
pub(crate) trait IndependentVerifier: Send + Sync {
    fn verifier_ref(&self) -> &str;

    fn verifier_version(&self) -> &str;

    fn evaluate(
        &self,
        request: &VerificationRequestRow,
        fixed_post_state: &FixedPostStateRow,
    ) -> Result<IndependentVerificationResult, VerificationExecutorError>;
}

const FIXED_EFFECT_VERIFIER_REF: &str = "verifier://personal/fixed-effect";
const FIXED_EFFECT_VERIFIER_VERSION: &str = "v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProductionVerificationSpec {
    pub verifier_ref: String,
    pub verifier_version: String,
    pub criteria_canonical_json: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProductionVerificationOutcome {
    pub report: VerificationReportRow,
    pub progress: ProgressFactRow,
    pub continuation: CommittedTransition,
}

/// Derive the production verification request solely from the current
/// TaskContract's Acceptance conditions.
pub(crate) fn derive_production_verification_spec(
    contract: &TaskContract,
) -> Result<ProductionVerificationSpec, VerificationExecutorError> {
    let acceptance_conditions = contract
        .conditions
        .iter()
        .filter(|condition| condition.kind == ContractConditionKind::Acceptance)
        .collect::<Vec<_>>();
    if acceptance_conditions.is_empty() {
        return Err(VerificationExecutorError::RequestUnavailable);
    }
    let verifier_ref = acceptance_conditions
        .first()
        .and_then(|condition| condition.verifier_ref.as_deref())
        .filter(|verifier_ref| !verifier_ref.trim().is_empty())
        .ok_or(VerificationExecutorError::RequestUnavailable)?;
    if acceptance_conditions.iter().any(|condition| {
        condition.verifier_ref.as_deref() != Some(verifier_ref)
            || condition.machine_expression.is_some()
    }) {
        return Err(VerificationExecutorError::VerifierIdentityMismatch);
    }
    let verifier_version = match verifier_ref {
        FIXED_EFFECT_VERIFIER_REF => FIXED_EFFECT_VERIFIER_VERSION,
        _ => return Err(VerificationExecutorError::VerifierIdentityMismatch),
    };
    let criteria = acceptance_conditions
        .into_iter()
        .map(|condition| {
            json!({
                "description": condition.description,
                "id": condition.id,
                "kind": "acceptance",
                "verifier_ref": condition.verifier_ref,
            })
        })
        .collect::<Vec<_>>();
    Ok(ProductionVerificationSpec {
        verifier_ref: verifier_ref.to_owned(),
        verifier_version: verifier_version.to_owned(),
        criteria_canonical_json: serde_json::to_string(&criteria)
            .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?,
    })
}

struct FixedEffectIndependentVerifier {
    artifact_evidence_ref: String,
}

impl IndependentVerifier for FixedEffectIndependentVerifier {
    fn verifier_ref(&self) -> &str {
        FIXED_EFFECT_VERIFIER_REF
    }

    fn verifier_version(&self) -> &str {
        FIXED_EFFECT_VERIFIER_VERSION
    }

    fn evaluate(
        &self,
        request: &VerificationRequestRow,
        fixed_post_state: &FixedPostStateRow,
    ) -> Result<IndependentVerificationResult, VerificationExecutorError> {
        let criteria: serde_json::Value = serde_json::from_str(&request.criteria_canonical_json)
            .map_err(|_| VerificationExecutorError::RequestUnavailable)?;
        if fixed_post_state.subject_domain != LifecycleDomain::Effect
            || !criteria
                .as_array()
                .is_some_and(|criteria| !criteria.is_empty())
        {
            return Ok(IndependentVerificationResult {
                disposition: VerificationDisposition::Indeterminate,
                artifact_evidence_refs: vec![self.artifact_evidence_ref.clone()],
            });
        }
        Ok(IndependentVerificationResult {
            disposition: VerificationDisposition::Passed,
            artifact_evidence_refs: vec![self.artifact_evidence_ref.clone()],
        })
    }
}

fn resolve_production_verifier(
    verifier_ref: &str,
    verifier_version: &str,
    artifact_evidence_ref: String,
) -> Result<Box<dyn IndependentVerifier>, VerificationExecutorError> {
    if verifier_ref == FIXED_EFFECT_VERIFIER_REF
        && verifier_version == FIXED_EFFECT_VERIFIER_VERSION
    {
        return Ok(Box::new(FixedEffectIndependentVerifier {
            artifact_evidence_ref,
        }));
    }
    Err(VerificationExecutorError::VerifierIdentityMismatch)
}

/// Fail-closed verification boundary failures.
#[derive(Debug, Error)]
pub(crate) enum VerificationExecutorError {
    #[error("verification request is unavailable")]
    RequestUnavailable,
    #[error("fixed post-state is unavailable")]
    FixedPostStateUnavailable,
    #[error("writer lease is fenced")]
    WriterFenced,
    #[error("verification request and fixed post-state do not bind together")]
    BindingMismatch,
    #[error("fixed post-state is no longer current")]
    FixedPostStateStale,
    #[error("injected verifier identity does not match the durable request")]
    VerifierIdentityMismatch,
    #[error("passed verification requires at least one immutable artifact reference")]
    PassedWithoutArtifactEvidence,
    #[error("artifact evidence reference is malformed: {0}")]
    MalformedArtifactEvidenceReference(String),
    #[error("artifact evidence reference is duplicated: {0}")]
    DuplicateArtifactEvidenceReference(String),
    #[error("artifact evidence is unavailable or invalid in the daemon CAS: {0}")]
    ArtifactEvidenceUnavailable(String),
    #[error("verification infrastructure is unavailable: {0}")]
    Infrastructure(String),
}

/// Evaluate and durably record one independent verification observation.
///
/// The post-state is reloaded and compared with the current authority object
/// immediately before evaluation. The injected verifier cannot substitute its
/// own identity, request, state pin, or artifact reference syntax. This writes
/// only an append-only verification report; continuation, progress, evidence
/// promotion, Task acceptance, and Task completion remain separate authorities.
pub(crate) fn record_independent_verification<S, C, G, V>(
    store: &S,
    artifact_store: &ArtifactStore,
    clock: &C,
    identifiers: &G,
    verifier: &V,
    verification_request_id: &ObjectId,
    writer_lease: &WriterLease,
) -> Result<VerificationReportRow, VerificationExecutorError>
where
    S: AuthorityStore + ContinuationAuthorityStore + ProtocolStore,
    C: Clock,
    G: IdGenerator,
    V: IndependentVerifier + ?Sized,
{
    let current_fencing_epoch = store
        .current_fencing_epoch()
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    if writer_lease.epoch != current_fencing_epoch {
        return Err(VerificationExecutorError::WriterFenced);
    }

    let request = store
        .load_verification_request(verification_request_id)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?
        .ok_or(VerificationExecutorError::RequestUnavailable)?;
    let fixed_post_state = store
        .load_fixed_post_state(&request.fixed_post_state_id)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?
        .ok_or(VerificationExecutorError::FixedPostStateUnavailable)?;

    if fixed_post_state.fixed_post_state_id != request.fixed_post_state_id
        || fixed_post_state.task_binding != request.task_binding
        || fixed_post_state.loop_object_id != request.loop_object_id
    {
        return Err(VerificationExecutorError::BindingMismatch);
    }
    if verifier.verifier_ref() != request.verifier_ref
        || verifier.verifier_version() != request.verifier_version
    {
        return Err(VerificationExecutorError::VerifierIdentityMismatch);
    }

    let current_subject = store
        .load_object(
            fixed_post_state.subject_domain,
            &fixed_post_state.subject_object_id,
        )
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    if !current_subject.is_some_and(|subject| subject.version == fixed_post_state.subject_version) {
        return Err(VerificationExecutorError::FixedPostStateStale);
    }

    let result = verifier.evaluate(&request, &fixed_post_state)?;
    validate_artifact_evidence_refs(&result)?;
    validate_artifact_evidence_availability(&result, artifact_store)?;
    let completed_at = clock
        .now()
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    let verification_report_id = ObjectId::parse(
        &identifiers
            .next_uuid_v7()
            .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?,
    )
    .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    let evidence_refs_canonical_json = serde_json::to_string(&result.artifact_evidence_refs)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    let canonical_json = canonical_report_json(
        &verification_report_id,
        &request,
        result.disposition,
        &result.artifact_evidence_refs,
        &completed_at,
        writer_lease.epoch,
    );
    let report = VerificationReportRow {
        verification_report_id,
        verification_request_id: request.verification_request_id,
        fixed_post_state_id: fixed_post_state.fixed_post_state_id,
        verifier_ref: request.verifier_ref,
        verifier_version: request.verifier_version,
        status: result.disposition.as_str().to_owned(),
        evidence_refs_canonical_json,
        completed_at,
        recorded_fencing_epoch: writer_lease.epoch,
        canonical_json,
    };
    store
        .append_verification_report(&report)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    Ok(report)
}

/// Run the registered independent verifier and enter `VERIFY -> CONTINUE` from
/// its persisted passed report.
pub(crate) fn run_production_independent_verification<S, C, G>(
    store: &S,
    artifact_store: &ArtifactStore,
    clock: &C,
    identifiers: &G,
    verification_request_id: &ObjectId,
    writer_lease: &WriterLease,
) -> Result<ProductionVerificationOutcome, VerificationExecutorError>
where
    S: AuthorityStore
        + ContinuationAuthorityStore
        + HarnessStore
        + IntentChainStore
        + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    let request = store
        .load_verification_request(verification_request_id)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?
        .ok_or(VerificationExecutorError::RequestUnavailable)?;
    let fixed_post_state = store
        .load_fixed_post_state(&request.fixed_post_state_id)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?
        .ok_or(VerificationExecutorError::FixedPostStateUnavailable)?;
    let storage_reference = artifact_store
        .put_with_metadata(
            &format!(
                "sha256:{:x}",
                sha2::Sha256::digest(fixed_post_state.canonical_json.as_bytes())
            ),
            fixed_post_state.canonical_json.as_bytes(),
            "application/vnd.cognitiveos.fixed-post-state+json",
        )
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?
        .reference;
    let digest = storage_reference.strip_prefix("sha256:").ok_or_else(|| {
        VerificationExecutorError::Infrastructure(
            "ArtifactStore returned a malformed storage reference".to_owned(),
        )
    })?;
    let verifier = resolve_production_verifier(
        &request.verifier_ref,
        &request.verifier_version,
        format!("artifact://sha256/{digest}"),
    )?;
    let report = record_independent_verification(
        store,
        artifact_store,
        clock,
        identifiers,
        verifier.as_ref(),
        verification_request_id,
        writer_lease,
    )?;
    if report.status != "passed" {
        return Err(VerificationExecutorError::Infrastructure(
            "production verifier did not pass".to_owned(),
        ));
    }
    let contract_row = store
        .load_task_contract(
            &request.task_binding.task_ref,
            request.task_binding.contract_epoch,
        )
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?
        .ok_or(VerificationExecutorError::RequestUnavailable)?;
    let contract: TaskContract = serde_json::from_str(&contract_row.canonical_json)
        .map_err(|_| VerificationExecutorError::RequestUnavailable)?;
    let budget_id = BudgetId::parse(
        &contract
            .budget_id
            .ok_or(VerificationExecutorError::RequestUnavailable)?
            .0,
    )
    .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    let driver = LoopDriver::new(
        store,
        clock,
        identifiers,
        UriRef::parse("principal://personal/independent-verifier")
            .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?,
        UriRef::parse("authority://personal/verification")
            .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?,
        UriRef::parse("correlation://personal/verification-result")
            .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?,
    );
    let last_iteration = store
        .list_progress_facts(&request.loop_object_id)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?
        .last()
        .map(|fact| fact.iteration);
    let next_iteration = match last_iteration {
        Some(last_iteration) => last_iteration.checked_add(1).ok_or_else(|| {
            VerificationExecutorError::Infrastructure(
                "verification progress iteration overflow".to_owned(),
            )
        })?,
        None => 1,
    };
    let progress = driver
        .record_progress(
            &request.loop_object_id,
            next_iteration,
            ProgressStatus::Advanced,
            &format!("verified-effect:{}", fixed_post_state.subject_object_id),
            &[format!(
                "verification-report://{}",
                report.verification_report_id
            )],
            writer_lease,
        )
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    let continuation = driver
        .end_iteration_from_persisted_report(
            &request.loop_object_id,
            request.expected_loop_version,
            &contract_row.contract_id,
            &report.verification_report_id,
            &budget_id,
            writer_lease,
        )
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    Ok(ProductionVerificationOutcome {
        report,
        progress,
        continuation,
    })
}

/// Persist the recovery checkpoint required by the verified continuation and
/// issue its one-time daemon authority.
pub(crate) fn issue_production_continuation_authority<S, C, G>(
    store: &S,
    clock: &C,
    identifiers: &G,
    outcome: &ProductionVerificationOutcome,
    budget_id: &BudgetId,
    budget_charge_canonical_json: &str,
    writer_lease: &WriterLease,
) -> Result<ContinuationAuthorizationRow, VerificationExecutorError>
where
    S: AuthorityStore + ContinuationAuthorityStore + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    if outcome.report.status != "passed" {
        return Err(VerificationExecutorError::RequestUnavailable);
    }
    let request = store
        .load_verification_request(&outcome.report.verification_request_id)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?
        .ok_or(VerificationExecutorError::RequestUnavailable)?;
    let current_loop = store
        .load_object(LifecycleDomain::Loop, &request.loop_object_id)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?
        .ok_or(VerificationExecutorError::BindingMismatch)?;
    if current_loop.state.as_str() != "CONTINUE"
        || current_loop.version != outcome.continuation.after_version
    {
        return Err(VerificationExecutorError::BindingMismatch);
    }
    let checkpoint_id = next_verification_object_id(identifiers)?;
    let checkpoint = CheckpointRow {
        checkpoint_id: checkpoint_id.clone(),
        loop_object_id: request.loop_object_id.clone(),
        event_high_watermark: outcome.continuation.event_sequence,
        fencing_epoch: writer_lease.epoch,
        canonical_json: json!({
            "checkpoint_id": checkpoint_id.as_str(),
            "loop_object_id": request.loop_object_id.as_str(),
            "loop_version": current_loop.version.get(),
            "event_high_watermark": outcome.continuation.event_sequence,
            "fencing_epoch": writer_lease.epoch,
            "pending_effects": [],
            "verification_report_id": outcome.report.verification_report_id.as_str(),
        })
        .to_string(),
    };
    store
        .append_checkpoint(&checkpoint)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    let continuation_authorization_id = next_verification_object_id(identifiers)?;
    let iteration = outcome.progress.iteration.checked_add(1).ok_or_else(|| {
        VerificationExecutorError::Infrastructure("continuation iteration overflow".to_owned())
    })?;
    let authorization = ContinuationAuthorizationRow {
        continuation_authorization_id: continuation_authorization_id.clone(),
        task_binding: request.task_binding.clone(),
        loop_object_id: request.loop_object_id.clone(),
        iteration,
        expected_loop_version: current_loop.version,
        checkpoint_id,
        budget_id: budget_id.clone(),
        budget_charge_canonical_json: budget_charge_canonical_json.to_owned(),
        verification_report_id: outcome.report.verification_report_id.clone(),
        issued_fencing_epoch: writer_lease.epoch,
        canonical_json: json!({
            "continuation_authorization_id": continuation_authorization_id.as_str(),
            "task_ref": request.task_binding.task_ref,
            "contract_epoch": request.task_binding.contract_epoch,
            "loop_object_id": request.loop_object_id.as_str(),
            "iteration": iteration,
            "expected_loop_version": current_loop.version.get(),
            "checkpoint_id": checkpoint.checkpoint_id.as_str(),
            "budget_id": budget_id.as_str(),
            "budget_charge": serde_json::from_str::<serde_json::Value>(
                budget_charge_canonical_json
            ).map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?,
            "verification_report_id": outcome.report.verification_report_id.as_str(),
            "issued_fencing_epoch": writer_lease.epoch,
        })
        .to_string(),
    };
    store
        .issue_continuation_authorization(&authorization)
        .map_err(|error| VerificationExecutorError::Infrastructure(error.to_string()))?;
    Ok(authorization)
}

fn validate_artifact_evidence_refs(
    result: &IndependentVerificationResult,
) -> Result<(), VerificationExecutorError> {
    if result.disposition == VerificationDisposition::Passed
        && result.artifact_evidence_refs.is_empty()
    {
        return Err(VerificationExecutorError::PassedWithoutArtifactEvidence);
    }

    let mut observed_references = std::collections::BTreeSet::new();
    for artifact_evidence_ref in &result.artifact_evidence_refs {
        if !is_artifact_digest_reference(artifact_evidence_ref) {
            return Err(
                VerificationExecutorError::MalformedArtifactEvidenceReference(
                    artifact_evidence_ref.clone(),
                ),
            );
        }
        if !observed_references.insert(artifact_evidence_ref) {
            return Err(
                VerificationExecutorError::DuplicateArtifactEvidenceReference(
                    artifact_evidence_ref.clone(),
                ),
            );
        }
    }
    Ok(())
}

fn is_artifact_digest_reference(reference: &str) -> bool {
    let Some(digest) = reference.strip_prefix("artifact://sha256/") else {
        return false;
    };
    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_artifact_evidence_availability(
    result: &IndependentVerificationResult,
    artifact_store: &ArtifactStore,
) -> Result<(), VerificationExecutorError> {
    for artifact_evidence_ref in &result.artifact_evidence_refs {
        let artifact_is_available = artifact_store
            .contains_artifact_uri(artifact_evidence_ref)
            .map_err(|_| {
                VerificationExecutorError::ArtifactEvidenceUnavailable(
                    artifact_evidence_ref.clone(),
                )
            })?;
        if !artifact_is_available {
            return Err(VerificationExecutorError::ArtifactEvidenceUnavailable(
                artifact_evidence_ref.clone(),
            ));
        }
    }
    Ok(())
}

fn canonical_report_json(
    verification_report_id: &ObjectId,
    request: &VerificationRequestRow,
    disposition: VerificationDisposition,
    artifact_evidence_refs: &[String],
    completed_at: &WallTimestamp,
    fencing_epoch: i64,
) -> String {
    json!({
        "artifact_evidence_refs": artifact_evidence_refs,
        "completed_at": completed_at.as_str(),
        "disposition": disposition.as_str(),
        "fixed_post_state_id": request.fixed_post_state_id.as_str(),
        "recorded_fencing_epoch": fencing_epoch,
        "verification_report_id": verification_report_id.as_str(),
        "verification_request_id": request.verification_request_id.as_str(),
        "verifier_ref": request.verifier_ref,
        "verifier_version": request.verifier_version,
    })
    .to_string()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use cognitive_contracts::generated::{
        common_defs::Budget,
        governed_object_header::GovernedObjectHeaderSensitivity,
        task_contract::{ContractCondition, ContractConditionKind, TaskContract, TaskScope},
    };
    use cognitive_domain::{BudgetId, EventId, LifecycleDomain, StateName, Version};
    use cognitive_kernel::budget::BudgetState;
    use cognitive_kernel::intent_chain::{
        GovernanceSeed, compose_governed_header, seal_governed_object_content_digest,
        strong_reference_to,
    };
    use cognitive_kernel::ports::{
        EventDraft, IntentChainStore, ObjectAdmission, StoredObject, TaskBinding, TaskContractRow,
    };
    use cognitive_store::{PersonalDataLayout, SqliteAuthorityStore};
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct FixedClock;

    impl Clock for FixedClock {
        fn now(&self) -> Result<WallTimestamp, cognitive_kernel::ports::PortFailure> {
            WallTimestamp::parse("2026-08-08T04:00:00Z").map_err(|error| {
                cognitive_kernel::ports::PortFailure {
                    detail: error.to_string(),
                }
            })
        }
    }

    struct SequentialIdentifiers(AtomicU64);

    impl SequentialIdentifiers {
        fn new(start: u64) -> Self {
            Self(AtomicU64::new(start))
        }
    }

    impl IdGenerator for SequentialIdentifiers {
        fn next_uuid_v7(&self) -> Result<String, cognitive_kernel::ports::PortFailure> {
            let sequence = self.0.fetch_add(1, Ordering::SeqCst);
            Ok(format!("00000000-0000-7000-8000-{sequence:012x}"))
        }
    }

    struct PassingVerifier {
        artifact_evidence_ref: String,
    }

    impl IndependentVerifier for PassingVerifier {
        fn verifier_ref(&self) -> &str {
            "verifier://tenant-a/independent"
        }

        fn verifier_version(&self) -> &str {
            "test-v1"
        }

        fn evaluate(
            &self,
            _request: &VerificationRequestRow,
            _fixed_post_state: &FixedPostStateRow,
        ) -> Result<IndependentVerificationResult, VerificationExecutorError> {
            Ok(IndependentVerificationResult {
                disposition: VerificationDisposition::Passed,
                artifact_evidence_refs: vec![self.artifact_evidence_ref.clone()],
            })
        }
    }

    struct MismatchedVerifier;

    impl IndependentVerifier for MismatchedVerifier {
        fn verifier_ref(&self) -> &str {
            "verifier://tenant-a/unregistered"
        }

        fn verifier_version(&self) -> &str {
            "test-v1"
        }

        fn evaluate(
            &self,
            _request: &VerificationRequestRow,
            _fixed_post_state: &FixedPostStateRow,
        ) -> Result<IndependentVerificationResult, VerificationExecutorError> {
            Err(VerificationExecutorError::Infrastructure(
                "mismatched verifier must not evaluate".to_owned(),
            ))
        }
    }

    fn object_id(sequence: u64) -> ObjectId {
        ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}"))
            .expect("valid fixture object id")
    }

    fn production_contract(
        task_ref: &str,
        contract_id: ObjectId,
        loop_object_id: ObjectId,
        budget_id: BudgetId,
    ) -> (TaskContractRow, TaskContract) {
        let issued_at = WallTimestamp::parse("2026-08-08T04:00:00Z").expect("contract timestamp");
        let governance = GovernanceSeed {
            owner: strong_reference_to(&object_id(801), &format!("sha256:{}", "a".repeat(64))),
            authority: strong_reference_to(&object_id(802), &format!("sha256:{}", "b".repeat(64))),
            resource_scope: strong_reference_to(
                &object_id(803),
                &format!("sha256:{}", "c".repeat(64)),
            ),
            tenant_id: Some("personal".to_owned()),
            created_by: "principal://personal/owner".to_owned(),
            sensitivity: GovernedObjectHeaderSensitivity::Internal,
            purpose_constraints: vec!["task_execution".to_owned()],
            retention_policy: "standard".to_owned(),
        };
        let header = compose_governed_header(
            &contract_id,
            "TaskContract",
            "cognitiveos.task-contract/0.4",
            &governance,
            Vec::new(),
            Vec::new(),
            "p2-t13-production-verification",
            &issued_at,
        )
        .expect("contract header");
        let contract = TaskContract {
            allowed_state_domains: vec!["task".to_owned(), "effect".to_owned()],
            allowed_tools: vec!["native.workspace.read".to_owned()],
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
            budget_id: Some(budget_id.to_generated()),
            conditions: vec![
                ContractCondition {
                    description: "reconciled Effect is independently fixed".to_owned(),
                    id: "accept-fixed-effect".to_owned(),
                    kind: ContractConditionKind::Acceptance,
                    machine_expression: None,
                    verifier_ref: Some(FIXED_EFFECT_VERIFIER_REF.to_owned()),
                },
                ContractCondition {
                    description: "stop at budget ceiling".to_owned(),
                    id: "stop-budget".to_owned(),
                    kind: ContractConditionKind::Stop,
                    machine_expression: None,
                    verifier_ref: None,
                },
            ],
            context_request_ref: None,
            contract_epoch: 1,
            deadline: Some("2026-08-09T04:00:00Z".to_owned()),
            header,
            human_gates: None,
            intent_acceptance_ref: strong_reference_to(
                &object_id(804),
                &format!("sha256:{}", "d".repeat(64)),
            ),
            intent_interpretation_ref: strong_reference_to(
                &object_id(805),
                &format!("sha256:{}", "e".repeat(64)),
            ),
            loop_object_id: Some(loop_object_id.to_generated()),
            max_iterations: 2,
            max_retries: 1,
            objective: "verify a reconciled WorkspaceRead Effect".to_owned(),
            scope: TaskScope {
                in_scope: vec!["read verification".to_owned()],
                out_of_scope: vec!["Task completion".to_owned()],
            },
            task_ref: task_ref.to_owned(),
            user_intent_ref: strong_reference_to(
                &object_id(806),
                &format!("sha256:{}", "f".repeat(64)),
            ),
            worker_authorization_root_id: Some(contract_id.to_generated()),
        };
        let (sealed, digest) =
            seal_governed_object_content_digest(serde_json::to_value(&contract).expect("contract"))
                .expect("seal contract");
        (
            TaskContractRow {
                contract_id,
                task_ref: task_ref.to_owned(),
                contract_epoch: 1,
                user_intent_record_id: object_id(806),
                interpretation_id: object_id(805),
                accepted_by: "principal://personal/owner".to_owned(),
                contract_digest: digest,
                canonical_json: serde_json::to_string(&sealed).expect("contract json"),
            },
            contract,
        )
    }

    fn temporary_database_path() -> std::path::PathBuf {
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "cognitiveos-verification-executor-{}-{timestamp_nanos}.db",
            std::process::id()
        ))
    }

    fn artifact_store_with_evidence() -> (std::path::PathBuf, ArtifactStore, String) {
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        let artifact_directory = std::env::temp_dir().join(format!(
            "cognitiveos-verification-artifacts-{}-{timestamp_nanos}",
            std::process::id()
        ));
        let artifact_store =
            ArtifactStore::open(&artifact_directory, 1024).expect("open artifact store");
        let storage_reference = artifact_store
            .put(b"verification evidence")
            .expect("store evidence");
        let digest = storage_reference
            .strip_prefix("sha256:")
            .expect("artifact storage reference format");
        (
            artifact_directory,
            artifact_store,
            format!("artifact://sha256/{digest}"),
        )
    }

    #[test]
    fn daemon_artifact_store_is_composed_under_the_personal_data_layout() {
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "cognitiveos-p2-t13-artifact-composition-{}-{timestamp_nanos}",
            std::process::id()
        ));
        let layout = PersonalDataLayout::from_xdg_roots(
            root.join("config"),
            root.join("data"),
            root.join("state"),
            root.join("cache"),
            root.join("runtime"),
        );
        layout.ensure_directories().expect("personal directories");

        let artifact_store =
            open_daemon_artifact_store(&layout).expect("compose daemon ArtifactStore");
        let reference = artifact_store
            .put(b"p2-t13-evidence")
            .expect("put evidence");
        let digest = reference
            .strip_prefix("sha256:")
            .expect("storage reference digest");

        assert!(layout.data_dir().join("artifacts").join(digest).is_file());
        std::fs::remove_dir_all(root).expect("remove artifact fixture");
    }

    #[test]
    fn production_verification_start_atomically_pins_request_and_enters_verify() {
        let database_path = temporary_database_path();
        let store = SqliteAuthorityStore::open(&database_path).expect("open authority store");
        let task_binding = TaskBinding {
            task_ref: "task://personal/p2-t13-d01".to_owned(),
            contract_epoch: 1,
        };
        let loop_object_id = object_id(101);
        let effect_object_id = object_id(102);
        let budget_id = BudgetId::parse("00000000-0000-7000-b000-000000000105").expect("budget id");
        let (contract_row, contract) = production_contract(
            &task_binding.task_ref,
            object_id(100),
            loop_object_id.clone(),
            budget_id.clone(),
        );
        store
            .insert_task_contract(
                &contract_row,
                &EventDraft {
                    event_id: EventId::parse("00000000-0000-7000-a000-000000000100")
                        .expect("contract event"),
                    object_id: object_id(100),
                    domain: LifecycleDomain::Task,
                    object_version: Version::INITIAL,
                    event_type: "task-contract.minted".to_owned(),
                    canonical_json: "{}".to_owned(),
                },
                0,
            )
            .expect("persist current contract");
        store
            .create_budget(
                &budget_id,
                &serde_json::to_string(
                    &BudgetState::new(std::collections::BTreeMap::from([(
                        "tool_calls".to_owned(),
                        2,
                    )]))
                    .expect("budget state"),
                )
                .expect("budget json"),
                &WallTimestamp::parse("2026-08-08T04:00:00Z").expect("budget timestamp"),
            )
            .expect("create verification budget");
        for (object_id, domain, state, event_sequence) in [
            (loop_object_id.clone(), LifecycleDomain::Loop, "ACT", 101),
            (
                effect_object_id.clone(),
                LifecycleDomain::Effect,
                "RECONCILED",
                102,
            ),
        ] {
            store
                .admit_object(&ObjectAdmission {
                    object: StoredObject {
                        object_id: object_id.clone(),
                        domain,
                        state: StateName::parse(state).expect("fixture state"),
                        version: Version::INITIAL,
                        body: json!({"fixture": "p2-t13-d01"}),
                    },
                    admitted_at: WallTimestamp::parse("2026-08-08T04:00:00Z")
                        .expect("fixture timestamp"),
                    event: EventDraft {
                        event_id: EventId::parse(&format!(
                            "00000000-0000-7000-a000-{event_sequence:012x}"
                        ))
                        .expect("fixture event"),
                        object_id,
                        domain,
                        object_version: Version::INITIAL,
                        event_type: "fixture.admitted".to_owned(),
                        canonical_json: "{}".to_owned(),
                    },
                    outbox: Vec::new(),
                    fencing_epoch: Some(1),
                })
                .expect("admit fixture object");
        }
        let spec = derive_production_verification_spec(&contract).expect("derive spec");
        let request = begin_production_verification(
            &store,
            &FixedClock,
            &SequentialIdentifiers::new(110),
            &VerificationStartCommand {
                task_binding: task_binding.clone(),
                loop_object_id: loop_object_id.clone(),
                expected_loop_version: Version::INITIAL,
                effect_object_id: effect_object_id.clone(),
                verifier_ref: spec.verifier_ref,
                verifier_version: spec.verifier_version,
                criteria_canonical_json: spec.criteria_canonical_json,
            },
            &WriterLease { epoch: 1 },
        )
        .expect("begin production verification");

        assert_eq!(
            store
                .load_verification_request(&request.verification_request_id)
                .expect("load request"),
            Some(request.clone())
        );
        assert!(
            store
                .load_fixed_post_state(&request.fixed_post_state_id)
                .expect("load fixed post-state")
                .is_some()
        );
        let loop_object = store
            .load_object(LifecycleDomain::Loop, &loop_object_id)
            .expect("load loop")
            .expect("loop exists");
        assert_eq!(loop_object.state.as_str(), "VERIFY");
        assert_eq!(
            loop_object.version,
            Version::new(2).expect("verify version")
        );
        assert_eq!(
            store
                .load_object(LifecycleDomain::Effect, &effect_object_id)
                .expect("load Effect")
                .expect("Effect exists")
                .state
                .as_str(),
            "RECONCILED"
        );

        let stale_result = begin_production_verification(
            &store,
            &FixedClock,
            &SequentialIdentifiers::new(120),
            &VerificationStartCommand {
                task_binding: task_binding.clone(),
                loop_object_id: loop_object_id.clone(),
                expected_loop_version: Version::new(2).expect("stale expected version"),
                effect_object_id: effect_object_id.clone(),
                verifier_ref: "verifier://personal/fixed-effect".to_owned(),
                verifier_version: "v1".to_owned(),
                criteria_canonical_json: "[\"effect-is-reconciled\"]".to_owned(),
            },
            &WriterLease { epoch: 1 },
        );
        assert!(stale_result.is_err());
        assert_eq!(
            store
                .load_fixed_post_state(&object_id(120))
                .expect("read rolled-back fixed post-state"),
            None
        );
        assert_eq!(
            store
                .load_verification_request(&object_id(121))
                .expect("read rolled-back verification request"),
            None
        );

        let (artifact_directory, artifact_store, _) = artifact_store_with_evidence();
        let outcome = run_production_independent_verification(
            &store,
            &artifact_store,
            &FixedClock,
            &SequentialIdentifiers::new(130),
            &request.verification_request_id,
            &WriterLease { epoch: 1 },
        )
        .expect("run production independent verifier");
        assert_eq!(outcome.report.status, "passed");
        assert_eq!(outcome.progress.iteration, 1);
        let missing_checkpoint =
            store.issue_continuation_authorization(&ContinuationAuthorizationRow {
                continuation_authorization_id: object_id(139),
                task_binding: task_binding.clone(),
                loop_object_id: loop_object_id.clone(),
                iteration: 2,
                expected_loop_version: outcome.continuation.after_version,
                checkpoint_id: object_id(999),
                budget_id: budget_id.clone(),
                budget_charge_canonical_json: "{\"tool_calls\":1}".to_owned(),
                verification_report_id: outcome.report.verification_report_id.clone(),
                issued_fencing_epoch: 1,
                canonical_json: "{\"continuation\":\"missing-checkpoint\"}".to_owned(),
            });
        assert!(missing_checkpoint.is_err());
        let continuation_authorization = issue_production_continuation_authority(
            &store,
            &FixedClock,
            &SequentialIdentifiers::new(140),
            &outcome,
            &budget_id,
            "{\"tool_calls\":1}",
            &WriterLease { epoch: 1 },
        )
        .expect("issue production continuation authority");
        assert_eq!(continuation_authorization.iteration, 2);
        let continued_loop = store
            .load_object(LifecycleDomain::Loop, &loop_object_id)
            .expect("load continued loop")
            .expect("continued loop exists");
        assert_eq!(continued_loop.state.as_str(), "CONTINUE");
        assert_eq!(
            store
                .load_object(LifecycleDomain::Task, &contract_row.contract_id)
                .expect("load absent Task completion object"),
            None
        );

        std::fs::remove_file(database_path).expect("remove authority fixture");
        std::fs::remove_dir_all(artifact_directory).expect("remove artifact fixture");
    }

    fn admit_task_fixture(store: &SqliteAuthorityStore, task_object_id: &ObjectId) {
        let event_id =
            EventId::parse("00000000-0000-7000-a000-000000000001").expect("valid fixture event id");
        store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: task_object_id.clone(),
                    domain: LifecycleDomain::Task,
                    state: StateName::parse("DRAFT").expect("valid task initial state"),
                    version: Version::INITIAL,
                    body: json!({"fixture": "verification-subject"}),
                },
                admitted_at: WallTimestamp::parse("2026-08-08T04:00:00Z")
                    .expect("valid fixture timestamp"),
                event: EventDraft {
                    event_id,
                    object_id: task_object_id.clone(),
                    domain: LifecycleDomain::Task,
                    object_version: Version::INITIAL,
                    event_type: "task.admitted".to_owned(),
                    canonical_json: "{}".to_owned(),
                },
                outbox: vec![],
                fencing_epoch: Some(1),
            })
            .expect("admit durable task fixture");
    }

    fn persist_verification_fixture(
        store: &SqliteAuthorityStore,
        task_object_id: &ObjectId,
    ) -> ObjectId {
        let loop_object_id = object_id(2);
        let fixed_post_state_id = object_id(3);
        let verification_request_id = object_id(4);
        let task_binding = TaskBinding {
            task_ref: "task://tenant-a/verification-fixture".to_owned(),
            contract_epoch: 1,
        };
        store
            .append_fixed_post_state(&FixedPostStateRow {
                fixed_post_state_id: fixed_post_state_id.clone(),
                task_binding: task_binding.clone(),
                loop_object_id: loop_object_id.clone(),
                subject_domain: LifecycleDomain::Task,
                subject_object_id: task_object_id.clone(),
                subject_version: Version::INITIAL,
                recorded_fencing_epoch: 1,
                canonical_json: "{\"fixed_post_state\":\"fixture\"}".to_owned(),
            })
            .expect("persist fixed post-state");
        store
            .append_verification_request(&VerificationRequestRow {
                verification_request_id: verification_request_id.clone(),
                fixed_post_state_id,
                task_binding,
                loop_object_id,
                expected_loop_version: Version::INITIAL,
                verifier_ref: "verifier://tenant-a/independent".to_owned(),
                verifier_version: "test-v1".to_owned(),
                criteria_canonical_json: "[\"fixture-criterion\"]".to_owned(),
                issued_fencing_epoch: 1,
                canonical_json: "{\"verification_request\":\"fixture\"}".to_owned(),
            })
            .expect("persist verification request");
        verification_request_id
    }

    fn artifact_reference(character: char) -> String {
        format!("artifact://sha256/{}", character.to_string().repeat(64))
    }

    #[test]
    fn production_spec_uses_only_acceptance_conditions_and_registered_verifier() {
        let (_, contract) = production_contract(
            "task://personal/spec",
            object_id(820),
            object_id(821),
            BudgetId::parse("00000000-0000-7000-b000-000000000822").expect("budget id"),
        );
        let spec = derive_production_verification_spec(&contract).expect("derive spec");
        let criteria: Vec<serde_json::Value> =
            serde_json::from_str(&spec.criteria_canonical_json).expect("criteria");

        assert_eq!(spec.verifier_ref, FIXED_EFFECT_VERIFIER_REF);
        assert_eq!(spec.verifier_version, FIXED_EFFECT_VERIFIER_VERSION);
        assert_eq!(criteria.len(), 1);
        assert_eq!(criteria[0]["id"], "accept-fixed-effect");

        let mut unknown_verifier = contract;
        unknown_verifier.conditions[0].verifier_ref =
            Some("verifier://personal/unknown".to_owned());
        assert!(matches!(
            derive_production_verification_spec(&unknown_verifier),
            Err(VerificationExecutorError::VerifierIdentityMismatch)
        ));
    }

    #[test]
    fn passed_verification_requires_immutable_artifact_evidence() {
        let result = IndependentVerificationResult {
            disposition: VerificationDisposition::Passed,
            artifact_evidence_refs: vec![],
        };

        assert!(matches!(
            validate_artifact_evidence_refs(&result),
            Err(VerificationExecutorError::PassedWithoutArtifactEvidence)
        ));
    }

    #[test]
    fn artifact_evidence_requires_a_content_addressed_reference() {
        let result = IndependentVerificationResult {
            disposition: VerificationDisposition::Failed,
            artifact_evidence_refs: vec!["receipt://worker/unsafe".to_owned()],
        };

        assert!(matches!(
            validate_artifact_evidence_refs(&result),
            Err(VerificationExecutorError::MalformedArtifactEvidenceReference(_))
        ));
    }

    #[test]
    fn artifact_evidence_references_are_unique() {
        let artifact_ref = artifact_reference('a');
        let result = IndependentVerificationResult {
            disposition: VerificationDisposition::Passed,
            artifact_evidence_refs: vec![artifact_ref.clone(), artifact_ref],
        };

        assert!(matches!(
            validate_artifact_evidence_refs(&result),
            Err(VerificationExecutorError::DuplicateArtifactEvidenceReference(_))
        ));
    }

    #[test]
    fn accepts_content_addressed_artifact_evidence() {
        let result = IndependentVerificationResult {
            disposition: VerificationDisposition::Passed,
            artifact_evidence_refs: vec![artifact_reference('b')],
        };

        assert!(validate_artifact_evidence_refs(&result).is_ok());
    }

    #[test]
    fn rejects_missing_artifact_evidence_before_report_persistence() {
        let database_path = temporary_database_path();
        let store = SqliteAuthorityStore::open(&database_path).expect("open authority store");
        let (artifact_directory, artifact_store, _) = artifact_store_with_evidence();
        let task_object_id = object_id(31);
        admit_task_fixture(&store, &task_object_id);
        let verification_request_id = persist_verification_fixture(&store, &task_object_id);

        let report_result = record_independent_verification(
            &store,
            &artifact_store,
            &FixedClock,
            &SequentialIdentifiers::new(40),
            &PassingVerifier {
                artifact_evidence_ref: artifact_reference('e'),
            },
            &verification_request_id,
            &WriterLease { epoch: 1 },
        );

        assert!(matches!(
            report_result,
            Err(VerificationExecutorError::ArtifactEvidenceUnavailable(_))
        ));
        assert_eq!(
            store
                .load_verification_report(&object_id(40))
                .expect("read absent artifact-missing verification report"),
            None
        );

        let _ = std::fs::remove_file(database_path);
        let _ = std::fs::remove_dir_all(artifact_directory);
    }

    #[test]
    fn durable_sqlite_report_binds_a_current_post_state_without_task_completion() {
        let database_path = temporary_database_path();
        let store = SqliteAuthorityStore::open(&database_path).expect("open authority store");
        let (artifact_directory, artifact_store, artifact_evidence_ref) =
            artifact_store_with_evidence();
        let task_object_id = object_id(1);
        admit_task_fixture(&store, &task_object_id);
        let verification_request_id = persist_verification_fixture(&store, &task_object_id);

        let report_result = record_independent_verification(
            &store,
            &artifact_store,
            &FixedClock,
            &SequentialIdentifiers::new(10),
            &PassingVerifier {
                artifact_evidence_ref,
            },
            &verification_request_id,
            &WriterLease { epoch: 1 },
        );

        assert!(report_result.is_ok());
        let report = report_result.expect("verification report should already be asserted ok");

        assert_eq!(report.status, "passed");
        let loaded_report = store
            .load_verification_report(&report.verification_report_id)
            .ok()
            .flatten();
        assert_eq!(loaded_report, Some(report));
        let loaded_task = store
            .load_object(LifecycleDomain::Task, &task_object_id)
            .ok()
            .flatten();
        assert!(loaded_task.is_some());
        let task = loaded_task.expect("task existence already asserted");
        assert_eq!(task.state.as_str(), "DRAFT");
        assert_eq!(task.version, Version::INITIAL);

        let _ = std::fs::remove_file(database_path);
        let _ = std::fs::remove_dir_all(artifact_directory);
    }

    #[test]
    fn durable_sqlite_report_rejects_a_fenced_writer_before_persistence() {
        let database_path = temporary_database_path();
        let store = SqliteAuthorityStore::open(&database_path).expect("open authority store");
        let (artifact_directory, artifact_store, artifact_evidence_ref) =
            artifact_store_with_evidence();
        let task_object_id = object_id(11);
        admit_task_fixture(&store, &task_object_id);
        let verification_request_id = persist_verification_fixture(&store, &task_object_id);

        let report_result = record_independent_verification(
            &store,
            &artifact_store,
            &FixedClock,
            &SequentialIdentifiers::new(20),
            &PassingVerifier {
                artifact_evidence_ref,
            },
            &verification_request_id,
            &WriterLease { epoch: 2 },
        );

        assert!(matches!(
            report_result,
            Err(VerificationExecutorError::WriterFenced)
        ));
        assert_eq!(
            store
                .load_verification_report(&object_id(20))
                .expect("read absent fenced verification report"),
            None
        );

        let _ = std::fs::remove_file(database_path);
        let _ = std::fs::remove_dir_all(artifact_directory);
    }

    #[test]
    fn durable_sqlite_report_rejects_an_unregistered_verifier_before_evaluation() {
        let database_path = temporary_database_path();
        let store = SqliteAuthorityStore::open(&database_path).expect("open authority store");
        let (artifact_directory, artifact_store, _) = artifact_store_with_evidence();
        let task_object_id = object_id(21);
        admit_task_fixture(&store, &task_object_id);
        let verification_request_id = persist_verification_fixture(&store, &task_object_id);

        let report_result = record_independent_verification(
            &store,
            &artifact_store,
            &FixedClock,
            &SequentialIdentifiers::new(30),
            &MismatchedVerifier,
            &verification_request_id,
            &WriterLease { epoch: 1 },
        );

        assert!(matches!(
            report_result,
            Err(VerificationExecutorError::VerifierIdentityMismatch)
        ));
        assert_eq!(
            store
                .load_verification_report(&object_id(30))
                .expect("read absent identity-mismatched verification report"),
            None
        );

        let _ = std::fs::remove_file(database_path);
        let _ = std::fs::remove_dir_all(artifact_directory);
    }

    #[test]
    fn runtime_spine_false_completion_self_check_rejects_passed_report_as_task_completion() {
        // Suite floor: a passed independent verification report is evidence only.
        // Remote-done / zero-exit / report-passed narratives must not complete Task.
        let database_path = temporary_database_path();
        let store = SqliteAuthorityStore::open(&database_path).expect("open authority store");
        let (artifact_directory, artifact_store, artifact_evidence_ref) =
            artifact_store_with_evidence();
        let task_object_id = object_id(41);
        admit_task_fixture(&store, &task_object_id);
        let verification_request_id = persist_verification_fixture(&store, &task_object_id);

        let report = record_independent_verification(
            &store,
            &artifact_store,
            &FixedClock,
            &SequentialIdentifiers::new(40),
            &PassingVerifier {
                artifact_evidence_ref,
            },
            &verification_request_id,
            &WriterLease { epoch: 1 },
        )
        .expect("verification report persists");

        assert_eq!(report.status, "passed");
        let task = store
            .load_object(LifecycleDomain::Task, &task_object_id)
            .expect("load task")
            .expect("task remains durable");
        assert_eq!(task.state.as_str(), "DRAFT");
        assert_eq!(task.version, Version::INITIAL);
        for forbidden in ["CANDIDATE_COMPLETE", "COMPLETED", "FAILED", "CANCELLED"] {
            assert_ne!(
                task.state.as_str(),
                forbidden,
                "passed report must not derive Task {forbidden}"
            );
        }

        let _ = std::fs::remove_file(database_path);
        let _ = std::fs::remove_dir_all(artifact_directory);
    }
}
