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
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use cognitive_domain::{EventId, LifecycleDomain, StateName, Version};
    use cognitive_kernel::ports::{EventDraft, ObjectAdmission, StoredObject, TaskBinding};
    use cognitive_store::SqliteAuthorityStore;
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

    struct PassingVerifier;

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
                artifact_evidence_refs: vec![artifact_reference('c')],
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
    fn durable_sqlite_report_binds_a_current_post_state_without_task_completion() {
        let database_path = temporary_database_path();
        let store = SqliteAuthorityStore::open(&database_path).expect("open authority store");
        let task_object_id = object_id(1);
        admit_task_fixture(&store, &task_object_id);
        let verification_request_id = persist_verification_fixture(&store, &task_object_id);

        let report_result = record_independent_verification(
            &store,
            &FixedClock,
            &SequentialIdentifiers::new(10),
            &PassingVerifier,
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
    }

    #[test]
    fn durable_sqlite_report_rejects_a_fenced_writer_before_persistence() {
        let database_path = temporary_database_path();
        let store = SqliteAuthorityStore::open(&database_path).expect("open authority store");
        let task_object_id = object_id(11);
        admit_task_fixture(&store, &task_object_id);
        let verification_request_id = persist_verification_fixture(&store, &task_object_id);

        let report_result = record_independent_verification(
            &store,
            &FixedClock,
            &SequentialIdentifiers::new(20),
            &PassingVerifier,
            &verification_request_id,
            &WriterLease { epoch: 2 },
        );

        assert!(matches!(
            report_result,
            Err(VerificationExecutorError::WriterFenced)
        ));
        assert_eq!(
            store.load_verification_report(&object_id(20)).unwrap(),
            None
        );

        let _ = std::fs::remove_file(database_path);
    }

    #[test]
    fn durable_sqlite_report_rejects_an_unregistered_verifier_before_evaluation() {
        let database_path = temporary_database_path();
        let store = SqliteAuthorityStore::open(&database_path).expect("open authority store");
        let task_object_id = object_id(21);
        admit_task_fixture(&store, &task_object_id);
        let verification_request_id = persist_verification_fixture(&store, &task_object_id);

        let report_result = record_independent_verification(
            &store,
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
            store.load_verification_report(&object_id(30)).unwrap(),
            None
        );

        let _ = std::fs::remove_file(database_path);
    }
}
