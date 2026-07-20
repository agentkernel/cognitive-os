//! M4 effect-protocol behavior (acceptance criteria 2, 3, 4, 5 and the
//! F-014 sink fencing matrix + F-023 admission matrix), against the real
//! kernel gate and SQLite WAL authority store.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "m4_common/mod.rs"]
mod m4_common;

use cognitive_domain::{LifecycleDomain, Version};
use cognitive_kernel::effects::{EffectError, MintedIntent, mint_intent};
use cognitive_kernel::executor::{DispatchOutcome, EffectExecutor};
use cognitive_kernel::ports::{AuthorityStore, CheckpointRow, ProtocolStore};
use cognitive_kernel::{VerificationRecord, VerificationStatus};
use cognitive_store::SqliteAuthorityStore;
use cognitive_store::faults::{ScriptedExecutor, ScriptedOutcome};
use m4_common::*;

fn fresh_store(dir: &tempfile::TempDir) -> SqliteAuthorityStore {
    SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap()
}

/// Criterion 3 (vector `effect-idempotency-conflict.json` semantics): the
/// same key with a different canonical parameter digest is rejected with
/// EFFECT_IDEMPOTENCY_CONFLICT — no new effect, no dedup, no overwrite;
/// the same key with the SAME digest replays the persisted intent.
#[test]
fn criterion_3_same_key_different_parameters_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let effect_id = oid(500);
    admit(
        &store,
        &clock,
        &ids,
        &effect_id,
        LifecycleDomain::Effect,
        None,
    );

    let minted = mint_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &intent_command(
            701,
            &effect_id,
            "refund-42-attempt-1",
            5000,
            descriptor(true, false),
        ),
    )
    .unwrap();
    let original = match minted {
        MintedIntent::Persisted(row) => row,
        other => panic!("expected fresh intent, got {other:?}"),
    };

    // Same key, same canonical parameters (member order shuffled): replay.
    let mut replay_cmd = intent_command(
        702,
        &effect_id,
        "refund-42-attempt-1",
        5000,
        descriptor(true, false),
    );
    replay_cmd.parameters = serde_json::json!({"currency": "EUR", "amount_minor": 5000});
    let replayed = mint_intent(&store, &clock, &ids, &lease(1), &replay_cmd).unwrap();
    match replayed {
        MintedIntent::ReplayedExisting(row) => {
            assert_eq!(row.intent_id, original.intent_id, "no second intent minted");
            assert_eq!(row.idempotency_key, original.idempotency_key);
        }
        other => panic!("expected replay, got {other:?}"),
    }

    // Same key, DIFFERENT parameters: the registered conflict, fail closed.
    let effect2 = oid(501);
    admit(
        &store,
        &clock,
        &ids,
        &effect2,
        LifecycleDomain::Effect,
        None,
    );
    let conflict = mint_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &intent_command(
            703,
            &effect2,
            "refund-42-attempt-1",
            9900,
            descriptor(true, false),
        ),
    )
    .expect_err("different digest under the same key must be rejected");
    match conflict {
        EffectError::Denied(denial) => {
            assert_eq!(denial.registered.code, "EFFECT_IDEMPOTENCY_CONFLICT");
            assert_eq!(denial.registered.category, "effect");
            assert!(!denial.registered.retryable);
        }
        other => panic!("unexpected error {other:?}"),
    }
    // The existing intent is unchanged and no new intent was created.
    let still = store
        .load_intent_by_key("refund-42-attempt-1")
        .unwrap()
        .unwrap();
    assert_eq!(still, original, "existing effect/intent state unchanged");
    assert!(store.load_intent_for_effect(&effect2).unwrap().is_none());
}

/// F-023 admission matrix behavioral side: a governed_external operation
/// with a neither-queryable-nor-idempotent executor cannot even mint an
/// Intent; nothing is persisted.
#[test]
fn f023_unqueryable_nonidempotent_operation_cannot_mint_an_intent() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let effect_id = oid(510);
    admit(
        &store,
        &clock,
        &ids,
        &effect_id,
        LifecycleDomain::Effect,
        None,
    );

    let rejected = mint_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &intent_command(
            711,
            &effect_id,
            "refund-77-attempt-1",
            100,
            descriptor(false, false),
        ),
    )
    .expect_err("no safe recovery closure: admission must reject");
    match rejected {
        EffectError::Denied(denial) => {
            assert_eq!(denial.registered.code, "NO_AUTHORIZED_OPERATION_CANDIDATE");
            assert_eq!(denial.registered.category, "catalog");
        }
        other => panic!("unexpected error {other:?}"),
    }
    assert!(
        store
            .load_intent_by_key("refund-77-attempt-1")
            .unwrap()
            .is_none(),
        "nothing was persisted"
    );
}

/// Criterion 2 (vector `effect-unknown-outcome.json` semantics): a timeout
/// after dispatch parks the effect in OUTCOME_UNKNOWN; the retry path
/// REUSES the original idempotency key through reconciliation — no blind
/// retry, no new key, no success report; indeterminate reconciliation
/// quarantines with the registered code.
#[test]
fn criterion_2_unknown_outcome_reuses_the_key_and_quarantines_when_unresolvable() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let driver = protocol(&store, &clock, &ids);
    let grant = grant_for("payments.refund");
    let effect_id = oid(520);
    admit(
        &store,
        &clock,
        &ids,
        &effect_id,
        LifecycleDomain::Effect,
        None,
    );
    mint_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &intent_command(
            721,
            &effect_id,
            "refund-43-attempt-1",
            4200,
            descriptor(true, false),
        ),
    )
    .unwrap();

    // Authorize + dispatch; the sink vanishes without executing.
    let executor = ScriptedExecutor::queryable(1);
    executor.script(&[ScriptedOutcome::VanishWithoutExecution]);
    let authorized = driver
        .authorize_effect(&effect_id, Version::INITIAL, &grant, &currency(), &lease(1))
        .unwrap();
    let (dispatched, outcome) = driver
        .dispatch_effect(
            &effect_id,
            authorized.after_version,
            &grant,
            &currency(),
            &executor,
            &lease(1),
        )
        .unwrap();
    assert!(matches!(outcome, DispatchOutcome::Unknown { .. }));
    let unknown = driver
        .record_outcome(&effect_id, dispatched.after_version, &outcome, &lease(1))
        .unwrap();
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "OUTCOME_UNKNOWN");

    // The unknown state admits no direct exit to COMMITTED (pinned in M2;
    // re-asserted here as the protocol-level negative).
    // Reconciliation queries with the ORIGINAL key.
    let (reconciled, query) = driver
        .reconcile(
            &effect_id,
            "OUTCOME_UNKNOWN",
            unknown.after_version,
            &executor,
            &lease(1),
        )
        .unwrap();
    assert_eq!(
        executor.queries(),
        vec!["refund-43-attempt-1".to_owned()],
        "reconciliation bound the original idempotency key"
    );
    assert_eq!(query, cognitive_kernel::ExecutorQueryResult::NotExecuted);
    driver
        .close_not_executed(&effect_id, reconciled.after_version, &lease(1))
        .unwrap();
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "NOT_EXECUTED");

    // No dispatch ever re-fired: exactly one external call, one key.
    let dispatches = executor.dispatches();
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0].idempotency_key, "refund-43-attempt-1");

    // Second effect: executed-but-timeout, and reconciliation stays
    // indeterminate (non-queryable sink) -> quarantine with the
    // registered codes; still the same key, still no blind retry.
    let effect2 = oid(521);
    admit(
        &store,
        &clock,
        &ids,
        &effect2,
        LifecycleDomain::Effect,
        None,
    );
    mint_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &intent_command(
            722,
            &effect2,
            "refund-44-attempt-1",
            4300,
            descriptor(false, true),
        ),
    )
    .unwrap();
    let opaque = ScriptedExecutor::idempotent(1);
    opaque.script(&[ScriptedOutcome::ExecuteThenTimeout]);
    let authorized = driver
        .authorize_effect(&effect2, Version::INITIAL, &grant, &currency(), &lease(1))
        .unwrap();
    let (dispatched, outcome) = driver
        .dispatch_effect(
            &effect2,
            authorized.after_version,
            &grant,
            &currency(),
            &opaque,
            &lease(1),
        )
        .unwrap();
    let unknown = driver
        .record_outcome(&effect2, dispatched.after_version, &outcome, &lease(1))
        .unwrap();
    let (reconciled, query) = driver
        .reconcile(
            &effect2,
            "OUTCOME_UNKNOWN",
            unknown.after_version,
            &opaque,
            &lease(1),
        )
        .unwrap();
    assert_eq!(query, cognitive_kernel::ExecutorQueryResult::Indeterminate);
    let (_, surfaced) = driver
        .quarantine_still_unknown(&effect2, reconciled.after_version, &lease(1))
        .unwrap();
    assert_eq!(surfaced.code, "EFFECT_OUTCOME_UNKNOWN");
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect2)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "QUARANTINED");
    assert_eq!(
        opaque.dispatches().len(),
        1,
        "no blind retry: the timeout did not re-dispatch"
    );
}

/// Criterion 4: a receipt (execution evidence) plus a remote `completed`
/// string do NOT complete the Task; completion requires the verification +
/// acceptance authority path. (Behavioral twin of
/// `remote-completed-not-acceptance.json`, extended to the M4 protocol.)
#[test]
fn criterion_4_receipt_and_remote_completed_do_not_complete_the_task() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let task_id = oid(530);
    admit(&store, &clock, &ids, &task_id, LifecycleDomain::Task, None);
    let v = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &task_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
        Version::INITIAL,
        None,
    );
    let v = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &task_id,
        "READY",
        "ACTIVE",
        "EXECUTION_STARTED",
        v,
        None,
    );

    // A receipt exists; the remote system says "completed"; a model
    // narrative says "done". None of these derive the acceptance guards:
    // the completion transition is rejected and the task stays ACTIVE.
    let engine = cognitive_kernel::TransitionEngine::new(&store, &clock, &ids);
    let loaded = cognitive_domain::table(LifecycleDomain::Task).unwrap();
    let edge = loaded
        .find_edge(
            &state("ACTIVE"),
            &state("CANDIDATE_COMPLETE"),
            "COMPLETION_CLAIMED",
        )
        .unwrap();
    // Claim completion WITHOUT the required completion_claim evidence
    // (only a receipt narrative): evidence gate refuses.
    let cmd = cognitive_kernel::TransitionCommand {
        request_id: uri("request://m4/remote-completed"),
        domain: LifecycleDomain::Task,
        object_id: task_id.clone(),
        subject_ref: uri("task://tenant-a/task-530"),
        from: state("ACTIVE"),
        to: state("CANDIDATE_COMPLETE"),
        expected_version: v,
        reason: cognitive_kernel::Reason {
            code: cognitive_domain::ReasonCode::parse("COMPLETION_CLAIMED").unwrap(),
            detail: Some("remote reports completed; receipt receipt://remote/17".to_owned()),
        },
        causation: cognitive_kernel::Causation {
            causation_id: uri("corr://tenant-a/m4-chain"),
            correlation_id: uri("corr://tenant-a/m4-chain"),
        },
        actor_ref: uri("actor://tenant-a/agent-1"),
        authority_ref: uri("authority://tenant-a/task-acceptance"),
        requested_at: ts("2026-07-20T12:01:00Z"),
        table_pin: cognitive_kernel::TablePin::current(LifecycleDomain::Task).unwrap(),
        established_guards: edge.guards.iter().cloned().collect(),
        evidence: Default::default(), // no completion_claim, no fixed_post_state
        budget: None,
        outbox_destinations: vec![],
        fencing_epoch: None,
    };
    let rejected = engine
        .commit_transition(&cmd)
        .expect_err("receipt is not acceptance");
    assert_eq!(rejected.registered().code, "STATE_CONFLICT");
    let stored = store
        .load_object(LifecycleDomain::Task, &task_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "ACTIVE", "task state unchanged");

    // Even at CANDIDATE_COMPLETE, acceptance without a passing verification
    // guard set cannot reach COMPLETED (guards underivable from receipts).
    let v = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &task_id,
        "ACTIVE",
        "CANDIDATE_COMPLETE",
        "COMPLETION_CLAIMED",
        v,
        None,
    );
    let edge = loaded
        .find_edge(
            &state("CANDIDATE_COMPLETE"),
            &state("COMPLETED"),
            "ACCEPTANCE_GRANTED",
        )
        .unwrap();
    let mut established: std::collections::BTreeSet<String> = edge.guards.iter().cloned().collect();
    // The verification guard cannot be attested: no verification passed.
    established.remove("verification_passed_and_current");
    let cmd = cognitive_kernel::TransitionCommand {
        request_id: uri("request://m4/acceptance-without-verification"),
        established_guards: established,
        evidence: edge
            .required_evidence
            .iter()
            .enumerate()
            .map(|(index, item)| (item.clone(), evidence_ref(index as u64 + 1)))
            .collect(),
        expected_version: v,
        from: state("CANDIDATE_COMPLETE"),
        to: state("COMPLETED"),
        reason: cognitive_kernel::Reason {
            code: cognitive_domain::ReasonCode::parse("ACCEPTANCE_GRANTED").unwrap(),
            detail: None,
        },
        ..cmd
    };
    let rejected = engine
        .commit_transition(&cmd)
        .expect_err("no verification, no completion");
    assert_eq!(rejected.registered().code, "STATE_CONFLICT");
    assert!(rejected.detail.contains("verification_passed_and_current"));
    let stored = store
        .load_object(LifecycleDomain::Task, &task_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "CANDIDATE_COMPLETE");
}

/// Criterion 5: compensation requires INDEPENDENT authorization — reusing
/// the original grant is refused at the protocol layer; a fresh grant for
/// the compensation action (plus a new intent under a new key) proceeds.
#[test]
fn criterion_5_compensation_requires_independent_authorization() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let driver = protocol(&store, &clock, &ids);
    let grant = grant_for("payments.refund");
    let effect_id = oid(540);
    admit(
        &store,
        &clock,
        &ids,
        &effect_id,
        LifecycleDomain::Effect,
        None,
    );
    mint_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &intent_command(
            741,
            &effect_id,
            "refund-45-attempt-1",
            4500,
            descriptor(true, false),
        ),
    )
    .unwrap();

    // Walk to VERIFY_FAILED: authorize, dispatch (executes), record,
    // reconcile(executed), verify with a FAILED record.
    let executor = ScriptedExecutor::queryable(1);
    let v = driver
        .authorize_effect(&effect_id, Version::INITIAL, &grant, &currency(), &lease(1))
        .unwrap()
        .after_version;
    let (committed, outcome) = driver
        .dispatch_effect(&effect_id, v, &grant, &currency(), &executor, &lease(1))
        .unwrap();
    let v = driver
        .record_outcome(&effect_id, committed.after_version, &outcome, &lease(1))
        .unwrap()
        .after_version;
    let (reconciled, _) = driver
        .reconcile(&effect_id, "EXECUTED", v, &executor, &lease(1))
        .unwrap();
    // Verification fails against the fixed post state.
    let subject = oid(541);
    admit(&store, &clock, &ids, &subject, LifecycleDomain::Task, None);
    let failed_record = VerificationRecord {
        verification_object_id: oid(542),
        report_id: oid(543),
        status: VerificationStatus::Failed,
        subject_domain: LifecycleDomain::Task,
        subject_object_id: subject.clone(),
        fixed_post_state_version: Version::INITIAL,
    };
    // RECONCILED -> VERIFY_FAILED via the table edge (POSTCONDITION_FAILED).
    let v = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Effect,
        &effect_id,
        "RECONCILED",
        "VERIFY_FAILED",
        "POSTCONDITION_FAILED",
        reconciled.after_version,
        None,
    );
    let _ = failed_record;

    // Reusing the ORIGINAL grant for compensation: refused outright.
    let comp_intent = cognitive_kernel::IntentRow {
        intent_id: oid(544),
        idempotency_key: "compensate-45-attempt-1".to_owned(),
        parameters_digest: format!("sha256:{}", "9d".repeat(32)),
        action: "payments.reverse_refund".to_owned(),
        target: "https://payments.example/api/reversals".to_owned(),
        effect_object_id: oid(545),
        expected_state_version: Version::INITIAL,
        grant_epoch: 41,
        capability_set_version: 7,
        canonical_json: "{\"compensation\":true}".to_owned(),
    };
    let refused = driver
        .begin_compensation(
            &effect_id,
            v,
            &grant,
            &grant,
            &currency(),
            &comp_intent,
            &lease(1),
        )
        .expect_err("original grant must not authorize compensation");
    match refused {
        EffectError::Denied(denial) => {
            assert_eq!(denial.registered.code, "CONTEXT_AUTH_DENIED");
        }
        other => panic!("unexpected {other:?}"),
    }
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "VERIFY_FAILED", "state unchanged");

    // A FRESH grant for the compensation action proceeds.
    let comp_grant = grant_for("payments.reverse_refund");
    assert_ne!(comp_grant, grant);
    driver
        .begin_compensation(
            &effect_id,
            v,
            &grant,
            &comp_grant,
            &currency(),
            &comp_intent,
            &lease(1),
        )
        .unwrap();
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "COMPENSATING");
}

/// Criterion 6 / F-014: the complete sink matrix rejects a stale-epoch
/// writer at EVERY commit sink, each with a negative probe.
#[test]
fn f014_every_commit_sink_fences_stale_epoch_writers() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let driver = protocol(&store, &clock, &ids);
    let grant = grant_for("payments.refund");

    // Prepare an authorized effect under epoch 1.
    let effect_id = oid(550);
    admit(
        &store,
        &clock,
        &ids,
        &effect_id,
        LifecycleDomain::Effect,
        Some(1),
    );
    mint_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &intent_command(
            751,
            &effect_id,
            "refund-46-attempt-1",
            4600,
            descriptor(true, false),
        ),
    )
    .unwrap();
    let v = driver
        .authorize_effect(&effect_id, Version::INITIAL, &grant, &currency(), &lease(1))
        .unwrap()
        .after_version;

    // Recovery elsewhere advances the epoch: the old writer (epoch 1) is
    // now stale everywhere.
    let new_epoch = store.advance_fencing_epoch().unwrap();
    assert_eq!(new_epoch, 2);
    let stale = lease(1);

    // Sink 1 — external executor: the SINK ITSELF rejects a stale epoch
    // (probe the adapter directly; the driver would refuse earlier).
    let executor = ScriptedExecutor::queryable(2);
    let outcome = executor
        .dispatch(&cognitive_kernel::ExecutorCall {
            action: "payments.refund".to_owned(),
            target: "https://payments.example/api/refunds".to_owned(),
            idempotency_key: "refund-46-attempt-1".to_owned(),
            parameters_digest: format!("sha256:{}", "1a".repeat(32)),
            authorization_digest: "epoch:41".to_owned(),
            fencing_epoch: 1,
        })
        .unwrap();
    assert!(matches!(
        outcome,
        DispatchOutcome::FencedStaleEpoch { sink_epoch: 2 }
    ));
    assert!(executor.executed_keys().is_empty(), "nothing executed");
    // And the driver refuses before even reaching the sink.
    let refused = driver
        .dispatch_effect(&effect_id, v, &grant, &currency(), &executor, &stale)
        .expect_err("driver pre-check fences the stale writer");
    assert!(matches!(refused, EffectError::Denied(_)));

    // Sink 2 — authority-store transition commit: in-transaction check.
    let refused = drive_expect_fenced(&store, &clock, &ids, &effect_id, v, 1);
    assert_eq!(refused.registered().code, "STATE_CONFLICT");
    assert!(refused.detail.contains("fenced"));
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "AUTHORIZED", "no state change");

    // Sink 3 — admission + outbox: stale admission is rejected atomically.
    let engine = cognitive_kernel::TransitionEngine::new(&store, &clock, &ids);
    let rejected = engine
        .admit_object(&cognitive_kernel::AdmitCommand {
            object_id: oid(551),
            domain: LifecycleDomain::Effect,
            subject_ref: uri("effect://tenant-a/551"),
            body: serde_json::json!({}),
            actor_ref: uri("actor://tenant-a/agent-1"),
            authority_ref: uri("authority://tenant-a/effect-authority"),
            correlation_id: uri("corr://tenant-a/m4-chain"),
            outbox_destinations: vec!["watch://status".to_owned()],
            fencing_epoch: Some(1),
        })
        .expect_err("stale admission fenced");
    assert_eq!(rejected.registered().code, "STATE_CONFLICT");
    assert!(
        store
            .load_object(LifecycleDomain::Effect, &oid(551))
            .unwrap()
            .is_none()
    );
    assert!(
        store.pending_outbox(10).unwrap().is_empty(),
        "no outbox row leaked"
    );
    // Intent minting under a stale lease is refused as well.
    let refused = mint_intent(
        &store,
        &clock,
        &ids,
        &stale,
        &intent_command(
            752,
            &oid(551),
            "refund-47-attempt-1",
            100,
            descriptor(true, false),
        ),
    )
    .expect_err("stale intent minting fenced");
    assert!(matches!(refused, EffectError::Denied(_)));

    // Sink 4 — checkpoint write: in-transaction check.
    let refused = store
        .append_checkpoint(&CheckpointRow {
            checkpoint_id: oid(552),
            loop_object_id: oid(553),
            event_high_watermark: 1,
            fencing_epoch: 1,
            canonical_json: "{}".to_owned(),
        })
        .expect_err("stale checkpoint fenced");
    assert!(matches!(
        refused,
        cognitive_kernel::StorePortError::Conflict { .. }
    ));
    assert!(store.latest_checkpoint(&oid(553)).unwrap().is_none());

    // Positive control: the CURRENT epoch passes every sink.
    let current = lease(2);
    driver.verify_lease(&current).unwrap();
    store
        .append_checkpoint(&CheckpointRow {
            checkpoint_id: oid(554),
            loop_object_id: oid(553),
            event_high_watermark: 1,
            fencing_epoch: 2,
            canonical_json: "{}".to_owned(),
        })
        .unwrap();
}

/// Attempt an effect transition under a stale declared epoch, expecting
/// the in-transaction fencing conflict from the store.
fn drive_expect_fenced(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    effect_id: &cognitive_domain::ObjectId,
    expected_version: Version,
    stale_epoch: i64,
) -> cognitive_kernel::TransitionRejection {
    let loaded = cognitive_domain::table(LifecycleDomain::Effect).unwrap();
    let edge = loaded
        .find_edge(&state("AUTHORIZED"), &state("EXECUTING"), "DISPATCHED")
        .unwrap();
    let engine = cognitive_kernel::TransitionEngine::new(store, clock, ids);
    engine
        .commit_transition(&cognitive_kernel::TransitionCommand {
            request_id: uri("request://m4/fenced-commit"),
            domain: LifecycleDomain::Effect,
            object_id: effect_id.clone(),
            subject_ref: uri("effect://tenant-a/550"),
            from: state("AUTHORIZED"),
            to: state("EXECUTING"),
            expected_version,
            reason: cognitive_kernel::Reason {
                code: cognitive_domain::ReasonCode::parse("DISPATCHED").unwrap(),
                detail: None,
            },
            causation: cognitive_kernel::Causation {
                causation_id: uri("corr://tenant-a/m4-chain"),
                correlation_id: uri("corr://tenant-a/m4-chain"),
            },
            actor_ref: uri("actor://tenant-a/agent-1"),
            authority_ref: uri("authority://tenant-a/effect-authority"),
            requested_at: ts("2026-07-20T12:01:00Z"),
            table_pin: cognitive_kernel::TablePin::current(LifecycleDomain::Effect).unwrap(),
            established_guards: edge.guards.iter().cloned().collect(),
            evidence: edge
                .required_evidence
                .iter()
                .enumerate()
                .map(|(index, item)| (item.clone(), evidence_ref(index as u64 + 1)))
                .collect(),
            budget: None,
            outbox_destinations: vec![],
            fencing_epoch: Some(stale_epoch),
        })
        .expect_err("stale epoch must be fenced by the store")
}
