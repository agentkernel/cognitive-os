//! P2-T03/D05 candidate persistence regression coverage.
//!
//! Operation candidates are non-authority observations. This suite proves the
//! durable input boundary is append-only and cannot replace a previously
//! observed proposal under the same identity.

#![allow(clippy::expect_used, clippy::unwrap_used)]

#[path = "m4_common/mod.rs"]
mod m4_common;

use cognitive_contracts::generated::common_defs::Budget;
use cognitive_contracts::generated::governed_object_header::GovernedObjectHeaderSensitivity;
use cognitive_contracts::generated::object_reference::UuidV7;
use cognitive_contracts::generated::operation_candidate_proposal::OperationCandidateProposal;
use cognitive_contracts::generated::task_contract::{
    ContractCondition, ContractConditionKind, TaskContract, TaskScope,
};
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, RecordId, StateName, Version, WallTimestamp,
};
use cognitive_kernel::candidate_admission::{
    CandidateAdmissionFacts, CandidateAdmissionIdentities, CandidateAdmissionInputs,
};
use cognitive_kernel::intent_chain::{
    GovernanceSeed, compose_governed_header, seal_governed_object_content_digest,
    strong_reference_to,
};
use cognitive_kernel::ports::{
    AuthorityStore, BoundWorkerAuthorizationConsumption, BudgetCas, CandidateAdmissionCommit,
    DaemonAuthorizationSnapshotRow, DaemonOperationDescriptorRow, EventDraft, IntentChainStore,
    IntentRow, ObjectAdmission, ObjectCas, OperationCandidateProposalRow, ProtocolStore,
    RecordDraft, SchedulerLeaseBinding, StorePortError, StoredObject, TaskBinding, TaskContractRow,
    TransitionCommit, WorkerAuthorizationStore, WorkerIterationAuthorizationConsumptionRow,
    WorkerIterationAuthorizationRow,
};
use cognitive_kernel::{
    BudgetCharge, BudgetState, EffectClass, ExecutorCapabilities, OperationDescriptor,
    TransitionEngine, compose_candidate_admission,
};
use cognitive_store::SqliteAuthorityStore;
use rusqlite::Connection;
use serde_json::json;
use std::collections::BTreeMap;

use m4_common::{FixedClock, SeqIds, admit, drive, lease, uri};

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
            // Intent rows are immutable protocol records rather than a
            // lifecycle domain; their provenance event is effect-scoped.
            domain: LifecycleDomain::Effect,
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

fn governance_seed() -> GovernanceSeed {
    GovernanceSeed {
        owner: strong_reference_to(&object_id(901), &format!("sha256:{}", "a".repeat(64))),
        authority: strong_reference_to(&object_id(902), &format!("sha256:{}", "b".repeat(64))),
        resource_scope: strong_reference_to(&object_id(903), &format!("sha256:{}", "c".repeat(64))),
        tenant_id: Some("00000000-0000-7000-9000-0000000000f1".to_owned()),
        created_by: "principal://personal/daemon".to_owned(),
        sensitivity: GovernedObjectHeaderSensitivity::Internal,
        purpose_constraints: vec!["task_execution".to_owned()],
        retention_policy: "standard".to_owned(),
    }
}

fn seal_canonical(value: serde_json::Value) -> (String, String) {
    let (sealed, digest) = seal_governed_object_content_digest(value).unwrap();
    (serde_json::to_string(&sealed).unwrap(), digest)
}

fn schema_valid_admission_inputs(
    loop_id: ObjectId,
    budget: BudgetId,
    expected_loop_version: Version,
) -> CandidateAdmissionInputs {
    let contract_id = object_id(500);
    let candidate_id = object_id(501);
    let descriptor_id = object_id(502);
    let admitted_at = WallTimestamp::parse("2026-08-04T12:00:00Z").unwrap();
    let governance = governance_seed();
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
            description: "independent verifier confirms completion".to_owned(),
            id: "acceptance".to_owned(),
            kind: ContractConditionKind::Acceptance,
            machine_expression: None,
            verifier_ref: Some("verifier://personal/test".to_owned()),
        }],
        contract_epoch: 1,
        deadline: None,
        header: compose_governed_header(
            &contract_id,
            "TaskContract",
            "cognitiveos.task-contract/0.3",
            &governance,
            Vec::new(),
            Vec::new(),
            "integration-contract",
            &admitted_at,
        )
        .unwrap(),
        human_gates: None,
        intent_acceptance_ref: strong_reference_to(
            &object_id(510),
            &format!("sha256:{}", "d".repeat(64)),
        ),
        intent_interpretation_ref: strong_reference_to(
            &object_id(511),
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
            &object_id(512),
            &format!("sha256:{}", "f".repeat(64)),
        ),
        worker_authorization_root_id: Some(UuidV7(contract_id.to_string())),
    };
    let (contract_canonical_json, contract_digest) =
        seal_canonical(serde_json::to_value(&contract).unwrap());
    let sealed_contract: TaskContract = serde_json::from_str(&contract_canonical_json).unwrap();
    let candidate = OperationCandidateProposal {
        action: "filesystem.read".to_owned(),
        candidate_source_ref: "observation://personal/test".to_owned(),
        contract_epoch: 1,
        expected_state_version: 1,
        header: compose_governed_header(
            &candidate_id,
            "OperationCandidateProposal",
            "cognitiveos.operation-candidate-proposal/0.1",
            &governance,
            vec!["observation://personal/test".to_owned()],
            vec![contract_id.to_string()],
            "integration-candidate",
            &admitted_at,
        )
        .unwrap(),
        operation_descriptor_ref: strong_reference_to(
            &descriptor_id,
            &format!("sha256:{}", "1".repeat(64)),
        ),
        parameters_digest: format!("sha256:{}", "2".repeat(64)),
        target: "file:///workspace/input.txt".to_owned(),
        task_contract_ref: strong_reference_to(
            &contract_id,
            &sealed_contract.header.content_digest.0,
        ),
        tool_ref: "operation://personal/filesystem/read".to_owned(),
    };
    let (candidate_canonical_json, _) = seal_canonical(serde_json::to_value(&candidate).unwrap());
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
        task_contract: TaskContractRow {
            contract_id: contract_id.clone(),
            task_ref: "task://personal/candidate-admission".to_owned(),
            contract_epoch: 1,
            user_intent_record_id: object_id(510),
            interpretation_id: object_id(511),
            accepted_by: "principal://personal/daemon".to_owned(),
            contract_digest,
            canonical_json: contract_canonical_json,
        },
        descriptor: daemon_descriptor(502),
        authorization: authorization_snapshot(503, "2026-08-04T12:00:00Z"),
        authorization_subject_ref: "principal://personal/daemon".to_owned(),
        authorization_purpose: "task_execution".to_owned(),
        facts: CandidateAdmissionFacts {
            loop_object_id: loop_id,
            budget_id: budget,
            expected_budget_version: Version::INITIAL,
            next_budget_state_canonical_json: "{\"tool_calls\":1}".to_owned(),
            expected_loop_version,
            iteration: 1,
        },
        budget_charge: BudgetCharge::new(BTreeMap::from([("tool_calls".to_owned(), 1)])).unwrap(),
        governance,
        identities: CandidateAdmissionIdentities {
            authorization_id: object_id(520),
            intent_id: object_id(521),
            effect_object_id: object_id(522),
            intent_event_id: event_id(523),
            effect_event_id: event_id(524),
            loop_event_id: event_id(525),
            loop_record_id: record_id(526),
        },
        actor_ref: uri("principal://personal/daemon"),
        authority_ref: uri("authority://personal/daemon"),
        correlation_id: uri("correlation://personal/candidate-admission"),
        admitted_at,
        writer_lease: lease(1),
    }
}

#[test]
fn composer_bundle_commits_all_candidate_admission_authority_atomically() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&temporary_directory.path().join("authority.db"))
        .expect("fresh authority database opens");
    let clock = FixedClock::new();
    let ids = SeqIds::from(700);
    let loop_id = object_id(530);
    let budget = budget_id(531);
    admit(
        &store,
        &clock,
        &ids,
        &loop_id,
        LifecycleDomain::Loop,
        Some(1),
    );
    let mut loop_version = Version::INITIAL;
    for (from_state, to_state, reason_code) in [
        ("START", "OBSERVE", "LOOP_STARTED"),
        ("OBSERVE", "RESOLVE", "EVIDENCE_OBSERVED"),
        ("RESOLVE", "ORIENT", "CONTEXT_COMPLETE"),
        ("ORIENT", "DECIDE", "ORIENTATION_COMPLETE"),
    ] {
        loop_version = drive(
            &store,
            &clock,
            &ids,
            LifecycleDomain::Loop,
            &loop_id,
            from_state,
            to_state,
            reason_code,
            loop_version,
            Some(1),
        );
    }
    let engine = TransitionEngine::new(&store, &clock, &ids);
    engine
        .create_budget(
            &budget,
            &BudgetState::new(BTreeMap::from([("tool_calls".to_owned(), 2)])).unwrap(),
        )
        .expect("budget creation succeeds");

    let inputs = schema_valid_admission_inputs(loop_id.clone(), budget.clone(), loop_version);
    store
        .insert_task_contract(
            &inputs.task_contract,
            &EventDraft {
                event_id: event_id(530),
                object_id: inputs.task_contract.contract_id.clone(),
                domain: LifecycleDomain::Task,
                object_version: Version::INITIAL,
                event_type: "task-contract.minted".to_owned(),
                canonical_json: "{\"event\":\"task-contract\"}".to_owned(),
            },
            0,
        )
        .expect("current TaskContract persists before admission");
    store
        .append_operation_candidate_proposal(&inputs.candidate)
        .expect("sealed candidate observation persists");
    store
        .append_daemon_operation_descriptor(&inputs.descriptor)
        .expect("daemon descriptor persists");
    store
        .append_daemon_authorization_snapshot(&inputs.authorization)
        .expect("daemon authorization snapshot persists");

    let commit = compose_candidate_admission(&inputs).expect("daemon inputs compose");
    let receipt = store
        .commit_candidate_admission(&commit)
        .expect("all candidate-admission authority persists together");

    assert_eq!(
        receipt.authorization_id,
        commit.worker_authorization.authorization_id
    );
    assert!(receipt.intent_event_sequence > 0);
    assert!(receipt.effect_admission_event_sequence > receipt.intent_event_sequence);
    assert!(receipt.loop_transition_event_sequence > receipt.effect_admission_event_sequence);
    let effect = store
        .load_object(
            LifecycleDomain::Effect,
            &commit.effect_admission.object.object_id,
        )
        .expect("effect lookup succeeds")
        .expect("candidate admission persists its Effect");
    assert_eq!(effect.state.as_str(), "PROPOSED");
    assert_eq!(effect.version, Version::INITIAL);
    let loop_object = store
        .load_object(LifecycleDomain::Loop, &loop_id)
        .expect("Loop lookup succeeds")
        .expect("Loop remains durable");
    assert_eq!(loop_object.state.as_str(), "ACT");
    assert_eq!(
        loop_object.version,
        Version::new(loop_version.get() + 1).unwrap()
    );
    let persisted_budget = store
        .load_budget(&budget)
        .expect("budget lookup succeeds")
        .expect("budget remains durable");
    assert_eq!(persisted_budget.version, Version::new(2).unwrap());
    assert_eq!(
        persisted_budget.state.remaining().get("tool_calls"),
        Some(&1)
    );
    assert_eq!(
        store
            .load_worker_iteration_authorization(&commit.worker_authorization.authorization_id)
            .expect("WIA lookup succeeds"),
        Some(commit.worker_authorization.clone())
    );
    assert_eq!(
        store
            .load_intent_for_effect(&commit.effect_admission.object.object_id)
            .expect("Intent lookup succeeds"),
        Some(commit.intent.clone())
    );
    assert!(
        store
            .list_consumed_worker_iteration_authorizations()
            .expect("recovery discovery before handoff succeeds")
            .is_empty(),
        "issued-but-unconsumed WIA is authority, not evidence of an active worker"
    );
    let consumption = WorkerIterationAuthorizationConsumptionRow {
        authorization_id: commit.worker_authorization.authorization_id.clone(),
        worker_attempt_id: object_id(527),
        consumed_fencing_epoch: 1,
        consumed_at: WallTimestamp::parse("2026-08-04T12:01:00Z").unwrap(),
        canonical_json: "{\"worker_authorization_consumption\":1}".to_owned(),
    };
    store
        .consume_worker_iteration_authorization(&consumption)
        .expect("a real daemon-issued WIA is consumed exactly once");
    assert_eq!(
        store
            .list_consumed_worker_iteration_authorizations()
            .expect("recovery discovery succeeds"),
        vec![
            cognitive_kernel::ports::ConsumedWorkerIterationAuthorization {
                authorization: commit.worker_authorization.clone(),
                consumption: consumption.clone(),
                scheduler_lease: None,
            }
        ],
        "recovery discovers only a daemon-recorded worker handoff"
    );
    let duplicate_consumption = store
        .consume_worker_iteration_authorization(&consumption)
        .expect_err("a WIA cannot be handed to a second worker attempt");
    assert!(matches!(
        duplicate_consumption,
        StorePortError::Conflict { .. }
    ));
    assert_eq!(
        store
            .load_object(
                LifecycleDomain::Effect,
                &commit.effect_admission.object.object_id,
            )
            .expect("effect lookup after handoff succeeds"),
        Some(effect),
        "WIA consumption records handoff only; it cannot change Effect state"
    );
    assert_eq!(
        store
            .load_object(LifecycleDomain::Loop, &loop_id)
            .expect("Loop lookup after handoff succeeds"),
        Some(loop_object),
        "WIA consumption cannot claim Loop progress"
    );
    assert_eq!(
        store
            .load_budget(&budget)
            .expect("budget lookup after handoff succeeds"),
        Some(persisted_budget),
        "WIA consumption cannot debit authority a second time"
    );
    assert_eq!(
        store
            .read_events(0, 100)
            .expect("event lookup succeeds")
            .len(),
        9,
        "three admission events join the Loop and TaskContract evidence; budget creation is ledger-only"
    );
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
fn atomic_admission_rejects_any_loop_edge_except_decide_to_act() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&temporary_directory.path().join("authority.db"))
        .expect("fresh authority database opens");
    let mut commit = admission_commit(object_id(498));
    commit.loop_transition.cas.to_state = state("VERIFY");

    let error = store
        .commit_candidate_admission(&commit)
        .expect_err("candidate admission may only move a Loop from DECIDE to ACT");
    assert!(matches!(error, StorePortError::Conflict { .. }));
    assert!(
        store
            .read_events(0, 10)
            .expect("event lookup succeeds")
            .is_empty(),
        "an invalid Loop edge must fail before any authority record is written"
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

#[test]
fn bound_wia_handoff_requires_and_recovers_the_exact_active_scheduler_lease() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let authority_database_path = temporary_directory.path().join("authority.db");
    let store = SqliteAuthorityStore::open(&authority_database_path)
        .expect("fresh authority database opens");
    let authorization_id = object_id(700);
    let worker_attempt_id = object_id(701);
    let task_ref = "task://personal/exact-worker-handoff";

    let connection = Connection::open(&authority_database_path).expect("open authority fixture");
    connection
        .execute(
            "INSERT INTO worker_iteration_authorizations
               (authorization_id, worker_authorization_root_id, task_ref, contract_epoch,
                loop_object_id, iteration, expected_loop_version, selected_candidate_id,
                intent_id, effect_object_id, budget_id, budget_charge_json,
                action_fingerprint, issued_fencing_epoch, canonical_json)
             VALUES (?1, ?2, ?3, 1, ?4, 1, 1, ?5, ?6, ?7, ?8, ?9, ?10, 1, ?11)",
            rusqlite::params![
                authorization_id.as_str(),
                object_id(702).as_str(),
                task_ref,
                object_id(703).as_str(),
                object_id(704).as_str(),
                object_id(705).as_str(),
                object_id(706).as_str(),
                budget_id(707).as_str(),
                "{\"tool_calls\":1}",
                "exact-worker-handoff",
                "{\"worker_authorization\":1}",
            ],
        )
        .expect("seed immutable WIA");
    connection
        .execute(
            "INSERT INTO scheduler_entries
               (task_ref, contract_epoch, state, lease_owner, lease_epoch, lease_expires,
                next_eligible, attempt_count, cancel_requested)
             VALUES (?1, 1, 'leased', 'daemon-worker-a', 11, ?2, ?3, 1, 0)",
            rusqlite::params![task_ref, "2026-08-04T12:05:00Z", "2026-08-04T12:00:00Z"],
        )
        .expect("seed exact active scheduler lease");
    drop(connection);

    let request = BoundWorkerAuthorizationConsumption {
        consumption: WorkerIterationAuthorizationConsumptionRow {
            authorization_id: authorization_id.clone(),
            worker_attempt_id: worker_attempt_id.clone(),
            consumed_fencing_epoch: 1,
            consumed_at: WallTimestamp::parse("2026-08-04T12:01:00Z").unwrap(),
            canonical_json: "{\"worker_authorization_consumption\":1}".to_owned(),
        },
        scheduler_lease: SchedulerLeaseBinding {
            task_ref: task_ref.to_owned(),
            contract_epoch: 1,
            lease_owner: "daemon-worker-a".to_owned(),
            lease_epoch: 11,
        },
    };
    store
        .consume_worker_iteration_authorization_bound_to_scheduler_lease(&request)
        .expect("handoff commits only for its exact active scheduler lease");
    let mut duplicate_request = request.clone();
    duplicate_request.consumption.worker_attempt_id = object_id(708);
    let duplicate = store
        .consume_worker_iteration_authorization_bound_to_scheduler_lease(&duplicate_request)
        .expect_err("a WIA cannot be bound to a second scheduler worker attempt");
    assert!(matches!(duplicate, StorePortError::Conflict { .. }));

    let recovered = store
        .list_consumed_worker_iteration_authorizations()
        .expect("recovery reads bound handoff");
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].consumption, request.consumption);
    assert_eq!(recovered[0].scheduler_lease, Some(request.scheduler_lease));
}

#[test]
fn bound_wia_handoff_rejects_a_replaced_lease_without_consuming_authority() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let authority_database_path = temporary_directory.path().join("authority.db");
    let store = SqliteAuthorityStore::open(&authority_database_path)
        .expect("fresh authority database opens");
    let authorization_id = object_id(720);
    let task_ref = "task://personal/replaced-worker-handoff";
    let connection = Connection::open(&authority_database_path).expect("open authority fixture");
    connection
        .execute(
            "INSERT INTO worker_iteration_authorizations
           (authorization_id, worker_authorization_root_id, task_ref, contract_epoch,
            loop_object_id, iteration, expected_loop_version, selected_candidate_id,
            intent_id, effect_object_id, budget_id, budget_charge_json,
            action_fingerprint, issued_fencing_epoch, canonical_json)
         VALUES (?1, ?2, ?3, 1, ?4, 1, 1, ?5, ?6, ?7, ?8, ?9, ?10, 1, ?11)",
            rusqlite::params![
                authorization_id.as_str(),
                object_id(721).as_str(),
                task_ref,
                object_id(722).as_str(),
                object_id(723).as_str(),
                object_id(724).as_str(),
                object_id(725).as_str(),
                budget_id(726).as_str(),
                "{\"tool_calls\":1}",
                "replaced-worker-handoff",
                "{\"worker_authorization\":1}"
            ],
        )
        .expect("seed immutable WIA");
    connection
        .execute(
            "INSERT INTO scheduler_entries
           (task_ref, contract_epoch, state, lease_owner, lease_epoch, lease_expires,
            next_eligible, attempt_count, cancel_requested)
         VALUES (?1, 1, 'leased', 'replacement-worker', 12, ?2, ?3, 1, 0)",
            rusqlite::params![task_ref, "2026-08-04T12:05:00Z", "2026-08-04T12:00:00Z"],
        )
        .expect("seed replacement scheduler lease");
    drop(connection);

    let error = store
        .consume_worker_iteration_authorization_bound_to_scheduler_lease(
            &BoundWorkerAuthorizationConsumption {
                consumption: WorkerIterationAuthorizationConsumptionRow {
                    authorization_id,
                    worker_attempt_id: object_id(727),
                    consumed_fencing_epoch: 1,
                    consumed_at: WallTimestamp::parse("2026-08-04T12:01:00Z").unwrap(),
                    canonical_json: "{\"worker_authorization_consumption\":1}".to_owned(),
                },
                scheduler_lease: SchedulerLeaseBinding {
                    task_ref: task_ref.to_owned(),
                    contract_epoch: 1,
                    lease_owner: "original-worker".to_owned(),
                    lease_epoch: 11,
                },
            },
        )
        .expect_err("a replacement owner and epoch cannot receive the original handoff");
    assert!(matches!(error, StorePortError::Conflict { .. }));
    assert!(
        store
            .list_consumed_worker_iteration_authorizations()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn bound_wia_handoff_rejects_cancelled_work_without_consuming_authority() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let authority_database_path = temporary_directory.path().join("authority.db");
    let store = SqliteAuthorityStore::open(&authority_database_path)
        .expect("fresh authority database opens");
    let authorization_id = object_id(730);
    let task_ref = "task://personal/cancelled-worker-handoff";
    let connection = Connection::open(&authority_database_path).expect("open authority fixture");
    connection
        .execute(
            "INSERT INTO worker_iteration_authorizations
               (authorization_id, worker_authorization_root_id, task_ref, contract_epoch,
                loop_object_id, iteration, expected_loop_version, selected_candidate_id,
                intent_id, effect_object_id, budget_id, budget_charge_json,
                action_fingerprint, issued_fencing_epoch, canonical_json)
             VALUES (?1, ?2, ?3, 1, ?4, 1, 1, ?5, ?6, ?7, ?8, ?9, ?10, 1, ?11)",
            rusqlite::params![
                authorization_id.as_str(),
                object_id(731).as_str(),
                task_ref,
                object_id(732).as_str(),
                object_id(733).as_str(),
                object_id(734).as_str(),
                object_id(735).as_str(),
                budget_id(736).as_str(),
                "{\"tool_calls\":1}",
                "cancelled-worker-handoff",
                "{\"worker_authorization\":1}",
            ],
        )
        .expect("seed immutable WIA");
    connection
        .execute(
            "INSERT INTO scheduler_entries
               (task_ref, contract_epoch, state, lease_owner, lease_epoch, lease_expires,
                next_eligible, attempt_count, cancel_requested)
             VALUES (?1, 1, 'leased', 'daemon-worker-a', 11, ?2, ?3, 1, 1)",
            rusqlite::params![task_ref, "2026-08-04T12:05:00Z", "2026-08-04T12:00:00Z"],
        )
        .expect("seed cancelled scheduler lease");
    drop(connection);

    let error = store
        .consume_worker_iteration_authorization_bound_to_scheduler_lease(
            &BoundWorkerAuthorizationConsumption {
                consumption: WorkerIterationAuthorizationConsumptionRow {
                    authorization_id,
                    worker_attempt_id: object_id(737),
                    consumed_fencing_epoch: 1,
                    consumed_at: WallTimestamp::parse("2026-08-04T12:01:00Z").unwrap(),
                    canonical_json: "{\"worker_authorization_consumption\":1}".to_owned(),
                },
                scheduler_lease: SchedulerLeaseBinding {
                    task_ref: task_ref.to_owned(),
                    contract_epoch: 1,
                    lease_owner: "daemon-worker-a".to_owned(),
                    lease_epoch: 11,
                },
            },
        )
        .expect_err("cancelled scheduler work cannot receive a worker handoff");
    assert!(matches!(error, StorePortError::Conflict { .. }));
    assert!(
        store
            .list_consumed_worker_iteration_authorizations()
            .unwrap()
            .is_empty(),
        "rejected handoff must not persist consumption without its lease binding"
    );
}

#[test]
fn bound_wia_handoff_rejects_task_mismatch_without_consuming_authority() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let authority_database_path = temporary_directory.path().join("authority.db");
    let store = SqliteAuthorityStore::open(&authority_database_path)
        .expect("fresh authority database opens");
    let authorization_id = object_id(740);
    let authorized_task_ref = "task://personal/authorized-worker-handoff";
    let scheduler_task_ref = "task://personal/mismatched-worker-handoff";
    let connection = Connection::open(&authority_database_path).expect("open authority fixture");
    connection
        .execute(
            "INSERT INTO worker_iteration_authorizations
               (authorization_id, worker_authorization_root_id, task_ref, contract_epoch,
                loop_object_id, iteration, expected_loop_version, selected_candidate_id,
                intent_id, effect_object_id, budget_id, budget_charge_json,
                action_fingerprint, issued_fencing_epoch, canonical_json)
             VALUES (?1, ?2, ?3, 1, ?4, 1, 1, ?5, ?6, ?7, ?8, ?9, ?10, 1, ?11)",
            rusqlite::params![
                authorization_id.as_str(),
                object_id(741).as_str(),
                authorized_task_ref,
                object_id(742).as_str(),
                object_id(743).as_str(),
                object_id(744).as_str(),
                object_id(745).as_str(),
                budget_id(746).as_str(),
                "{\"tool_calls\":1}",
                "mismatched-worker-handoff",
                "{\"worker_authorization\":1}",
            ],
        )
        .expect("seed immutable WIA for another task");
    connection
        .execute(
            "INSERT INTO scheduler_entries
               (task_ref, contract_epoch, state, lease_owner, lease_epoch, lease_expires,
                next_eligible, attempt_count, cancel_requested)
             VALUES (?1, 1, 'leased', 'daemon-worker-a', 11, ?2, ?3, 1, 0)",
            rusqlite::params![
                scheduler_task_ref,
                "2026-08-04T12:05:00Z",
                "2026-08-04T12:00:00Z"
            ],
        )
        .expect("seed unrelated active scheduler lease");
    drop(connection);

    let error = store
        .consume_worker_iteration_authorization_bound_to_scheduler_lease(
            &BoundWorkerAuthorizationConsumption {
                consumption: WorkerIterationAuthorizationConsumptionRow {
                    authorization_id,
                    worker_attempt_id: object_id(747),
                    consumed_fencing_epoch: 1,
                    consumed_at: WallTimestamp::parse("2026-08-04T12:01:00Z").unwrap(),
                    canonical_json: "{\"worker_authorization_consumption\":1}".to_owned(),
                },
                scheduler_lease: SchedulerLeaseBinding {
                    task_ref: scheduler_task_ref.to_owned(),
                    contract_epoch: 1,
                    lease_owner: "daemon-worker-a".to_owned(),
                    lease_epoch: 11,
                },
            },
        )
        .expect_err("a WIA cannot be consumed for a different scheduler task");
    assert!(matches!(error, StorePortError::Conflict { .. }));
    assert!(
        store
            .list_consumed_worker_iteration_authorizations()
            .unwrap()
            .is_empty(),
        "a task-mismatched handoff must not persist a partial consumption"
    );
}
