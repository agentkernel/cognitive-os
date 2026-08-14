//! Focused tests extracted from `scheduler_authority` (P9-T02/D01).

#![allow(clippy::unwrap_used)]

use super::{
    ContextResolutionCommand, RecoveredWorkerAttempt, SchedulerAuthorityBinding,
    SchedulerAuthorityError, SchedulerDispatchAdmission, SchedulerEffectClosure,
    SchedulerWorkerAttempt, UntrustedPiCandidate, WorkerAuthorizationHandoff,
    candidate_admission_command_from_policy, classify_scheduler_effect_closure,
    complete_resolved_effect_and_release, complete_scheduler_admission,
    complete_scheduler_worker_attempt, dispatch_native_worker_effect,
    ensure_current_contract_epoch, parse_execution_bound_contract,
    propose_persist_and_admit_candidate_after_metadata, reconcile_interrupted_native_worker_effect,
    release_closed_effect_dispatch, release_closed_recovered_attempt,
    resolve_native_worker_dispatch_with_families, resolve_scheduler_work_for_task,
    select_single_effect_intent, validate_untrusted_pi_candidate,
    validate_worker_authorization_evidence, verify_scheduler_dispatch_current,
};
use cognitive_contracts::{
    canonical,
    generated::governed_object_header::GovernedObjectHeaderSensitivity,
    generated::{
        common_defs::Budget,
        context_view::{
            LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
        },
        task_contract::{ContractCondition, ContractConditionKind, TaskContract, TaskScope},
        worker_iteration_authorization::WorkerIterationAuthorization,
    },
};
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, RecordId, StateName, UriRef, Version,
    WallTimestamp,
    capability::{CapabilityConstraints, LeaseWindow},
};
use cognitive_kernel::GovernedContextCache;
use cognitive_kernel::authz::{
    AccessRequest, ActorChainFacts, AuthzSnapshot, MembershipFacts, ObjectGovernance,
    PrincipalFacts, authorize,
};
use cognitive_kernel::budget::BudgetCharge;
use cognitive_kernel::budget::BudgetState;
use cognitive_kernel::effects::{
    EffectProtocol, GovernanceCurrency, VerificationRecord, VerificationStatus, WriterLease,
};
use cognitive_kernel::engine::CommittedTransition;
use cognitive_kernel::executor::ExecutorCapabilities;
use cognitive_kernel::intent_chain::{
    GovernanceSeed, compose_governed_header, prepare_task_execution_bootstrap,
    seal_governed_object_content_digest, strong_reference_to,
};
use cognitive_kernel::ports::{
    AuthorityStore, BudgetCas, CandidateAdmissionCommit, ContextAuthorizationFactStore,
    ContextAuthorizationFactsRow, ContextRequestRow, ContextRevocationFactRow, ContextStore,
    ContinuationAuthorityStore, DaemonOperationDescriptorRow, EventDraft, IntentChainStore,
    IntentRow, ObjectAdmission, ObjectCas, OperationCandidateProposalRow, ProgressFactRow,
    ProtocolStore, RecordDraft, SchedulerExecutionPolicyRow, SchedulerExecutionPolicyStore,
    SchedulerLeaseBinding, StoredObject, TaskBinding, TaskContractRow, TaskExecutionBootstrap,
    TransitionCommit, WorkerAuthorizationStore, WorkerIterationAuthorizationRow,
    WorkspaceContextSourceRow,
};
use cognitive_kernel::tool_registry::{BUILTIN_TOOL_CATALOG, NativeOperationFamily};
use cognitive_kernel::{EffectClass, OperationDescriptor};
use cognitive_runtime::{SchedulerCeilingDispatch, SchedulerDispatch};
use cognitive_store::{
    PersonalDataLayout, ScriptedExecutor, SqliteAuthorityStore, SystemClock, UuidV7Generator,
    prepare_personal_databases,
    scheduler::{SchedulerRepository, SchedulerRow, SchedulerState, SchedulerWorkKey},
};
use serde_json::json;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::personal::tool_executor::{
    ASSEMBLED_EXECUTOR_FAMILIES, NativeToolExecutionError, ProductionNativeToolExecutorRouter,
};

fn scheduler_row(task_ref: &str) -> SchedulerRow {
    SchedulerRow {
        task_ref: task_ref.to_owned(),
        contract_epoch: 1,
        state: SchedulerState::Runnable.as_str().to_owned(),
        lease_owner: None,
        lease_epoch: 0,
        lease_expires: None,
        next_eligible: "2026-08-03T00:00:00Z".to_owned(),
        attempt_count: 0,
        cancel_requested: false,
    }
}

fn scheduler_work_key(task_ref: &str) -> SchedulerWorkKey {
    SchedulerWorkKey {
        task_ref: task_ref.to_owned(),
        contract_epoch: 1,
    }
}

fn object_id(sequence: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}")).unwrap()
}

fn progress_fact(
    iteration: i64,
    status: &str,
    action_fingerprint: &str,
    evidence_refs_json: &str,
) -> ProgressFactRow {
    ProgressFactRow {
        loop_object_id: object_id(950),
        iteration,
        status: status.to_owned(),
        action_fingerprint: action_fingerprint.to_owned(),
        evidence_refs_json: evidence_refs_json.to_owned(),
        recorded_at: WallTimestamp::parse("2026-08-07T00:00:00Z").unwrap(),
        fencing_epoch: 1,
    }
}

#[test]
fn loop_control_switches_after_a_repeated_daemon_signature() {
    let facts = vec![
        progress_fact(1, "none", "sha256:action", "[\"artifact://sha256/a\"]"),
        progress_fact(2, "none", "sha256:action", "[\"artifact://sha256/a\"]"),
    ];

    let decision = super::derive_loop_control_from_facts(&facts, "sha256:action", 3, 5).unwrap();
    assert!(matches!(
        decision,
        super::LoopControlDecision::Switch { .. }
    ));
}

#[test]
fn loop_control_blocks_at_the_retry_or_stagnation_ceiling() {
    let repeated_facts = vec![
        progress_fact(1, "none", "sha256:action", "[]"),
        progress_fact(2, "none", "sha256:action", "[]"),
        progress_fact(3, "none", "sha256:action", "[]"),
    ];
    let retry_decision =
        super::derive_loop_control_from_facts(&repeated_facts, "sha256:action", 2, 5).unwrap();
    assert!(matches!(
        retry_decision,
        super::LoopControlDecision::Block {
            reason_code: "repeat_retry_ceiling_reached"
        }
    ));

    let stagnation_decision =
        super::derive_loop_control_from_facts(&repeated_facts, "sha256:action", 5, 3).unwrap();
    assert!(matches!(
        stagnation_decision,
        super::LoopControlDecision::Block {
            reason_code: "no_progress_ceiling_reached"
        }
    ));
}

#[test]
fn loop_control_rejects_malformed_durable_facts_and_resets_on_new_evidence() {
    let malformed_evidence = vec![progress_fact(1, "none", "sha256:action", "{}")];
    assert!(matches!(
        super::derive_loop_control_from_facts(&malformed_evidence, "sha256:action", 1, 3),
        Err(super::SchedulerAuthorityError::LoopControlUnavailable(_))
    ));

    let malformed_status = vec![progress_fact(1, "model_says_done", "sha256:action", "[]")];
    assert!(matches!(
        super::derive_loop_control_from_facts(&malformed_status, "sha256:action", 1, 3),
        Err(super::SchedulerAuthorityError::LoopControlUnavailable(_))
    ));

    let changed_evidence = vec![
        progress_fact(1, "none", "sha256:action", "[\"artifact://sha256/a\"]"),
        progress_fact(2, "none", "sha256:action", "[\"artifact://sha256/b\"]"),
    ];
    assert_eq!(
        super::derive_loop_control_from_facts(&changed_evidence, "sha256:action", 3, 5).unwrap(),
        super::LoopControlDecision::Continue
    );
}

fn context_governance() -> GovernanceSeed {
    GovernanceSeed {
        owner: strong_reference_to(&object_id(910), &format!("sha256:{}", "a".repeat(64))),
        authority: strong_reference_to(&object_id(911), &format!("sha256:{}", "b".repeat(64))),
        resource_scope: strong_reference_to(&object_id(912), &format!("sha256:{}", "c".repeat(64))),
        tenant_id: Some("tenant-a".to_owned()),
        created_by: "principal://tenant-a/daemon".to_owned(),
        sensitivity: GovernedObjectHeaderSensitivity::Internal,
        purpose_constraints: vec!["task_execution".to_owned()],
        retention_policy: "standard".to_owned(),
    }
}

fn seal_payload(payload: serde_json::Value) -> (String, String) {
    let (sealed_payload, digest) = seal_governed_object_content_digest(payload).unwrap();
    (serde_json::to_string(&sealed_payload).unwrap(), digest)
}

fn append_context_race_fixture(
    store: &SqliteAuthorityStore,
    task_ref: &str,
    required_context_ref: Option<&str>,
) -> (ContextResolutionCommand, ContextRevocationFactRow) {
    append_context_race_fixture_with_budget(store, task_ref, required_context_ref, json!({}))
}

fn append_context_race_fixture_with_budget(
    store: &SqliteAuthorityStore,
    task_ref: &str,
    required_context_ref: Option<&str>,
    context_budget: serde_json::Value,
) -> (ContextResolutionCommand, ContextRevocationFactRow) {
    let governance = context_governance();
    let issued_at = WallTimestamp::parse("2026-08-07T00:00:00Z").unwrap();
    let request_id = object_id(920);
    let request_header = compose_governed_header(
        &request_id,
        "ContextRequest",
        "cognitiveos.context-request/0.1",
        &governance,
        Vec::new(),
        Vec::new(),
        "p2-t04-race-test-request",
        &issued_at,
    )
    .unwrap();
    let (request_json, request_digest) = seal_payload(json!({
        "header": request_header,
        "purpose": "task_execution",
        "perspective": {
            "principal": "principal://tenant-a/daemon",
            "task": task_ref,
            "episode": "episode://tenant-a/p2-t04-race",
        },
        "budget": context_budget,
        "priority": ["task", "working"],
        "required": required_context_ref.map(|object_ref| vec![json!({"ref": object_ref})]).unwrap_or_default(),
        "forbidden": [],
        "freshness": {"world_max_age_ms": 0},
        "sensitivity": {"max_input": "internal", "egress": "none"},
        "target_profile": {"kind": "structured", "schema": "p2-t04-race/v1"},
        "allow_partial": false,
    }));
    let request = ContextRequestRow {
        request_id: request_id.clone(),
        task_ref: task_ref.to_owned(),
        request_digest: request_digest.clone(),
        canonical_json: request_json,
    };
    store.append_context_request(&request).unwrap();

    let source_id = object_id(921);
    let source_header = compose_governed_header(
        &source_id,
        "WorkspaceContextSource",
        "cognitiveos.workspace-context-source/0.1",
        &governance,
        Vec::new(),
        Vec::new(),
        "p2-t04-race-test-source",
        &issued_at,
    )
    .unwrap();
    let (source_json, source_digest) = seal_payload(json!({
        "header": source_header,
        "tenant_id": "tenant-a",
        "owner_ref": "principal://tenant-a/daemon",
        "resource_scope": "workspace://tenant-a/project/alpha",
        "conversation_ref": "conversation://tenant-a/one",
        "role": "working",
        "trust_level": "verified",
        "representation": "text",
        "provenance_ref": "admission://tenant-a/daemon/race-test",
        "content_bytes": 20,
        "content_tokens": 5,
        "body": {"text": "must-not-reach-pi"},
    }));
    store
        .append_workspace_context_source(&WorkspaceContextSourceRow {
            source_id: source_id.clone(),
            source_digest,
            governance: ObjectGovernance {
                object_ref: source_id.to_string(),
                tenant_id: Some("tenant-a".to_owned()),
                owner_ref: "principal://tenant-a/daemon".to_owned(),
                resource_scope: "workspace://tenant-a/project/alpha".to_owned(),
                conversation_ref: Some("conversation://tenant-a/one".to_owned()),
            },
            role: LoadedContextItemRole::Working,
            trust_level: LoadedContextItemTrustLevel::Verified,
            representation: LoadedContextItemRepresentation::Text,
            provenance_ref: "admission://tenant-a/daemon/race-test".to_owned(),
            content_bytes: 20,
            content_tokens: Some(5),
            canonical_json: source_json,
        })
        .unwrap();

    let principal = PrincipalFacts {
        principal_ref: UriRef::parse("principal://tenant-a/daemon").unwrap(),
        authenticated: true,
        active: true,
        tenant_id: Some("tenant-a".to_owned()),
    };
    let facts_id = object_id(922);
    let capability = CapabilityConstraints {
        subject: principal.principal_ref.to_string(),
        audience: "daemon://tenant-a/context".to_owned(),
        resource: "workspace://tenant-a/project".to_owned(),
        purpose: "task_execution".to_owned(),
        actions: ["read_body".to_owned()].into(),
        parameter_bounds: BTreeMap::new(),
        lease: LeaseWindow {
            not_before: WallTimestamp::parse("2026-08-06T00:00:00Z").unwrap(),
            expires: WallTimestamp::parse("2026-08-08T00:00:00Z").unwrap(),
        },
        depth_remaining: 1,
        issued_epoch: 1,
    };
    let actor_chain = ActorChainFacts {
        chain_digest: format!("sha256:{}", "d".repeat(64)),
        resolved: true,
    };
    let membership = Some(MembershipFacts {
        valid: true,
        roles: ["owner".to_owned()].into(),
    });
    let facts_header = compose_governed_header(
        &facts_id,
        "ContextAuthorizationFacts",
        "cognitiveos.context-authorization-facts/0.1",
        &governance,
        Vec::new(),
        Vec::new(),
        "p2-t04-race-test-facts",
        &issued_at,
    )
    .unwrap();
    let (facts_json, _) = seal_payload(json!({
        "header": facts_header,
        "fact_set_id": facts_id,
        "subject_ref": principal.principal_ref,
        "tenant_id": "tenant-a",
        "principal": principal,
        "actor_chain": actor_chain,
        "membership": membership,
        "capability_links": [capability],
        "explicit_denies": [],
        "capability_set_version": 1,
        "issued_revocation_epoch": 1,
    }));
    store
        .append_context_authorization_facts(&ContextAuthorizationFactsRow {
            fact_set_id: facts_id,
            subject_ref: "principal://tenant-a/daemon".to_owned(),
            tenant_id: "tenant-a".to_owned(),
            principal,
            actor_chain,
            membership,
            capability_links: vec![capability],
            explicit_denies: Vec::new(),
            capability_set_version: 1,
            issued_revocation_epoch: 1,
            canonical_json: facts_json,
        })
        .unwrap();

    let initial_revocation = context_revocation_fact(&governance, object_id(923), 1, &issued_at);
    store
        .append_context_revocation_fact(&initial_revocation)
        .unwrap();
    let later_revocation = context_revocation_fact(&governance, object_id(924), 2, &issued_at);

    let contract_id = object_id(925);
    let contract_header = compose_governed_header(
        &contract_id,
        "TaskContract",
        "cognitiveos.task-contract/0.4",
        &governance,
        Vec::new(),
        Vec::new(),
        "p2-t04-race-test-contract",
        &issued_at,
    )
    .unwrap();
    let contract = TaskContract {
        allowed_state_domains: vec!["task".to_owned(), "effect".to_owned()],
        allowed_tools: Vec::new(),
        budget: Budget {
            attention_slots: None,
            context_bytes: None,
            egress_bytes: None,
            input_tokens: None,
            money_microunits: None,
            output_tokens: None,
            semantic_calls: None,
            tool_calls: Some(1),
            wall_time_ms: None,
        },
        budget_id: Some(
            BudgetId::parse("00000000-0000-7000-b000-000000000926")
                .unwrap()
                .to_generated(),
        ),
        conditions: vec![ContractCondition {
            description: "test acceptance".to_owned(),
            id: "accept".to_owned(),
            kind: ContractConditionKind::Acceptance,
            machine_expression: None,
            verifier_ref: None,
        }],
        context_request_ref: Some(strong_reference_to(&request_id, &request_digest)),
        contract_epoch: 1,
        deadline: None,
        header: contract_header,
        human_gates: None,
        intent_acceptance_ref: strong_reference_to(
            &object_id(927),
            &format!("sha256:{}", "e".repeat(64)),
        ),
        intent_interpretation_ref: strong_reference_to(
            &object_id(928),
            &format!("sha256:{}", "f".repeat(64)),
        ),
        loop_object_id: Some(object_id(929).to_generated()),
        max_iterations: 1,
        max_retries: 0,
        objective: "race regression".to_owned(),
        scope: TaskScope {
            in_scope: vec!["test".to_owned()],
            out_of_scope: Vec::new(),
        },
        task_ref: task_ref.to_owned(),
        user_intent_ref: strong_reference_to(
            &object_id(930),
            &format!("sha256:{}", "1".repeat(64)),
        ),
        worker_authorization_root_id: Some(contract_id.to_generated()),
    };
    let (contract_json, contract_digest) = seal_payload(serde_json::to_value(contract).unwrap());
    store
        .insert_task_contract(
            &TaskContractRow {
                contract_id: contract_id.clone(),
                task_ref: task_ref.to_owned(),
                contract_epoch: 1,
                user_intent_record_id: object_id(930),
                interpretation_id: object_id(928),
                accepted_by: "principal://tenant-a/daemon".to_owned(),
                contract_digest,
                canonical_json: contract_json,
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000925").unwrap(),
                object_id: contract_id,
                domain: LifecycleDomain::Task,
                object_version: Version::INITIAL,
                event_type: "task-contract.minted".to_owned(),
                canonical_json: "{\"event\":\"p2-t04-race\"}".to_owned(),
            },
            0,
        )
        .unwrap();

    (
        ContextResolutionCommand {
            task_ref: task_ref.to_owned(),
            request_id,
            authorization_subject_ref: "principal://tenant-a/daemon".to_owned(),
            tenant_id: "tenant-a".to_owned(),
            resource_scope_prefix: "workspace://tenant-a/project".to_owned(),
            conversation_ref: Some("conversation://tenant-a/one".to_owned()),
            source_limit: 1,
            decided_at: issued_at,
        },
        later_revocation,
    )
}

fn context_revocation_fact(
    governance: &GovernanceSeed,
    fact_id: ObjectId,
    epoch: i64,
    issued_at: &WallTimestamp,
) -> ContextRevocationFactRow {
    let header = compose_governed_header(
        &fact_id,
        "ContextRevocationFact",
        "cognitiveos.context-revocation-fact/0.1",
        governance,
        Vec::new(),
        Vec::new(),
        "p2-t04-race-test-revocation",
        issued_at,
    )
    .unwrap();
    let (canonical_json, _) = seal_payload(
        json!({"header": header, "revocation_fact_id": fact_id, "tenant_id": "tenant-a", "revocation_epoch": epoch, "revoked_subject_ref": null, "revoked_capability_ref": null}),
    );
    ContextRevocationFactRow {
        revocation_fact_id: fact_id,
        tenant_id: "tenant-a".to_owned(),
        revocation_epoch: epoch,
        revoked_subject_ref: None,
        revoked_capability_ref: None,
        canonical_json,
    }
}

#[derive(Default)]
struct CountingPiProposer {
    calls: Cell<usize>,
}

impl super::PrivatePiCandidateProposer for CountingPiProposer {
    fn propose_candidate(
        &self,
        _resolved_context: &super::ResolvedContextView,
        _task_ref: &str,
        _contract_epoch: i64,
    ) -> Result<UntrustedPiCandidate, String> {
        self.calls.set(self.calls.get() + 1);
        Err("Pi must not receive revoked Context".to_owned())
    }
}

#[test]
fn task_context_builder_requires_daemon_system_and_task_fragments() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    prepare_personal_databases(&layout).unwrap();
    let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
    let task_ref = "task://tenant-a/p3-t02-required-fragments";
    let (context_command, _) = append_context_race_fixture(&store, task_ref, None);

    let resolved_context =
        super::resolve_authorized_task_context(&store, &context_command).unwrap();

    let system_fragment_ref = object_id(920).to_string();
    let task_fragment_ref = object_id(925).to_string();
    let working_fragment_ref = object_id(921).to_string();
    assert!(resolved_context.complete);
    assert!(resolved_context.loaded.iter().any(|item| {
        item.object_ref == system_fragment_ref
            && item.role == LoadedContextItemRole::Control
            && item.body["fragment"] == "system"
            && item.body["authority"] == "daemon_observational_only"
    }));
    assert!(resolved_context.loaded.iter().any(|item| {
        item.object_ref == task_fragment_ref
            && item.role == LoadedContextItemRole::AuthoritativeState
            && item.body["fragment"] == "task"
            && item.body["task_ref"] == task_ref
    }));
    assert!(resolved_context.loaded.iter().any(|item| {
        item.object_ref == working_fragment_ref && item.role == LoadedContextItemRole::Working
    }));

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn governed_context_cache_revalidates_before_reporting_a_reusable_prefix() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    prepare_personal_databases(&layout).unwrap();
    let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
    let task_ref = "task://tenant-a/p3-t04-governed-cache";
    let (context_command, _) = append_context_race_fixture(&store, task_ref, None);
    let mut context_cache = GovernedContextCache::default();

    let first_resolution = super::resolve_authorized_task_context_with_cache(
        &store,
        &context_command,
        &mut context_cache,
    )
    .unwrap();
    let second_resolution = super::resolve_authorized_task_context_with_cache(
        &store,
        &context_command,
        &mut context_cache,
    )
    .unwrap();

    assert!(!first_resolution.cache_telemetry.cache_hit);
    assert!(second_resolution.cache_telemetry.cache_hit);
    assert_eq!(
        first_resolution.resolved_view.render.digest,
        second_resolution.resolved_view.render.digest
    );
    assert!(
        second_resolution
            .cache_telemetry
            .stable_prefix_segment_count
            > 0,
        "the renderer header is always an authority-bound stable prefix"
    );
    assert_eq!(context_cache.len(), 1);

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn governed_context_cache_rejects_revoked_sources_instead_of_reusing_metadata() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    prepare_personal_databases(&layout).unwrap();
    let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
    let task_ref = "task://tenant-a/p3-t04-cache-revocation";
    let (context_command, later_revocation) = append_context_race_fixture(&store, task_ref, None);
    let mut context_cache = GovernedContextCache::default();

    super::resolve_authorized_task_context_with_cache(&store, &context_command, &mut context_cache)
        .unwrap();
    store
        .append_context_revocation_fact(&later_revocation)
        .unwrap();

    let result = super::resolve_authorized_task_context_with_cache(
        &store,
        &context_command,
        &mut context_cache,
    );

    assert!(matches!(
        result,
        Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(detail))
            if detail.contains("denied before body materialization")
    ));
    assert_eq!(
        context_cache.len(),
        1,
        "a rejected request cannot add a cache entry"
    );

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn required_task_context_fragments_fail_closed_when_the_request_budget_cannot_fit() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    prepare_personal_databases(&layout).unwrap();
    let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
    let task_ref = "task://tenant-a/p3-t02-required-fragment-budget";
    let (context_command, _) = append_context_race_fixture_with_budget(
        &store,
        task_ref,
        None,
        json!({"context_bytes": 1, "input_tokens": 1}),
    );

    let error = super::resolve_authorized_task_context(&store, &context_command).err();

    assert!(matches!(
        error,
        Some(SchedulerAuthorityError::ContextResolution(detail))
            if detail.contains("CONTEXT_BUDGET_EXCEEDED")
    ));

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn stale_workspace_source_is_excluded_before_body_loading_with_explicit_loss() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    prepare_personal_databases(&layout).unwrap();
    let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
    let task_ref = "task://tenant-a/p3-t02-stale-source";
    let (mut context_command, _) = append_context_race_fixture(&store, task_ref, None);
    context_command.decided_at = WallTimestamp::parse("2026-08-07T00:00:00.001Z").unwrap();

    let resolved_context =
        super::resolve_authorized_task_context(&store, &context_command).unwrap();
    let stale_source_ref = object_id(921).to_string();

    assert!(
        !resolved_context
            .loaded
            .iter()
            .any(|item| item.object_ref == stale_source_ref),
        "a source older than world_max_age_ms must not reach rendering"
    );
    assert!(resolved_context.rejected.iter().any(|rejected| {
        rejected.candidate_ref == stale_source_ref && rejected.reason == "CONTEXT_SOURCE_STALE"
    }));
    assert!(resolved_context.loss_declaration.iter().any(|loss| {
        loss.source == stale_source_ref
            && loss.transform == "omitted_stale_source"
            && loss.omitted_classes == ["working"]
            && loss
                .verification
                .as_deref()
                .is_some_and(|digest| digest.starts_with("sha256:"))
    }));

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn revocation_after_metadata_discovery_blocks_body_ranking_and_private_pi() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    prepare_personal_databases(&layout).unwrap();
    let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
    let task_ref = "task://tenant-a/p2-t04-revocation-race";
    let (context_command, later_revocation) = append_context_race_fixture(&store, task_ref, None);
    let proposer = CountingPiProposer::default();
    let candidate_id = object_id(931);
    let admission_command = super::DaemonCandidateAdmissionCommand {
        candidate_id: candidate_id.clone(),
        authorization_subject_ref: "principal://tenant-a/daemon".to_owned(),
        authorization_purpose: "task_execution".to_owned(),
        budget_charge: BudgetCharge::new(BTreeMap::from([("tool_calls".to_owned(), 1)])).unwrap(),
        governance: context_governance(),
        actor_ref: UriRef::parse("principal://tenant-a/daemon").unwrap(),
        authority_ref: UriRef::parse("authority://tenant-a/daemon").unwrap(),
        correlation_id: UriRef::parse("correlation://tenant-a/p2-t04-race").unwrap(),
    };

    let result = propose_persist_and_admit_candidate_after_metadata(
        &store,
        &super::FixedSchedulerClock::parse("2026-08-07T00:00:00Z").unwrap(),
        &UuidV7Generator,
        &context_command,
        &proposer,
        &admission_command,
        || {
            store
                .append_context_revocation_fact(&later_revocation)
                .map_err(|error| {
                    SchedulerAuthorityError::ContextAuthorizationUnavailable(error.to_string())
                })
        },
    );

    assert!(matches!(
        result,
        Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(detail))
            if detail.contains("denied before body materialization")
    ));
    assert_eq!(proposer.calls.get(), 0, "revoked Context must not reach Pi");
    assert_eq!(
        store
            .load_current_context_revocation_epoch("tenant-a")
            .unwrap(),
        Some(2)
    );
    assert!(
        store
            .load_operation_candidate_proposal(&candidate_id)
            .unwrap()
            .is_none(),
        "a rejected Context must not persist a candidate"
    );

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn missing_required_context_blocks_private_pi_and_candidate_admission() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    prepare_personal_databases(&layout).unwrap();
    let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
    let task_ref = "task://tenant-a/p2-t04-required-context";
    let (context_command, _) = append_context_race_fixture(
        &store,
        task_ref,
        Some("workspace://tenant-a/project/required-but-missing"),
    );
    let proposer = CountingPiProposer::default();
    let candidate_id = object_id(932);
    let admission_command = super::DaemonCandidateAdmissionCommand {
        candidate_id: candidate_id.clone(),
        authorization_subject_ref: "principal://tenant-a/daemon".to_owned(),
        authorization_purpose: "task_execution".to_owned(),
        budget_charge: BudgetCharge::new(BTreeMap::from([("tool_calls".to_owned(), 1)])).unwrap(),
        governance: context_governance(),
        actor_ref: UriRef::parse("principal://tenant-a/daemon").unwrap(),
        authority_ref: UriRef::parse("authority://tenant-a/daemon").unwrap(),
        correlation_id: UriRef::parse("correlation://tenant-a/p2-t04-required").unwrap(),
    };

    let result = super::propose_persist_and_admit_candidate(
        &store,
        &super::FixedSchedulerClock::parse("2026-08-07T00:00:00Z").unwrap(),
        &UuidV7Generator,
        &context_command,
        &proposer,
        &admission_command,
    );

    assert!(matches!(
        result,
        Err(SchedulerAuthorityError::ContextResolution(detail))
            if detail.contains("CONTEXT_INCOMPLETE")
    ));
    assert_eq!(
        proposer.calls.get(),
        0,
        "incomplete Context must not reach Pi"
    );
    assert!(
        store
            .load_operation_candidate_proposal(&candidate_id)
            .unwrap()
            .is_none()
    );

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn duplicate_candidate_retry_does_not_reinvoke_private_pi() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    prepare_personal_databases(&layout).unwrap();
    let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
    let task_ref = "task://tenant-a/p2-t04-duplicate-candidate";
    let (context_command, _) = append_context_race_fixture(&store, task_ref, None);
    let candidate_id = object_id(933);
    store
        .append_operation_candidate_proposal(&OperationCandidateProposalRow {
            candidate_id: candidate_id.clone(),
            task_ref: task_ref.to_owned(),
            contract_epoch: 1,
            candidate_source_ref: "observation://tenant-a/pi/previous-attempt".to_owned(),
            tool_ref: "operation://tenant-a/observe".to_owned(),
            action: "observe".to_owned(),
            target: "workspace://tenant-a/project/alpha".to_owned(),
            parameters_digest: format!("sha256:{}", "2".repeat(64)),
            expected_state_version: 1,
            operation_descriptor_ref: object_id(934),
            canonical_json: "{\"candidate\":\"previous-attempt\"}".to_owned(),
        })
        .unwrap();
    let proposer = CountingPiProposer::default();
    let admission_command = super::DaemonCandidateAdmissionCommand {
        candidate_id: candidate_id.clone(),
        authorization_subject_ref: "principal://tenant-a/daemon".to_owned(),
        authorization_purpose: "task_execution".to_owned(),
        budget_charge: BudgetCharge::new(BTreeMap::from([("tool_calls".to_owned(), 1)])).unwrap(),
        governance: context_governance(),
        actor_ref: UriRef::parse("principal://tenant-a/daemon").unwrap(),
        authority_ref: UriRef::parse("authority://tenant-a/daemon").unwrap(),
        correlation_id: UriRef::parse("correlation://tenant-a/p2-t04-duplicate").unwrap(),
    };

    let result = super::propose_persist_and_admit_candidate(
        &store,
        &super::FixedSchedulerClock::parse("2026-08-07T00:00:00Z").unwrap(),
        &UuidV7Generator,
        &context_command,
        &proposer,
        &admission_command,
    );

    assert!(
        result.is_err(),
        "the deliberately incomplete daemon-only admission fixture must not succeed"
    );
    assert_eq!(
        proposer.calls.get(),
        0,
        "a duplicate candidate identity must resume daemon admission without another Pi proposal"
    );
    assert_eq!(
        store
            .load_operation_candidate_proposal(&candidate_id)
            .unwrap()
            .unwrap()
            .canonical_json,
        "{\"candidate\":\"previous-attempt\"}"
    );

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn candidate_admission_rejects_policy_with_mismatched_durable_task_binding() {
    let context_request_id = object_id(901);
    let policy = SchedulerExecutionPolicyRow {
        task_ref: "task://personal/expected".to_owned(),
        contract_epoch: 1,
        context_request_id: context_request_id.clone(),
        canonical_json: json!({
            "schema_version": 1,
            "task_ref": "task://personal/substituted",
            "contract_epoch": 1,
            "context": {
                "request_id": context_request_id.as_str(),
                "authorization_subject_ref": "principal://personal/owner",
                "tenant_id": "personal",
                "resource_scope_prefix": "workspace://personal/",
                "conversation_ref": null,
                "source_limit": 1,
            },
            "admission": {
                "candidate_id": object_id(902).as_str(),
                "authorization_subject_ref": "principal://personal/owner",
                "authorization_purpose": "task_execution",
                "budget_charge": {"semantic_calls": 1},
                "governance": {
                    "owner": strong_reference_to(&object_id(903), &format!("sha256:{}", "a".repeat(64))),
                    "authority": strong_reference_to(&object_id(904), &format!("sha256:{}", "b".repeat(64))),
                    "resource_scope": strong_reference_to(&object_id(905), &format!("sha256:{}", "c".repeat(64))),
                    "tenant_id": null,
                    "created_by": "principal://personal/daemon",
                    "sensitivity": "internal",
                    "purpose_constraints": ["task_execution"],
                    "retention_policy": "standard",
                },
                "actor_ref": "principal://personal/daemon",
                "authority_ref": "authority://personal/daemon",
                "correlation_id": "correlation://personal/scheduler",
            },
        })
        .to_string(),
    };

    let error = candidate_admission_command_from_policy(&policy).unwrap_err();

    assert!(matches!(
        error,
        SchedulerAuthorityError::CandidateAdmissionComposition(detail)
            if detail.contains("durable binding")
    ));
}

fn sealed_worker_authorization_row() -> WorkerIterationAuthorizationRow {
    let authorization_id = object_id(810);
    let worker_authorization_root_id = object_id(811);
    let selected_candidate_id = object_id(812);
    let intent_id = object_id(813);
    let effect_object_id = object_id(814);
    let task_contract_id = object_id(815);
    let budget_id = BudgetId::parse("00000000-0000-7000-b000-000000000816").unwrap();
    let budget_charge = Budget {
        attention_slots: None,
        context_bytes: None,
        egress_bytes: None,
        input_tokens: None,
        money_microunits: None,
        output_tokens: None,
        semantic_calls: None,
        tool_calls: Some(1),
        wall_time_ms: None,
    };
    let governance = GovernanceSeed {
        owner: strong_reference_to(&object_id(817), &format!("sha256:{}", "a".repeat(64))),
        authority: strong_reference_to(
            &object_id(818),
            &format!("sha256:{}", "b".repeat(64)),
        ),
        resource_scope: strong_reference_to(
            &object_id(819),
            &format!("sha256:{}", "c".repeat(64)),
        ),
        tenant_id: Some("00000000-0000-7000-9000-000000000820".to_owned()),
        created_by: "principal://personal/daemon".to_owned(),
        sensitivity: cognitive_contracts::generated::governed_object_header::GovernedObjectHeaderSensitivity::Internal,
        purpose_constraints: vec!["task_execution".to_owned()],
        retention_policy: "standard".to_owned(),
    };
    let issued_at = WallTimestamp::parse("2026-08-04T12:00:00Z").unwrap();
    let header = compose_governed_header(
        &authorization_id,
        "WorkerIterationAuthorization",
        "cognitiveos.worker-iteration-authorization/0.1",
        &governance,
        Vec::new(),
        Vec::new(),
        "scheduler-authority-evidence-test",
        &issued_at,
    )
    .unwrap();
    let payload = WorkerIterationAuthorization {
        action_fingerprint: format!("sha256:{}", "d".repeat(64)),
        budget_charge: budget_charge.clone(),
        budget_id: budget_id.to_generated(),
        contract_epoch: 1,
        effect_ref: strong_reference_to(&effect_object_id, &format!("sha256:{}", "e".repeat(64))),
        expected_loop_version: 1,
        header,
        intent_ref: strong_reference_to(&intent_id, &format!("sha256:{}", "f".repeat(64))),
        issued_fencing_epoch: 1,
        iteration: 1,
        selected_candidate_ref: strong_reference_to(
            &selected_candidate_id,
            &format!("sha256:{}", "1".repeat(64)),
        ),
        task_contract_ref: strong_reference_to(
            &task_contract_id,
            &format!("sha256:{}", "2".repeat(64)),
        ),
        worker_authorization_root_id: worker_authorization_root_id.to_generated(),
    };
    let payload_value = serde_json::to_value(&payload).unwrap();
    let (sealed_payload, _) = seal_governed_object_content_digest(payload_value).unwrap();
    let budget_charge_canonical_json = String::from_utf8(
        canonical::canonical_bytes_of_value(&serde_json::to_value(budget_charge).unwrap()).unwrap(),
    )
    .unwrap();

    WorkerIterationAuthorizationRow {
        authorization_id,
        worker_authorization_root_id,
        task_ref: "task://personal/sealed-worker-authorization".to_owned(),
        contract_epoch: 1,
        loop_object_id: object_id(821),
        iteration: 1,
        expected_loop_version: Version::INITIAL,
        selected_candidate_id,
        intent_id,
        effect_object_id,
        budget_id,
        budget_charge_canonical_json,
        action_fingerprint: payload.action_fingerprint,
        issued_fencing_epoch: 1,
        canonical_json: serde_json::to_string(&sealed_payload).unwrap(),
    }
}

fn recovered_closed_attempt(task_ref: &str, lease_epoch: i64) -> RecoveredWorkerAttempt {
    RecoveredWorkerAttempt {
        handoff: WorkerAuthorizationHandoff {
            authorization: WorkerIterationAuthorizationRow {
                authorization_id: object_id(800),
                worker_authorization_root_id: object_id(801),
                task_ref: task_ref.to_owned(),
                contract_epoch: 1,
                loop_object_id: object_id(802),
                iteration: 1,
                expected_loop_version: Version::INITIAL,
                selected_candidate_id: object_id(803),
                intent_id: object_id(804),
                effect_object_id: object_id(805),
                budget_id: BudgetId::parse("00000000-0000-7000-b000-000000000806").unwrap(),
                budget_charge_canonical_json: "{\"tool_calls\":1}".to_owned(),
                action_fingerprint: "recovered-lease-release".to_owned(),
                issued_fencing_epoch: 1,
                canonical_json: "{\"worker_authorization\":1}".to_owned(),
            },
            worker_attempt_id: object_id(807),
            scheduler_lease: Some(SchedulerLeaseBinding {
                task_ref: task_ref.to_owned(),
                contract_epoch: 1,
                lease_owner: "scheduler-worker".to_owned(),
                lease_epoch,
            }),
        },
        effect_closure: SchedulerEffectClosure::Closed,
    }
}

fn temporary_scheduler_database_path() -> std::path::PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "cognitiveos-scheduler-authority-{}-{unique_suffix}.db",
        std::process::id()
    ))
}

fn persist_repairable_task_contract(
    store: &SqliteAuthorityStore,
    base: u64,
    task_ref: &str,
) -> (TaskContractRow, ObjectId, BudgetId) {
    let issued_at = WallTimestamp::parse("2026-08-13T05:00:00Z").unwrap();
    let contract_id = object_id(base);
    let loop_object_id = object_id(base + 1);
    let budget_id = BudgetId::parse(&format!("00000000-0000-7000-b000-{base:012x}")).unwrap();
    let header = compose_governed_header(
        &contract_id,
        "TaskContract",
        "cognitiveos.task-contract/0.3",
        &context_governance(),
        Vec::new(),
        Vec::new(),
        "p2-t12-startup-repair",
        &issued_at,
    )
    .unwrap();
    let contract = TaskContract {
        allowed_state_domains: vec!["task".to_owned(), "effect".to_owned()],
        allowed_tools: vec!["operation://personal/filesystem/read".to_owned()],
        budget: Budget {
            attention_slots: None,
            context_bytes: None,
            egress_bytes: None,
            input_tokens: None,
            money_microunits: None,
            output_tokens: None,
            semantic_calls: Some(1),
            tool_calls: Some(2),
            wall_time_ms: None,
        },
        budget_id: Some(budget_id.to_generated()),
        conditions: vec![ContractCondition {
            description: "independently verified read".to_owned(),
            id: "accept-read".to_owned(),
            kind: ContractConditionKind::Acceptance,
            machine_expression: None,
            verifier_ref: Some("verifier://personal/fixed-effect".to_owned()),
        }],
        context_request_ref: None,
        contract_epoch: 1,
        deadline: Some("2027-12-31T00:00:00Z".to_owned()),
        header,
        human_gates: None,
        intent_acceptance_ref: strong_reference_to(
            &object_id(base + 2),
            &format!("sha256:{}", "a".repeat(64)),
        ),
        intent_interpretation_ref: strong_reference_to(
            &object_id(base + 3),
            &format!("sha256:{}", "b".repeat(64)),
        ),
        loop_object_id: Some(loop_object_id.to_generated()),
        max_iterations: 2,
        max_retries: 1,
        objective: "read one governed workspace file".to_owned(),
        scope: TaskScope {
            in_scope: vec!["workspace read".to_owned()],
            out_of_scope: vec!["mutation".to_owned()],
        },
        task_ref: task_ref.to_owned(),
        user_intent_ref: strong_reference_to(
            &object_id(base + 4),
            &format!("sha256:{}", "c".repeat(64)),
        ),
        worker_authorization_root_id: Some(contract_id.to_generated()),
    };
    let (canonical_json, contract_digest) = seal_payload(serde_json::to_value(contract).unwrap());
    let row = TaskContractRow {
        contract_id: contract_id.clone(),
        task_ref: task_ref.to_owned(),
        contract_epoch: 1,
        user_intent_record_id: object_id(base + 4),
        interpretation_id: object_id(base + 3),
        accepted_by: "principal://personal/owner".to_owned(),
        contract_digest,
        canonical_json,
    };
    store
        .insert_task_contract(
            &row,
            &EventDraft {
                event_id: EventId::parse(&format!("00000000-0000-7000-a000-{base:012x}")).unwrap(),
                object_id: contract_id,
                domain: LifecycleDomain::Task,
                object_version: Version::INITIAL,
                event_type: "task-contract.minted".to_owned(),
                canonical_json: "{\"event\":\"p2-t12-repair-contract\"}".to_owned(),
            },
            0,
        )
        .unwrap();
    (row, loop_object_id, budget_id)
}

fn prepared_repair_bootstrap(
    store: &SqliteAuthorityStore,
    contract: &TaskContractRow,
) -> TaskExecutionBootstrap {
    prepare_task_execution_bootstrap(
        store,
        &SystemClock,
        &UuidV7Generator,
        &WriterLease {
            epoch: store.current_fencing_epoch().unwrap(),
        },
        contract,
        &UriRef::parse("correlation://personal/p2-t12-repair-test").unwrap(),
    )
    .unwrap()
}

fn temporary_personal_layout() -> PersonalDataLayout {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "cognitiveos-server-recovery-{}-{unique_suffix}",
        std::process::id()
    ));
    PersonalDataLayout::from_xdg_roots(
        root.join("config"),
        root.join("data"),
        root.join("state"),
        root.join("cache"),
        root.join("runtime"),
    )
}

fn endpoint_document_path(layout: &PersonalDataLayout) -> std::path::PathBuf {
    layout.state_dir().join("daemon-endpoint.json")
}

fn wait_for_published_endpoint(layout: &PersonalDataLayout) -> Option<String> {
    let endpoint_path = endpoint_document_path(layout);
    // Recovery tests perform SQLite replay before the server publishes the
    // endpoint. Windows CI can take longer than the original two-second
    // polling window under concurrent workspace test load.
    for _ in 0..300 {
        if let Ok(document) = std::fs::read_to_string(&endpoint_path) {
            let endpoint =
                serde_json::from_str::<serde_json::Value>(&document).unwrap()["endpoint"]
                    .as_str()
                    .unwrap()
                    .to_owned();
            return Some(endpoint);
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    None
}

fn send_health_request_to_once_server(endpoint: &str) {
    let mut stream = TcpStream::connect(endpoint).unwrap();
    stream
        .write_all(b"GET /personal/health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .unwrap();
}

fn state(value: &str) -> StateName {
    StateName::parse(value).unwrap()
}

fn recovery_effect_grant() -> cognitive_kernel::authz::AuthorizationGrant {
    let authorization_time = WallTimestamp::parse("2026-08-04T12:02:00Z").unwrap();
    authorize(
        &AuthzSnapshot {
            tenant_id: "personal-test".to_owned(),
            principal: PrincipalFacts {
                principal_ref: UriRef::parse("principal://personal/daemon").unwrap(),
                authenticated: true,
                active: true,
                tenant_id: Some("personal-test".to_owned()),
            },
            actor_chain: ActorChainFacts {
                chain_digest: format!("sha256:{}", "c".repeat(64)),
                resolved: true,
            },
            membership: Some(MembershipFacts {
                valid: true,
                roles: ["daemon".to_owned()].into(),
            }),
            capability_links: vec![CapabilityConstraints {
                subject: "principal://personal/daemon".to_owned(),
                audience: "authority://personal/effect-authority".to_owned(),
                resource: "scope://personal/restart-recovery".to_owned(),
                purpose: "task_execution".to_owned(),
                actions: ["filesystem.read".to_owned()].into(),
                parameter_bounds: BTreeMap::new(),
                lease: LeaseWindow {
                    not_before: WallTimestamp::parse("2026-08-04T12:00:00Z").unwrap(),
                    expires: WallTimestamp::parse("2026-08-04T12:05:00Z").unwrap(),
                },
                depth_remaining: 1,
                issued_epoch: 1,
            }],
            capability_set_version: 1,
            explicit_denies: Vec::new(),
            revocation_epoch: 1,
            decided_at: authorization_time,
        },
        &ObjectGovernance {
            object_ref: "effect://personal/restart-recovery".to_owned(),
            tenant_id: Some("personal-test".to_owned()),
            owner_ref: "principal://personal/daemon".to_owned(),
            resource_scope: "scope://personal/restart-recovery/effect".to_owned(),
            conversation_ref: None,
        },
        &AccessRequest {
            action: "filesystem.read".to_owned(),
            purpose: "task_execution".to_owned(),
        },
    )
    .unwrap()
}

fn reconcile_effect_for_restart_recovery(
    store: &SqliteAuthorityStore,
    effect_object_id: &ObjectId,
) {
    let clock = super::FixedSchedulerClock::parse("2026-08-04T12:02:00Z").unwrap();
    let identifiers = UuidV7Generator;
    let effect_protocol = EffectProtocol::new(
        store,
        &clock,
        &identifiers,
        UriRef::parse("actor://personal/daemon").unwrap(),
        UriRef::parse("authority://personal/effect-authority").unwrap(),
        UriRef::parse("correlation://personal/restart-recovery").unwrap(),
    );
    let grant = recovery_effect_grant();
    let currency = GovernanceCurrency {
        revocation_epoch: 1,
        capability_set_version: 1,
    };
    let writer_lease = WriterLease { epoch: 1 };
    let executor = ScriptedExecutor::queryable(1);

    let authorized = effect_protocol
        .authorize_effect(
            effect_object_id,
            Version::INITIAL,
            &grant,
            &currency,
            &writer_lease,
        )
        .unwrap();
    let (dispatched, outcome) = effect_protocol
        .dispatch_effect(
            effect_object_id,
            authorized.after_version,
            &grant,
            &currency,
            &executor,
            &writer_lease,
        )
        .unwrap();
    let executed = effect_protocol
        .record_outcome(
            effect_object_id,
            dispatched.after_version,
            &outcome,
            &writer_lease,
        )
        .unwrap();
    effect_protocol
        .reconcile(
            effect_object_id,
            "EXECUTED",
            executed.after_version,
            &executor,
            &writer_lease,
        )
        .unwrap();
}

/// Persist the minimum complete D05 handoff evidence through the normal
/// store APIs. The Effect intentionally remains PROPOSED: this fixture
/// exercises restart recovery's retain path without executing a tool or
/// manufacturing a terminal Effect transition.
fn persist_pending_bound_handoff(
    database_path: &std::path::Path,
    consume_authorization: bool,
) -> (ObjectId, SchedulerWorkKey) {
    let store = SqliteAuthorityStore::open(database_path).unwrap();
    let authorization = sealed_worker_authorization_row();
    let task_ref = authorization.task_ref.clone();
    let scheduler_work_key = SchedulerWorkKey {
        task_ref: task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
    };
    let admitted_at = WallTimestamp::parse("2026-08-04T12:00:00Z").unwrap();

    store
        .insert_task_contract(
            &TaskContractRow {
                contract_id: object_id(830),
                task_ref: task_ref.clone(),
                contract_epoch: authorization.contract_epoch,
                user_intent_record_id: object_id(831),
                interpretation_id: object_id(832),
                accepted_by: "principal://personal/daemon".to_owned(),
                contract_digest: format!("sha256:{}", "a".repeat(64)),
                canonical_json: "{\"task_contract\":\"recovery-fixture\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000830").unwrap(),
                object_id: object_id(830),
                domain: LifecycleDomain::Task,
                object_version: Version::INITIAL,
                event_type: "task-contract.minted".to_owned(),
                canonical_json: "{\"event\":\"task-contract\"}".to_owned(),
            },
            0,
        )
        .unwrap();
    store
        .append_operation_candidate_proposal(&OperationCandidateProposalRow {
            candidate_id: authorization.selected_candidate_id.clone(),
            task_ref: task_ref.clone(),
            contract_epoch: authorization.contract_epoch,
            candidate_source_ref: "observation://personal/restart-recovery".to_owned(),
            tool_ref: "operation://personal/filesystem/read".to_owned(),
            action: "filesystem.read".to_owned(),
            target: "file:///workspace/input.txt".to_owned(),
            parameters_digest: format!("sha256:{}", "b".repeat(64)),
            expected_state_version: Version::INITIAL.get(),
            operation_descriptor_ref: object_id(833),
            canonical_json: "{\"candidate\":\"recovery-fixture\"}".to_owned(),
        })
        .unwrap();
    store
        .admit_object(&ObjectAdmission {
            object: StoredObject {
                object_id: authorization.loop_object_id.clone(),
                domain: LifecycleDomain::Loop,
                state: state("DECIDE"),
                version: authorization.expected_loop_version,
                body: json!({"fixture": "restart-recovery"}),
            },
            admitted_at: admitted_at.clone(),
            event: EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000821").unwrap(),
                object_id: authorization.loop_object_id.clone(),
                domain: LifecycleDomain::Loop,
                object_version: authorization.expected_loop_version,
                event_type: "loop.fixture-admitted".to_owned(),
                canonical_json: "{\"event\":\"loop\"}".to_owned(),
            },
            outbox: Vec::new(),
            fencing_epoch: Some(authorization.issued_fencing_epoch),
        })
        .unwrap();
    let budget_state = BudgetState::new(BTreeMap::from([("tool_calls".to_owned(), 2)])).unwrap();
    let budget_state_json = serde_json::to_string(&budget_state).unwrap();
    store
        .create_budget(&authorization.budget_id, &budget_state_json, &admitted_at)
        .unwrap();

    let candidate_admission = CandidateAdmissionCommit {
        selected_candidate_id: authorization.selected_candidate_id.clone(),
        intent: IntentRow {
            intent_id: authorization.intent_id.clone(),
            idempotency_key: "restart-recovery-pending".to_owned(),
            parameters_digest: format!("sha256:{}", "b".repeat(64)),
            action: "filesystem.read".to_owned(),
            target: "file:///workspace/input.txt".to_owned(),
            effect_object_id: authorization.effect_object_id.clone(),
            expected_state_version: Version::INITIAL,
            grant_epoch: 1,
            capability_set_version: 1,
            task_binding: Some(TaskBinding {
                task_ref: task_ref.clone(),
                contract_epoch: authorization.contract_epoch,
            }),
            canonical_json: "{\"intent\":\"restart-recovery\"}".to_owned(),
        },
        intent_event: EventDraft {
            event_id: EventId::parse("00000000-0000-7000-a000-000000000813").unwrap(),
            object_id: authorization.intent_id.clone(),
            domain: LifecycleDomain::Effect,
            object_version: Version::INITIAL,
            event_type: "intent.minted".to_owned(),
            canonical_json: "{\"event\":\"intent\"}".to_owned(),
        },
        effect_admission: ObjectAdmission {
            object: StoredObject {
                object_id: authorization.effect_object_id.clone(),
                domain: LifecycleDomain::Effect,
                state: state("PROPOSED"),
                version: Version::INITIAL,
                body: json!({"effect": "restart-recovery"}),
            },
            admitted_at: admitted_at.clone(),
            event: EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000814").unwrap(),
                object_id: authorization.effect_object_id.clone(),
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "effect.admitted".to_owned(),
                canonical_json: "{\"event\":\"effect\"}".to_owned(),
            },
            outbox: Vec::new(),
            fencing_epoch: Some(authorization.issued_fencing_epoch),
        },
        worker_authorization: authorization.clone(),
        loop_transition: TransitionCommit {
            cas: ObjectCas {
                object_id: authorization.loop_object_id.clone(),
                domain: LifecycleDomain::Loop,
                from_state: state("DECIDE"),
                to_state: state("ACT"),
                expected_version: authorization.expected_loop_version,
                next_version: authorization.expected_loop_version.next().unwrap(),
                committed_at: admitted_at.clone(),
            },
            event: EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000822").unwrap(),
                object_id: authorization.loop_object_id.clone(),
                domain: LifecycleDomain::Loop,
                object_version: authorization.expected_loop_version.next().unwrap(),
                event_type: "loop.operation-admitted".to_owned(),
                canonical_json: "{\"event\":\"loop\"}".to_owned(),
            },
            record: RecordDraft {
                record_id: RecordId::parse("00000000-0000-7000-8000-000000000821").unwrap(),
                object_id: authorization.loop_object_id.clone(),
                domain: LifecycleDomain::Loop,
                object_version: authorization.expected_loop_version.next().unwrap(),
                canonical_json: "{\"record\":\"loop\"}".to_owned(),
            },
            budget: Some(BudgetCas {
                budget_id: authorization.budget_id.clone(),
                expected_version: Version::INITIAL,
                next_version: Version::INITIAL.next().unwrap(),
                charge_canonical_json: "{\"tool_calls\":1}".to_owned(),
                next_state_canonical_json: serde_json::to_string(
                    &BudgetState::new(BTreeMap::from([("tool_calls".to_owned(), 1)])).unwrap(),
                )
                .unwrap(),
            }),
            outbox: Vec::new(),
            fencing_epoch: Some(authorization.issued_fencing_epoch),
        },
        fencing_epoch: authorization.issued_fencing_epoch,
    };
    store
        .commit_candidate_admission(&candidate_admission)
        .unwrap();

    let mut scheduler_repository = SchedulerRepository::open(database_path).unwrap();
    scheduler_repository
        .upsert(&scheduler_row(&task_ref))
        .unwrap();
    if consume_authorization {
        let leased_row = scheduler_repository
            .acquire_lease(
                &scheduler_work_key,
                "restart-recovery-worker",
                41,
                "2026-08-04T12:05:00Z",
            )
            .unwrap();
        let dispatch = SchedulerDispatch {
            task_ref,
            contract_epoch: authorization.contract_epoch,
            lease_owner: leased_row.lease_owner.unwrap(),
            lease_epoch: leased_row.lease_epoch,
            lease_expires: leased_row.lease_expires.unwrap(),
            attempt_count: leased_row.attempt_count,
        };
        super::consume_worker_authorization_for_attempt(
            &store,
            &super::FixedSchedulerClock::parse("2026-08-04T12:01:00Z").unwrap(),
            &authorization.authorization_id,
            object_id(834),
            &dispatch,
        )
        .unwrap();
    }
    drop(scheduler_repository);
    drop(store);
    (authorization.effect_object_id, scheduler_work_key)
}

fn committed_ceiling_stop() -> CommittedTransition {
    CommittedTransition {
        record_id: RecordId::parse("00000000-0000-7000-8000-000000000001").unwrap(),
        event_id: EventId::parse("00000000-0000-7000-8000-000000000002").unwrap(),
        event_sequence: 1,
        after_version: Version::new(2).unwrap(),
        committed_at: WallTimestamp::parse("2026-08-02T00:00:00Z").unwrap(),
    }
}

fn task_binding() -> TaskBinding {
    TaskBinding {
        task_ref: "task://personal/durable-effect-resolution".to_owned(),
        contract_epoch: 4,
    }
}

fn effect_intent(intent_suffix: u64, binding: Option<TaskBinding>) -> IntentRow {
    IntentRow {
        intent_id: ObjectId::parse(&format!("00000000-0000-7000-8000-{intent_suffix:012x}"))
            .unwrap(),
        idempotency_key: format!("scheduler-effect-{intent_suffix}"),
        parameters_digest: format!("sha256:{}", "ab".repeat(32)),
        action: "scheduler.effect".to_owned(),
        target: "effect://personal/scheduler".to_owned(),
        effect_object_id: ObjectId::parse(&format!("00000000-0000-7000-9000-{intent_suffix:012x}"))
            .unwrap(),
        expected_state_version: Version::INITIAL,
        grant_epoch: 1,
        capability_set_version: 1,
        task_binding: binding,
        canonical_json: "{}".to_owned(),
    }
}

#[test]
fn restarted_recovery_retains_a_pending_effects_exact_bound_lease() {
    let database_path = temporary_scheduler_database_path();
    let (effect_object_id, scheduler_work_key) =
        persist_pending_bound_handoff(&database_path, true);

    let reopened_store = SqliteAuthorityStore::open(&database_path).unwrap();
    let mut reopened_scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    let recovered_attempts = super::reconcile_recovered_worker_attempts(
        &reopened_store,
        &mut reopened_scheduler_repository,
        &super::FixedSchedulerClock::parse("2026-08-04T12:02:00Z").unwrap(),
    )
    .unwrap();

    assert_eq!(recovered_attempts.len(), 1);
    assert_eq!(
        recovered_attempts[0].effect_closure,
        SchedulerEffectClosure::PendingReconciliation
    );
    assert_eq!(
        recovered_attempts[0]
            .handoff
            .scheduler_lease
            .as_ref()
            .unwrap()
            .lease_epoch,
        41
    );
    assert_eq!(
        reopened_store
            .load_object(LifecycleDomain::Effect, &effect_object_id)
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "PROPOSED"
    );

    let scheduler_row = reopened_scheduler_repository
        .load(&scheduler_work_key)
        .unwrap()
        .unwrap();
    assert_eq!(scheduler_row.state, SchedulerState::Leased.as_str());
    assert_eq!(
        scheduler_row.lease_owner.as_deref(),
        Some("restart-recovery-worker")
    );
    assert_eq!(scheduler_row.lease_epoch, 41);
    assert_eq!(scheduler_row.attempt_count, 1);

    drop(reopened_scheduler_repository);
    drop(reopened_store);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn restarted_recovery_releases_only_a_reconciled_effects_exact_bound_lease() {
    let database_path = temporary_scheduler_database_path();
    let (effect_object_id, scheduler_work_key) =
        persist_pending_bound_handoff(&database_path, true);

    let closing_store = SqliteAuthorityStore::open(&database_path).unwrap();
    reconcile_effect_for_restart_recovery(&closing_store, &effect_object_id);
    let reconciled_effect = closing_store
        .load_object(LifecycleDomain::Effect, &effect_object_id)
        .unwrap()
        .unwrap();
    assert_eq!(reconciled_effect.state.as_str(), "RECONCILED");
    assert_eq!(reconciled_effect.version, Version::new(5).unwrap());
    drop(closing_store);

    let reopened_store = SqliteAuthorityStore::open(&database_path).unwrap();
    let mut reopened_scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    let recovered_attempts = super::reconcile_recovered_worker_attempts(
        &reopened_store,
        &mut reopened_scheduler_repository,
        &super::FixedSchedulerClock::parse("2026-08-04T12:03:00Z").unwrap(),
    )
    .unwrap();

    assert_eq!(recovered_attempts.len(), 1);
    assert_eq!(
        recovered_attempts[0].effect_closure,
        SchedulerEffectClosure::Closed
    );
    let recovered_lease = recovered_attempts[0]
        .handoff
        .scheduler_lease
        .as_ref()
        .unwrap();
    assert_eq!(recovered_lease.lease_owner, "restart-recovery-worker");
    assert_eq!(recovered_lease.lease_epoch, 41);

    let scheduler_row = reopened_scheduler_repository
        .load(&scheduler_work_key)
        .unwrap()
        .unwrap();
    assert_eq!(scheduler_row.state, SchedulerState::Succeeded.as_str());
    assert_eq!(scheduler_row.lease_owner, None);
    assert_eq!(scheduler_row.lease_expires, None);
    assert_eq!(scheduler_row.lease_epoch, 41);
    assert_eq!(scheduler_row.attempt_count, 1);

    drop(reopened_scheduler_repository);
    drop(reopened_store);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn runtime_spine_daemon_close_recoverable_without_duplicate_dispatch() {
    // B05 observation floor: daemon restart recovers a reconciled Effect and
    // releases the exact lease without minting a second attempt/dispatch.
    let database_path = temporary_scheduler_database_path();
    let (effect_object_id, scheduler_work_key) =
        persist_pending_bound_handoff(&database_path, true);

    let closing_store = SqliteAuthorityStore::open(&database_path).unwrap();
    reconcile_effect_for_restart_recovery(&closing_store, &effect_object_id);
    drop(closing_store);

    let reopened_store = SqliteAuthorityStore::open(&database_path).unwrap();
    let mut reopened_scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    let recovered_attempts = super::reconcile_recovered_worker_attempts(
        &reopened_store,
        &mut reopened_scheduler_repository,
        &super::FixedSchedulerClock::parse("2026-08-04T12:03:00Z").unwrap(),
    )
    .unwrap();

    assert_eq!(recovered_attempts.len(), 1);
    assert_eq!(
        recovered_attempts[0].effect_closure,
        SchedulerEffectClosure::Closed
    );
    let recovered_lease = recovered_attempts[0]
        .handoff
        .scheduler_lease
        .as_ref()
        .unwrap();
    assert_eq!(recovered_lease.lease_epoch, 41);
    let scheduler_row = reopened_scheduler_repository
        .load(&scheduler_work_key)
        .unwrap()
        .unwrap();
    assert_eq!(scheduler_row.state, SchedulerState::Succeeded.as_str());
    assert_eq!(scheduler_row.attempt_count, 1);
    assert_eq!(scheduler_row.lease_owner, None);
    assert_eq!(scheduler_row.lease_epoch, 41);

    drop(reopened_scheduler_repository);
    drop(reopened_store);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn server_startup_recovers_closed_effect_before_publishing_endpoint() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let authority_database_path = layout.authority_database_path();
    let (effect_object_id, scheduler_work_key) =
        persist_pending_bound_handoff(&authority_database_path, true);
    let closing_store = SqliteAuthorityStore::open(&authority_database_path).unwrap();
    reconcile_effect_for_restart_recovery(&closing_store, &effect_object_id);
    drop(closing_store);

    let (result_sender, result_receiver) = mpsc::channel();
    let server_layout = layout.clone();
    let server_thread = std::thread::spawn(move || {
        let result = super::super::server::serve_personal_loopback(
            super::super::server::PersonalDaemonConfig {
                bind_address: "127.0.0.1:0".to_owned(),
                layout: server_layout,
                bounds: super::super::bounds::PersonalResourceBounds::personal_v1_baseline(),
                once: true,
            },
        );
        result_sender.send(result).unwrap();
    });

    let endpoint = wait_for_published_endpoint(&layout);
    assert!(
        endpoint.is_some(),
        "server did not publish its endpoint document"
    );
    let endpoint = endpoint.unwrap();
    send_health_request_to_once_server(&endpoint);
    assert!(result_receiver.recv().unwrap().is_ok());
    server_thread.join().unwrap();

    let mut scheduler_repository = SchedulerRepository::open(&authority_database_path).unwrap();
    let scheduler_row = scheduler_repository
        .load(&scheduler_work_key)
        .unwrap()
        .unwrap();
    assert_eq!(scheduler_row.state, SchedulerState::Succeeded.as_str());
    assert_eq!(scheduler_row.lease_owner, None);
    assert_eq!(scheduler_row.lease_expires, None);
    assert_eq!(scheduler_row.lease_epoch, 41);
    assert_eq!(scheduler_row.attempt_count, 1);
    assert!(!endpoint_document_path(&layout).exists());

    drop(scheduler_repository);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn server_startup_recovery_stale_contract_does_not_publish_endpoint() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let authority_database_path = layout.authority_database_path();
    let (_, scheduler_work_key) = persist_pending_bound_handoff(&authority_database_path, true);
    let store = SqliteAuthorityStore::open(&authority_database_path).unwrap();
    store
        .insert_task_contract(
            &TaskContractRow {
                contract_id: object_id(840),
                task_ref: scheduler_work_key.task_ref,
                contract_epoch: 2,
                user_intent_record_id: object_id(841),
                interpretation_id: object_id(842),
                accepted_by: "principal://personal/daemon".to_owned(),
                contract_digest: format!("sha256:{}", "d".repeat(64)),
                canonical_json: "{\"task_contract\":\"superseding-fixture\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000840").unwrap(),
                object_id: object_id(840),
                domain: LifecycleDomain::Task,
                object_version: Version::INITIAL,
                event_type: "task-contract.superseded".to_owned(),
                canonical_json: "{\"event\":\"task-contract\"}".to_owned(),
            },
            1,
        )
        .unwrap();
    drop(store);

    let result =
        super::super::server::serve_personal_loopback(super::super::server::PersonalDaemonConfig {
            bind_address: "127.0.0.1:0".to_owned(),
            layout: layout.clone(),
            bounds: super::super::bounds::PersonalResourceBounds::personal_v1_baseline(),
            once: true,
        });

    assert!(matches!(
        result,
        Err(super::super::server::PersonalDaemonError::Io { detail })
            if detail.contains("reconcile durable scheduler recovery before startup")
    ));
    assert!(!endpoint_document_path(&layout).exists());
    assert!(!layout.daemon_lock_path().exists());

    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn private_scheduler_tick_isolates_unreadable_contract_without_wia_handoff() {
    let database_path = temporary_scheduler_database_path();
    let (_, scheduler_work_key) = persist_pending_bound_handoff(&database_path, false);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    store
        .insert_task_contract(
            &TaskContractRow {
                contract_id: object_id(850),
                task_ref: scheduler_work_key.task_ref.clone(),
                contract_epoch: 2,
                user_intent_record_id: object_id(851),
                interpretation_id: object_id(852),
                accepted_by: "principal://personal/daemon".to_owned(),
                contract_digest: format!("sha256:{}", "e".repeat(64)),
                canonical_json: "{\"task_contract\":\"superseding-tick-fixture\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000850").unwrap(),
                object_id: object_id(850),
                domain: LifecycleDomain::Task,
                object_version: Version::INITIAL,
                event_type: "task-contract.superseded".to_owned(),
                canonical_json: "{\"event\":\"task-contract\"}".to_owned(),
            },
            1,
        )
        .unwrap();
    drop(store);

    // This row fails closed before the handoff, while the bounded pass remains
    // available to later rows. The exact rejected authority read is
    // deliberately not part of this safety boundary.
    assert!(super::run_private_scheduler_tick(&database_path).is_ok());

    let reopened_store = SqliteAuthorityStore::open(&database_path).unwrap();
    assert!(
        reopened_store
            .list_consumed_worker_iteration_authorizations()
            .unwrap()
            .is_empty()
    );
    let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    let scheduler_row = scheduler_repository
        .load(&scheduler_work_key)
        .unwrap()
        .unwrap();
    assert_eq!(scheduler_row.state, SchedulerState::Runnable.as_str());
    assert_eq!(scheduler_row.attempt_count, 0);
    assert_eq!(scheduler_row.lease_owner, None);

    drop(scheduler_repository);
    drop(reopened_store);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn legacy_contract_is_rejected_before_execution_binding_deserialization() {
    let legacy_contract = r#"{
        "header": {
            "schema_version": "cognitiveos.task-contract/0.1"
        }
    }"#;

    assert!(matches!(parse_execution_bound_contract(legacy_contract),
        Err(SchedulerAuthorityError::LegacyContract(version))
        if version == "cognitiveos.task-contract/0.1"));
}

#[test]
fn execution_schema_without_required_bindings_is_rejected_as_malformed() {
    let incomplete_execution_contract = r#"{
        "header": {
            "schema_version": "cognitiveos.task-contract/0.3"
        }
    }"#;

    assert!(matches!(
        parse_execution_bound_contract(incomplete_execution_contract),
        Err(SchedulerAuthorityError::MalformedContract(_))
    ));
}

#[test]
fn context_bound_execution_schema_is_not_rejected_as_legacy() {
    let incomplete_context_bound_contract = r#"{
        "header": {
            "schema_version": "cognitiveos.task-contract/0.4"
        }
    }"#;

    assert!(matches!(
        parse_execution_bound_contract(incomplete_context_bound_contract),
        Err(SchedulerAuthorityError::MalformedContract(_))
    ));
}

#[test]
fn private_pi_candidate_rejects_invalid_non_authority_fields() {
    let invalid_digest_candidate = UntrustedPiCandidate {
        tool_ref: "operation://personal/filesystem/read".to_owned(),
        action: "filesystem.read".to_owned(),
        target: "file:///workspace/input.txt".to_owned(),
        parameters_digest: "not-a-digest".to_owned(),
        expected_state_version: 1,
        operation_descriptor_id: object_id(990),
    };
    assert!(matches!(
        validate_untrusted_pi_candidate(&invalid_digest_candidate),
        Err(SchedulerAuthorityError::PrivatePiProposal(_))
    ));

    let invalid_version_candidate = UntrustedPiCandidate {
        parameters_digest: format!("sha256:{}", "a".repeat(64)),
        expected_state_version: 0,
        ..invalid_digest_candidate
    };
    assert!(matches!(
        validate_untrusted_pi_candidate(&invalid_version_candidate),
        Err(SchedulerAuthorityError::PrivatePiProposal(_))
    ));
}

#[test]
fn stale_contract_epoch_is_rejected_before_scheduler_admission() {
    let binding = SchedulerAuthorityBinding {
        task_ref: "task://personal/superseded-contract".to_owned(),
        contract_epoch: 4,
        action_fingerprint: "scheduler.effect:sha256:test".to_owned(),
    };

    assert!(matches!(
        ensure_current_contract_epoch(&binding, 5),
        Err(SchedulerAuthorityError::StaleContractEpoch {
            task_ref,
            requested_epoch: 4,
            current_epoch: 5,
        }) if task_ref == binding.task_ref
    ));
}

#[test]
fn sealed_wia_evidence_rejects_budget_charge_and_loop_version_row_mismatches() {
    let matching_row = sealed_worker_authorization_row();
    assert!(
        validate_worker_authorization_evidence(&matching_row).is_ok(),
        "a row derived from its sealed WIA payload must validate"
    );

    let mut charge_mismatch = matching_row.clone();
    charge_mismatch.budget_charge_canonical_json = "{\"tool_calls\":2}".to_owned();
    assert!(matches!(
        validate_worker_authorization_evidence(&charge_mismatch),
        Err(SchedulerAuthorityError::CandidateAdmissionComposition(_))
    ));

    let mut loop_version_mismatch = matching_row;
    loop_version_mismatch.expected_loop_version = Version::new(2).unwrap();
    assert!(matches!(
        validate_worker_authorization_evidence(&loop_version_mismatch),
        Err(SchedulerAuthorityError::CandidateAdmissionComposition(_))
    ));
}

#[test]
fn effect_resolution_rejects_missing_ambiguous_and_inconsistent_bindings() {
    let binding = task_binding();

    assert!(matches!(
        select_single_effect_intent(&binding, &[]),
        Err(SchedulerAuthorityError::MissingEffectBinding {
            task_ref,
            contract_epoch: 4,
        }) if task_ref == binding.task_ref
    ));

    let first_intent = effect_intent(11, Some(binding.clone()));
    let second_intent = effect_intent(12, Some(binding.clone()));
    assert!(matches!(
        select_single_effect_intent(&binding, &[first_intent, second_intent]),
        Err(SchedulerAuthorityError::AmbiguousEffectBindings {
            task_ref,
            contract_epoch: 4,
        }) if task_ref == binding.task_ref
    ));

    let inconsistent_intent = effect_intent(
        13,
        Some(TaskBinding {
            task_ref: binding.task_ref.clone(),
            contract_epoch: 5,
        }),
    );
    assert!(matches!(
        select_single_effect_intent(&binding, &[inconsistent_intent]),
        Err(SchedulerAuthorityError::InconsistentEffectBinding(_))
    ));
}

#[test]
fn fresh_zero_intent_task_resolves_to_pre_admission_without_leasing_worker_authority() {
    let database_path = temporary_scheduler_database_path();
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let task_ref = "task://personal/p2-t12-zero-intent";
    persist_repairable_task_contract(&store, 1_400, task_ref);
    let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    scheduler_repository
        .upsert(&scheduler_row(task_ref))
        .unwrap();

    let resolved = resolve_scheduler_work_for_task(&store, task_ref).unwrap();
    assert_eq!(resolved.task_binding.task_ref, task_ref);
    assert_eq!(resolved.task_binding.contract_epoch, 1);
    assert!(
        resolved.authority_binding.is_none(),
        "zero Intent is the pre-admission case, not a missing-Effect error"
    );
    let scheduler_row = scheduler_repository
        .load(&scheduler_work_key(task_ref))
        .unwrap()
        .unwrap();
    assert_eq!(scheduler_row.state, SchedulerState::Runnable.as_str());
    assert_eq!(scheduler_row.attempt_count, 0);
    assert!(
        store
            .list_consumed_worker_iteration_authorizations()
            .unwrap()
            .is_empty(),
        "pre-admission resolution cannot consume worker authority"
    );

    drop(scheduler_repository);
    drop(store);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn scheduler_row_processing_isolates_one_failure_and_reaches_the_next_row() {
    let database_path = temporary_scheduler_database_path();
    let mut repository = SchedulerRepository::open(&database_path).unwrap();
    repository
        .upsert(&scheduler_row("task://personal/a-malformed"))
        .unwrap();
    repository
        .upsert(&scheduler_row("task://personal/b-healthy"))
        .unwrap();
    let rows = repository.list_recoverable().unwrap();
    let mut visited = Vec::new();

    let failures = super::process_scheduler_rows_isolated(rows, |row| {
        visited.push(row.task_ref.clone());
        if row.task_ref.ends_with("a-malformed") {
            Err(SchedulerAuthorityError::MalformedContract(
                "injected row-local failure".to_owned(),
            ))
        } else {
            Ok(())
        }
    });

    assert_eq!(failures, 1);
    assert_eq!(
        visited,
        vec![
            "task://personal/a-malformed".to_owned(),
            "task://personal/b-healthy".to_owned()
        ]
    );

    drop(repository);
    std::fs::remove_file(database_path).unwrap();
}

fn persist_native_workspace_read_dispatch_fixture(
    database_path: &std::path::Path,
) -> WorkerIterationAuthorizationRow {
    let store = SqliteAuthorityStore::open(database_path).unwrap();
    let authorization = sealed_worker_authorization_row();
    let descriptor_id = object_id(1_500);
    let descriptor = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|descriptor| descriptor.family == NativeOperationFamily::WorkspaceRead)
        .unwrap();
    let issued_at = WallTimestamp::parse("2026-08-13T08:00:00Z").unwrap();
    let contract_id = authorization.worker_authorization_root_id.clone();
    let context_request_id = object_id(1_510);
    let context_request_digest = format!("sha256:{}", "6".repeat(64));
    let contract_header = compose_governed_header(
        &contract_id,
        "TaskContract",
        "cognitiveos.task-contract/0.4",
        &context_governance(),
        Vec::new(),
        Vec::new(),
        "p2-t12-native-dispatch-fixture",
        &issued_at,
    )
    .unwrap();
    let contract = TaskContract {
        allowed_state_domains: vec!["task".to_owned(), "effect".to_owned()],
        allowed_tools: vec![descriptor.operation_id.clone()],
        budget: Budget {
            attention_slots: None,
            context_bytes: None,
            egress_bytes: None,
            input_tokens: None,
            money_microunits: None,
            output_tokens: None,
            semantic_calls: Some(1),
            tool_calls: Some(2),
            wall_time_ms: None,
        },
        budget_id: Some(authorization.budget_id.to_generated()),
        conditions: vec![ContractCondition {
            description: "independently verified read".to_owned(),
            id: "accept-read".to_owned(),
            kind: ContractConditionKind::Acceptance,
            machine_expression: None,
            verifier_ref: Some("verifier://personal/fixed-effect".to_owned()),
        }],
        context_request_ref: Some(strong_reference_to(
            &context_request_id,
            &context_request_digest,
        )),
        contract_epoch: authorization.contract_epoch,
        deadline: Some("2027-12-31T00:00:00Z".to_owned()),
        header: contract_header,
        human_gates: None,
        intent_acceptance_ref: strong_reference_to(
            &object_id(1_511),
            &format!("sha256:{}", "a".repeat(64)),
        ),
        intent_interpretation_ref: strong_reference_to(
            &object_id(1_512),
            &format!("sha256:{}", "b".repeat(64)),
        ),
        loop_object_id: Some(authorization.loop_object_id.to_generated()),
        max_iterations: 2,
        max_retries: 1,
        objective: "read one governed workspace file".to_owned(),
        scope: TaskScope {
            in_scope: vec!["workspace read".to_owned()],
            out_of_scope: vec!["mutation".to_owned()],
        },
        task_ref: authorization.task_ref.clone(),
        user_intent_ref: strong_reference_to(
            &object_id(1_513),
            &format!("sha256:{}", "c".repeat(64)),
        ),
        worker_authorization_root_id: Some(
            authorization.worker_authorization_root_id.to_generated(),
        ),
    };
    let (contract_json, contract_digest) = seal_payload(serde_json::to_value(contract).unwrap());
    let contract_row = TaskContractRow {
        contract_id: contract_id.clone(),
        task_ref: authorization.task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
        user_intent_record_id: object_id(1_513),
        interpretation_id: object_id(1_512),
        accepted_by: "principal://personal/daemon".to_owned(),
        contract_digest,
        canonical_json: contract_json,
    };
    store
        .insert_task_contract(
            &contract_row,
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000001501").unwrap(),
                object_id: contract_id.clone(),
                domain: LifecycleDomain::Task,
                object_version: Version::INITIAL,
                event_type: "task-contract.minted".to_owned(),
                canonical_json: "{\"event\":\"task-contract\"}".to_owned(),
            },
            0,
        )
        .unwrap();
    store
        .materialize_draft_task_projection(&contract_row, &issued_at)
        .unwrap();
    store
        .append_scheduler_execution_policy(&SchedulerExecutionPolicyRow {
            task_ref: authorization.task_ref.clone(),
            contract_epoch: authorization.contract_epoch,
            context_request_id: context_request_id.clone(),
            canonical_json: json!({
                "schema_version": 1,
                "task_ref": authorization.task_ref,
                "contract_epoch": authorization.contract_epoch,
                "context": {
                    "request_id": context_request_id.as_str(),
                    "authorization_subject_ref": "principal://tenant-a/daemon",
                    "tenant_id": "tenant-a",
                    "resource_scope_prefix": "workspace://",
                    "conversation_ref": null,
                    "source_limit": 1,
                },
                "admission": {
                    "candidate_id": authorization.selected_candidate_id.as_str(),
                    "authorization_subject_ref": "principal://tenant-a/daemon",
                    "authorization_purpose": "task_execution",
                    "budget_charge": {"semantic_calls": 1},
                    "governance": {
                        "owner": context_governance().owner,
                        "authority": context_governance().authority,
                        "resource_scope": context_governance().resource_scope,
                        "tenant_id": "tenant-a",
                        "created_by": "principal://tenant-a/daemon",
                        "sensitivity": "internal",
                        "purpose_constraints": ["task_execution"],
                        "retention_policy": "standard",
                    },
                    "actor_ref": "principal://personal/daemon",
                    "authority_ref": "authority://personal/daemon",
                    "correlation_id": "correlation://personal/d04-fixture",
                },
            })
            .to_string(),
        })
        .unwrap();
    let principal = PrincipalFacts {
        principal_ref: UriRef::parse("principal://tenant-a/daemon").unwrap(),
        authenticated: true,
        active: true,
        tenant_id: Some("tenant-a".to_owned()),
    };
    let capability = CapabilityConstraints {
        subject: principal.principal_ref.to_string(),
        audience: "authority://personal/effect-authority".to_owned(),
        resource: "workspace://input.txt".to_owned(),
        purpose: "task_execution".to_owned(),
        actions: ["read".to_owned()].into(),
        parameter_bounds: BTreeMap::new(),
        lease: LeaseWindow {
            // 生产 tick (`run_private_scheduler_tick_with_store`) 用 SystemClock
            // 校验 capability 租约；固定 2026-08-14T00:00Z 的过期点会随真实时间跨过
            // UTC 午夜而触发 AUTH_CAPABILITY_EXPIRED。改为覆盖整个项目时间线的宽
            // 窗口，使 fixture 不再时间脆弱（tests-only；不改变生产校验语义）。
            not_before: WallTimestamp::parse("2026-01-01T00:00:00Z").unwrap(),
            expires: WallTimestamp::parse("2027-12-31T00:00:00Z").unwrap(),
        },
        depth_remaining: 1,
        issued_epoch: 1,
    };
    let actor_chain = ActorChainFacts {
        chain_digest: format!("sha256:{}", "d".repeat(64)),
        resolved: true,
    };
    let membership = Some(MembershipFacts {
        valid: true,
        roles: ["owner".to_owned()].into(),
    });
    let facts_id = object_id(1_514);
    let facts_header = compose_governed_header(
        &facts_id,
        "ContextAuthorizationFacts",
        "cognitiveos.context-authorization-facts/0.1",
        &context_governance(),
        Vec::new(),
        Vec::new(),
        "p2-t12-native-dispatch-facts",
        &issued_at,
    )
    .unwrap();
    let (facts_json, _) = seal_payload(json!({
        "header": facts_header,
        "fact_set_id": facts_id,
        "subject_ref": principal.principal_ref,
        "tenant_id": "tenant-a",
        "principal": principal,
        "actor_chain": actor_chain,
        "membership": membership,
        "capability_links": [capability],
        "explicit_denies": [],
        "capability_set_version": 1,
        "issued_revocation_epoch": 1,
    }));
    store
        .append_context_authorization_facts(&ContextAuthorizationFactsRow {
            fact_set_id: facts_id,
            subject_ref: "principal://tenant-a/daemon".to_owned(),
            tenant_id: "tenant-a".to_owned(),
            principal,
            actor_chain,
            membership,
            capability_links: vec![capability],
            explicit_denies: Vec::new(),
            capability_set_version: 1,
            issued_revocation_epoch: 1,
            canonical_json: facts_json,
        })
        .unwrap();
    store
        .append_context_revocation_fact(&context_revocation_fact(
            &context_governance(),
            object_id(1_515),
            1,
            &issued_at,
        ))
        .unwrap();
    store
        .append_daemon_operation_descriptor(&DaemonOperationDescriptorRow {
            descriptor_id: descriptor_id.clone(),
            descriptor: OperationDescriptor {
                operation_id: descriptor.operation_id.clone(),
                action: descriptor.action.clone(),
                effect_class: EffectClass::Pure,
                executor: descriptor.executor.clone(),
                capabilities: ExecutorCapabilities {
                    queryable: true,
                    idempotent: true,
                },
                descriptor_version: descriptor.descriptor_version,
            },
            canonical_json: "{\"descriptor\":\"native.workspace.read\"}".to_owned(),
        })
        .unwrap();
    let parameters_digest = format!("sha256:{}", "8".repeat(64));
    store
        .append_operation_candidate_proposal(&OperationCandidateProposalRow {
            candidate_id: authorization.selected_candidate_id.clone(),
            task_ref: authorization.task_ref.clone(),
            contract_epoch: authorization.contract_epoch,
            candidate_source_ref: "observation://personal/d04-failure-first".to_owned(),
            tool_ref: descriptor.operation_id.clone(),
            action: descriptor.action.clone(),
            target: "workspace://input.txt".to_owned(),
            parameters_digest: parameters_digest.clone(),
            expected_state_version: Version::INITIAL.get(),
            operation_descriptor_ref: descriptor_id,
            canonical_json: "{\"candidate\":\"native.workspace.read\"}".to_owned(),
        })
        .unwrap();
    let admitted_at = WallTimestamp::parse("2026-08-13T08:00:00Z").unwrap();
    store
        .admit_object(&ObjectAdmission {
            object: StoredObject {
                object_id: authorization.loop_object_id.clone(),
                domain: LifecycleDomain::Loop,
                state: state("DECIDE"),
                version: Version::INITIAL,
                body: json!({"loop": "native.workspace.read"}),
            },
            admitted_at: admitted_at.clone(),
            event: EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000001505").unwrap(),
                object_id: authorization.loop_object_id.clone(),
                domain: LifecycleDomain::Loop,
                object_version: Version::INITIAL,
                event_type: "loop.admitted".to_owned(),
                canonical_json: "{\"event\":\"loop\"}".to_owned(),
            },
            outbox: Vec::new(),
            fencing_epoch: Some(1),
        })
        .unwrap();
    let initial_budget = BudgetState::new(BTreeMap::from([("tool_calls".to_owned(), 2)])).unwrap();
    store
        .create_budget(
            &authorization.budget_id,
            &serde_json::to_string(&initial_budget).unwrap(),
            &admitted_at,
        )
        .unwrap();
    store
        .commit_candidate_admission(&CandidateAdmissionCommit {
            selected_candidate_id: authorization.selected_candidate_id.clone(),
            intent: IntentRow {
                intent_id: authorization.intent_id.clone(),
                idempotency_key: "p2-t12-d04-workspace-read".to_owned(),
                parameters_digest,
                action: descriptor.action.clone(),
                target: "workspace://input.txt".to_owned(),
                effect_object_id: authorization.effect_object_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: Some(TaskBinding {
                    task_ref: authorization.task_ref.clone(),
                    contract_epoch: authorization.contract_epoch,
                }),
                canonical_json: "{\"intent\":\"native.workspace.read\"}".to_owned(),
            },
            intent_event: EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000001504").unwrap(),
                object_id: authorization.intent_id.clone(),
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"event\":\"intent\",\"event_time\":\"2026-08-13T08:00:00Z\"}"
                    .to_owned(),
            },
            effect_admission: ObjectAdmission {
                object: StoredObject {
                    object_id: authorization.effect_object_id.clone(),
                    domain: LifecycleDomain::Effect,
                    state: state("PROPOSED"),
                    version: Version::INITIAL,
                    body: json!({"effect": "native.workspace.read"}),
                },
                admitted_at: admitted_at.clone(),
                event: EventDraft {
                    event_id: EventId::parse("00000000-0000-7000-a000-000000001506").unwrap(),
                    object_id: authorization.effect_object_id.clone(),
                    domain: LifecycleDomain::Effect,
                    object_version: Version::INITIAL,
                    event_type: "effect.admitted".to_owned(),
                    canonical_json: "{\"event\":\"effect\"}".to_owned(),
                },
                outbox: Vec::new(),
                fencing_epoch: Some(1),
            },
            worker_authorization: authorization.clone(),
            loop_transition: TransitionCommit {
                cas: ObjectCas {
                    object_id: authorization.loop_object_id.clone(),
                    domain: LifecycleDomain::Loop,
                    from_state: state("DECIDE"),
                    to_state: state("ACT"),
                    expected_version: Version::INITIAL,
                    next_version: Version::INITIAL.next().unwrap(),
                    committed_at: admitted_at.clone(),
                },
                event: EventDraft {
                    event_id: EventId::parse("00000000-0000-7000-a000-000000001507").unwrap(),
                    object_id: authorization.loop_object_id.clone(),
                    domain: LifecycleDomain::Loop,
                    object_version: Version::INITIAL.next().unwrap(),
                    event_type: "loop.operation-admitted".to_owned(),
                    canonical_json: "{\"event\":\"loop\"}".to_owned(),
                },
                record: RecordDraft {
                    record_id: RecordId::parse("00000000-0000-7000-8000-000000001508").unwrap(),
                    object_id: authorization.loop_object_id.clone(),
                    domain: LifecycleDomain::Loop,
                    object_version: Version::INITIAL.next().unwrap(),
                    canonical_json: "{\"record\":\"loop\"}".to_owned(),
                },
                budget: Some(BudgetCas {
                    budget_id: authorization.budget_id.clone(),
                    expected_version: Version::INITIAL,
                    next_version: Version::INITIAL.next().unwrap(),
                    charge_canonical_json: "{\"tool_calls\":1}".to_owned(),
                    next_state_canonical_json: serde_json::to_string(
                        &BudgetState::new(BTreeMap::from([("tool_calls".to_owned(), 1)])).unwrap(),
                    )
                    .unwrap(),
                }),
                outbox: Vec::new(),
                fencing_epoch: Some(1),
            },
            fencing_epoch: 1,
        })
        .unwrap();
    authorization
}

#[test]
fn native_worker_dispatch_reloads_the_selected_persisted_descriptor() {
    let database_path = temporary_scheduler_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();

    let resolved = resolve_native_worker_dispatch_with_families(
        &store,
        &authorization,
        &ASSEMBLED_EXECUTOR_FAMILIES,
    )
    .unwrap();

    assert_eq!(
        resolved.native_tool.descriptor.family,
        NativeOperationFamily::WorkspaceRead
    );
    assert_eq!(
        resolved.candidate.candidate_id,
        authorization.selected_candidate_id
    );
    assert_eq!(resolved.intent.intent_id, authorization.intent_id);
    assert_eq!(
        resolved.intent.effect_object_id,
        authorization.effect_object_id
    );

    drop(store);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn unassembled_persisted_family_fails_before_effect_authorization() {
    let database_path = temporary_scheduler_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();

    assert!(matches!(
        resolve_native_worker_dispatch_with_families(&store, &authorization, &[]),
        Err(SchedulerAuthorityError::CandidateDescriptorUnavailable(_))
    ));
    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &authorization.effect_object_id)
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "PROPOSED",
        "unsupported execution must fail before Effect authorization or I/O"
    );

    drop(store);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn production_router_stages_process_check_and_http_fetch_carriers() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let resolved = resolve_native_worker_dispatch_with_families(
        &store,
        &authorization,
        &ASSEMBLED_EXECUTOR_FAMILIES,
    )
    .unwrap();

    let router = ProductionNativeToolExecutorRouter::open(1, workspace_root).unwrap();

    // ProcessCheck: the production router now carries the bounded process check.
    let mut process_resolved = resolved.clone();
    process_resolved.native_tool.descriptor = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|descriptor| descriptor.family == NativeOperationFamily::ProcessCheck)
        .unwrap()
        .clone();
    process_resolved.candidate.target = "process://123".to_owned();
    router.stage_resolved(&process_resolved).unwrap();

    // HttpFetchReadOnly: the carrier is wired but fails closed before dispatch
    // because no origin is registered (empty allowlist) — not because the family
    // has no carrier.
    let mut http_resolved = resolved.clone();
    http_resolved.native_tool.descriptor = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|descriptor| descriptor.family == NativeOperationFamily::HttpFetchReadOnly)
        .unwrap()
        .clone();
    http_resolved.candidate.target = "https://example.com/data".to_owned();
    assert!(matches!(
        router.stage_resolved(&http_resolved),
        Err(NativeToolExecutionError::InvalidDescriptor(_))
    ));

    drop(store);
    drop(router);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn production_router_stages_workspace_search_with_persisted_query() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let resolved = resolve_native_worker_dispatch_with_families(
        &store,
        &authorization,
        &ASSEMBLED_EXECUTOR_FAMILIES,
    )
    .unwrap();

    let mut search_resolved = resolved.clone();
    search_resolved.native_tool.descriptor = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|descriptor| descriptor.family == NativeOperationFamily::WorkspaceSearch)
        .unwrap()
        .clone();
    search_resolved.intent.canonical_json = json!({
        "parameters": {"query": "durable input"}
    })
    .to_string();

    let router = ProductionNativeToolExecutorRouter::open(1, workspace_root).unwrap();
    // The production router now carries WorkspaceSearch: it stages the governed
    // query into the search sink instead of failing closed.
    router.stage_resolved(&search_resolved).unwrap();

    // A missing or unparseable query still fails closed before any staging.
    let mut missing_query = search_resolved.clone();
    missing_query.intent.canonical_json = "{}".to_owned();
    assert!(matches!(
        router.stage_resolved(&missing_query),
        Err(NativeToolExecutionError::InvalidDescriptor(_))
    ));

    drop(store);
    drop(router);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn production_router_stages_workspace_write_with_persisted_preimage() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let resolved = resolve_native_worker_dispatch_with_families(
        &store,
        &authorization,
        &ASSEMBLED_EXECUTOR_FAMILIES,
    )
    .unwrap();

    let mut write_resolved = resolved.clone();
    write_resolved.native_tool.descriptor = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|descriptor| descriptor.family == NativeOperationFamily::WorkspaceWrite)
        .unwrap()
        .clone();
    // "ZHVyYWJsZSBpbnB1dA==" is the standard base64 of "durable input".
    write_resolved.intent.canonical_json = json!({
        "parameters": {
            "input_b64": "ZHVyYWJsZSBpbnB1dA==",
            "preimage": "absent"
        }
    })
    .to_string();

    let router = ProductionNativeToolExecutorRouter::open(1, workspace_root).unwrap();
    // The production router now carries WorkspaceWrite: it stages the governed
    // payload + expected preimage into the mutation sink.
    router.stage_resolved(&write_resolved).unwrap();

    // A mutation without a declared preimage still fails closed before staging.
    let mut missing_preimage = write_resolved.clone();
    missing_preimage.intent.canonical_json =
        json!({"parameters": {"input_b64": "ZHVyYWJsZSBpbnB1dA=="}}).to_string();
    assert!(matches!(
        router.stage_resolved(&missing_preimage),
        Err(NativeToolExecutionError::InvalidDescriptor(_))
    ));

    drop(store);
    drop(router);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn production_native_caller_persists_executing_before_workspace_io() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();
    std::fs::write(workspace_root.join("input.txt"), b"durable input").unwrap();
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = Arc::new(SqliteAuthorityStore::open(&database_path).unwrap());
    let resolved = resolve_native_worker_dispatch_with_families(
        store.as_ref(),
        &authorization,
        &ASSEMBLED_EXECUTOR_FAMILIES,
    )
    .unwrap();
    let router = ProductionNativeToolExecutorRouter::open(1, workspace_root).unwrap();
    let executing_observed = Arc::new(AtomicBool::new(false));
    let executing_observed_at_io = Arc::clone(&executing_observed);
    let store_at_io = Arc::clone(&store);
    let effect_id_at_io = authorization.effect_object_id.clone();
    router.install_workspace_read_before_io_hook(move || {
        let effect = store_at_io
            .load_object(LifecycleDomain::Effect, &effect_id_at_io)
            .unwrap()
            .unwrap();
        executing_observed_at_io.store(effect.state.as_str() == "EXECUTING", Ordering::SeqCst);
    });
    let clock = super::FixedSchedulerClock::parse("2026-08-04T12:02:00Z").unwrap();
    let ids = UuidV7Generator;
    let protocol = EffectProtocol::new(
        store.as_ref(),
        &clock,
        &ids,
        UriRef::parse("actor://personal/daemon").unwrap(),
        UriRef::parse("authority://personal/effect-authority").unwrap(),
        UriRef::parse("correlation://personal/p2-t12-d04").unwrap(),
    );

    let closure = dispatch_native_worker_effect(
        &protocol,
        &resolved,
        &router,
        &recovery_effect_grant(),
        &GovernanceCurrency {
            revocation_epoch: 1,
            capability_set_version: 1,
        },
        &WriterLease { epoch: 1 },
    )
    .unwrap();

    assert_eq!(closure, SchedulerEffectClosure::Closed);
    assert!(executing_observed.load(Ordering::SeqCst));
    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &authorization.effect_object_id)
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "RECONCILED"
    );
    // Windows 不能在 cap-std 目录句柄仍打开时删除树；router 持有 durable state Dir。
    drop(router);
    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn private_tick_dispatches_admitted_workspace_read_through_production_router() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();
    std::fs::write(workspace_root.join("input.txt"), b"durable input").unwrap();
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let mut repository = SchedulerRepository::open(&database_path).unwrap();
    repository
        .upsert(&scheduler_row(&authorization.task_ref))
        .unwrap();
    let router = ProductionNativeToolExecutorRouter::open(1, workspace_root).unwrap();
    let artifact_store =
        cognitive_store::ArtifactStore::open(layout.data_dir().join("artifacts"), 1024 * 1024)
            .unwrap();

    super::run_private_scheduler_tick_with_store(
        &store,
        &mut repository,
        layout.config_dir(),
        &router,
        &artifact_store,
    )
    .unwrap();

    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &authorization.effect_object_id)
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "RECONCILED"
    );
    assert_eq!(
        store
            .load_object(LifecycleDomain::Loop, &authorization.loop_object_id)
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "CONTINUE"
    );
    assert!(
        std::fs::read_dir(layout.data_dir().join("artifacts"))
            .unwrap()
            .next()
            .is_some()
    );
    assert_eq!(
        repository
            .load(&scheduler_work_key(&authorization.task_ref))
            .unwrap()
            .unwrap()
            .state,
        SchedulerState::Succeeded.as_str()
    );
    assert!(
        store
            .load_unconsumed_continuation_authorization(&TaskBinding {
                task_ref: authorization.task_ref.clone(),
                contract_epoch: authorization.contract_epoch,
            })
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .load_object(
                LifecycleDomain::Task,
                &authorization.worker_authorization_root_id,
            )
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "COMPLETED"
    );
    assert_eq!(
        store
            .list_consumed_worker_iteration_authorizations()
            .unwrap()
            .len(),
        1
    );

    // Windows 不能在 cap-std 目录句柄仍打开时删除树；router 持有 durable state Dir。
    drop(router);
    drop(artifact_store);
    drop(repository);
    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn public_c1_workspace_read_reaches_independent_verified_task_completion() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();
    std::fs::write(workspace_root.join("input.txt"), b"durable input").unwrap();
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let mut repository = SchedulerRepository::open(&database_path).unwrap();
    repository
        .upsert(&scheduler_row(&authorization.task_ref))
        .unwrap();
    let router = ProductionNativeToolExecutorRouter::open(1, workspace_root).unwrap();
    let artifact_store =
        cognitive_store::ArtifactStore::open(layout.data_dir().join("artifacts"), 1024 * 1024)
            .unwrap();

    super::run_private_scheduler_tick_with_store(
        &store,
        &mut repository,
        layout.config_dir(),
        &router,
        &artifact_store,
    )
    .unwrap();

    let task = store
        .load_object(
            LifecycleDomain::Task,
            &authorization.worker_authorization_root_id,
        )
        .unwrap()
        .unwrap();
    assert_eq!(task.state.as_str(), "COMPLETED");
    assert_eq!(
        repository
            .load(&scheduler_work_key(&authorization.task_ref))
            .unwrap()
            .unwrap()
            .state,
        SchedulerState::Succeeded.as_str()
    );
    assert!(
        store
            .load_unconsumed_continuation_authorization(&TaskBinding {
                task_ref: authorization.task_ref.clone(),
                contract_epoch: authorization.contract_epoch,
            })
            .unwrap()
            .is_none(),
        "terminal Task acceptance must not issue continuation authority"
    );

    // Windows 不能在 cap-std 目录句柄仍打开时删除树；router 持有 durable state Dir。
    drop(router);
    drop(artifact_store);
    drop(repository);
    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn non_authority_completion_signals_cannot_complete_a_draft_task() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();
    std::fs::write(workspace_root.join("input.txt"), b"durable input").unwrap();
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let artifact_store =
        cognitive_store::ArtifactStore::open(layout.data_dir().join("artifacts"), 1024 * 1024)
            .unwrap();
    let task_binding = TaskBinding {
        task_ref: authorization.task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
    };

    let error = super::complete_task_from_persisted_verification(
        &store,
        &artifact_store,
        &SystemClock,
        &UuidV7Generator,
        &task_binding,
        &object_id(9_999),
        &WriterLease { epoch: 1 },
    )
    .err()
    .unwrap();
    assert!(
        matches!(error, super::TaskCompletionError::TaskUnavailable),
        "DRAFT is ineligible for candidate/acceptance; got {error:?}"
    );
    let task = store
        .load_object(
            LifecycleDomain::Task,
            &authorization.worker_authorization_root_id,
        )
        .unwrap()
        .unwrap();
    assert_eq!(task.state.as_str(), "DRAFT");
    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &authorization.effect_object_id)
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "PROPOSED",
        "Provider success, Tool exit 0, process exit and Pi agent_end have no Task completion caller"
    );

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn duplicate_acceptance_is_rejected_after_verified_completion() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();
    std::fs::write(workspace_root.join("input.txt"), b"durable input").unwrap();
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let mut repository = SchedulerRepository::open(&database_path).unwrap();
    repository
        .upsert(&scheduler_row(&authorization.task_ref))
        .unwrap();
    let router = ProductionNativeToolExecutorRouter::open(1, workspace_root).unwrap();
    let artifact_store =
        cognitive_store::ArtifactStore::open(layout.data_dir().join("artifacts"), 1024 * 1024)
            .unwrap();
    let task_binding = TaskBinding {
        task_ref: authorization.task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
    };

    super::run_private_scheduler_tick_with_store(
        &store,
        &mut repository,
        layout.config_dir(),
        &router,
        &artifact_store,
    )
    .unwrap();
    let report = store
        .load_latest_verification_report_for_task_binding(&task_binding)
        .unwrap()
        .unwrap();
    let error = super::complete_task_from_persisted_verification(
        &store,
        &artifact_store,
        &SystemClock,
        &UuidV7Generator,
        &task_binding,
        &report.verification_report_id,
        &WriterLease { epoch: 1 },
    )
    .err()
    .unwrap();
    assert!(matches!(
        error,
        super::TaskCompletionError::DuplicateAcceptance
    ));
    assert_eq!(
        store
            .load_object(
                LifecycleDomain::Task,
                &authorization.worker_authorization_root_id,
            )
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "COMPLETED"
    );

    // Windows 不能在 cap-std 目录句柄仍打开时删除树；router 持有 durable state Dir。
    drop(router);
    drop(artifact_store);
    drop(repository);
    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

fn persist_verified_workspace_read_before_acceptance(
    layout: &PersonalDataLayout,
    store: &SqliteAuthorityStore,
    authorization: &WorkerIterationAuthorizationRow,
) -> (cognitive_store::ArtifactStore, ObjectId) {
    let workspace_root = layout.data_dir().join("workspace");
    let router = ProductionNativeToolExecutorRouter::open(1, workspace_root).unwrap();
    let artifact_store =
        cognitive_store::ArtifactStore::open(layout.data_dir().join("artifacts"), 1024 * 1024)
            .unwrap();
    let writer_lease = WriterLease { epoch: 1 };
    let clock = super::FixedSchedulerClock::parse("2026-08-04T12:02:00Z").unwrap();
    let ids = UuidV7Generator;
    super::activate_task_for_worker_authorization(
        store,
        &clock,
        &ids,
        authorization,
        &writer_lease,
    )
    .unwrap();
    let resolved = resolve_native_worker_dispatch_with_families(
        store,
        authorization,
        &ASSEMBLED_EXECUTOR_FAMILIES,
    )
    .unwrap();
    let protocol = EffectProtocol::new(
        store,
        &clock,
        &ids,
        UriRef::parse("actor://personal/daemon").unwrap(),
        UriRef::parse("authority://personal/effect-authority").unwrap(),
        UriRef::parse("correlation://personal/p2-t14-before-acceptance").unwrap(),
    );
    dispatch_native_worker_effect(
        &protocol,
        &resolved,
        &router,
        &recovery_effect_grant(),
        &GovernanceCurrency {
            revocation_epoch: 1,
            capability_set_version: 1,
        },
        &writer_lease,
    )
    .unwrap();
    let current_loop = store
        .load_object(LifecycleDomain::Loop, &authorization.loop_object_id)
        .unwrap()
        .unwrap();
    let task_binding = TaskBinding {
        task_ref: authorization.task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
    };
    let request =
        crate::personal::verification_executor::begin_verification_from_current_task_contract(
            store,
            &clock,
            &ids,
            &task_binding,
            &authorization.loop_object_id,
            current_loop.version,
            &authorization.effect_object_id,
            &writer_lease,
        )
        .unwrap();
    let outcome = crate::personal::verification_executor::run_production_independent_verification(
        store,
        &artifact_store,
        &clock,
        &ids,
        &request.verification_request_id,
        &writer_lease,
    )
    .unwrap();
    (artifact_store, outcome.report.verification_report_id)
}

fn persist_extra_open_task_effect(
    store: &SqliteAuthorityStore,
    authorization: &WorkerIterationAuthorizationRow,
) -> ObjectId {
    let extra_effect_id = object_id(9_980);
    let extra_intent_id = object_id(9_981);
    let admitted_at = WallTimestamp::parse("2026-08-13T08:01:00Z").unwrap();
    store
        .admit_object(&ObjectAdmission {
            object: StoredObject {
                object_id: extra_effect_id.clone(),
                domain: LifecycleDomain::Effect,
                state: state("PROPOSED"),
                version: Version::INITIAL,
                body: json!({"effect": "open-extra"}),
            },
            admitted_at: admitted_at.clone(),
            event: EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000009980").unwrap(),
                object_id: extra_effect_id.clone(),
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "effect.admitted".to_owned(),
                canonical_json: "{\"event\":\"effect-open\"}".to_owned(),
            },
            outbox: Vec::new(),
            fencing_epoch: Some(1),
        })
        .unwrap();
    store
        .insert_intent(
            &IntentRow {
                intent_id: extra_intent_id.clone(),
                idempotency_key: "p2-t14-open-effect".to_owned(),
                parameters_digest: format!("sha256:{}", "9".repeat(64)),
                action: "filesystem.read".to_owned(),
                target: "workspace://other.txt".to_owned(),
                effect_object_id: extra_effect_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: Some(TaskBinding {
                    task_ref: authorization.task_ref.clone(),
                    contract_epoch: authorization.contract_epoch,
                }),
                canonical_json: "{\"intent\":\"open-extra\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000009981").unwrap(),
                object_id: extra_intent_id,
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"event\":\"intent\",\"event_time\":\"2026-08-13T08:01:00Z\"}"
                    .to_owned(),
            },
        )
        .unwrap();
    extra_effect_id
}

#[test]
fn open_effect_blocks_candidate_complete() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();
    std::fs::write(workspace_root.join("input.txt"), b"durable input").unwrap();
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let (artifact_store, report_id) =
        persist_verified_workspace_read_before_acceptance(&layout, &store, &authorization);
    let extra_effect_id = persist_extra_open_task_effect(&store, &authorization);
    let task_binding = TaskBinding {
        task_ref: authorization.task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
    };

    let error = super::complete_task_from_persisted_verification(
        &store,
        &artifact_store,
        &SystemClock,
        &UuidV7Generator,
        &task_binding,
        &report_id,
        &WriterLease { epoch: 1 },
    )
    .err()
    .unwrap();
    assert!(matches!(error, super::TaskCompletionError::EffectsOpen));
    assert_eq!(
        store
            .load_object(
                LifecycleDomain::Task,
                &authorization.worker_authorization_root_id,
            )
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "ACTIVE"
    );
    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &extra_effect_id)
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "PROPOSED"
    );

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn superseded_verification_report_cannot_complete_a_task() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();
    std::fs::write(workspace_root.join("input.txt"), b"durable input").unwrap();
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let (artifact_store, original_report_id) =
        persist_verified_workspace_read_before_acceptance(&layout, &store, &authorization);
    let task_binding = TaskBinding {
        task_ref: authorization.task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
    };
    let original = store
        .load_verification_report(&original_report_id)
        .unwrap()
        .unwrap();
    let mut successor = original.clone();
    successor.verification_report_id = object_id(9_990);
    successor.verifier_version = "v1-superseding".to_owned();
    successor.completed_at = WallTimestamp::parse("2026-08-13T08:03:00Z").unwrap();
    store.append_verification_report(&successor).unwrap();
    let error = super::complete_task_from_persisted_verification(
        &store,
        &artifact_store,
        &SystemClock,
        &UuidV7Generator,
        &task_binding,
        &original_report_id,
        &WriterLease { epoch: 1 },
    )
    .err()
    .unwrap();
    assert!(matches!(
        error,
        super::TaskCompletionError::VerificationUnavailable
    ));
    assert_ne!(
        store
            .load_object(
                LifecycleDomain::Task,
                &authorization.worker_authorization_root_id,
            )
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "COMPLETED"
    );

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn missing_cas_evidence_cannot_complete_a_task() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();
    std::fs::write(workspace_root.join("input.txt"), b"durable input").unwrap();
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let (artifact_store, report_id) =
        persist_verified_workspace_read_before_acceptance(&layout, &store, &authorization);
    let artifacts_root = layout.data_dir().join("artifacts");
    let task_binding = TaskBinding {
        task_ref: authorization.task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
    };
    for entry in std::fs::read_dir(&artifacts_root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            std::fs::remove_file(path).unwrap();
        }
    }
    let error = super::complete_task_from_persisted_verification(
        &store,
        &artifact_store,
        &SystemClock,
        &UuidV7Generator,
        &task_binding,
        &report_id,
        &WriterLease { epoch: 1 },
    )
    .err()
    .unwrap();
    assert!(matches!(
        error,
        super::TaskCompletionError::EvidenceUnavailable(_)
    ));
    assert_ne!(
        store
            .load_object(
                LifecycleDomain::Task,
                &authorization.worker_authorization_root_id,
            )
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "COMPLETED"
    );

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn stale_fixed_post_state_cannot_complete_a_task() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();
    std::fs::write(workspace_root.join("input.txt"), b"durable input").unwrap();
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let (artifact_store, report_id) =
        persist_verified_workspace_read_before_acceptance(&layout, &store, &authorization);

    // The production path leaves the verified Effect at RECONCILED@V with
    // fixed_post_state.subject_version = V (they match, so a normal
    // completion would proceed). Advance the Effect RECONCILED -> VERIFIED
    // through the sanctioned verify_effect boundary so its durable version
    // becomes V+1 while the pinned fixed_post_state still references V; the
    // pinned state is now stale relative to the authoritative Effect.
    let effect_id = authorization.effect_object_id.clone();
    let fixed_version = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap()
        .version;
    let verify_clock = super::FixedSchedulerClock::parse("2026-08-04T12:03:00Z").unwrap();
    let protocol = EffectProtocol::new(
        &store,
        &verify_clock,
        &UuidV7Generator,
        UriRef::parse("actor://personal/daemon").unwrap(),
        UriRef::parse("authority://personal/effect-authority").unwrap(),
        UriRef::parse("correlation://personal/p2-t14-stale-fixed-post-state").unwrap(),
    );
    let record = VerificationRecord {
        verification_object_id: report_id.clone(),
        report_id: report_id.clone(),
        status: VerificationStatus::Passed,
        subject_domain: LifecycleDomain::Effect,
        subject_object_id: effect_id.clone(),
        fixed_post_state_version: fixed_version,
    };
    protocol
        .verify_effect(
            &effect_id,
            fixed_version,
            &record,
            &WriterLease { epoch: 1 },
        )
        .unwrap();

    let task_binding = TaskBinding {
        task_ref: authorization.task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
    };
    let error = super::complete_task_from_persisted_verification(
        &store,
        &artifact_store,
        &SystemClock,
        &UuidV7Generator,
        &task_binding,
        &report_id,
        &WriterLease { epoch: 1 },
    )
    .err()
    .unwrap();
    assert!(matches!(
        error,
        super::TaskCompletionError::VerificationUnavailable
    ));
    assert_ne!(
        store
            .load_object(
                LifecycleDomain::Task,
                &authorization.worker_authorization_root_id,
            )
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "COMPLETED"
    );

    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn interrupted_native_dispatch_reconciles_original_key_without_second_io() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();
    std::fs::write(workspace_root.join("input.txt"), b"durable input").unwrap();
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = Arc::new(SqliteAuthorityStore::open(&database_path).unwrap());
    let resolved = resolve_native_worker_dispatch_with_families(
        store.as_ref(),
        &authorization,
        &ASSEMBLED_EXECUTOR_FAMILIES,
    )
    .unwrap();
    let router = ProductionNativeToolExecutorRouter::open(1, workspace_root).unwrap();
    router.stage_resolved(&resolved).unwrap();
    let io_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let io_count_at_read = Arc::clone(&io_count);
    router.install_workspace_read_before_io_hook(move || {
        io_count_at_read.fetch_add(1, Ordering::SeqCst);
    });
    let clock = super::FixedSchedulerClock::parse("2026-08-04T12:02:00Z").unwrap();
    let ids = UuidV7Generator;
    let protocol = EffectProtocol::new(
        store.as_ref(),
        &clock,
        &ids,
        UriRef::parse("actor://personal/daemon").unwrap(),
        UriRef::parse("authority://personal/effect-authority").unwrap(),
        UriRef::parse("correlation://personal/p2-t12-d04-crash").unwrap(),
    );
    let grant = recovery_effect_grant();
    let currency = GovernanceCurrency {
        revocation_epoch: 1,
        capability_set_version: 1,
    };
    let lease = WriterLease { epoch: 1 };
    let authorized = protocol
        .authorize_effect(
            &authorization.effect_object_id,
            Version::INITIAL,
            &grant,
            &currency,
            &lease,
        )
        .unwrap();
    let (_, outcome) = protocol
        .dispatch_effect(
            &authorization.effect_object_id,
            authorized.after_version,
            &grant,
            &currency,
            &router,
            &lease,
        )
        .unwrap();
    assert!(matches!(
        outcome,
        cognitive_kernel::executor::DispatchOutcome::Executed { .. }
    ));
    assert_eq!(io_count.load(Ordering::SeqCst), 1);
    let interrupted = resolve_native_worker_dispatch_with_families(
        store.as_ref(),
        &authorization,
        &ASSEMBLED_EXECUTOR_FAMILIES,
    )
    .unwrap();
    assert_eq!(interrupted.effect_state, "EXECUTING");

    assert_eq!(
        reconcile_interrupted_native_worker_effect(&protocol, &interrupted, &router, &lease)
            .unwrap(),
        SchedulerEffectClosure::Closed
    );
    assert_eq!(io_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &authorization.effect_object_id)
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "RECONCILED"
    );

    // Windows 不能在 cap-std 目录句柄仍打开时删除树；router 持有 durable state Dir。
    drop(router);
    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn restarted_periodic_recovery_never_repeats_an_unrecorded_workspace_read() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    let workspace_root = layout.data_dir().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();
    std::fs::write(workspace_root.join("input.txt"), b"durable input").unwrap();
    let database_path = layout.authority_database_path();
    let authorization = persist_native_workspace_read_dispatch_fixture(&database_path);
    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let mut repository = SchedulerRepository::open(&database_path).unwrap();
    repository
        .upsert(&scheduler_row(&authorization.task_ref))
        .unwrap();
    let leased = repository
        .acquire_lease(
            &scheduler_work_key(&authorization.task_ref),
            "personal-daemon-scheduler",
            51,
            "2026-08-13T08:05:00Z",
        )
        .unwrap();
    let dispatch = SchedulerDispatch {
        task_ref: authorization.task_ref.clone(),
        contract_epoch: authorization.contract_epoch,
        lease_owner: leased.lease_owner.unwrap(),
        lease_epoch: leased.lease_epoch,
        lease_expires: leased.lease_expires.unwrap(),
        attempt_count: leased.attempt_count,
    };
    super::consume_worker_authorization_for_attempt(
        &store,
        &super::FixedSchedulerClock::parse("2026-08-13T08:01:00Z").unwrap(),
        &authorization.authorization_id,
        object_id(1_509),
        &dispatch,
    )
    .unwrap();
    let resolved = resolve_native_worker_dispatch_with_families(
        &store,
        &authorization,
        &ASSEMBLED_EXECUTOR_FAMILIES,
    )
    .unwrap();
    let first_router = ProductionNativeToolExecutorRouter::open(1, workspace_root.clone()).unwrap();
    first_router.stage_resolved(&resolved).unwrap();
    let first_io_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let first_io_count_at_read = Arc::clone(&first_io_count);
    first_router.install_workspace_read_before_io_hook(move || {
        first_io_count_at_read.fetch_add(1, Ordering::SeqCst);
    });
    let crash_clock = super::FixedSchedulerClock::parse("2026-08-04T12:02:00Z").unwrap();
    let crash_ids = UuidV7Generator;
    let crash_protocol = EffectProtocol::new(
        &store,
        &crash_clock,
        &crash_ids,
        UriRef::parse("actor://personal/daemon").unwrap(),
        UriRef::parse("authority://personal/effect-authority").unwrap(),
        UriRef::parse("correlation://personal/p2-t12-d04-restart").unwrap(),
    );
    let grant = recovery_effect_grant();
    let currency = GovernanceCurrency {
        revocation_epoch: 1,
        capability_set_version: 1,
    };
    let lease = WriterLease { epoch: 1 };
    let authorized = crash_protocol
        .authorize_effect(
            &authorization.effect_object_id,
            Version::INITIAL,
            &grant,
            &currency,
            &lease,
        )
        .unwrap();
    crash_protocol
        .dispatch_effect(
            &authorization.effect_object_id,
            authorized.after_version,
            &grant,
            &currency,
            &first_router,
            &lease,
        )
        .unwrap();
    assert_eq!(first_io_count.load(Ordering::SeqCst), 1);
    drop(first_router);

    let restarted_router = ProductionNativeToolExecutorRouter::open(1, workspace_root).unwrap();
    let restarted_io_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let restarted_io_count_at_read = Arc::clone(&restarted_io_count);
    restarted_router.install_workspace_read_before_io_hook(move || {
        restarted_io_count_at_read.fetch_add(1, Ordering::SeqCst);
    });
    let artifact_store =
        cognitive_store::ArtifactStore::open(layout.data_dir().join("artifacts"), 1024 * 1024)
            .unwrap();
    super::run_private_scheduler_tick_with_store(
        &store,
        &mut repository,
        layout.config_dir(),
        &restarted_router,
        &artifact_store,
    )
    .unwrap();

    assert_eq!(first_io_count.load(Ordering::SeqCst), 1);
    assert_eq!(restarted_io_count.load(Ordering::SeqCst), 0);
    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &authorization.effect_object_id)
            .unwrap()
            .unwrap()
            .state
            .as_str(),
        "NOT_EXECUTED"
    );
    assert_eq!(
        repository
            .load(&scheduler_work_key(&authorization.task_ref))
            .unwrap()
            .unwrap()
            .state,
        SchedulerState::Failed.as_str()
    );

    // Windows 不能在 cap-std 目录句柄仍打开时删除树；router 持有 durable state Dir。
    drop(restarted_router);
    drop(artifact_store);
    drop(repository);
    drop(store);
    std::fs::remove_dir_all(layout.data_dir().parent().unwrap().parent().unwrap()).unwrap();
}

#[test]
fn replaced_scheduler_lease_is_rejected_before_native_effect_dispatch() {
    let database_path = temporary_scheduler_database_path();
    let mut repository = SchedulerRepository::open(&database_path).unwrap();
    let task_ref = "task://personal/d04-stale-dispatch";
    repository.upsert(&scheduler_row(task_ref)).unwrap();
    repository
        .acquire_eligible_lease(
            &scheduler_work_key(task_ref),
            "personal-daemon-scheduler",
            41,
            "2026-08-13T08:00:00Z",
            "2026-08-13T08:00:30Z",
        )
        .unwrap();
    repository
        .acquire_eligible_lease(
            &scheduler_work_key(task_ref),
            "personal-daemon-scheduler",
            42,
            "2026-08-13T08:00:30Z",
            "2026-08-13T08:01:30Z",
        )
        .unwrap();
    let stale_dispatch = SchedulerDispatch {
        task_ref: task_ref.to_owned(),
        contract_epoch: 1,
        lease_owner: "personal-daemon-scheduler".to_owned(),
        lease_epoch: 41,
        lease_expires: "2026-08-13T08:01:00Z".to_owned(),
        attempt_count: 1,
    };

    assert!(matches!(
        verify_scheduler_dispatch_current(&mut repository, &stale_dispatch),
        Err(SchedulerAuthorityError::DispatchBindingMismatch(_))
    ));

    drop(repository);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn reached_ceiling_returns_durable_stop_without_attempting_a_scheduler_lease() {
    let lease_acquisition_attempted = Cell::new(false);

    let admission = complete_scheduler_admission(
        SchedulerCeilingDispatch::Stopped(committed_ceiling_stop()),
        || {
            lease_acquisition_attempted.set(true);
            unreachable!("a reached ceiling must not acquire a scheduler lease")
        },
    )
    .unwrap();

    assert!(matches!(admission, SchedulerDispatchAdmission::Stopped(_)));
    assert!(
        !lease_acquisition_attempted.get(),
        "a terminal ceiling STOP must precede every lease attempt"
    );
}

#[test]
fn clear_ceiling_acquires_exactly_one_scheduler_lease() {
    let lease_acquisition_count = Cell::new(0);

    let admission = complete_scheduler_admission(SchedulerCeilingDispatch::Proceed, || {
        lease_acquisition_count.set(lease_acquisition_count.get() + 1);
        Ok(SchedulerDispatch {
            task_ref: "task://personal/admission-order".to_owned(),
            contract_epoch: 1,
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 3,
            lease_expires: "2026-08-02T00:01:00Z".to_owned(),
            attempt_count: 1,
        })
    })
    .unwrap();

    assert!(matches!(admission, SchedulerDispatchAdmission::Leased(_)));
    assert_eq!(lease_acquisition_count.get(), 1);
}

#[test]
fn ceiling_stop_skips_the_effect_closure_callback() {
    let effect_closure_attempted = Cell::new(false);

    let attempt = complete_scheduler_worker_attempt(
        SchedulerDispatchAdmission::Stopped(committed_ceiling_stop()),
        |_| {
            effect_closure_attempted.set(true);
            unreachable!("a stopped scheduler attempt must not process an Effect")
        },
    )
    .unwrap();

    assert!(matches!(attempt, SchedulerWorkerAttempt::Stopped(_)));
    assert!(
        !effect_closure_attempted.get(),
        "a durable ceiling STOP must precede every Effect-closure callback"
    );
}

#[test]
fn unresolved_effect_keeps_the_fenced_dispatch_for_reconciliation() {
    let dispatch = SchedulerDispatch {
        task_ref: "task://personal/effect-reconciliation".to_owned(),
        contract_epoch: 1,
        lease_owner: "scheduler-worker".to_owned(),
        lease_epoch: 7,
        lease_expires: "2026-08-02T00:01:00Z".to_owned(),
        attempt_count: 1,
    };

    let attempt = complete_scheduler_worker_attempt(
        SchedulerDispatchAdmission::Leased(dispatch.clone()),
        |received_dispatch| {
            assert_eq!(received_dispatch, dispatch);
            Ok(SchedulerEffectClosure::PendingReconciliation)
        },
    )
    .unwrap();

    assert_eq!(
        attempt,
        SchedulerWorkerAttempt::AwaitingReconciliation(dispatch),
        "an unresolved Effect must not be converted into a scheduler success"
    );
}

#[test]
fn only_a_closed_effect_can_release_the_exact_fenced_scheduler_dispatch() {
    let dispatch = SchedulerDispatch {
        task_ref: "task://personal/closed-effect".to_owned(),
        contract_epoch: 1,
        lease_owner: "scheduler-worker".to_owned(),
        lease_epoch: 8,
        lease_expires: "2026-08-02T00:01:00Z".to_owned(),
        attempt_count: 1,
    };
    let release_count = Cell::new(0);

    let released = release_closed_effect_dispatch(
        SchedulerWorkerAttempt::EffectClosed(dispatch.clone()),
        |received_dispatch| {
            release_count.set(release_count.get() + 1);
            assert_eq!(received_dispatch, dispatch);
            Ok(())
        },
    )
    .unwrap();

    assert_eq!(released, SchedulerWorkerAttempt::EffectClosed(dispatch));
    assert_eq!(release_count.get(), 1);
}

#[test]
fn pending_effect_reconciliation_does_not_release_its_scheduler_lease() {
    let dispatch = SchedulerDispatch {
        task_ref: "task://personal/pending-effect".to_owned(),
        contract_epoch: 1,
        lease_owner: "scheduler-worker".to_owned(),
        lease_epoch: 9,
        lease_expires: "2026-08-02T00:01:00Z".to_owned(),
        attempt_count: 1,
    };
    let release_attempted = Cell::new(false);

    let retained = release_closed_effect_dispatch(
        SchedulerWorkerAttempt::AwaitingReconciliation(dispatch.clone()),
        |_| {
            release_attempted.set(true);
            unreachable!("a pending Effect must retain its fenced scheduler lease")
        },
    )
    .unwrap();

    assert_eq!(
        retained,
        SchedulerWorkerAttempt::AwaitingReconciliation(dispatch)
    );
    assert!(
        !release_attempted.get(),
        "a pending Effect must not release its scheduler lease"
    );
}

#[test]
fn closed_effect_releases_the_matching_durable_lease_without_completing_the_task() {
    let database_path = temporary_scheduler_database_path();
    let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    let task_ref = "task://personal/durable-effect-closure";
    scheduler_repository
        .upsert(&scheduler_row(task_ref))
        .unwrap();
    scheduler_repository
        .acquire_lease(
            &scheduler_work_key(task_ref),
            "scheduler-worker",
            10,
            "2026-08-03T00:00:00Z",
        )
        .unwrap();
    let dispatch = SchedulerDispatch {
        task_ref: task_ref.to_owned(),
        contract_epoch: 1,
        lease_owner: "scheduler-worker".to_owned(),
        lease_epoch: 10,
        lease_expires: "2026-08-03T00:01:00Z".to_owned(),
        attempt_count: 1,
    };

    let completed_attempt = complete_resolved_effect_and_release(
        SchedulerWorkerAttempt::EffectClosed(dispatch.clone()),
        &mut scheduler_repository,
        "2026-08-03T00:00:30Z",
    )
    .unwrap();

    assert_eq!(
        completed_attempt,
        SchedulerWorkerAttempt::EffectClosed(dispatch),
        "a closed Effect ends this scheduler attempt, not Task acceptance"
    );
    let durable_row = scheduler_repository
        .load(&scheduler_work_key(task_ref))
        .unwrap()
        .unwrap();
    assert_eq!(durable_row.state, SchedulerState::Succeeded.as_str());
    assert_eq!(durable_row.lease_owner, None);
    drop(scheduler_repository);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn recovered_closed_effect_releases_only_its_persisted_owner_and_epoch_lease() {
    let database_path = temporary_scheduler_database_path();
    let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    let task_ref = "task://personal/recovered-exact-lease";
    scheduler_repository
        .upsert(&scheduler_row(task_ref))
        .unwrap();
    scheduler_repository
        .acquire_lease(
            &scheduler_work_key(task_ref),
            "scheduler-worker",
            21,
            "2026-08-04T12:05:00Z",
        )
        .unwrap();

    release_closed_recovered_attempt(
        &recovered_closed_attempt(task_ref, 21),
        &mut scheduler_repository,
        "2026-08-04T12:01:00Z",
    )
    .unwrap();

    let row = scheduler_repository
        .load(&scheduler_work_key(task_ref))
        .unwrap()
        .unwrap();
    assert_eq!(row.state, SchedulerState::Succeeded.as_str());
    assert_eq!(row.lease_owner, None);
    drop(scheduler_repository);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn recovered_legacy_unbound_handoff_retains_its_scheduler_lease() {
    let database_path = temporary_scheduler_database_path();
    let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    let task_ref = "task://personal/recovered-legacy-unbound";
    scheduler_repository
        .upsert(&scheduler_row(task_ref))
        .unwrap();
    scheduler_repository
        .acquire_lease(
            &scheduler_work_key(task_ref),
            "scheduler-worker",
            23,
            "2026-08-04T12:05:00Z",
        )
        .unwrap();

    let mut recovered_attempt = recovered_closed_attempt(task_ref, 23);
    recovered_attempt.handoff.scheduler_lease = None;
    release_closed_recovered_attempt(
        &recovered_attempt,
        &mut scheduler_repository,
        "2026-08-04T12:01:00Z",
    )
    .unwrap();

    let row = scheduler_repository
        .load(&scheduler_work_key(task_ref))
        .unwrap()
        .unwrap();
    assert_eq!(row.state, SchedulerState::Leased.as_str());
    assert_eq!(row.lease_epoch, 23);
    assert_eq!(row.lease_owner.as_deref(), Some("scheduler-worker"));
    drop(scheduler_repository);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn recovered_closed_effect_cannot_release_a_successor_lease_epoch() {
    let database_path = temporary_scheduler_database_path();
    let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    let task_ref = "task://personal/recovered-stale-lease";
    scheduler_repository
        .upsert(&scheduler_row(task_ref))
        .unwrap();
    scheduler_repository
        .acquire_lease(
            &scheduler_work_key(task_ref),
            "scheduler-worker",
            22,
            "2026-08-04T12:05:00Z",
        )
        .unwrap();

    let result = release_closed_recovered_attempt(
        &recovered_closed_attempt(task_ref, 21),
        &mut scheduler_repository,
        "2026-08-04T12:01:00Z",
    );
    assert!(
        result.is_err(),
        "a recovered handoff cannot release a successor lease epoch"
    );
    let Err(error) = result else {
        return;
    };
    assert!(matches!(error, SchedulerAuthorityError::Repository(_)));
    let row = scheduler_repository
        .load(&scheduler_work_key(task_ref))
        .unwrap()
        .unwrap();
    assert_eq!(row.state, SchedulerState::Leased.as_str());
    assert_eq!(row.lease_epoch, 22);
    assert_eq!(row.lease_owner.as_deref(), Some("scheduler-worker"));
    drop(scheduler_repository);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn malformed_release_time_preserves_the_closed_effects_fenced_lease() {
    let database_path = temporary_scheduler_database_path();
    let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    let task_ref = "task://personal/malformed-release-time";
    scheduler_repository
        .upsert(&scheduler_row(task_ref))
        .unwrap();
    scheduler_repository
        .acquire_lease(
            &scheduler_work_key(task_ref),
            "scheduler-worker",
            11,
            "2026-08-03T00:00:00Z",
        )
        .unwrap();
    let dispatch = SchedulerDispatch {
        task_ref: task_ref.to_owned(),
        contract_epoch: 1,
        lease_owner: "scheduler-worker".to_owned(),
        lease_epoch: 11,
        lease_expires: "2026-08-03T00:01:00Z".to_owned(),
        attempt_count: 1,
    };

    assert!(matches!(
        complete_resolved_effect_and_release(
            SchedulerWorkerAttempt::EffectClosed(dispatch),
            &mut scheduler_repository,
            "not-a-timestamp",
        ),
        Err(SchedulerAuthorityError::InvalidReleaseTime(value)) if value == "not-a-timestamp"
    ));

    let durable_row = scheduler_repository
        .load(&scheduler_work_key(task_ref))
        .unwrap()
        .unwrap();
    assert_eq!(durable_row.state, SchedulerState::Leased.as_str());
    assert_eq!(durable_row.lease_owner.as_deref(), Some("scheduler-worker"));
    assert_eq!(durable_row.lease_epoch, 11);
    drop(scheduler_repository);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn stale_closed_effect_release_preserves_a_successor_fenced_lease() {
    let database_path = temporary_scheduler_database_path();
    let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    let task_ref = "task://personal/stale-closed-effect";
    scheduler_repository
        .upsert(&scheduler_row(task_ref))
        .unwrap();
    scheduler_repository
        .acquire_eligible_lease(
            &scheduler_work_key(task_ref),
            "scheduler-worker",
            12,
            "2026-08-03T00:00:00Z",
            "2026-08-03T00:00:30Z",
        )
        .unwrap();
    scheduler_repository
        .acquire_eligible_lease(
            &scheduler_work_key(task_ref),
            "scheduler-worker",
            13,
            "2026-08-03T00:00:30Z",
            "2026-08-03T00:01:30Z",
        )
        .unwrap();
    let stale_dispatch = SchedulerDispatch {
        task_ref: task_ref.to_owned(),
        contract_epoch: 1,
        lease_owner: "scheduler-worker".to_owned(),
        lease_epoch: 12,
        lease_expires: "2026-08-03T00:01:00Z".to_owned(),
        attempt_count: 1,
    };

    assert!(matches!(
        complete_resolved_effect_and_release(
            SchedulerWorkerAttempt::EffectClosed(stale_dispatch),
            &mut scheduler_repository,
            "2026-08-03T00:01:30Z",
        ),
        Err(SchedulerAuthorityError::Repository(_))
    ));

    let durable_row = scheduler_repository
        .load(&scheduler_work_key(task_ref))
        .unwrap()
        .unwrap();
    assert_eq!(durable_row.state, SchedulerState::Leased.as_str());
    assert_eq!(durable_row.lease_owner.as_deref(), Some("scheduler-worker"));
    assert_eq!(durable_row.lease_epoch, 13);
    drop(scheduler_repository);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn only_durable_terminal_effect_states_close_a_scheduler_attempt() {
    assert_eq!(
        classify_scheduler_effect_closure("RECONCILED").unwrap(),
        SchedulerEffectClosure::Closed
    );
    assert_eq!(
        classify_scheduler_effect_closure("EXECUTING").unwrap(),
        SchedulerEffectClosure::PendingReconciliation
    );
    assert_eq!(
        classify_scheduler_effect_closure("NOT_EXECUTED").unwrap(),
        SchedulerEffectClosure::PendingReconciliation
    );
    assert!(matches!(
        classify_scheduler_effect_closure("UNRECOGNIZED"),
        Err(SchedulerAuthorityError::UnsupportedEffectState(state)) if state == "UNRECOGNIZED"
    ));
}

#[test]
fn startup_recovery_repairs_only_missing_loop_without_duplicate_scheduler_work() {
    let database_path = temporary_scheduler_database_path();
    let authority_store = SqliteAuthorityStore::open(&database_path).unwrap();
    let task_ref = "task://personal/p2-t12-repair-loop";
    let (contract, loop_object_id, budget_id) =
        persist_repairable_task_contract(&authority_store, 1_200, task_ref);
    let budget_state = BudgetState::new(BTreeMap::from([
        ("semantic_calls".to_owned(), 1),
        ("tool_calls".to_owned(), 2),
    ]))
    .unwrap();
    authority_store
        .create_budget(
            &budget_id,
            &serde_json::to_string(&budget_state).unwrap(),
            &WallTimestamp::parse("2026-08-13T05:00:00Z").unwrap(),
        )
        .unwrap();
    let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    scheduler_repository
        .upsert(&scheduler_row(task_ref))
        .unwrap();
    assert!(
        authority_store
            .load_object(LifecycleDomain::Loop, &loop_object_id)
            .unwrap()
            .is_none()
    );

    super::reconcile_scheduler_recovery_with_store(&authority_store, &mut scheduler_repository)
        .unwrap();
    let repaired_loop = authority_store
        .load_object(LifecycleDomain::Loop, &loop_object_id)
        .unwrap()
        .unwrap();
    assert_eq!(repaired_loop.state.as_str(), "START");
    assert_eq!(
        authority_store
            .load_budget(&budget_id)
            .unwrap()
            .unwrap()
            .state,
        budget_state
    );
    assert_eq!(scheduler_repository.list_recoverable().unwrap().len(), 1);

    // Restarting recovery is idempotent: existing Budget/scheduler authority
    // is neither reset nor duplicated.
    super::reconcile_scheduler_recovery_with_store(&authority_store, &mut scheduler_repository)
        .unwrap();
    assert_eq!(scheduler_repository.list_recoverable().unwrap().len(), 1);
    assert_eq!(
        authority_store
            .load_task_contract(task_ref, 1)
            .unwrap()
            .unwrap(),
        contract
    );

    drop(scheduler_repository);
    drop(authority_store);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn startup_recovery_repairs_only_missing_budget_without_replacing_loop() {
    let database_path = temporary_scheduler_database_path();
    let authority_store = SqliteAuthorityStore::open(&database_path).unwrap();
    let task_ref = "task://personal/p2-t12-repair-budget";
    let (contract, loop_object_id, budget_id) =
        persist_repairable_task_contract(&authority_store, 1_300, task_ref);
    let bootstrap = prepared_repair_bootstrap(&authority_store, &contract);
    authority_store
        .admit_object(&bootstrap.loop_admission)
        .unwrap();
    let loop_before = authority_store
        .load_object(LifecycleDomain::Loop, &loop_object_id)
        .unwrap()
        .unwrap();
    let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    scheduler_repository
        .upsert(&scheduler_row(task_ref))
        .unwrap();
    assert!(authority_store.load_budget(&budget_id).unwrap().is_none());

    super::reconcile_scheduler_recovery_with_store(&authority_store, &mut scheduler_repository)
        .unwrap();
    assert_eq!(
        authority_store
            .load_object(LifecycleDomain::Loop, &loop_object_id)
            .unwrap()
            .unwrap(),
        loop_before,
        "startup repair must not replace an existing Loop"
    );
    let repaired_budget = authority_store.load_budget(&budget_id).unwrap().unwrap();
    assert_eq!(
        repaired_budget.state.remaining(),
        &BTreeMap::from([
            ("semantic_calls".to_owned(), 1),
            ("tool_calls".to_owned(), 2)
        ])
    );
    assert_eq!(scheduler_repository.list_recoverable().unwrap().len(), 1);

    super::reconcile_scheduler_recovery_with_store(&authority_store, &mut scheduler_repository)
        .unwrap();
    assert_eq!(
        authority_store
            .load_object(LifecycleDomain::Loop, &loop_object_id)
            .unwrap()
            .unwrap(),
        loop_before
    );
    assert_eq!(scheduler_repository.list_recoverable().unwrap().len(), 1);

    drop(scheduler_repository);
    drop(authority_store);
    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn shared_authority_store_drives_startup_recovery_and_private_tick() {
    let layout = temporary_personal_layout();
    layout.ensure_directories().unwrap();
    prepare_personal_databases(&layout).unwrap();
    let database_path = layout.authority_database_path();
    let authority_store = SqliteAuthorityStore::open(&database_path).unwrap();
    let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
    let executor_router =
        ProductionNativeToolExecutorRouter::open(1, layout.data_dir().join("workspace")).unwrap();
    let artifact_store =
        cognitive_store::ArtifactStore::open(layout.data_dir().join("artifacts"), 1024 * 1024)
            .unwrap();

    super::reconcile_scheduler_recovery_with_store(&authority_store, &mut scheduler_repository)
        .unwrap();
    super::run_private_scheduler_tick_with_store(
        &authority_store,
        &mut scheduler_repository,
        layout.config_dir(),
        &executor_router,
        &artifact_store,
    )
    .unwrap();

    // A second recovery+tick pass on the same open store must remain fail-closed
    // for empty work without requiring another SqliteAuthorityStore::open.
    super::reconcile_scheduler_recovery_with_store(&authority_store, &mut scheduler_repository)
        .unwrap();
    super::run_private_scheduler_tick_with_store(
        &authority_store,
        &mut scheduler_repository,
        layout.config_dir(),
        &executor_router,
        &artifact_store,
    )
    .unwrap();
}
