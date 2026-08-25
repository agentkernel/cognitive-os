//! M4 crash-point and recovery behavior (acceptance criteria 1 and 6):
//! the three canonical crash points of `eff-crash-001..003.json`, driven
//! through the CrashHarness (drop-and-reopen WAL simulation) and the
//! eight-step recovery sequencer.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "m4_common/mod.rs"]
mod m4_common;

use cognitive_domain::{LifecycleDomain, Version};
use cognitive_kernel::effects::mint_intent;
use cognitive_kernel::ports::{AuthorityStore, CheckpointRow, ProtocolStore};
use cognitive_kernel::recovery::{EffectDisposition, RECOVERY_ORDER, run_recovery};
use cognitive_kernel::replay_projection;
use cognitive_kernel::{VerificationRecord, VerificationStatus};
use cognitive_store::faults::{CrashHarness, ScriptedExecutor, ScriptedOutcome};
use m4_common::*;

/// EFF-CRASH-001: crash after the Intent persisted, before dispatch. The
/// recovered effect is still AUTHORIZED; recovery confirms no dispatch
/// record, re-authorization happens under the new epoch, and the effect
/// dispatches EXACTLY ONCE with the ORIGINAL idempotency key.
#[test]
fn crash_point_1_recovers_to_single_dispatch_with_the_original_key() {
    let dir = tempfile::tempdir().unwrap();
    let harness = CrashHarness::new(&dir.path().join("authority.db"));
    let clock = FixedClock::new();
    let effect_id = oid(600);

    // Pre-crash: admit effect, mint intent, authorize. NO dispatch.
    {
        let store = harness.open().unwrap();
        let ids = SeqIds::new();
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
                801,
                &effect_id,
                "refund-42-attempt-1",
                4200,
                descriptor(true, false),
            ),
        )
        .unwrap();
        let driver = protocol(&store, &clock, &ids);
        let grant = grant_for("payments.refund");
        driver
            .authorize_effect(&effect_id, Version::INITIAL, &grant, &currency(), &lease(1))
            .unwrap();
        // CRASH: everything in memory dies; only committed WAL survives.
        harness.crash(store);
    }

    // The external world: queryable, has never seen the key.
    let executor = ScriptedExecutor::queryable(2);

    // Recovery over the reopened store.
    let store = harness.recover_handle().unwrap();
    let ids = SeqIds::from(100);
    let driver = protocol(&store, &clock, &ids);
    let report = run_recovery(&store, lease(1), &executor, &driver).unwrap();
    assert_eq!(report.step_order, RECOVERY_ORDER, "eight steps in order");
    assert_eq!(report.new_epoch, 2);
    assert_eq!(report.fenced_epoch, 1);

    // Recovered state is AUTHORIZED with the disposition "re-dispatch once
    // with the original key".
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "AUTHORIZED");
    assert_eq!(
        report.reconciled,
        vec![(
            effect_id.clone(),
            EffectDisposition::ReadyToRedispatchOriginalKey {
                idempotency_key: "refund-42-attempt-1".to_owned(),
            }
        )]
    );

    // Post-recovery: re-authorize under the new epoch and dispatch once.
    // The intent is NOT re-minted (same durable row), the key is reused.
    executor.trust_epoch(2);
    let grant = grant_for("payments.refund");
    let (committed, outcome) = driver
        .dispatch_effect(
            &effect_id,
            stored.version,
            &grant,
            &currency(),
            &executor,
            &lease(2),
        )
        .unwrap();
    driver
        .record_outcome(&effect_id, committed.after_version, &outcome, &lease(2))
        .unwrap();

    let dispatches = executor.dispatches();
    assert_eq!(
        dispatches.len(),
        1,
        "dispatched exactly once across crash+recovery"
    );
    assert_eq!(dispatches[0].idempotency_key, "refund-42-attempt-1");
    assert_eq!(
        executor.executed_keys(),
        vec!["refund-42-attempt-1".to_owned()]
    );
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "EXECUTED");
}

/// EFF-CRASH-002: crash after external execution, before the outcome was
/// persisted. The recovered effect is EXECUTING -> OUTCOME_UNKNOWN;
/// reconciliation queries the executor with the original key, confirms
/// execution, and continues to verification — never a blind retry.
#[test]
fn crash_point_2_reconciles_never_blind_retries() {
    let dir = tempfile::tempdir().unwrap();
    let harness = CrashHarness::new(&dir.path().join("authority.db"));
    let clock = FixedClock::new();
    let effect_id = oid(610);
    let executor = ScriptedExecutor::queryable(1);
    executor.script(&[ScriptedOutcome::ExecuteThenTimeout]);

    {
        let store = harness.open().unwrap();
        let ids = SeqIds::new();
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
                811,
                &effect_id,
                "refund-43-attempt-1",
                4300,
                descriptor(true, false),
            ),
        )
        .unwrap();
        let driver = protocol(&store, &clock, &ids);
        let grant = grant_for("payments.refund");
        let v = driver
            .authorize_effect(&effect_id, Version::INITIAL, &grant, &currency(), &lease(1))
            .unwrap()
            .after_version;
        // Dispatch: the external side effect HAPPENS, but the process dies
        // before recording any outcome (the timeout was never even seen).
        let (_committed, _outcome) = driver
            .dispatch_effect(&effect_id, v, &grant, &currency(), &executor, &lease(1))
            .unwrap();
        harness.crash(store);
    }
    assert_eq!(
        executor.executed_keys(),
        vec!["refund-43-attempt-1".to_owned()],
        "the external world already executed"
    );

    let store = harness.recover_handle().unwrap();
    let ids = SeqIds::from(100);
    let driver = protocol(&store, &clock, &ids);
    executor.trust_epoch(2);
    let report = run_recovery(&store, lease(1), &executor, &driver).unwrap();
    assert_eq!(report.step_order, RECOVERY_ORDER);

    // Reconciled to EXECUTED via the original-key query; no re-dispatch.
    assert_eq!(
        report.reconciled,
        vec![(effect_id.clone(), EffectDisposition::ReconciledExecuted)]
    );
    assert_eq!(
        executor.queries(),
        vec!["refund-43-attempt-1".to_owned()],
        "reconciliation queried the original key"
    );
    assert_eq!(
        executor.dispatches().len(),
        1,
        "no second dispatch: crash recovery never blind-retries"
    );
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        stored.state.as_str(),
        "RECONCILED",
        "next gate: verification"
    );
    assert_eq!(executor.executed_keys().len(), 1, "exactly one side effect");
}

/// EFF-CRASH-003: crash after verification passed, before the commit. The
/// recovered effect is VERIFIED; recovery re-checks the verification
/// currency and commits from evidence WITHOUT re-executing anything.
#[test]
fn crash_point_3_commits_from_evidence_without_reexecution() {
    let dir = tempfile::tempdir().unwrap();
    let harness = CrashHarness::new(&dir.path().join("authority.db"));
    let clock = FixedClock::new();
    let effect_id = oid(620);
    let subject_id = oid(621);
    let executor = ScriptedExecutor::queryable(1);

    {
        let store = harness.open().unwrap();
        let ids = SeqIds::new();
        admit(
            &store,
            &clock,
            &ids,
            &effect_id,
            LifecycleDomain::Effect,
            Some(1),
        );
        admit(
            &store,
            &clock,
            &ids,
            &subject_id,
            LifecycleDomain::Task,
            Some(1),
        );
        mint_intent(
            &store,
            &clock,
            &ids,
            &lease(1),
            &intent_command(
                821,
                &effect_id,
                "refund-44-attempt-1",
                4400,
                descriptor(true, false),
            ),
        )
        .unwrap();
        let driver = protocol(&store, &clock, &ids);
        let grant = grant_for("payments.refund");
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
        let record = VerificationRecord {
            verification_object_id: oid(622),
            report_id: oid(623),
            status: VerificationStatus::Passed,
            subject_domain: LifecycleDomain::Task,
            subject_object_id: subject_id.clone(),
            fixed_post_state_version: Version::INITIAL,
        };
        driver
            .verify_effect(&effect_id, reconciled.after_version, &record, &lease(1))
            .unwrap();
        // CRASH before the commit decision.
        harness.crash(store);
    }

    let store = harness.recover_handle().unwrap();
    let ids = SeqIds::from(100);
    let driver = protocol(&store, &clock, &ids);
    executor.trust_epoch(2);
    let report = run_recovery(&store, lease(1), &executor, &driver).unwrap();
    assert_eq!(report.step_order, RECOVERY_ORDER);
    // A VERIFIED effect is not in-flight: nothing to reconcile.
    assert!(report.reconciled.is_empty());
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "VERIFIED", "recovered state");

    // Commit from evidence under the new epoch: verification currency is
    // re-checked against the reloaded subject; nothing re-executes.
    let record = VerificationRecord {
        verification_object_id: oid(622),
        report_id: oid(623),
        status: VerificationStatus::Passed,
        subject_domain: LifecycleDomain::Task,
        subject_object_id: subject_id.clone(),
        fixed_post_state_version: Version::INITIAL,
    };
    let grant = grant_for("payments.refund");
    driver
        .commit_effect(
            &effect_id,
            stored.version,
            &record,
            &grant,
            &currency(),
            &uri("authority://tenant-a/effect-authority"),
            &lease(2),
        )
        .unwrap();
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "COMMITTED");
    assert_eq!(
        executor.dispatches().len(),
        1,
        "external action was NOT re-executed during commit recovery"
    );

    // Negative twin: if the fixed post-state CHANGED during the outage,
    // the commit is blocked (verification no longer current).
    let effect2 = oid(630);
    admit(
        &store,
        &clock,
        &ids,
        &effect2,
        LifecycleDomain::Effect,
        Some(2),
    );
    // Reuse the flow up to VERIFIED against subject_id, then move the
    // subject forward so the binding goes stale.
    mint_intent(
        &store,
        &clock,
        &ids,
        &lease(2),
        &intent_command(
            831,
            &effect2,
            "refund-45-attempt-1",
            4500,
            descriptor(true, false),
        ),
    )
    .unwrap();
    let v = driver
        .authorize_effect(&effect2, Version::INITIAL, &grant, &currency(), &lease(2))
        .unwrap()
        .after_version;
    let (committed, outcome) = driver
        .dispatch_effect(&effect2, v, &grant, &currency(), &executor, &lease(2))
        .unwrap();
    let v = driver
        .record_outcome(&effect2, committed.after_version, &outcome, &lease(2))
        .unwrap()
        .after_version;
    let (reconciled, _) = driver
        .reconcile(&effect2, "EXECUTED", v, &executor, &lease(2))
        .unwrap();
    let record2 = VerificationRecord {
        verification_object_id: oid(632),
        report_id: oid(633),
        status: VerificationStatus::Passed,
        subject_domain: LifecycleDomain::Task,
        subject_object_id: subject_id.clone(),
        fixed_post_state_version: Version::INITIAL,
    };
    driver
        .verify_effect(&effect2, reconciled.after_version, &record2, &lease(2))
        .unwrap();
    // The subject moves on: DRAFT -> READY bumps its version.
    drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &subject_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
        Version::INITIAL,
        Some(2),
    );
    let refused = driver
        .commit_effect(
            &effect2,
            reconciled.after_version.next().unwrap(),
            &record2,
            &grant,
            &currency(),
            &uri("authority://tenant-a/effect-authority"),
            &lease(2),
        )
        .expect_err("post-state changed: commit blocked pending re-verification");
    let rejection = match refused {
        cognitive_kernel::effects::EffectError::Rejected(rejection) => rejection,
        other => panic!("unexpected {other:?}"),
    };
    assert_eq!(rejection.registered().code, "STATE_CONFLICT");
    assert!(rejection.detail.contains("verification_still_current"));
    let stored = store
        .load_object(LifecycleDomain::Effect, &effect2)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "VERIFIED", "blocked, not committed");
}

/// The recovered projection is byte-stable across the crash boundary, and
/// checkpoints from the crashed epoch validate only against the NEW epoch
/// (recovery order facts, criterion 6 support).
#[test]
fn recovery_replays_committed_history_and_validates_checkpoints() {
    let dir = tempfile::tempdir().unwrap();
    let harness = CrashHarness::new(&dir.path().join("authority.db"));
    let clock = FixedClock::new();
    let loop_id = oid(640);

    let pre_crash_digest;
    {
        let store = harness.open().unwrap();
        let ids = SeqIds::new();
        admit(
            &store,
            &clock,
            &ids,
            &loop_id,
            LifecycleDomain::Loop,
            Some(1),
        );
        drive(
            &store,
            &clock,
            &ids,
            LifecycleDomain::Loop,
            &loop_id,
            "START",
            "OBSERVE",
            "LOOP_STARTED",
            Version::INITIAL,
            Some(1),
        );
        let projection = replay_projection(&store).unwrap();
        pre_crash_digest = projection.digest.clone();
        store
            .append_checkpoint(&CheckpointRow {
                checkpoint_id: oid(641),
                loop_object_id: loop_id.clone(),
                event_high_watermark: projection.high_watermark,
                fencing_epoch: 1,
                canonical_json: "{\"phase\":\"OBSERVE\"}".to_owned(),
            })
            .unwrap();
        harness.crash(store);
    }

    let store = harness.recover_handle().unwrap();
    let ids = SeqIds::from(100);
    let driver = protocol(&store, &clock, &ids);
    let executor = ScriptedExecutor::queryable(2);
    let report = run_recovery(&store, lease(1), &executor, &driver).unwrap();

    // Replay reproduced the committed history byte-for-byte.
    assert_eq!(report.projection_digest, pre_crash_digest);
    // The loop's checkpoint validated against the new epoch and the loop
    // is resumable.
    assert_eq!(report.resumable_loops, vec![loop_id.clone()]);
    // The crashed writer cannot append a checkpoint anymore (fenced), and
    // a checkpoint claiming the CURRENT epoch fails validation for the
    // NEXT recovery (guards against skipped fencing).
    assert!(
        store
            .append_checkpoint(&CheckpointRow {
                checkpoint_id: oid(642),
                loop_object_id: loop_id.clone(),
                event_high_watermark: report.replayed_events as i64,
                fencing_epoch: 1,
                canonical_json: "{}".to_owned(),
            })
            .is_err()
    );
}
