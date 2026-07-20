//! M2 acceptance suite (DEVELOPMENT-PLAN M2 判据 1-5 + crash consistency),
//! executed against the real SQLite (WAL) authority store through the
//! deterministic kernel gate.
//!
//! Criterion -> test mapping:
//!
//! 1. Concurrent CAS, exactly one winner ............ `criterion_1_*`
//! 2. Illegal transitions all rejected, state unchanged,
//!    registry-consistent codes (all five tables) ... `criterion_2_*`
//! 3. Replay digest stability ....................... `criterion_3_*`
//! 4. Events immutable in storage (UPDATE/DELETE) ... `criterion_4_*`
//! 5. Hard budget fail-closed + same-transaction .... `criterion_5_*`
//! 6. Mid-transaction failure leaves no partial
//!    commit; read-only store fails closed .......... `criterion_6_*`

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_contracts::canonical;
use cognitive_contracts::generated::common_defs::Digest;
use cognitive_contracts::generated::object_reference::{
    StrongReference, StrongReferenceKind, UuidV7,
};
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef, Version,
    WallTimestamp, table,
};
use cognitive_kernel::ports::{
    AuthorityStore, Clock, EventDraft, IdGenerator, ObjectAdmission, PortFailure, StoredObject,
};
use cognitive_kernel::{
    AdmitCommand, BudgetCharge, BudgetChargeCommand, BudgetState, Causation, Reason, TablePin,
    TransitionCommand, TransitionEngine, replay_projection,
};
use cognitive_store::{SqliteAuthorityStore, SystemClock, UuidV7Generator};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Barrier, Mutex};

// ---------------------------------------------------------------------
// Deterministic test adapters and helpers
// ---------------------------------------------------------------------

struct FixedClock(WallTimestamp);

impl FixedClock {
    fn new() -> Self {
        Self(WallTimestamp::parse("2026-07-20T06:00:00Z").unwrap())
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        Ok(self.0.clone())
    }
}

/// Deterministic sequential UUIDv7-shaped IDs (reproducible histories).
struct SeqIds(AtomicU64);

impl SeqIds {
    fn new() -> Self {
        Self(AtomicU64::new(1))
    }

    fn uuid_for(n: u64) -> String {
        format!("00000000-0000-7000-8000-{n:012x}")
    }
}

impl IdGenerator for SeqIds {
    fn next_uuid_v7(&self) -> Result<String, PortFailure> {
        Ok(Self::uuid_for(self.0.fetch_add(1, Ordering::SeqCst)))
    }
}

fn oid(n: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{n:012x}")).unwrap()
}

fn uri(text: &str) -> UriRef {
    UriRef::parse(text).unwrap()
}

fn state(name: &str) -> StateName {
    StateName::parse(name).unwrap()
}

fn strong_ref(n: u64) -> StrongReference {
    StrongReference {
        content_digest: Digest(format!(
            "sha256:{}",
            format!("{n:x}").repeat(64)[..64].to_owned()
        )),
        id: UuidV7(format!("00000000-0000-7000-a000-{n:012x}")),
        kind: StrongReferenceKind::Strong,
        object_version: 1,
    }
}

/// Seed one governed object at an arbitrary state through the port (models
/// an object whose committed history reached that state), with a
/// well-formed admission event.
fn seed(store: &SqliteAuthorityStore, id: &ObjectId, domain: LifecycleDomain, at: &str) {
    let admitted_at = WallTimestamp::parse("2026-07-20T05:00:00Z").unwrap();
    let event_id = EventId::parse(&format!(
        "00000000-0000-7000-8000-9{}",
        &id.as_str()[id.as_str().len() - 11..]
    ))
    .unwrap();
    let event_value = json!({
        "event_id": event_id.as_str(),
        "event_type": "cognitiveos.object.admitted",
        "domain": domain.as_str(),
        "object_id": id.as_str(),
        "subject_ref": format!("{}://tenant-test/{}", domain.as_str(), id.as_str()),
        "after_state": at,
        "after_version": 1,
        "event_time": admitted_at.as_str(),
    });
    let canonical_json =
        String::from_utf8(canonical::canonical_bytes_of_value(&event_value).unwrap()).unwrap();
    store
        .admit_object(&ObjectAdmission {
            object: StoredObject {
                object_id: id.clone(),
                domain,
                state: state(at),
                version: Version::INITIAL,
                body: json!({"seeded": true}),
            },
            admitted_at,
            event: EventDraft {
                event_id: event_id.clone(),
                object_id: id.clone(),
                domain,
                object_version: Version::INITIAL,
                event_type: "cognitiveos.object.admitted".to_owned(),
                canonical_json,
            },
            outbox: vec![],
        })
        .unwrap();
}

/// Build a command for one edge, with the edge's own guards attested and
/// its required evidence supplied.
fn edge_command(
    domain: LifecycleDomain,
    object_id: &ObjectId,
    from: &str,
    to: &str,
    reason: &str,
) -> TransitionCommand {
    let loaded = table(domain).unwrap();
    let (guards, evidence_items) = match loaded.find_edge(&state(from), &state(to), reason) {
        Ok(edge) => (edge.guards.clone(), edge.required_evidence.clone()),
        Err(_) => (Vec::new(), Vec::new()),
    };
    let evidence: BTreeMap<String, StrongReference> = evidence_items
        .iter()
        .enumerate()
        .map(|(index, item)| (item.clone(), strong_ref(index as u64 + 1)))
        .collect();
    TransitionCommand {
        request_id: uri(&format!("request://m2/{}/{from}-{to}", object_id.as_str())),
        domain,
        object_id: object_id.clone(),
        subject_ref: uri(&format!("{}://tenant-test/{}", domain.as_str(), object_id)),
        from: state(from),
        to: state(to),
        expected_version: Version::INITIAL,
        reason: Reason {
            code: ReasonCode::parse(reason).unwrap(),
            detail: None,
        },
        causation: Causation {
            causation_id: uri("cause://m2/origin"),
            correlation_id: uri("corr://m2/chain-1"),
        },
        actor_ref: uri("actor://tenant-test/agent-1"),
        authority_ref: uri("authority://tenant-test/state-authority"),
        requested_at: WallTimestamp::parse("2026-07-20T05:59:00Z").unwrap(),
        table_pin: TablePin::current(domain).unwrap(),
        established_guards: guards.into_iter().collect::<BTreeSet<_>>(),
        evidence,
        budget: None,
        outbox_destinations: vec![],
    }
}

fn raw_connection(path: &Path) -> rusqlite::Connection {
    rusqlite::Connection::open(path).unwrap()
}

fn count(conn: &rusqlite::Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |row| row.get(0)).unwrap()
}

// ---------------------------------------------------------------------
// Criterion 1: concurrent CAS, exactly one winner
// ---------------------------------------------------------------------

#[test]
fn criterion_1_concurrent_cas_exactly_one_winner_others_state_conflict() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("authority.db");
    let store = SqliteAuthorityStore::open(&path).unwrap();
    let clock = SystemClock;
    let ids = UuidV7Generator;

    let object_id = oid(1);
    seed(&store, &object_id, LifecycleDomain::Task, "DRAFT");

    const WRITERS: usize = 8;
    let barrier = Barrier::new(WRITERS);
    let outcomes: Mutex<Vec<Result<Version, String>>> = Mutex::new(Vec::new());

    std::thread::scope(|scope| {
        for _ in 0..WRITERS {
            scope.spawn(|| {
                // Every writer decided against the SAME observed version.
                let cmd = edge_command(
                    LifecycleDomain::Task,
                    &object_id,
                    "DRAFT",
                    "READY",
                    "CONTRACT_ACCEPTED",
                );
                let engine = TransitionEngine::new(&store, &clock, &ids);
                barrier.wait();
                let outcome = engine
                    .commit_transition(&cmd)
                    .map(|committed| committed.after_version)
                    .map_err(|rejection| {
                        // Losers surface the registered STATE_CONFLICT code
                        // whether they lost before or inside the store CAS.
                        assert_eq!(rejection.registered().code, "STATE_CONFLICT");
                        assert!(rejection.registered().retryable);
                        rejection.detail
                    });
                outcomes.lock().unwrap().push(outcome);
            });
        }
    });

    let outcomes = outcomes.into_inner().unwrap();
    let winners = outcomes.iter().filter(|outcome| outcome.is_ok()).count();
    assert_eq!(winners, 1, "exactly one CAS winner: {outcomes:?}");

    // Authoritative row advanced exactly once; exactly one transition event
    // and one record were appended (no side effects from the losers).
    let stored = store
        .load_object(LifecycleDomain::Task, &object_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "READY");
    assert_eq!(stored.version, Version::INITIAL.next().unwrap());
    let conn = raw_connection(&path);
    assert_eq!(
        count(
            &conn,
            "SELECT COUNT(*) FROM events WHERE event_type='cognitiveos.state.transition.committed'"
        ),
        1
    );
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM transition_records"), 1);
}

// ---------------------------------------------------------------------
// Criterion 2: illegal transitions exhaustively rejected on all 5 tables
// ---------------------------------------------------------------------

#[test]
fn criterion_2_every_unregistered_pair_rejected_with_registry_codes_and_state_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("authority.db");
    let store = SqliteAuthorityStore::open(&path).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);

    let mut counter = 1u64;
    let mut rejected = 0usize;
    let mut effect_unknown_denials = 0usize;
    for domain in LifecycleDomain::ALL {
        let loaded = table(domain).unwrap();
        let legal: BTreeSet<(&str, &str)> = loaded
            .table
            .transitions
            .iter()
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect();
        for from in &loaded.table.states {
            let object_id = oid(counter);
            counter += 1;
            seed(&store, &object_id, domain, from);
            let events_before = {
                let conn = raw_connection(&path);
                count(&conn, "SELECT COUNT(*) FROM events")
            };
            for to in &loaded.table.states {
                if legal.contains(&(from.as_str(), to.as_str())) {
                    continue;
                }
                let mut cmd = edge_command(domain, &object_id, from, to, "FORCED_ILLEGAL_ATTEMPT");
                cmd.request_id = uri(&format!("request://m2/illegal/{domain}/{from}/{to}"));
                let rejection = engine
                    .commit_transition(&cmd)
                    .expect_err("illegal edge must be rejected");
                let registered = rejection.registered();
                if domain == LifecycleDomain::Effect && from == "OUTCOME_UNKNOWN" {
                    assert_eq!(registered.code, "EFFECT_OUTCOME_UNKNOWN");
                    assert_eq!(rejection.available_exits, vec!["RECONCILED"]);
                    effect_unknown_denials += 1;
                } else {
                    assert_eq!(registered.code, "STATE_CONFLICT", "{domain} {from} -> {to}");
                    assert_eq!(registered.category, "state");
                }
                rejected += 1;
            }
            // The seeded object never moved and nothing was appended.
            let stored = store.load_object(domain, &object_id).unwrap().unwrap();
            assert_eq!(stored.state.as_str(), from.as_str());
            assert_eq!(stored.version, Version::INITIAL);
            let conn = raw_connection(&path);
            assert_eq!(count(&conn, "SELECT COUNT(*) FROM events"), events_before);
            assert_eq!(count(&conn, "SELECT COUNT(*) FROM transition_records"), 0);
        }
    }
    assert!(
        rejected > 400,
        "exhaustive sweep covered {rejected} illegal pairs"
    );
    assert!(
        effect_unknown_denials > 0,
        "OUTCOME_UNKNOWN denials exercised"
    );
}

// ---------------------------------------------------------------------
// Criterion 3: replay digest stability
// ---------------------------------------------------------------------

fn drive_reference_history(path: &Path) -> SqliteAuthorityStore {
    let store = SqliteAuthorityStore::open(path).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    {
        let engine = TransitionEngine::new(&store, &clock, &ids);
        let admissions = [
            (1u64, LifecycleDomain::Task),
            (2, LifecycleDomain::Effect),
            (3, LifecycleDomain::Verification),
        ];
        for (n, domain) in admissions {
            engine
                .admit_object(&AdmitCommand {
                    object_id: oid(n),
                    domain,
                    subject_ref: uri(&format!("{}://tenant-test/{n}", domain.as_str())),
                    body: json!({"n": n}),
                    actor_ref: uri("actor://tenant-test/agent-1"),
                    authority_ref: uri("authority://tenant-test/state-authority"),
                    correlation_id: uri("corr://m2/replay"),
                    outbox_destinations: vec!["watch://status".to_owned()],
                })
                .unwrap();
        }
        // task: DRAFT -> READY -> ACTIVE
        engine
            .commit_transition(&edge_command(
                LifecycleDomain::Task,
                &oid(1),
                "DRAFT",
                "READY",
                "CONTRACT_ACCEPTED",
            ))
            .unwrap();
        let mut second = edge_command(
            LifecycleDomain::Task,
            &oid(1),
            "READY",
            "ACTIVE",
            "EXECUTION_STARTED",
        );
        second.expected_version = Version::INITIAL.next().unwrap();
        engine.commit_transition(&second).unwrap();
        // effect: PROPOSED -> AUTHORIZED
        engine
            .commit_transition(&edge_command(
                LifecycleDomain::Effect,
                &oid(2),
                "PROPOSED",
                "AUTHORIZED",
                "AUTHORIZATION_GRANTED",
            ))
            .unwrap();
        // verification: NOT_REQUESTED -> PENDING
        engine
            .commit_transition(&edge_command(
                LifecycleDomain::Verification,
                &oid(3),
                "NOT_REQUESTED",
                "PENDING",
                "VERIFICATION_REQUESTED",
            ))
            .unwrap();
    }
    store
}

#[test]
fn criterion_3_replaying_committed_history_yields_byte_identical_projection_digests() {
    let dir = tempfile::tempdir().unwrap();
    let path_a = dir.path().join("authority-a.db");
    let store_a = drive_reference_history(&path_a);

    // Replay the same committed history twice: byte-identical projection.
    let first = replay_projection(&store_a).unwrap();
    let second = replay_projection(&store_a).unwrap();
    assert_eq!(first.canonical_bytes, second.canonical_bytes);
    assert_eq!(first.digest, second.digest);
    assert_eq!(first.event_count, 7);
    assert_eq!(first.value["objects"].as_array().unwrap().len(), 3);

    // A fresh handle over the same database replays to the same digest.
    drop(store_a);
    let reopened = SqliteAuthorityStore::open(&path_a).unwrap();
    let third = replay_projection(&reopened).unwrap();
    assert_eq!(first.digest, third.digest);
    assert_eq!(first.canonical_bytes, third.canonical_bytes);

    // An independent store fed the same deterministic inputs converges on
    // the same canonical bytes and digest (REQ-STATE-002).
    let path_b = dir.path().join("authority-b.db");
    let store_b = drive_reference_history(&path_b);
    let other = replay_projection(&store_b).unwrap();
    assert_eq!(first.canonical_bytes, other.canonical_bytes);
    assert_eq!(first.digest, other.digest);
}

// ---------------------------------------------------------------------
// Criterion 4: committed events cannot be edited in place
// ---------------------------------------------------------------------

#[test]
fn criterion_4_committed_events_and_records_reject_update_and_delete() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("authority.db");
    let store = drive_reference_history(&path);
    let before = replay_projection(&store).unwrap();

    // The negative is enforced at the STORAGE layer: any connection,
    // including a brand-new raw one, is refused.
    let conn = raw_connection(&path);
    for sql in [
        "UPDATE events SET canonical_json = '{}' WHERE sequence = 1",
        "UPDATE events SET event_type = 'rewritten' WHERE 1=1",
        "DELETE FROM events WHERE sequence = 1",
        "DELETE FROM events",
        "UPDATE transition_records SET canonical_json = '{}' WHERE record_seq = 1",
        "DELETE FROM transition_records",
    ] {
        let err = conn.execute(sql, []).expect_err(sql);
        assert!(err.to_string().contains("append-only"), "{sql} -> {err}");
    }

    // Timestamp rewrite attempts are rejected identically (REQ-EVT-002:
    // clock correction creates a new linked record, never an edit).
    let err = conn
        .execute(
            "UPDATE events SET canonical_json = replace(canonical_json, '2026-07-20', '2020-01-01')",
            [],
        )
        .expect_err("timestamp rewrite");
    assert!(err.to_string().contains("append-only"));

    // History is intact and replays to the identical digest.
    let after = replay_projection(&store).unwrap();
    assert_eq!(before.digest, after.digest);
    assert_eq!(
        count(&raw_connection(&path), "SELECT COUNT(*) FROM events"),
        7
    );
}

// ---------------------------------------------------------------------
// Criterion 5: hard budget fail-closed, debit rides the same transaction
// ---------------------------------------------------------------------

#[test]
fn criterion_5_over_budget_rejected_fail_closed_and_debit_commits_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("authority.db");
    let store = SqliteAuthorityStore::open(&path).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);

    let object_id = oid(1);
    seed(&store, &object_id, LifecycleDomain::Task, "DRAFT");
    let budget_id = BudgetId::parse("00000000-0000-7000-b000-000000000001").unwrap();
    engine
        .create_budget(
            &budget_id,
            &BudgetState::new([("tool_calls".to_owned(), 1)].into()).unwrap(),
        )
        .unwrap();

    let charge = |amount: i64| {
        Some(BudgetChargeCommand {
            budget_id: budget_id.clone(),
            charge: BudgetCharge::new([("tool_calls".to_owned(), amount)].into()).unwrap(),
        })
    };

    // Over budget: deterministically rejected, nothing changed anywhere.
    let mut cmd = edge_command(
        LifecycleDomain::Task,
        &object_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
    );
    cmd.budget = charge(2);
    let rejection = engine.commit_transition(&cmd).expect_err("over budget");
    let registered = rejection.registered();
    assert_eq!(registered.code, "RESOURCE_BUDGET_EXHAUSTED");
    assert_eq!(registered.category, "resource");
    assert!(!registered.retryable, "hard budget denial is not retryable");
    let stored = store
        .load_object(LifecycleDomain::Task, &object_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        (stored.state.as_str(), stored.version),
        ("DRAFT", Version::INITIAL)
    );
    let budget = store.load_budget(&budget_id).unwrap().unwrap();
    assert_eq!(budget.version, Version::INITIAL);
    assert_eq!(budget.state.remaining()["tool_calls"], 1);
    let conn = raw_connection(&path);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM transition_records"), 0);

    // Admissible charge: object CAS and budget debit commit together.
    cmd.budget = charge(1);
    engine.commit_transition(&cmd).unwrap();
    let stored = store
        .load_object(LifecycleDomain::Task, &object_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "READY");
    let budget = store.load_budget(&budget_id).unwrap().unwrap();
    assert_eq!(budget.version, Version::INITIAL.next().unwrap());
    assert_eq!(budget.state.remaining()["tool_calls"], 0);

    // Budget drained: the next charged transition is denied fail-closed.
    let mut next = edge_command(
        LifecycleDomain::Task,
        &object_id,
        "READY",
        "ACTIVE",
        "EXECUTION_STARTED",
    );
    next.expected_version = Version::INITIAL.next().unwrap();
    next.budget = charge(1);
    let rejection = engine.commit_transition(&next).expect_err("drained budget");
    assert_eq!(rejection.registered().code, "RESOURCE_BUDGET_EXHAUSTED");
    let stored = store
        .load_object(LifecycleDomain::Task, &object_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        stored.state.as_str(),
        "READY",
        "state unchanged after denial"
    );
}

// ---------------------------------------------------------------------
// Criterion 6: crash consistency and fail-closed degradation
// ---------------------------------------------------------------------

/// Inject a failure in the MIDDLE of the commit transaction (after the
/// object CAS and budget debit statements, at the event append) and prove
/// the whole authoritative commit rolled back: no state change, no debit,
/// no event, no record, no outbox row.
#[test]
fn criterion_6_mid_transaction_failure_leaves_no_partial_commit() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("authority.db");
    let store = SqliteAuthorityStore::open(&path).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);

    let object_id = oid(1);
    seed(&store, &object_id, LifecycleDomain::Task, "DRAFT");
    let budget_id = BudgetId::parse("00000000-0000-7000-b000-000000000002").unwrap();
    engine
        .create_budget(
            &budget_id,
            &BudgetState::new([("tool_calls".to_owned(), 5)].into()).unwrap(),
        )
        .unwrap();

    // The deterministic generator will hand the engine event id 1 next.
    // Pre-inserting a row with that event_id makes the event append fail
    // AFTER the object CAS and budget debit already executed in the same
    // transaction — a genuine mid-commit fault.
    let colliding = SeqIds::uuid_for(1);
    raw_connection(&path)
        .execute(
            "INSERT INTO events (event_id, object_id, domain, object_version, event_type, canonical_json)
             VALUES (?1, ?2, 'task', 999, 'cognitiveos.object.admitted', '{}')",
            (colliding.as_str(), oid(999).as_str()),
        )
        .unwrap();
    let events_before = count(&raw_connection(&path), "SELECT COUNT(*) FROM events");

    let mut cmd = edge_command(
        LifecycleDomain::Task,
        &object_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
    );
    cmd.budget = Some(BudgetChargeCommand {
        budget_id: budget_id.clone(),
        charge: BudgetCharge::new([("tool_calls".to_owned(), 1)].into()).unwrap(),
    });
    let rejection = engine
        .commit_transition(&cmd)
        .expect_err("mid-commit fault");
    assert_eq!(
        rejection.registered().code,
        "STATE_STORE_UNAVAILABLE",
        "failed commit path surfaces the fail-closed code: {rejection:?}"
    );

    // Nothing of the atomic unit persisted — including the budget debit
    // that had already executed inside the transaction.
    let stored = store
        .load_object(LifecycleDomain::Task, &object_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        (stored.state.as_str(), stored.version),
        ("DRAFT", Version::INITIAL)
    );
    let budget = store.load_budget(&budget_id).unwrap().unwrap();
    assert_eq!(budget.version, Version::INITIAL);
    assert_eq!(budget.state.remaining()["tool_calls"], 5);
    let conn = raw_connection(&path);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM events"), events_before);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM transition_records"), 0);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM outbox"), 0);

    // The store is still healthy: the same transition commits once the
    // fault is gone (fresh ids avoid the injected collision).
    let healthy_ids = SeqIds(AtomicU64::new(100));
    let engine = TransitionEngine::new(&store, &clock, &healthy_ids);
    engine.commit_transition(&cmd).unwrap();
    let stored = store
        .load_object(LifecycleDomain::Task, &object_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "READY");
}

/// REQ-REC-003 / vector `state-store-degradation.json` behavioral side:
/// a read-only (degraded) authority volume rejects every governed write
/// with STATE_STORE_UNAVAILABLE, keeps read paths alive, buffers nothing,
/// and loses no committed history.
#[test]
fn criterion_6_read_only_store_fails_closed_and_keeps_reads_available() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("authority.db");
    let store = drive_reference_history(&path);
    let before = replay_projection(&store).unwrap();
    drop(store);

    let degraded = SqliteAuthorityStore::open_read_only(&path).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds(AtomicU64::new(500));
    let engine = TransitionEngine::new(&degraded, &clock, &ids);

    // Governed write: rejected fail-closed with the registered code.
    let mut cmd = edge_command(
        LifecycleDomain::Task,
        &oid(1),
        "ACTIVE",
        "CANDIDATE_COMPLETE",
        "COMPLETION_CLAIMED",
    );
    cmd.expected_version = Version::new(3).unwrap();
    let rejection = engine.commit_transition(&cmd).expect_err("degraded write");
    let registered = rejection.registered();
    assert_eq!(registered.code, "STATE_STORE_UNAVAILABLE");
    assert_eq!(registered.category, "state");
    assert!(registered.retryable);

    // New admissions are refused as well (no Intent-less dispatch paths).
    let admission = AdmitCommand {
        object_id: oid(50),
        domain: LifecycleDomain::Task,
        subject_ref: uri("task://tenant-test/50"),
        body: json!({}),
        actor_ref: uri("actor://tenant-test/agent-1"),
        authority_ref: uri("authority://tenant-test/state-authority"),
        correlation_id: uri("corr://m2/degraded"),
        outbox_destinations: vec![],
    };
    let rejection = engine.admit_object(&admission).expect_err("degraded admit");
    assert_eq!(rejection.registered().code, "STATE_STORE_UNAVAILABLE");

    // Read-only inspection stays available; nothing was buffered as
    // committed; committed history is not lost (same digest).
    let stored = degraded
        .load_object(LifecycleDomain::Task, &oid(1))
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "ACTIVE");
    assert_eq!(stored.version, Version::new(3).unwrap());
    let replayed = replay_projection(&degraded).unwrap();
    assert_eq!(replayed.digest, before.digest);

    // After the volume recovers (reopen read-write), the same write
    // commits — proving the earlier rejection left no hidden state.
    drop(degraded);
    let recovered = SqliteAuthorityStore::open(&path).unwrap();
    let engine = TransitionEngine::new(&recovered, &clock, &ids);
    engine.commit_transition(&cmd).unwrap();
    let stored = recovered
        .load_object(LifecycleDomain::Task, &oid(1))
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "CANDIDATE_COMPLETE");
}

// ---------------------------------------------------------------------
// Outbox bookkeeping (same-transaction enqueue, delivery marking)
// ---------------------------------------------------------------------

#[test]
fn outbox_rows_enqueue_with_commits_and_marking_never_touches_events() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("authority.db");
    let store = SqliteAuthorityStore::open(&path).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);

    let object_id = oid(1);
    seed(&store, &object_id, LifecycleDomain::Task, "DRAFT");
    let mut cmd = edge_command(
        LifecycleDomain::Task,
        &object_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
    );
    cmd.outbox_destinations = vec!["watch://status".to_owned(), "audit://trail".to_owned()];
    let committed = engine.commit_transition(&cmd).unwrap();

    let pending = store.pending_outbox(10).unwrap();
    assert_eq!(pending.len(), 2);
    assert!(
        pending
            .iter()
            .all(|entry| entry.event_id == committed.event_id)
    );

    let dispatched_at = WallTimestamp::parse("2026-07-20T06:05:00Z").unwrap();
    store
        .mark_outbox_dispatched(pending[0].outbox_sequence, &dispatched_at)
        .unwrap();
    assert_eq!(store.pending_outbox(10).unwrap().len(), 1);
    // Double-marking the same row is a conflict, not a silent no-op.
    assert!(
        store
            .mark_outbox_dispatched(pending[0].outbox_sequence, &dispatched_at)
            .is_err()
    );
    // Delivery bookkeeping never rewrites the event log.
    let conn = raw_connection(&path);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM events"), 2);
}
