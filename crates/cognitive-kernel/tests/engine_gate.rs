//! Behavioral tests of the deterministic transition gate against an
//! in-memory port fake (M2 acceptance criteria 1/2/5 at the kernel layer;
//! the SQLite-backed acceptance suite lives in `cognitive-store`).
//!
//! Every rejection asserts BOTH the registered error code and that the
//! authoritative state is unchanged — schema-valid shapes alone never pass
//! (`docs/standards/conformance-evidence.md`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_contracts::generated::common_defs::Digest;
use cognitive_contracts::generated::object_reference::{
    StrongReference, StrongReferenceKind, UuidV7,
};
use cognitive_domain::{
    BudgetId, LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef, Version, WallTimestamp,
    table,
};
use cognitive_kernel::ports::{
    AuthorityStore, Clock, CommitReceipt, CommittedEvent, IdGenerator, ObjectAdmission,
    OutboxEntry, PortFailure, StorePortError, StoredBudget, StoredObject, TransitionCommit,
};
use cognitive_kernel::{
    AdmitCommand, BudgetCharge, BudgetChargeCommand, BudgetState, Causation, Reason, RejectionKind,
    TablePin, TransitionCommand, TransitionEngine, replay_projection,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------
// Port fakes
// ---------------------------------------------------------------------

#[derive(Default)]
struct FakeInner {
    objects: BTreeMap<String, StoredObject>,
    budgets: BTreeMap<String, (String, Version)>, // canonical json, version
    events: Vec<CommittedEvent>,
    records: Vec<String>,
    outbox: Vec<OutboxEntry>,
    fail_commits: Option<StorePortError>,
}

#[derive(Default)]
struct FakeStore {
    inner: Mutex<FakeInner>,
}

impl FakeStore {
    fn seed_object(&self, object: StoredObject) {
        let mut inner = self.inner.lock().unwrap();
        inner.objects.insert(object.object_id.to_string(), object);
    }

    fn object(&self, id: &ObjectId) -> Option<StoredObject> {
        self.inner.lock().unwrap().objects.get(id.as_str()).cloned()
    }

    fn budget_version(&self, id: &BudgetId) -> Option<Version> {
        self.inner
            .lock()
            .unwrap()
            .budgets
            .get(id.as_str())
            .map(|(_, version)| *version)
    }

    fn event_count(&self) -> usize {
        self.inner.lock().unwrap().events.len()
    }

    fn record_count(&self) -> usize {
        self.inner.lock().unwrap().records.len()
    }

    fn outbox_count(&self) -> usize {
        self.inner.lock().unwrap().outbox.len()
    }

    fn fail_next_commits(&self, error: StorePortError) {
        self.inner.lock().unwrap().fail_commits = Some(error);
    }

    fn corrupt_last_event_json(&self, replacement: &str) {
        let mut inner = self.inner.lock().unwrap();
        let last = inner.events.last_mut().unwrap();
        last.canonical_json = replacement.to_owned();
    }
}

fn locked<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, StorePortError> {
    mutex.lock().map_err(|_| StorePortError::Unavailable {
        detail: "fake store poisoned".to_owned(),
    })
}

impl AuthorityStore for FakeStore {
    fn load_object(
        &self,
        domain: LifecycleDomain,
        object_id: &ObjectId,
    ) -> Result<Option<StoredObject>, StorePortError> {
        let inner = locked(&self.inner)?;
        Ok(inner
            .objects
            .get(object_id.as_str())
            .filter(|object| object.domain == domain)
            .cloned())
    }

    fn admit_object(&self, admission: &ObjectAdmission) -> Result<CommitReceipt, StorePortError> {
        let mut inner = locked(&self.inner)?;
        if let Some(error) = &inner.fail_commits {
            return Err(error.clone());
        }
        let key = admission.object.object_id.to_string();
        if inner.objects.contains_key(&key) {
            return Err(StorePortError::Conflict {
                detail: format!("object {key} already exists"),
            });
        }
        inner.objects.insert(key, admission.object.clone());
        let sequence = inner.events.len() as i64 + 1;
        inner.events.push(CommittedEvent {
            sequence,
            event_id: admission.event.event_id.clone(),
            object_id: admission.event.object_id.clone(),
            domain: admission.event.domain,
            object_version: admission.event.object_version,
            event_type: admission.event.event_type.clone(),
            canonical_json: admission.event.canonical_json.clone(),
        });
        for draft in &admission.outbox {
            let outbox_sequence = inner.outbox.len() as i64 + 1;
            inner.outbox.push(OutboxEntry {
                outbox_sequence,
                event_id: draft.event_id.clone(),
                destination: draft.destination.clone(),
                dispatched: false,
            });
        }
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn commit_transition(
        &self,
        commit: &TransitionCommit,
    ) -> Result<CommitReceipt, StorePortError> {
        let mut inner = locked(&self.inner)?;
        if let Some(error) = &inner.fail_commits {
            return Err(error.clone());
        }
        // Object CAS.
        let key = commit.cas.object_id.to_string();
        let matches = inner.objects.get(&key).is_some_and(|object| {
            object.domain == commit.cas.domain
                && object.state == commit.cas.from_state
                && object.version == commit.cas.expected_version
        });
        if !matches {
            return Err(StorePortError::Conflict {
                detail: format!("object cas raced for {key}"),
            });
        }
        // Budget CAS.
        if let Some(budget) = &commit.budget {
            let budget_key = budget.budget_id.to_string();
            let budget_matches = inner
                .budgets
                .get(&budget_key)
                .is_some_and(|(_, version)| *version == budget.expected_version);
            if !budget_matches {
                return Err(StorePortError::Conflict {
                    detail: format!("budget cas raced for {budget_key}"),
                });
            }
            inner.budgets.insert(
                budget_key,
                (
                    budget.next_state_canonical_json.clone(),
                    budget.next_version,
                ),
            );
        }
        // Apply all writes of the atomic unit.
        if let Some(object) = inner.objects.get_mut(&key) {
            object.state = commit.cas.to_state.clone();
            object.version = commit.cas.next_version;
        }
        let sequence = inner.events.len() as i64 + 1;
        inner.events.push(CommittedEvent {
            sequence,
            event_id: commit.event.event_id.clone(),
            object_id: commit.event.object_id.clone(),
            domain: commit.event.domain,
            object_version: commit.event.object_version,
            event_type: commit.event.event_type.clone(),
            canonical_json: commit.event.canonical_json.clone(),
        });
        inner.records.push(commit.record.canonical_json.clone());
        for draft in &commit.outbox {
            let outbox_sequence = inner.outbox.len() as i64 + 1;
            inner.outbox.push(OutboxEntry {
                outbox_sequence,
                event_id: draft.event_id.clone(),
                destination: draft.destination.clone(),
                dispatched: false,
            });
        }
        Ok(CommitReceipt {
            event_sequence: sequence,
        })
    }

    fn load_budget(&self, budget_id: &BudgetId) -> Result<Option<StoredBudget>, StorePortError> {
        let inner = locked(&self.inner)?;
        inner
            .budgets
            .get(budget_id.as_str())
            .map(|(canonical_json, version)| {
                let state: BudgetState = serde_json::from_str(canonical_json).map_err(|err| {
                    StorePortError::Unavailable {
                        detail: format!("stored budget unparseable: {err}"),
                    }
                })?;
                Ok(StoredBudget {
                    budget_id: budget_id.clone(),
                    state,
                    version: *version,
                })
            })
            .transpose()
    }

    fn create_budget(
        &self,
        budget_id: &BudgetId,
        state_canonical_json: &str,
        _created_at: &WallTimestamp,
    ) -> Result<(), StorePortError> {
        let mut inner = locked(&self.inner)?;
        let key = budget_id.to_string();
        if inner.budgets.contains_key(&key) {
            return Err(StorePortError::Conflict {
                detail: format!("budget {key} already exists"),
            });
        }
        inner
            .budgets
            .insert(key, (state_canonical_json.to_owned(), Version::INITIAL));
        Ok(())
    }

    fn read_events(
        &self,
        after_sequence: i64,
        limit: usize,
    ) -> Result<Vec<CommittedEvent>, StorePortError> {
        let inner = locked(&self.inner)?;
        Ok(inner
            .events
            .iter()
            .filter(|event| event.sequence > after_sequence)
            .take(limit)
            .cloned()
            .collect())
    }

    fn pending_outbox(&self, limit: usize) -> Result<Vec<OutboxEntry>, StorePortError> {
        let inner = locked(&self.inner)?;
        Ok(inner
            .outbox
            .iter()
            .filter(|entry| !entry.dispatched)
            .take(limit)
            .cloned()
            .collect())
    }

    fn mark_outbox_dispatched(
        &self,
        outbox_sequence: i64,
        _dispatched_at: &WallTimestamp,
    ) -> Result<(), StorePortError> {
        let mut inner = locked(&self.inner)?;
        for entry in &mut inner.outbox {
            if entry.outbox_sequence == outbox_sequence {
                entry.dispatched = true;
                return Ok(());
            }
        }
        Err(StorePortError::Conflict {
            detail: format!("no outbox row {outbox_sequence}"),
        })
    }
}

struct FixedClock(WallTimestamp);

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        Ok(self.0.clone())
    }
}

/// Deterministic sequential UUIDv7-shaped IDs for reproducible tests.
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

// ---------------------------------------------------------------------
// Command helpers
// ---------------------------------------------------------------------

fn fixed_clock() -> FixedClock {
    FixedClock(WallTimestamp::parse("2026-07-20T06:00:00Z").unwrap())
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

fn seeded(store: &FakeStore, n: u64, domain: LifecycleDomain, at: &str) -> ObjectId {
    let id = oid(n);
    store.seed_object(StoredObject {
        object_id: id.clone(),
        domain,
        state: state(at),
        version: Version::INITIAL,
        body: json!({"seeded": true}),
    });
    id
}

fn command(
    domain: LifecycleDomain,
    object_id: &ObjectId,
    from: &str,
    to: &str,
    reason: &str,
    guards: &[String],
    evidence_items: &[String],
) -> TransitionCommand {
    let evidence: BTreeMap<String, StrongReference> = evidence_items
        .iter()
        .enumerate()
        .map(|(index, item)| (item.clone(), strong_ref(index as u64 + 1)))
        .collect();
    TransitionCommand {
        request_id: uri(&format!("request://test/{}", object_id.as_str())),
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
            causation_id: uri("cause://test/origin"),
            correlation_id: uri("corr://test/chain-1"),
        },
        actor_ref: uri("actor://tenant-test/agent-1"),
        authority_ref: uri("authority://tenant-test/state-authority"),
        requested_at: WallTimestamp::parse("2026-07-20T05:59:00Z").unwrap(),
        table_pin: TablePin::current(domain).unwrap(),
        established_guards: guards.iter().cloned().collect::<BTreeSet<_>>(),
        evidence,
        budget: None,
        outbox_destinations: vec!["watch://status".to_owned()],
        fencing_epoch: None,
    }
}

// ---------------------------------------------------------------------
// Acceptance-facing behavior
// ---------------------------------------------------------------------

/// Every legal row of every registered table commits when its guards are
/// established and its evidence present; the version advances by exactly
/// one and the event/record/outbox rows join the same commit.
#[test]
fn every_registered_edge_commits_through_the_gate() {
    let store = FakeStore::default();
    let clock = fixed_clock();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);
    let mut counter = 1u64;
    let mut committed_edges = 0usize;
    for domain in LifecycleDomain::ALL {
        let loaded = table(domain).unwrap();
        for edge in &loaded.table.transitions {
            let object_id = seeded(&store, counter, domain, &edge.from);
            counter += 1;
            let cmd = command(
                domain,
                &object_id,
                &edge.from,
                &edge.to,
                &edge.reason_codes[0],
                &edge.guards,
                &edge.required_evidence,
            );
            let events_before = store.event_count();
            let committed = engine.commit_transition(&cmd).unwrap_or_else(|rejection| {
                panic!(
                    "{domain} {} -> {} rejected: {rejection:?}",
                    edge.from, edge.to
                )
            });
            assert_eq!(committed.after_version, Version::INITIAL.next().unwrap());
            let stored = store.object(&object_id).unwrap();
            assert_eq!(stored.state.as_str(), edge.to);
            assert_eq!(stored.version, committed.after_version);
            assert_eq!(store.event_count(), events_before + 1);
            committed_edges += 1;
        }
    }
    assert!(committed_edges > 80, "all tables were exercised");
    assert_eq!(store.record_count(), committed_edges);
    assert_eq!(store.outbox_count(), committed_edges);
}

/// M2 acceptance 2 (kernel layer): for all five tables, EVERY ordered state
/// pair without a registered row is rejected with the registry-consistent
/// code and the authoritative state does not change. Terminal states have
/// no legal exits at all.
#[test]
fn every_unregistered_state_pair_is_rejected_and_state_is_unchanged() {
    let store = FakeStore::default();
    let clock = fixed_clock();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);
    let mut counter = 10_000u64;
    let mut rejected_pairs = 0usize;
    for domain in LifecycleDomain::ALL {
        let loaded = table(domain).unwrap();
        let legal: BTreeSet<(&str, &str)> = loaded
            .table
            .transitions
            .iter()
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect();
        for from in &loaded.table.states {
            let object_id = seeded(&store, counter, domain, from);
            counter += 1;
            for to in &loaded.table.states {
                if legal.contains(&(from.as_str(), to.as_str())) {
                    continue;
                }
                let cmd = command(
                    domain,
                    &object_id,
                    from,
                    to,
                    "FORCED_ILLEGAL_ATTEMPT",
                    &[],
                    &[],
                );
                let events_before = store.event_count();
                let rejection = engine.commit_transition(&cmd).expect_err("must reject");
                let registered = rejection.registered();
                if domain == LifecycleDomain::Effect && from == "OUTCOME_UNKNOWN" {
                    // Pinned by vector effect-state-closure-008.json.
                    assert_eq!(registered.code, "EFFECT_OUTCOME_UNKNOWN");
                    assert_eq!(registered.category, "effect");
                    assert_eq!(rejection.available_exits, vec!["RECONCILED"]);
                } else {
                    assert_eq!(registered.code, "STATE_CONFLICT", "{domain} {from} -> {to}");
                    assert_eq!(registered.category, "state");
                }
                // Authoritative state and version unchanged, nothing written.
                let stored = store.object(&object_id).unwrap();
                assert_eq!(stored.state.as_str(), from);
                assert_eq!(stored.version, Version::INITIAL);
                assert_eq!(store.event_count(), events_before);
                assert_eq!(
                    rejection.current_state.as_ref().unwrap().as_str(),
                    from,
                    "rejection reports the authoritative state"
                );
                rejected_pairs += 1;
            }
        }
    }
    assert!(
        rejected_pairs > 400,
        "exhaustive sweep covered {rejected_pairs} pairs"
    );
}

/// Vector STATE-CAS-002 semantics: a stale expected version is rejected as
/// STATE_CONFLICT with no write applied.
#[test]
fn stale_expected_version_is_state_conflict_without_side_effects() {
    let store = FakeStore::default();
    let clock = fixed_clock();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);
    let object_id = seeded(&store, 1, LifecycleDomain::Task, "DRAFT");

    let mut cmd = command(
        LifecycleDomain::Task,
        &object_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
        &[
            "task_contract_complete".to_owned(),
            "acceptance_criteria_fixed".to_owned(),
        ],
        &["task_contract".to_owned()],
    );
    cmd.expected_version = Version::new(12).unwrap(); // authoritative is 1
    let rejection = engine.commit_transition(&cmd).expect_err("stale CAS");
    assert_eq!(rejection.kind, RejectionKind::VersionMismatch);
    assert_eq!(rejection.registered().code, "STATE_CONFLICT");
    assert!(rejection.registered().retryable);
    let stored = store.object(&object_id).unwrap();
    assert_eq!(
        (stored.state.as_str(), stored.version),
        ("DRAFT", Version::INITIAL)
    );
    assert_eq!(store.event_count(), 0);
}

/// A row that matches the state pair but not the requested reason must not
/// fire, and a guard or evidence item that is not established fails closed.
#[test]
fn reason_guard_and_evidence_gates_fail_closed() {
    let store = FakeStore::default();
    let clock = fixed_clock();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);
    let guards = [
        "task_contract_complete".to_owned(),
        "acceptance_criteria_fixed".to_owned(),
    ];
    let evidence = ["task_contract".to_owned()];

    // Wrong reason.
    let object_id = seeded(&store, 1, LifecycleDomain::Task, "DRAFT");
    let mut cmd = command(
        LifecycleDomain::Task,
        &object_id,
        "DRAFT",
        "READY",
        "EXECUTION_STARTED",
        &guards,
        &evidence,
    );
    let rejection = engine.commit_transition(&cmd).expect_err("wrong reason");
    assert_eq!(rejection.kind, RejectionKind::ReasonNotAllowed);
    assert_eq!(rejection.registered().code, "STATE_CONFLICT");

    // Guard missing: one of the two required guards is not established.
    cmd = command(
        LifecycleDomain::Task,
        &object_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
        &guards[..1],
        &evidence,
    );
    let rejection = engine.commit_transition(&cmd).expect_err("guard missing");
    assert_eq!(rejection.kind, RejectionKind::GuardUnsatisfied);
    assert_eq!(rejection.registered().code, "STATE_CONFLICT");

    // Evidence missing.
    cmd = command(
        LifecycleDomain::Task,
        &object_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
        &guards,
        &[],
    );
    let rejection = engine
        .commit_transition(&cmd)
        .expect_err("evidence missing");
    assert_eq!(rejection.kind, RejectionKind::EvidenceMissing);
    assert_eq!(rejection.registered().code, "STATE_CONFLICT");

    // Nothing was written by any rejected attempt.
    let stored = store.object(&object_id).unwrap();
    assert_eq!(
        (stored.state.as_str(), stored.version),
        ("DRAFT", Version::INITIAL)
    );
    assert_eq!(store.event_count(), 0);
    assert_eq!(store.record_count(), 0);
}

/// A request that decided under a different table version/digest is
/// rejected before any store access (DIGEST_MISMATCH, fail closed).
#[test]
fn unpinned_or_mismatched_table_digest_is_rejected() {
    let store = FakeStore::default();
    let clock = fixed_clock();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);
    let object_id = seeded(&store, 1, LifecycleDomain::Task, "DRAFT");
    let mut cmd = command(
        LifecycleDomain::Task,
        &object_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
        &[],
        &[],
    );
    cmd.table_pin.digest = format!("sha256:{}", "0".repeat(64));
    let rejection = engine.commit_transition(&cmd).expect_err("wrong pin");
    assert_eq!(rejection.kind, RejectionKind::TablePinMismatch);
    assert_eq!(rejection.registered().code, "DIGEST_MISMATCH");
    let stored = store.object(&object_id).unwrap();
    assert_eq!(stored.state.as_str(), "DRAFT");
}

/// Effect OUTCOME_UNKNOWN never reaches COMMITTED or VERIFIED directly:
/// the deny surfaces EFFECT_OUTCOME_UNKNOWN and reconciliation is the only
/// exit (vector effect-state-closure-008.json, behavioral side).
#[test]
fn outcome_unknown_denies_direct_commit_with_effect_outcome_unknown() {
    let store = FakeStore::default();
    let clock = fixed_clock();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);
    let object_id = seeded(&store, 1, LifecycleDomain::Effect, "OUTCOME_UNKNOWN");
    for target in ["COMMITTED", "VERIFIED", "EXECUTED"] {
        let cmd = command(
            LifecycleDomain::Effect,
            &object_id,
            "OUTCOME_UNKNOWN",
            target,
            "COMMIT_AUTHORIZED",
            &[],
            &[],
        );
        let rejection = engine.commit_transition(&cmd).expect_err("must deny");
        let registered = rejection.registered();
        assert_eq!(registered.code, "EFFECT_OUTCOME_UNKNOWN");
        assert_eq!(registered.category, "effect");
        assert_eq!(rejection.available_exits, vec!["RECONCILED"]);
        let stored = store.object(&object_id).unwrap();
        assert_eq!(stored.state.as_str(), "OUTCOME_UNKNOWN");
    }
}

/// M2 acceptance 5 (kernel layer): a hard-budget charge that cannot be
/// covered is rejected deterministically (RESOURCE_BUDGET_EXHAUSTED,
/// fail closed, not retryable) and neither state nor budget changes. An
/// admissible charge debits in the same commit.
#[test]
fn hard_budget_admission_is_deterministic_and_rides_the_commit() {
    let store = FakeStore::default();
    let clock = fixed_clock();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);
    let budget_id = BudgetId::parse("00000000-0000-7000-b000-000000000001").unwrap();
    engine
        .create_budget(
            &budget_id,
            &BudgetState::new([("tool_calls".to_owned(), 1)].into()).unwrap(),
        )
        .unwrap();

    let charge = |amount: i64| BudgetChargeCommand {
        budget_id: budget_id.clone(),
        charge: BudgetCharge::new([("tool_calls".to_owned(), amount)].into()).unwrap(),
    };
    let guards = [
        "task_contract_complete".to_owned(),
        "acceptance_criteria_fixed".to_owned(),
    ];
    let evidence = ["task_contract".to_owned()];

    // Over budget: rejected before any write.
    let object_id = seeded(&store, 1, LifecycleDomain::Task, "DRAFT");
    let mut cmd = command(
        LifecycleDomain::Task,
        &object_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
        &guards,
        &evidence,
    );
    cmd.budget = Some(charge(2));
    let rejection = engine.commit_transition(&cmd).expect_err("over budget");
    assert_eq!(rejection.kind, RejectionKind::BudgetExhausted);
    let registered = rejection.registered();
    assert_eq!(registered.code, "RESOURCE_BUDGET_EXHAUSTED");
    assert_eq!(registered.category, "resource");
    assert!(!registered.retryable);
    let stored = store.object(&object_id).unwrap();
    assert_eq!(
        (stored.state.as_str(), stored.version),
        ("DRAFT", Version::INITIAL)
    );
    assert_eq!(store.event_count(), 0);
    assert_eq!(store.budget_version(&budget_id).unwrap(), Version::INITIAL);

    // Within budget: the debit and the transition commit together.
    cmd.budget = Some(charge(1));
    engine.commit_transition(&cmd).unwrap();
    let stored = store.object(&object_id).unwrap();
    assert_eq!(stored.state.as_str(), "READY");
    assert_eq!(
        store.budget_version(&budget_id).unwrap(),
        Version::INITIAL.next().unwrap()
    );

    // The budget is now drained; the next charged transition is denied.
    let mut next = command(
        LifecycleDomain::Task,
        &object_id,
        "READY",
        "ACTIVE",
        "EXECUTION_STARTED",
        &[
            "execution_admitted".to_owned(),
            "dependencies_satisfied".to_owned(),
        ],
        &["execution_binding".to_owned()],
    );
    next.expected_version = Version::INITIAL.next().unwrap();
    next.budget = Some(charge(1));
    let rejection = engine.commit_transition(&next).expect_err("drained");
    assert_eq!(rejection.registered().code, "RESOURCE_BUDGET_EXHAUSTED");
    // A budget row that does not exist fails closed as well.
    next.budget = Some(BudgetChargeCommand {
        budget_id: BudgetId::parse("00000000-0000-7000-b000-00000000dead").unwrap(),
        charge: BudgetCharge::new(BTreeMap::new()).unwrap(),
    });
    let rejection = engine.commit_transition(&next).expect_err("missing budget");
    assert_eq!(rejection.kind, RejectionKind::BudgetNotFound);
    assert_eq!(rejection.registered().code, "STATE_CONFLICT");
}

/// REQ-REC-003 (kernel layer): when the store cannot persist, the gate
/// surfaces STATE_STORE_UNAVAILABLE and does not pretend the write
/// happened (no receipt, no state change on retry path).
#[test]
fn store_unavailability_fails_closed_as_state_store_unavailable() {
    let store = FakeStore::default();
    let clock = fixed_clock();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);
    let object_id = seeded(&store, 1, LifecycleDomain::Task, "DRAFT");
    store.fail_next_commits(StorePortError::Unavailable {
        detail: "disk full".to_owned(),
    });
    let cmd = command(
        LifecycleDomain::Task,
        &object_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
        &[
            "task_contract_complete".to_owned(),
            "acceptance_criteria_fixed".to_owned(),
        ],
        &["task_contract".to_owned()],
    );
    let rejection = engine.commit_transition(&cmd).expect_err("unavailable");
    assert_eq!(rejection.kind, RejectionKind::StoreUnavailable);
    let registered = rejection.registered();
    assert_eq!(registered.code, "STATE_STORE_UNAVAILABLE");
    assert_eq!(registered.category, "state");
    // The authoritative row still shows the old state (nothing buffered).
    let stored = store.object(&object_id).unwrap();
    assert_eq!(
        (stored.state.as_str(), stored.version),
        ("DRAFT", Version::INITIAL)
    );
    assert_eq!(store.event_count(), 0);
}

/// Admission: a new governed object enters at the table's initial state,
/// version 1; a duplicate identity is a conflict.
#[test]
fn admission_uses_initial_state_and_rejects_duplicates() {
    let store = FakeStore::default();
    let clock = fixed_clock();
    let ids = SeqIds::new();
    let engine = TransitionEngine::new(&store, &clock, &ids);
    let cmd = AdmitCommand {
        object_id: oid(77),
        domain: LifecycleDomain::Verification,
        subject_ref: uri("verification://tenant-test/v-77"),
        body: json!({"criteria": "fixed"}),
        actor_ref: uri("actor://tenant-test/agent-1"),
        authority_ref: uri("authority://tenant-test/verification-authority"),
        correlation_id: uri("corr://test/chain-2"),
        outbox_destinations: vec![],
        fencing_epoch: None,
    };
    let admitted = engine.admit_object(&cmd).unwrap();
    assert_eq!(admitted.initial_state.as_str(), "NOT_REQUESTED");
    assert_eq!(admitted.version, Version::INITIAL);
    let rejection = engine.admit_object(&cmd).expect_err("duplicate");
    assert_eq!(rejection.kind, RejectionKind::StoreConflict);
    assert_eq!(rejection.registered().code, "STATE_CONFLICT");
}

/// M2 acceptance 3 (kernel layer): replaying the same committed history
/// twice yields byte-identical canonical projections, and two independent
/// stores fed the same commands converge on the same digest. A corrupted
/// history is a replay barrier, never a guess.
#[test]
fn replay_projection_is_deterministic_and_barriers_on_corruption() {
    let build = || {
        let store = FakeStore::default();
        let clock = fixed_clock();
        let ids = SeqIds::new();
        {
            let engine = TransitionEngine::new(&store, &clock, &ids);
            for n in 1..=3u64 {
                let admit = AdmitCommand {
                    object_id: oid(n),
                    domain: LifecycleDomain::Task,
                    subject_ref: uri(&format!("task://tenant-test/{n}")),
                    body: json!({"n": n}),
                    actor_ref: uri("actor://tenant-test/agent-1"),
                    authority_ref: uri("authority://tenant-test/task-acceptance"),
                    correlation_id: uri("corr://test/replay"),
                    outbox_destinations: vec![],
                    fencing_epoch: None,
                };
                engine.admit_object(&admit).unwrap();
            }
            let guards = [
                "task_contract_complete".to_owned(),
                "acceptance_criteria_fixed".to_owned(),
            ];
            let evidence = ["task_contract".to_owned()];
            for n in 1..=2u64 {
                let cmd = command(
                    LifecycleDomain::Task,
                    &oid(n),
                    "DRAFT",
                    "READY",
                    "CONTRACT_ACCEPTED",
                    &guards,
                    &evidence,
                );
                engine.commit_transition(&cmd).unwrap();
            }
        }
        store
    };

    let store_a = build();
    let first = replay_projection(&store_a).unwrap();
    let second = replay_projection(&store_a).unwrap();
    assert_eq!(
        first.canonical_bytes, second.canonical_bytes,
        "byte-identical replay"
    );
    assert_eq!(first.digest, second.digest);
    assert_eq!(first.event_count, 5);

    // Independent store, same deterministic inputs -> same digest.
    let store_b = build();
    let other = replay_projection(&store_b).unwrap();
    assert_eq!(first.digest, other.digest);
    assert_eq!(first.canonical_bytes, other.canonical_bytes);

    // Corrupted committed history is a barrier.
    store_b.corrupt_last_event_json(
        r#"{"event_type":"cognitiveos.state.transition.committed","object_id":"mismatch"}"#,
    );
    assert!(matches!(
        replay_projection(&store_b),
        Err(cognitive_kernel::ReplayError::Barrier(_))
    ));
}
