//! M2 behavioral vector execution against the real kernel authority path.
//!
//! The implementation under test is `cognitive_kernel::TransitionEngine`
//! over `cognitive_store::SqliteAuthorityStore` (WAL) — the same
//! centralized deterministic gate and adapter the M2 acceptance suite
//! exercises. The runner drives each vector's `input` scenario through
//! real committed histories in a throwaway SQLite database and compares
//! the observable outcome (registered error codes, authoritative state
//! and version invariance, event-log growth) with `expected`.
//!
//! Discipline: no gate here re-derives expectations from the vector
//! document; every observation is read back from the engine rejection or
//! the store. The deliberately wrong implementation for the behavioral
//! self-check is a gate-bypassing direct store writer — schema-shaped
//! events and records, but no table lookup, no CAS respect and no guard
//! or evidence checks (exactly the "write authority state outside the
//! centralized transition entry" anti-pattern the architecture bans).
//!
//! Clock and ID adapters are deterministic runner harness (fixed wall
//! time, sequential UUIDv7-shaped ids), so committed histories and replay
//! digests are reproducible run to run.

use super::{AssetContext, ExecError, GateOutput, ImplementationKind};
use crate::LoadedVector;
use cognitive_domain::{
    EventId, LifecycleDomain, ObjectId, ReasonCode, RecordId, StateName, UriRef, Version,
    WallTimestamp, table,
};
use cognitive_kernel::ports::{
    AuthorityStore, Clock, EventDraft, IdGenerator, ObjectAdmission, ObjectCas, PortFailure,
    RecordDraft, StoredObject, TransitionCommit,
};
use cognitive_kernel::{
    Causation, Reason, TablePin, TransitionCommand, TransitionEngine, TransitionRejection,
    replay_projection,
};
use cognitive_store::SqliteAuthorityStore;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const REFERENCE_IMPLEMENTATION: &str = "cognitive-kernel TransitionEngine + cognitive-store SqliteAuthorityStore (real authority path)";
const WRONG_IMPLEMENTATION: &str =
    "gate-bypassing direct store writer (deliberately wrong: no table lookup, no CAS, no guards)";

fn env_err(what: impl Into<String>) -> ExecError {
    ExecError::Environment(what.into())
}

// ---------------------------------------------------------------------------
// Deterministic harness adapters (not the implementation under test)
// ---------------------------------------------------------------------------

struct FixedClock(WallTimestamp);

impl FixedClock {
    fn new() -> Result<Self, ExecError> {
        Ok(Self(
            WallTimestamp::parse("2026-07-20T06:00:00Z")
                .map_err(|err| env_err(format!("harness clock: {err}")))?,
        ))
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        Ok(self.0.clone())
    }
}

/// Sequential UUIDv7-shaped identifiers for reproducible histories.
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

/// One throwaway SQLite (WAL) authority database plus the deterministic
/// harness adapters.
struct KernelHarness {
    // Keeps the temp directory (and thus the db file) alive.
    _dir: tempfile::TempDir,
    db_path: PathBuf,
    store: SqliteAuthorityStore,
    clock: FixedClock,
    ids: SeqIds,
}

impl KernelHarness {
    fn new() -> Result<Self, ExecError> {
        let dir = tempfile::tempdir()
            .map_err(|err| env_err(format!("temp dir for behavioral store: {err}")))?;
        let db_path = dir.path().join("authority.db");
        let store = SqliteAuthorityStore::open(&db_path)
            .map_err(|err| env_err(format!("open behavioral store: {err}")))?;
        Ok(Self {
            _dir: dir,
            db_path,
            store,
            clock: FixedClock::new()?,
            ids: SeqIds::new(),
        })
    }

    fn engine(&self) -> TransitionEngine<'_, SqliteAuthorityStore, FixedClock, SeqIds> {
        TransitionEngine::new(&self.store, &self.clock, &self.ids)
    }

    /// Seed one governed object at an arbitrary state through the store
    /// port (models an object whose committed history reached that state),
    /// with a well-formed admission event.
    fn seed(
        &self,
        object_id: &ObjectId,
        domain: LifecycleDomain,
        at: &str,
        subject_ref: &str,
    ) -> Result<(), ExecError> {
        let admitted_at = WallTimestamp::parse("2026-07-20T05:00:00Z")
            .map_err(|err| env_err(format!("seed timestamp: {err}")))?;
        let raw_event_id = self
            .ids
            .next_uuid_v7()
            .map_err(|err| env_err(format!("seed id: {}", err.detail)))?;
        let event_id = EventId::parse(&raw_event_id)
            .map_err(|err| env_err(format!("seed event id: {err}")))?;
        let state = StateName::parse(at).map_err(|err| env_err(format!("seed state: {err}")))?;
        let event_value = json!({
            "event_id": event_id.as_str(),
            "event_type": "cognitiveos.object.admitted",
            "domain": domain.as_str(),
            "object_id": object_id.as_str(),
            "subject_ref": subject_ref,
            "after_state": at,
            "after_version": 1,
            "event_time": admitted_at.as_str(),
        });
        let canonical_json = canonical_text(&event_value)?;
        self.store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: object_id.clone(),
                    domain,
                    state,
                    version: Version::INITIAL,
                    body: json!({ "seeded_by": "conformance-behavioral-harness" }),
                },
                admitted_at,
                event: EventDraft {
                    event_id,
                    object_id: object_id.clone(),
                    domain,
                    object_version: Version::INITIAL,
                    event_type: "cognitiveos.object.admitted".to_owned(),
                    canonical_json,
                },
                outbox: vec![],
            })
            .map_err(|err| env_err(format!("seed admission: {err}")))?;
        Ok(())
    }

    fn load(
        &self,
        domain: LifecycleDomain,
        object_id: &ObjectId,
    ) -> Result<StoredObject, ExecError> {
        self.store
            .load_object(domain, object_id)
            .map_err(|err| env_err(format!("load object: {err}")))?
            .ok_or_else(|| env_err(format!("object {object_id} missing after seed")))
    }

    fn event_count(&self) -> Result<usize, ExecError> {
        let mut total = 0usize;
        let mut after = 0i64;
        loop {
            let page = self
                .store
                .read_events(after, 256)
                .map_err(|err| env_err(format!("read events: {err}")))?;
            if page.is_empty() {
                return Ok(total);
            }
            total += page.len();
            after = page.last().map(|e| e.sequence).unwrap_or(after);
        }
    }
}

fn canonical_text(value: &Value) -> Result<String, ExecError> {
    let bytes = cognitive_contracts::canonical::canonical_bytes_of_value(value)
        .map_err(|err| env_err(format!("canonical encoding: {err}")))?;
    String::from_utf8(bytes).map_err(|err| env_err(format!("canonical bytes not utf-8: {err}")))
}

fn oid(tag: u64) -> Result<ObjectId, ExecError> {
    ObjectId::parse(&format!("00000000-0000-7000-b000-{tag:012x}"))
        .map_err(|err| env_err(format!("harness object id: {err}")))
}

fn uri(text: &str) -> Result<UriRef, ExecError> {
    UriRef::parse(text).map_err(|err| env_err(format!("harness uri `{text}`: {err}")))
}

fn state_name(text: &str) -> Result<StateName, ExecError> {
    StateName::parse(text).map_err(|err| env_err(format!("state `{text}`: {err}")))
}

/// Build a transition command for `(from, to, reason)`. When the pinned
/// registered table carries a matching row, that row's own guards are
/// attested and its required evidence supplied (deterministic upstream
/// establishment); otherwise guards and evidence stay empty — the gate is
/// expected to reject before they matter.
fn command(
    domain: LifecycleDomain,
    object_id: &ObjectId,
    subject_ref: &str,
    from: &str,
    to: &str,
    reason: &str,
    expected_version: Version,
) -> Result<TransitionCommand, ExecError> {
    let loaded = table(domain).map_err(|err| env_err(format!("registered table: {err}")))?;
    let from_state = state_name(from)?;
    let to_state = state_name(to)?;
    let (guards, evidence_items) = match loaded.find_edge(&from_state, &to_state, reason) {
        Ok(edge) => (edge.guards.clone(), edge.required_evidence.clone()),
        Err(_) => (Vec::new(), Vec::new()),
    };
    let evidence: BTreeMap<
        String,
        cognitive_contracts::generated::object_reference::StrongReference,
    > = evidence_items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let tag = index as u64 + 1;
            let reference = cognitive_contracts::generated::object_reference::StrongReference {
                content_digest: cognitive_contracts::generated::common_defs::Digest(format!(
                    "sha256:{}",
                    format!("{tag:x}").repeat(64)[..64].to_owned()
                )),
                id: cognitive_contracts::generated::object_reference::UuidV7(format!(
                    "00000000-0000-7000-a000-{tag:012x}"
                )),
                kind: cognitive_contracts::generated::object_reference::StrongReferenceKind::Strong,
                object_version: 1,
            };
            (item.clone(), reference)
        })
        .collect();
    Ok(TransitionCommand {
        request_id: uri(&format!(
            "request://conformance/{}/{from}-{to}",
            object_id.as_str()
        ))?,
        domain,
        object_id: object_id.clone(),
        subject_ref: uri(subject_ref)?,
        from: from_state,
        to: to_state,
        expected_version,
        reason: Reason {
            code: ReasonCode::parse(reason).map_err(|err| env_err(format!("reason: {err}")))?,
            detail: None,
        },
        causation: Causation {
            causation_id: uri("cause://conformance/behavioral-batch")?,
            correlation_id: uri("corr://conformance/behavioral-batch")?,
        },
        actor_ref: uri("actor://conformance/runner")?,
        authority_ref: uri("authority://conformance/state-authority")?,
        requested_at: WallTimestamp::parse("2026-07-20T05:59:00Z")
            .map_err(|err| env_err(format!("requested_at: {err}")))?,
        table_pin: TablePin::current(domain).map_err(|err| env_err(format!("table pin: {err}")))?,
        established_guards: guards.into_iter().collect::<BTreeSet<_>>(),
        evidence,
        budget: None,
        outbox_destinations: vec![],
    })
}

/// Registered `{code, category}` object for the code a rejection surfaced,
/// resolved through the runner's own registry truth (never hardcoded).
fn rejection_error(
    ctx: &AssetContext,
    rejection: &TransitionRejection,
) -> Result<Value, ExecError> {
    let code = rejection.registered().code;
    ctx.registered_error(code)
        .ok_or_else(|| env_err(format!("rejection surfaced unregistered code {code}")))
}

/// The deliberately wrong implementation: commit a transition directly
/// through the store port, skipping the centralized gate entirely. Events
/// and records are well-formed (schema-shaped canonical JSON) — only the
/// behavior is wrong.
fn bypass_gate_commit(
    harness: &KernelHarness,
    current: &StoredObject,
    to: &str,
    subject_ref: &str,
) -> Result<(), ExecError> {
    let to_state = state_name(to)?;
    let next_version = current
        .version
        .next()
        .map_err(|err| env_err(format!("version overflow: {err}")))?;
    let committed_at = WallTimestamp::parse("2026-07-20T06:01:00Z")
        .map_err(|err| env_err(format!("bypass timestamp: {err}")))?;
    let raw_event = harness
        .ids
        .next_uuid_v7()
        .map_err(|err| env_err(format!("bypass id: {}", err.detail)))?;
    let event_id =
        EventId::parse(&raw_event).map_err(|err| env_err(format!("bypass event id: {err}")))?;
    let raw_record = harness
        .ids
        .next_uuid_v7()
        .map_err(|err| env_err(format!("bypass id: {}", err.detail)))?;
    let record_id =
        RecordId::parse(&raw_record).map_err(|err| env_err(format!("bypass record id: {err}")))?;

    let event_value = json!({
        "event_id": event_id.as_str(),
        "event_type": "cognitiveos.state.transition.committed",
        "domain": current.domain.as_str(),
        "object_id": current.object_id.as_str(),
        "subject_ref": subject_ref,
        "before_state": current.state.as_str(),
        "after_state": to,
        "before_version": current.version.get(),
        "after_version": next_version.get(),
        "event_time": committed_at.as_str(),
        "gate": "BYPASSED (deliberately wrong implementation)",
    });
    let record_value = json!({
        "record_id": record_id.as_str(),
        "domain": current.domain.as_str(),
        "subject_ref": subject_ref,
        "before_state": current.state.as_str(),
        "after_state": to,
        "after_version": next_version.get(),
        "committed_at": committed_at.as_str(),
        "gate": "BYPASSED (deliberately wrong implementation)",
    });
    harness
        .store
        .commit_transition(&TransitionCommit {
            cas: ObjectCas {
                object_id: current.object_id.clone(),
                domain: current.domain,
                from_state: current.state.clone(),
                to_state,
                // The wrong implementation reads the current version and
                // commits regardless of what the caller decided against:
                // blind last-write-wins.
                expected_version: current.version,
                next_version,
                committed_at,
            },
            event: EventDraft {
                event_id: event_id.clone(),
                object_id: current.object_id.clone(),
                domain: current.domain,
                object_version: next_version,
                event_type: "cognitiveos.state.transition.committed".to_owned(),
                canonical_json: canonical_text(&event_value)?,
            },
            record: RecordDraft {
                record_id,
                object_id: current.object_id.clone(),
                domain: current.domain,
                object_version: next_version,
                canonical_json: canonical_text(&record_value)?,
            },
            budget: None,
            outbox: vec![],
        })
        .map_err(|err| env_err(format!("bypass commit failed unexpectedly: {err}")))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Behavioral gates
// ---------------------------------------------------------------------------

/// `STATE-CAS-002` (state-conflict.json): drive a real committed history to
/// the vector's authoritative version, then attempt the stale write.
pub(super) fn cas_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let expected_version = vector
        .input
        .get("expected_version")
        .and_then(Value::as_i64)
        .ok_or_else(|| env_err("vector lacks integer expected_version"))?;
    let authoritative_version = vector
        .input
        .get("authoritative_version")
        .and_then(Value::as_i64)
        .ok_or_else(|| env_err("vector lacks integer authoritative_version"))?;
    let subject_ref = vector
        .input
        .get("state_ref")
        .and_then(Value::as_str)
        .unwrap_or("task://conformance/cas-subject");
    if authoritative_version < 2 || expected_version >= authoritative_version {
        return Err(env_err(
            "CAS vector no longer describes a stale write; refusing to fabricate a scenario",
        ));
    }

    let harness = KernelHarness::new()?;
    let engine = harness.engine();
    let object_id = oid(0x42)?;
    // Committed history: seed at ACTIVE (version 1), then ping-pong
    // ACTIVE<->BLOCKED through the real gate until the authoritative
    // version of the vector input is reached.
    harness.seed(&object_id, LifecycleDomain::Task, "ACTIVE", subject_ref)?;
    let mut current_version = Version::INITIAL;
    let mut stale_version: Option<Version> = None;
    let mut at_active = true;
    while current_version.get() < authoritative_version {
        if current_version.get() == expected_version {
            stale_version = Some(current_version);
        }
        let (from, to, reason) = if at_active {
            ("ACTIVE", "BLOCKED", "DEPENDENCY_PENDING")
        } else {
            ("BLOCKED", "ACTIVE", "DEPENDENCY_SATISFIED")
        };
        let step = command(
            LifecycleDomain::Task,
            &object_id,
            subject_ref,
            from,
            to,
            reason,
            current_version,
        )?;
        let committed = engine
            .commit_transition(&step)
            .map_err(|err| env_err(format!("history commit {from}->{to} rejected: {err}")))?;
        current_version = committed.after_version;
        at_active = !at_active;
    }
    let stale_version =
        stale_version.ok_or_else(|| env_err("stale version never observed in history"))?;
    let before = harness.load(LifecycleDomain::Task, &object_id)?;
    if before.version.get() != authoritative_version {
        return Err(env_err(format!(
            "harness reached version {} instead of {authoritative_version}",
            before.version.get()
        )));
    }
    let events_before = harness.event_count()?;

    // The stale write: from-state matches the authoritative state so the
    // version comparison is the deciding check.
    let (from, to, reason) = if at_active {
        ("ACTIVE", "BLOCKED", "DEPENDENCY_PENDING")
    } else {
        ("BLOCKED", "ACTIVE", "DEPENDENCY_SATISFIED")
    };
    let stale = command(
        LifecycleDomain::Task,
        &object_id,
        subject_ref,
        from,
        to,
        reason,
        stale_version,
    )?;

    let (actual, rejection_detail) = match kind {
        ImplementationKind::Reference => match engine.commit_transition(&stale) {
            Ok(committed) => (
                json!({
                    "decision": "accept",
                    "error": Value::Null,
                    "write_applied": true,
                    "audit_required": false,
                }),
                json!({ "unexpected_commit_version": committed.after_version.get() }),
            ),
            Err(rejection) => {
                let error = rejection_error(ctx, &rejection)?;
                (
                    json!({
                        "decision": "reject",
                        "error": error,
                        // Verified below against the reloaded row.
                        "write_applied": false,
                        // Contract constant: every governed denial is
                        // auditable (REQ-AUDIT-001); denial audit trail
                        // events are M3+ evidence.
                        "audit_required": true,
                    }),
                    json!({
                        "kind": format!("{:?}", rejection.kind),
                        "current_state": rejection.current_state.as_ref().map(|s| s.as_str().to_owned()),
                        "current_version": rejection.current_version.map(Version::get),
                        "available_exits": rejection.available_exits,
                    }),
                )
            }
        },
        ImplementationKind::DeliberatelyWrong => {
            // Blind last-write-wins: the wrong implementation ignores the
            // caller's stale decision basis and writes anyway.
            bypass_gate_commit(&harness, &before, to, subject_ref)?;
            (
                json!({
                    "decision": "accept",
                    "error": Value::Null,
                    "write_applied": true,
                    "audit_required": false,
                }),
                json!({ "gate": "bypassed" }),
            )
        }
    };

    // Post-conditions read back from the store (never assumed).
    let after = harness.load(LifecycleDomain::Task, &object_id)?;
    let events_after = harness.event_count()?;
    let state_unchanged = after.state == before.state && after.version == before.version;
    let write_observed = !state_unchanged || events_after != events_before;
    let mut actual = actual;
    actual["write_applied"] = json!(write_observed);

    Ok(GateOutput {
        actual,
        grounding: vec![
            "crates/cognitive-kernel (TransitionEngine centralized gate)".to_owned(),
            "crates/cognitive-store (SqliteAuthorityStore, WAL)".to_owned(),
            "specs/transitions/task.transitions.json".to_owned(),
            "specs/registry/errors.yaml#STATE_CONFLICT".to_owned(),
        ],
        informative: vec![],
        implementation: Some(match kind {
            ImplementationKind::Reference => REFERENCE_IMPLEMENTATION,
            ImplementationKind::DeliberatelyWrong => WRONG_IMPLEMENTATION,
        }),
        evidence: json!({
            "mode_note": "behavioral execution: real committed history in a throwaway SQLite WAL authority database",
            "history_commits": authoritative_version - 1,
            "authoritative_version_reached": before.version.get(),
            "stale_expected_version": stale_version.get(),
            "rejection": rejection_detail,
            "state_unchanged_after_attempt": state_unchanged,
            "event_log_growth": events_after as i64 - events_before as i64,
            "audit_required_basis": "contract constant for governed denials (REQ-AUDIT-001); denial audit events are M3+ behavior",
        }),
    })
}

/// `EFFECT-STATE-CLOSURE-008`: an Effect in `OUTCOME_UNKNOWN` must refuse
/// the illegal COMMITTED exit with the registered code, and the only
/// post-reconcile continuations under `still_unknown` are COMPENSATING and
/// QUARANTINED — both demonstrated by real commits.
pub(super) fn effect_closure_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let from = vector
        .input
        .get("effect_state")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("vector lacks effect_state"))?;
    let requested = vector
        .input
        .get("requested_transition")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("vector lacks requested_transition"))?;

    let harness = KernelHarness::new()?;
    let engine = harness.engine();
    let subject = "effect://conformance/closure-008";
    let effect_id = oid(0xe1)?;
    harness.seed(&effect_id, LifecycleDomain::Effect, from, subject)?;
    let before = harness.load(LifecycleDomain::Effect, &effect_id)?;

    let illegal = command(
        LifecycleDomain::Effect,
        &effect_id,
        subject,
        from,
        requested,
        "COMMIT_AUTHORIZED",
        before.version,
    )?;

    let (decision, error, allowed_exits) = match kind {
        ImplementationKind::Reference => match engine.commit_transition(&illegal) {
            Ok(_) => ("allow".to_owned(), Value::Null, vec![requested.to_owned()]),
            Err(rejection) => {
                let error = rejection_error(ctx, &rejection)?;
                let mut exits = rejection.available_exits.clone();
                exits.sort_unstable();
                ("deny".to_owned(), error, exits)
            }
        },
        ImplementationKind::DeliberatelyWrong => {
            // The wrong implementation commits the illegal exit directly
            // through the store port.
            bypass_gate_commit(&harness, &before, requested, subject)?;
            let mut exits = vec![requested.to_owned(), "RECONCILED".to_owned()];
            exits.sort_unstable();
            ("allow".to_owned(), Value::Null, exits)
        }
    };

    let after = harness.load(LifecycleDomain::Effect, &effect_id)?;
    let state_unchanged = after.state == before.state && after.version == before.version;

    // Post-reconcile continuations under still_unknown, demonstrated by
    // real commits on two further effects.
    let loaded = table(LifecycleDomain::Effect)
        .map_err(|err| env_err(format!("registered effect table: {err}")))?;
    let mut committed_exits: BTreeSet<String> = BTreeSet::new();
    for (tag, (target, reason)) in [
        ("COMPENSATING", "COMPENSATION_AUTHORIZED"),
        ("QUARANTINED", "SAFE_RECOVERY_UNAVAILABLE"),
    ]
    .iter()
    .enumerate()
    {
        let probe_id = oid(0xe2 + tag as u64)?;
        let probe_subject = format!("effect://conformance/still-unknown-{tag}");
        harness.seed(
            &probe_id,
            LifecycleDomain::Effect,
            "OUTCOME_UNKNOWN",
            &probe_subject,
        )?;
        let reconcile = command(
            LifecycleDomain::Effect,
            &probe_id,
            &probe_subject,
            "OUTCOME_UNKNOWN",
            "RECONCILED",
            "RECONCILIATION_STILL_UNKNOWN",
            Version::INITIAL,
        )?;
        let reconciled = engine
            .commit_transition(&reconcile)
            .map_err(|err| env_err(format!("still-unknown reconcile rejected: {err}")))?;
        let exit = command(
            LifecycleDomain::Effect,
            &probe_id,
            &probe_subject,
            "RECONCILED",
            target,
            reason,
            reconciled.after_version,
        )?;
        engine
            .commit_transition(&exit)
            .map_err(|err| env_err(format!("RECONCILED->{target} rejected: {err}")))?;
        committed_exits.insert((*target).to_owned());
    }
    // Completeness cross-check against the registered table: the committed
    // set must be exactly the still-unknown-guarded RECONCILED exits.
    let table_exits: BTreeSet<String> = loaded
        .table
        .transitions
        .iter()
        .filter(|edge| {
            edge.from == "RECONCILED"
                && edge
                    .guards
                    .iter()
                    .any(|g| g == "reconciliation_result_equals_still_unknown")
        })
        .map(|edge| edge.to.clone())
        .collect();
    if table_exits != committed_exits {
        return Err(env_err(format!(
            "registered still-unknown exits {table_exits:?} != behaviorally committed {committed_exits:?}"
        )));
    }

    Ok(GateOutput {
        actual: json!({
            "decision": decision,
            "error": error,
            "allowed_exits": allowed_exits,
            "post_reconcile_exits_when_still_unknown": committed_exits.iter().collect::<Vec<_>>(),
        }),
        grounding: vec![
            "crates/cognitive-kernel (TransitionEngine centralized gate)".to_owned(),
            "crates/cognitive-store (SqliteAuthorityStore, WAL)".to_owned(),
            "specs/transitions/effect.transitions.json".to_owned(),
            "specs/registry/errors.yaml#EFFECT_OUTCOME_UNKNOWN".to_owned(),
        ],
        informative: vec![],
        implementation: Some(match kind {
            ImplementationKind::Reference => REFERENCE_IMPLEMENTATION,
            ImplementationKind::DeliberatelyWrong => WRONG_IMPLEMENTATION,
        }),
        evidence: json!({
            "mode_note": "behavioral execution: illegal exit attempted on a real seeded Effect; still-unknown continuations committed for real on probe effects",
            "illegal_attempt_state_unchanged": state_unchanged,
            "still_unknown_exits_committed": committed_exits,
            "registered_table_cross_check": "committed set equals still-unknown-guarded RECONCILED exits",
        }),
    })
}

/// `GW-REMOTE-COMPLETE-001` (remote-completed-not-acceptance): a remote
/// `completed` report never moves the local task authority state; forcing
/// ACTIVE -> COMPLETED is rejected by the real gate with the registered
/// code, and CANDIDATE_COMPLETE is reachable only with the completion
/// claim and fixed post-state evidence.
pub(super) fn task_acceptance_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let local_state = vector
        .input
        .pointer("/local_task/state")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("vector lacks local_task.state"))?;
    let subject = vector
        .input
        .pointer("/local_task/ref")
        .and_then(Value::as_str)
        .unwrap_or("task://conformance/task-88");

    let harness = KernelHarness::new()?;
    let engine = harness.engine();
    let task_id = oid(0x88)?;
    harness.seed(&task_id, LifecycleDomain::Task, local_state, subject)?;
    let before = harness.load(LifecycleDomain::Task, &task_id)?;
    let events_before = harness.event_count()?;

    // The forced completion: remote "completed" used as if it were the
    // acceptance authority.
    let forced = command(
        LifecycleDomain::Task,
        &task_id,
        subject,
        local_state,
        "COMPLETED",
        "ACCEPTANCE_GRANTED",
        before.version,
    )?;

    let (error_if_forced, forced_detail) = match kind {
        ImplementationKind::Reference => match engine.commit_transition(&forced) {
            Ok(committed) => (
                Value::Null,
                json!({ "unexpected_commit_version": committed.after_version.get() }),
            ),
            Err(rejection) => (
                rejection_error(ctx, &rejection)?,
                json!({
                    "kind": format!("{:?}", rejection.kind),
                    "available_exits": rejection.available_exits,
                }),
            ),
        },
        ImplementationKind::DeliberatelyWrong => {
            bypass_gate_commit(&harness, &before, "COMPLETED", subject)?;
            (Value::Null, json!({ "gate": "bypassed" }))
        }
    };

    let after = harness.load(LifecycleDomain::Task, &task_id)?;
    let events_after = harness.event_count()?;
    let acceptance_committed = after.state.as_str() == "COMPLETED"
        || after.version != before.version
        || events_after != events_before;

    // CANDIDATE_COMPLETE admission discipline, demonstrated on a probe
    // task: without the completion claim / fixed post-state evidence the
    // gate refuses; with them it commits.
    let probe_id = oid(0x89)?;
    let probe_subject = "task://conformance/task-88-probe";
    harness.seed(&probe_id, LifecycleDomain::Task, "ACTIVE", probe_subject)?;
    let mut without_evidence = command(
        LifecycleDomain::Task,
        &probe_id,
        probe_subject,
        "ACTIVE",
        "CANDIDATE_COMPLETE",
        "COMPLETION_CLAIMED",
        Version::INITIAL,
    )?;
    without_evidence.evidence.clear();
    let refused_without_evidence = engine.commit_transition(&without_evidence).is_err();
    let with_evidence = command(
        LifecycleDomain::Task,
        &probe_id,
        probe_subject,
        "ACTIVE",
        "CANDIDATE_COMPLETE",
        "COMPLETION_CLAIMED",
        Version::INITIAL,
    )?;
    let committed_with_evidence = engine.commit_transition(&with_evidence).is_ok();
    let loaded = table(LifecycleDomain::Task)
        .map_err(|err| env_err(format!("registered task table: {err}")))?;
    let candidate_edge_evidence: BTreeSet<&str> = loaded
        .find_edge(
            &state_name("ACTIVE")?,
            &state_name("CANDIDATE_COMPLETE")?,
            "COMPLETION_CLAIMED",
        )
        .map(|edge| edge.required_evidence.iter().map(String::as_str).collect())
        .unwrap_or_default();
    let candidate_discipline = refused_without_evidence
        && committed_with_evidence
        && candidate_edge_evidence == BTreeSet::from(["completion_claim", "fixed_post_state"]);

    let treated_as = match kind {
        // Deterministic rule: the remote report is observation evidence;
        // the only path to authority state runs through the gate, which
        // refused it above.
        ImplementationKind::Reference => "observation_evidence",
        ImplementationKind::DeliberatelyWrong => "authority_state",
    };

    Ok(GateOutput {
        actual: json!({
            "local_task_state": after.state.as_str(),
            "acceptance_committed": acceptance_committed,
            "remote_completed_treated_as": treated_as,
            "candidate_complete_allowed_only_with_completion_claim_and_fixed_post_state":
                candidate_discipline,
            "error_if_forced": error_if_forced,
            "audit_required": matches!(kind, ImplementationKind::Reference),
        }),
        grounding: vec![
            "crates/cognitive-kernel (TransitionEngine centralized gate)".to_owned(),
            "crates/cognitive-store (SqliteAuthorityStore, WAL)".to_owned(),
            "specs/transitions/task.transitions.json".to_owned(),
            "specs/registry/errors.yaml#STATE_CONFLICT".to_owned(),
        ],
        informative: vec!["transition_to_completed_requires"],
        implementation: Some(match kind {
            ImplementationKind::Reference => REFERENCE_IMPLEMENTATION,
            ImplementationKind::DeliberatelyWrong => WRONG_IMPLEMENTATION,
        }),
        evidence: json!({
            "mode_note": "behavioral execution: forced completion attempted on a real seeded ACTIVE task",
            "forced_attempt": forced_detail,
            "task_state_after_attempt": after.state.as_str(),
            "event_log_growth": events_after as i64 - events_before as i64,
            "candidate_complete_probe": {
                "refused_without_evidence": refused_without_evidence,
                "committed_with_evidence": committed_with_evidence,
                "registered_edge_required_evidence": candidate_edge_evidence,
            },
            "transition_to_completed_requires_recorded_not_compared": {
                "registered_path": "ACTIVE -> CANDIDATE_COMPLETE (COMPLETION_CLAIMED; evidence completion_claim + fixed_post_state) -> COMPLETED (ACCEPTANCE_GRANTED; guards acceptance_authority_matches + verification_passed_and_current + fixed_post_state_unchanged; evidence verification_report + acceptance_decision)",
                "vector_prose": vector.expected.get("transition_to_completed_requires"),
            },
            "audit_required_basis": "contract constant for governed denials (REQ-AUDIT-001); denial audit events are M3+ behavior",
        }),
    })
}

/// Real read-only degradation probe for `STATE-STORE-DEGRADE-001` (M2
/// behavioral subset; recorded as assertions only — the vector stays
/// not-run until the M4 fault-injection framework covers disk-full and the
/// dispatch/stop/revoke expectations).
pub(super) fn store_degradation_behavioral_subset() -> Value {
    match store_degradation_probe() {
        Ok(value) => value,
        Err(err) => json!({ "probe_error": err.to_string() }),
    }
}

fn store_degradation_probe() -> Result<Value, ExecError> {
    let harness = KernelHarness::new()?;
    let subject = "task://conformance/degradation-probe";
    let object_id = oid(0xd0)?;
    harness.seed(&object_id, LifecycleDomain::Task, "ACTIVE", subject)?;
    {
        let engine = harness.engine();
        let step = command(
            LifecycleDomain::Task,
            &object_id,
            subject,
            "ACTIVE",
            "BLOCKED",
            "DEPENDENCY_PENDING",
            Version::INITIAL,
        )?;
        engine
            .commit_transition(&step)
            .map_err(|err| env_err(format!("probe history commit: {err}")))?;
    }
    let digest_before = replay_projection(&harness.store)
        .map_err(|err| env_err(format!("replay before degradation: {err}")))?
        .digest;
    let clock = FixedClock::new()?;
    let ids = SeqIds(AtomicU64::new(9000));
    // Close the writer connection but KEEP the temp directory guard alive:
    // dropping the whole harness would delete the database out from under
    // the reopen below (Linux deletes it for real; Windows only masks the
    // failure because the in-use delete is silently skipped).
    let KernelHarness {
        _dir: _dir_guard,
        db_path,
        store,
        clock: _,
        ids: _,
    } = harness;
    drop(store);

    // Degraded volume model: the same database opened read-only.
    let degraded = SqliteAuthorityStore::open_read_only(&db_path)
        .map_err(|err| env_err(format!("open read-only: {err}")))?;
    let engine = TransitionEngine::new(&degraded, &clock, &ids);
    let object_id = oid(0xd0)?;
    let unblock = command(
        LifecycleDomain::Task,
        &object_id,
        subject,
        "BLOCKED",
        "ACTIVE",
        "DEPENDENCY_SATISFIED",
        Version::new(2).map_err(|err| env_err(format!("version: {err}")))?,
    )?;
    let (write_rejected_fail_closed, degraded_code) = match engine.commit_transition(&unblock) {
        Ok(_) => (false, Value::Null),
        Err(rejection) => {
            let registered = rejection.registered();
            (
                true,
                json!({ "code": registered.code, "category": registered.category }),
            )
        }
    };
    let read_alive = degraded
        .load_object(LifecycleDomain::Task, &object_id)
        .ok()
        .flatten()
        .is_some();
    let state_after_rejection = degraded
        .load_object(LifecycleDomain::Task, &object_id)
        .ok()
        .flatten()
        .map(|o| (o.state.as_str().to_owned(), o.version.get()));
    let nothing_buffered = state_after_rejection
        .as_ref()
        .is_some_and(|(state, version)| state == "BLOCKED" && *version == 2);
    drop(degraded);

    // Recovery: storage restored, history intact, the same write commits.
    let restored = SqliteAuthorityStore::open(&db_path)
        .map_err(|err| env_err(format!("reopen writable: {err}")))?;
    let digest_after = replay_projection(&restored)
        .map_err(|err| env_err(format!("replay after recovery: {err}")))?
        .digest;
    let engine = TransitionEngine::new(&restored, &clock, &ids);
    let recovered_commit = engine.commit_transition(&unblock).is_ok();

    Ok(json!({
        "scope": "M2 behavioral read-only degradation subset, executed for real (KRN criterion 6 twin); recorded as assertions only — the vector stays not-run",
        "implementation": REFERENCE_IMPLEMENTATION,
        "governed_write_rejected_fail_closed": write_rejected_fail_closed,
        "degraded_write_error": degraded_code,
        "read_only_inspection_available": read_alive,
        "nothing_buffered_as_committed": nothing_buffered,
        "committed_history_lost": digest_before != digest_after,
        "replay_digest_stable_across_degradation": digest_before == digest_after,
        "same_write_commits_after_recovery": recovered_commit,
        "deferred_to_m4_or_later": [
            "disk_full fault injection (fault framework, M4)",
            "new_effect_dispatch_result (Effect dispatch path, M4)",
            "deterministic_stop_and_revoke_available (management plane, M5)",
            "already_executing_effects reconcile-or-quarantine (M4 recovery)",
            "telemetry_loss_affects_authoritative_state (telemetry plane, M6)",
        ],
    }))
}
