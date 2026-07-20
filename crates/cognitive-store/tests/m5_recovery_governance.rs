//! M5 recovery steps 6/7 as orchestrable facts (M4 handoff §2 leftover;
//! REQ-REC-001/002 continuation side): the recovery report now carries
//! the step-6 reauthorization obligations (durable intent bindings of
//! every non-terminal continuation) and the step-7 context rebinding, and
//! these tests prove the teeth — a pre-crash grant is not admissible
//! material under advanced governance facts, and a pre-crash context
//! cache binding cannot be served, only purged.
//!
//! Also the D-018 KRN cooperation fact: outbox rows resolve to their
//! committed event values through the new `load_event_by_id` port (the
//! M5 Lane-RUN envelope assembler input).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "m4_common/mod.rs"]
mod m4_common;

use cognitive_domain::capability::{CapabilityConstraints, LeaseWindow};
use cognitive_domain::{LifecycleDomain, Version};
use cognitive_kernel::authz::{AccessRequest, authorize};
use cognitive_kernel::context_cache::{
    CachedView, ContextViewCache, DerivedCacheKind, GovernanceBinding,
};
use cognitive_kernel::effects::{EffectError, GovernanceCurrency, mint_intent};
use cognitive_kernel::ports::{AuthorityStore, ProtocolStore};
use cognitive_kernel::{
    AdmitCommand, EffectDisposition, RejectionKind, TransitionEngine, reauthorization_satisfied,
    run_recovery,
};
use cognitive_store::SqliteAuthorityStore;
use cognitive_store::faults::{ScriptedExecutor, ScriptedOutcome};
use m4_common::*;
use serde_json::json;

fn fresh_store(dir: &tempfile::TempDir) -> SqliteAuthorityStore {
    SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap()
}

/// A grant decided under ADVANCED governance facts (revocation epoch 42):
/// what step 6 requires continuations to obtain freshly.
fn fresh_grant_at_epoch_42() -> cognitive_kernel::AuthorizationGrant {
    let mut snap = snapshot(&["payments.refund"]);
    snap.revocation_epoch = 42;
    snap.capability_links = vec![CapabilityConstraints {
        issued_epoch: 42,
        lease: LeaseWindow {
            not_before: ts("2026-07-20T11:00:00Z"),
            expires: ts("2026-07-20T14:00:00Z"),
        },
        ..capability_link(&["payments.refund"])
    }];
    authorize(
        &snap,
        &payments_target(),
        &AccessRequest {
            action: "payments.refund".to_owned(),
            purpose: "refund_processing".to_owned(),
        },
    )
    .unwrap()
}

fn binding(revocation_epoch: i64, capability_set_version: i64) -> GovernanceBinding {
    GovernanceBinding {
        tenant: "tenant-a".to_owned(),
        actor_chain_digest: format!("sha256:{}", "aa11".repeat(16)),
        capability_set_version,
        revocation_epoch,
        purpose: "refund_processing".to_owned(),
        schema_digest: format!("sha256:{}", "cc22".repeat(16)),
        encoding_profile: "rfc8785-utf8".to_owned(),
        conversation: Some("conversation://tenant-a/thread-1".to_owned()),
    }
}

/// Steps 6/7 as report facts with teeth: recovery lists the durable
/// authorization bindings of every non-terminal continuation; the
/// pre-crash grant fails the step-6 arithmetic AND the dispatch guard
/// under advanced governance facts, while a freshly decided grant passes;
/// the report's context rebinding marks the epoch continuations must
/// re-resolve under.
#[test]
fn recovery_reports_reauthorization_obligations_and_old_grants_cannot_continue() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let driver = protocol(&store, &clock, &ids);
    let old_grant = grant_for("payments.refund");

    // Crash point 1 shape: intent persisted + authorized, never
    // dispatched (the continuation that MUST re-authorize before its
    // single original-key re-dispatch).
    let effect_id = oid(600);
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
            601,
            &effect_id,
            "recovery-refund-attempt-1",
            7000,
            descriptor(true, false),
        ),
    )
    .unwrap();
    let authorized = driver
        .authorize_effect(
            &effect_id,
            Version::INITIAL,
            &old_grant,
            &currency(),
            &lease(1),
        )
        .unwrap();

    // The writer crashes; recovery runs under a fresh process view.
    let executor = ScriptedExecutor::queryable(1);
    let report = run_recovery(&store, lease(1), &executor, &driver).unwrap();
    assert_eq!(report.new_epoch, 2);
    assert_eq!(report.fenced_epoch, 1);

    // Step 5 disposition: ready for ONE original-key re-dispatch.
    assert_eq!(report.reconciled.len(), 1);
    assert!(matches!(
        &report.reconciled[0].1,
        EffectDisposition::ReadyToRedispatchOriginalKey { idempotency_key }
            if idempotency_key == "recovery-refund-attempt-1"
    ));

    // Step 6 fact: the obligation carries the durable pre-crash binding.
    assert_eq!(report.reauthorization_obligations.len(), 1);
    let obligation = &report.reauthorization_obligations[0];
    assert_eq!(obligation.effect_object_id, effect_id);
    assert_eq!(obligation.idempotency_key, "recovery-refund-attempt-1");
    assert_eq!(obligation.grant_epoch, 41);
    assert_eq!(obligation.capability_set_version, 7);

    // Step 7 fact: the rebinding continuations resolve under.
    assert_eq!(report.context_rebinding.fenced_epoch, 1);
    assert_eq!(report.context_rebinding.new_epoch, 2);

    // Step 6 arithmetic: governance advanced during the outage
    // (revocation epoch 41 -> 42). The pre-crash grant is NOT admissible;
    // a freshly decided grant is.
    let advanced = GovernanceCurrency {
        revocation_epoch: 42,
        capability_set_version: 7,
    };
    let now = ts("2026-07-20T12:00:00Z");
    assert!(!reauthorization_satisfied(
        obligation, &old_grant, &advanced, &now
    ));
    let fresh_grant = fresh_grant_at_epoch_42();
    assert!(reauthorization_satisfied(
        obligation,
        &fresh_grant,
        &advanced,
        &now
    ));

    // The teeth at the dispatch gate: re-dispatch under the OLD grant
    // fails the capability_and_revocation_current guard closed — zero
    // executor calls; the FRESH grant dispatches exactly once with the
    // ORIGINAL key.
    executor.trust_epoch(2);
    executor.script(&[ScriptedOutcome::Execute]);
    let stale = driver
        .dispatch_effect(
            &effect_id,
            authorized.after_version,
            &old_grant,
            &advanced,
            &executor,
            &lease(2),
        )
        .expect_err("a pre-crash grant must not continue after recovery");
    match stale {
        EffectError::Rejected(rejection) => {
            assert_eq!(rejection.kind, RejectionKind::GuardUnsatisfied);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(
        executor.dispatches().is_empty(),
        "no external call under the stale grant"
    );

    driver
        .dispatch_effect(
            &effect_id,
            authorized.after_version,
            &fresh_grant,
            &advanced,
            &executor,
            &lease(2),
        )
        .unwrap();
    let calls = executor.dispatches();
    assert_eq!(calls.len(), 1, "exactly one re-dispatch");
    assert_eq!(calls[0].idempotency_key, "recovery-refund-attempt-1");
    assert_eq!(calls[0].fencing_epoch, 2);
}

/// Step 7 teeth (M3 structural guarantee upgraded to a step fact): a
/// continuation declaring its pre-crash context binding is refused with
/// the registered denial and the stale entry plus every derived cache is
/// purged BY KEY; nothing under the current binding exists until fresh
/// resolution. (The in-memory cache of the crashed process died with it;
/// this covers the declared-binding replay path.)
#[test]
fn stale_context_bindings_are_refused_and_purged_after_recovery() {
    let mut cache = ContextViewCache::default();
    let pre_crash = binding(41, 7);
    cache.insert(
        pre_crash.clone(),
        CachedView {
            render_digest: format!("sha256:{}", "dd33".repeat(16)),
            loaded_refs: vec!["state://tenant-a/orders/17".to_owned()],
            derived: vec![DerivedCacheKind::KvCache, DerivedCacheKind::Summary],
        },
    );
    assert_eq!(cache.len(), 1);

    // Governance advanced during the outage: the current binding differs
    // in the revocation epoch component.
    let current = binding(42, 7);

    let (denial, invalidation) = cache
        .serve_declared(&pre_crash, &current)
        .expect_err("a pre-crash binding must not be served");
    assert_eq!(denial.code, "CONTEXT_AUTH_DENIED");
    let invalidation = invalidation.expect("the stale entry is purged with a report");
    assert_eq!(invalidation.stale_binding, pre_crash);
    assert_eq!(
        invalidation.derived_caches_invalidated,
        vec![DerivedCacheKind::KvCache, DerivedCacheKind::Summary],
        "derived caches die with the entry"
    );
    assert!(cache.is_empty(), "purged by key, not by scan");

    // Fresh resolution under the current binding is the only path.
    assert!(cache.lookup_current(&current).is_none());
}

/// D-018 cooperation fact: an outbox row resolves to its committed event
/// value through `load_event_by_id` — the M5 runtime envelope assembler
/// reads (event_type, subject, causation, event_time, payload bytes)
/// without scanning the log. No new outbox columns are needed.
#[test]
fn outbox_rows_resolve_to_committed_events_for_envelope_assembly() {
    let dir = tempfile::tempdir().unwrap();
    let store = fresh_store(&dir);
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);
    let object_id = oid(700);
    engine
        .admit_object(&AdmitCommand {
            object_id: object_id.clone(),
            domain: LifecycleDomain::Task,
            subject_ref: uri("task://tenant-a/envelope-subject"),
            body: json!({"m5": "outbox"}),
            actor_ref: uri("actor://tenant-a/agent-1"),
            authority_ref: uri("authority://tenant-a/state-authority"),
            correlation_id: uri("corr://tenant-a/m5-outbox"),
            outbox_destinations: vec!["watch".to_owned()],
            fencing_epoch: None,
        })
        .unwrap();

    let pending = store.pending_outbox(10).unwrap();
    assert_eq!(pending.len(), 1);
    let entry = &pending[0];
    assert_eq!(entry.destination, "watch");

    let event = store
        .load_event_by_id(&entry.event_id)
        .unwrap()
        .expect("the outbox row's event is committed by the same transaction");
    assert_eq!(event.event_id, entry.event_id);
    assert_eq!(event.object_id, object_id);
    let value: serde_json::Value = serde_json::from_str(&event.canonical_json).unwrap();
    // The envelope assembler's fact surface (D-018 ruling: header refs
    // come from M5 governance objects; delivery fields from the watch
    // profile constants; everything event-side is already here).
    assert_eq!(value["event_type"], "cognitiveos.object.admitted");
    assert_eq!(value["subject_ref"], "task://tenant-a/envelope-subject");
    assert!(value["causation"]["correlation_id"].is_string());
    assert!(value["event_time"].is_string());

    // Unknown ids resolve to None (the assembler fails closed, no guess).
    assert!(
        store
            .load_event_by_id(
                &cognitive_domain::EventId::parse("00000000-0000-7000-8000-00000000dead").unwrap()
            )
            .unwrap()
            .is_none()
    );
}
