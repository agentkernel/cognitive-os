//! M5 intent-chain behavior: UserIntentRecord fixing, candidate/authority
//! admission isolation, TaskContract minting, user-correction supersession
//! and `INTENT_VERSION_SUPERSEDED` fencing — against the real kernel gate
//! and SQLite WAL authority store.
//!
//! Vector semantics covered (behavioral twins; vector execution itself
//! stays with Lane-CFR): `intent-supersede-002.json` (epoch advance
//! rejects old dispatch, pending effect must reconcile before continue,
//! zero execution) and the `INTENT_CLARIFICATION_REQUIRED` clarification
//! rule of `task-loop-verification.md` section 2.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "m4_common/mod.rs"]
mod m4_common;

use cognitive_contracts::generated::governed_object_header::GovernedObjectHeaderSensitivity;
use cognitive_contracts::generated::task_contract::{ContractConditionKind, TaskContract};
use cognitive_contracts::generated::user_intent_record::UserIntentRecord;
use cognitive_domain::{LifecycleDomain, Version};
use cognitive_kernel::effects::{EffectError, mint_intent};
use cognitive_kernel::intent_chain::{
    AcceptanceCommand, AmbiguityFact, ConditionSpec, GovernanceSeed, InterpretationCandidate,
    PendingWorkDisposition, SupersedeCommand, TaskContractCommand, UserIntentCommand,
    admit_interpretation, mint_task_contract, record_interpretation_candidate, record_user_intent,
    supersede_task_contract,
};
use cognitive_kernel::ports::{AuthorityStore, IntentChainStore, ProtocolStore, TaskBinding};
use cognitive_store::SqliteAuthorityStore;
use cognitive_store::faults::{ScriptedExecutor, ScriptedOutcome};
use m4_common::*;

fn fresh_store(dir: &tempfile::TempDir) -> SqliteAuthorityStore {
    SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap()
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

fn user_intent_cmd(record_n: u64, expression: &str) -> UserIntentCommand {
    UserIntentCommand {
        record_id: oid(record_n),
        actor_chain_digest: format!("sha256:{}", "aa11".repeat(16)),
        conversation_or_scope_ref: uri("conversation://tenant-a/thread-1"),
        input_refs: vec![uri("state://tenant-a/attachments/spec-v1")],
        raw_expression: expression.to_owned(),
        intent_authority_ref: uri("principal://tenant-a/user-1"),
        governance: seed(),
        correlation_id: uri("corr://tenant-a/m5-chain"),
    }
}

fn clean_candidate(interp_n: u64) -> InterpretationCandidate {
    InterpretationCandidate {
        interpretation_id: oid(interp_n),
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
        contract_id: oid(contract_n),
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
        allowed_state_domains: vec!["task".to_owned(), "effect".to_owned()],
        allowed_tools: vec!["operation://tenant-a/payments/refund".to_owned()],
        governance: seed(),
        correlation_id: uri("corr://tenant-a/m5-chain"),
    }
}

/// Full happy chain to an epoch-1 contract; returns (record, contract).
fn chain_to_contract(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    record_n: u64,
    interp_n: u64,
    contract_n: u64,
    task_ref: &str,
) -> (
    cognitive_kernel::ports::UserIntentRecordRow,
    cognitive_kernel::ports::TaskContractRow,
) {
    let record = record_user_intent(
        store,
        clock,
        ids,
        &lease(1),
        &user_intent_cmd(record_n, "deploy service v2 to staging"),
    )
    .unwrap();
    let interpretation = record_interpretation_candidate(
        store,
        clock,
        ids,
        &lease(1),
        &record.record_id,
        &clean_candidate(interp_n),
        &seed(),
        &uri("corr://tenant-a/m5-chain"),
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
    let contract = mint_task_contract(
        store,
        clock,
        ids,
        &lease(1),
        &admitted,
        &contract_cmd(contract_n, task_ref),
        0,
    )
    .unwrap();
    (record, contract)
}

fn denial_code(err: &EffectError) -> (&'static str, &'static str, bool) {
    match err {
        EffectError::Denied(denial) => (
            denial.registered.code,
            denial.registered.category,
            denial.registered.retryable,
        ),
        other => panic!("expected protocol denial, got {other:?}"),
    }
}

/// REQ-INTENT-RECORD-001: the record is fixed durably BEFORE any semantic
/// interpretation, its canonical shape round-trips through the generated
/// `user-intent-record` binding, and nothing can overwrite it — not a
/// duplicate insert, not raw SQL, not a later correction.
#[test]
fn user_intent_record_is_fixed_first_and_never_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();

    // Interpretation BEFORE the record is fixed: refused, nothing persists.
    let premature = record_interpretation_candidate(
        &store,
        &clock,
        &ids,
        &lease(1),
        &oid(100),
        &clean_candidate(150),
        &seed(),
        &uri("corr://tenant-a/m5-chain"),
    )
    .expect_err("interpretation must not precede the fixed record");
    assert_eq!(denial_code(&premature).0, "STATE_CONFLICT");
    assert!(store.load_interpretation(&oid(150)).unwrap().is_none());

    let row = record_user_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &user_intent_cmd(100, "deploy service v2 to staging"),
    )
    .unwrap();
    assert!(row.intent_digest.starts_with("sha256:"));

    // The canonical value IS the registered schema shape: it parses
    // through the generated binding (deny_unknown_fields) and the header
    // carries the sealed content digest.
    let parsed: UserIntentRecord = serde_json::from_str(&row.canonical_json).unwrap();
    assert_eq!(parsed.raw_expression, "deploy service v2 to staging");
    assert_eq!(parsed.intent_digest.0, row.intent_digest);
    assert!(parsed.header.content_digest.0.starts_with("sha256:"));
    assert_ne!(
        parsed.header.content_digest.0,
        format!("sha256:{}", "0".repeat(64))
    );

    // Duplicate identity: conflict, original untouched.
    let duplicate = record_user_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &user_intent_cmd(100, "OVERWRITTEN"),
    )
    .expect_err("a record identity is fixed once");
    assert!(matches!(duplicate, EffectError::Rejected(_)));
    let reloaded = store.load_user_intent(&oid(100)).unwrap().unwrap();
    assert_eq!(reloaded.raw_expression, "deploy service v2 to staging");

    // Storage-layer immutability: raw UPDATE/DELETE abort in ANY
    // connection (the append-only trigger discipline of the event log).
    let raw = rusqlite::Connection::open(dir.path().join("authority.db")).unwrap();
    let update = raw
        .execute(
            "UPDATE user_intent_records SET raw_expression = 'summary'",
            [],
        )
        .unwrap_err();
    assert!(update.to_string().contains("append-only"));
    let delete = raw
        .execute("DELETE FROM user_intent_records", [])
        .unwrap_err();
    assert!(delete.to_string().contains("append-only"));

    // Scope query returns the record in insertion order.
    let listed = store
        .list_user_intents_for_scope("conversation://tenant-a/thread-1")
        .unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].record_id, row.record_id);
}

/// REQ-INTENT-ADMISSION-001 negative: a candidate carrying a MATERIAL
/// ambiguity is recorded `clarification_required` by the deterministic
/// schema conditional, and admission refuses it with the registered
/// `INTENT_CLARIFICATION_REQUIRED` — no top-1 selection, no contract, no
/// dispatch surface.
#[test]
fn material_ambiguity_forces_clarification_not_top1() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let record = record_user_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &user_intent_cmd(110, "restart it"),
    )
    .unwrap();

    let mut ambiguous = clean_candidate(160);
    ambiguous.ambiguities.push(AmbiguityFact {
        id: "amb-target".to_owned(),
        material: true,
        question: "which of the two running executions is 'it'?".to_owned(),
    });
    let interpretation = record_interpretation_candidate(
        &store,
        &clock,
        &ids,
        &lease(1),
        &record.record_id,
        &ambiguous,
        &seed(),
        &uri("corr://tenant-a/m5-chain"),
    )
    .unwrap();
    // The recorded status is the deterministic derivation, not a model
    // choice (registered schema conditional).
    assert_eq!(interpretation.recorded_status, "clarification_required");
    assert_eq!(interpretation.material_ambiguity_count, 1);

    let refused = admit_interpretation(
        &store,
        &AcceptanceCommand {
            interpretation_id: interpretation.interpretation_id.clone(),
            accepted_by: uri("principal://tenant-a/user-1"),
            accepted_digest: interpretation.interpretation_digest.clone(),
        },
    )
    .expect_err("material ambiguity must clarify, never admit");
    let (code, category, retryable) = denial_code(&refused);
    assert_eq!(code, "INTENT_CLARIFICATION_REQUIRED");
    assert_eq!(category, "intent");
    assert!(
        retryable,
        "clarification is retryable after the authority answers"
    );

    // Decision surface stayed closed: no contract epoch, no intents.
    assert_eq!(
        store
            .current_contract_epoch("task://tenant-a/rollout")
            .unwrap(),
        0
    );
    assert!(
        store
            .list_intents_for_task("task://tenant-a/rollout")
            .unwrap()
            .is_empty()
    );
}

/// REQ-INTENT-ADMISSION-001 authority + digest binding: only the record's
/// registered intent authority may accept, and only the exact digest it
/// reviewed; the accepted chain then mints the epoch-1 TaskContract whose
/// canonical shape round-trips through the generated binding.
#[test]
fn acceptance_is_authority_and_digest_bound_then_mints_the_contract() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let record = record_user_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &user_intent_cmd(120, "deploy service v2 to staging"),
    )
    .unwrap();
    let interpretation = record_interpretation_candidate(
        &store,
        &clock,
        &ids,
        &lease(1),
        &record.record_id,
        &clean_candidate(170),
        &seed(),
        &uri("corr://tenant-a/m5-chain"),
    )
    .unwrap();
    assert_eq!(interpretation.recorded_status, "candidate");

    // An agent narrating "the user approved" is not the intent authority.
    let unauthorized = admit_interpretation(
        &store,
        &AcceptanceCommand {
            interpretation_id: interpretation.interpretation_id.clone(),
            accepted_by: uri("agent://tenant-a/planner"),
            accepted_digest: interpretation.interpretation_digest.clone(),
        },
    )
    .expect_err("only the registered intent authority accepts");
    assert_eq!(denial_code(&unauthorized).0, "CONTEXT_AUTH_DENIED");

    // Acceptance binds the exact reviewed digest, not an impression.
    let wrong_digest = admit_interpretation(
        &store,
        &AcceptanceCommand {
            interpretation_id: interpretation.interpretation_id.clone(),
            accepted_by: uri("principal://tenant-a/user-1"),
            accepted_digest: format!("sha256:{}", "ff".repeat(32)),
        },
    )
    .expect_err("digest mismatch must fail");
    assert_eq!(denial_code(&wrong_digest).0, "STATE_CONFLICT");

    let admitted = admit_interpretation(
        &store,
        &AcceptanceCommand {
            interpretation_id: interpretation.interpretation_id.clone(),
            accepted_by: uri("principal://tenant-a/user-1"),
            accepted_digest: interpretation.interpretation_digest.clone(),
        },
    )
    .unwrap();

    // A contract without a decidable acceptance condition is refused
    // (REQ-RUN-004) before any epoch is consumed.
    let mut undecidable = contract_cmd(180, "task://tenant-a/rollout");
    undecidable.conditions = vec![ConditionSpec {
        id: "stop-1".to_owned(),
        kind: ContractConditionKind::Stop,
        description: "stop on demand".to_owned(),
        verifier_ref: None,
    }];
    let refused = mint_task_contract(&store, &clock, &ids, &lease(1), &admitted, &undecidable, 0)
        .expect_err("completion must be decidable");
    assert_eq!(denial_code(&refused).0, "STATE_CONFLICT");
    assert_eq!(
        store
            .current_contract_epoch("task://tenant-a/rollout")
            .unwrap(),
        0
    );

    let contract = mint_task_contract(
        &store,
        &clock,
        &ids,
        &lease(1),
        &admitted,
        &contract_cmd(180, "task://tenant-a/rollout"),
        0,
    )
    .unwrap();
    assert_eq!(contract.contract_epoch, 1);
    assert_eq!(
        store
            .current_contract_epoch("task://tenant-a/rollout")
            .unwrap(),
        1
    );

    // Canonical shape parses through the generated task-contract binding
    // and binds the full chain: record, interpretation, acceptance.
    let parsed: TaskContract = serde_json::from_str(&contract.canonical_json).unwrap();
    assert_eq!(parsed.contract_epoch, 1);
    assert_eq!(parsed.user_intent_ref.id.0, record.record_id.as_str());
    assert_eq!(
        parsed.intent_interpretation_ref.id.0,
        interpretation.interpretation_id.as_str()
    );
    assert_eq!(
        parsed.intent_interpretation_ref.content_digest.0,
        interpretation.interpretation_digest
    );
    assert_eq!(parsed.max_iterations, 8);

    // The epoch CAS is monotonic: a second epoch-1 mint (stale expected
    // epoch) is refused and nothing persists beside epoch 1.
    let raced = mint_task_contract(
        &store,
        &clock,
        &ids,
        &lease(1),
        &admitted,
        &contract_cmd(181, "task://tenant-a/rollout"),
        0,
    )
    .expect_err("stale expected epoch must lose the CAS");
    assert_eq!(denial_code(&raced).0, "STATE_CONFLICT");
    assert_eq!(
        store
            .current_contract_epoch("task://tenant-a/rollout")
            .unwrap(),
        1
    );
}

/// Vector `intent-supersede-002` semantics (REQ-INTENT-SUPERSEDE-001,
/// RFC-0001 REQ-SHELL-CORRECTION-001): a user correction advances the
/// contract epoch; the pending old-epoch effect is classified
/// must-reconcile (reconcile_before_continue); a NEW dispatch bound to the
/// old epoch is rejected with `INTENT_VERSION_SUPERSEDED` and ZERO
/// execution reaches the sink; prior records are never rewritten.
#[test]
fn user_correction_advances_epoch_and_fences_old_dispatch() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let driver = protocol(&store, &clock, &ids);
    let grant = grant_for("payments.refund");
    let task_ref = "task://tenant-a/rollout-7";

    let (original_record, contract_v1) =
        chain_to_contract(&store, &clock, &ids, 200, 210, 220, task_ref);
    assert_eq!(contract_v1.contract_epoch, 1);

    // A pending effect dispatched under epoch 1 (EXECUTING at correction
    // time — the vector's `pending_effect: true`).
    let pending_effect = oid(230);
    admit(
        &store,
        &clock,
        &ids,
        &pending_effect,
        LifecycleDomain::Effect,
        None,
    );
    let executor = ScriptedExecutor::queryable(1);
    executor.script(&[ScriptedOutcome::ExecuteThenTimeout]);
    let mut pending_cmd = intent_command(
        231,
        &pending_effect,
        "rollout-7-step-1-attempt-1",
        1000,
        descriptor(true, false),
    );
    pending_cmd.task_binding = Some(TaskBinding {
        task_ref: task_ref.to_owned(),
        contract_epoch: 1,
    });
    mint_intent(&store, &clock, &ids, &lease(1), &pending_cmd).unwrap();
    let authorized = driver
        .authorize_effect(
            &pending_effect,
            Version::INITIAL,
            &grant,
            &currency(),
            &lease(1),
        )
        .unwrap();
    driver
        .dispatch_effect(
            &pending_effect,
            authorized.after_version,
            &grant,
            &currency(),
            &executor,
            &lease(1),
        )
        .unwrap();

    // A second old-epoch intent minted but NOT yet dispatched (the "new
    // dispatch under the old epoch" the vector rejects).
    let undispatched_effect = oid(240);
    admit(
        &store,
        &clock,
        &ids,
        &undispatched_effect,
        LifecycleDomain::Effect,
        None,
    );
    let mut undispatched_cmd = intent_command(
        241,
        &undispatched_effect,
        "rollout-7-step-2-attempt-1",
        2000,
        descriptor(true, false),
    );
    undispatched_cmd.task_binding = Some(TaskBinding {
        task_ref: task_ref.to_owned(),
        contract_epoch: 1,
    });
    mint_intent(&store, &clock, &ids, &lease(1), &undispatched_cmd).unwrap();
    let undispatched_authorized = driver
        .authorize_effect(
            &undispatched_effect,
            Version::INITIAL,
            &grant,
            &currency(),
            &lease(1),
        )
        .unwrap();

    // The user corrects: "no, roll out to staging EU only". The
    // correction is fixed as a NEW record; the original stays untouched.
    let correction_record = record_user_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &user_intent_cmd(250, "no - staging EU only"),
    )
    .unwrap();
    let mut correction_candidate = clean_candidate(260);
    correction_candidate.interpretation_id = oid(260);
    correction_candidate.objectives = vec!["roll out v2 to staging EU only".to_owned()];
    correction_candidate.supersedes = Some(contract_v1.interpretation_id.clone());
    let superseding = record_interpretation_candidate(
        &store,
        &clock,
        &ids,
        &lease(1),
        &correction_record.record_id,
        &correction_candidate,
        &seed(),
        &uri("corr://tenant-a/m5-chain"),
    )
    .unwrap();
    let mut correction_contract = contract_cmd(270, task_ref);
    correction_contract.objective = "staging EU rollout of service v2".to_owned();

    // Corrections pass the SAME admission gate: a digest the authority
    // never reviewed fails closed, and no epoch advances.
    let refused = supersede_task_contract(
        &store,
        &clock,
        &ids,
        &lease(1),
        &SupersedeCommand {
            acceptance: AcceptanceCommand {
                interpretation_id: oid(260),
                accepted_by: uri("principal://tenant-a/user-1"),
                accepted_digest: format!("sha256:{}", "ee".repeat(32)),
            },
            contract: correction_contract.clone(),
            expected_current_epoch: 1,
        },
    )
    .expect_err("an unreviewed digest must fail the admission gate");
    assert_eq!(denial_code(&refused).0, "STATE_CONFLICT");
    assert_eq!(store.current_contract_epoch(task_ref).unwrap(), 1);

    let report = supersede_task_contract(
        &store,
        &clock,
        &ids,
        &lease(1),
        &SupersedeCommand {
            acceptance: AcceptanceCommand {
                interpretation_id: oid(260),
                accepted_by: uri("principal://tenant-a/user-1"),
                accepted_digest: superseding.interpretation_digest.clone(),
            },
            contract: correction_contract,
            expected_current_epoch: 1,
        },
    )
    .unwrap();

    // Epoch advanced 1 -> 2; the report carries the correction chain.
    assert_eq!(report.superseded_epoch, 1);
    assert_eq!(report.new_contract.contract_epoch, 2);
    assert_eq!(store.current_contract_epoch(task_ref).unwrap(), 2);
    assert_eq!(
        report.correction_record.record_id,
        correction_record.record_id
    );
    assert_eq!(
        report.superseding_interpretation.supersedes_interpretation,
        Some(contract_v1.interpretation_id.clone())
    );

    // Pending old-epoch work is classified: the EXECUTING effect must be
    // reconciled before continuing (vector `pending_effect_action`); the
    // authorized-but-undispatched one is safely cancelled by the fence.
    let by_effect: std::collections::BTreeMap<_, _> = report
        .pending
        .iter()
        .map(|p| (p.effect_object_id.clone(), p))
        .collect();
    assert_eq!(
        by_effect.get(&pending_effect).unwrap().disposition,
        PendingWorkDisposition::MustReconcile,
        "dispatched old-epoch effect: reconcile_before_continue"
    );
    assert_eq!(
        by_effect.get(&undispatched_effect).unwrap().disposition,
        PendingWorkDisposition::SafelyCancelled,
        "undispatched old-epoch intent: nothing ran, the fence retires it"
    );

    // `old_epoch_new_dispatch_rejected`, arm 1: minting a NEW intent
    // bound to the superseded epoch is refused with the registered code.
    let late_effect = oid(280);
    admit(
        &store,
        &clock,
        &ids,
        &late_effect,
        LifecycleDomain::Effect,
        None,
    );
    let mut late_cmd = intent_command(
        281,
        &late_effect,
        "rollout-7-step-3-attempt-1",
        3000,
        descriptor(true, false),
    );
    late_cmd.task_binding = Some(TaskBinding {
        task_ref: task_ref.to_owned(),
        contract_epoch: 1,
    });
    let fenced = mint_intent(&store, &clock, &ids, &lease(1), &late_cmd)
        .expect_err("old-epoch proposal must not mint");
    let (code, category, retryable) = denial_code(&fenced);
    assert_eq!(code, "INTENT_VERSION_SUPERSEDED");
    assert_eq!(category, "intent");
    assert!(
        retryable,
        "re-proposing under the current epoch is possible"
    );
    assert!(
        store
            .load_intent_by_key("rollout-7-step-3-attempt-1")
            .unwrap()
            .is_none(),
        "nothing persisted for the fenced proposal"
    );

    // Arm 2: dispatching the ALREADY-MINTED old-epoch intent is refused at
    // the dispatch sink — zero executor calls, effect stays AUTHORIZED.
    let sink_calls_before = executor.dispatches().len();
    let fenced_dispatch = driver
        .dispatch_effect(
            &undispatched_effect,
            undispatched_authorized.after_version,
            &grant,
            &currency(),
            &executor,
            &lease(1),
        )
        .expect_err("old-epoch dispatch must be fenced");
    assert_eq!(denial_code(&fenced_dispatch).0, "INTENT_VERSION_SUPERSEDED");
    assert_eq!(
        executor.dispatches().len(),
        sink_calls_before,
        "zero execution: the sink never saw the fenced dispatch"
    );
    let still_authorized = store
        .load_object(LifecycleDomain::Effect, &undispatched_effect)
        .unwrap()
        .unwrap();
    assert_eq!(still_authorized.state.as_str(), "AUTHORIZED");

    // Supersession rewrote NOTHING: the original record, interpretation
    // and epoch-1 contract are still there, verbatim.
    let original_reloaded = store
        .load_user_intent(&original_record.record_id)
        .unwrap()
        .unwrap();
    assert_eq!(original_reloaded, original_record);
    assert!(
        store
            .load_interpretation(&contract_v1.interpretation_id)
            .unwrap()
            .is_some()
    );
    let epoch1 = store.load_task_contract(task_ref, 1).unwrap().unwrap();
    assert_eq!(epoch1.contract_id, contract_v1.contract_id);

    // A racing second supersede against the stale epoch loses the CAS.
    let race = supersede_task_contract(
        &store,
        &clock,
        &ids,
        &lease(1),
        &SupersedeCommand {
            acceptance: AcceptanceCommand {
                interpretation_id: oid(260),
                accepted_by: uri("principal://tenant-a/user-1"),
                accepted_digest: superseding.interpretation_digest.clone(),
            },
            contract: contract_cmd(292, task_ref),
            expected_current_epoch: 1,
        },
    );
    assert!(race.is_err(), "stale expected epoch must lose");
    assert_eq!(store.current_contract_epoch(task_ref).unwrap(), 2);
}

/// Replay integrity: the chain's provenance events (user intent recorded,
/// interpretation recorded, contract minted) fold as provenance without
/// disturbing the byte-stable projection digest.
#[test]
fn chain_events_fold_as_provenance_in_replay() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let before = cognitive_kernel::replay_projection(&store).unwrap();
    chain_to_contract(
        &store,
        &clock,
        &ids,
        300,
        310,
        320,
        "task://tenant-a/replay",
    );
    let after = cognitive_kernel::replay_projection(&store).unwrap();
    assert_eq!(after.event_count, before.event_count + 3);
    // Provenance events fold no object state.
    assert_eq!(after.value["objects"], before.value["objects"]);
    let again = cognitive_kernel::replay_projection(&store).unwrap();
    assert_eq!(again.digest, after.digest, "replay digest is byte-stable");
}
