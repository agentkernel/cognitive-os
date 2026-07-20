//! M4 acceptance criterion 7: the tracer-bullet end-to-end vertical slice.
//!
//! One minimal legal chain runs across REAL components on one node —
//! UserIntent (pinned record reference) → M3 authorization gate → Intent
//! minting (canonical parameter digest, stable idempotency key) → Effect
//! protocol (authorize → dispatch through the executor sink → outcome →
//! reconcile) → Verification OBJECT lifecycle (its own registered machine)
//! → Effect verify + commit → Task acceptance by the acceptance authority.
//! A negative chain proves the same machinery refuses completion without a
//! passing verification.
//!
//! Evidence is written to
//! `artifacts/evidence/faults/tracer-bullet-evidence.json` (gitignored;
//! regenerate with `cargo test -p cognitive-store --test m4_tracer_bullet`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "m4_common/mod.rs"]
mod m4_common;

use cognitive_domain::{LifecycleDomain, Version};
use cognitive_kernel::effects::mint_intent;
use cognitive_kernel::ports::AuthorityStore;
use cognitive_kernel::replay_projection;
use cognitive_kernel::{VerificationRecord, VerificationStatus};
use cognitive_store::SqliteAuthorityStore;
use cognitive_store::faults::ScriptedExecutor;
use m4_common::*;
use serde_json::json;

fn load_state(
    store: &SqliteAuthorityStore,
    domain: LifecycleDomain,
    id: &cognitive_domain::ObjectId,
) -> (String, Version) {
    let object = store.load_object(domain, id).unwrap().unwrap();
    (object.state.as_str().to_owned(), object.version)
}

#[test]
fn tracer_bullet_intent_to_acceptance_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let driver = protocol(&store, &clock, &ids);
    let mut chain: Vec<serde_json::Value> = Vec::new();

    // --- 0. Pinned user intent record (the fixed origin of the chain;
    // semantic interpretation is a candidate producer and stays out of the
    // deterministic path).
    let user_intent_ref = "user-intent://tenant-a/uir-9001?version=1";
    chain.push(json!({"step": "user_intent_pinned", "ref": user_intent_ref}));

    // --- 1. M3 authorization gate issues the grant.
    let grant = grant_for("payments.refund");
    chain.push(json!({
        "step": "authorized",
        "epoch": grant.decided_at_epoch,
        "capability_set_version": grant.capability_set_version,
    }));

    // --- 2. Task lifecycle to ACTIVE (task authority machine).
    let task_id = oid(900);
    admit(
        &store,
        &clock,
        &ids,
        &task_id,
        LifecycleDomain::Task,
        Some(1),
    );
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
        Some(1),
    );
    let task_v = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &task_id,
        "READY",
        "ACTIVE",
        "EXECUTION_STARTED",
        v,
        Some(1),
    );
    chain.push(json!({"step": "task_active", "task": task_id.as_str(), "version": task_v.get()}));

    // --- 3. Intent minted: canonical digest, stable key, one transaction.
    let effect_id = oid(901);
    admit(
        &store,
        &clock,
        &ids,
        &effect_id,
        LifecycleDomain::Effect,
        Some(1),
    );
    let minted = mint_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &intent_command(
            902,
            &effect_id,
            "tracer-refund-1",
            4200,
            descriptor(true, false),
        ),
    )
    .unwrap();
    let intent = match minted {
        cognitive_kernel::MintedIntent::Persisted(row) => row,
        other => panic!("fresh chain: {other:?}"),
    };
    chain.push(json!({
        "step": "intent_persisted",
        "intent": intent.intent_id.as_str(),
        "idempotency_key": intent.idempotency_key,
        "parameters_digest": intent.parameters_digest,
    }));

    // --- 4. Effect protocol: authorize -> dispatch -> outcome -> reconcile.
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
    let (reconciled, query) = driver
        .reconcile(&effect_id, "EXECUTED", v, &executor, &lease(1))
        .unwrap();
    chain.push(json!({
        "step": "effect_reconciled",
        "effect": effect_id.as_str(),
        "query": format!("{query:?}"),
        "executor_ledger": executor.executed_keys(),
    }));

    // --- 5. Verification OBJECT lifecycle (registered machine): the
    // verification is requested against the FIXED task post-state, its
    // evidence closes, and the verifier authority passes it.
    let verification_id = oid(903);
    admit(
        &store,
        &clock,
        &ids,
        &verification_id,
        LifecycleDomain::Verification,
        Some(1),
    );
    let vv = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Verification,
        &verification_id,
        "NOT_REQUESTED",
        "PENDING",
        "VERIFICATION_REQUESTED",
        Version::INITIAL,
        Some(1),
    );
    let vv = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Verification,
        &verification_id,
        "PENDING",
        "EVIDENCE_READY",
        "EVIDENCE_COLLECTION_COMPLETED",
        vv,
        Some(1),
    );
    let vv = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Verification,
        &verification_id,
        "EVIDENCE_READY",
        "PASSED",
        "CRITERIA_PASSED",
        vv,
        Some(1),
    );
    let (verification_state, _) =
        load_state(&store, LifecycleDomain::Verification, &verification_id);
    assert_eq!(verification_state, "PASSED");
    chain.push(json!({
        "step": "verification_passed",
        "verification": verification_id.as_str(),
        "version": vv.get(),
    }));

    // --- 6. Effect verify + commit from the verification evidence.
    let record = VerificationRecord {
        verification_object_id: verification_id.clone(),
        report_id: oid(904),
        status: VerificationStatus::Passed,
        subject_domain: LifecycleDomain::Task,
        subject_object_id: task_id.clone(),
        fixed_post_state_version: task_v,
    };
    let v = driver
        .verify_effect(&effect_id, reconciled.after_version, &record, &lease(1))
        .unwrap()
        .after_version;
    driver
        .commit_effect(
            &effect_id,
            v,
            &record,
            &grant,
            &currency(),
            &uri("authority://tenant-a/effect-authority"),
            &lease(1),
        )
        .unwrap();
    let (effect_state, _) = load_state(&store, LifecycleDomain::Effect, &effect_id);
    assert_eq!(effect_state, "COMMITTED");
    chain.push(json!({"step": "effect_committed", "effect": effect_id.as_str()}));

    // --- 7. Task acceptance by the acceptance AUTHORITY: guards derived
    // from reloaded authority state (verification object PASSED, fixed
    // post-state unchanged) — never from the receipt or executor output.
    let v = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &task_id,
        "ACTIVE",
        "CANDIDATE_COMPLETE",
        "COMPLETION_CLAIMED",
        task_v,
        Some(1),
    );
    let (verification_state, _) =
        load_state(&store, LifecycleDomain::Verification, &verification_id);
    let subject_unchanged = store
        .load_object(LifecycleDomain::Task, &task_id)
        .unwrap()
        .unwrap()
        .version
        == v;
    assert_eq!(verification_state, "PASSED");
    assert!(subject_unchanged);
    let final_v = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &task_id,
        "CANDIDATE_COMPLETE",
        "COMPLETED",
        "ACCEPTANCE_GRANTED",
        v,
        Some(1),
    );
    let (task_state, _) = load_state(&store, LifecycleDomain::Task, &task_id);
    assert_eq!(task_state, "COMPLETED");
    chain.push(json!({
        "step": "task_completed_by_acceptance_authority",
        "task": task_id.as_str(),
        "version": final_v.get(),
    }));

    // --- 8. Whole-chain evidence: replay digest stability across the
    // chain, executor ledger exactly one execution, event log intact.
    let projection = replay_projection(&store).unwrap();
    let replayed_again = replay_projection(&store).unwrap();
    assert_eq!(projection.digest, replayed_again.digest);
    assert_eq!(executor.dispatches().len(), 1);
    chain.push(json!({
        "step": "chain_evidence",
        "projection_digest": projection.digest,
        "event_count": projection.event_count,
        "high_watermark": projection.high_watermark,
    }));

    // --- Negative chain: a second task/effect WITHOUT a passing
    // verification cannot complete; the acceptance guard is underivable.
    let task2 = oid(910);
    admit(&store, &clock, &ids, &task2, LifecycleDomain::Task, Some(1));
    let v = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &task2,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
        Version::INITIAL,
        Some(1),
    );
    let v = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &task2,
        "READY",
        "ACTIVE",
        "EXECUTION_STARTED",
        v,
        Some(1),
    );
    let v = drive(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &task2,
        "ACTIVE",
        "CANDIDATE_COMPLETE",
        "COMPLETION_CLAIMED",
        v,
        Some(1),
    );
    let pending_verification = oid(911);
    admit(
        &store,
        &clock,
        &ids,
        &pending_verification,
        LifecycleDomain::Verification,
        Some(1),
    );
    // The verification never passed: attesting the guard would be a lie —
    // derive it from the reloaded state instead, and refuse to attest.
    let (verification_state, _) =
        load_state(&store, LifecycleDomain::Verification, &pending_verification);
    assert_ne!(verification_state, "PASSED");
    let engine = cognitive_kernel::TransitionEngine::new(&store, &clock, &ids);
    let loaded = cognitive_domain::table(LifecycleDomain::Task).unwrap();
    let edge = loaded
        .find_edge(
            &state("CANDIDATE_COMPLETE"),
            &state("COMPLETED"),
            "ACCEPTANCE_GRANTED",
        )
        .unwrap();
    let mut established: std::collections::BTreeSet<String> = edge.guards.iter().cloned().collect();
    established.remove("verification_passed_and_current"); // underivable
    let rejected = engine
        .commit_transition(&cognitive_kernel::TransitionCommand {
            request_id: uri("request://m4/tracer-negative"),
            domain: LifecycleDomain::Task,
            object_id: task2.clone(),
            subject_ref: uri("task://tenant-a/task-910"),
            from: state("CANDIDATE_COMPLETE"),
            to: state("COMPLETED"),
            expected_version: v,
            reason: cognitive_kernel::Reason {
                code: cognitive_domain::ReasonCode::parse("ACCEPTANCE_GRANTED").unwrap(),
                detail: None,
            },
            causation: cognitive_kernel::Causation {
                causation_id: uri("corr://tenant-a/m4-chain"),
                correlation_id: uri("corr://tenant-a/m4-chain"),
            },
            actor_ref: uri("actor://tenant-a/agent-1"),
            authority_ref: uri("authority://tenant-a/task-acceptance"),
            requested_at: ts("2026-07-20T12:05:00Z"),
            table_pin: cognitive_kernel::TablePin::current(LifecycleDomain::Task).unwrap(),
            established_guards: established,
            evidence: edge
                .required_evidence
                .iter()
                .enumerate()
                .map(|(index, item)| (item.clone(), evidence_ref(index as u64 + 1)))
                .collect(),
            budget: None,
            outbox_destinations: vec![],
            fencing_epoch: Some(1),
        })
        .expect_err("no passing verification, no completion");
    assert_eq!(rejected.registered().code, "STATE_CONFLICT");
    let (task2_state, _) = load_state(&store, LifecycleDomain::Task, &task2);
    assert_eq!(task2_state, "CANDIDATE_COMPLETE");
    chain.push(json!({
        "step": "negative_chain_completion_refused",
        "task": task2.as_str(),
        "final_state": task2_state,
    }));

    // --- Evidence artifact.
    let evidence_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("artifacts")
        .join("evidence")
        .join("faults");
    std::fs::create_dir_all(&evidence_dir).unwrap();
    let evidence = json!({
        "tracer_bullet": "intent-to-acceptance",
        "generated_by": "cargo test -p cognitive-store --test m4_tracer_bullet",
        "chain": chain,
    });
    std::fs::write(
        evidence_dir.join("tracer-bullet-evidence.json"),
        serde_json::to_string_pretty(&evidence).unwrap(),
    )
    .unwrap();
}
