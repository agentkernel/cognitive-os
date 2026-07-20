//! M5 bounded-loop kernel ports behavior (REQ-RUN-004/005/007/008):
//! contract-pinned loop start, hard-precondition iteration gate with
//! same-transaction budget debit, typed progress facts, deterministic
//! stagnation/retry arithmetic — against the real kernel gate and SQLite
//! WAL authority store. The OODA phase orchestration itself is Lane-RUN;
//! these tests freeze the kernel port surface it consumes.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "m4_common/mod.rs"]
mod m4_common;

use cognitive_contracts::generated::governed_object_header::GovernedObjectHeaderSensitivity;
use cognitive_contracts::generated::task_contract::ContractConditionKind;
use cognitive_domain::{BudgetId, LifecycleDomain, ObjectId, Version};
use cognitive_kernel::budget::{BudgetCharge, BudgetState};
use cognitive_kernel::effects::EffectError;
use cognitive_kernel::harness::{LoopDriver, ProgressStatus};
use cognitive_kernel::intent_chain::{
    AcceptanceCommand, AmbiguityFact, ConditionSpec, GovernanceSeed, InterpretationCandidate,
    TaskContractCommand, UserIntentCommand, admit_interpretation, mint_task_contract,
    record_interpretation_candidate, record_user_intent,
};
use cognitive_kernel::ports::{AuthorityStore, CheckpointRow, ProtocolStore};
use cognitive_kernel::{RejectionKind, TransitionEngine};
use cognitive_store::SqliteAuthorityStore;
use m4_common::*;
use std::collections::BTreeMap;

fn fresh_store(dir: &tempfile::TempDir) -> SqliteAuthorityStore {
    SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap()
}

fn seed() -> GovernanceSeed {
    GovernanceSeed {
        owner: evidence_ref(9101),
        authority: evidence_ref(9102),
        resource_scope: evidence_ref(9103),
        tenant_id: Some("00000000-0000-7000-9000-0000000000f2".to_owned()),
        created_by: "principal://tenant-a/user-1".to_owned(),
        sensitivity: GovernedObjectHeaderSensitivity::Internal,
        purpose_constraints: vec!["task_execution".to_owned()],
        retention_policy: "standard".to_owned(),
    }
}

/// Chain a contract for `task_ref` with the given hard ceilings.
fn mint_contract(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    base_n: u64,
    task_ref: &str,
    max_iterations: i64,
    max_retries: i64,
) {
    let record = record_user_intent(
        store,
        clock,
        ids,
        &lease(1),
        &UserIntentCommand {
            record_id: oid(base_n),
            actor_chain_digest: format!("sha256:{}", "aa11".repeat(16)),
            conversation_or_scope_ref: uri("conversation://tenant-a/loop-thread"),
            input_refs: vec![],
            raw_expression: "run the bounded rollout loop".to_owned(),
            intent_authority_ref: uri("principal://tenant-a/user-1"),
            governance: seed(),
            correlation_id: uri("corr://tenant-a/m5-loop"),
        },
    )
    .unwrap();
    let interpretation = record_interpretation_candidate(
        store,
        clock,
        ids,
        &lease(1),
        &record.record_id,
        &InterpretationCandidate {
            interpretation_id: oid(base_n + 1),
            objectives: vec!["bounded rollout".to_owned()],
            constraints: vec![],
            forbidden: vec![],
            assumptions: vec![],
            ambiguities: vec![AmbiguityFact {
                id: "amb-0".to_owned(),
                material: false,
                question: "naming only".to_owned(),
            }],
            information_gaps: vec![],
            supersedes: None,
        },
        &seed(),
        &uri("corr://tenant-a/m5-loop"),
    )
    .unwrap();
    let admitted = admit_interpretation(
        store,
        &AcceptanceCommand {
            interpretation_id: interpretation.interpretation_id.clone(),
            accepted_by: uri("principal://tenant-a/user-1"),
            accepted_digest: interpretation.interpretation_digest.clone(),
        },
    )
    .unwrap();
    mint_task_contract(
        store,
        clock,
        ids,
        &lease(1),
        &admitted,
        &TaskContractCommand {
            contract_id: oid(base_n + 2),
            task_ref: uri(task_ref),
            objective: "bounded rollout".to_owned(),
            in_scope: vec!["staging".to_owned()],
            out_of_scope: vec![],
            conditions: vec![ConditionSpec {
                id: "acc-1".to_owned(),
                kind: ContractConditionKind::Acceptance,
                description: "verifier passes".to_owned(),
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
                tool_calls: Some(10),
                wall_time_ms: None,
            },
            max_iterations,
            max_retries,
            allowed_state_domains: vec!["task".to_owned(), "effect".to_owned()],
            allowed_tools: vec![],
            governance: seed(),
            correlation_id: uri("corr://tenant-a/m5-loop"),
        },
        0,
    )
    .unwrap();
}

fn budget_id(n: u64) -> BudgetId {
    BudgetId::parse(&format!("00000000-0000-7000-b000-{n:012x}")).unwrap()
}

fn create_budget(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    id: &BudgetId,
    tool_calls: i64,
) {
    let engine = TransitionEngine::new(store, clock, ids);
    let state = BudgetState::new(BTreeMap::from([("tool_calls".to_owned(), tool_calls)])).unwrap();
    engine.create_budget(id, &state).unwrap();
}

fn charge(tool_calls: i64) -> BudgetCharge {
    BudgetCharge::new(BTreeMap::from([("tool_calls".to_owned(), tool_calls)])).unwrap()
}

fn driver<'a>(
    store: &'a SqliteAuthorityStore,
    clock: &'a FixedClock,
    ids: &'a SeqIds,
) -> LoopDriver<'a, SqliteAuthorityStore, FixedClock, SeqIds> {
    LoopDriver::new(
        store,
        clock,
        ids,
        uri("actor://tenant-a/agent-1"),
        uri("authority://tenant-a/state-authority"),
        uri("corr://tenant-a/m5-loop"),
    )
}

fn checkpoint_row(n: u64, loop_id: &ObjectId, watermark: i64, epoch: i64) -> CheckpointRow {
    CheckpointRow {
        checkpoint_id: oid(n),
        loop_object_id: loop_id.clone(),
        event_high_watermark: watermark,
        fencing_epoch: epoch,
        canonical_json: format!("{{\"iteration_watermark\":{watermark}}}"),
    }
}

fn denial_code(err: &EffectError) -> &'static str {
    match err {
        EffectError::Denied(denial) => denial.registered.code,
        other => panic!("expected protocol denial, got {other:?}"),
    }
}

/// Drive one loop from OBSERVE around the OODA cycle to CONTINUE using the
/// registered table edges (guards attested as fixture facts — the phase
/// orchestration itself is Lane-RUN's).
fn drive_observe_to_continue(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    loop_id: &ObjectId,
    from_version: Version,
) -> Version {
    let mut v = from_version;
    for (from, to, reason) in [
        ("OBSERVE", "RESOLVE", "EVIDENCE_OBSERVED"),
        ("RESOLVE", "ORIENT", "CONTEXT_COMPLETE"),
        ("ORIENT", "DECIDE", "ORIENTATION_COMPLETE"),
        ("DECIDE", "ACT", "OPERATION_ADMITTED"),
        ("ACT", "VERIFY", "PROGRESS_CLAIMED"),
        ("VERIFY", "CONTINUE", "PROGRESS_VERIFIED"),
    ] {
        v = drive(
            store,
            clock,
            ids,
            LifecycleDomain::Loop,
            loop_id,
            from,
            to,
            reason,
            v,
            None,
        );
    }
    v
}

/// REQ-RUN-004: a loop cannot start uncontracted; with a pinned durable
/// contract and a live hard budget it starts through the registered
/// START -> OBSERVE edge.
#[test]
fn start_loop_requires_pinned_contract_and_live_budget() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let harness = driver(&store, &clock, &ids);
    let loop_id = oid(400);
    admit(&store, &clock, &ids, &loop_id, LifecycleDomain::Loop, None);
    let budget = budget_id(400);
    create_budget(&store, &clock, &ids, &budget, 10);

    // No contract: refused before any gate consultation.
    let uncontracted = harness
        .start_loop(
            &loop_id,
            Version::INITIAL,
            "task://tenant-a/loop-task",
            &budget,
            &lease(1),
        )
        .expect_err("a loop cannot run uncontracted");
    assert_eq!(denial_code(&uncontracted), "STATE_CONFLICT");

    mint_contract(&store, &clock, &ids, 410, "task://tenant-a/loop-task", 8, 3);

    // A drained budget refuses the start (loop_budget_available guard).
    let drained = budget_id(401);
    create_budget(&store, &clock, &ids, &drained, 0);
    let refused = harness
        .start_loop(
            &loop_id,
            Version::INITIAL,
            "task://tenant-a/loop-task",
            &drained,
            &lease(1),
        )
        .expect_err("a drained budget cannot admit a loop start");
    match refused {
        EffectError::Rejected(rejection) => {
            assert_eq!(rejection.kind, RejectionKind::GuardUnsatisfied);
        }
        other => panic!("unexpected {other:?}"),
    }

    let started = harness
        .start_loop(
            &loop_id,
            Version::INITIAL,
            "task://tenant-a/loop-task",
            &budget,
            &lease(1),
        )
        .unwrap();
    assert_eq!(started.after_version.get(), 2);
    let stored = store
        .load_object(LifecycleDomain::Loop, &loop_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "OBSERVE");
}

/// REQ-RUN-005 + REQ-RES-001: the iteration gate checks hard
/// preconditions each cycle and fails closed — monotonic accounting,
/// checkpoint-bound continuation, iteration ceiling as a registered
/// hard-limit denial, budget debit in the SAME transaction.
#[test]
fn begin_iteration_enforces_hard_preconditions_and_debits_budget() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let harness = driver(&store, &clock, &ids);
    let task_ref = "task://tenant-a/loop-task-2";
    mint_contract(&store, &clock, &ids, 420, task_ref, 2, 3);
    let loop_id = oid(430);
    admit(&store, &clock, &ids, &loop_id, LifecycleDomain::Loop, None);
    let budget = budget_id(430);
    create_budget(&store, &clock, &ids, &budget, 10);

    let started = harness
        .start_loop(&loop_id, Version::INITIAL, task_ref, &budget, &lease(1))
        .unwrap();
    // Iteration 1 runs: record its progress fact, then drive the loop
    // around the cycle to CONTINUE.
    harness
        .record_progress(
            &loop_id,
            1,
            ProgressStatus::Advanced,
            "fp-deploy",
            &["event://tenant-a/deploy-1".to_owned()],
            &lease(1),
        )
        .unwrap();
    let v = drive_observe_to_continue(&store, &clock, &ids, &loop_id, started.after_version);

    // No checkpoint yet: the continuation is refused (REQ-RUN-006 facts
    // must exist before the next iteration).
    let no_checkpoint = harness
        .begin_iteration(&loop_id, v, task_ref, 2, &budget, &charge(1), &lease(1))
        .expect_err("continuation without a durable checkpoint is refused");
    assert_eq!(denial_code(&no_checkpoint), "STATE_CONFLICT");

    store
        .append_checkpoint(&checkpoint_row(440, &loop_id, 1, 1))
        .unwrap();

    // Iteration accounting is monotonic: skipping to 3 is refused.
    let skipped = harness
        .begin_iteration(&loop_id, v, task_ref, 3, &budget, &charge(1), &lease(1))
        .expect_err("iteration accounting must be monotonic");
    assert_eq!(denial_code(&skipped), "STATE_CONFLICT");

    // The gate admits iteration 2 and debits the budget in the same
    // transaction.
    let before = store.load_budget(&budget).unwrap().unwrap();
    let committed = harness
        .begin_iteration(&loop_id, v, task_ref, 2, &budget, &charge(3), &lease(1))
        .unwrap();
    let after = store.load_budget(&budget).unwrap().unwrap();
    assert_eq!(
        after.state.remaining()["tool_calls"],
        before.state.remaining()["tool_calls"] - 3
    );
    let stored = store
        .load_object(LifecycleDomain::Loop, &loop_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "OBSERVE");

    // Iteration 2 records no progress; the loop cycles back to CONTINUE.
    harness
        .record_progress(
            &loop_id,
            2,
            ProgressStatus::NoProgress,
            "fp-deploy",
            &[],
            &lease(1),
        )
        .unwrap();
    let v3 = drive_observe_to_continue(&store, &clock, &ids, &loop_id, committed.after_version);

    // Iteration 3 exceeds max_iterations=2: the registered hard-limit
    // code, BEFORE any transition or debit (the loop stops or escalates,
    // it does not spin).
    let before = store.load_budget(&budget).unwrap().unwrap();
    let ceiling = harness
        .begin_iteration(&loop_id, v3, task_ref, 3, &budget, &charge(1), &lease(1))
        .expect_err("the iteration ceiling is a hard limit");
    assert_eq!(denial_code(&ceiling), "RESOURCE_BUDGET_EXHAUSTED");
    let after = store.load_budget(&budget).unwrap().unwrap();
    assert_eq!(after.state.remaining(), before.state.remaining());
    let still = store
        .load_object(LifecycleDomain::Loop, &loop_id)
        .unwrap()
        .unwrap();
    assert_eq!(still.state.as_str(), "CONTINUE", "no transition committed");
}

/// An over-budget charge fails closed inside the engine: no debit, no
/// transition, the registered exhaustion code (REQ-RES-001 consumption
/// point).
#[test]
fn iteration_charge_over_budget_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let harness = driver(&store, &clock, &ids);
    let task_ref = "task://tenant-a/loop-task-3";
    mint_contract(&store, &clock, &ids, 450, task_ref, 8, 3);
    let loop_id = oid(460);
    admit(&store, &clock, &ids, &loop_id, LifecycleDomain::Loop, None);
    let budget = budget_id(460);
    create_budget(&store, &clock, &ids, &budget, 2);

    let started = harness
        .start_loop(&loop_id, Version::INITIAL, task_ref, &budget, &lease(1))
        .unwrap();
    harness
        .record_progress(
            &loop_id,
            1,
            ProgressStatus::Advanced,
            "fp-a",
            &["event://tenant-a/e1".to_owned()],
            &lease(1),
        )
        .unwrap();
    let v = drive_observe_to_continue(&store, &clock, &ids, &loop_id, started.after_version);
    store
        .append_checkpoint(&checkpoint_row(470, &loop_id, 1, 1))
        .unwrap();

    let over = harness
        .begin_iteration(&loop_id, v, task_ref, 2, &budget, &charge(3), &lease(1))
        .expect_err("a charge beyond the ledger must fail closed");
    match over {
        EffectError::Rejected(rejection) => {
            assert_eq!(rejection.registered().code, "RESOURCE_BUDGET_EXHAUSTED");
        }
        other => panic!("unexpected {other:?}"),
    }
    let untouched = store.load_budget(&budget).unwrap().unwrap();
    assert_eq!(untouched.state.remaining()["tool_calls"], 2);
    let still = store
        .load_object(LifecycleDomain::Loop, &loop_id)
        .unwrap()
        .unwrap();
    assert_eq!(still.state.as_str(), "CONTINUE");
}

/// REQ-RUN-007 + REQ-RUN-008: progress facts are typed and evidence-bound
/// (a bare "advanced" claim is unrecordable); stagnation and retry
/// arithmetic fold deterministically over the durable rows; the retry
/// bound denies with the registered hard-limit code; stale writers cannot
/// poison the counters.
#[test]
fn progress_facts_stagnation_and_retry_bounds_are_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let harness = driver(&store, &clock, &ids);
    let task_ref = "task://tenant-a/loop-task-4";
    mint_contract(&store, &clock, &ids, 480, task_ref, 8, 3);
    let loop_id = oid(490);
    admit(&store, &clock, &ids, &loop_id, LifecycleDomain::Loop, None);

    // A progress claim without evidence is not recordable.
    let bare_claim = harness
        .record_progress(
            &loop_id,
            1,
            ProgressStatus::Advanced,
            "fp-x",
            &[],
            &lease(1),
        )
        .expect_err("advanced without evidence is a bare claim, not progress");
    assert_eq!(denial_code(&bare_claim), "STATE_CONFLICT");

    harness
        .record_progress(
            &loop_id,
            1,
            ProgressStatus::Advanced,
            "fp-setup",
            &["event://tenant-a/e1".to_owned()],
            &lease(1),
        )
        .unwrap();
    harness
        .record_progress(
            &loop_id,
            2,
            ProgressStatus::NoProgress,
            "fp-x",
            &[],
            &lease(1),
        )
        .unwrap();
    harness
        .record_progress(
            &loop_id,
            3,
            ProgressStatus::NoProgress,
            "fp-x",
            &[],
            &lease(1),
        )
        .unwrap();
    harness
        .record_progress(
            &loop_id,
            4,
            ProgressStatus::Uncertain,
            "fp-x",
            &[],
            &lease(1),
        )
        .unwrap();

    // Stagnation facts are a pure fold over the durable rows.
    let stagnation = harness.stagnation(&loop_id).unwrap();
    assert_eq!(stagnation.consecutive_without_progress, 3);
    assert_eq!(stagnation.last_advanced_iteration, Some(1));
    assert_eq!(stagnation.recorded_iterations, 4);

    // Retry arithmetic is fingerprint-scoped (REQ-RUN-008).
    assert_eq!(harness.retry_count(&loop_id, "fp-x").unwrap(), 3);
    assert_eq!(harness.retry_count(&loop_id, "fp-setup").unwrap(), 0);
    let facts = harness.contract_facts(task_ref).unwrap();
    assert_eq!(facts.max_retries, 3);
    let denied = harness
        .admit_retry(&facts, 3)
        .expect_err("the retry bound is a hard limit");
    assert_eq!(denied.registered.code, "RESOURCE_BUDGET_EXHAUSTED");
    harness.admit_retry(&facts, 2).unwrap();

    // Duplicate iteration accounting is refused (monotonic rule at the
    // driver, UNIQUE constraint as the structural backstop).
    let replay = harness
        .record_progress(
            &loop_id,
            4,
            ProgressStatus::NoProgress,
            "fp-x",
            &[],
            &lease(1),
        )
        .expect_err("iteration facts are recorded once");
    assert_eq!(denial_code(&replay), "STATE_CONFLICT");

    // A stale writer (pre-recovery epoch) cannot append facts: the store
    // re-verifies the fencing epoch INSIDE the transaction.
    store.advance_fencing_epoch().unwrap();
    let stale = harness
        .record_progress(
            &loop_id,
            5,
            ProgressStatus::NoProgress,
            "fp-x",
            &[],
            &lease(1),
        )
        .expect_err("stale writers cannot poison the stagnation counters");
    assert!(matches!(
        stale,
        EffectError::Denied(_) | EffectError::Rejected(_)
    ));
    assert_eq!(
        harness.stagnation(&loop_id).unwrap().recorded_iterations,
        4,
        "no fact appended by the fenced writer"
    );
}

/// The iteration close (VERIFY -> CONTINUE) binds a verification report
/// and reloads the governed task: a COMPLETED task admits no further
/// iterations (task_not_accepted guard fails closed).
#[test]
fn end_iteration_reloads_task_state_and_binds_verification() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let harness = driver(&store, &clock, &ids);
    let task_ref = "task://tenant-a/loop-task-5";
    mint_contract(&store, &clock, &ids, 500, task_ref, 8, 3);
    let loop_id = oid(510);
    admit(&store, &clock, &ids, &loop_id, LifecycleDomain::Loop, None);
    let budget = budget_id(514);
    create_budget(&store, &clock, &ids, &budget, 10);
    // A governed task object for the acceptance reload.
    let task_id = oid(511);
    admit(&store, &clock, &ids, &task_id, LifecycleDomain::Task, None);

    let started = harness
        .start_loop(&loop_id, Version::INITIAL, task_ref, &budget, &lease(1))
        .unwrap();
    // Drive the loop to VERIFY.
    let mut v = started.after_version;
    for (from, to, reason) in [
        ("OBSERVE", "RESOLVE", "EVIDENCE_OBSERVED"),
        ("RESOLVE", "ORIENT", "CONTEXT_COMPLETE"),
        ("ORIENT", "DECIDE", "ORIENTATION_COMPLETE"),
        ("DECIDE", "ACT", "OPERATION_ADMITTED"),
        ("ACT", "VERIFY", "PROGRESS_CLAIMED"),
    ] {
        v = drive(
            &store,
            &clock,
            &ids,
            LifecycleDomain::Loop,
            &loop_id,
            from,
            to,
            reason,
            v,
            None,
        );
    }

    let ended = harness
        .end_iteration(
            &loop_id,
            v,
            &task_id,
            &oid(512),
            "verification-report: staging healthy",
            &budget,
            &lease(1),
        )
        .unwrap();
    let stored = store
        .load_object(LifecycleDomain::Loop, &loop_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "CONTINUE");

    // Drive back to VERIFY; then complete the task and try to close
    // another iteration — the task_not_accepted guard fails closed.
    let mut v2 = ended.after_version;
    for (from, to, reason) in [
        ("CONTINUE", "OBSERVE", "NEXT_ITERATION"),
        ("OBSERVE", "RESOLVE", "EVIDENCE_OBSERVED"),
        ("RESOLVE", "ORIENT", "CONTEXT_COMPLETE"),
        ("ORIENT", "DECIDE", "ORIENTATION_COMPLETE"),
        ("DECIDE", "ACT", "OPERATION_ADMITTED"),
        ("ACT", "VERIFY", "PROGRESS_CLAIMED"),
    ] {
        v2 = drive(
            &store,
            &clock,
            &ids,
            LifecycleDomain::Loop,
            &loop_id,
            from,
            to,
            reason,
            v2,
            None,
        );
    }
    let mut tv = Version::INITIAL;
    for (from, to, reason) in [
        ("DRAFT", "READY", "CONTRACT_ACCEPTED"),
        ("READY", "ACTIVE", "EXECUTION_STARTED"),
        ("ACTIVE", "CANDIDATE_COMPLETE", "COMPLETION_CLAIMED"),
        ("CANDIDATE_COMPLETE", "COMPLETED", "ACCEPTANCE_GRANTED"),
    ] {
        tv = drive(
            &store,
            &clock,
            &ids,
            LifecycleDomain::Task,
            &task_id,
            from,
            to,
            reason,
            tv,
            None,
        );
    }
    let _ = tv;
    let refused = harness
        .end_iteration(
            &loop_id,
            v2,
            &task_id,
            &oid(513),
            "verification-report: post-acceptance",
            &budget,
            &lease(1),
        )
        .expect_err("a COMPLETED task admits no further iteration close");
    match refused {
        EffectError::Rejected(rejection) => {
            assert_eq!(rejection.kind, RejectionKind::GuardUnsatisfied);
        }
        other => panic!("unexpected {other:?}"),
    }
}
