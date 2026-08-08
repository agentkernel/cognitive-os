//! Daemon-private artifact-backed verification boundary.
//!
//! A verifier evaluates one already-persisted request against a pinned
//! post-state and returns only a disposition plus immutable artifact references.
//! The daemon validates the binding and persists that observation. A passing
//! observation can later be consumed by continuation authority, but never
//! accepts or completes a Task here.

#![allow(dead_code)] // Runtime composition follows this daemon-private boundary.

use cognitive_domain::{ObjectId, WallTimestamp};
use cognitive_kernel::{
    effects::WriterLease,
    ports::{
        AuthorityStore, Clock, ContinuationAuthorityStore, FixedPostStateRow, IdGenerator,
        ProtocolStore, VerificationReportRow, VerificationRequestRow,
    },
};
use serde_json::json;
use thiserror::Error;

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
    V: IndependentVerifier,
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
mod tests {
    use super::*;

    fn artifact_reference(character: char) -> String {
        format!("artifact://sha256/{}", character.to_string().repeat(64))
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

        assert_eq!(validate_artifact_evidence_refs(&result), Ok(()));
    }
}
