//! P2-T03/D05 candidate persistence regression coverage.
//!
//! Operation candidates are non-authority observations. This suite proves the
//! durable input boundary is append-only and cannot replace a previously
//! observed proposal under the same identity.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use cognitive_domain::ObjectId;
use cognitive_kernel::ports::{
    OperationCandidateProposalRow, StorePortError, WorkerAuthorizationStore,
};
use cognitive_store::SqliteAuthorityStore;

fn object_id(sequence: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}")).unwrap()
}

fn candidate_proposal(candidate_sequence: u64) -> OperationCandidateProposalRow {
    OperationCandidateProposalRow {
        candidate_id: object_id(candidate_sequence),
        task_ref: "task://personal/worker-authorization".to_owned(),
        contract_epoch: 1,
        candidate_source_ref: "observation://personal/pi-shell/attempt-1".to_owned(),
        tool_ref: "operation://personal/filesystem/read".to_owned(),
        action: "filesystem.read".to_owned(),
        target: "file:///workspace/input.txt".to_owned(),
        parameters_digest: format!("sha256:{}", "ab".repeat(32)),
        expected_state_version: 1,
        operation_descriptor_ref: object_id(candidate_sequence + 1),
        canonical_json: format!("{{\"candidate\":{candidate_sequence}}}"),
    }
}

#[test]
fn duplicate_candidate_identity_cannot_replace_observed_input() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&temporary_directory.path().join("authority.db"))
        .expect("fresh authority database opens");
    let original = candidate_proposal(100);

    store
        .append_operation_candidate_proposal(&original)
        .expect("first candidate observation persists");

    let mut replacement = original.clone();
    replacement.action = "filesystem.delete".to_owned();
    replacement.canonical_json = "{\"candidate\":\"replacement\"}".to_owned();
    let duplicate = store
        .append_operation_candidate_proposal(&replacement)
        .expect_err("candidate identities are append-only");
    assert!(matches!(duplicate, StorePortError::Conflict { .. }));

    assert_eq!(
        store
            .load_operation_candidate_proposal(&original.candidate_id)
            .expect("candidate load succeeds"),
        Some(original),
        "a rejected duplicate must not overwrite the auditable observation"
    );
}
