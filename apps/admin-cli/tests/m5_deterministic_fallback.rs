//! M5 batch 1 end-to-end: the deterministic admin CLI keeps the four
//! fallback verbs — inspect / stop / revoke / reconcile — available in a
//! model-disconnected environment (REQ-MGMT-FALLBACK-001; semantics of
//! vector `management-deterministic-fallback.json` / MGMT-FALLBACK-008,
//! which itself stays `not-run` for Lane-CFR). Session-gate denials follow
//! MGMT-SESSION-DENY-002 / MGMT-GATE-DENY-003 semantics
//! (REQ-MGMT-SESSION-002/003, REQ-MGMT-GATE-001,
//! REQ-MGMT-SESSION-LIFECYCLE-001).
//!
//! The suite spawns the real `admin-cli` binary against a real SQLite WAL
//! authority store, then REOPENS the store and asserts authority facts
//! from durable state — never from CLI echo. No model provider is
//! configured, reachable or even linkable: the workspace has no model SDK
//! dependency and the CLI is spawned with model-ish environment variables
//! removed. The CLI is the deterministic emergency path.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_arguments
)]

use cognitive_domain::capability::{CapabilityConstraints, LeaseWindow};
use cognitive_domain::{
    LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef, Version, WallTimestamp, table,
};
use cognitive_kernel::authz::{
    AccessRequest, ActorChainFacts, AuthorizationGrant, AuthzSnapshot, MembershipFacts,
    ObjectGovernance, PrincipalFacts, authorize,
};
use cognitive_kernel::effects::{
    EffectClass, EffectProtocol, GovernanceCurrency, IntentCommand, OperationDescriptor,
    WriterLease, mint_intent,
};
use cognitive_kernel::executor::ExecutorCapabilities;
use cognitive_kernel::ports::{AuthorityStore, Clock, IdGenerator, PortFailure, ProtocolStore};
use cognitive_kernel::{
    AdmitCommand, Causation, Reason, TablePin, TransitionCommand, TransitionEngine,
};
use cognitive_store::SqliteAuthorityStore;
use cognitive_store::faults::{ScriptedExecutor, ScriptedOutcome};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------
// Deterministic fixtures (same shapes as crates/cognitive-store/tests/
// m4_common; duplicated here because Cargo integration suites cannot
// share test modules across crates).
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
            correlation_id: uri("corr://tenant-a/m5-fallback"),
            outbox_destinations: vec![],
            fencing_epoch: None,
        })
        .unwrap();
}

/// Drive one legal transition with the edge's guards attested as fixture
/// facts (test seeding only; the CLI under test must NOT get this shortcut).
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
    let committed = engine
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
                causation_id: uri("corr://tenant-a/m5-fallback"),
                correlation_id: uri("corr://tenant-a/m5-fallback"),
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
        .unwrap();
    committed.after_version
}

/// Seed an agent-execution at RUNNABLE (CREATED -> ADMITTED -> RUNNABLE
/// through the real gate). Returns (execution_id, version).
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
        uri("corr://tenant-a/m5-fallback"),
    )
}

/// Seed an effect that reached AUTHORIZED with a durable intent (crash
/// point 1 shape: minted + authorized, never dispatched).
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
            correlation_id: uri("corr://tenant-a/m5-fallback"),
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

/// Seed an effect stuck in EXECUTING (crash point 2 shape: dispatched, the
/// outcome was never recorded — the pre-crash writer died first).
fn seed_stuck_executing_effect(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    effect_n: u64,
    intent_n: u64,
    key: &str,
    script: ScriptedOutcome,
) -> ObjectId {
    let (effect_id, version) = seed_authorized_effect(store, clock, ids, effect_n, intent_n, key);
    let executor = ScriptedExecutor::queryable(1);
    executor.script(&[script]);
    let (_committed, _outcome) = protocol(store, clock, ids)
        .dispatch_effect(
            &effect_id,
            version,
            &grant_for("payments.refund"),
            &currency(),
            &executor,
            &WriterLease { epoch: 1 },
        )
        .unwrap();
    // Crash before record_outcome: the durable state stays EXECUTING.
    effect_id
}

// ---------------------------------------------------------------------
// Session fixtures (shape source: specs/schemas/
// privileged-management-session.schema.json)
// ---------------------------------------------------------------------

fn session_value(state: &str, expires: &str, actions: &[&str]) -> Value {
    json!({
        "schema_version": "cognitiveos.privileged-management-session/0.1",
        "session_id": "pms_m5-fallback-e2e-01",
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
        "authority_signature": "sig-m5-e2e-fixture-0001"
    })
}

const ALL_ACTIONS: [&str; 4] = [
    "status.inspect",
    "execution.stop",
    "capability.revoke",
    "effect.reconcile",
];

fn write_session(dir: &Path, name: &str, value: &Value) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    path
}

fn write_ledger(
    dir: &Path,
    name: &str,
    revocation_epoch: i64,
    capability_set_version: i64,
) -> PathBuf {
    let path = dir.join(name);
    let value = json!({
        "capability_set_version": capability_set_version,
        "revocation_epoch": revocation_epoch,
        "updated_at": "2026-07-20T12:00:00Z"
    });
    std::fs::write(&path, serde_json::to_string(&value).unwrap()).unwrap();
    path
}

// ---------------------------------------------------------------------
// CLI runner
// ---------------------------------------------------------------------

struct CliResult {
    code: i32,
    stdout: String,
    stderr: String,
}

/// Spawn the real binary with a model-disconnected environment: no model
/// provider is configured and common provider variables are removed.
fn run_cli(args: &[&str]) -> CliResult {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_admin-cli"))
        .args(args)
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("MODEL_PROVIDER_URL")
        .output()
        .expect("admin-cli binary runs");
    CliResult {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn stdout_json(result: &CliResult) -> Value {
    serde_json::from_str(result.stdout.trim()).unwrap_or_else(|err| {
        panic!(
            "expected JSON on stdout, got error {err}\nstdout: {:?}\nstderr: {:?}",
            result.stdout, result.stderr
        )
    })
}

fn stderr_error_code(result: &CliResult) -> String {
    let value: Value = serde_json::from_str(result.stderr.trim()).unwrap_or_else(|err| {
        panic!(
            "expected registered-error JSON on stderr, got error {err}\nstdout: {:?}\nstderr: {:?}",
            result.stdout, result.stderr
        )
    });
    value["error"]["code"]
        .as_str()
        .expect("error.code present")
        .to_owned()
}

fn event_count(store: &SqliteAuthorityStore) -> usize {
    store.read_events(0, 100_000).unwrap().len()
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

/// Verb 1 — inspect reads authority state (state/version/events) from the
/// durable store, is deterministic JSON, and performs zero writes.
#[test]
fn inspect_reads_authority_state_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("authority.db");
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, version) = {
        let store = SqliteAuthorityStore::open(&db).unwrap();
        let seeded = seed_runnable_execution(&store, &clock, &ids, 0x2001);
        drop(store);
        seeded
    };
    let session = write_session(
        dir.path(),
        "session.json",
        &session_value("active", "2030-01-01T00:00:00Z", &ALL_ACTIONS),
    );

    let result = run_cli(&[
        "inspect",
        "--store",
        db.to_str().unwrap(),
        "--session",
        session.to_str().unwrap(),
        "--domain",
        "agent-execution",
        "--object",
        execution_id.as_str(),
    ]);
    assert_eq!(result.code, 0, "stderr: {}", result.stderr);
    let report = stdout_json(&result);
    assert_eq!(report["domain"], "agent-execution");
    assert_eq!(report["object_id"], execution_id.as_str());
    assert_eq!(report["state"], "RUNNABLE");
    assert_eq!(report["version"], version.get());
    assert_eq!(report["event_count"], 3, "admit + 2 transitions");
    assert_eq!(report["fencing_epoch"], 1);

    // Read-only proof from the durable store, not from CLI echo.
    let store = SqliteAuthorityStore::open(&db).unwrap();
    assert_eq!(event_count(&store), 3, "inspect appended nothing");
    let reloaded = store
        .load_object(LifecycleDomain::AgentExecution, &execution_id)
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.state.as_str(), "RUNNABLE");
    assert_eq!(reloaded.version, version);
}

/// Verb 2 — stop terminates a RUNNABLE execution through the central
/// transition gate (legal edge, TERMINATION_REQUESTED) and refuses an
/// illegal target (already TERMINATED) with a registered code.
#[test]
fn stop_terminates_through_the_gate_and_rejects_illegal_targets() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("authority.db");
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, version) = {
        let store = SqliteAuthorityStore::open(&db).unwrap();
        let seeded = seed_runnable_execution(&store, &clock, &ids, 0x2002);
        drop(store);
        seeded
    };
    let session = write_session(
        dir.path(),
        "session.json",
        &session_value("active", "2030-01-01T00:00:00Z", &ALL_ACTIONS),
    );

    let result = run_cli(&[
        "stop",
        "--store",
        db.to_str().unwrap(),
        "--session",
        session.to_str().unwrap(),
        "--execution",
        execution_id.as_str(),
    ]);
    assert_eq!(result.code, 0, "stderr: {}", result.stderr);
    let report = stdout_json(&result);
    assert_eq!(report["from_state"], "RUNNABLE");
    assert_eq!(report["to_state"], "TERMINATED");

    // Authority fact: reload from the durable store.
    let store = SqliteAuthorityStore::open(&db).unwrap();
    let reloaded = store
        .load_object(LifecycleDomain::AgentExecution, &execution_id)
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.state.as_str(), "TERMINATED");
    assert_eq!(reloaded.version.get(), version.get() + 1);
    // The stop event was appended to the log.
    assert_eq!(event_count(&store), 4);
    // The stopped writer generation is fenced: epoch advanced.
    assert!(store.current_fencing_epoch().unwrap() > 1);
    drop(store);

    // Illegal target: stopping a TERMINATED execution is refused by the
    // central gate with the registered conflict code; state unchanged.
    let denied = run_cli(&[
        "stop",
        "--store",
        db.to_str().unwrap(),
        "--session",
        session.to_str().unwrap(),
        "--execution",
        execution_id.as_str(),
    ]);
    assert_eq!(denied.code, 1, "stdout: {}", denied.stdout);
    assert_eq!(stderr_error_code(&denied), "STATE_CONFLICT");
    let store = SqliteAuthorityStore::open(&db).unwrap();
    let reloaded = store
        .load_object(LifecycleDomain::AgentExecution, &execution_id)
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.state.as_str(), "TERMINATED");
    assert_eq!(reloaded.version.get(), version.get() + 1, "no second write");
}

/// Stop fails closed while a pending (non-terminal) effect exists: the
/// `pending_effects_closed_or_quarantined` guard cannot be established, the
/// central gate rejects, and the execution stays RUNNABLE.
#[test]
fn stop_is_refused_while_effects_are_pending() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("authority.db");
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, version) = {
        let store = SqliteAuthorityStore::open(&db).unwrap();
        let seeded = seed_runnable_execution(&store, &clock, &ids, 0x2003);
        seed_authorized_effect(&store, &clock, &ids, 0x2103, 0x2203, "stop-pending-key-1");
        drop(store);
        seeded
    };
    let session = write_session(
        dir.path(),
        "session.json",
        &session_value("active", "2030-01-01T00:00:00Z", &ALL_ACTIONS),
    );

    let denied = run_cli(&[
        "stop",
        "--store",
        db.to_str().unwrap(),
        "--session",
        session.to_str().unwrap(),
        "--execution",
        execution_id.as_str(),
    ]);
    assert_eq!(denied.code, 1, "stdout: {}", denied.stdout);
    assert_eq!(stderr_error_code(&denied), "STATE_CONFLICT");
    assert!(
        denied
            .stderr
            .contains("pending_effects_closed_or_quarantined"),
        "denial names the missing guard: {}",
        denied.stderr
    );

    let store = SqliteAuthorityStore::open(&db).unwrap();
    let reloaded = store
        .load_object(LifecycleDomain::AgentExecution, &execution_id)
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.state.as_str(), "RUNNABLE", "stop did not land");
    assert_eq!(reloaded.version, version);
}

/// Verb 3 — revoke advances the revocation epoch in the governance ledger;
/// grants decided under the previous epoch stop authorizing dispatch
/// (F-007 revalidation), proven against the real gate and durable state.
#[test]
fn revoke_advances_the_epoch_and_stale_grants_stop_dispatching() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("authority.db");
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (effect_id, effect_version) = {
        let store = SqliteAuthorityStore::open(&db).unwrap();
        let seeded = seed_authorized_effect(&store, &clock, &ids, 0x2004, 0x2104, "revoke-key-1");
        drop(store);
        seeded
    };
    let session = write_session(
        dir.path(),
        "session.json",
        &session_value("active", "2030-01-01T00:00:00Z", &ALL_ACTIONS),
    );
    let ledger = write_ledger(dir.path(), "governance.json", 41, 7);

    let result = run_cli(&[
        "revoke",
        "--store",
        db.to_str().unwrap(),
        "--session",
        session.to_str().unwrap(),
        "--ledger",
        ledger.to_str().unwrap(),
    ]);
    assert_eq!(result.code, 0, "stderr: {}", result.stderr);
    let report = stdout_json(&result);
    assert_eq!(report["previous_revocation_epoch"], 41);
    assert_eq!(report["revocation_epoch"], 42);

    // Durable ledger fact (reload the file, not the CLI echo).
    let persisted: Value =
        serde_json::from_str(&std::fs::read_to_string(&ledger).unwrap()).unwrap();
    assert_eq!(persisted["revocation_epoch"], 42);
    assert_eq!(persisted["capability_set_version"], 7);

    // The grant decided under epoch 41 is stale under the revoked
    // currency: the sanctioned derivation yields false and the central
    // gate refuses dispatch; the effect stays AUTHORIZED, zero dispatches.
    let store = SqliteAuthorityStore::open(&db).unwrap();
    let stale_grant = grant_for("payments.refund");
    let revoked_currency = GovernanceCurrency {
        revocation_epoch: persisted["revocation_epoch"].as_i64().unwrap(),
        capability_set_version: persisted["capability_set_version"].as_i64().unwrap(),
    };
    let executor = ScriptedExecutor::queryable(1);
    let refusal = protocol(&store, &clock, &ids)
        .dispatch_effect(
            &effect_id,
            effect_version,
            &stale_grant,
            &revoked_currency,
            &executor,
            &WriterLease { epoch: 1 },
        )
        .expect_err("stale grant must not dispatch");
    let rejection = match refusal {
        cognitive_kernel::effects::EffectError::Rejected(rejection) => rejection,
        other => panic!("expected gate rejection, got {other:?}"),
    };
    assert_eq!(rejection.registered().code, "STATE_CONFLICT");
    assert!(
        rejection
            .detail
            .contains("capability_and_revocation_current"),
        "{}",
        rejection.detail
    );
    assert!(executor.dispatches().is_empty(), "zero external dispatches");
    let reloaded = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.state.as_str(), "AUTHORIZED", "state unchanged");
    assert_eq!(reloaded.version, effect_version);
}

/// Verb 4 — reconcile drives the M4 recovery path over the in-flight
/// effect: EXECUTING converges through OUTCOME_UNKNOWN -> RECONCILED to a
/// safe closure. With no external adapter configured the CLI cannot query
/// outcomes, so still-unknown quarantines (fail-safe, never blind-retry);
/// afterwards the converged store admits a deterministic stop.
#[test]
fn reconcile_converges_in_flight_effects_then_stop_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("authority.db");
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, effect_id) = {
        let store = SqliteAuthorityStore::open(&db).unwrap();
        let (execution_id, _) = seed_runnable_execution(&store, &clock, &ids, 0x2005);
        let effect_id = seed_stuck_executing_effect(
            &store,
            &clock,
            &ids,
            0x2105,
            0x2205,
            "reconcile-key-1",
            ScriptedOutcome::ExecuteThenTimeout,
        );
        drop(store);
        (execution_id, effect_id)
    };
    let session = write_session(
        dir.path(),
        "session.json",
        &session_value("active", "2030-01-01T00:00:00Z", &ALL_ACTIONS),
    );

    let result = run_cli(&[
        "reconcile",
        "--store",
        db.to_str().unwrap(),
        "--session",
        session.to_str().unwrap(),
    ]);
    assert_eq!(result.code, 0, "stderr: {}", result.stderr);
    let report = stdout_json(&result);
    assert_eq!(report["fenced_epoch"], 1);
    assert_eq!(report["new_epoch"], 2);
    let dispositions = report["reconciled"].as_array().unwrap();
    assert_eq!(dispositions.len(), 1);
    assert_eq!(dispositions[0]["effect_id"], effect_id.as_str());
    assert_eq!(dispositions[0]["disposition"], "quarantined");
    assert_eq!(dispositions[0]["error_code"], "EFFECT_RECOVERY_QUARANTINED");

    // Authority facts: the effect converged to the QUARANTINED terminal
    // state under the ORIGINAL idempotency key (no new intent, no blind
    // re-dispatch), and the old writer epoch is fenced.
    let store = SqliteAuthorityStore::open(&db).unwrap();
    let reloaded = store
        .load_object(LifecycleDomain::Effect, &effect_id)
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.state.as_str(), "QUARANTINED");
    let intent = store
        .load_intent_for_effect(&effect_id)
        .unwrap()
        .expect("original intent retained");
    assert_eq!(intent.idempotency_key, "reconcile-key-1");
    assert_eq!(store.current_fencing_epoch().unwrap(), 2);
    drop(store);

    // Cross-verb closure: with every effect terminally closed the same
    // model-disconnected CLI can stop the execution.
    let stop = run_cli(&[
        "stop",
        "--store",
        db.to_str().unwrap(),
        "--session",
        session.to_str().unwrap(),
        "--execution",
        execution_id.as_str(),
    ]);
    assert_eq!(stop.code, 0, "stderr: {}", stop.stderr);
    let store = SqliteAuthorityStore::open(&db).unwrap();
    let reloaded = store
        .load_object(LifecycleDomain::AgentExecution, &execution_id)
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.state.as_str(), "TERMINATED");
}

/// Session gate — expired and revoked sessions are denied with the
/// registered codes BEFORE any dispatch or write; pending effects are
/// retained untouched (MGMT-SESSION-DENY-002 semantics).
#[test]
fn expired_and_revoked_sessions_are_denied_before_any_write() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("authority.db");
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, execution_version) = {
        let store = SqliteAuthorityStore::open(&db).unwrap();
        let seeded = seed_runnable_execution(&store, &clock, &ids, 0x2006);
        seed_authorized_effect(&store, &clock, &ids, 0x2106, 0x2206, "deny-key-1");
        drop(store);
        seeded
    };
    let baseline_events = {
        let store = SqliteAuthorityStore::open(&db).unwrap();
        event_count(&store)
    };
    let ledger = write_ledger(dir.path(), "governance.json", 41, 7);

    // state=expired -> MANAGEMENT_SESSION_EXPIRED on every verb.
    let expired = write_session(
        dir.path(),
        "expired.json",
        &session_value("expired", "2030-01-01T00:00:00Z", &ALL_ACTIONS),
    );
    // state=active but past absolute expiry -> same registered denial
    // (fail-closed time derivation).
    let stale = write_session(
        dir.path(),
        "stale.json",
        &session_value("active", "2020-01-01T00:00:00Z", &ALL_ACTIONS),
    );
    // state=revoked -> MANAGEMENT_SESSION_REVOKED.
    let revoked = write_session(
        dir.path(),
        "revoked.json",
        &session_value("revoked", "2030-01-01T00:00:00Z", &ALL_ACTIONS),
    );

    for (session, expected_code) in [
        (&expired, "MANAGEMENT_SESSION_EXPIRED"),
        (&stale, "MANAGEMENT_SESSION_EXPIRED"),
        (&revoked, "MANAGEMENT_SESSION_REVOKED"),
    ] {
        for args in [
            vec![
                "inspect",
                "--store",
                db.to_str().unwrap(),
                "--session",
                session.to_str().unwrap(),
                "--domain",
                "agent-execution",
                "--object",
                execution_id.as_str(),
            ],
            vec![
                "stop",
                "--store",
                db.to_str().unwrap(),
                "--session",
                session.to_str().unwrap(),
                "--execution",
                execution_id.as_str(),
            ],
            vec![
                "revoke",
                "--store",
                db.to_str().unwrap(),
                "--session",
                session.to_str().unwrap(),
                "--ledger",
                ledger.to_str().unwrap(),
            ],
            vec![
                "reconcile",
                "--store",
                db.to_str().unwrap(),
                "--session",
                session.to_str().unwrap(),
            ],
        ] {
            let denied = run_cli(&args);
            assert_eq!(denied.code, 1, "verb {} stdout: {}", args[0], denied.stdout);
            assert_eq!(
                stderr_error_code(&denied),
                expected_code,
                "verb {}",
                args[0]
            );
        }
    }

    // Zero dispatches, zero new effects, pending effects retained: the
    // durable store and ledger are byte-for-byte where they started.
    let store = SqliteAuthorityStore::open(&db).unwrap();
    assert_eq!(event_count(&store), baseline_events, "no new events");
    let execution = store
        .load_object(LifecycleDomain::AgentExecution, &execution_id)
        .unwrap()
        .unwrap();
    assert_eq!(execution.state.as_str(), "RUNNABLE");
    assert_eq!(execution.version, execution_version);
    let effect = store
        .load_object(LifecycleDomain::Effect, &oid(0x2106))
        .unwrap()
        .unwrap();
    assert_eq!(
        effect.state.as_str(),
        "AUTHORIZED",
        "pending effect retained"
    );
    assert_eq!(store.current_fencing_epoch().unwrap(), 1, "no fencing done");
    let persisted: Value =
        serde_json::from_str(&std::fs::read_to_string(&ledger).unwrap()).unwrap();
    assert_eq!(persisted["revocation_epoch"], 41, "ledger untouched");
}

/// Session gate — an action outside the session scope fails closed with
/// MANAGEMENT_SCOPE_MISMATCH and dispatches nothing
/// (MGMT-GATE-DENY-003 semantics).
#[test]
fn out_of_scope_actions_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("authority.db");
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    let (execution_id, execution_version) = {
        let store = SqliteAuthorityStore::open(&db).unwrap();
        let seeded = seed_runnable_execution(&store, &clock, &ids, 0x2007);
        drop(store);
        seeded
    };
    // The session only carries status.inspect: every other verb is out of
    // scope.
    let session = write_session(
        dir.path(),
        "inspect-only.json",
        &session_value("active", "2030-01-01T00:00:00Z", &["status.inspect"]),
    );

    let denied = run_cli(&[
        "stop",
        "--store",
        db.to_str().unwrap(),
        "--session",
        session.to_str().unwrap(),
        "--execution",
        execution_id.as_str(),
    ]);
    assert_eq!(denied.code, 1, "stdout: {}", denied.stdout);
    assert_eq!(stderr_error_code(&denied), "MANAGEMENT_SCOPE_MISMATCH");

    let store = SqliteAuthorityStore::open(&db).unwrap();
    let reloaded = store
        .load_object(LifecycleDomain::AgentExecution, &execution_id)
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.state.as_str(), "RUNNABLE", "nothing dispatched");
    assert_eq!(reloaded.version, execution_version);

    // The in-scope verb still works with the same session: the denial was
    // the scope arithmetic, not the session.
    let allowed = run_cli(&[
        "inspect",
        "--store",
        db.to_str().unwrap(),
        "--session",
        session.to_str().unwrap(),
        "--domain",
        "agent-execution",
        "--object",
        execution_id.as_str(),
    ]);
    assert_eq!(allowed.code, 0, "stderr: {}", allowed.stderr);
}

/// A management session document that does not validate against the
/// registered schema shape is not a session at all: fail closed with the
/// registered auth denial, before any read or write.
#[test]
fn malformed_session_documents_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("authority.db");
    let clock = FixedClock::new();
    let ids = SeqIds::new();
    {
        let store = SqliteAuthorityStore::open(&db).unwrap();
        seed_runnable_execution(&store, &clock, &ids, 0x2008);
    }
    let mut broken = session_value("active", "2030-01-01T00:00:00Z", &ALL_ACTIONS);
    broken["session_id"] = json!("not-a-pms-id");
    let session = write_session(dir.path(), "broken.json", &broken);

    let denied = run_cli(&[
        "reconcile",
        "--store",
        db.to_str().unwrap(),
        "--session",
        session.to_str().unwrap(),
    ]);
    assert_eq!(denied.code, 1, "stdout: {}", denied.stdout);
    assert_eq!(stderr_error_code(&denied), "CONTEXT_AUTH_DENIED");

    let store = SqliteAuthorityStore::open(&db).unwrap();
    assert_eq!(store.current_fencing_epoch().unwrap(), 1, "no fencing done");
}
