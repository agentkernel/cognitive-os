//! Focused tests extracted from `tool_executor` (P9-T02/D03).

#![allow(clippy::expect_used, clippy::panic, unused_imports)]

use super::*;
use cognitive_domain::{
    EventId, LifecycleDomain, ObjectId, StateName, UriRef, Version, WallTimestamp,
};
use cognitive_kernel::authz::{
    AccessRequest, ActorChainFacts, AuthorizationGrant, AuthzSnapshot, MembershipFacts,
    ObjectGovernance, PrincipalFacts, authorize,
};
use cognitive_kernel::effects::{EffectError, EffectProtocol, GovernanceCurrency, WriterLease};
use cognitive_kernel::engine::CommittedTransition;
use cognitive_kernel::executor::{
    DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
};
use cognitive_kernel::ports::{
    AuthorityStore, Clock, EventDraft, IntentRow, ObjectAdmission, PortFailure, ProtocolStore,
    StoredObject,
};
use cognitive_kernel::tool_registry::{
    BUILTIN_TOOL_CATALOG, NativeOperationFamily, NativeToolDescriptor, ToolAvailability, ToolRisk,
};
use cognitive_store::{SqliteAuthorityStore, UuidV7Generator};
use serde_json::json;
use std::path::PathBuf;
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

struct TestWorkspace {
    path: PathBuf,
}

impl TestWorkspace {
    fn new(test_name: &str) -> Self {
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "cognitiveos-tool-executor-{test_name}-{}-{timestamp_nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temporary workspace");
        Self { path }
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn request_for(family: NativeOperationFamily) -> NativeToolExecutionRequest {
    let descriptor = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|descriptor| descriptor.family == family)
        .cloned()
        .unwrap_or_else(|| panic!("catalog family missing: {family:?}"));
    NativeToolExecutionRequest {
        descriptor,
        target: "workspace://notes/today.txt".to_owned(),
        input: b"bounded input".to_vec(),
        workspace_root: Some(PathBuf::from("/tmp/cognitiveos-workspace")),
    }
}

fn process_check_request() -> NativeToolExecutionRequest {
    let mut request = request_for(NativeOperationFamily::ProcessCheck);
    request.target = "process://4242".to_owned();
    request.input.clear();
    request.workspace_root = None;
    request
}

fn process_check_call(
    idempotency_key: &str,
    parameters_digest: &str,
    target: &str,
    fencing_epoch: i64,
) -> ExecutorCall {
    ExecutorCall {
        action: "check".to_owned(),
        target: target.to_owned(),
        idempotency_key: idempotency_key.to_owned(),
        parameters_digest: parameters_digest.to_owned(),
        authorization_digest: "authorization-digest".to_owned(),
        fencing_epoch,
    }
}

fn staged_process_executor(
    output_limit_bytes: usize,
) -> (
    Arc<BoundedProcessCheckSupervisor>,
    NativeProcessCheckExecutor<BoundedProcessCheckSupervisor>,
) {
    let supervisor = Arc::new(BoundedProcessCheckSupervisor::new(Duration::from_secs(2)));
    supervisor.register(
        4242,
        b"state=ready token=secret output-that-is-too-long",
        Duration::from_millis(1),
    );
    let mut request = process_check_request();
    request.descriptor.output_limit_bytes = output_limit_bytes;
    let validated_request = validate_native_tool_request(&request).expect("valid process check");
    let executor =
        NativeProcessCheckExecutor::new(7, Arc::clone(&supervisor), Duration::from_secs(1));
    executor
        .stage_request(
            "process-key".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        )
        .expect("stage process check");
    (supervisor, executor)
}

struct TestProcessObservationSource {
    output: Vec<u8>,
}

impl ProcessObservationSource for TestProcessObservationSource {
    fn observe(
        &self,
        _process_id: u32,
        _timeout: Duration,
    ) -> Result<Vec<u8>, ProcessCheckSupervisorError> {
        Ok(self.output.clone())
    }
}

fn daemon_supervisor() -> DaemonProcessSupervisor<TestProcessObservationSource> {
    DaemonProcessSupervisor::new(
        7,
        Duration::from_secs(2),
        8,
        Arc::new(TestProcessObservationSource {
            output: b"1234567890".to_vec(),
        }),
    )
}

#[test]
fn daemon_supervisor_registers_stable_attempt_identity() {
    let supervisor = daemon_supervisor();
    assert_eq!(supervisor.register("attempt-1".to_owned(), 4242, 7), Ok(()));
    assert_eq!(
        supervisor.register("attempt-1".to_owned(), 4243, 7),
        Err(ProcessCheckSupervisorError::Orphaned)
    );
    assert_eq!(
        supervisor.check_process(4242, Duration::from_millis(1)),
        Ok(b"12345678".to_vec())
    );
}

#[test]
fn daemon_supervisor_rejects_stale_epoch_and_orphan_until_recovered() {
    let supervisor = daemon_supervisor();
    supervisor
        .register("attempt-1".to_owned(), 4242, 7)
        .expect("register process");
    supervisor.fence(8).expect("advance fencing epoch");
    assert_eq!(
        supervisor.check_process(4242, Duration::from_millis(1)),
        Err(ProcessCheckSupervisorError::Orphaned)
    );
    supervisor
        .recover("attempt-1", 4242, 8)
        .expect("recover process at current epoch");
    assert_eq!(
        supervisor.check_process(4242, Duration::from_millis(1)),
        Ok(b"12345678".to_vec())
    );
}

#[test]
fn daemon_supervisor_rejects_orphans_and_shutdowns_fail_closed() {
    let supervisor = daemon_supervisor();
    supervisor
        .register("attempt-1".to_owned(), 4242, 7)
        .expect("register process");
    supervisor.unregister(4242).expect("unregister process");
    assert_eq!(
        supervisor.check_process(4242, Duration::from_millis(1)),
        Err(ProcessCheckSupervisorError::NotRegistered)
    );
    supervisor
        .register("attempt-2".to_owned(), 4243, 7)
        .expect("register second process");
    supervisor.shutdown().expect("shutdown supervisor");
    assert_eq!(
        supervisor.check_process(4243, Duration::from_millis(1)),
        Err(ProcessCheckSupervisorError::Orphaned)
    );
    assert_eq!(
        supervisor.register("attempt-3".to_owned(), 4244, 7),
        Err(ProcessCheckSupervisorError::Orphaned)
    );
}

#[test]
fn daemon_supervisor_bounds_timeout_before_observation() {
    let supervisor = daemon_supervisor();
    supervisor
        .register("attempt-1".to_owned(), 4242, 7)
        .expect("register process");
    assert_eq!(
        supervisor.check_process(4242, Duration::ZERO),
        Err(ProcessCheckSupervisorError::TimedOut)
    );
    assert_eq!(
        supervisor.check_process(4242, Duration::from_secs(3)),
        Err(ProcessCheckSupervisorError::TimedOut)
    );
}

#[test]
fn daemon_supervisor_default_source_fails_closed() {
    let supervisor = DaemonProcessSupervisor::fail_closed(7, Duration::from_secs(1), 32);
    supervisor
        .register("attempt-1".to_owned(), 4242, 7)
        .expect("register process");
    assert_eq!(
        supervisor.check_process(4242, Duration::from_millis(1)),
        Err(ProcessCheckSupervisorError::ObservationUnavailable)
    );
}

#[test]
fn process_check_validates_bounded_target_and_executes_registered_observation() {
    let request = process_check_request();
    let validated_request = validate_native_tool_request(&request).expect("valid process check");
    let supervisor = Arc::new(BoundedProcessCheckSupervisor::new(Duration::from_secs(2)));
    supervisor.register(4242, b"state=ready", Duration::from_millis(1));
    let executor = NativeProcessCheckExecutor::new(7, supervisor, Duration::from_secs(1));
    executor
        .stage_request(
            "process-key".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        )
        .expect("stage process check");

    assert!(matches!(
        executor.dispatch(&process_check_call(
            "process-key",
            "digest-1",
            "process://4242",
            7
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert_eq!(
        executor.completed_output("process-key"),
        Some(b"state=ready".to_vec())
    );
}

#[test]
fn process_check_rejects_invalid_and_unregistered_processes() {
    let mut request = process_check_request();
    request.target = "process://4242/child".to_owned();
    assert_eq!(
        validate_native_tool_request(&request),
        Err(NativeToolExecutionError::InvalidProcessTarget)
    );

    let validated_request =
        validate_native_tool_request(&process_check_request()).expect("valid process check");
    let supervisor = Arc::new(BoundedProcessCheckSupervisor::new(Duration::from_secs(2)));
    let executor = NativeProcessCheckExecutor::new(7, supervisor, Duration::from_secs(1));
    executor
        .stage_request(
            "missing-process".to_owned(),
            "digest-missing".to_owned(),
            &validated_request,
        )
        .expect("stage process check");
    let dispatch_result = executor.dispatch(&process_check_call(
        "missing-process",
        "digest-missing",
        "process://4242",
        7,
    ));
    assert!(dispatch_result.is_err());
    assert_eq!(
        executor.query_outcome("missing-process"),
        Ok(ExecutorQueryResult::NotExecuted)
    );
}

#[test]
fn process_check_missing_stage_fails_before_supervisor_access() {
    let supervisor = Arc::new(BoundedProcessCheckSupervisor::new(Duration::from_secs(2)));
    let executor =
        NativeProcessCheckExecutor::new(7, Arc::clone(&supervisor), Duration::from_secs(1));

    assert_eq!(
        executor.dispatch(&process_check_call(
            "unstaged-process",
            "digest-missing",
            "process://4242",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted {
            reason: "no daemon-staged process check for idempotency key".to_owned(),
        })
    );
    assert_eq!(supervisor.access_count(), 0);
    assert_eq!(
        executor.query_outcome("unstaged-process"),
        Ok(ExecutorQueryResult::NotExecuted)
    );
}

#[test]
fn process_check_timeout_and_orphan_fail_without_completed_outcome() {
    for (process_id, required_runtime, orphan_process) in [
        (4242, Duration::from_secs(2), false),
        (4243, Duration::from_millis(1), true),
    ] {
        let supervisor = Arc::new(BoundedProcessCheckSupervisor::new(Duration::from_secs(2)));
        supervisor.register(process_id, b"state=ready", required_runtime);
        if orphan_process {
            supervisor.orphan(process_id);
        }
        let mut request = process_check_request();
        request.target = format!("process://{process_id}");
        let validated_request =
            validate_native_tool_request(&request).expect("valid process check");
        let idempotency_key = format!("process-fault-{process_id}");
        let executor =
            NativeProcessCheckExecutor::new(7, Arc::clone(&supervisor), Duration::from_secs(1));
        executor
            .stage_request(
                idempotency_key.clone(),
                "digest-fault".to_owned(),
                &validated_request,
            )
            .expect("stage process check");

        assert!(
            executor
                .dispatch(&process_check_call(
                    &idempotency_key,
                    "digest-fault",
                    &request.target,
                    7,
                ))
                .is_err()
        );
        assert_eq!(supervisor.access_count(), 1);
        assert_eq!(executor.completed_output(&idempotency_key), None);
        assert_eq!(
            executor.query_outcome(&idempotency_key),
            Ok(ExecutorQueryResult::NotExecuted)
        );
    }
}

#[test]
fn process_check_redacts_and_bounds_output_before_queryable_storage() {
    let (_supervisor, executor) = staged_process_executor(20);
    let call = process_check_call("process-key", "digest-1", "process://4242", 7);
    let first_outcome = executor.dispatch(&call).expect("execute process check");
    let second_outcome = executor
        .dispatch(&call)
        .expect("absorb duplicate process check");
    assert_eq!(first_outcome, second_outcome);
    let output = executor
        .completed_output("process-key")
        .expect("stored output");
    assert!(output.len() <= 20);
    assert!(!String::from_utf8_lossy(&output).contains("secret"));
    assert_eq!(
        executor.query_outcome("process-key"),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
}

#[test]
fn process_check_fails_closed_for_timeout_and_orphaned_processes() {
    let supervisor = Arc::new(BoundedProcessCheckSupervisor::new(Duration::from_secs(2)));
    supervisor.register(4242, b"late", Duration::from_secs(3));
    let mut request = process_check_request();
    let validated_request = validate_native_tool_request(&request).expect("valid process check");
    let timeout_executor =
        NativeProcessCheckExecutor::new(7, Arc::clone(&supervisor), Duration::from_secs(1));
    timeout_executor
        .stage_request(
            "timeout".to_owned(),
            "digest-timeout".to_owned(),
            &validated_request,
        )
        .expect("stage timeout check");
    assert!(
        timeout_executor
            .dispatch(&process_check_call(
                "timeout",
                "digest-timeout",
                "process://4242",
                7
            ))
            .is_err()
    );
    assert_eq!(
        timeout_executor.query_outcome("timeout"),
        Ok(ExecutorQueryResult::NotExecuted)
    );

    supervisor.register(4242, b"orphan", Duration::from_millis(1));
    supervisor.orphan(4242);
    request.target = "process://4242".to_owned();
    let orphan_request = validate_native_tool_request(&request).expect("valid process check");
    let orphan_executor =
        NativeProcessCheckExecutor::new(7, Arc::clone(&supervisor), Duration::from_secs(1));
    orphan_executor
        .stage_request(
            "orphan".to_owned(),
            "digest-orphan".to_owned(),
            &orphan_request,
        )
        .expect("stage orphan check");
    assert!(
        orphan_executor
            .dispatch(&process_check_call(
                "orphan",
                "digest-orphan",
                "process://4242",
                7
            ))
            .is_err()
    );
    assert_eq!(
        orphan_executor.query_outcome("orphan"),
        Ok(ExecutorQueryResult::NotExecuted)
    );
}

#[test]
fn process_check_fences_stale_dispatch_before_supervisor_access() {
    let (supervisor, executor) = staged_process_executor(64);
    assert_eq!(
        executor.dispatch(&process_check_call(
            "process-key",
            "digest-1",
            "process://4242",
            6
        )),
        Ok(DispatchOutcome::FencedStaleEpoch { sink_epoch: 7 })
    );
    assert_eq!(
        executor.query_outcome("process-key"),
        Ok(ExecutorQueryResult::NotExecuted)
    );
    assert_eq!(supervisor.access_count(), 0);
}

#[test]
fn durable_process_check_dispatch_records_executing_before_supervisor_access_without_advancing_task()
 {
    let database_path = temporary_authority_database_path();
    let store = Arc::new(SqliteAuthorityStore::open(&database_path).expect("open authority store"));
    let task_object_id = object_id(521);
    let effect_object_id = object_id(522);
    let intent_object_id = object_id(523);
    let admitted_at = WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid admission time");

    for (object_id, domain, lifecycle_state, event_id) in [
        (
            task_object_id.clone(),
            LifecycleDomain::Task,
            "RUNNING",
            521,
        ),
        (
            effect_object_id.clone(),
            LifecycleDomain::Effect,
            "PROPOSED",
            522,
        ),
    ] {
        store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: object_id.clone(),
                    domain,
                    state: state(lifecycle_state),
                    version: Version::INITIAL,
                    body: json!({"fixture": "p2-t06-d03"}),
                },
                admitted_at: admitted_at.clone(),
                event: EventDraft {
                    event_id: EventId::parse(&format!("00000000-0000-7000-a000-{event_id:012x}"))
                        .expect("valid event identifier"),
                    object_id,
                    domain,
                    object_version: Version::INITIAL,
                    event_type: "fixture.admitted".to_owned(),
                    canonical_json: "{\"fixture\":true}".to_owned(),
                },
                outbox: Vec::new(),
                fencing_epoch: Some(1),
            })
            .expect("admit durable fixture object");
    }
    let idempotency_key = "p2-t06-d03-process-check";
    let parameters_digest = "sha256:p2-t06-d03-process-check";
    store
        .insert_intent(
            &IntentRow {
                intent_id: intent_object_id.clone(),
                idempotency_key: idempotency_key.to_owned(),
                parameters_digest: parameters_digest.to_owned(),
                action: "check".to_owned(),
                target: "process://4242".to_owned(),
                effect_object_id: effect_object_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: None,
                canonical_json: "{\"intent\":\"p2-t06-d03\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000523")
                    .expect("valid intent event identifier"),
                object_id: intent_object_id,
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"intent\":\"p2-t06-d03\"}".to_owned(),
            },
        )
        .expect("persist durable intent");

    let mut request = process_check_request();
    request.descriptor.output_limit_bytes = 32;
    let validated_request =
        validate_native_tool_request(&request).expect("valid process check request");
    let supervisor = Arc::new(BoundedProcessCheckSupervisor::new(Duration::from_secs(2)));
    supervisor.register(
        4242,
        b"state=ready token=secret process output that is too long",
        Duration::from_millis(1),
    );
    let executor = NativeProcessCheckExecutor::new(1, supervisor, Duration::from_secs(1));
    executor
        .stage_request(
            idempotency_key.to_owned(),
            parameters_digest.to_owned(),
            &validated_request,
        )
        .expect("stage durable intent identity");
    let hook_store = Arc::clone(&store);
    let hook_effect_object_id = effect_object_id.clone();
    let hook_task_object_id = task_object_id.clone();
    let supervisor_accesses = Arc::new(AtomicUsize::new(0));
    let hook_supervisor_accesses = Arc::clone(&supervisor_accesses);
    executor.install_before_check_hook(move || {
        let effect = hook_store
            .load_object(LifecycleDomain::Effect, &hook_effect_object_id)
            .expect("load effect before supervisor access")
            .expect("durable effect exists");
        let task = hook_store
            .load_object(LifecycleDomain::Task, &hook_task_object_id)
            .expect("load task before supervisor access")
            .expect("durable task exists");
        assert_eq!(effect.state.as_str(), "EXECUTING");
        assert_eq!(
            effect.version,
            Version::new(3).expect("valid executing version")
        );
        assert_eq!(task.state.as_str(), "RUNNING");
        assert_eq!(task.version, Version::INITIAL);
        hook_supervisor_accesses.fetch_add(1, Ordering::SeqCst);
    });

    let clock = FixedEffectClock(admitted_at);
    let identifiers = UuidV7Generator;
    let effect_protocol = EffectProtocol::new(
        store.as_ref(),
        &clock,
        &identifiers,
        UriRef::parse("actor://personal/daemon").expect("valid actor reference"),
        UriRef::parse("authority://personal/effect-authority").expect("valid authority reference"),
        UriRef::parse("correlation://personal/p2-t06-d03").expect("valid correlation reference"),
    );
    let grant = effect_grant();
    let governance_currency = GovernanceCurrency {
        revocation_epoch: 1,
        capability_set_version: 1,
    };
    let writer_lease = WriterLease { epoch: 1 };

    dispatch_staged_process_check_effect(
        &effect_protocol,
        &effect_object_id,
        Version::INITIAL,
        &grant,
        &governance_currency,
        &executor,
        &writer_lease,
    )
    .expect("dispatch durable process check");

    let completed_output = executor
        .completed_output(idempotency_key)
        .expect("process check retains output under original key");
    assert_eq!(
        completed_output,
        b"state=ready token=[REDACTED] pro".to_vec()
    );
    assert_eq!(completed_output.len(), 32);
    assert!(!String::from_utf8_lossy(&completed_output).contains("secret"));
    assert_eq!(supervisor_accesses.load(Ordering::SeqCst), 1);
    assert_eq!(
        executor.query_outcome(idempotency_key),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
    assert!(matches!(
        executor.dispatch(&process_check_call(
            idempotency_key,
            parameters_digest,
            "process://4242",
            1,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert_eq!(supervisor_accesses.load(Ordering::SeqCst), 1);
    let effect = store
        .load_object(LifecycleDomain::Effect, &effect_object_id)
        .expect("load effect after process check")
        .expect("durable effect exists");
    assert_eq!(effect.state.as_str(), "EXECUTED");
    assert_eq!(
        effect.version,
        Version::new(4).expect("valid executed version")
    );
    let task = store
        .load_object(LifecycleDomain::Task, &task_object_id)
        .expect("load task after process check")
        .expect("durable task exists");
    assert_eq!(task.state.as_str(), "RUNNING");
    assert_eq!(task.version, Version::INITIAL);

    std::fs::remove_file(database_path).unwrap_or(());
}

#[test]
fn unknown_process_check_dispatch_reconciles_original_key_without_advancing_task() {
    let database_path = temporary_authority_database_path();
    let store = Arc::new(SqliteAuthorityStore::open(&database_path).expect("open authority store"));
    let task_object_id = object_id(531);
    let effect_object_id = object_id(532);
    let intent_object_id = object_id(533);
    let admitted_at = WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid admission time");

    for (object_id, domain, lifecycle_state, event_id) in [
        (
            task_object_id.clone(),
            LifecycleDomain::Task,
            "RUNNING",
            531,
        ),
        (
            effect_object_id.clone(),
            LifecycleDomain::Effect,
            "PROPOSED",
            532,
        ),
    ] {
        store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: object_id.clone(),
                    domain,
                    state: state(lifecycle_state),
                    version: Version::INITIAL,
                    body: json!({"fixture": "p2-t06-d03-unknown"}),
                },
                admitted_at: admitted_at.clone(),
                event: EventDraft {
                    event_id: EventId::parse(&format!("00000000-0000-7000-a000-{event_id:012x}"))
                        .expect("valid event identifier"),
                    object_id,
                    domain,
                    object_version: Version::INITIAL,
                    event_type: "fixture.admitted".to_owned(),
                    canonical_json: "{\"fixture\":true}".to_owned(),
                },
                outbox: Vec::new(),
                fencing_epoch: Some(1),
            })
            .expect("admit durable fixture object");
    }

    let idempotency_key = "p2-t06-d03-unknown-process-check";
    let parameters_digest = "sha256:p2-t06-d03-unknown-process-check";
    store
        .insert_intent(
            &IntentRow {
                intent_id: intent_object_id.clone(),
                idempotency_key: idempotency_key.to_owned(),
                parameters_digest: parameters_digest.to_owned(),
                action: "check".to_owned(),
                target: "process://4242".to_owned(),
                effect_object_id: effect_object_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: None,
                canonical_json: "{\"intent\":\"p2-t06-d03-unknown\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000533")
                    .expect("valid intent event identifier"),
                object_id: intent_object_id,
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"intent\":\"p2-t06-d03-unknown\"}".to_owned(),
            },
        )
        .expect("persist durable intent");

    let mut request = process_check_request();
    request.descriptor.output_limit_bytes = 32;
    let validated_request =
        validate_native_tool_request(&request).expect("valid process check request");
    let supervisor = Arc::new(BoundedProcessCheckSupervisor::new(Duration::from_secs(2)));
    supervisor.register(
        4242,
        b"state=ready token=secret process output that is too long",
        Duration::from_millis(1),
    );
    let executor =
        NativeProcessCheckExecutor::new(1, Arc::clone(&supervisor), Duration::from_secs(1));
    executor
        .stage_request(
            idempotency_key.to_owned(),
            parameters_digest.to_owned(),
            &validated_request,
        )
        .expect("stage durable intent identity");
    let hook_store = Arc::clone(&store);
    let hook_effect_object_id = effect_object_id.clone();
    let hook_task_object_id = task_object_id.clone();
    let supervisor_accesses = Arc::new(AtomicUsize::new(0));
    let hook_supervisor_accesses = Arc::clone(&supervisor_accesses);
    executor.install_before_check_hook(move || {
        let effect = hook_store
            .load_object(LifecycleDomain::Effect, &hook_effect_object_id)
            .expect("load effect before supervisor access")
            .expect("durable effect exists");
        let task = hook_store
            .load_object(LifecycleDomain::Task, &hook_task_object_id)
            .expect("load task before supervisor access")
            .expect("durable task exists");
        assert_eq!(effect.state.as_str(), "EXECUTING");
        assert_eq!(
            effect.version,
            Version::new(3).expect("valid executing version")
        );
        assert_eq!(task.state.as_str(), "RUNNING");
        assert_eq!(task.version, Version::INITIAL);
        hook_supervisor_accesses.fetch_add(1, Ordering::SeqCst);
    });

    let clock = FixedEffectClock(admitted_at);
    let identifiers = UuidV7Generator;
    let effect_protocol = EffectProtocol::new(
        store.as_ref(),
        &clock,
        &identifiers,
        UriRef::parse("actor://personal/daemon").expect("valid actor reference"),
        UriRef::parse("authority://personal/effect-authority").expect("valid authority reference"),
        UriRef::parse("correlation://personal/p2-t06-d03-unknown")
            .expect("valid correlation reference"),
    );
    let grant = effect_grant();
    let governance_currency = GovernanceCurrency {
        revocation_epoch: 1,
        capability_set_version: 1,
    };
    let writer_lease = WriterLease { epoch: 1 };
    let authorized = effect_protocol
        .authorize_effect(
            &effect_object_id,
            Version::INITIAL,
            &grant,
            &governance_currency,
            &writer_lease,
        )
        .expect("authorize durable process check");
    let unknown_executor = UnknownAfterNativeProcessCheckDispatchExecutor {
        native_executor: &executor,
    };
    let (dispatched, outcome) = effect_protocol
        .dispatch_effect(
            &effect_object_id,
            authorized.after_version,
            &grant,
            &governance_currency,
            &unknown_executor,
            &writer_lease,
        )
        .expect("dispatch unknown process check");
    assert!(matches!(outcome, DispatchOutcome::Unknown { .. }));
    effect_protocol
        .record_outcome(
            &effect_object_id,
            dispatched.after_version,
            &outcome,
            &writer_lease,
        )
        .expect("record unknown process check outcome");

    let (reconciled, query) = effect_protocol
        .reconcile(
            &effect_object_id,
            "OUTCOME_UNKNOWN",
            Version::new(4).expect("valid unknown outcome version"),
            &unknown_executor,
            &writer_lease,
        )
        .expect("reconcile unknown process check");
    assert_eq!(query, ExecutorQueryResult::ExecutedWithOriginalKey);
    assert_eq!(supervisor_accesses.load(Ordering::SeqCst), 1);
    assert_eq!(
        reconciled.after_version,
        Version::new(5).expect("valid reconciled version")
    );

    let completed_output = executor
        .completed_output(idempotency_key)
        .expect("process check retains output under original key");
    assert_eq!(
        completed_output,
        b"state=ready token=[REDACTED] pro".to_vec()
    );
    assert_eq!(completed_output.len(), 32);
    assert!(!String::from_utf8_lossy(&completed_output).contains("secret"));
    assert_eq!(
        executor.query_outcome(idempotency_key),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
    let effect = store
        .load_object(LifecycleDomain::Effect, &effect_object_id)
        .expect("load reconciled effect")
        .expect("durable effect exists");
    assert_eq!(effect.state.as_str(), "RECONCILED");
    let task = store
        .load_object(LifecycleDomain::Task, &task_object_id)
        .expect("load unchanged task")
        .expect("durable task exists");
    assert_eq!(task.state.as_str(), "RUNNING");
    assert_eq!(task.version, Version::INITIAL);

    std::fs::remove_file(database_path).unwrap_or(());
}

#[test]
fn runtime_spine_outcome_unknown_reconciles_original_key_and_rejects_blind_retry() {
    // B12 observation floor: OUTCOME_UNKNOWN reconciles by the original key
    // and never remints a second supervisor access / blind retry.
    let database_path = temporary_authority_database_path();
    let store = Arc::new(SqliteAuthorityStore::open(&database_path).expect("open authority store"));
    let task_object_id = object_id(541);
    let effect_object_id = object_id(542);
    let intent_object_id = object_id(543);
    let admitted_at = WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid admission time");

    for (object_id, domain, lifecycle_state, event_id) in [
        (
            task_object_id.clone(),
            LifecycleDomain::Task,
            "RUNNING",
            541,
        ),
        (
            effect_object_id.clone(),
            LifecycleDomain::Effect,
            "PROPOSED",
            542,
        ),
    ] {
        store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: object_id.clone(),
                    domain,
                    state: state(lifecycle_state),
                    version: Version::INITIAL,
                    body: json!({"fixture": "p2-t08-d03-unknown"}),
                },
                admitted_at: admitted_at.clone(),
                event: EventDraft {
                    event_id: EventId::parse(&format!("00000000-0000-7000-a000-{event_id:012x}"))
                        .expect("valid event identifier"),
                    object_id,
                    domain,
                    object_version: Version::INITIAL,
                    event_type: "fixture.admitted".to_owned(),
                    canonical_json: "{\"fixture\":true}".to_owned(),
                },
                outbox: Vec::new(),
                fencing_epoch: Some(1),
            })
            .expect("admit durable fixture object");
    }

    let idempotency_key = "p2-t08-d03-unknown-process-check";
    let parameters_digest = "sha256:p2-t08-d03-unknown-process-check";
    store
        .insert_intent(
            &IntentRow {
                intent_id: intent_object_id.clone(),
                idempotency_key: idempotency_key.to_owned(),
                parameters_digest: parameters_digest.to_owned(),
                action: "check".to_owned(),
                target: "process://4242".to_owned(),
                effect_object_id: effect_object_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: None,
                canonical_json: "{\"intent\":\"p2-t08-d03-unknown\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000543")
                    .expect("valid intent event identifier"),
                object_id: intent_object_id,
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"intent\":\"p2-t08-d03-unknown\"}".to_owned(),
            },
        )
        .expect("persist durable intent");

    let mut request = process_check_request();
    request.descriptor.output_limit_bytes = 32;
    let validated_request =
        validate_native_tool_request(&request).expect("valid process check request");
    let supervisor = Arc::new(BoundedProcessCheckSupervisor::new(Duration::from_secs(2)));
    supervisor.register(
        4242,
        b"state=ready token=secret process output that is too long",
        Duration::from_millis(1),
    );
    let executor =
        NativeProcessCheckExecutor::new(1, Arc::clone(&supervisor), Duration::from_secs(1));
    executor
        .stage_request(
            idempotency_key.to_owned(),
            parameters_digest.to_owned(),
            &validated_request,
        )
        .expect("stage durable intent identity");
    let supervisor_accesses = Arc::new(AtomicUsize::new(0));
    let hook_supervisor_accesses = Arc::clone(&supervisor_accesses);
    executor.install_before_check_hook(move || {
        hook_supervisor_accesses.fetch_add(1, Ordering::SeqCst);
    });

    let clock = FixedEffectClock(admitted_at);
    let identifiers = UuidV7Generator;
    let effect_protocol = EffectProtocol::new(
        store.as_ref(),
        &clock,
        &identifiers,
        UriRef::parse("actor://personal/daemon").expect("valid actor reference"),
        UriRef::parse("authority://personal/effect-authority").expect("valid authority reference"),
        UriRef::parse("correlation://personal/p2-t08-d03-unknown")
            .expect("valid correlation reference"),
    );
    let grant = effect_grant();
    let governance_currency = GovernanceCurrency {
        revocation_epoch: 1,
        capability_set_version: 1,
    };
    let writer_lease = WriterLease { epoch: 1 };
    let authorized = effect_protocol
        .authorize_effect(
            &effect_object_id,
            Version::INITIAL,
            &grant,
            &governance_currency,
            &writer_lease,
        )
        .expect("authorize durable process check");
    let unknown_executor = UnknownAfterNativeProcessCheckDispatchExecutor {
        native_executor: &executor,
    };
    let (dispatched, outcome) = effect_protocol
        .dispatch_effect(
            &effect_object_id,
            authorized.after_version,
            &grant,
            &governance_currency,
            &unknown_executor,
            &writer_lease,
        )
        .expect("dispatch unknown process check");
    assert!(matches!(outcome, DispatchOutcome::Unknown { .. }));
    effect_protocol
        .record_outcome(
            &effect_object_id,
            dispatched.after_version,
            &outcome,
            &writer_lease,
        )
        .expect("record unknown process check outcome");

    let (reconciled, query) = effect_protocol
        .reconcile(
            &effect_object_id,
            "OUTCOME_UNKNOWN",
            Version::new(4).expect("valid unknown outcome version"),
            &unknown_executor,
            &writer_lease,
        )
        .expect("reconcile unknown process check");
    assert_eq!(query, ExecutorQueryResult::ExecutedWithOriginalKey);
    assert_eq!(supervisor_accesses.load(Ordering::SeqCst), 1);
    assert_eq!(
        reconciled.after_version,
        Version::new(5).expect("valid reconciled version")
    );

    // Blind retry / reminted key must not re-enter the supervisor. Query and
    // duplicate dispatch keep the original key and do not increment access.
    assert_eq!(
        executor.query_outcome(idempotency_key),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
    let reminted = process_check_call(
        "p2-t08-d03-reminted-key",
        parameters_digest,
        "process://4242",
        1,
    );
    let remint_outcome = executor
        .dispatch(&reminted)
        .expect("reminted key fails closed without supervisor access");
    assert!(matches!(
        remint_outcome,
        DispatchOutcome::NotExecuted { .. }
    ));
    assert_eq!(supervisor_accesses.load(Ordering::SeqCst), 1);
    let duplicate = process_check_call(idempotency_key, parameters_digest, "process://4242", 1);
    let duplicate_outcome = executor
        .dispatch(&duplicate)
        .expect("original key absorbs without second supervisor access");
    assert!(matches!(
        duplicate_outcome,
        DispatchOutcome::Executed { .. }
    ));
    assert_eq!(supervisor_accesses.load(Ordering::SeqCst), 1);

    let task = store
        .load_object(LifecycleDomain::Task, &task_object_id)
        .expect("load unchanged task")
        .expect("durable task exists");
    assert_eq!(task.state.as_str(), "RUNNING");
    assert_eq!(task.version, Version::INITIAL);

    std::fs::remove_file(database_path).unwrap_or(());
}

#[test]
fn workspace_target_cannot_escape_approved_root() {
    let mut request = request_for(NativeOperationFamily::WorkspaceRead);
    request.target = "workspace://../secrets.txt".to_owned();
    assert_eq!(
        validate_native_tool_request(&request),
        Err(NativeToolExecutionError::WorkspaceTargetEscapesRoot)
    );
}

#[test]
fn disabled_descriptor_fails_before_execution() {
    let mut request = request_for(NativeOperationFamily::WorkspaceRead);
    request.descriptor.availability = ToolAvailability::Disabled;
    assert!(matches!(
        validate_native_tool_request(&request),
        Err(NativeToolExecutionError::InvalidDescriptor(_))
    ));
}

#[test]
fn mutation_requires_bounded_input() {
    let mut request = request_for(NativeOperationFamily::WorkspaceWrite);
    request.input.clear();
    assert_eq!(
        validate_native_tool_request(&request),
        Err(NativeToolExecutionError::MutationInputRequired)
    );
}

#[test]
fn network_target_rejects_credentials_and_plain_http() {
    let mut request = request_for(NativeOperationFamily::HttpFetchReadOnly);
    request.target = "http://example.test/data".to_owned();
    assert_eq!(
        validate_native_tool_request(&request),
        Err(NativeToolExecutionError::NetworkTargetMustUseHttps)
    );
    request.target = "https://user:secret@example.test/data".to_owned();
    assert_eq!(
        validate_native_tool_request(&request),
        Err(NativeToolExecutionError::NetworkTargetContainsCredentials)
    );
}

#[test]
fn output_cursor_requires_monotonic_resume_and_enforces_limit() {
    let mut cursor = BoundedOutputCursor::new(5);
    assert_eq!(
        cursor.next_chunk(b"123456789", 0, 2),
        Ok(Some((0, b"12".to_vec())))
    );
    assert_eq!(
        cursor.next_chunk(b"123456789", 2, 2),
        Ok(Some((2, b"34".to_vec())))
    );
    assert_eq!(
        cursor.next_chunk(b"123456789", 1, 2),
        Err(NativeToolExecutionError::InvalidDescriptor(
            "output cursor is stale or out of order".to_owned()
        ))
    );
    assert_eq!(
        cursor.next_chunk(b"123456789", 4, 2),
        Ok(Some((4, b"5".to_vec())))
    );
    assert_eq!(cursor.next_chunk(b"123456789", 5, 2), Ok(None));
}

#[test]
fn sensitive_output_is_redacted_before_projection() {
    assert_eq!(
        redact_sensitive_output("ok api_key=secret token=hidden done"),
        "ok api_key=[REDACTED] token=[REDACTED] done"
    );
}

fn workspace_read_call(
    idempotency_key: &str,
    parameters_digest: &str,
    target: &str,
    fencing_epoch: i64,
) -> ExecutorCall {
    ExecutorCall {
        action: "read".to_owned(),
        target: target.to_owned(),
        idempotency_key: idempotency_key.to_owned(),
        parameters_digest: parameters_digest.to_owned(),
        authorization_digest: "authorization-digest".to_owned(),
        fencing_epoch,
    }
}

struct FixedEffectClock(WallTimestamp);

impl Clock for FixedEffectClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        Ok(self.0.clone())
    }
}

/// Simulates a lost response only after the native sink has completed its
/// idempotent filesystem read. Queries still reach that real sink.
struct UnknownAfterNativeDispatchExecutor<'executor> {
    native_executor: &'executor NativeWorkspaceReadExecutor,
}

impl EffectExecutor for UnknownAfterNativeDispatchExecutor<'_> {
    fn capabilities(&self) -> ExecutorCapabilities {
        self.native_executor.capabilities()
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        self.native_executor.dispatch(call)?;
        Ok(DispatchOutcome::Unknown {
            detail: "simulated lost post-I/O response".to_owned(),
        })
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        self.native_executor.query_outcome(idempotency_key)
    }
}

/// Simulates a lost response after the native process check completes.
/// Queries still reach the real process-check executor and its original
/// idempotency key.
struct UnknownAfterNativeProcessCheckDispatchExecutor<'executor, S> {
    native_executor: &'executor NativeProcessCheckExecutor<S>,
}

impl<S> EffectExecutor for UnknownAfterNativeProcessCheckDispatchExecutor<'_, S>
where
    S: ProcessCheckSupervisor,
{
    fn capabilities(&self) -> ExecutorCapabilities {
        self.native_executor.capabilities()
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        self.native_executor.dispatch(call)?;
        Ok(DispatchOutcome::Unknown {
            detail: "simulated lost post-process-check response".to_owned(),
        })
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        self.native_executor.query_outcome(idempotency_key)
    }
}

fn state(value: &str) -> StateName {
    StateName::parse(value).expect("valid lifecycle state")
}

fn object_id(value: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{value:012x}"))
        .expect("valid object identifier")
}

fn effect_grant() -> AuthorizationGrant {
    let authorization_time =
        WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid authorization time");
    authorize(
        &AuthzSnapshot {
            tenant_id: "personal-test".to_owned(),
            principal: PrincipalFacts {
                principal_ref: UriRef::parse("principal://personal/daemon")
                    .expect("valid principal reference"),
                authenticated: true,
                active: true,
                tenant_id: Some("personal-test".to_owned()),
            },
            actor_chain: ActorChainFacts {
                chain_digest: format!("sha256:{}", "c".repeat(64)),
                resolved: true,
            },
            membership: Some(MembershipFacts {
                valid: true,
                roles: ["daemon".to_owned()].into(),
            }),
            capability_links: vec![cognitive_domain::capability::CapabilityConstraints {
                subject: "principal://personal/daemon".to_owned(),
                audience: "authority://personal/effect-authority".to_owned(),
                resource: "scope://personal/workspace-read".to_owned(),
                purpose: "task_execution".to_owned(),
                actions: ["filesystem.read".to_owned()].into(),
                parameter_bounds: BTreeMap::new(),
                lease: cognitive_domain::capability::LeaseWindow {
                    not_before: WallTimestamp::parse("2026-08-04T12:00:00Z")
                        .expect("valid lease start"),
                    expires: WallTimestamp::parse("2026-08-04T12:05:00Z").expect("valid lease end"),
                },
                depth_remaining: 1,
                issued_epoch: 1,
            }],
            capability_set_version: 1,
            explicit_denies: Vec::new(),
            revocation_epoch: 1,
            decided_at: authorization_time,
        },
        &ObjectGovernance {
            object_ref: "effect://personal/workspace-read".to_owned(),
            tenant_id: Some("personal-test".to_owned()),
            owner_ref: "principal://personal/daemon".to_owned(),
            resource_scope: "scope://personal/workspace-read/effect".to_owned(),
            conversation_ref: None,
        },
        &AccessRequest {
            action: "filesystem.read".to_owned(),
            purpose: "task_execution".to_owned(),
        },
    )
    .expect("grant workspace-read authority")
}

fn temporary_authority_database_path() -> PathBuf {
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "cognitiveos-tool-executor-{}-{timestamp_nanos}.db",
        std::process::id()
    ))
}

#[test]
fn workspace_read_requires_a_staged_digest_bound_request_before_io() {
    let temporary_workspace = TestWorkspace::new("digest-binding");
    let workspace_file = temporary_workspace.path.join("notes.txt");
    std::fs::write(&workspace_file, "safe output").expect("write workspace fixture");
    let mut request = request_for(NativeOperationFamily::WorkspaceRead);
    request.target = "workspace://notes.txt".to_owned();
    request.workspace_root = Some(temporary_workspace.path.clone());
    let validated_request = validate_native_tool_request(&request).expect("valid request");
    let executor = NativeWorkspaceReadExecutor::new(7);
    executor
        .stage_request(
            "read-key-1".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        )
        .expect("stage daemon-bound request");

    let mismatched_call =
        workspace_read_call("read-key-1", "different-digest", "workspace://notes.txt", 7);
    assert!(matches!(
        executor.dispatch(&mismatched_call),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(executor.completed_output("read-key-1"), None);

    let matched_call = workspace_read_call("read-key-1", "digest-1", "workspace://notes.txt", 7);
    assert!(matches!(
        executor.dispatch(&matched_call),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert_eq!(
        executor.completed_output("read-key-1"),
        Some(b"safe output".to_vec())
    );
}

#[test]
fn non_read_descriptor_cannot_be_staged_for_effect_protocol_dispatch() {
    let mut request = request_for(NativeOperationFamily::WorkspaceWrite);
    request.input = b"required mutation input".to_vec();
    let validated_request = validate_native_tool_request(&request).expect("valid write request");
    let executor = NativeWorkspaceReadExecutor::new(7);

    assert_eq!(
        executor.stage_request(
            "write-key-1".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        ),
        Err(NativeToolExecutionError::UnsupportedExecutionFamily)
    );
    assert!(matches!(
        executor.dispatch(&workspace_read_call(
            "write-key-1",
            "digest-1",
            "workspace://notes/today.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
}

#[test]
fn workspace_read_bounds_redacts_and_idempotently_queries_output() {
    let temporary_workspace = TestWorkspace::new("redaction-bounds");
    let workspace_file = temporary_workspace.path.join("notes.txt");
    std::fs::write(&workspace_file, "token=secret 123456789").expect("write workspace fixture");
    let mut request = request_for(NativeOperationFamily::WorkspaceRead);
    request.target = "workspace://notes.txt".to_owned();
    request.workspace_root = Some(temporary_workspace.path.clone());
    request.descriptor.output_limit_bytes = 16;
    let validated_request = validate_native_tool_request(&request).expect("valid request");
    let executor = NativeWorkspaceReadExecutor::new(7);
    executor
        .stage_request(
            "read-key-2".to_owned(),
            "digest-2".to_owned(),
            &validated_request,
        )
        .expect("stage daemon-bound request");
    let call = workspace_read_call("read-key-2", "digest-2", "workspace://notes.txt", 7);

    let first_outcome = executor.dispatch(&call).expect("execute workspace read");
    let second_outcome = executor.dispatch(&call).expect("absorb duplicate dispatch");
    assert_eq!(first_outcome, second_outcome);
    assert_eq!(
        executor.completed_output("read-key-2"),
        Some(b"token=[REDACTED]".to_vec())
    );
    assert_eq!(
        executor.query_outcome("read-key-2"),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
}

#[test]
fn durable_workspace_read_dispatch_records_executing_before_io_without_advancing_task() {
    let temporary_workspace = TestWorkspace::new("durable-effect-dispatch");
    let workspace_file = temporary_workspace.path.join("notes.txt");
    std::fs::write(&workspace_file, "token=secret 123456789").expect("write workspace fixture");
    let database_path = temporary_authority_database_path();
    let store = Arc::new(SqliteAuthorityStore::open(&database_path).expect("open authority store"));
    let task_object_id = object_id(501);
    let effect_object_id = object_id(502);
    let intent_object_id = object_id(503);
    let admitted_at = WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid admission time");

    for (object_id, domain, lifecycle_state, event_id) in [
        (
            task_object_id.clone(),
            LifecycleDomain::Task,
            "RUNNING",
            501,
        ),
        (
            effect_object_id.clone(),
            LifecycleDomain::Effect,
            "PROPOSED",
            502,
        ),
    ] {
        store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: object_id.clone(),
                    domain,
                    state: state(lifecycle_state),
                    version: Version::INITIAL,
                    body: json!({"fixture": "p2-t06-d02"}),
                },
                admitted_at: admitted_at.clone(),
                event: EventDraft {
                    event_id: EventId::parse(&format!("00000000-0000-7000-a000-{event_id:012x}"))
                        .expect("valid event identifier"),
                    object_id,
                    domain,
                    object_version: Version::INITIAL,
                    event_type: "fixture.admitted".to_owned(),
                    canonical_json: "{\"fixture\":true}".to_owned(),
                },
                outbox: Vec::new(),
                fencing_epoch: Some(1),
            })
            .expect("admit durable fixture object");
    }
    let idempotency_key = "p2-t06-d02-workspace-read";
    let parameters_digest = "sha256:p2-t06-d02-workspace-read";
    store
        .insert_intent(
            &IntentRow {
                intent_id: intent_object_id.clone(),
                idempotency_key: idempotency_key.to_owned(),
                parameters_digest: parameters_digest.to_owned(),
                action: "read".to_owned(),
                target: "workspace://notes.txt".to_owned(),
                effect_object_id: effect_object_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: None,
                canonical_json: "{\"intent\":\"p2-t06-d02\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000503")
                    .expect("valid intent event identifier"),
                object_id: intent_object_id,
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"intent\":\"p2-t06-d02\"}".to_owned(),
            },
        )
        .expect("persist durable intent");

    let mut request = request_for(NativeOperationFamily::WorkspaceRead);
    request.target = "workspace://notes.txt".to_owned();
    request.workspace_root = Some(temporary_workspace.path.clone());
    request.descriptor.output_limit_bytes = 16;
    let validated_request = validate_native_tool_request(&request).expect("valid workspace read");
    let executor = NativeWorkspaceReadExecutor::new(1);
    executor
        .stage_request(
            idempotency_key.to_owned(),
            parameters_digest.to_owned(),
            &validated_request,
        )
        .expect("stage durable intent identity");
    let hook_store = Arc::clone(&store);
    let hook_effect_object_id = effect_object_id.clone();
    let hook_task_object_id = task_object_id.clone();
    executor.install_before_read_hook(move || {
        let effect = hook_store
            .load_object(LifecycleDomain::Effect, &hook_effect_object_id)
            .expect("load effect before read")
            .expect("durable effect exists");
        let task = hook_store
            .load_object(LifecycleDomain::Task, &hook_task_object_id)
            .expect("load task before read")
            .expect("durable task exists");
        assert_eq!(effect.state.as_str(), "EXECUTING");
        assert_eq!(
            effect.version,
            Version::new(3).expect("valid executing version")
        );
        assert_eq!(task.state.as_str(), "RUNNING");
        assert_eq!(task.version, Version::INITIAL);
    });

    let clock = FixedEffectClock(admitted_at);
    let identifiers = UuidV7Generator;
    let effect_protocol = EffectProtocol::new(
        store.as_ref(),
        &clock,
        &identifiers,
        UriRef::parse("actor://personal/daemon").expect("valid actor reference"),
        UriRef::parse("authority://personal/effect-authority").expect("valid authority reference"),
        UriRef::parse("correlation://personal/p2-t06-d02").expect("valid correlation reference"),
    );
    let grant = effect_grant();
    let governance_currency = GovernanceCurrency {
        revocation_epoch: 1,
        capability_set_version: 1,
    };
    let writer_lease = WriterLease { epoch: 1 };

    dispatch_staged_workspace_read_effect(
        &effect_protocol,
        &effect_object_id,
        Version::INITIAL,
        &grant,
        &governance_currency,
        &executor,
        &writer_lease,
    )
    .expect("dispatch durable workspace read");

    let completed_output = executor
        .completed_output(idempotency_key)
        .expect("workspace read retains output under original key");
    assert_eq!(completed_output, b"token=[REDACTED]".to_vec());
    assert_eq!(completed_output.len(), 16);
    assert_eq!(
        executor.query_outcome(idempotency_key),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
    assert!(matches!(
        executor.dispatch(&workspace_read_call(
            idempotency_key,
            parameters_digest,
            "workspace://notes.txt",
            1,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &effect_object_id)
            .expect("load effect after read")
            .expect("durable effect exists")
            .state
            .as_str(),
        "EXECUTED"
    );
    assert_eq!(
        store
            .load_object(LifecycleDomain::Task, &task_object_id)
            .expect("load task after read")
            .expect("durable task exists")
            .version,
        Version::INITIAL
    );

    std::fs::remove_file(database_path).unwrap_or(());
}

#[test]
fn unknown_native_workspace_read_reconciles_original_key_without_second_read() {
    let temporary_workspace = TestWorkspace::new("unknown-outcome-reconciliation");
    let workspace_file = temporary_workspace.path.join("notes.txt");
    std::fs::write(&workspace_file, "token=secret durable workspace output")
        .expect("write workspace fixture");
    let database_path = temporary_authority_database_path();
    let store = Arc::new(SqliteAuthorityStore::open(&database_path).expect("open authority store"));
    let task_object_id = object_id(511);
    let effect_object_id = object_id(512);
    let intent_object_id = object_id(513);
    let admitted_at = WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid admission time");

    for (object_id, domain, lifecycle_state, event_id) in [
        (
            task_object_id.clone(),
            LifecycleDomain::Task,
            "RUNNING",
            511,
        ),
        (
            effect_object_id.clone(),
            LifecycleDomain::Effect,
            "PROPOSED",
            512,
        ),
    ] {
        store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: object_id.clone(),
                    domain,
                    state: state(lifecycle_state),
                    version: Version::INITIAL,
                    body: json!({"fixture": "p2-t06-d02-unknown-outcome"}),
                },
                admitted_at: admitted_at.clone(),
                event: EventDraft {
                    event_id: EventId::parse(&format!("00000000-0000-7000-a000-{event_id:012x}"))
                        .expect("valid event identifier"),
                    object_id,
                    domain,
                    object_version: Version::INITIAL,
                    event_type: "fixture.admitted".to_owned(),
                    canonical_json: "{\"fixture\":true}".to_owned(),
                },
                outbox: Vec::new(),
                fencing_epoch: Some(1),
            })
            .expect("admit durable fixture object");
    }
    let idempotency_key = "p2-t06-d02-unknown-native-workspace-read";
    let parameters_digest = "sha256:p2-t06-d02-unknown-native-workspace-read";
    store
        .insert_intent(
            &IntentRow {
                intent_id: intent_object_id.clone(),
                idempotency_key: idempotency_key.to_owned(),
                parameters_digest: parameters_digest.to_owned(),
                action: "read".to_owned(),
                target: "workspace://notes.txt".to_owned(),
                effect_object_id: effect_object_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: None,
                canonical_json: "{\"intent\":\"p2-t06-d02\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000513")
                    .expect("valid intent event identifier"),
                object_id: intent_object_id,
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"intent\":\"p2-t06-d02\"}".to_owned(),
            },
        )
        .expect("persist durable intent");

    let mut request = request_for(NativeOperationFamily::WorkspaceRead);
    request.target = "workspace://notes.txt".to_owned();
    request.workspace_root = Some(temporary_workspace.path.clone());
    request.descriptor.output_limit_bytes = 16;
    let validated_request = validate_native_tool_request(&request).expect("valid workspace read");
    let native_executor = NativeWorkspaceReadExecutor::new(1);
    native_executor
        .stage_request(
            idempotency_key.to_owned(),
            parameters_digest.to_owned(),
            &validated_request,
        )
        .expect("stage original durable intent identity");
    let read_count = Arc::new(AtomicUsize::new(0));
    let read_count_for_hook = Arc::clone(&read_count);
    native_executor.install_before_read_hook(move || {
        read_count_for_hook.fetch_add(1, Ordering::SeqCst);
    });
    let unknown_executor = UnknownAfterNativeDispatchExecutor {
        native_executor: &native_executor,
    };

    let clock = FixedEffectClock(admitted_at);
    let identifiers = UuidV7Generator;
    let effect_protocol = EffectProtocol::new(
        store.as_ref(),
        &clock,
        &identifiers,
        UriRef::parse("actor://personal/daemon").expect("valid actor reference"),
        UriRef::parse("authority://personal/effect-authority").expect("valid authority reference"),
        UriRef::parse("correlation://personal/p2-t06-d02-unknown")
            .expect("valid correlation reference"),
    );
    let grant = effect_grant();
    let governance_currency = GovernanceCurrency {
        revocation_epoch: 1,
        capability_set_version: 1,
    };
    let writer_lease = WriterLease { epoch: 1 };

    let authorized = effect_protocol
        .authorize_effect(
            &effect_object_id,
            Version::INITIAL,
            &grant,
            &governance_currency,
            &writer_lease,
        )
        .expect("authorize staged effect");
    let (dispatched, outcome) = effect_protocol
        .dispatch_effect(
            &effect_object_id,
            authorized.after_version,
            &grant,
            &governance_currency,
            &unknown_executor,
            &writer_lease,
        )
        .expect("dispatch native read through lost-response wrapper");
    assert!(matches!(outcome, DispatchOutcome::Unknown { .. }));
    let unknown = effect_protocol
        .record_outcome(
            &effect_object_id,
            dispatched.after_version,
            &outcome,
            &writer_lease,
        )
        .expect("record unknown post-I/O outcome");

    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &effect_object_id)
            .expect("load unknown effect")
            .expect("durable effect exists")
            .state
            .as_str(),
        "OUTCOME_UNKNOWN"
    );
    assert_eq!(read_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        native_executor.completed_output(idempotency_key),
        Some(b"token=[REDACTED]".to_vec())
    );

    let (_, reconciliation_result) = effect_protocol
        .reconcile(
            &effect_object_id,
            "OUTCOME_UNKNOWN",
            unknown.after_version,
            &unknown_executor,
            &writer_lease,
        )
        .expect("reconcile the original idempotency key");
    assert_eq!(
        reconciliation_result,
        ExecutorQueryResult::ExecutedWithOriginalKey
    );
    assert_eq!(read_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &effect_object_id)
            .expect("load reconciled effect")
            .expect("durable effect exists")
            .state
            .as_str(),
        "RECONCILED"
    );
    assert_eq!(
        store
            .load_object(LifecycleDomain::Task, &task_object_id)
            .expect("load unchanged task")
            .expect("durable task exists")
            .version,
        Version::INITIAL
    );

    std::fs::remove_file(database_path).unwrap_or(());
}

#[test]
fn workspace_read_sink_rejects_stale_fencing_before_io() {
    let executor = NativeWorkspaceReadExecutor::new(7);
    let call = workspace_read_call("read-key-3", "digest-3", "workspace://notes.txt", 6);
    assert_eq!(
        executor.dispatch(&call),
        Ok(DispatchOutcome::FencedStaleEpoch { sink_epoch: 7 })
    );
    assert_eq!(executor.completed_output("read-key-3"), None);
}
