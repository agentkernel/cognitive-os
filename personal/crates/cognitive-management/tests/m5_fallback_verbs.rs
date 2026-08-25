//! M5 batch 1 library behavior: the deterministic management plane keeps
//! inspect / stop / revoke / reconcile available with no model anywhere
//! (REQ-MGMT-FALLBACK-001, vector `management-deterministic-fallback.json`
//! semantics — the vector itself stays not-run for Lane-CFR), gates every
//! verb behind a valid PrivilegedManagementSession
//! (REQ-MGMT-SESSION-002/003, REQ-MGMT-GATE-001,
//! REQ-MGMT-SESSION-LIFECYCLE-001), and reuses the M3/M4 kernel public
//! APIs for revocation and reconciliation.
//!
//! All assertions are made against the REAL SQLite WAL authority store,
//! reloaded after each verb — never against in-memory echoes.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_arguments
)]

use cognitive_contracts::generated::audit_commit_receipt::OrdinaryCoreAuditCommitReceipt;
use cognitive_contracts::generated::privileged_read_decision::{
    OrdinaryCorePrivilegedReadDecision, OrdinaryCorePrivilegedReadDecisionOutcome,
    OrdinaryCorePrivilegedReadDecisionRecordKind, OrdinaryCorePrivilegedReadDecisionSafeReason,
};
use cognitive_domain::capability::{CapabilityConstraints, LeaseWindow};
use cognitive_domain::{
    LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef, Version, WallTimestamp, table,
};
use cognitive_kernel::authz::{
    AccessRequest, ActorChainFacts, AuthorizationGrant, AuthzSnapshot, MembershipFacts,
    ObjectGovernance, PrincipalFacts, authorize, revalidate_grant,
};
use cognitive_kernel::effects::{
    EffectClass, EffectProtocol, GovernanceCurrency, IntentCommand, OperationDescriptor,
    WriterLease, mint_intent,
};
use cognitive_kernel::executor::ExecutorCapabilities;
use cognitive_kernel::ports::{AuthorityStore, Clock, IdGenerator, PortFailure, ProtocolStore};
use cognitive_kernel::recovery::RECOVERY_ORDER;
use cognitive_kernel::{
    AdmitCommand, Causation, Reason, TablePin, TransitionCommand, TransitionEngine,
};
use cognitive_management::{
    AuditPortFailure, AuditedInspectError, GovernanceLedger, InspectRequest, ManagementAuditPort,
    ManagementError, ManagementPlane, ModelProvider, PrivilegedManagementSession, StopRequest,
};
use cognitive_store::SqliteAuthorityStore;
use cognitive_store::faults::{ScriptedExecutor, ScriptedOutcome};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------
// Fixtures (m4_common shapes, duplicated: integration suites cannot be
// shared across crates)
// ---------------------------------------------------------------------

struct FixedClock(WallTimestamp);

impl FixedClock {
    fn new() -> Self {
        Self(WallTimestamp::parse("2026-07-20T12:00:00Z").unwrap())
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        Ok(self.0.clone())
    }
}

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

fn ts(text: &str) -> WallTimestamp {
    WallTimestamp::parse(text).unwrap()
}

fn uri(text: &str) -> UriRef {
    UriRef::parse(text).unwrap()
}

fn state(name: &str) -> StateName {
    StateName::parse(name).unwrap()
}

fn oid(n: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{n:012x}")).unwrap()
}

fn evidence_ref(n: u64) -> cognitive_contracts::generated::object_reference::StrongReference {
    use cognitive_contracts::generated::common_defs::Digest;
    use cognitive_contracts::generated::object_reference::{
        StrongReference, StrongReferenceKind, UuidV7,
    };
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

fn admit(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    object_id: &ObjectId,
    domain: LifecycleDomain,
) {
    let engine = TransitionEngine::new(store, clock, ids);
    engine
        .admit_object(&AdmitCommand {
            object_id: object_id.clone(),
            domain,
            subject_ref: uri(&format!("{}://tenant-a/{}", domain.as_str(), object_id)),
            body: json!({"m5": true}),
            actor_ref: uri("actor://tenant-a/agent-1"),
            authority_ref: uri("authority://tenant-a/state-authority"),
            correlation_id: uri("corr://tenant-a/m5-verbs"),
            outbox_destinations: vec![],
            fencing_epoch: None,
        })
        .unwrap();
}

fn drive(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    domain: LifecycleDomain,
    object_id: &ObjectId,
    from: &str,
    to: &str,
    reason: &str,
    expected_version: Version,
) -> Version {
    let loaded = table(domain).unwrap();
    let edge = loaded.find_edge(&state(from), &state(to), reason).unwrap();
    let established: BTreeSet<String> = edge.guards.iter().cloned().collect();
    let evidence: BTreeMap<String, _> = edge
        .required_evidence
        .iter()
        .enumerate()
        .map(|(index, item)| (item.clone(), evidence_ref(index as u64 + 1)))
        .collect();
    let engine = TransitionEngine::new(store, clock, ids);
    engine
        .commit_transition(&TransitionCommand {
            request_id: uri(&format!("request://m5/{}/{from}-{to}", object_id.as_str())),
            domain,
            object_id: object_id.clone(),
            subject_ref: uri(&format!("{}://tenant-a/{}", domain.as_str(), object_id)),
            from: state(from),
            to: state(to),
            expected_version,
            reason: Reason {
                code: ReasonCode::parse(reason).unwrap(),
                detail: None,
            },
            causation: Causation {
                causation_id: uri("corr://tenant-a/m5-verbs"),
                correlation_id: uri("corr://tenant-a/m5-verbs"),
            },
            actor_ref: uri("actor://tenant-a/agent-1"),
            authority_ref: uri("authority://tenant-a/state-authority"),
            requested_at: ts("2026-07-20T11:59:00Z"),
            table_pin: TablePin::current(domain).unwrap(),
            established_guards: established,
            evidence,
            budget: None,
            outbox_destinations: vec![],
            fencing_epoch: None,
        })
        .unwrap()
        .after_version
}

fn seed_runnable_execution(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    n: u64,
) -> (ObjectId, Version) {
    let id = oid(n);
    admit(store, clock, ids, &id, LifecycleDomain::AgentExecution);
    let v2 = drive(
        store,
        clock,
        ids,
        LifecycleDomain::AgentExecution,
        &id,
        "CREATED",
        "ADMITTED",
        "ADMISSION_GRANTED",
        Version::INITIAL,
    );
    let v3 = drive(
        store,
        clock,
        ids,
        LifecycleDomain::AgentExecution,
        &id,
        "ADMITTED",
        "RUNNABLE",
        "SCHEDULING_ENABLED",
        v2,
    );
    (id, v3)
}

fn capability_link(actions: &[&str]) -> CapabilityConstraints {
    CapabilityConstraints {
        subject: "principal://tenant-a/agent-1".to_owned(),
        audience: "service://tenant-a/payments".to_owned(),
        resource: "scope://tenant-a/payments".to_owned(),
        purpose: "refund_processing".to_owned(),
        actions: actions.iter().map(|a| (*a).to_owned()).collect(),
        parameter_bounds: Default::default(),
        lease: LeaseWindow {
            not_before: ts("2026-07-20T11:00:00Z"),
            expires: ts("2030-01-01T00:00:00Z"),
        },
        depth_remaining: 1,
        issued_epoch: 41,
    }
}

fn snapshot(actions: &[&str]) -> AuthzSnapshot {
    AuthzSnapshot {
        tenant_id: "tenant-a".to_owned(),
        principal: PrincipalFacts {
            principal_ref: uri("principal://tenant-a/agent-1"),
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
        capability_links: vec![capability_link(actions)],
        capability_set_version: 7,
        explicit_denies: vec![],
        revocation_epoch: 41,
        decided_at: ts("2026-07-20T12:00:00Z"),
    }
}

fn grant_for(action: &str) -> AuthorizationGrant {
    authorize(
        &snapshot(&[action]),
        &ObjectGovernance {
            object_ref: "effect://tenant-a/refund-17".to_owned(),
            tenant_id: Some("tenant-a".to_owned()),
            owner_ref: "principal://tenant-a/agent-1".to_owned(),
            resource_scope: "scope://tenant-a/payments/refunds".to_owned(),
            conversation_ref: None,
        },
        &AccessRequest {
            action: action.to_owned(),
            purpose: "refund_processing".to_owned(),
        },
    )
    .unwrap()
}

fn currency() -> GovernanceCurrency {
    GovernanceCurrency {
        revocation_epoch: 41,
        capability_set_version: 7,
    }
}

fn descriptor() -> OperationDescriptor {
    OperationDescriptor {
        operation_id: "op://tenant-a/payments/refund".to_owned(),
        action: "payments.refund".to_owned(),
        effect_class: EffectClass::GovernedExternal,
        executor: "executor://tenant-a/payments".to_owned(),
        capabilities: ExecutorCapabilities {
            queryable: true,
            idempotent: false,
        },
        descriptor_version: 1,
    }
}

fn protocol<'a>(
    store: &'a SqliteAuthorityStore,
    clock: &'a FixedClock,
    ids: &'a SeqIds,
) -> EffectProtocol<'a, SqliteAuthorityStore, FixedClock, SeqIds> {
    EffectProtocol::new(
        store,
        clock,
        ids,
        uri("actor://tenant-a/agent-1"),
        uri("authority://tenant-a/effect-authority"),
        uri("corr://tenant-a/m5-verbs"),
    )
}

fn seed_authorized_effect(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    effect_n: u64,
    intent_n: u64,
    key: &str,
) -> (ObjectId, Version) {
    let effect_id = oid(effect_n);
    admit(store, clock, ids, &effect_id, LifecycleDomain::Effect);
    mint_intent(
        store,
        clock,
        ids,
        &WriterLease { epoch: 1 },
        &IntentCommand {
            intent_id: oid(intent_n),
            effect_object_id: effect_id.clone(),
            descriptor: descriptor(),
            target: "https://payments.example/api/refunds".to_owned(),
            parameters: json!({"amount_minor": 5000, "currency": "EUR"}),
            idempotency_key: key.to_owned(),
            expected_state_version: Version::INITIAL,
            grant_epoch: 41,
            capability_set_version: 7,
            actor_ref: uri("actor://tenant-a/agent-1"),
            authority_ref: uri("authority://tenant-a/effect-authority"),
            correlation_id: uri("corr://tenant-a/m5-verbs"),
            // Mechanical M5 cross-lane patch (Lane-KRN): new optional
            // field, None = the pre-M5 unbound path; behavior unchanged.
            task_binding: None,
        },
    )
    .unwrap();
    let committed = protocol(store, clock, ids)
        .authorize_effect(
            &effect_id,
            Version::INITIAL,
            &grant_for("payments.refund"),
            &currency(),
            &WriterLease { epoch: 1 },
        )
        .unwrap();
    (effect_id, committed.after_version)
}

fn seed_stuck_executing_effect(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    effect_n: u64,
    intent_n: u64,
    key: &str,
    executor: &ScriptedExecutor,
    script: ScriptedOutcome,
) -> ObjectId {
    let (effect_id, version) = seed_authorized_effect(store, clock, ids, effect_n, intent_n, key);
    executor.script(&[script]);
    protocol(store, clock, ids)
        .dispatch_effect(
            &effect_id,
            version,
            &grant_for("payments.refund"),
            &currency(),
            executor,
            &WriterLease { epoch: 1 },
        )
        .unwrap();
    // Crash before record_outcome: durable state stays EXECUTING.
    effect_id
}

// ---------------------------------------------------------------------
// Session and model fixtures
// ---------------------------------------------------------------------

const ALL_ACTIONS: [&str; 4] = [
    "status.inspect",
    "execution.stop",
    "capability.revoke",
    "effect.reconcile",
];

fn session_value(state: &str, expires: &str, actions: &[&str]) -> Value {
    json!({
        "schema_version": "cognitiveos.privileged-management-session/0.1",
        "session_id": "pms_m5-fallback-lib-01",
        "object_version": 1,
        "management_domain": "cognitiveos.management",
        "session_authority": "authority://tenant-a/management-authority",
        "human_principal": "principal://tenant-a/operator-1",
        "actor_chain_digest": format!("sha256:{}", "ab12".repeat(16)),
        "authentication_context_ref": "authn://tenant-a/webauthn-9",
        "activity_context_ref": "activity://tenant-a/m5-fallback",
        "scope": {
            "domains": [
                "cognitiveos.management.status",
                "cognitiveos.management.execution",
                "cognitiveos.management.capability",
                "cognitiveos.management.effect"
            ],
            "actions": actions,
            "resources": [
                "agent-execution://",
                "effect://",
                "task://",
                "loop://",
                "verification://",
                "governance://"
            ]
        },
        "risk_ceiling": "R1",
        "policy_version": 1,
        "revocation_epoch": 41,
        "issued_at": "2026-07-20T12:00:00Z",
        "last_activity_at": "2026-07-20T12:00:00Z",
        "idle_timeout_seconds": 3600,
        "absolute_expires_at": expires,
        "state": state,
        "session_digest": format!("sha256:{}", "cd34".repeat(16)),
        "authority_signature": "sig-m5-lib-fixture-0001"
    })
}

fn active_session() -> PrivilegedManagementSession {
    PrivilegedManagementSession::from_json_value(&session_value(
        "active",
        "2030-01-01T00:00:00Z",
        &ALL_ACTIONS,
    ))
    .unwrap()
}

/// A model provider double that counts every call: the deterministic
/// management plane must NEVER consult it, even when the experimental
/// shell slot is wired (REQ-MGMT-FALLBACK-001 zero-model-call proof).
struct CountingModel(AtomicU64);

impl CountingModel {
    fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    fn calls(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

impl ModelProvider for CountingModel {
    fn complete(&self, _prompt: &str) -> Result<String, PortFailure> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(PortFailure {
            detail: "model must never be consulted on deterministic paths".to_owned(),
        })
    }
}

fn denial_code(error: &ManagementError) -> &'static str {
    match error {
        ManagementError::Denied(denial) => denial.code_str(),
        other => panic!("expected management denial, got {other:?}"),
    }
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

/// The plane is constructible with NO model anywhere (the vector's
/// `model_available: false` environment), all four verbs run end to end,
/// and a wired-but-forbidden experimental model slot records ZERO calls.
#[test]
fn all_four_verbs_run_with_zero_model_calls() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, _) = seed_runnable_execution(&store, &clock, &ids, 0x3001);
    let executor = ScriptedExecutor::queryable(1);
    let effect_id = seed_stuck_executing_effect(
        &store,
        &clock,
        &ids,
        0x3101,
        0x3201,
        "zero-model-key-1",
        &executor,
        ScriptedOutcome::VanishWithoutExecution,
    );

    let probe = CountingModel::new();
    // The experimental Intelligent Management Shell slot exists but the
    // deterministic verbs must never touch it.
    let plane =
        ManagementPlane::deterministic(&store, &clock, &ids).with_experimental_shell_model(&probe);
    let session = active_session();

    // inspect
    let inspect_audit = RecordingAuditPort::new(AuditMode::Commit);
    let inspected = plane
        .inspect_with_audit(
            &session,
            &InspectRequest {
                domain: LifecycleDomain::AgentExecution,
                object_id: execution_id.clone(),
            },
            &inspect_audit,
        )
        .unwrap();
    assert_eq!(inspected.state, "RUNNABLE");

    // reconcile (the executor answers queries with the original key; the
    // sink never executed, so the effect closes as NOT_EXECUTED)
    executor.trust_epoch(2);
    let report = plane.reconcile(&session, &executor).unwrap();
    assert_eq!(report.step_order, RECOVERY_ORDER.to_vec());

    // stop (all effects are now terminally closed)
    let stopped = plane
        .stop(
            &session,
            &StopRequest {
                execution_id: execution_id.clone(),
            },
        )
        .unwrap();
    assert_eq!(stopped.to_state, "TERMINATED");

    // revoke
    let ledger_path = dir.path().join("governance.json");
    let mut ledger = GovernanceLedger::create(&ledger_path, 41, 7).unwrap();
    let revoked = plane.revoke(&session, &mut ledger).unwrap();
    assert_eq!(revoked.revocation_epoch, 42);

    // Authority facts, reloaded.
    let effect = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(effect.state.as_str(), "NOT_EXECUTED");
    let execution = store
        .load_object(LifecycleDomain::AgentExecution, &execution_id)
        .unwrap()
        .unwrap();
    assert_eq!(execution.state.as_str(), "TERMINATED");

    // The whole management pass consulted the model exactly zero times.
    assert_eq!(probe.calls(), 0, "deterministic paths must not call models");
}

/// MGMT-SESSION-DENY-002 semantics: expired and revoked sessions are
/// denied with the registered codes, pending effects are retained, no new
/// effects or events are created, and reconciliation later reuses the
/// original idempotency and effect records.
#[test]
fn expired_and_revoked_sessions_deny_without_losing_pending_effects() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, _) = seed_runnable_execution(&store, &clock, &ids, 0x3002);
    let executor = ScriptedExecutor::queryable(1);
    let effect_id = seed_stuck_executing_effect(
        &store,
        &clock,
        &ids,
        0x3102,
        0x3202,
        "session-deny-key-1",
        &executor,
        ScriptedOutcome::ExecuteThenTimeout,
    );
    let baseline_events = store.read_events(0, 100_000).unwrap().len();
    let baseline_dispatches = executor.dispatches().len();

    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    let ledger_path = dir.path().join("governance.json");
    let mut ledger = GovernanceLedger::create(&ledger_path, 41, 7).unwrap();

    let cases = [
        (
            PrivilegedManagementSession::from_json_value(&session_value(
                "expired",
                "2030-01-01T00:00:00Z",
                &ALL_ACTIONS,
            ))
            .unwrap(),
            "MANAGEMENT_SESSION_EXPIRED",
        ),
        (
            // active state but past absolute expiry: the deterministic
            // time derivation fails closed with the same registered code.
            PrivilegedManagementSession::from_json_value(&session_value(
                "active",
                "2020-01-01T00:00:00Z",
                &ALL_ACTIONS,
            ))
            .unwrap(),
            "MANAGEMENT_SESSION_EXPIRED",
        ),
        (
            PrivilegedManagementSession::from_json_value(&session_value(
                "revoked",
                "2030-01-01T00:00:00Z",
                &ALL_ACTIONS,
            ))
            .unwrap(),
            "MANAGEMENT_SESSION_REVOKED",
        ),
    ];
    for (session, expected) in &cases {
        let inspect_audit = RecordingAuditPort::new(AuditMode::Commit);
        let inspect = plane
            .inspect_with_audit(
                session,
                &InspectRequest {
                    domain: LifecycleDomain::AgentExecution,
                    object_id: execution_id.clone(),
                },
                &inspect_audit,
            )
            .unwrap_err();
        let inspect = expect_management_error(inspect);
        assert_eq!(denial_code(&inspect), *expected);
        let stop = plane
            .stop(
                session,
                &StopRequest {
                    execution_id: execution_id.clone(),
                },
            )
            .unwrap_err();
        assert_eq!(denial_code(&stop), *expected);
        let revoke = plane.revoke(session, &mut ledger).unwrap_err();
        assert_eq!(denial_code(&revoke), *expected);
        let reconcile = plane.reconcile(session, &executor).unwrap_err();
        assert_eq!(denial_code(&reconcile), *expected);
    }

    // Nothing was created, dispatched, committed or deleted: the pending
    // effect and its intent are exactly where they were.
    assert_eq!(
        store.read_events(0, 100_000).unwrap().len(),
        baseline_events,
        "new_effects_created: false / no events appended"
    );
    assert_eq!(
        executor.dispatches().len(),
        baseline_dispatches,
        "dispatches: 0"
    );
    let effect = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        effect.state.as_str(),
        "EXECUTING",
        "pending effect retained"
    );
    let intent = store
        .load_intent_for_effect(&effect_id)
        .unwrap()
        .expect("intent record retained");
    assert_eq!(intent.idempotency_key, "session-deny-key-1");
    assert_eq!(store.current_fencing_epoch().unwrap(), 1);

    // Recovery under a NEW valid session reconciles with the ORIGINAL
    // idempotency key and effect record (resume_requires_new_session +
    // reconciliation_uses_original_idempotency_and_effect_records).
    executor.trust_epoch(2);
    let session = active_session();
    let report = plane.reconcile(&session, &executor).unwrap();
    assert_eq!(report.new_epoch, 2);
    assert_eq!(executor.queries(), vec!["session-deny-key-1".to_owned()]);
    let effect = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        effect.state.as_str(),
        "RECONCILED",
        "executed-with-original-key reconciliation"
    );
}

/// MGMT-GATE-DENY-003 semantics: scope mismatch denies, unsatisfied
/// step-up challenges — both fail closed with dispatches=0.
#[test]
fn scope_mismatch_and_step_up_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, version) = seed_runnable_execution(&store, &clock, &ids, 0x3003);
    let baseline_events = store.read_events(0, 100_000).unwrap().len();

    let plane = ManagementPlane::deterministic(&store, &clock, &ids);

    // resource_outside_scope shape: the session does not carry the verb's
    // action.
    let inspect_only = PrivilegedManagementSession::from_json_value(&session_value(
        "active",
        "2030-01-01T00:00:00Z",
        &["status.inspect"],
    ))
    .unwrap();
    let denied = plane
        .stop(
            &inspect_only,
            &StopRequest {
                execution_id: execution_id.clone(),
            },
        )
        .unwrap_err();
    assert_eq!(denial_code(&denied), "MANAGEMENT_SCOPE_MISMATCH");

    // step_up_unsatisfied shape: the gate primitive challenges with the
    // registered code instead of granting.
    let session = active_session();
    let challenged = plane
        .gate_with_step_up(&session, "execution.stop", true, false)
        .unwrap_err();
    assert_eq!(denial_code(&challenged), "MANAGEMENT_STEP_UP_REQUIRED");
    let satisfied = plane.gate_with_step_up(&session, "execution.stop", true, true);
    assert!(satisfied.is_ok(), "satisfied step-up passes the gate");

    // fail_closed: nothing moved, nothing dispatched.
    assert_eq!(
        store.read_events(0, 100_000).unwrap().len(),
        baseline_events
    );
    let reloaded = store
        .load_object(LifecycleDomain::AgentExecution, &execution_id)
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.state.as_str(), "RUNNABLE");
    assert_eq!(reloaded.version, version);
}

/// Revoke reuses the M3 revocation arithmetic: the ledger epoch advances
/// durably, `revalidate_grant` fails for grants decided under the old
/// epoch, and the central gate blocks their dispatch (F-007).
#[test]
fn revoke_makes_stale_grants_fail_the_m3_revalidation() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (effect_id, effect_version) =
        seed_authorized_effect(&store, &clock, &ids, 0x3004, 0x3204, "revoke-lib-key-1");

    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    let session = active_session();
    let ledger_path = dir.path().join("governance.json");
    let mut ledger = GovernanceLedger::create(&ledger_path, 41, 7).unwrap();
    let grant = grant_for("payments.refund");

    // Before the revocation the grant revalidates cleanly.
    assert!(
        revalidate_grant(&grant, ledger.currency().revocation_epoch, 7, &clock.0).is_ok(),
        "grant is current before revocation"
    );

    let report = plane.revoke(&session, &mut ledger).unwrap();
    assert_eq!(report.previous_revocation_epoch, 41);
    assert_eq!(report.revocation_epoch, 42);

    // Durable fact: reload the ledger from disk.
    let reloaded = GovernanceLedger::load(&ledger_path).unwrap();
    assert_eq!(reloaded.currency().revocation_epoch, 42);
    assert_eq!(reloaded.currency().capability_set_version, 7);

    // The M3 revalidation now fails for the stale grant...
    assert!(
        revalidate_grant(&grant, reloaded.currency().revocation_epoch, 7, &clock.0).is_err(),
        "authorization check fails after revocation"
    );

    // ...and the central gate refuses to dispatch under it.
    let executor = ScriptedExecutor::queryable(1);
    let refusal = protocol(&store, &clock, &ids)
        .dispatch_effect(
            &effect_id,
            effect_version,
            &grant,
            &reloaded.currency(),
            &executor,
            &WriterLease { epoch: 1 },
        )
        .expect_err("stale grant must not dispatch");
    match refusal {
        cognitive_kernel::effects::EffectError::Rejected(rejection) => {
            assert_eq!(rejection.registered().code, "STATE_CONFLICT");
            assert!(
                rejection
                    .detail
                    .contains("capability_and_revocation_current")
            );
        }
        other => panic!("expected gate rejection, got {other:?}"),
    }
    assert!(executor.dispatches().is_empty());
    let effect = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(effect.state.as_str(), "AUTHORIZED", "no state change");
}

/// Reconcile converges every in-flight shape through the M4 path: an
/// executed-but-unreported effect reconciles to RECONCILED(executed), a
/// vanished dispatch closes NOT_EXECUTED, an authorized-undispatched
/// intent is confirmed ready for single re-dispatch with its ORIGINAL key,
/// and recovery re-dispatches NOTHING.
#[test]
fn reconcile_converges_every_in_flight_shape_with_original_keys() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let executor = ScriptedExecutor::queryable(1);
    // (a) dispatched, executed externally, receipt lost.
    let executed_id = seed_stuck_executing_effect(
        &store,
        &clock,
        &ids,
        0x3105,
        0x3205,
        "reconcile-executed-key",
        &executor,
        ScriptedOutcome::ExecuteThenTimeout,
    );
    // (b) dispatched, vanished without execution.
    let vanished_id = seed_stuck_executing_effect(
        &store,
        &clock,
        &ids,
        0x3106,
        0x3206,
        "reconcile-vanished-key",
        &executor,
        ScriptedOutcome::VanishWithoutExecution,
    );
    // (c) authorized, never dispatched (crash point 1).
    let (authorized_id, _) = seed_authorized_effect(
        &store,
        &clock,
        &ids,
        0x3107,
        0x3207,
        "reconcile-authorized-key",
    );
    let dispatches_before = executor.dispatches().len();

    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    let session = active_session();
    executor.trust_epoch(2);
    let report = plane.reconcile(&session, &executor).unwrap();

    assert_eq!(report.step_order, RECOVERY_ORDER.to_vec());
    assert_eq!(report.fenced_epoch, 1);
    assert_eq!(report.new_epoch, 2);
    assert_eq!(report.reconciled.len(), 3);

    // Authority facts, reloaded per effect.
    let executed = store
        .load_object(LifecycleDomain::Effect, &executed_id)
        .unwrap()
        .unwrap();
    assert_eq!(executed.state.as_str(), "RECONCILED");
    let vanished = store
        .load_object(LifecycleDomain::Effect, &vanished_id)
        .unwrap()
        .unwrap();
    assert_eq!(vanished.state.as_str(), "NOT_EXECUTED");
    let authorized = store
        .load_object(LifecycleDomain::Effect, &authorized_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        authorized.state.as_str(),
        "AUTHORIZED",
        "undispatched intent waits for governed single re-dispatch"
    );
    let ready = report
        .reconciled
        .iter()
        .find(|entry| entry.effect_id == authorized_id.as_str())
        .expect("authorized effect reported");
    assert_eq!(ready.disposition, "ready_to_redispatch_original_key");
    assert_eq!(
        ready.idempotency_key.as_deref(),
        Some("reconcile-authorized-key")
    );

    // Reconciliation queried with the ORIGINAL keys and dispatched nothing.
    let queries = executor.queries();
    assert!(queries.contains(&"reconcile-executed-key".to_owned()));
    assert!(queries.contains(&"reconcile-vanished-key".to_owned()));
    assert_eq!(
        executor.dispatches().len(),
        dispatches_before,
        "recovery never re-dispatches"
    );
}

/// Inspect is a pure read of authority state: correct fields, zero writes.
#[test]
fn inspect_reports_authority_state_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, version) = seed_runnable_execution(&store, &clock, &ids, 0x3007);
    let baseline_events = store.read_events(0, 100_000).unwrap().len();

    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    let session = active_session();
    let audit = RecordingAuditPort::new(AuditMode::Commit);
    let report = plane
        .inspect_with_audit(
            &session,
            &InspectRequest {
                domain: LifecycleDomain::AgentExecution,
                object_id: execution_id.clone(),
            },
            &audit,
        )
        .unwrap();
    assert_eq!(report.domain, "agent-execution");
    assert_eq!(report.object_id, execution_id.as_str());
    assert_eq!(report.state, "RUNNABLE");
    assert_eq!(report.version, version.get());
    assert_eq!(report.event_count, 3);
    assert_eq!(report.fencing_epoch, 1);
    assert_eq!(
        store.read_events(0, 100_000).unwrap().len(),
        baseline_events,
        "inspect wrote nothing"
    );

    // Deny/not-found isomorphism (M3 protected-read discipline): a
    // missing object surfaces the same registered denial as an
    // unauthorized one.
    let missing = plane
        .inspect_with_audit(
            &session,
            &InspectRequest {
                domain: LifecycleDomain::AgentExecution,
                object_id: oid(0xdead),
            },
            &audit,
        )
        .unwrap_err();
    let missing = expect_management_error(missing);
    assert_eq!(denial_code(&missing), "CONTEXT_AUTH_DENIED");
}

fn expect_management_error(error: AuditedInspectError) -> ManagementError {
    match error {
        AuditedInspectError::Management(error) => error,
        AuditedInspectError::Audit(error) => panic!("unexpected audit failure: {error}"),
    }
}

#[derive(Clone, Copy)]
enum AuditMode {
    Commit,
    Fail,
    Mismatch,
}

struct RecordingAuditPort {
    mode: AuditMode,
    committed: RefCell<Vec<OrdinaryCorePrivilegedReadDecision>>,
}

impl RecordingAuditPort {
    fn new(mode: AuditMode) -> Self {
        Self {
            mode,
            committed: RefCell::new(Vec::new()),
        }
    }
}

impl ManagementAuditPort for RecordingAuditPort {
    fn commit_privileged_read_decision(
        &self,
        record: &OrdinaryCorePrivilegedReadDecision,
        record_digest: &str,
    ) -> Result<OrdinaryCoreAuditCommitReceipt, AuditPortFailure> {
        if matches!(self.mode, AuditMode::Fail) {
            return Err(AuditPortFailure::new("injected audit persistence failure"));
        }
        self.committed.borrow_mut().push(record.clone());
        Ok(OrdinaryCoreAuditCommitReceipt {
            committed_at: "2026-07-20T12:00:00Z".to_owned(),
            record_id: record.record_id.clone(),
            record_digest: if matches!(self.mode, AuditMode::Mismatch) {
                "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_owned()
            } else {
                record_digest.to_owned()
            },
            request_digest: record.request_digest.clone(),
            sequence: 1,
            writer_epoch: 1,
        })
    }
}

#[test]
fn ordinary_core_inspect_releases_only_after_matching_audit_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, _) = seed_runnable_execution(&store, &clock, &ids, 0x3301);
    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    let audit = RecordingAuditPort::new(AuditMode::Commit);

    let report = plane
        .inspect_with_audit(
            &active_session(),
            &InspectRequest {
                domain: LifecycleDomain::AgentExecution,
                object_id: execution_id,
            },
            &audit,
        )
        .unwrap();

    assert_eq!(report.state, "RUNNABLE");
    let committed = audit.committed.borrow();
    assert_eq!(committed.len(), 1);
    assert_eq!(
        committed[0].record_kind,
        OrdinaryCorePrivilegedReadDecisionRecordKind::PrivilegedReadDecision
    );
    assert_eq!(
        committed[0].outcome,
        OrdinaryCorePrivilegedReadDecisionOutcome::Success
    );
    assert!(committed[0].result_digest.is_some());
    assert!(committed[0].safe_reason.is_none());
    let serialized = serde_json::to_value(&committed[0]).unwrap();
    let decoded: OrdinaryCorePrivilegedReadDecision =
        serde_json::from_value(serialized.clone()).unwrap();
    assert_eq!(decoded, committed[0]);
    assert_eq!(serialized["record_kind"], "privileged_read_decision");
    assert_eq!(serialized["outcome"], "success");
}

#[test]
fn ordinary_core_inspect_withholds_result_on_audit_failure_or_receipt_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, _) = seed_runnable_execution(&store, &clock, &ids, 0x3302);
    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    let request = InspectRequest {
        domain: LifecycleDomain::AgentExecution,
        object_id: execution_id,
    };

    for mode in [AuditMode::Fail, AuditMode::Mismatch] {
        let audit = RecordingAuditPort::new(mode);
        let err = plane
            .inspect_with_audit(&active_session(), &request, &audit)
            .unwrap_err();
        assert!(matches!(err, AuditedInspectError::Audit(_)));
    }
}

#[test]
fn ordinary_core_missing_object_is_audited_without_existence_facts() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    let audit = RecordingAuditPort::new(AuditMode::Commit);

    let err = plane
        .inspect_with_audit(
            &active_session(),
            &InspectRequest {
                domain: LifecycleDomain::AgentExecution,
                object_id: oid(0xdead),
            },
            &audit,
        )
        .unwrap_err();

    match err {
        AuditedInspectError::Management(denial) => {
            assert_eq!(denial_code(&denial), "CONTEXT_AUTH_DENIED")
        }
        other => panic!("expected protected-read denial, got {other:?}"),
    }
    let committed = audit.committed.borrow();
    assert_eq!(committed.len(), 1);
    assert_eq!(
        committed[0].outcome,
        OrdinaryCorePrivilegedReadDecisionOutcome::Denied
    );
    assert_eq!(
        committed[0].safe_reason,
        Some(OrdinaryCorePrivilegedReadDecisionSafeReason::ContextAuthDenied)
    );
    assert!(committed[0].result_digest.is_none());
}
