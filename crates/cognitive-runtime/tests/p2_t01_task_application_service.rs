//! P2-T01 failure-first behavior: TaskApplicationService exposes the
//! L5 task lifecycle over the L3/L4 intent-chain kernel. These tests
//! verify the acceptance invariants:
//!
//! - raw intent is durably fixed BEFORE any interpretation or contract
//!   (a fresh store reopened after `propose` can read the record);
//! - the admission preview digest binds the admit call (digest mismatch
//!   is refused before any kernel mutation);
//! - supersession mints a new epoch and `verify_task_binding_current`
//!   fences the old binding;
//! - a stale writer lease is refused.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_contracts::generated::common_defs::Digest;
use cognitive_contracts::generated::governed_object_header::GovernedObjectHeaderSensitivity;
use cognitive_contracts::generated::object_reference::{StrongReference, StrongReferenceKind};
use cognitive_contracts::generated::task_contract::ContractConditionKind;
use cognitive_domain::{LifecycleDomain, ObjectId, UriRef, WallTimestamp};
use cognitive_kernel::effects::WriterLease;
use cognitive_kernel::intent_chain::{
    AcceptanceCommand, AmbiguityFact, ConditionSpec, GovernanceSeed, InterpretationCandidate,
    SupersedeCommand, TaskContractCommand, UserIntentCommand, verify_task_binding_current,
};
use cognitive_kernel::ports::{
    AuthorityStore, Clock, IdGenerator, PortFailure, ProtocolStore, TaskBinding,
};
use cognitive_management::{KernelTaskApplicationService, TaskApplicationService};
use cognitive_store::{
    SqliteAuthorityStore,
    scheduler::{SchedulerRepository, SchedulerState, SchedulerWorkKey},
};
use std::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------
// Deterministic infrastructure helpers
// ---------------------------------------------------------------------

struct FixedClock(WallTimestamp);

impl FixedClock {
    fn new() -> Self {
        Self(WallTimestamp::parse("2026-08-01T12:00:00Z").unwrap())
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        Ok(self.0.clone())
    }
}

struct SeqIds(AtomicU64);

impl SeqIds {
    fn new() -> Self {
        Self(AtomicU64::new(1))
    }
}

impl IdGenerator for SeqIds {
    fn next_uuid_v7(&self) -> Result<String, PortFailure> {
        let n = self.0.fetch_add(1, Ordering::SeqCst);
        Ok(format!("00000000-0000-7000-8000-{n:012x}"))
    }
}

fn uri(text: &str) -> UriRef {
    UriRef::parse(text).unwrap()
}

fn oid(n: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{n:012x}")).unwrap()
}

fn evidence_ref(n: u64) -> StrongReference {
    StrongReference {
        content_digest: Digest(format!(
            "sha256:{}",
            format!("{n:x}").repeat(64)[..64].to_owned()
        )),
        id: cognitive_contracts::generated::object_reference::UuidV7(format!(
            "00000000-0000-7000-a000-{n:012x}"
        )),
        kind: StrongReferenceKind::Strong,
        object_version: 1,
    }
}

fn seed() -> GovernanceSeed {
    GovernanceSeed {
        owner: evidence_ref(9001),
        authority: evidence_ref(9002),
        resource_scope: evidence_ref(9003),
        tenant_id: Some("00000000-0000-7000-9000-0000000000f1".to_owned()),
        created_by: "principal://tenant-a/user-1".to_owned(),
        sensitivity: GovernedObjectHeaderSensitivity::Internal,
        purpose_constraints: vec!["task_execution".to_owned()],
        retention_policy: "standard".to_owned(),
    }
}

fn intent_cmd(record_n: u64, expression: &str) -> UserIntentCommand {
    UserIntentCommand {
        record_id: oid(1000 * record_n),
        actor_chain_digest: format!("sha256:{}", "aa11".repeat(16)),
        conversation_or_scope_ref: uri("conversation://tenant-a/thread-1"),
        input_refs: vec![uri("state://tenant-a/attachments/spec-v1")],
        raw_expression: expression.to_owned(),
        intent_authority_ref: uri("principal://tenant-a/user-1"),
        governance: seed(),
        correlation_id: uri("corr://tenant-a/p2-t01"),
    }
}

fn clean_candidate(interp_n: u64) -> InterpretationCandidate {
    InterpretationCandidate {
        interpretation_id: oid(2000 * interp_n),
        objectives: vec!["roll out service v2 to staging".to_owned()],
        constraints: vec!["no production changes".to_owned()],
        forbidden: vec!["deleting user data".to_owned()],
        assumptions: vec!["staging cluster is reachable".to_owned()],
        ambiguities: vec![AmbiguityFact {
            id: "amb-cosmetic".to_owned(),
            material: false,
            question: "prefer blue or green deployment naming?".to_owned(),
        }],
        information_gaps: vec![],
        supersedes: None,
    }
}

fn contract_cmd(contract_n: u64, task_ref: &str) -> TaskContractCommand {
    TaskContractCommand {
        contract_id: oid(3000 * contract_n),
        task_ref: uri(task_ref),
        objective: "staging rollout of service v2".to_owned(),
        in_scope: vec!["staging deployment".to_owned()],
        out_of_scope: vec!["production".to_owned()],
        conditions: vec![ConditionSpec {
            id: "acc-1".to_owned(),
            kind: ContractConditionKind::Acceptance,
            description: "service v2 healthy in staging per verifier".to_owned(),
            verifier_ref: Some("verifier://tenant-a/http-health".to_owned()),
        }],
        budget: cognitive_contracts::generated::common_defs::Budget {
            attention_slots: None,
            context_bytes: None,
            egress_bytes: None,
            input_tokens: None,
            money_microunits: None,
            output_tokens: None,
            semantic_calls: None,
            tool_calls: Some(50),
            wall_time_ms: None,
        },
        max_iterations: 8,
        max_retries: 3,
        deadline: WallTimestamp::parse("2026-08-02T12:00:00Z").unwrap(),
        loop_object_id: oid(contract_n + 100),
        budget_id: cognitive_domain::BudgetId::parse(&format!(
            "00000000-0000-7000-b000-{contract_n:012x}"
        ))
        .unwrap(),
        allowed_state_domains: vec!["task".to_owned(), "effect".to_owned()],
        allowed_tools: vec!["operation://tenant-a/payments/refund".to_owned()],
        context_request_ref: None,
        governance: seed(),
        correlation_id: uri("corr://tenant-a/p2-t01"),
    }
}

fn lease(epoch: i64) -> WriterLease {
    WriterLease { epoch }
}

fn open_store(dir: &tempfile::TempDir) -> SqliteAuthorityStore {
    SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap()
}

type Service = KernelTaskApplicationService<SqliteAuthorityStore, FixedClock, SeqIds>;

fn make_service(dir: &tempfile::TempDir) -> Service {
    KernelTaskApplicationService::new(open_store(dir), FixedClock::new(), SeqIds::new())
}

// ---------------------------------------------------------------------
// 1. raw intent persists before any interpretation or contract
// ---------------------------------------------------------------------

#[test]
fn proposal_persists_raw_intent_before_any_interpretation_or_task_contract() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = make_service(&dir);

    let record = service
        .propose(&lease(1), &intent_cmd(1, "roll out service v2 to staging"))
        .unwrap();
    assert_eq!(record.raw_expression, "roll out service v2 to staging");
    assert!(!record.intent_digest.is_empty());

    // Crash + restart: reopen the same DB file and verify the raw intent
    // survived durably.
    drop(service);
    let mut service2 = make_service(&dir);
    let loaded = service2.query_intent(&oid(1000)).unwrap();
    assert!(loaded.is_some());
    assert_eq!(
        loaded.unwrap().raw_expression,
        "roll out service v2 to staging"
    );

    // The kernel ordering gate refuses an interpretation before a record;
    // there is no record 999, so clarify must fail closed.
    let result = service2.clarify(
        &lease(1),
        &oid(999),
        &clean_candidate(1),
        &seed(),
        &uri("corr://tenant-a/p2-t01"),
    );
    assert!(
        result.is_err(),
        "interpretation without a durable record must be refused"
    );
}

// ---------------------------------------------------------------------
// 2. preview digest binds the admission
// ---------------------------------------------------------------------

#[test]
fn preview_digest_mismatch_is_refused_before_any_kernel_mutation() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = make_service(&dir);

    let record = service
        .propose(&lease(1), &intent_cmd(2, "roll out service v2 to staging"))
        .unwrap();
    let interpretation = service
        .clarify(
            &lease(1),
            &record.record_id,
            &clean_candidate(2),
            &seed(),
            &uri("corr://tenant-a/p2-t01"),
        )
        .unwrap();

    let preview = service
        .preview(&contract_cmd(2, "task://tenant-a/rollout-v2"))
        .unwrap();
    assert!(!preview.preview_digest.is_empty());
    assert_eq!(preview.objective, "staging rollout of service v2");
    assert_eq!(preview.condition_count, 1);
    assert_eq!(preview.tool_calls_frozen, Some(50));

    // Correct acceptance bound to the persisted digest + preview digest.
    let acceptance = AcceptanceCommand {
        interpretation_id: interpretation.interpretation_id.clone(),
        accepted_by: uri("principal://tenant-a/user-1"),
        accepted_digest: interpretation.interpretation_digest.clone(),
    };
    let contract = service
        .admit(
            &lease(1),
            &preview.preview_digest,
            &acceptance,
            &contract_cmd(2, "task://tenant-a/rollout-v2"),
            0,
        )
        .unwrap();
    assert_eq!(contract.contract_epoch, 1);

    // A mismatched preview digest is refused; the task has no contract.
    let dir2 = tempfile::tempdir().unwrap();
    let mut service2 = make_service(&dir2);
    let record2 = service2
        .propose(&lease(1), &intent_cmd(3, "roll out service v2 to staging"))
        .unwrap();
    let interp2 = service2
        .clarify(
            &lease(1),
            &record2.record_id,
            &clean_candidate(3),
            &seed(),
            &uri("corr://tenant-a/p2-t01"),
        )
        .unwrap();
    let acceptance2 = AcceptanceCommand {
        interpretation_id: interp2.interpretation_id.clone(),
        accepted_by: uri("principal://tenant-a/user-1"),
        accepted_digest: interp2.interpretation_digest.clone(),
    };
    let bad_digest = format!("sha256:{}", "00".repeat(32));
    let result = service2.admit(
        &lease(1),
        &bad_digest,
        &acceptance2,
        &contract_cmd(3, "task://tenant-a/rollout-v2"),
        0,
    );
    assert!(matches!(
        result,
        Err(cognitive_management::TaskApplicationError::PreviewDigestMismatch)
    ));
    assert_eq!(
        service2
            .store()
            .current_contract_epoch("task://tenant-a/rollout-v2")
            .unwrap(),
        0
    );
}

// ---------------------------------------------------------------------
// 3. supersession mints a new epoch and fences the old binding
// ---------------------------------------------------------------------

#[test]
fn supersede_mints_new_epoch_and_fences_old_binding() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = make_service(&dir);

    let record = service
        .propose(&lease(1), &intent_cmd(4, "roll out service v2 to staging"))
        .unwrap();
    let interpretation = service
        .clarify(
            &lease(1),
            &record.record_id,
            &clean_candidate(4),
            &seed(),
            &uri("corr://tenant-a/p2-t01"),
        )
        .unwrap();
    let preview = service
        .preview(&contract_cmd(4, "task://tenant-a/rollout-v2"))
        .unwrap();
    let contract = service
        .admit(
            &lease(1),
            &preview.preview_digest,
            &AcceptanceCommand {
                interpretation_id: interpretation.interpretation_id.clone(),
                accepted_by: uri("principal://tenant-a/user-1"),
                accepted_digest: interpretation.interpretation_digest.clone(),
            },
            &contract_cmd(4, "task://tenant-a/rollout-v2"),
            0,
        )
        .unwrap();
    assert_eq!(contract.contract_epoch, 1);

    // A user correction: fix a NEW record + candidate, then supersede.
    let record2 = service
        .propose(&lease(1), &intent_cmd(5, "roll out service v3 to staging"))
        .unwrap();
    let mut superseding_candidate = clean_candidate(5);
    superseding_candidate.supersedes = Some(interpretation.interpretation_id.clone());
    let interp2 = service
        .clarify(
            &lease(1),
            &record2.record_id,
            &superseding_candidate,
            &seed(),
            &uri("corr://tenant-a/p2-t01"),
        )
        .unwrap();

    let report = service
        .control(
            &lease(1),
            &SupersedeCommand {
                acceptance: AcceptanceCommand {
                    interpretation_id: interp2.interpretation_id.clone(),
                    accepted_by: uri("principal://tenant-a/user-1"),
                    accepted_digest: interp2.interpretation_digest.clone(),
                },
                contract: contract_cmd(5, "task://tenant-a/rollout-v2"),
                expected_current_epoch: 1,
            },
        )
        .unwrap();

    // The authoritative epoch is now 2; an old-epoch binding is fenced.
    assert_eq!(
        service
            .store()
            .current_contract_epoch("task://tenant-a/rollout-v2")
            .unwrap(),
        2
    );
    assert_eq!(report.new_contract.contract_epoch, 2);
    let stale_binding = TaskBinding {
        task_ref: "task://tenant-a/rollout-v2".to_owned(),
        contract_epoch: 1,
    };
    assert!(verify_task_binding_current(service.store(), &stale_binding).is_err());
    let current_binding = TaskBinding {
        task_ref: "task://tenant-a/rollout-v2".to_owned(),
        contract_epoch: 2,
    };
    assert!(verify_task_binding_current(service.store(), &current_binding).is_ok());
}

// ---------------------------------------------------------------------
// 4. stale writer lease is refused
// ---------------------------------------------------------------------

#[test]
fn stale_writer_lease_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = make_service(&dir);

    // The current fencing epoch is 1 after the first durable write.
    let record = service
        .propose(&lease(1), &intent_cmd(6, "roll out service v2 to staging"))
        .unwrap();

    // A stale lease (epoch 0) is refused by the kernel fencing check
    // before any mutation.
    let result = service.clarify(
        &lease(0),
        &record.record_id,
        &clean_candidate(6),
        &seed(),
        &uri("corr://tenant-a/p2-t01"),
    );
    assert!(result.is_err(), "stale writer lease must be refused");
}

// ---------------------------------------------------------------------
// P2-T12/D01 failure-first: admission publishes runnable work atomically
// ---------------------------------------------------------------------

#[test]
fn admit_atomically_publishes_runnable_scheduler_work_and_authority_prerequisites() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = make_service(&dir);
    let record = service
        .propose(&lease(1), &intent_cmd(7, "read the governed workspace"))
        .unwrap();
    let interpretation = service
        .clarify(
            &lease(1),
            &record.record_id,
            &clean_candidate(7),
            &seed(),
            &uri("corr://tenant-a/p2-t12"),
        )
        .unwrap();
    let command = contract_cmd(7, "task://tenant-a/read-workspace");
    let preview = service.preview(&command).unwrap();
    let contract = service
        .admit(
            &lease(1),
            &preview.preview_digest,
            &AcceptanceCommand {
                interpretation_id: interpretation.interpretation_id,
                accepted_by: uri("principal://tenant-a/user-1"),
                accepted_digest: interpretation.interpretation_digest,
            },
            &command,
            0,
        )
        .unwrap();
    assert_eq!(contract.contract_epoch, 1);

    // Model a crash immediately after the successful admission response.
    // Reopening the durable database must reveal one indivisible publication:
    // contract + runnable scheduler row + START Loop + hard Budget.
    drop(service);
    let reopened = open_store(&dir);
    let mut scheduler =
        SchedulerRepository::open(&dir.path().join("authority.db")).expect("reopen scheduler");
    let scheduler_row = scheduler
        .load(&SchedulerWorkKey {
            task_ref: command.task_ref.as_str().to_owned(),
            contract_epoch: contract.contract_epoch,
        })
        .expect("load scheduler work")
        .expect("admission must publish runnable scheduler work");
    assert_eq!(scheduler_row.state, SchedulerState::Runnable.as_str());
    assert_eq!(scheduler_row.lease_owner, None);
    assert_eq!(scheduler_row.lease_epoch, 0);
    assert_eq!(scheduler_row.attempt_count, 0);

    let loop_object = reopened
        .load_object(LifecycleDomain::Loop, &command.loop_object_id)
        .expect("load admitted Loop")
        .expect("admission must publish its contract-named Loop");
    assert_eq!(loop_object.state.as_str(), "START");
    assert_eq!(loop_object.version.get(), 1);

    let budget = reopened
        .load_budget(&command.budget_id)
        .expect("load admitted Budget")
        .expect("admission must publish its contract-named Budget");
    assert_eq!(budget.state.remaining().get("tool_calls"), Some(&50));
}
