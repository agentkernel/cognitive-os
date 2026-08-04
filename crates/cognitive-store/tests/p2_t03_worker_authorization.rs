//! P2-T03/D05 candidate persistence regression coverage.
//!
//! Operation candidates are non-authority observations. This suite proves the
//! durable input boundary is append-only and cannot replace a previously
//! observed proposal under the same identity.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, RecordId, StateName, Version, WallTimestamp,
};
use cognitive_kernel::ports::{
    AuthorityStore, BudgetCas, CandidateAdmissionCommit, DaemonAuthorizationSnapshotRow,
    DaemonOperationDescriptorRow, EventDraft, IntentRow, ObjectAdmission, ObjectCas,
    OperationCandidateProposalRow, RecordDraft, StorePortError, StoredObject, TaskBinding,
    TransitionCommit, WorkerAuthorizationStore, WorkerIterationAuthorizationConsumptionRow,
    WorkerIterationAuthorizationRow,
};
use cognitive_kernel::{EffectClass, ExecutorCapabilities, OperationDescriptor};
use cognitive_store::SqliteAuthorityStore;
use serde_json::json;

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

fn daemon_descriptor(descriptor_sequence: u64) -> DaemonOperationDescriptorRow {
    DaemonOperationDescriptorRow {
        descriptor_id: object_id(descriptor_sequence),
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
        canonical_json: format!("{{\"descriptor\":{descriptor_sequence}}}"),
    }
}

fn authorization_snapshot(
    snapshot_sequence: u64,
    observed_at: &str,
) -> DaemonAuthorizationSnapshotRow {
    DaemonAuthorizationSnapshotRow {
        snapshot_id: object_id(snapshot_sequence),
        subject_ref: "principal://personal/daemon".to_owned(),
        target_ref: "file:///workspace/input.txt".to_owned(),
        action: "filesystem.read".to_owned(),
        purpose: "task_execution".to_owned(),
        grant_epoch: 1,
        capability_set_version: 1,
        revocation_epoch: 1,
        observed_at: WallTimestamp::parse(observed_at).unwrap(),
        canonical_json: format!("{{\"authorization_snapshot\":{snapshot_sequence}}}"),
    }
}

fn budget_id(sequence: u64) -> BudgetId {
    BudgetId::parse(&format!("00000000-0000-7000-b000-{sequence:012x}")).unwrap()
}

fn event_id(sequence: u64) -> EventId {
    EventId::parse(&format!("00000000-0000-7000-a000-{sequence:012x}")).unwrap()
}

fn record_id(sequence: u64) -> RecordId {
    RecordId::parse(&format!("00000000-0000-7000-8000-{sequence:012x}")).unwrap()
}

fn state(value: &str) -> StateName {
    StateName::parse(value).unwrap()
}

fn admission_commit(candidate_id: ObjectId) -> CandidateAdmissionCommit {
    let intent_id = object_id(410);
    let effect_id = object_id(411);
    let loop_id = object_id(412);
    let budget = budget_id(413);
    CandidateAdmissionCommit {
        selected_candidate_id: candidate_id.clone(),
        intent: IntentRow {
            intent_id: intent_id.clone(),
            idempotency_key: "candidate-admission-missing-candidate".to_owned(),
            parameters_digest: format!("sha256:{}", "ab".repeat(32)),
            action: "filesystem.read".to_owned(),
            target: "file:///workspace/input.txt".to_owned(),
            effect_object_id: effect_id.clone(),
            expected_state_version: Version::INITIAL,
            grant_epoch: 1,
            capability_set_version: 1,
            task_binding: Some(TaskBinding {
                task_ref: "task://personal/worker-authorization".to_owned(),
                contract_epoch: 1,
            }),
            canonical_json: "{\"intent\":\"candidate-admission\"}".to_owned(),
        },
        intent_event: EventDraft {
            event_id: event_id(410),
            object_id: intent_id.clone(),
            domain: LifecycleDomain::Intent,
            object_version: Version::INITIAL,
            event_type: "intent.minted".to_owned(),
            canonical_json: "{\"event\":\"intent\"}".to_owned(),
        },
        effect_admission: ObjectAdmission {
            object: StoredObject {
                object_id: effect_id.clone(),
                domain: LifecycleDomain::Effect,
                state: state("PROPOSED"),
                version: Version::INITIAL,
                body: json!({"effect": "candidate-admission"}),
            },
            admitted_at: WallTimestamp::parse("2026-08-03T12:00:00Z").unwrap(),
            event: EventDraft {
                event_id: event_id(411),
                object_id: effect_id.clone(),
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "effect.admitted".to_owned(),
                canonical_json: "{\"event\":\"effect\"}".to_owned(),
            },
            outbox: vec![],
            fencing_epoch: Some(1),
        },
        worker_authorization: WorkerIterationAuthorizationRow {
            authorization_id: object_id(414),
            worker_authorization_root_id: object_id(415),
            task_ref: "task://personal/worker-authorization".to_owned(),
            contract_epoch: 1,
            loop_object_id: loop_id.clone(),
            iteration: 1,
            expected_loop_version: Version::INITIAL,
            selected_candidate_id: candidate_id,
            intent_id,
            effect_object_id: effect_id,
            budget_id: budget.clone(),
            budget_charge_canonical_json: "{\"tool_calls\":1}".to_owned(),
            action_fingerprint: "candidate-admission-1".to_owned(),
            issued_fencing_epoch: 1,
            canonical_json: "{\"worker_authorization\":1}".to_owned(),
        },
        loop_transition: TransitionCommit {
            cas: ObjectCas {
                object_id: loop_id.clone(),
                domain: LifecycleDomain::Loop,
                from_state: state("DECIDE"),
                to_state: state("ACT"),
                expected_version: Version::INITIAL,
                next_version: Version::new(2).unwrap(),
                committed_at: WallTimestamp::parse("2026-08-03T12:00:00Z").unwrap(),
            },
            event: EventDraft {
                event_id: event_id(412),
                object_id: loop_id.clone(),
                domain: LifecycleDomain::Loop,
                object_version: Version::new(2).unwrap(),
                event_type: "loop.operation-admitted".to_owned(),
                canonical_json: "{\"event\":\"loop\"}".to_owned(),
            },
            record: RecordDraft {
                record_id: record_id(412),
                object_id: loop_id,
                domain: LifecycleDomain::Loop,
                object_version: Version::new(2).unwrap(),
                canonical_json: "{\"record\":\"loop\"}".to_owned(),
            },
            budget: Some(BudgetCas {
                budget_id: budget,
                expected_version: Version::INITIAL,
                next_version: Version::new(2).unwrap(),
                next_state_canonical_json: "{\"tool_calls\":0}".to_owned(),
            }),
            outbox: vec![],
            fencing_epoch: Some(1),
        },
        fencing_epoch: 1,
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

#[test]
fn daemon_descriptor_registry_is_append_only_and_resolves_exact_reference() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&temporary_directory.path().join("authority.db"))
        .expect("fresh authority database opens");
    let original = daemon_descriptor(200);

    store
        .append_daemon_operation_descriptor(&original)
        .expect("daemon descriptor persists");

    let mut replacement = original.clone();
    replacement.descriptor.action = "filesystem.delete".to_owned();
    let duplicate = store
        .append_daemon_operation_descriptor(&replacement)
        .expect_err("descriptor identities are append-only");
    assert!(matches!(duplicate, StorePortError::Conflict { .. }));

    assert_eq!(
        store
            .load_daemon_operation_descriptor(&original.descriptor_id)
            .expect("descriptor load succeeds"),
        Some(original),
        "candidate admission must resolve the daemon-recorded descriptor, not a replacement"
    );
}

#[test]
fn latest_daemon_authorization_snapshot_is_binding_specific_and_immutable() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&temporary_directory.path().join("authority.db"))
        .expect("fresh authority database opens");
    let first = authorization_snapshot(300, "2026-08-03T12:00:00Z");
    let latest = authorization_snapshot(301, "2026-08-03T12:01:00Z");

    store
        .append_daemon_authorization_snapshot(&first)
        .expect("first daemon authorization snapshot persists");
    store
        .append_daemon_authorization_snapshot(&latest)
        .expect("later daemon authorization snapshot persists");

    assert_eq!(
        store
            .load_latest_daemon_authorization_snapshot(
                "principal://personal/daemon",
                "file:///workspace/input.txt",
                "filesystem.read",
                "task_execution",
            )
            .expect("snapshot lookup succeeds"),
        Some(latest.clone())
    );
    assert!(
        store
            .load_latest_daemon_authorization_snapshot(
                "principal://personal/daemon",
                "file:///workspace/input.txt",
                "filesystem.delete",
                "task_execution",
            )
            .expect("mismatched lookup succeeds")
            .is_none(),
        "an authorization snapshot cannot be reused for another action"
    );

    let duplicate = store
        .append_daemon_authorization_snapshot(&latest)
        .expect_err("snapshot identities are append-only");
    assert!(matches!(duplicate, StorePortError::Conflict { .. }));
}

#[test]
fn missing_candidate_rejects_atomic_admission_without_authority_residue() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&temporary_directory.path().join("authority.db"))
        .expect("fresh authority database opens");
    let commit = admission_commit(object_id(499));

    let error = store
        .commit_candidate_admission(&commit)
        .expect_err("a daemon admission cannot mint authority for an absent candidate");
    assert!(matches!(error, StorePortError::Conflict { .. }));
    assert!(
        store
            .load_object(LifecycleDomain::Effect, &commit.intent.effect_object_id)
            .expect("effect lookup succeeds")
            .is_none()
    );
    assert!(
        store
            .read_events(0, 10)
            .expect("event lookup succeeds")
            .is_empty()
    );
}

#[test]
fn missing_wia_cannot_be_consumed_as_a_worker_handoff() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&temporary_directory.path().join("authority.db"))
        .expect("fresh authority database opens");
    let consumption = WorkerIterationAuthorizationConsumptionRow {
        authorization_id: object_id(600),
        worker_attempt_id: object_id(601),
        consumed_fencing_epoch: 1,
        consumed_at: WallTimestamp::parse("2026-08-03T12:00:00Z").unwrap(),
        canonical_json: "{\"worker_authorization_consumption\":1}".to_owned(),
    };

    let error = store
        .consume_worker_iteration_authorization(&consumption)
        .expect_err("a worker cannot consume authority that the daemon did not issue");
    assert!(matches!(error, StorePortError::Conflict { .. }));
}
