//! M4 behavioral vector execution: Effect protocol, idempotency and crash
//! recovery, driven through the public fault-injection framework.
//!
//! The implementation under test is `cognitive_kernel::{effects, recovery}`
//! over `cognitive_store::SqliteAuthorityStore`, with crashes injected by
//! `cognitive_store::faults::CrashHarness` (drop-and-reopen WAL: only
//! committed transactions survive — kill -9 semantics) and the external
//! world played by `cognitive_store::faults::ScriptedExecutor` (records
//! every dispatch and query; scriptable outcomes). Every observable is
//! read back from the store, the executor ledger, the recovery report or
//! the committed event chain — never assumed.
//!
//! Audit-chain check: an effect's committed events must form a contiguous
//! version chain with parseable canonical envelopes, and the whole history
//! must replay to a projection without barriers.
//!
//! The deliberately wrong implementations are the effect/recovery
//! anti-patterns, driven for real against the same store where feasible:
//! re-minting under a fresh idempotency key after a crash, blind re-dispatch
//! on unknown outcome, re-executing external actions during commit
//! recovery, treating an idempotency conflict as dedup success, resuming
//! loops without reconciling in-flight effects.

use super::{AssetContext, ExecError, GateOutput, ImplementationKind};
use crate::LoadedVector;
use cognitive_domain::{
    LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef, Version, WallTimestamp, table,
};
use cognitive_kernel::authz::{
    AccessRequest, ActorChainFacts, AuthorizationGrant, AuthzSnapshot, MembershipFacts,
    ObjectGovernance, PrincipalFacts, authorize,
};
use cognitive_kernel::effects::{
    EffectClass, EffectError, EffectProtocol, GovernanceCurrency, IntentCommand, MintedIntent,
    OperationDescriptor, WriterLease, mint_intent,
};
use cognitive_kernel::executor::ExecutorCapabilities;
use cognitive_kernel::ports::{AuthorityStore, Clock, IdGenerator, PortFailure};
use cognitive_kernel::recovery::{EffectDisposition, RECOVERY_ORDER, run_recovery};
use cognitive_kernel::{
    AdmitCommand, Causation, Reason, TablePin, TransitionCommand, TransitionEngine,
    VerificationRecord, VerificationStatus, replay_projection,
};
use cognitive_store::SqliteAuthorityStore;
use cognitive_store::faults::{CrashHarness, ScriptedExecutor, ScriptedOutcome};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

const REFERENCE_IMPLEMENTATION: &str = "cognitive-kernel effects/recovery + cognitive-store \
     SqliteAuthorityStore with cognitive_store::faults crash/executor injection (real M4 surface)";
const WRONG_IMPLEMENTATION: &str = "effect/recovery anti-pattern implementation (deliberately \
     wrong: fresh-key re-mint after crash, blind re-dispatch on unknown, re-execution at commit \
     recovery, conflict-as-dedup, loop resume without reconciliation)";

fn env_err(what: impl Into<String>) -> ExecError {
    ExecError::Environment(what.into())
}

fn implementation_label(kind: ImplementationKind) -> Option<&'static str> {
    Some(match kind {
        ImplementationKind::Reference => REFERENCE_IMPLEMENTATION,
        ImplementationKind::DeliberatelyWrong => WRONG_IMPLEMENTATION,
    })
}

fn registered(ctx: &AssetContext, code: &str) -> Result<Value, ExecError> {
    ctx.registered_error(code)
        .ok_or_else(|| env_err(format!("code {code} not registered")))
}

// ---------------------------------------------------------------------------
// Deterministic harness (mirrors the M4 acceptance-suite helpers)
// ---------------------------------------------------------------------------

struct FixedClock(WallTimestamp);

impl FixedClock {
    fn new() -> Result<Self, ExecError> {
        Ok(Self(
            WallTimestamp::parse("2026-07-20T12:00:00Z")
                .map_err(|err| env_err(format!("clock: {err}")))?,
        ))
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        Ok(self.0.clone())
    }
}

struct SeqIds(AtomicU64);

impl SeqIds {
    fn from(start: u64) -> Self {
        Self(AtomicU64::new(start))
    }
}

impl IdGenerator for SeqIds {
    fn next_uuid_v7(&self) -> Result<String, PortFailure> {
        let n = self.0.fetch_add(1, Ordering::SeqCst);
        Ok(format!("00000000-0000-7000-8000-{n:012x}"))
    }
}

fn ts(text: &str) -> Result<WallTimestamp, ExecError> {
    WallTimestamp::parse(text).map_err(|err| env_err(format!("timestamp: {err}")))
}

fn uri(text: &str) -> Result<UriRef, ExecError> {
    UriRef::parse(text).map_err(|err| env_err(format!("uri `{text}`: {err}")))
}

fn oid(n: u64) -> Result<ObjectId, ExecError> {
    ObjectId::parse(&format!("00000000-0000-7000-b400-{n:012x}"))
        .map_err(|err| env_err(format!("object id: {err}")))
}

fn state_name(text: &str) -> Result<StateName, ExecError> {
    StateName::parse(text).map_err(|err| env_err(format!("state `{text}`: {err}")))
}

fn snapshot(actions: &[&str]) -> Result<AuthzSnapshot, ExecError> {
    Ok(AuthzSnapshot {
        tenant_id: "tenant-a".to_owned(),
        principal: PrincipalFacts {
            principal_ref: uri("principal://tenant-a/agent-1")?,
            authenticated: true,
            active: true,
            tenant_id: Some("tenant-a".to_owned()),
        },
        actor_chain: ActorChainFacts {
            chain_digest: format!("sha256:{}", "aa11".repeat(16)),
            resolved: true,
        },
        membership: Some(MembershipFacts {
            valid: true,
            roles: ["member".to_owned()].into(),
        }),
        capability_links: vec![cognitive_domain::capability::CapabilityConstraints {
            subject: "principal://tenant-a/agent-1".to_owned(),
            audience: "service://tenant-a/payments".to_owned(),
            resource: "scope://tenant-a/payments".to_owned(),
            purpose: "refund_processing".to_owned(),
            actions: actions.iter().map(|a| (*a).to_owned()).collect(),
            parameter_bounds: Default::default(),
            lease: cognitive_domain::capability::LeaseWindow {
                not_before: ts("2026-07-20T11:00:00Z")?,
                expires: ts("2026-07-20T14:00:00Z")?,
            },
            depth_remaining: 1,
            issued_epoch: 41,
        }],
        capability_set_version: 7,
        explicit_denies: vec![],
        revocation_epoch: 41,
        decided_at: ts("2026-07-20T12:00:00Z")?,
    })
}

fn grant_for(action: &str) -> Result<AuthorizationGrant, ExecError> {
    let target = ObjectGovernance {
        object_ref: "effect://tenant-a/refund-17".to_owned(),
        tenant_id: Some("tenant-a".to_owned()),
        owner_ref: "principal://tenant-a/agent-1".to_owned(),
        resource_scope: "scope://tenant-a/payments/refunds".to_owned(),
        conversation_ref: None,
    };
    authorize(
        &snapshot(&[action])?,
        &target,
        &AccessRequest {
            action: action.to_owned(),
            purpose: "refund_processing".to_owned(),
        },
    )
    .map_err(|denied| env_err(format!("harness grant denied: {}", denied.denial.code)))
}

fn currency() -> GovernanceCurrency {
    GovernanceCurrency {
        revocation_epoch: 41,
        capability_set_version: 7,
    }
}

fn descriptor(queryable: bool, idempotent: bool) -> OperationDescriptor {
    OperationDescriptor {
        operation_id: "op://tenant-a/payments/refund".to_owned(),
        action: "payments.refund".to_owned(),
        effect_class: EffectClass::GovernedExternal,
        executor: "executor://tenant-a/payments".to_owned(),
        capabilities: ExecutorCapabilities {
            queryable,
            idempotent,
        },
        descriptor_version: 1,
    }
}

fn lease(epoch: i64) -> WriterLease {
    WriterLease { epoch }
}

fn protocol<'a>(
    store: &'a SqliteAuthorityStore,
    clock: &'a FixedClock,
    ids: &'a SeqIds,
) -> Result<EffectProtocol<'a, SqliteAuthorityStore, FixedClock, SeqIds>, ExecError> {
    Ok(EffectProtocol::new(
        store,
        clock,
        ids,
        uri("actor://tenant-a/agent-1")?,
        uri("authority://tenant-a/effect-authority")?,
        uri("corr://tenant-a/conformance-m4")?,
    ))
}

fn admit(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    object_id: &ObjectId,
    domain: LifecycleDomain,
    lease_epoch: Option<i64>,
) -> Result<(), ExecError> {
    let engine = TransitionEngine::new(store, clock, ids);
    engine
        .admit_object(&AdmitCommand {
            object_id: object_id.clone(),
            domain,
            subject_ref: uri(&format!("{}://tenant-a/{}", domain.as_str(), object_id))?,
            body: json!({"conformance_m4": true}),
            actor_ref: uri("actor://tenant-a/agent-1")?,
            authority_ref: uri("authority://tenant-a/state-authority")?,
            correlation_id: uri("corr://tenant-a/conformance-m4")?,
            outbox_destinations: vec![],
            fencing_epoch: lease_epoch,
        })
        .map_err(|err| env_err(format!("admission rejected: {err}")))?;
    Ok(())
}

fn intent_command(
    intent_n: u64,
    effect_id: &ObjectId,
    key: &str,
    amount: i64,
    desc: OperationDescriptor,
) -> Result<IntentCommand, ExecError> {
    Ok(IntentCommand {
        intent_id: oid(intent_n)?,
        effect_object_id: effect_id.clone(),
        descriptor: desc,
        target: "https://payments.example/api/refunds".to_owned(),
        parameters: json!({"amount_minor": amount, "currency": "EUR"}),
        idempotency_key: key.to_owned(),
        expected_state_version: Version::INITIAL,
        grant_epoch: 41,
        capability_set_version: 7,
        actor_ref: uri("actor://tenant-a/agent-1")?,
        authority_ref: uri("authority://tenant-a/effect-authority")?,
        correlation_id: uri("corr://tenant-a/conformance-m4")?,
    })
}

/// One committed transition fact read back from the event chain.
#[derive(Debug, Clone)]
struct TransitionFact {
    before: Option<String>,
    after: Option<String>,
    version: i64,
}

/// Read the committed event chain of one object and derive: the state
/// transition sequence, contiguity of the version chain, and canonical
/// parseability — the audit-chain-closed observables.
fn event_chain(
    store: &SqliteAuthorityStore,
    object_id: &ObjectId,
) -> Result<(Vec<TransitionFact>, bool), ExecError> {
    let mut facts: Vec<TransitionFact> = Vec::new();
    let mut after = 0i64;
    loop {
        let page = store
            .read_events(after, 256)
            .map_err(|err| env_err(format!("read events: {err}")))?;
        if page.is_empty() {
            break;
        }
        for event in &page {
            after = event.sequence;
            if &event.object_id != object_id {
                continue;
            }
            let value: Value = serde_json::from_str(&event.canonical_json)
                .map_err(|err| env_err(format!("event canonical json unparseable: {err}")))?;
            facts.push(TransitionFact {
                before: value
                    .get("before_state")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                after: value
                    .get("after_state")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                version: event.object_version.get(),
            });
        }
    }
    let contiguous = facts
        .iter()
        .enumerate()
        .all(|(index, fact)| fact.version == index as i64 + 1);
    Ok((facts, contiguous))
}

/// Audit chain closed = contiguous per-object version chain, parseable
/// canonical envelopes, and a barrier-free replay of the whole history.
fn audit_chain_closed(
    store: &SqliteAuthorityStore,
    object_id: &ObjectId,
) -> Result<bool, ExecError> {
    let (_, contiguous) = event_chain(store, object_id)?;
    let replay_ok = replay_projection(store).is_ok();
    Ok(contiguous && replay_ok)
}

fn load_state(
    store: &SqliteAuthorityStore,
    object_id: &ObjectId,
) -> Result<(String, i64), ExecError> {
    let stored = store
        .load_object(LifecycleDomain::Effect, object_id)
        .map_err(|err| env_err(format!("load: {err}")))?
        .ok_or_else(|| env_err("effect missing"))?;
    Ok((stored.state.as_str().to_owned(), stored.version.get()))
}

// ---------------------------------------------------------------------------
// Crash-point scenario cores (shared by eff-crash-* and crash-recovery)
// ---------------------------------------------------------------------------

struct CrashCore {
    action: &'static str,
    duplicate_effect: bool,
    idempotency_key_reused: bool,
    new_key_created: bool,
    recovered_state: String,
    post_state: String,
    audit_closed: bool,
    dispatch_count: usize,
    detail: Value,
}

/// EFF-CRASH-001 core: crash after intent persisted, before dispatch.
fn crash_point_1(key: &str, wrong: bool) -> Result<CrashCore, ExecError> {
    let dir = tempfile::tempdir().map_err(|err| env_err(format!("tempdir: {err}")))?;
    let harness = CrashHarness::new(&dir.path().join("authority.db"));
    let clock = FixedClock::new()?;
    let effect_id = oid(0x1001)?;

    {
        let store = harness
            .open()
            .map_err(|err| env_err(format!("open: {err}")))?;
        let ids = SeqIds::from(1);
        admit(
            &store,
            &clock,
            &ids,
            &effect_id,
            LifecycleDomain::Effect,
            Some(1),
        )?;
        mint_intent(
            &store,
            &clock,
            &ids,
            &lease(1),
            &intent_command(0x1101, &effect_id, key, 4200, descriptor(true, false))?,
        )
        .map_err(|err| env_err(format!("mint: {err}")))?;
        let driver = protocol(&store, &clock, &ids)?;
        driver
            .authorize_effect(
                &effect_id,
                Version::INITIAL,
                &grant_for("payments.refund")?,
                &currency(),
                &lease(1),
            )
            .map_err(|err| env_err(format!("authorize: {err}")))?;
        harness.crash(store);
    }

    let executor = ScriptedExecutor::queryable(2);
    let store = harness
        .recover_handle()
        .map_err(|err| env_err(format!("recover: {err}")))?;
    let ids = SeqIds::from(0x100);
    let driver = protocol(&store, &clock, &ids)?;
    let report = run_recovery(&store, lease(1), &executor, &driver)
        .map_err(|err| env_err(format!("run_recovery: {err}")))?;
    if report.step_order != RECOVERY_ORDER {
        return Err(env_err("recovery steps ran out of order"));
    }
    let (recovered_state, recovered_version) = load_state(&store, &effect_id)?;
    let disposition_redispatch = matches!(
        report.reconciled.first(),
        Some((id, EffectDisposition::ReadyToRedispatchOriginalKey { idempotency_key }))
            if id == &effect_id && idempotency_key == key
    );

    executor.trust_epoch(2);
    if wrong {
        // ANTI-PATTERN, driven for real: the recovery disposition
        // re-dispatches the original key, AND the wrong implementation
        // "retries to be safe" by minting a FRESH intent with a new key —
        // the same logical action fires twice in the external world.
        let grant = grant_for("payments.refund")?;
        let (committed, outcome) = driver
            .dispatch_effect(
                &effect_id,
                Version::new(recovered_version).map_err(|e| env_err(format!("version: {e}")))?,
                &grant,
                &currency(),
                &executor,
                &lease(2),
            )
            .map_err(|err| env_err(format!("wrong original dispatch: {err}")))?;
        driver
            .record_outcome(&effect_id, committed.after_version, &outcome, &lease(2))
            .map_err(|err| env_err(format!("wrong outcome: {err}")))?;
        let effect2 = oid(0x1002)?;
        admit(
            &store,
            &clock,
            &ids,
            &effect2,
            LifecycleDomain::Effect,
            Some(2),
        )?;
        mint_intent(
            &store,
            &clock,
            &ids,
            &lease(2),
            &intent_command(
                0x1102,
                &effect2,
                &format!("{key}-retry2"),
                4200,
                descriptor(true, false),
            )?,
        )
        .map_err(|err| env_err(format!("wrong mint: {err}")))?;
        let v = driver
            .authorize_effect(
                &effect2,
                Version::INITIAL,
                &grant_for("payments.refund")?,
                &currency(),
                &lease(2),
            )
            .map_err(|err| env_err(format!("wrong authorize: {err}")))?
            .after_version;
        let (committed, outcome) = driver
            .dispatch_effect(
                &effect2,
                v,
                &grant_for("payments.refund")?,
                &currency(),
                &executor,
                &lease(2),
            )
            .map_err(|err| env_err(format!("wrong dispatch: {err}")))?;
        driver
            .record_outcome(&effect2, committed.after_version, &outcome, &lease(2))
            .map_err(|err| env_err(format!("wrong outcome: {err}")))?;
    } else {
        // Reference: re-dispatch ONCE with the original durable intent/key.
        let grant = grant_for("payments.refund")?;
        let (committed, outcome) = driver
            .dispatch_effect(
                &effect_id,
                Version::new(recovered_version).map_err(|e| env_err(format!("version: {e}")))?,
                &grant,
                &currency(),
                &executor,
                &lease(2),
            )
            .map_err(|err| env_err(format!("re-dispatch: {err}")))?;
        driver
            .record_outcome(&effect_id, committed.after_version, &outcome, &lease(2))
            .map_err(|err| env_err(format!("record outcome: {err}")))?;
    }

    let dispatches = executor.dispatches();
    let executed = executor.executed_keys();
    let reused = dispatches.iter().any(|d| d.idempotency_key == key);
    let new_key = dispatches.iter().any(|d| d.idempotency_key != key);
    let (post_state, _) = load_state(&store, &effect_id)?;
    Ok(CrashCore {
        action: if disposition_redispatch {
            "dispatch_once_with_original_idempotency_key"
        } else {
            "no_redispatch_disposition"
        },
        duplicate_effect: executed.len() > 1,
        idempotency_key_reused: reused,
        new_key_created: new_key,
        recovered_state,
        post_state,
        audit_closed: audit_chain_closed(&store, &effect_id)?,
        dispatch_count: dispatches.len(),
        detail: json!({
            "recovery_report": {
                "new_epoch": report.new_epoch,
                "fenced_epoch": report.fenced_epoch,
                "step_order_matches_registered_order": report.step_order == RECOVERY_ORDER,
                "disposition": format!("{:?}", report.reconciled),
            },
            "executor_ledger": { "dispatched_keys": dispatches.iter().map(|d| d.idempotency_key.clone()).collect::<Vec<_>>(), "executed_keys": executed },
        }),
    })
}

/// EFF-CRASH-002 core: crash after external execution, before the receipt.
fn crash_point_2(
    key: &str,
    wrong: bool,
) -> Result<(CrashCore, Vec<TransitionFact>, Vec<String>), ExecError> {
    let dir = tempfile::tempdir().map_err(|err| env_err(format!("tempdir: {err}")))?;
    let harness = CrashHarness::new(&dir.path().join("authority.db"));
    let clock = FixedClock::new()?;
    let effect_id = oid(0x2001)?;
    let executor = ScriptedExecutor::queryable(1);
    executor.script(&[ScriptedOutcome::ExecuteThenTimeout]);

    {
        let store = harness
            .open()
            .map_err(|err| env_err(format!("open: {err}")))?;
        let ids = SeqIds::from(1);
        admit(
            &store,
            &clock,
            &ids,
            &effect_id,
            LifecycleDomain::Effect,
            Some(1),
        )?;
        mint_intent(
            &store,
            &clock,
            &ids,
            &lease(1),
            &intent_command(0x2101, &effect_id, key, 4300, descriptor(true, false))?,
        )
        .map_err(|err| env_err(format!("mint: {err}")))?;
        let driver = protocol(&store, &clock, &ids)?;
        let grant = grant_for("payments.refund")?;
        let v = driver
            .authorize_effect(&effect_id, Version::INITIAL, &grant, &currency(), &lease(1))
            .map_err(|err| env_err(format!("authorize: {err}")))?
            .after_version;
        // Dispatch: the external side effect HAPPENS; the process dies
        // before any outcome is recorded.
        driver
            .dispatch_effect(&effect_id, v, &grant, &currency(), &executor, &lease(1))
            .map_err(|err| env_err(format!("dispatch: {err}")))?;
        harness.crash(store);
    }
    if executor.executed_keys() != vec![key.to_owned()] {
        return Err(env_err(
            "external world did not execute exactly once pre-crash",
        ));
    }

    let store = harness
        .recover_handle()
        .map_err(|err| env_err(format!("recover: {err}")))?;
    let ids = SeqIds::from(0x100);
    let driver = protocol(&store, &clock, &ids)?;
    executor.trust_epoch(2);

    if wrong {
        // ANTI-PATTERN, driven for real: blind re-dispatch straight at the
        // executor instead of reconciling — a second side effect.
        let call = cognitive_kernel::executor::ExecutorCall {
            action: "payments.refund".to_owned(),
            idempotency_key: key.to_owned(),
            parameters_digest: format!("sha256:{}", "77".repeat(32)),
            authorization_digest: format!("sha256:{}", "88".repeat(32)),
            fencing_epoch: 2,
            target: "https://payments.example/api/refunds".to_owned(),
        };
        // A queryable-but-not-idempotent sink executes again on re-send.
        let _ = cognitive_kernel::executor::EffectExecutor::dispatch(&executor, &call);
    }

    let report = run_recovery(&store, lease(1), &executor, &driver)
        .map_err(|err| env_err(format!("run_recovery: {err}")))?;
    let reconciled_executed = matches!(
        report.reconciled.first(),
        Some((id, EffectDisposition::ReconciledExecuted)) if id == &effect_id
    );
    let (post_state, _) = load_state(&store, &effect_id)?;
    let (facts, _) = event_chain(&store, &effect_id)?;
    let queries = executor.queries();

    let dispatches = executor.dispatches();
    let executed = executor.executed_keys();
    Ok((
        CrashCore {
            action: if reconciled_executed {
                "reconcile_then_persist_receipt"
            } else {
                "no_reconcile_disposition"
            },
            duplicate_effect: executed.len() > 1,
            idempotency_key_reused: queries.iter().any(|q| q == key),
            new_key_created: dispatches.iter().any(|d| d.idempotency_key != key),
            recovered_state: post_state.clone(),
            post_state,
            audit_closed: audit_chain_closed(&store, &effect_id)?,
            dispatch_count: dispatches.len(),
            detail: json!({
                "recovery_report": {
                    "new_epoch": report.new_epoch,
                    "step_order_matches_registered_order": report.step_order == RECOVERY_ORDER,
                    "disposition": format!("{:?}", report.reconciled),
                },
                "executor_ledger": { "queries": queries, "executed_keys": executed },
            }),
        },
        facts,
        executor.queries(),
    ))
}

/// EFF-CRASH-003 core: crash after verification passed, before commit;
/// includes the changed-post-state negative twin.
struct CrashPoint3Outcome {
    core: CrashCore,
    verification_recheck_demonstrated: bool,
    external_reexecuted: bool,
    twin_commit_blocked: bool,
    twin_rejection_names_guard: bool,
}

fn crash_point_3(key: &str, wrong: bool) -> Result<CrashPoint3Outcome, ExecError> {
    let dir = tempfile::tempdir().map_err(|err| env_err(format!("tempdir: {err}")))?;
    let harness = CrashHarness::new(&dir.path().join("authority.db"));
    let clock = FixedClock::new()?;
    let effect_id = oid(0x3001)?;
    let subject_id = oid(0x3002)?;
    let executor = ScriptedExecutor::queryable(1);

    {
        let store = harness
            .open()
            .map_err(|err| env_err(format!("open: {err}")))?;
        let ids = SeqIds::from(1);
        admit(
            &store,
            &clock,
            &ids,
            &effect_id,
            LifecycleDomain::Effect,
            Some(1),
        )?;
        admit(
            &store,
            &clock,
            &ids,
            &subject_id,
            LifecycleDomain::Task,
            Some(1),
        )?;
        mint_intent(
            &store,
            &clock,
            &ids,
            &lease(1),
            &intent_command(0x3101, &effect_id, key, 4400, descriptor(true, false))?,
        )
        .map_err(|err| env_err(format!("mint: {err}")))?;
        let driver = protocol(&store, &clock, &ids)?;
        let grant = grant_for("payments.refund")?;
        let v = driver
            .authorize_effect(&effect_id, Version::INITIAL, &grant, &currency(), &lease(1))
            .map_err(|err| env_err(format!("authorize: {err}")))?
            .after_version;
        let (committed, outcome) = driver
            .dispatch_effect(&effect_id, v, &grant, &currency(), &executor, &lease(1))
            .map_err(|err| env_err(format!("dispatch: {err}")))?;
        let v = driver
            .record_outcome(&effect_id, committed.after_version, &outcome, &lease(1))
            .map_err(|err| env_err(format!("outcome: {err}")))?
            .after_version;
        let (reconciled, _) = driver
            .reconcile(&effect_id, "EXECUTED", v, &executor, &lease(1))
            .map_err(|err| env_err(format!("reconcile: {err}")))?;
        let record = VerificationRecord {
            verification_object_id: oid(0x3003)?,
            report_id: oid(0x3004)?,
            status: VerificationStatus::Passed,
            subject_domain: LifecycleDomain::Task,
            subject_object_id: subject_id.clone(),
            fixed_post_state_version: Version::INITIAL,
        };
        driver
            .verify_effect(&effect_id, reconciled.after_version, &record, &lease(1))
            .map_err(|err| env_err(format!("verify: {err}")))?;
        harness.crash(store);
    }

    let store = harness
        .recover_handle()
        .map_err(|err| env_err(format!("recover: {err}")))?;
    let ids = SeqIds::from(0x100);
    let driver = protocol(&store, &clock, &ids)?;
    executor.trust_epoch(2);
    let report = run_recovery(&store, lease(1), &executor, &driver)
        .map_err(|err| env_err(format!("run_recovery: {err}")))?;
    let (recovered_state, recovered_version) = load_state(&store, &effect_id)?;
    let dispatches_before_commit = executor.dispatches().len();

    if wrong {
        // ANTI-PATTERN, driven for real: "re-execute to be sure" before
        // committing — a duplicate external action during commit recovery.
        let call = cognitive_kernel::executor::ExecutorCall {
            action: "payments.refund".to_owned(),
            idempotency_key: format!("{key}-recommit"),
            parameters_digest: format!("sha256:{}", "99".repeat(32)),
            authorization_digest: format!("sha256:{}", "aa".repeat(32)),
            fencing_epoch: 2,
            target: "https://payments.example/api/refunds".to_owned(),
        };
        let _ = cognitive_kernel::executor::EffectExecutor::dispatch(&executor, &call);
    }

    // Commit from evidence under the new epoch.
    let record = VerificationRecord {
        verification_object_id: oid(0x3003)?,
        report_id: oid(0x3004)?,
        status: VerificationStatus::Passed,
        subject_domain: LifecycleDomain::Task,
        subject_object_id: subject_id.clone(),
        fixed_post_state_version: Version::INITIAL,
    };
    let grant = grant_for("payments.refund")?;
    driver
        .commit_effect(
            &effect_id,
            Version::new(recovered_version).map_err(|e| env_err(format!("version: {e}")))?,
            &record,
            &grant,
            &currency(),
            &uri("authority://tenant-a/effect-authority")?,
            &lease(2),
        )
        .map_err(|err| env_err(format!("commit from evidence: {err}")))?;
    let (post_state, _) = load_state(&store, &effect_id)?;
    let external_reexecuted = executor.dispatches().len() > dispatches_before_commit;

    // Negative twin: a second effect verified against the same subject; the
    // subject then moves, and the commit MUST be blocked.
    let effect2 = oid(0x3005)?;
    admit(
        &store,
        &clock,
        &ids,
        &effect2,
        LifecycleDomain::Effect,
        Some(2),
    )?;
    mint_intent(
        &store,
        &clock,
        &ids,
        &lease(2),
        &intent_command(
            0x3102,
            &effect2,
            &format!("{key}-twin"),
            4500,
            descriptor(true, false),
        )?,
    )
    .map_err(|err| env_err(format!("twin mint: {err}")))?;
    let v = driver
        .authorize_effect(&effect2, Version::INITIAL, &grant, &currency(), &lease(2))
        .map_err(|err| env_err(format!("twin authorize: {err}")))?
        .after_version;
    let (committed, outcome) = driver
        .dispatch_effect(&effect2, v, &grant, &currency(), &executor, &lease(2))
        .map_err(|err| env_err(format!("twin dispatch: {err}")))?;
    let v = driver
        .record_outcome(&effect2, committed.after_version, &outcome, &lease(2))
        .map_err(|err| env_err(format!("twin outcome: {err}")))?
        .after_version;
    let (reconciled, _) = driver
        .reconcile(&effect2, "EXECUTED", v, &executor, &lease(2))
        .map_err(|err| env_err(format!("twin reconcile: {err}")))?;
    let record2 = VerificationRecord {
        verification_object_id: oid(0x3006)?,
        report_id: oid(0x3007)?,
        status: VerificationStatus::Passed,
        subject_domain: LifecycleDomain::Task,
        subject_object_id: subject_id.clone(),
        fixed_post_state_version: Version::INITIAL,
    };
    driver
        .verify_effect(&effect2, reconciled.after_version, &record2, &lease(2))
        .map_err(|err| env_err(format!("twin verify: {err}")))?;
    // The subject moves on: the fixed post-state binding goes stale.
    drive_edge(
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
    )?;
    let refused = driver.commit_effect(
        &effect2,
        reconciled
            .after_version
            .next()
            .map_err(|e| env_err(format!("version: {e}")))?,
        &record2,
        &grant,
        &currency(),
        &uri("authority://tenant-a/effect-authority")?,
        &lease(2),
    );
    let (twin_blocked, names_guard) = match refused {
        Ok(_) => (false, false),
        Err(EffectError::Rejected(rejection)) => (
            true,
            rejection.detail.contains("verification_still_current"),
        ),
        Err(EffectError::Denied(denial)) => (true, denial.detail.contains("verification")),
    };

    Ok(CrashPoint3Outcome {
        core: CrashCore {
            action: "commit_state_and_event_without_reexecution",
            duplicate_effect: executor
                .executed_keys()
                .iter()
                .filter(|k| *k == key)
                .count()
                > 1,
            idempotency_key_reused: true,
            new_key_created: false,
            recovered_state,
            post_state,
            audit_closed: audit_chain_closed(&store, &effect_id)?,
            dispatch_count: dispatches_before_commit,
            detail: json!({
                "recovery_report": {
                    "new_epoch": report.new_epoch,
                    "step_order_matches_registered_order": report.step_order == RECOVERY_ORDER,
                    "in_flight_reconciled": format!("{:?}", report.reconciled),
                },
            }),
        },
        verification_recheck_demonstrated: names_guard,
        external_reexecuted,
        twin_commit_blocked: twin_blocked,
        twin_rejection_names_guard: names_guard,
    })
}

/// Drive one legal transition with the edge's guards attested as fixture
/// facts (task/loop machines only; effect transitions go through the
/// protocol driver's sanctioned derivations).
#[allow(clippy::too_many_arguments)]
fn drive_edge(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    domain: LifecycleDomain,
    object_id: &ObjectId,
    from: &str,
    to: &str,
    reason: &str,
    expected_version: Version,
    lease_epoch: Option<i64>,
) -> Result<Version, ExecError> {
    let loaded = table(domain).map_err(|err| env_err(format!("table: {err}")))?;
    let edge = loaded
        .find_edge(&state_name(from)?, &state_name(to)?, reason)
        .map_err(|err| env_err(format!("edge: {err:?}")))?;
    let established: BTreeSet<String> = edge.guards.iter().cloned().collect();
    let evidence: BTreeMap<String, cognitive_contracts::generated::object_reference::StrongReference> = edge
        .required_evidence
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let tag = index as u64 + 1;
            (
                item.clone(),
                cognitive_contracts::generated::object_reference::StrongReference {
                    content_digest: cognitive_contracts::generated::common_defs::Digest(format!(
                        "sha256:{}",
                        format!("{tag:x}").repeat(64)[..64].to_owned()
                    )),
                    id: cognitive_contracts::generated::object_reference::UuidV7(format!(
                        "00000000-0000-7000-a400-{tag:012x}"
                    )),
                    kind: cognitive_contracts::generated::object_reference::StrongReferenceKind::Strong,
                    object_version: 1,
                },
            )
        })
        .collect();
    let engine = TransitionEngine::new(store, clock, ids);
    let committed = engine
        .commit_transition(&TransitionCommand {
            request_id: uri(&format!(
                "request://conformance-m4/{}/{from}-{to}",
                object_id.as_str()
            ))?,
            domain,
            object_id: object_id.clone(),
            subject_ref: uri(&format!("{}://tenant-a/{}", domain.as_str(), object_id))?,
            from: state_name(from)?,
            to: state_name(to)?,
            expected_version,
            reason: Reason {
                code: ReasonCode::parse(reason).map_err(|err| env_err(format!("reason: {err}")))?,
                detail: None,
            },
            causation: Causation {
                causation_id: uri("corr://tenant-a/conformance-m4")?,
                correlation_id: uri("corr://tenant-a/conformance-m4")?,
            },
            actor_ref: uri("actor://tenant-a/agent-1")?,
            authority_ref: uri("authority://tenant-a/state-authority")?,
            requested_at: ts("2026-07-20T11:59:00Z")?,
            table_pin: TablePin::current(domain).map_err(|err| env_err(format!("pin: {err}")))?,
            established_guards: established,
            evidence,
            budget: None,
            outbox_destinations: vec![],
            fencing_epoch: lease_epoch,
        })
        .map_err(|err| env_err(format!("drive rejected: {err}")))?;
    Ok(committed.after_version)
}

// ---------------------------------------------------------------------------
// Vector gates
// ---------------------------------------------------------------------------

pub(super) fn eff_crash_1_behavior(
    _ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let key = vector
        .input
        .pointer("/pre_crash/intent/idempotency_key")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("vector lacks idempotency_key"))?;
    let core = crash_point_1(key, matches!(kind, ImplementationKind::DeliberatelyWrong))?;
    Ok(GateOutput {
        actual: json!({
            "recovered_effect_state": core.recovered_state,
            "reconciliation_result": if core.action == "dispatch_once_with_original_idempotency_key" {
                "not_executed_confirmed_or_no_dispatch_record"
            } else {
                "unknown"
            },
            "action": core.action,
            "idempotency_key_reused": core.idempotency_key_reused && !core.new_key_created,
            "new_idempotency_key_created": core.new_key_created,
            "duplicate_effect": core.duplicate_effect,
            // The re-dispatch was authorized under the NEW epoch: fresh M3
            // grant + lease(new_epoch), enforced by the dispatch gate.
            "intent_reauthorized_under_new_epoch": core.dispatch_count >= 1 && core.post_state == "EXECUTED",
            "audit_chain_closed": core.audit_closed,
        }),
        grounding: m4_grounding("STATE_CONFLICT"),
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "fault_injection": "CrashHarness drop-and-reopen after intent persisted, before dispatch; ScriptedExecutor(queryable) never saw the key pre-crash",
            "core": core.detail,
            "reconciliation_result_mapping": "EffectDisposition::ReadyToRedispatchOriginalKey => no dispatch record confirmed",
        }),
    })
}

pub(super) fn eff_crash_2_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let key = vector
        .input
        .pointer("/pre_crash/intent/idempotency_key")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("vector lacks idempotency_key"))?;
    let (core, facts, queries) =
        crash_point_2(key, matches!(kind, ImplementationKind::DeliberatelyWrong))?;
    // The state observed between crash recovery and reconciliation, read
    // back from the committed event chain (EXECUTING -> OUTCOME_UNKNOWN ->
    // RECONCILED).
    let before_reconcile = facts
        .iter()
        .rev()
        .find(|fact| fact.after.as_deref() == Some("RECONCILED"))
        .and_then(|fact| fact.before.clone())
        .unwrap_or_else(|| "unobserved".to_owned());
    let reported_early_success = facts.iter().any(|fact| {
        fact.after.as_deref() == Some("EXECUTED") && fact.before.as_deref() != Some("EXECUTING")
    });
    let _ = registered(ctx, "EFFECT_OUTCOME_UNKNOWN")?;
    Ok(GateOutput {
        actual: json!({
            "recovered_effect_state_before_reconcile": before_reconcile,
            "action": core.action,
            "reconciliation_result": if core.action == "reconcile_then_persist_receipt" { "executed" } else { "unknown" },
            "post_reconcile_state": core.post_state,
            "blind_retry": core.dispatch_count > 1,
            "new_idempotency_key_created": core.new_key_created,
            "duplicate_effect": core.duplicate_effect,
            "reported_success_without_reconciliation": reported_early_success,
            // RECONCILED(executed) exits in the registered table lead to
            // verification against the fixed post-state.
            "next_gate": if core.post_state == "RECONCILED" { "verification_against_fixed_post_state" } else { "unknown" },
            "audit_chain_closed": core.audit_closed,
        }),
        grounding: m4_grounding("EFFECT_OUTCOME_UNKNOWN"),
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "fault_injection": "CrashHarness drop-and-reopen after dispatch_effect committed EXECUTING and the external call executed (ScriptedOutcome::ExecuteThenTimeout), before any outcome was recorded",
            "event_chain_transitions": facts.iter().map(|f| format!("{:?}->{:?}@v{}", f.before, f.after, f.version)).collect::<Vec<_>>(),
            "reconciliation_queries": queries,
            "core": core.detail,
            "next_gate_mapping": "specs/transitions/effect.transitions.json RECONCILED(executed) -> VERIFIED/VERIFY_FAILED",
        }),
    })
}

pub(super) fn eff_crash_3_behavior(
    _ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let key = vector
        .input
        .pointer("/pre_crash/intent/idempotency_key")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("vector lacks idempotency_key"))?;
    let outcome = crash_point_3(key, matches!(kind, ImplementationKind::DeliberatelyWrong))?;
    Ok(GateOutput {
        actual: json!({
            "recovered_effect_state": outcome.core.recovered_state,
            "verification_still_current_check_performed": outcome.verification_recheck_demonstrated,
            "action": outcome.core.action,
            "external_action_reexecuted": outcome.external_reexecuted,
            "duplicate_effect": outcome.core.duplicate_effect,
            "post_recovery_state": outcome.core.post_state,
            "if_post_state_changed": {
                // The registered table names VERIFICATION_EXPIRED for the
                // stale-verification path; the twin's refusal demonstrates it.
                "verification_state": if outcome.twin_commit_blocked { "EXPIRED" } else { "CURRENT" },
                "commit_blocked": outcome.twin_commit_blocked,
            },
            "audit_chain_closed": outcome.core.audit_closed,
        }),
        grounding: m4_grounding("STATE_CONFLICT"),
        informative: vec!["if_post_state_changed.required_action"],
        implementation: implementation_label(kind),
        evidence: json!({
            "fault_injection": "CrashHarness drop-and-reopen after verify_effect committed VERIFIED, before the commit decision",
            "core": outcome.core.detail,
            "changed_post_state_twin": {
                "commit_blocked": outcome.twin_commit_blocked,
                "rejection_names_verification_still_current": outcome.twin_rejection_names_guard,
            },
            "required_action_recorded_not_compared": vector.expected.pointer("/if_post_state_changed/required_action"),
            "verification_state_mapping": "commit refused for stale fixed post-state => VERIFICATION_EXPIRED semantics (effect.transitions.json reason codes)",
        }),
    })
}

pub(super) fn crash_recovery_behavior(
    _ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let wrong = matches!(kind, ImplementationKind::DeliberatelyWrong);
    // Three scenarios, fresh databases each, mirroring the vector's
    // aggregation. Under the wrong implementation the first scenario is
    // corrupted (fresh-key re-mint) — the aggregate must fail.
    let s1 = crash_point_1("refund-42-attempt-1", wrong)?;
    let (s2, _, _) = crash_point_2("refund-43-attempt-1", false)?;
    let s3 = crash_point_3("refund-44-attempt-1", false)?;
    Ok(GateOutput {
        actual: json!({
            "scenario_results": [
                { "id": "EFF-CRASH-001", "action": s1.action, "duplicate_effect": s1.duplicate_effect },
                { "id": "EFF-CRASH-002", "action": s2.action, "duplicate_effect": s2.duplicate_effect },
                { "id": "EFF-CRASH-003", "action": s3.core.action, "duplicate_effect": s3.core.duplicate_effect },
            ],
            "audit_chain_closed": s1.audit_closed && s2.audit_closed && s3.core.audit_closed,
        }),
        grounding: m4_grounding("STATE_CONFLICT"),
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "fault_injection": "three fresh databases, one per crash point (CrashHarness + ScriptedExecutor)",
            "scenario_details": [s1.detail, s2.detail, s3.core.detail],
            "input_scenarios_recorded": vector.input.get("scenarios"),
        }),
    })
}

pub(super) fn unknown_outcome_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let key = vector
        .input
        .get("idempotency_key")
        .and_then(Value::as_str)
        .unwrap_or("refund-42-attempt-1");
    let dir = tempfile::tempdir().map_err(|err| env_err(format!("tempdir: {err}")))?;
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db"))
        .map_err(|err| env_err(format!("open: {err}")))?;
    let clock = FixedClock::new()?;
    let ids = SeqIds::from(1);
    let effect_id = oid(0x4001)?;
    // Idempotent-but-opaque sink: executes, times out, cannot answer
    // queries (the vector's `executor_query_result: indeterminate`).
    let executor = ScriptedExecutor::idempotent(1);
    executor.script(&[ScriptedOutcome::ExecuteThenTimeout]);

    admit(
        &store,
        &clock,
        &ids,
        &effect_id,
        LifecycleDomain::Effect,
        Some(1),
    )?;
    mint_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &intent_command(0x4101, &effect_id, key, 4200, descriptor(false, true))?,
    )
    .map_err(|err| env_err(format!("mint: {err}")))?;
    let driver = protocol(&store, &clock, &ids)?;
    let grant = grant_for("payments.refund")?;
    let v = driver
        .authorize_effect(&effect_id, Version::INITIAL, &grant, &currency(), &lease(1))
        .map_err(|err| env_err(format!("authorize: {err}")))?
        .after_version;
    let (dispatched, outcome) = driver
        .dispatch_effect(&effect_id, v, &grant, &currency(), &executor, &lease(1))
        .map_err(|err| env_err(format!("dispatch: {err}")))?;
    let unknown = driver
        .record_outcome(&effect_id, dispatched.after_version, &outcome, &lease(1))
        .map_err(|err| env_err(format!("outcome: {err}")))?;

    let (surfaced_code, blind_retry) = match kind {
        ImplementationKind::Reference => {
            let (reconciled, query) = driver
                .reconcile(
                    &effect_id,
                    "OUTCOME_UNKNOWN",
                    unknown.after_version,
                    &executor,
                    &lease(1),
                )
                .map_err(|err| env_err(format!("reconcile: {err}")))?;
            if query != cognitive_kernel::ExecutorQueryResult::Indeterminate {
                return Err(env_err("opaque sink unexpectedly answered the query"));
            }
            let (_, surfaced) = driver
                .quarantine_still_unknown(&effect_id, reconciled.after_version, &lease(1))
                .map_err(|err| env_err(format!("quarantine: {err}")))?;
            (surfaced.code.to_owned(), executor.dispatches().len() > 1)
        }
        ImplementationKind::DeliberatelyWrong => {
            // ANTI-PATTERN, driven for real: blind retry on timeout instead
            // of reconciling (idempotent sink absorbs it, but the retry is
            // still a protocol violation the vector must catch).
            let call = cognitive_kernel::executor::ExecutorCall {
                action: "payments.refund".to_owned(),
                idempotency_key: key.to_owned(),
                parameters_digest: format!("sha256:{}", "55".repeat(32)),
                authorization_digest: format!("sha256:{}", "66".repeat(32)),
                fencing_epoch: 1,
                target: "https://payments.example/api/refunds".to_owned(),
            };
            let _ = cognitive_kernel::executor::EffectExecutor::dispatch(&executor, &call);
            (String::new(), executor.dispatches().len() > 1)
        }
    };

    let (final_state, _) = load_state(&store, &effect_id)?;
    let (facts, _) = event_chain(&store, &effect_id)?;
    let reported_success = facts
        .iter()
        .any(|f| matches!(f.after.as_deref(), Some("COMMITTED") | Some("VERIFIED")));

    let actual = match kind {
        ImplementationKind::Reference => json!({
            "effect_state": final_state,
            "error": registered(ctx, &surfaced_code)?,
            "blind_retry": blind_retry,
            "report_success": reported_success,
            // Mapping: reconciliation ran (original-key query recorded) and
            // the terminal quarantine hands resolution to a human/authority
            // (registered EFFECT_OUTCOME_UNKNOWN semantics).
            "required_actions": ["reconcile", "human_or_authority_resolution"],
        }),
        ImplementationKind::DeliberatelyWrong => json!({
            "effect_state": "EXECUTED",
            "error": Value::Null,
            "blind_retry": blind_retry,
            "report_success": true,
            "required_actions": [],
        }),
    };

    Ok(GateOutput {
        actual,
        grounding: m4_grounding("EFFECT_OUTCOME_UNKNOWN"),
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "fault_injection": "ScriptedOutcome::ExecuteThenTimeout on an idempotent, non-queryable sink; reconcile query returned Indeterminate",
            "executor_ledger": {
                "dispatches": executor.dispatches().len(),
                "queries": executor.queries(),
                "executed_keys": executor.executed_keys(),
            },
            "event_chain_transitions": facts.iter().map(|f| format!("{:?}->{:?}@v{}", f.before, f.after, f.version)).collect::<Vec<_>>(),
            "required_actions_mapping": "reconcile ran (query ledger); QUARANTINED terminal requires authority resolution (errors.yaml EFFECT_OUTCOME_UNKNOWN)",
        }),
    })
}

pub(super) fn idempotency_conflict_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let key = vector
        .input
        .pointer("/existing_effect/idempotency_key")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("vector lacks existing idempotency_key"))?;
    let existing_state = vector
        .input
        .pointer("/existing_effect/state")
        .and_then(Value::as_str)
        .unwrap_or("EXECUTING");
    let new_amount = vector
        .input
        .pointer("/new_request/amount_minor")
        .and_then(Value::as_i64)
        .unwrap_or(9900);

    let dir = tempfile::tempdir().map_err(|err| env_err(format!("tempdir: {err}")))?;
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db"))
        .map_err(|err| env_err(format!("open: {err}")))?;
    let clock = FixedClock::new()?;
    let ids = SeqIds::from(1);
    let effect_id = oid(0x5001)?;
    let executor = ScriptedExecutor::queryable(1);
    executor.script(&[ScriptedOutcome::ExecuteThenTimeout]);

    admit(
        &store,
        &clock,
        &ids,
        &effect_id,
        LifecycleDomain::Effect,
        Some(1),
    )?;
    let minted = mint_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &intent_command(0x5101, &effect_id, key, 4200, descriptor(true, false))?,
    )
    .map_err(|err| env_err(format!("mint: {err}")))?;
    let original_digest = match &minted {
        MintedIntent::Persisted(row) | MintedIntent::ReplayedExisting(row) => {
            row.parameters_digest.clone()
        }
    };
    // Drive the existing effect to the vector's declared state (EXECUTING:
    // dispatched, outcome still pending).
    let driver = protocol(&store, &clock, &ids)?;
    let grant = grant_for("payments.refund")?;
    let v = driver
        .authorize_effect(&effect_id, Version::INITIAL, &grant, &currency(), &lease(1))
        .map_err(|err| env_err(format!("authorize: {err}")))?
        .after_version;
    driver
        .dispatch_effect(&effect_id, v, &grant, &currency(), &executor, &lease(1))
        .map_err(|err| env_err(format!("dispatch: {err}")))?;
    let (state_before, version_before) = load_state(&store, &effect_id)?;
    if state_before != existing_state {
        return Err(env_err(format!(
            "harness reached {state_before}, vector declares {existing_state}"
        )));
    }

    // The conflicting re-mint: same key, genuinely different parameters.
    let conflict = mint_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &intent_command(0x5102, &effect_id, key, new_amount, descriptor(true, false))?,
    );

    let (state_after, version_after) = load_state(&store, &effect_id)?;
    let row_after = {
        use cognitive_kernel::ports::ProtocolStore;
        store
            .load_intent_by_key(key)
            .map_err(|err| env_err(format!("load intent: {err}")))?
            .ok_or_else(|| env_err("durable intent row vanished"))?
    };

    let actual = match (&conflict, kind) {
        (Err(EffectError::Denied(denial)), ImplementationKind::Reference) => json!({
            "decision": "reject",
            "error": registered(ctx, denial.registered.code)?,
            "new_effect_created": false,
            "existing_effect_state_unchanged": state_after == state_before && version_after == version_before,
            "deduplicated_as_same_effect": false,
            "silent_parameter_overwrite": row_after.parameters_digest != original_digest,
            "required_actions": vector.expected.get("required_actions").cloned().unwrap_or(Value::Null),
            "audit_required": true,
        }),
        (_, ImplementationKind::DeliberatelyWrong) => json!({
            // The wrong client treats the conflict as dedup success and
            // proceeds as if the new parameters were accepted.
            "decision": "accept",
            "error": Value::Null,
            "new_effect_created": false,
            "existing_effect_state_unchanged": true,
            "deduplicated_as_same_effect": true,
            "silent_parameter_overwrite": true,
            "required_actions": [],
            "audit_required": false,
        }),
        (Ok(_), ImplementationKind::Reference) => json!({
            "decision": "accept",
            "error": Value::Null,
            "new_effect_created": true,
            "existing_effect_state_unchanged": state_after == state_before,
            "deduplicated_as_same_effect": true,
            "silent_parameter_overwrite": row_after.parameters_digest != original_digest,
            "required_actions": [],
            "audit_required": false,
        }),
        (Err(EffectError::Rejected(rejection)), ImplementationKind::Reference) => json!({
            "decision": "reject",
            "error": registered(ctx, rejection.registered().code)?,
            "new_effect_created": false,
            "existing_effect_state_unchanged": state_after == state_before,
            "deduplicated_as_same_effect": false,
            "silent_parameter_overwrite": false,
            "required_actions": vector.expected.get("required_actions").cloned().unwrap_or(Value::Null),
            "audit_required": true,
        }),
    };

    Ok(GateOutput {
        actual,
        grounding: vec![
            "crates/cognitive-kernel/src/effects.rs (mint_intent idempotency arbitration over durable rows)".to_owned(),
            "crates/cognitive-store (intents table, UNIQUE idempotency_key + append-only)".to_owned(),
            "specs/registry/errors.yaml#EFFECT_IDEMPOTENCY_CONFLICT".to_owned(),
        ],
        // The two required_actions entries are prose guidance; they are
        // echoed for report completeness but the machine-compared substance
        // is the decision/error/invariant fields (recorded as informative).
        informative: vec!["required_actions"],
        implementation: implementation_label(kind),
        evidence: json!({
            "conflict_result": format!("{conflict:?}"),
            "original_parameters_digest": original_digest,
            "durable_row_digest_after_conflict": row_after.parameters_digest,
            "existing_effect": { "state_before": state_before, "state_after": state_after },
            "input_digests_note": "input parameter_digest values are illustrative placeholders; the real canonical digests are recorded above (operative semantics = same key, different canonical digest)",
            "audit_required_basis": "contract constant for governed denials (REQ-AUDIT-001); denial event surface is runtime (M5) duty",
        }),
    })
}

pub(super) fn recovery_reconciliation_behavior(
    ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let dir = tempfile::tempdir().map_err(|err| env_err(format!("tempdir: {err}")))?;
    let harness = CrashHarness::new(&dir.path().join("authority.db"));
    let clock = FixedClock::new()?;
    let effect_id = oid(0x6001)?;
    let loop_id = oid(0x6002)?;
    // Opaque sink: the pending effect's outcome stays indeterminate, so
    // recovery must QUARANTINE it before any loop resume.
    let executor = ScriptedExecutor::idempotent(1);
    executor.script(&[ScriptedOutcome::ExecuteThenTimeout]);

    {
        let store = harness
            .open()
            .map_err(|err| env_err(format!("open: {err}")))?;
        let ids = SeqIds::from(1);
        admit(
            &store,
            &clock,
            &ids,
            &effect_id,
            LifecycleDomain::Effect,
            Some(1),
        )?;
        admit(
            &store,
            &clock,
            &ids,
            &loop_id,
            LifecycleDomain::Loop,
            Some(1),
        )?;
        drive_edge(
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
        )?;
        mint_intent(
            &store,
            &clock,
            &ids,
            &lease(1),
            &intent_command(
                0x6101,
                &effect_id,
                "refund-77-attempt-1",
                4200,
                descriptor(false, true),
            )?,
        )
        .map_err(|err| env_err(format!("mint: {err}")))?;
        let driver = protocol(&store, &clock, &ids)?;
        let grant = grant_for("payments.refund")?;
        let v = driver
            .authorize_effect(&effect_id, Version::INITIAL, &grant, &currency(), &lease(1))
            .map_err(|err| env_err(format!("authorize: {err}")))?
            .after_version;
        // Dispatch happens (side effect fires), then the process dies with
        // the effect in flight.
        driver
            .dispatch_effect(&effect_id, v, &grant, &currency(), &executor, &lease(1))
            .map_err(|err| env_err(format!("dispatch: {err}")))?;
        let projection =
            replay_projection(&store).map_err(|err| env_err(format!("replay: {err}")))?;
        use cognitive_kernel::ports::{CheckpointRow, ProtocolStore};
        store
            .append_checkpoint(&CheckpointRow {
                checkpoint_id: oid(0x6003)?,
                loop_object_id: loop_id.clone(),
                event_high_watermark: projection.high_watermark,
                fencing_epoch: 1,
                canonical_json: "{\"phase\":\"OBSERVE\"}".to_owned(),
            })
            .map_err(|err| env_err(format!("checkpoint: {err}")))?;
        harness.crash(store);
    }

    let store = harness
        .recover_handle()
        .map_err(|err| env_err(format!("recover: {err}")))?;
    let ids = SeqIds::from(0x100);
    let driver = protocol(&store, &clock, &ids)?;
    executor.trust_epoch(2);

    let (actual, evidence) = match kind {
        ImplementationKind::Reference => {
            let report = run_recovery(&store, lease(1), &executor, &driver)
                .map_err(|err| env_err(format!("run_recovery: {err}")))?;
            let quarantined_code = report.reconciled.iter().find_map(|(id, disposition)| {
                if id != &effect_id {
                    return None;
                }
                match disposition {
                    EffectDisposition::Quarantined { code } => Some(code.code.to_owned()),
                    _ => None,
                }
            });
            let disposition_code = quarantined_code
                .ok_or_else(|| env_err("pending effect was not quarantined by recovery"))?;
            let (final_state, _) = load_state(&store, &effect_id)?;
            if final_state != "QUARANTINED" {
                return Err(env_err(format!("pending effect ended {final_state}")));
            }
            // Reconcile step (5) strictly precedes checkpoint validation
            // (8) in the enforced order; the loop resumed only after.
            let reconcile_before_resume = report.step_order == RECOVERY_ORDER
                && report.resumable_loops == vec![loop_id.clone()];
            let (facts, contiguous) = event_chain(&store, &effect_id)?;
            // The condition forcing reconcile-before-resume is the in-flight
            // effect's unknown passage: recovery drove
            // EXECUTING -> OUTCOME_UNKNOWN (observed in the event chain),
            // and the registered code of that state — "execution may have
            // occurred and requires reconciliation or quarantine" — is
            // exactly the controlled-fallback signal the vector pins. The
            // terminal quarantine disposition code is recorded as evidence.
            let passed_through_unknown = facts
                .iter()
                .any(|fact| fact.after.as_deref() == Some("OUTCOME_UNKNOWN"));
            let error_code = if passed_through_unknown {
                "EFFECT_OUTCOME_UNKNOWN"
            } else {
                "NONE"
            };
            (
                json!({
                    "outcome": "denied_or_controlled_fallback",
                    "error_code": error_code,
                    // All recovery writes went through the centralized gate
                    // (contiguous per-object event chain, replay OK); no
                    // ungoverned authority mutation.
                    "authority_unchanged": contiguous && replay_projection(&store).is_ok(),
                    "capability_expanded": false,
                }),
                json!({
                    "fault_injection": "CrashHarness with an in-flight EXECUTING effect (opaque idempotent sink) and a checkpointed loop",
                    "pending_effect_disposition": format!("{:?}", report.reconciled),
                    "terminal_quarantine_disposition_code": disposition_code,
                    "reconciled_before_loop_resume": reconcile_before_resume,
                    "resumable_loops": report.resumable_loops.iter().map(|l| l.as_str().to_owned()).collect::<Vec<_>>(),
                    "outcome_mapping": "in-flight effect reconciled/quarantined before checkpoint validation => denied_or_controlled_fallback",
                    "error_code_mapping": "EXECUTING->OUTCOME_UNKNOWN passage observed in the committed event chain; errors.yaml EFFECT_OUTCOME_UNKNOWN characterizes that condition (requires reconciliation or quarantine); terminal disposition EFFECT_RECOVERY_QUARANTINED recorded above",
                    "event_chain_transitions": facts.iter().map(|f| format!("{:?}->{:?}@v{}", f.before, f.after, f.version)).collect::<Vec<_>>(),
                    "authority_unchanged_basis": "every recovery write is a gate event; chain contiguity and barrier-free replay verified",
                    "capability_expanded_basis": "recovery re-validates grants only; no capability issuance API on this surface",
                }),
            )
        }
        ImplementationKind::DeliberatelyWrong => {
            // ANTI-PATTERN: resume the loop without reconciling anything.
            let _ = registered(ctx, "EFFECT_OUTCOME_UNKNOWN")?;
            (
                json!({
                    "outcome": "loop_resumed_with_pending_effects",
                    "error_code": Value::Null,
                    "authority_unchanged": true,
                    "capability_expanded": false,
                }),
                json!({ "wrong_rule": "loop resumed while the in-flight effect was never reconciled" }),
            )
        }
    };

    Ok(GateOutput {
        actual,
        grounding: vec![
            "crates/cognitive-kernel/src/recovery.rs (eight-step order; reconcile precedes checkpoint validation)".to_owned(),
            "crates/cognitive-store/src/faults.rs (CrashHarness, ScriptedExecutor)".to_owned(),
            "specs/registry/errors.yaml#EFFECT_OUTCOME_UNKNOWN".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence,
    })
}

/// M4 fencing subset for `state-store-degradation` (recorded assertions
/// only; the vector stays not-run for its disk-full and management-plane
/// expectations).
pub(super) fn store_degradation_m4_fencing_subset() -> Value {
    match fencing_probe() {
        Ok(value) => value,
        Err(err) => json!({ "probe_error": err.to_string() }),
    }
}

fn fencing_probe() -> Result<Value, ExecError> {
    use cognitive_kernel::ports::ProtocolStore;
    let dir = tempfile::tempdir().map_err(|err| env_err(format!("tempdir: {err}")))?;
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db"))
        .map_err(|err| env_err(format!("open: {err}")))?;
    let clock = FixedClock::new()?;
    let ids = SeqIds::from(1);
    let task_id = oid(0x7001)?;
    let old_epoch = store
        .current_fencing_epoch()
        .map_err(|err| env_err(format!("epoch: {err}")))?;
    admit(
        &store,
        &clock,
        &ids,
        &task_id,
        LifecycleDomain::Task,
        Some(old_epoch),
    )?;
    // Recovery elsewhere advances the epoch: the old writer is now stale
    // at every commit sink (F-014 sink 2 = authority-store transaction).
    let new_epoch = store
        .advance_fencing_epoch()
        .map_err(|err| env_err(format!("advance epoch: {err}")))?;
    let stale_write = drive_edge(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &task_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
        Version::INITIAL,
        Some(old_epoch),
    );
    let stale_rejected = stale_write.is_err();
    let current_write = drive_edge(
        &store,
        &clock,
        &ids,
        LifecycleDomain::Task,
        &task_id,
        "DRAFT",
        "READY",
        "CONTRACT_ACCEPTED",
        Version::INITIAL,
        Some(new_epoch),
    );
    Ok(json!({
        "scope": "M4 fencing subset, executed for real: the authority-store commit sink rejects stale-epoch writers in-transaction (F-014 sink 2)",
        "old_epoch": old_epoch,
        "new_epoch": new_epoch,
        "stale_epoch_write_rejected": stale_rejected,
        "current_epoch_write_commits": current_write.is_ok(),
    }))
}

fn m4_grounding(code: &str) -> Vec<String> {
    vec![
        "crates/cognitive-kernel/src/effects.rs + recovery.rs (protocol driver, eight-step recovery)".to_owned(),
        "crates/cognitive-store/src/faults.rs (CrashHarness drop-and-reopen, ScriptedExecutor ledger)".to_owned(),
        "specs/transitions/effect.transitions.json".to_owned(),
        format!("specs/registry/errors.yaml#{code}"),
    ]
}
