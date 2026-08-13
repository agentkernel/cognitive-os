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
use cognitive_provider_transport::{
    ReadOnlyFetchError, ReadOnlyFetchMethod, ReadOnlyFetchRequest, ReadOnlyFetchResponse,
    ReadOnlyFetchTransport,
};
use cognitive_store::{SqliteAuthorityStore, UuidV7Generator};
use serde_json::json;
use std::path::PathBuf;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

struct TestWorkspace {
    path: PathBuf,
}

impl TestWorkspace {
    fn new(test_name: &str) -> Self {
        Self {
            path: temporary_workspace_path(test_name),
        }
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
        let _ = std::fs::remove_dir_all(durable_state_path(&self.path));
    }
}

fn temporary_workspace_path(test_name: &str) -> PathBuf {
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "cognitiveos-tool-executor-{test_name}-{}-{timestamp_nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&path).expect("create temporary workspace");
    path
}

fn durable_state_store(root: &std::path::Path) -> Arc<DurableExecutorStateStore> {
    Arc::new(
        DurableExecutorStateStore::open(&durable_state_path(root))
            .expect("open isolated durable executor state"),
    )
}

fn durable_state_path(root: &std::path::Path) -> PathBuf {
    let parent = root.parent().unwrap_or_else(|| std::path::Path::new("."));
    let name = root
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("workspace"))
        .to_string_lossy();
    parent.join(format!(".{name}-executor-state"))
}

fn mutation_executor(
    trusted_fencing_epoch: i64,
    workspace: &TestWorkspace,
) -> NativeWorkspaceMutationExecutor {
    NativeWorkspaceMutationExecutor::new(
        trusted_fencing_epoch,
        durable_state_store(&workspace.path),
    )
}

#[cfg(unix)]
fn create_test_file_link(source: &std::path::Path, target: &std::path::Path) {
    std::os::unix::fs::symlink(source, target).expect("create test file symlink");
}

#[cfg(windows)]
fn create_test_file_link(source: &std::path::Path, target: &std::path::Path) {
    std::os::windows::fs::symlink_file(source, target).expect("create test file reparse point");
}

#[cfg(unix)]
fn create_test_directory_link(source: &std::path::Path, target: &std::path::Path) {
    std::os::unix::fs::symlink(source, target).expect("create test directory symlink");
}

#[cfg(windows)]
fn create_test_directory_link(source: &std::path::Path, target: &std::path::Path) {
    std::os::windows::fs::symlink_dir(source, target).expect("create test directory reparse point");
}

fn request_for(family: NativeOperationFamily) -> NativeToolExecutionRequest {
    let descriptor = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|descriptor| descriptor.family == family)
        .cloned()
        .unwrap_or_else(|| panic!("catalog family missing: {family:?}"));
    let expected_preimage = matches!(
        family,
        NativeOperationFamily::WorkspaceWrite | NativeOperationFamily::WorkspacePatch
    )
    .then_some(WorkspacePreimage::Absent);
    NativeToolExecutionRequest {
        descriptor,
        target: "workspace://notes/today.txt".to_owned(),
        input: b"bounded input".to_vec(),
        workspace_root: Some(PathBuf::from("/tmp/cognitiveos-workspace")),
        expected_preimage,
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
    _output_limit_bytes: usize,
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
    let request = process_check_request();
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
    assert_eq!(
        output,
        b"state=ready token=[REDACTED] output-that-is-too-long".to_vec()
    );
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

    let request = process_check_request();
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
        b"state=ready token=[REDACTED] process output that is too long".to_vec()
    );
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

    let request = process_check_request();
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
        b"state=ready token=[REDACTED] process output that is too long".to_vec()
    );
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

    let request = process_check_request();
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
fn every_immutable_descriptor_field_is_catalog_bound_for_every_family() {
    for catalog_descriptor in BUILTIN_TOOL_CATALOG.iter() {
        let mut request = request_for(catalog_descriptor.family);
        match catalog_descriptor.family {
            NativeOperationFamily::ProcessCheck => request = process_check_request(),
            NativeOperationFamily::HttpFetchReadOnly => {
                request.target = format!("{FETCH_ORIGIN}/data");
                request.input.clear();
                request.workspace_root = None;
            }
            _ => {}
        }
        validate_native_tool_request(&request).expect("catalog descriptor must validate");

        let mut drifted_descriptors = Vec::new();
        let mut drifted = catalog_descriptor.clone();
        drifted.operation_id.push_str(".drift");
        drifted_descriptors.push(("operation_id", drifted));
        let mut drifted = catalog_descriptor.clone();
        drifted.action.push_str("-drift");
        drifted_descriptors.push(("action", drifted));
        let mut drifted = catalog_descriptor.clone();
        drifted.descriptor_version += 1;
        drifted_descriptors.push(("descriptor_version", drifted));
        let mut drifted = catalog_descriptor.clone();
        drifted.descriptor_digest = format!("sha256:{}", "0".repeat(64));
        drifted_descriptors.push(("descriptor_digest", drifted));
        let mut drifted = catalog_descriptor.clone();
        drifted.risk = if catalog_descriptor.risk == ToolRisk::ReadOnly {
            ToolRisk::NetworkRead
        } else {
            ToolRisk::ReadOnly
        };
        drifted_descriptors.push(("risk", drifted));
        let mut drifted = catalog_descriptor.clone();
        drifted.executor.push_str(".drift");
        drifted_descriptors.push(("executor", drifted));
        let mut drifted = catalog_descriptor.clone();
        drifted.required_capability.push_str(".drift");
        drifted_descriptors.push(("required_capability", drifted));
        let mut drifted = catalog_descriptor.clone();
        drifted.family = if catalog_descriptor.family == NativeOperationFamily::WorkspaceRead {
            NativeOperationFamily::WorkspaceSearch
        } else {
            NativeOperationFamily::WorkspaceRead
        };
        drifted_descriptors.push(("family", drifted));
        let mut drifted = catalog_descriptor.clone();
        drifted.availability = ToolAvailability::Disabled;
        drifted_descriptors.push(("availability", drifted));
        let mut drifted = catalog_descriptor.clone();
        drifted.input_limit_bytes = drifted.input_limit_bytes.saturating_add(1);
        drifted_descriptors.push(("input_limit_bytes", drifted));
        let mut drifted = catalog_descriptor.clone();
        drifted.output_limit_bytes = drifted.output_limit_bytes.saturating_add(1);
        drifted_descriptors.push(("output_limit_bytes", drifted));

        for (field, descriptor) in drifted_descriptors {
            let mut drifted_request = request.clone();
            drifted_request.descriptor = descriptor;
            assert!(
                matches!(
                    validate_native_tool_request(&drifted_request),
                    Err(NativeToolExecutionError::InvalidDescriptor(_))
                ),
                "{field} drift must fail for {:?}",
                catalog_descriptor.family
            );
        }
    }
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

struct DurableEffectFixture {
    database_path: PathBuf,
    store: Arc<SqliteAuthorityStore>,
    task_object_id: ObjectId,
    effect_object_id: ObjectId,
    admitted_at: WallTimestamp,
}

fn durable_effect_fixture(
    seed: u64,
    action: &str,
    target: &str,
    idempotency_key: &str,
    parameters_digest: &str,
    label: &str,
) -> DurableEffectFixture {
    let database_path = temporary_authority_database_path();
    let store = Arc::new(SqliteAuthorityStore::open(&database_path).expect("open authority store"));
    let task_object_id = object_id(seed);
    let effect_object_id = object_id(seed + 1);
    let intent_object_id = object_id(seed + 2);
    let admitted_at = WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid admission time");
    for (object_id, domain, lifecycle_state, event_seed) in [
        (
            task_object_id.clone(),
            LifecycleDomain::Task,
            "RUNNING",
            seed,
        ),
        (
            effect_object_id.clone(),
            LifecycleDomain::Effect,
            "PROPOSED",
            seed + 1,
        ),
    ] {
        store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: object_id.clone(),
                    domain,
                    state: state(lifecycle_state),
                    version: Version::INITIAL,
                    body: json!({"fixture": label}),
                },
                admitted_at: admitted_at.clone(),
                event: EventDraft {
                    event_id: EventId::parse(&format!("00000000-0000-7000-a000-{event_seed:012x}"))
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
    store
        .insert_intent(
            &IntentRow {
                intent_id: intent_object_id.clone(),
                idempotency_key: idempotency_key.to_owned(),
                parameters_digest: parameters_digest.to_owned(),
                action: action.to_owned(),
                target: target.to_owned(),
                effect_object_id: effect_object_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: None,
                canonical_json: format!("{{\"fixture\":\"{label}\"}}"),
            },
            &EventDraft {
                event_id: EventId::parse(&format!("00000000-0000-7000-a000-{:012x}", seed + 2))
                    .expect("valid intent event identifier"),
                object_id: intent_object_id,
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"intent\":\"fixture\"}".to_owned(),
            },
        )
        .expect("persist durable intent");
    DurableEffectFixture {
        database_path,
        store,
        task_object_id,
        effect_object_id,
        admitted_at,
    }
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
        Some(b"token=[REDACTED] 123456789".to_vec())
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
    assert_eq!(completed_output, b"token=[REDACTED] 123456789".to_vec());
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
        Some(b"token=[REDACTED] durable workspace output".to_vec())
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

// ---------------------------------------------------------------------------
// P2-T10/D01 — WorkspaceSearch executor
// ---------------------------------------------------------------------------

fn workspace_search_call(
    idempotency_key: &str,
    parameters_digest: &str,
    target: &str,
    fencing_epoch: i64,
) -> ExecutorCall {
    ExecutorCall {
        action: "search".to_owned(),
        target: target.to_owned(),
        idempotency_key: idempotency_key.to_owned(),
        parameters_digest: parameters_digest.to_owned(),
        authorization_digest: "authorization-digest".to_owned(),
        fencing_epoch,
    }
}

fn staged_search_request(
    workspace_root: &std::path::Path,
    target: &str,
    query: &[u8],
) -> ValidatedNativeToolRequest {
    let mut request = request_for(NativeOperationFamily::WorkspaceSearch);
    request.target = target.to_owned();
    request.input = query.to_vec();
    request.workspace_root = Some(workspace_root.to_path_buf());
    validate_native_tool_request(&request).expect("valid workspace search")
}

/// Simulates a lost response after the native search completed. Queries still
/// reach the real sink and its original idempotency key.
struct UnknownAfterNativeSearchDispatchExecutor<'executor> {
    native_executor: &'executor NativeWorkspaceSearchExecutor,
}

impl EffectExecutor for UnknownAfterNativeSearchDispatchExecutor<'_> {
    fn capabilities(&self) -> ExecutorCapabilities {
        self.native_executor.capabilities()
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        self.native_executor.dispatch(call)?;
        Ok(DispatchOutcome::Unknown {
            detail: "simulated lost post-scan response".to_owned(),
        })
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        self.native_executor.query_outcome(idempotency_key)
    }
}

#[test]
fn workspace_search_requires_a_staged_digest_bound_request_before_io() {
    let temporary_workspace = TestWorkspace::new("search-digest-binding");
    std::fs::create_dir_all(temporary_workspace.path.join("tree")).expect("create search tree");
    std::fs::write(
        temporary_workspace.path.join("tree/notes.txt"),
        "alpha needle beta\n",
    )
    .expect("write search fixture");
    let validated_request =
        staged_search_request(&temporary_workspace.path, "workspace://tree", b"needle");
    let executor = NativeWorkspaceSearchExecutor::new(7);
    executor
        .stage_request(
            "search-key-1".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        )
        .expect("stage daemon-bound search");

    let mismatched_call =
        workspace_search_call("search-key-1", "different-digest", "workspace://tree", 7);
    assert!(matches!(
        executor.dispatch(&mismatched_call),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(executor.completed_output("search-key-1"), None);
    assert_eq!(executor.scan_count(), 0);

    let unstaged_call =
        workspace_search_call("search-key-absent", "digest-1", "workspace://tree", 7);
    assert!(matches!(
        executor.dispatch(&unstaged_call),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(executor.scan_count(), 0);

    let matched_call = workspace_search_call("search-key-1", "digest-1", "workspace://tree", 7);
    assert!(matches!(
        executor.dispatch(&matched_call),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert_eq!(
        executor.completed_output("search-key-1"),
        Some(b"tree/notes.txt:1:alpha needle beta\n".to_vec())
    );
    assert_eq!(executor.scan_count(), 1);
}

#[test]
fn non_search_descriptor_cannot_be_staged_for_search_dispatch() {
    let mut request = request_for(NativeOperationFamily::WorkspaceRead);
    request.input.clear();
    let validated_request = validate_native_tool_request(&request).expect("valid read request");
    let executor = NativeWorkspaceSearchExecutor::new(7);

    assert_eq!(
        executor.stage_request(
            "read-key-1".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        ),
        Err(NativeToolExecutionError::UnsupportedExecutionFamily)
    );
    assert!(matches!(
        executor.dispatch(&workspace_search_call(
            "read-key-1",
            "digest-1",
            "workspace://notes/today.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
}

#[test]
fn workspace_search_rejects_an_empty_or_oversized_query_at_the_sink() {
    let temporary_workspace = TestWorkspace::new("search-query-bounds");
    std::fs::create_dir_all(temporary_workspace.path.join("tree")).expect("create search tree");
    let executor = NativeWorkspaceSearchExecutor::new(7);

    // The validator permits an empty payload for read-risk families, so the
    // sink is the boundary that must refuse an unbounded scan request.
    let empty_query = staged_search_request(&temporary_workspace.path, "workspace://tree", b"");
    assert!(matches!(
        executor.stage_request(
            "search-empty".to_owned(),
            "digest-1".to_owned(),
            &empty_query,
        ),
        Err(NativeToolExecutionError::InvalidDescriptor(_))
    ));

    let oversized_query = staged_search_request(
        &temporary_workspace.path,
        "workspace://tree",
        &b"n".repeat(cognitive_kernel::tool_registry::MAXIMUM_WORKSPACE_SEARCH_QUERY_BYTES + 1),
    );
    assert!(matches!(
        executor.stage_request(
            "search-oversized".to_owned(),
            "digest-1".to_owned(),
            &oversized_query,
        ),
        Err(NativeToolExecutionError::InvalidDescriptor(_))
    ));
    assert_eq!(executor.scan_count(), 0);
}

#[cfg(unix)]
#[test]
fn workspace_search_never_leaves_the_approved_root_through_a_symlink() {
    use std::os::unix::fs::symlink;

    let temporary_workspace = TestWorkspace::new("search-symlink-containment");
    std::fs::create_dir_all(temporary_workspace.path.join("tree")).expect("create search tree");
    std::fs::create_dir_all(temporary_workspace.path.join("outside")).expect("create outside tree");
    std::fs::write(
        temporary_workspace.path.join("tree/inside.txt"),
        "needle inside the root\n",
    )
    .expect("write contained fixture");
    std::fs::write(
        temporary_workspace.path.join("outside/secret.txt"),
        "needle outside the root\n",
    )
    .expect("write uncontained fixture");
    symlink(
        temporary_workspace.path.join("outside/secret.txt"),
        temporary_workspace.path.join("tree/linked.txt"),
    )
    .expect("plant a symlink inside the search tree");

    let approved_root = temporary_workspace.path.join("tree");
    let validated_request = staged_search_request(&approved_root, "workspace://", b"needle");
    let executor = NativeWorkspaceSearchExecutor::new(7);
    executor
        .stage_request(
            "search-symlink".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        )
        .expect("stage contained search");

    assert!(matches!(
        executor.dispatch(&workspace_search_call(
            "search-symlink",
            "digest-1",
            "workspace://",
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    let output = String::from_utf8(
        executor
            .completed_output("search-symlink")
            .expect("search retains bounded output"),
    )
    .expect("search output is UTF-8");
    assert!(output.contains("inside.txt:1:needle inside the root"));
    assert!(
        !output.contains("outside"),
        "a symlink must never be traversed out of the approved root: {output}"
    );

    // A staged search root that is itself a link out of the approved root is
    // refused after canonicalization, before any directory is read.
    symlink(
        temporary_workspace.path.join("outside"),
        temporary_workspace.path.join("tree/escape"),
    )
    .expect("plant an escaping search root");
    let escaping_request = staged_search_request(&approved_root, "workspace://escape", b"needle");
    executor
        .stage_request(
            "search-escape".to_owned(),
            "digest-2".to_owned(),
            &escaping_request,
        )
        .expect("stage escaping search root");
    assert!(matches!(
        executor.dispatch(&workspace_search_call(
            "search-escape",
            "digest-2",
            "workspace://escape",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(executor.completed_output("search-escape"), None);
}

#[cfg(any(unix, windows))]
#[test]
fn workspace_search_rejects_active_file_swap_to_a_link_or_reparse_point() {
    let temporary_workspace = TestWorkspace::new("search-active-file-swap");
    let tree = temporary_workspace.path.join("tree");
    let outside = temporary_workspace.path.join("outside");
    std::fs::create_dir_all(&tree).expect("create search tree");
    std::fs::create_dir_all(&outside).expect("create outside tree");
    let victim = tree.join("victim.txt");
    let outside_file = outside.join("secret.txt");
    std::fs::write(&victim, "needle safe\n").expect("write victim");
    std::fs::write(&outside_file, "needle outside\n").expect("write outside");
    let request = staged_search_request(&temporary_workspace.path, "workspace://tree", b"needle");
    let executor = NativeWorkspaceSearchExecutor::new(7);
    executor
        .stage_request(
            "search-active-file".to_owned(),
            "digest-1".to_owned(),
            &request,
        )
        .expect("stage search");
    executor.install_before_entry_open_hook(move |relative_path| {
        if relative_path.ends_with("victim.txt") {
            std::fs::remove_file(&victim).expect("remove victim before swap");
            create_test_file_link(&outside_file, &victim);
        }
    });

    assert!(matches!(
        executor.dispatch(&workspace_search_call(
            "search-active-file",
            "digest-1",
            "workspace://tree",
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    let output = String::from_utf8(
        executor
            .completed_output("search-active-file")
            .expect("search output"),
    )
    .expect("utf8");
    assert!(!output.contains("outside"));
}

#[cfg(any(unix, windows))]
#[test]
fn workspace_search_rejects_active_directory_swap_to_a_link_or_reparse_point() {
    let temporary_workspace = TestWorkspace::new("search-active-directory-swap");
    let tree = temporary_workspace.path.join("tree");
    let victim = tree.join("victim");
    let moved_victim = tree.join("victim-original");
    let outside = temporary_workspace.path.join("outside");
    std::fs::create_dir_all(&victim).expect("create victim directory");
    std::fs::create_dir_all(&outside).expect("create outside directory");
    std::fs::write(victim.join("inside.txt"), "needle safe\n").expect("write safe file");
    std::fs::write(outside.join("secret.txt"), "needle outside\n").expect("write outside file");
    let request = staged_search_request(&temporary_workspace.path, "workspace://tree", b"needle");
    let executor = NativeWorkspaceSearchExecutor::new(7);
    executor
        .stage_request(
            "search-active-directory".to_owned(),
            "digest-1".to_owned(),
            &request,
        )
        .expect("stage search");
    executor.install_before_entry_open_hook(move |relative_path| {
        if relative_path.ends_with("victim") {
            std::fs::rename(&victim, &moved_victim).expect("move victim directory");
            create_test_directory_link(&outside, &victim);
        }
    });

    assert!(matches!(
        executor.dispatch(&workspace_search_call(
            "search-active-directory",
            "digest-1",
            "workspace://tree",
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    let output = String::from_utf8(
        executor
            .completed_output("search-active-directory")
            .expect("search output"),
    )
    .expect("utf8");
    assert!(!output.contains("outside"));
}

#[test]
fn workspace_search_bounds_matches_and_redacts_before_retention() {
    let temporary_workspace = TestWorkspace::new("search-bounds");
    let tree = temporary_workspace.path.join("tree");
    std::fs::create_dir_all(&tree).expect("create search tree");
    for file_name in ["a.txt", "b.txt", "c.txt"] {
        std::fs::write(tree.join(file_name), "token=secret needle\n").expect("write fixture");
    }
    std::fs::write(tree.join("big.txt"), "needle\n".repeat(64)).expect("write oversized fixture");

    let validated_request =
        staged_search_request(&temporary_workspace.path, "workspace://tree", b"needle");
    let executor = NativeWorkspaceSearchExecutor::with_bounds(
        7,
        WorkspaceSearchBounds {
            maximum_visited_entries: 4096,
            maximum_matches: 2,
            maximum_file_bytes: 64,
            maximum_line_bytes: 512,
        },
    );
    executor
        .stage_request(
            "search-bounds".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        )
        .expect("stage bounded search");
    assert!(matches!(
        executor.dispatch(&workspace_search_call(
            "search-bounds",
            "digest-1",
            "workspace://tree",
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));

    let output = String::from_utf8(
        executor
            .completed_output("search-bounds")
            .expect("search retains bounded output"),
    )
    .expect("search output is UTF-8");
    assert_eq!(
        output.lines().count(),
        2,
        "the match ceiling must truncate the scan: {output}"
    );
    assert!(
        !output.contains("secret"),
        "sensitive values must be redacted before retention: {output}"
    );
    assert!(output.contains("token=[REDACTED]"));
    assert!(
        !output.contains("big.txt"),
        "a file over the size ceiling must be skipped, not read: {output}"
    );
}

#[test]
fn workspace_search_enforces_visit_ceiling_while_enumerating_a_huge_directory() {
    let temporary_workspace = TestWorkspace::new("search-enumeration-bound");
    let tree = temporary_workspace.path.join("tree");
    std::fs::create_dir_all(&tree).expect("create search tree");
    for index in 0..512 {
        std::fs::write(tree.join(format!("{index:04}.txt")), "no match\n")
            .expect("write oversized directory fixture");
    }
    let request = staged_search_request(&temporary_workspace.path, "workspace://tree", b"needle");
    let bounds = WorkspaceSearchBounds {
        maximum_visited_entries: 5,
        maximum_matches: 5,
        maximum_file_bytes: 64,
        maximum_line_bytes: 64,
    };
    let executor = NativeWorkspaceSearchExecutor::with_bounds(7, bounds);
    executor
        .stage_request(
            "search-enumeration-bound".to_owned(),
            "digest-1".to_owned(),
            &request,
        )
        .expect("stage search");
    assert!(matches!(
        executor.dispatch(&workspace_search_call(
            "search-enumeration-bound",
            "digest-1",
            "workspace://tree",
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert!(
        executor.enumerated_entry_count() < bounds.maximum_visited_entries,
        "directory enumeration itself must stop at the remaining visit budget"
    );
}

#[test]
fn workspace_search_sink_rejects_stale_fencing_before_io() {
    let executor = NativeWorkspaceSearchExecutor::new(7);
    let call = workspace_search_call("search-key-3", "digest-3", "workspace://tree", 6);
    assert_eq!(
        executor.dispatch(&call),
        Ok(DispatchOutcome::FencedStaleEpoch { sink_epoch: 7 })
    );
    assert_eq!(executor.completed_output("search-key-3"), None);
    assert_eq!(executor.scan_count(), 0);
}

#[test]
fn durable_workspace_search_dispatch_records_executing_before_io_without_advancing_task() {
    let temporary_workspace = TestWorkspace::new("durable-search-dispatch");
    let tree = temporary_workspace.path.join("tree");
    std::fs::create_dir_all(&tree).expect("create search tree");
    std::fs::write(tree.join("notes.txt"), "token=secret needle\n").expect("write fixture");
    let database_path = temporary_authority_database_path();
    let store = Arc::new(SqliteAuthorityStore::open(&database_path).expect("open authority store"));
    let task_object_id = object_id(601);
    let effect_object_id = object_id(602);
    let intent_object_id = object_id(603);
    let admitted_at = WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid admission time");

    for (object_id, domain, lifecycle_state, event_id) in [
        (
            task_object_id.clone(),
            LifecycleDomain::Task,
            "RUNNING",
            601,
        ),
        (
            effect_object_id.clone(),
            LifecycleDomain::Effect,
            "PROPOSED",
            602,
        ),
    ] {
        store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: object_id.clone(),
                    domain,
                    state: state(lifecycle_state),
                    version: Version::INITIAL,
                    body: json!({"fixture": "p2-t10-d01"}),
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
    let idempotency_key = "p2-t10-d01-workspace-search";
    let parameters_digest = "sha256:p2-t10-d01-workspace-search";
    store
        .insert_intent(
            &IntentRow {
                intent_id: intent_object_id.clone(),
                idempotency_key: idempotency_key.to_owned(),
                parameters_digest: parameters_digest.to_owned(),
                action: "search".to_owned(),
                target: "workspace://tree".to_owned(),
                effect_object_id: effect_object_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: None,
                canonical_json: "{\"intent\":\"p2-t10-d01\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000603")
                    .expect("valid intent event identifier"),
                object_id: intent_object_id,
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"intent\":\"p2-t10-d01\"}".to_owned(),
            },
        )
        .expect("persist durable intent");

    let validated_request =
        staged_search_request(&temporary_workspace.path, "workspace://tree", b"needle");
    let executor = NativeWorkspaceSearchExecutor::new(1);
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
    executor.install_before_search_hook(move || {
        let effect = hook_store
            .load_object(LifecycleDomain::Effect, &hook_effect_object_id)
            .expect("load effect before scan")
            .expect("durable effect exists");
        let task = hook_store
            .load_object(LifecycleDomain::Task, &hook_task_object_id)
            .expect("load task before scan")
            .expect("durable task exists");
        assert_eq!(effect.state.as_str(), "EXECUTING");
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
        UriRef::parse("correlation://personal/p2-t10-d01").expect("valid correlation reference"),
    );
    let grant = effect_grant();
    let governance_currency = GovernanceCurrency {
        revocation_epoch: 1,
        capability_set_version: 1,
    };
    let writer_lease = WriterLease { epoch: 1 };

    dispatch_staged_workspace_search_effect(
        &effect_protocol,
        &effect_object_id,
        Version::INITIAL,
        &grant,
        &governance_currency,
        &executor,
        &writer_lease,
    )
    .expect("dispatch durable workspace search");

    assert_eq!(
        executor.completed_output(idempotency_key),
        Some(b"tree/notes.txt:1:token=[REDACTED] needle\n".to_vec())
    );
    assert_eq!(executor.scan_count(), 1);
    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &effect_object_id)
            .expect("load effect after scan")
            .expect("durable effect exists")
            .state
            .as_str(),
        "EXECUTED"
    );
    assert_eq!(
        store
            .load_object(LifecycleDomain::Task, &task_object_id)
            .expect("load task after scan")
            .expect("durable task exists")
            .version,
        Version::INITIAL
    );

    std::fs::remove_file(database_path).unwrap_or(());
}

#[test]
fn unknown_native_workspace_search_reconciles_original_key_without_second_scan() {
    let temporary_workspace = TestWorkspace::new("search-unknown-outcome");
    let tree = temporary_workspace.path.join("tree");
    std::fs::create_dir_all(&tree).expect("create search tree");
    std::fs::write(tree.join("notes.txt"), "needle\n").expect("write fixture");
    let database_path = temporary_authority_database_path();
    let store = Arc::new(SqliteAuthorityStore::open(&database_path).expect("open authority store"));
    let task_object_id = object_id(611);
    let effect_object_id = object_id(612);
    let intent_object_id = object_id(613);
    let admitted_at = WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid admission time");

    for (object_id, domain, lifecycle_state, event_id) in [
        (
            task_object_id.clone(),
            LifecycleDomain::Task,
            "RUNNING",
            611,
        ),
        (
            effect_object_id.clone(),
            LifecycleDomain::Effect,
            "PROPOSED",
            612,
        ),
    ] {
        store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: object_id.clone(),
                    domain,
                    state: state(lifecycle_state),
                    version: Version::INITIAL,
                    body: json!({"fixture": "p2-t10-d01-unknown"}),
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
    let idempotency_key = "p2-t10-d01-workspace-search-unknown";
    let parameters_digest = "sha256:p2-t10-d01-workspace-search-unknown";
    store
        .insert_intent(
            &IntentRow {
                intent_id: intent_object_id.clone(),
                idempotency_key: idempotency_key.to_owned(),
                parameters_digest: parameters_digest.to_owned(),
                action: "search".to_owned(),
                target: "workspace://tree".to_owned(),
                effect_object_id: effect_object_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: None,
                canonical_json: "{\"intent\":\"p2-t10-d01-unknown\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000613")
                    .expect("valid intent event identifier"),
                object_id: intent_object_id,
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"intent\":\"p2-t10-d01-unknown\"}".to_owned(),
            },
        )
        .expect("persist durable intent");

    let validated_request =
        staged_search_request(&temporary_workspace.path, "workspace://tree", b"needle");
    let native_executor = NativeWorkspaceSearchExecutor::new(1);
    native_executor
        .stage_request(
            idempotency_key.to_owned(),
            parameters_digest.to_owned(),
            &validated_request,
        )
        .expect("stage durable intent identity");
    let unknown_executor = UnknownAfterNativeSearchDispatchExecutor {
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
        UriRef::parse("correlation://personal/p2-t10-d01-unknown")
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
        .expect("dispatch native search through lost-response wrapper");
    assert!(matches!(outcome, DispatchOutcome::Unknown { .. }));
    let unknown = effect_protocol
        .record_outcome(
            &effect_object_id,
            dispatched.after_version,
            &outcome,
            &writer_lease,
        )
        .expect("record unknown post-scan outcome");
    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &effect_object_id)
            .expect("load unknown effect")
            .expect("durable effect exists")
            .state
            .as_str(),
        "OUTCOME_UNKNOWN"
    );
    assert_eq!(native_executor.scan_count(), 1);

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
    assert_eq!(
        native_executor.scan_count(),
        1,
        "reconciliation must query the original key, never rescan"
    );
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

// ---------------------------------------------------------------------------
// P2-T10/D02 — WorkspaceWrite / WorkspacePatch mutation executor
// ---------------------------------------------------------------------------

fn image_digest(bytes: &[u8]) -> String {
    workspace_image_digest(bytes).expect("workspace image digest")
}

fn workspace_mutation_call(
    action: &str,
    idempotency_key: &str,
    parameters_digest: &str,
    target: &str,
    fencing_epoch: i64,
) -> ExecutorCall {
    ExecutorCall {
        action: action.to_owned(),
        target: target.to_owned(),
        idempotency_key: idempotency_key.to_owned(),
        parameters_digest: parameters_digest.to_owned(),
        authorization_digest: "authorization-digest".to_owned(),
        fencing_epoch,
    }
}

fn staged_mutation_request(
    family: NativeOperationFamily,
    workspace_root: &std::path::Path,
    target: &str,
    payload: &[u8],
    expected_preimage: WorkspacePreimage,
) -> ValidatedNativeToolRequest {
    let mut request = request_for(family);
    request.target = target.to_owned();
    request.input = payload.to_vec();
    request.workspace_root = Some(workspace_root.to_path_buf());
    request.expected_preimage = Some(expected_preimage);
    validate_native_tool_request(&request).expect("valid workspace mutation")
}

/// Names in the target directory, so a test can prove no staging residue.
fn directory_entry_names(directory: &std::path::Path) -> Vec<String> {
    let mut names = std::fs::read_dir(directory)
        .expect("read target directory")
        .map(|entry| {
            entry
                .expect("directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name != ".executor-state")
        .collect::<Vec<_>>();
    names.sort();
    names
}

struct UnknownAfterNativeMutationDispatchExecutor<'executor> {
    native_executor: &'executor NativeWorkspaceMutationExecutor,
}

impl EffectExecutor for UnknownAfterNativeMutationDispatchExecutor<'_> {
    fn capabilities(&self) -> ExecutorCapabilities {
        self.native_executor.capabilities()
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        self.native_executor.dispatch(call)?;
        Ok(DispatchOutcome::Unknown {
            detail: "simulated lost post-publication response".to_owned(),
        })
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        self.native_executor.query_outcome(idempotency_key)
    }
}

#[test]
fn workspace_mutation_cannot_be_validated_without_an_expected_preimage() {
    for family in [
        NativeOperationFamily::WorkspaceWrite,
        NativeOperationFamily::WorkspacePatch,
    ] {
        let mut request = request_for(family);
        request.input = b"@@ -1 +1 @@\n-old\n+new\n".to_vec();
        request.expected_preimage = None;
        assert_eq!(
            validate_native_tool_request(&request),
            Err(NativeToolExecutionError::MutationPreimageRequired),
            "{family:?} must declare the state it replaces"
        );
    }
}

#[test]
fn workspace_mutation_rejects_a_receipt_store_inside_the_approved_workspace() {
    let temporary_workspace = TestWorkspace::new("write-state-isolation");
    let state_store = Arc::new(
        DurableExecutorStateStore::open(&temporary_workspace.path.join(".unsafe-state"))
            .expect("open intentionally unsafe state location"),
    );
    let executor = NativeWorkspaceMutationExecutor::new(7, state_store);
    let request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"after\n",
        WorkspacePreimage::Absent,
    );
    assert!(matches!(
        executor.stage_request(
            "write-unsafe-state".to_owned(),
            "digest-1".to_owned(),
            &request,
        ),
        Err(NativeToolExecutionError::ExecutorUnavailable(_))
    ));
}

#[cfg(any(unix, windows))]
#[test]
fn durable_executor_state_creation_never_follows_a_link_or_reparse_point() {
    let temporary_workspace = TestWorkspace::new("state-root-link");
    let outside = temporary_workspace.path.join("outside");
    let linked_state = temporary_workspace.path.join("linked-state");
    std::fs::create_dir_all(&outside).expect("create outside state target");
    create_test_directory_link(&outside, &linked_state);

    assert!(
        DurableExecutorStateStore::open(&linked_state.join("nested")).is_err(),
        "state construction must reject a linked path component"
    );
    assert!(
        !outside.join("nested").exists(),
        "rejected state construction must not create through the link"
    );
}

#[test]
fn workspace_write_refuses_a_preimage_mismatch_without_touching_the_target() {
    let temporary_workspace = TestWorkspace::new("write-preimage-mismatch");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "current content\n").expect("write fixture");

    let validated_request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"replacement content\n",
        WorkspacePreimage::Digest(image_digest(b"a different preimage\n")),
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "write-mismatch".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        )
        .expect("stage mutation");

    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-mismatch",
            "digest-1",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(&target_path).expect("target still readable"),
        "current content\n"
    );
    assert_eq!(executor.publish_count(), 0);
    assert_eq!(
        directory_entry_names(&temporary_workspace.path),
        vec!["notes.txt".to_owned()],
        "a refused mutation must leave no staging residue"
    );
    assert_eq!(executor.completed_output("write-mismatch"), None);
}

#[test]
fn workspace_write_publishes_atomically_and_leaves_no_staging_residue() {
    let temporary_workspace = TestWorkspace::new("write-atomic-publish");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "before\n").expect("write fixture");

    let validated_request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"after\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "write-atomic".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        )
        .expect("stage mutation");
    executor.install_after_staging_write_hook(|target, staging| {
        // The postimage is fully on disk in the staging file while the target
        // still holds the preimage: a concurrent reader cannot observe a
        // partially written target.
        assert!(
            staging.is_file(),
            "staging file must exist before the rename"
        );
        assert_eq!(
            std::fs::read_to_string(staging).expect("staging readable"),
            "after\n"
        );
        assert_eq!(
            std::fs::read_to_string(target).expect("target readable"),
            "before\n"
        );
    });

    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-atomic",
            "digest-1",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(&target_path).expect("target readable"),
        "after\n"
    );
    assert_eq!(executor.publish_count(), 1);
    assert_eq!(
        directory_entry_names(&temporary_workspace.path),
        vec!["notes.txt".to_owned()],
        "a published mutation must leave no staging residue"
    );
    let receipt = String::from_utf8(
        executor
            .completed_output("write-atomic")
            .expect("mutation retains a bounded receipt"),
    )
    .expect("receipt is UTF-8");
    assert!(receipt.starts_with("workspace://notes.txt:write:6:sha256:"));
    assert!(
        !receipt.contains("after"),
        "the receipt must record what changed, never the bytes: {receipt}"
    );
}

#[test]
fn workspace_mutation_refuses_a_target_that_changed_before_publication() {
    let temporary_workspace = TestWorkspace::new("write-publication-race");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "before\n").expect("write fixture");

    let validated_request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"after\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "write-race".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        )
        .expect("stage mutation");
    executor.install_after_staging_write_hook(|target, _staging| {
        // A concurrent writer wins the race between the preimage check and the
        // rename. The mutation must not clobber it.
        std::fs::write(target, "concurrent writer\n").expect("concurrent write");
    });

    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-race",
            "digest-1",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(&target_path).expect("target readable"),
        "concurrent writer\n"
    );
    assert_eq!(executor.publish_count(), 0);
    assert_eq!(
        directory_entry_names(&temporary_workspace.path),
        vec!["notes.txt".to_owned()]
    );
}

#[test]
fn workspace_mutation_target_lock_closes_the_final_check_to_rename_window() {
    let temporary_workspace = TestWorkspace::new("write-final-cas-window");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "before\n").expect("write fixture");
    let primary_request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"primary\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let competitor_request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"competitor\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let primary = mutation_executor(7, &temporary_workspace);
    primary
        .stage_request(
            "write-primary".to_owned(),
            "digest-primary".to_owned(),
            &primary_request,
        )
        .expect("stage primary");
    let competitor = mutation_executor(7, &temporary_workspace);
    competitor
        .stage_request(
            "write-competitor".to_owned(),
            "digest-competitor".to_owned(),
            &competitor_request,
        )
        .expect("stage competitor");
    let competitor_outcome = Arc::new(Mutex::new(None));
    let observed_outcome = Arc::clone(&competitor_outcome);
    primary.install_after_final_preimage_check_hook(move || {
        let outcome = competitor
            .dispatch(&workspace_mutation_call(
                "write",
                "write-competitor",
                "digest-competitor",
                "workspace://notes.txt",
                7,
            ))
            .expect("competitor lock result");
        *observed_outcome.lock().expect("outcome lock") = Some(outcome);
    });

    assert!(matches!(
        primary.dispatch(&workspace_mutation_call(
            "write",
            "write-primary",
            "digest-primary",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert!(matches!(
        competitor_outcome.lock().expect("outcome lock").as_ref(),
        Some(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(target_path).expect("target readable"),
        "primary\n",
        "the losing mutation must not overwrite after the winner's final CAS check"
    );
}

#[test]
fn workspace_mutation_rechecks_an_uncooperative_write_in_the_final_rename_window() {
    let temporary_workspace = TestWorkspace::new("write-final-window-recheck");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "before\n").expect("write fixture");
    let request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"after\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "write-final-window".to_owned(),
            "digest-1".to_owned(),
            &request,
        )
        .expect("stage mutation");
    let competing_target = target_path.clone();
    executor.install_after_final_preimage_check_hook(move || {
        std::fs::write(&competing_target, "uncooperative writer\n")
            .expect("write in final CAS window");
    });

    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-final-window",
            "digest-1",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(target_path).expect("target readable"),
        "uncooperative writer\n"
    );
    assert_eq!(executor.publish_count(), 0);
}

#[cfg(unix)]
#[test]
fn workspace_mutation_anchors_parent_against_an_active_directory_swap() {
    let temporary_workspace = TestWorkspace::new("write-active-parent-swap");
    let approved_root = temporary_workspace.path.join("root");
    let target_parent = approved_root.join("work");
    let moved_parent = approved_root.join("work-original");
    let outside = temporary_workspace.path.join("outside");
    std::fs::create_dir_all(&target_parent).expect("create target parent");
    std::fs::create_dir_all(&outside).expect("create outside directory");
    std::fs::write(target_parent.join("notes.txt"), "before\n").expect("write target");
    let request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &approved_root,
        "workspace://work/notes.txt",
        b"after\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "write-parent-swap".to_owned(),
            "digest-1".to_owned(),
            &request,
        )
        .expect("stage mutation");
    executor.install_after_staging_write_hook(move |_target, _staging| {
        std::fs::rename(&target_parent, &moved_parent).expect("swap target parent");
        create_test_directory_link(&outside, &target_parent);
    });

    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-parent-swap",
            "digest-1",
            "workspace://work/notes.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert!(
        !temporary_workspace.path.join("outside/notes.txt").exists(),
        "a swapped pathname must never redirect handle-relative publication"
    );
    assert_eq!(
        std::fs::read_to_string(
            temporary_workspace
                .path
                .join("root/work-original/notes.txt")
        )
        .expect("original target remains"),
        "before\n"
    );
}

#[cfg(any(unix, windows))]
#[test]
fn workspace_mutation_refuses_a_symlinked_target_and_an_escaping_parent() {
    let temporary_workspace = TestWorkspace::new("write-symlink-refusal");
    let approved_root = temporary_workspace.path.join("root");
    std::fs::create_dir_all(&approved_root).expect("create approved root");
    std::fs::create_dir_all(temporary_workspace.path.join("outside")).expect("create outside tree");
    std::fs::write(
        temporary_workspace.path.join("outside/secret.txt"),
        "outside content\n",
    )
    .expect("write outside fixture");
    create_test_file_link(
        &temporary_workspace.path.join("outside/secret.txt"),
        &approved_root.join("linked.txt"),
    );
    create_test_directory_link(
        &temporary_workspace.path.join("outside"),
        &approved_root.join("escape"),
    );

    let executor = mutation_executor(7, &temporary_workspace);
    let linked_request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &approved_root,
        "workspace://linked.txt",
        b"clobber\n",
        WorkspacePreimage::Digest(image_digest(b"outside content\n")),
    );
    executor
        .stage_request(
            "write-linked".to_owned(),
            "digest-1".to_owned(),
            &linked_request,
        )
        .expect("stage linked mutation");
    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-linked",
            "digest-1",
            "workspace://linked.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));

    let escaping_request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &approved_root,
        "workspace://escape/planted.txt",
        b"clobber\n",
        WorkspacePreimage::Absent,
    );
    executor
        .stage_request(
            "write-escape".to_owned(),
            "digest-2".to_owned(),
            &escaping_request,
        )
        .expect("stage escaping mutation");
    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-escape",
            "digest-2",
            "workspace://escape/planted.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));

    assert_eq!(
        std::fs::read_to_string(temporary_workspace.path.join("outside/secret.txt"))
            .expect("outside fixture readable"),
        "outside content\n",
        "a symlinked target must never be written through"
    );
    assert!(
        !temporary_workspace
            .path
            .join("outside/planted.txt")
            .exists(),
        "an escaping parent must never receive a new file"
    );
    assert_eq!(executor.publish_count(), 0);
}

#[test]
fn duplicate_workspace_write_dispatch_publishes_exactly_once() {
    let temporary_workspace = TestWorkspace::new("write-duplicate-dispatch");
    let target_path = temporary_workspace.path.join("notes.txt");

    let validated_request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"created\n",
        WorkspacePreimage::Absent,
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "write-once".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        )
        .expect("stage mutation");
    let call = workspace_mutation_call(
        "write",
        "write-once",
        "digest-1",
        "workspace://notes.txt",
        7,
    );

    let first_outcome = executor.dispatch(&call).expect("first publication");
    let second_outcome = executor.dispatch(&call).expect("absorbed duplicate");
    assert_eq!(first_outcome, second_outcome);
    assert_eq!(
        executor.publish_count(),
        1,
        "one original key must publish exactly once"
    );
    assert_eq!(
        std::fs::read_to_string(&target_path).expect("target readable"),
        "created\n"
    );
    assert_eq!(
        executor.query_outcome("write-once"),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
}

#[test]
fn workspace_patch_applies_only_when_every_context_line_matches() {
    let temporary_workspace = TestWorkspace::new("patch-context");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "alpha\nbeta\ngamma\n").expect("write fixture");
    let preimage_digest = image_digest(b"alpha\nbeta\ngamma\n");
    let executor = mutation_executor(7, &temporary_workspace);

    // A patch whose context does not match the preimage must fail closed.
    let drifted_request = staged_mutation_request(
        NativeOperationFamily::WorkspacePatch,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"@@ -1,3 +1,3 @@\n alpha\n-DRIFTED\n+delta\n gamma\n",
        WorkspacePreimage::Digest(preimage_digest.clone()),
    );
    executor
        .stage_request(
            "patch-drift".to_owned(),
            "digest-1".to_owned(),
            &drifted_request,
        )
        .expect("stage drifted patch");
    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "patch",
            "patch-drift",
            "digest-1",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(&target_path).expect("target readable"),
        "alpha\nbeta\ngamma\n"
    );

    // A hunk header that does not describe its own body must fail closed too.
    let miscounted_request = staged_mutation_request(
        NativeOperationFamily::WorkspacePatch,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"@@ -1,9 +1,3 @@\n alpha\n-beta\n+delta\n gamma\n",
        WorkspacePreimage::Digest(preimage_digest.clone()),
    );
    executor
        .stage_request(
            "patch-miscounted".to_owned(),
            "digest-2".to_owned(),
            &miscounted_request,
        )
        .expect("stage miscounted patch");
    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "patch",
            "patch-miscounted",
            "digest-2",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(&target_path).expect("target readable"),
        "alpha\nbeta\ngamma\n"
    );
    assert_eq!(executor.publish_count(), 0);

    // The matching patch applies through the same atomic publish.
    let matching_request = staged_mutation_request(
        NativeOperationFamily::WorkspacePatch,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"@@ -1,3 +1,3 @@\n alpha\n-beta\n+delta\n gamma\n",
        WorkspacePreimage::Digest(preimage_digest),
    );
    executor
        .stage_request(
            "patch-apply".to_owned(),
            "digest-3".to_owned(),
            &matching_request,
        )
        .expect("stage matching patch");
    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "patch",
            "patch-apply",
            "digest-3",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(&target_path).expect("target readable"),
        "alpha\ndelta\ngamma\n"
    );
    assert_eq!(executor.publish_count(), 1);
    assert_eq!(
        directory_entry_names(&temporary_workspace.path),
        vec!["notes.txt".to_owned()]
    );
}

#[test]
fn workspace_patch_honors_old_and_new_no_newline_markers() {
    assert_eq!(
        apply_unified_patch(
            "final line",
            "@@ -1 +1 @@\n-final line\n\\ No newline at end of file\n+final line\n",
        ),
        Ok("final line\n".to_owned()),
        "an old-side marker permits adding the final newline"
    );
    assert_eq!(
        apply_unified_patch(
            "final line\n",
            "@@ -1 +1 @@\n-final line\n+final line\n\\ No newline at end of file\n",
        ),
        Ok("final line".to_owned()),
        "a new-side marker removes the final newline"
    );
    assert!(
        apply_unified_patch(
            "final line\n",
            "@@ -1 +1 @@\n-final line\n\\ No newline at end of file\n+changed\n",
        )
        .is_err(),
        "an old-side marker that contradicts the preimage must fail closed"
    );
}

#[test]
fn workspace_patch_rejects_sparse_preimages_over_the_explicit_ceiling() {
    let temporary_workspace = TestWorkspace::new("patch-preimage-ceiling");
    let target_path = temporary_workspace.path.join("sparse.txt");
    let target = std::fs::File::create(&target_path).expect("create sparse fixture");
    target
        .set_len(MAXIMUM_WORKSPACE_PATCH_PREIMAGE_BYTES + 1)
        .expect("extend sparse fixture");
    let request = staged_mutation_request(
        NativeOperationFamily::WorkspacePatch,
        &temporary_workspace.path,
        "workspace://sparse.txt",
        b"@@ -1 +1 @@\n-\n+\n",
        WorkspacePreimage::Digest(image_digest(b"")),
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "patch-over-limit".to_owned(),
            "digest-1".to_owned(),
            &request,
        )
        .expect("stage patch");
    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "patch",
            "patch-over-limit",
            "digest-1",
            "workspace://sparse.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(
        std::fs::metadata(target_path)
            .expect("sparse metadata")
            .len(),
        MAXIMUM_WORKSPACE_PATCH_PREIMAGE_BYTES + 1
    );
}

#[test]
fn workspace_write_streams_a_large_sparse_preimage_without_retaining_it() {
    let temporary_workspace = TestWorkspace::new("write-streamed-preimage");
    let target_path = temporary_workspace.path.join("sparse.txt");
    let target = std::fs::File::create(&target_path).expect("create sparse fixture");
    let sparse_size = MAXIMUM_WORKSPACE_PATCH_PREIMAGE_BYTES * 2;
    target.set_len(sparse_size).expect("extend sparse fixture");
    let request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://sparse.txt",
        b"replacement\n",
        WorkspacePreimage::Digest(image_digest(b"not-the-sparse-file")),
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "write-streamed-preimage".to_owned(),
            "digest-1".to_owned(),
            &request,
        )
        .expect("stage write");
    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-streamed-preimage",
            "digest-1",
            "workspace://sparse.txt",
            7,
        )),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(
        std::fs::metadata(target_path)
            .expect("sparse metadata")
            .len(),
        sparse_size
    );
}

#[test]
fn workspace_mutation_query_outcome_reconciles_from_durable_target_state() {
    let temporary_workspace = TestWorkspace::new("write-durable-query");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "before\n").expect("write fixture");
    let build_request = || {
        staged_mutation_request(
            NativeOperationFamily::WorkspaceWrite,
            &temporary_workspace.path,
            "workspace://notes.txt",
            b"after\n",
            WorkspacePreimage::Digest(image_digest(b"before\n")),
        )
    };

    // Before the mutation runs, a restarted daemon that re-stages the same
    // Intent must read `NotExecuted` from the filesystem alone.
    let restarted_before = mutation_executor(7, &temporary_workspace);
    restarted_before
        .stage_request(
            "write-durable".to_owned(),
            "digest-1".to_owned(),
            &build_request(),
        )
        .expect("stage mutation");
    assert_eq!(
        restarted_before.query_outcome("write-durable"),
        Ok(ExecutorQueryResult::NotExecuted)
    );
    // An unstaged key is never claimed either way.
    assert_eq!(
        restarted_before.query_outcome("write-unknown-key"),
        Ok(ExecutorQueryResult::Indeterminate)
    );

    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "write-durable".to_owned(),
            "digest-1".to_owned(),
            &build_request(),
        )
        .expect("stage mutation");
    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-durable",
            "digest-1",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));

    // After the mutation, a fresh executor with an empty completed ledger --
    // the state a restarted daemon is in -- still resolves the original key
    // from the durable target.
    let restarted_after = mutation_executor(7, &temporary_workspace);
    restarted_after
        .stage_request(
            "write-durable".to_owned(),
            "digest-1".to_owned(),
            &build_request(),
        )
        .expect("stage mutation");
    assert_eq!(restarted_after.publish_count(), 0);
    assert_eq!(
        restarted_after.query_outcome("write-durable"),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
}

#[test]
fn workspace_mutation_never_attributes_a_competitors_same_postimage_to_the_original_key() {
    let temporary_workspace = TestWorkspace::new("write-same-postimage-competitor");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "before\n").expect("write fixture");
    let request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"after\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "write-original-key".to_owned(),
            "digest-1".to_owned(),
            &request,
        )
        .expect("stage original key");
    std::fs::write(&target_path, "after\n").expect("competitor writes same postimage");

    assert_eq!(
        executor.query_outcome("write-original-key"),
        Ok(ExecutorQueryResult::Indeterminate),
        "matching bytes without a completed key-bound receipt prove nothing"
    );
}

#[test]
fn workspace_mutation_receipt_survives_restart_and_post_execution_reversion() {
    let temporary_workspace = TestWorkspace::new("write-reverted-after-execution");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "before\n").expect("write fixture");
    let request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"after\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request("write-reverted".to_owned(), "digest-1".to_owned(), &request)
        .expect("stage mutation");
    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-reverted",
            "digest-1",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    std::fs::write(&target_path, "before\n").expect("revert target after execution");

    let restarted = mutation_executor(7, &temporary_workspace);
    restarted
        .stage_request("write-reverted".to_owned(), "digest-1".to_owned(), &request)
        .expect("restage durable intent");
    assert_eq!(
        restarted.query_outcome("write-reverted"),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey),
        "the durable receipt proves execution even when later state differs"
    );
}

#[test]
fn workspace_mutation_missing_seen_state_cannot_be_recreated_as_not_executed() {
    let temporary_workspace = TestWorkspace::new("write-missing-seen-state");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "before\n").expect("write fixture");
    let request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"after\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "write-missing-state".to_owned(),
            "digest-1".to_owned(),
            &request,
        )
        .expect("stage mutation");
    assert!(matches!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-missing-state",
            "digest-1",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert!(
        executor
            .remove_durable_state("write-missing-state")
            .expect("remove test state")
    );

    let restarted = mutation_executor(7, &temporary_workspace);
    assert!(matches!(
        restarted.stage_request(
            "write-missing-state".to_owned(),
            "digest-1".to_owned(),
            &request,
        ),
        Err(NativeToolExecutionError::ExecutorUnavailable(_))
    ));
    assert_eq!(
        restarted.query_outcome("write-missing-state"),
        Ok(ExecutorQueryResult::Indeterminate)
    );
}

#[test]
fn workspace_mutation_restart_cleans_a_durable_orphan_staging_file() {
    let temporary_workspace = TestWorkspace::new("write-orphan-recovery");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "before\n").expect("write fixture");
    let request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"after\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let crashing = mutation_executor(7, &temporary_workspace);
    crashing
        .stage_request("write-orphan".to_owned(), "digest-1".to_owned(), &request)
        .expect("stage mutation");
    let staging_path = Arc::new(Mutex::new(None::<PathBuf>));
    let observed_staging = Arc::clone(&staging_path);
    crashing.install_after_staging_write_hook(move |_target, staging| {
        *observed_staging.lock().expect("staging path lock") = Some(staging.to_path_buf());
        panic!("simulated crash after durable staging write");
    });
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = crashing.dispatch(&workspace_mutation_call(
                "write",
                "write-orphan",
                "digest-1",
                "workspace://notes.txt",
                7,
            ));
        }))
        .is_err()
    );
    let staging_path = staging_path
        .lock()
        .expect("staging path lock")
        .clone()
        .expect("staging path observed");
    assert!(staging_path.is_file(), "crash must leave a real orphan");

    let restarted = mutation_executor(7, &temporary_workspace);
    restarted
        .stage_request("write-orphan".to_owned(), "digest-1".to_owned(), &request)
        .expect("restage durable intent");
    assert!(matches!(
        restarted.dispatch(&workspace_mutation_call(
            "write",
            "write-orphan",
            "digest-1",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::Unknown { .. })
    ));
    assert!(!staging_path.exists(), "restart must clean the orphan");
    assert_eq!(
        restarted.query_outcome("write-orphan"),
        Ok(ExecutorQueryResult::Indeterminate)
    );
}

#[test]
fn workspace_mutation_cleanup_failure_stays_unknown_and_indeterminate() {
    let temporary_workspace = TestWorkspace::new("write-orphan-cleanup-failure");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "before\n").expect("write fixture");
    let request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"after\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let crashing = mutation_executor(7, &temporary_workspace);
    crashing
        .stage_request(
            "write-cleanup-failure".to_owned(),
            "digest-1".to_owned(),
            &request,
        )
        .expect("stage mutation");
    let staging_path = Arc::new(Mutex::new(None::<PathBuf>));
    let observed_staging = Arc::clone(&staging_path);
    crashing.install_after_staging_write_hook(move |_target, staging| {
        std::fs::remove_file(staging).expect("replace staging file");
        std::fs::create_dir(staging).expect("plant unremovable staging directory");
        *observed_staging.lock().expect("staging path lock") = Some(staging.to_path_buf());
        panic!("simulated crash with hostile staging residue");
    });
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = crashing.dispatch(&workspace_mutation_call(
                "write",
                "write-cleanup-failure",
                "digest-1",
                "workspace://notes.txt",
                7,
            ));
        }))
        .is_err()
    );
    let staging_path = staging_path
        .lock()
        .expect("staging path lock")
        .clone()
        .expect("staging path observed");

    let restarted = mutation_executor(7, &temporary_workspace);
    restarted
        .stage_request(
            "write-cleanup-failure".to_owned(),
            "digest-1".to_owned(),
            &request,
        )
        .expect("restage durable intent");
    assert!(matches!(
        restarted.dispatch(&workspace_mutation_call(
            "write",
            "write-cleanup-failure",
            "digest-1",
            "workspace://notes.txt",
            7,
        )),
        Ok(DispatchOutcome::Unknown { .. })
    ));
    assert!(staging_path.is_dir(), "failed cleanup must not be hidden");
    assert_eq!(
        restarted.query_outcome("write-cleanup-failure"),
        Ok(ExecutorQueryResult::Indeterminate)
    );
}

#[test]
fn workspace_mutation_sink_rejects_stale_fencing_before_any_write() {
    let temporary_workspace = TestWorkspace::new("write-stale-fencing");
    let validated_request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"created\n",
        WorkspacePreimage::Absent,
    );
    let executor = mutation_executor(7, &temporary_workspace);
    executor
        .stage_request(
            "write-fenced".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        )
        .expect("stage mutation");

    assert_eq!(
        executor.dispatch(&workspace_mutation_call(
            "write",
            "write-fenced",
            "digest-1",
            "workspace://notes.txt",
            6,
        )),
        Ok(DispatchOutcome::FencedStaleEpoch { sink_epoch: 7 })
    );
    assert_eq!(executor.publish_count(), 0);
    assert!(!temporary_workspace.path.join("notes.txt").exists());
}

#[test]
fn durable_workspace_write_records_executing_before_mutation_and_reconciles_once() {
    let temporary_workspace = TestWorkspace::new("durable-write-dispatch");
    let target_path = temporary_workspace.path.join("notes.txt");
    std::fs::write(&target_path, "before\n").expect("write fixture");
    let database_path = temporary_authority_database_path();
    let store = Arc::new(SqliteAuthorityStore::open(&database_path).expect("open authority store"));
    let task_object_id = object_id(621);
    let effect_object_id = object_id(622);
    let intent_object_id = object_id(623);
    let admitted_at = WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid admission time");

    for (object_id, domain, lifecycle_state, event_id) in [
        (
            task_object_id.clone(),
            LifecycleDomain::Task,
            "RUNNING",
            621,
        ),
        (
            effect_object_id.clone(),
            LifecycleDomain::Effect,
            "PROPOSED",
            622,
        ),
    ] {
        store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: object_id.clone(),
                    domain,
                    state: state(lifecycle_state),
                    version: Version::INITIAL,
                    body: json!({"fixture": "p2-t10-d02"}),
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
    let idempotency_key = "p2-t10-d02-workspace-write";
    let parameters_digest = "sha256:p2-t10-d02-workspace-write";
    store
        .insert_intent(
            &IntentRow {
                intent_id: intent_object_id.clone(),
                idempotency_key: idempotency_key.to_owned(),
                parameters_digest: parameters_digest.to_owned(),
                action: "write".to_owned(),
                target: "workspace://notes.txt".to_owned(),
                effect_object_id: effect_object_id.clone(),
                expected_state_version: Version::INITIAL,
                grant_epoch: 1,
                capability_set_version: 1,
                task_binding: None,
                canonical_json: "{\"intent\":\"p2-t10-d02\"}".to_owned(),
            },
            &EventDraft {
                event_id: EventId::parse("00000000-0000-7000-a000-000000000623")
                    .expect("valid intent event identifier"),
                object_id: intent_object_id,
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "intent.minted".to_owned(),
                canonical_json: "{\"intent\":\"p2-t10-d02\"}".to_owned(),
            },
        )
        .expect("persist durable intent");

    let validated_request = staged_mutation_request(
        NativeOperationFamily::WorkspaceWrite,
        &temporary_workspace.path,
        "workspace://notes.txt",
        b"after\n",
        WorkspacePreimage::Digest(image_digest(b"before\n")),
    );
    let native_executor = mutation_executor(1, &temporary_workspace);
    native_executor
        .stage_request(
            idempotency_key.to_owned(),
            parameters_digest.to_owned(),
            &validated_request,
        )
        .expect("stage durable intent identity");
    let hook_store = Arc::clone(&store);
    let hook_effect_object_id = effect_object_id.clone();
    let hook_task_object_id = task_object_id.clone();
    native_executor.install_after_staging_write_hook(move |target, _staging| {
        let effect = hook_store
            .load_object(LifecycleDomain::Effect, &hook_effect_object_id)
            .expect("load effect before publication")
            .expect("durable effect exists");
        let task = hook_store
            .load_object(LifecycleDomain::Task, &hook_task_object_id)
            .expect("load task before publication")
            .expect("durable task exists");
        assert_eq!(effect.state.as_str(), "EXECUTING");
        assert_eq!(task.state.as_str(), "RUNNING");
        assert_eq!(task.version, Version::INITIAL);
        assert_eq!(
            std::fs::read_to_string(target).expect("target readable"),
            "before\n",
            "the durable EXECUTING record must precede the mutation"
        );
    });
    let unknown_executor = UnknownAfterNativeMutationDispatchExecutor {
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
        UriRef::parse("correlation://personal/p2-t10-d02").expect("valid correlation reference"),
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
        .expect("dispatch native mutation through lost-response wrapper");
    assert!(matches!(outcome, DispatchOutcome::Unknown { .. }));
    let unknown = effect_protocol
        .record_outcome(
            &effect_object_id,
            dispatched.after_version,
            &outcome,
            &writer_lease,
        )
        .expect("record unknown post-publication outcome");
    assert_eq!(
        store
            .load_object(LifecycleDomain::Effect, &effect_object_id)
            .expect("load unknown effect")
            .expect("durable effect exists")
            .state
            .as_str(),
        "OUTCOME_UNKNOWN"
    );
    assert_eq!(native_executor.publish_count(), 1);
    assert_eq!(
        std::fs::read_to_string(&target_path).expect("target readable"),
        "after\n"
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
    assert_eq!(
        native_executor.publish_count(),
        1,
        "reconciliation must query the original key, never write again"
    );
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

// ---------------------------------------------------------------------------
// P2-T10/D03 — HttpFetchReadOnly executor
// ---------------------------------------------------------------------------

const FETCH_ORIGIN: &str = "https://registered.example";

/// Records every request the sink attempts and returns a scripted result, so
/// the executor's Effect-boundary behaviour is provable without the network.
/// The real TLS policy is proven separately against a loopback server in
/// `cognitive-provider-transport/tests/p2_t10_read_only_fetch.rs`.
struct ScriptedFetchTransport {
    result: Mutex<Result<ReadOnlyFetchResponse, ReadOnlyFetchError>>,
    observed_urls: Mutex<Vec<String>>,
    state_path: PathBuf,
}

impl ScriptedFetchTransport {
    fn responding(status: u16, body: &[u8]) -> Self {
        Self {
            result: Mutex::new(Ok(ReadOnlyFetchResponse {
                status,
                body: body.to_vec(),
            })),
            observed_urls: Mutex::new(Vec::new()),
            state_path: temporary_workspace_path("http-fetch-state"),
        }
    }

    fn failing(error: ReadOnlyFetchError) -> Self {
        Self {
            result: Mutex::new(Err(error)),
            observed_urls: Mutex::new(Vec::new()),
            state_path: temporary_workspace_path("http-fetch-state"),
        }
    }

    fn observed_urls(&self) -> Vec<String> {
        self.observed_urls
            .lock()
            .map(|urls| urls.clone())
            .unwrap_or_default()
    }
}

impl Drop for ScriptedFetchTransport {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.state_path);
    }
}

impl ReadOnlyFetchTransport for ScriptedFetchTransport {
    fn fetch(
        &self,
        request: &ReadOnlyFetchRequest,
    ) -> Result<ReadOnlyFetchResponse, ReadOnlyFetchError> {
        assert_eq!(
            request.method,
            ReadOnlyFetchMethod::Get,
            "the MVP sink issues GET only"
        );
        if let Ok(mut observed_urls) = self.observed_urls.lock() {
            observed_urls.push(request.url.clone());
        }
        match self.result.lock() {
            Ok(result) => result.clone(),
            Err(_) => Err(ReadOnlyFetchError::Network {
                detail: "scripted transport is poisoned",
            }),
        }
    }
}

fn http_fetch_call(
    idempotency_key: &str,
    parameters_digest: &str,
    target: &str,
    fencing_epoch: i64,
) -> ExecutorCall {
    ExecutorCall {
        action: "fetch".to_owned(),
        target: target.to_owned(),
        idempotency_key: idempotency_key.to_owned(),
        parameters_digest: parameters_digest.to_owned(),
        authorization_digest: "authorization-digest".to_owned(),
        fencing_epoch,
    }
}

fn staged_fetch_request(target: &str, _output_limit_bytes: usize) -> ValidatedNativeToolRequest {
    let mut request = request_for(NativeOperationFamily::HttpFetchReadOnly);
    request.target = target.to_owned();
    request.input.clear();
    request.workspace_root = None;
    validate_native_tool_request(&request).expect("valid read-only fetch")
}

fn scripted_fetch_executor(
    transport: Arc<ScriptedFetchTransport>,
) -> NativeHttpFetchReadOnlyExecutor<ScriptedFetchTransport> {
    scripted_fetch_executor_at_epoch(transport, 7)
}

fn scripted_fetch_executor_at_epoch(
    transport: Arc<ScriptedFetchTransport>,
    trusted_fencing_epoch: i64,
) -> NativeHttpFetchReadOnlyExecutor<ScriptedFetchTransport> {
    NativeHttpFetchReadOnlyExecutor::new(
        trusted_fencing_epoch,
        Arc::clone(&transport),
        vec![FETCH_ORIGIN.to_owned()],
        5_000,
        Arc::new(
            DurableExecutorStateStore::open(&transport.state_path).expect("open fetch state store"),
        ),
    )
}

struct UnknownAfterNativeHttpFetchExecutor<'executor> {
    native_executor: &'executor NativeHttpFetchReadOnlyExecutor<ScriptedFetchTransport>,
}

impl EffectExecutor for UnknownAfterNativeHttpFetchExecutor<'_> {
    fn capabilities(&self) -> ExecutorCapabilities {
        self.native_executor.capabilities()
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        self.native_executor.dispatch(call)?;
        Ok(DispatchOutcome::Unknown {
            detail: "simulated lost post-fetch response".to_owned(),
        })
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        self.native_executor.query_outcome(idempotency_key)
    }
}

#[test]
fn http_fetch_staging_applies_the_registered_network_policy() {
    let transport = Arc::new(ScriptedFetchTransport::responding(200, b"body"));
    let executor = scripted_fetch_executor(Arc::clone(&transport));

    for unsafe_target in [
        "https://unregistered.example/data",
        "https://registered.example/data?query=1",
        "https://registered.example/data#fragment",
    ] {
        let validated_request = staged_fetch_request(unsafe_target, 512);
        assert!(
            matches!(
                executor.stage_request(
                    "fetch-policy".to_owned(),
                    "digest-1".to_owned(),
                    &validated_request,
                ),
                Err(NativeToolExecutionError::InvalidDescriptor(_))
            ),
            "the registered validator must refuse {unsafe_target}"
        );
    }

    // A plaintext or credential-bearing URL never even validates.
    let mut plaintext = request_for(NativeOperationFamily::HttpFetchReadOnly);
    plaintext.target = "http://registered.example/data".to_owned();
    assert_eq!(
        validate_native_tool_request(&plaintext),
        Err(NativeToolExecutionError::NetworkTargetMustUseHttps)
    );

    // A read-only fetch has no request body.
    let mut with_body = request_for(NativeOperationFamily::HttpFetchReadOnly);
    with_body.target = format!("{FETCH_ORIGIN}/data");
    with_body.input = b"payload".to_vec();
    with_body.workspace_root = None;
    let validated_with_body =
        validate_native_tool_request(&with_body).expect("body is not a validator concern");
    assert!(matches!(
        executor.stage_request(
            "fetch-body".to_owned(),
            "digest-1".to_owned(),
            &validated_with_body,
        ),
        Err(NativeToolExecutionError::InvalidDescriptor(_))
    ));

    assert_eq!(executor.fetch_count(), 0);
    assert!(transport.observed_urls().is_empty());
}

#[test]
fn http_fetch_requires_a_staged_digest_bound_request_before_egress() {
    let transport = Arc::new(ScriptedFetchTransport::responding(200, b"payload body"));
    let executor = scripted_fetch_executor(Arc::clone(&transport));
    let target = format!("{FETCH_ORIGIN}/data");
    executor
        .stage_request(
            "fetch-key-1".to_owned(),
            "digest-1".to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("stage read-only fetch");

    assert!(matches!(
        executor.dispatch(&http_fetch_call("fetch-key-1", "wrong-digest", &target, 7)),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert!(matches!(
        executor.dispatch(&http_fetch_call("fetch-key-absent", "digest-1", &target, 7)),
        Ok(DispatchOutcome::NotExecuted { .. })
    ));
    assert_eq!(executor.fetch_count(), 0);
    assert!(transport.observed_urls().is_empty());

    assert!(matches!(
        executor.dispatch(&http_fetch_call("fetch-key-1", "digest-1", &target, 7)),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert_eq!(transport.observed_urls(), vec![target]);
}

#[test]
fn http_fetch_bounds_and_redacts_the_retained_response() {
    let transport = Arc::new(ScriptedFetchTransport::responding(
        200,
        b"token=secret trailing content that must be truncated",
    ));
    let executor = scripted_fetch_executor(Arc::clone(&transport));
    let target = format!("{FETCH_ORIGIN}/data");
    executor
        .stage_request(
            "fetch-bounds".to_owned(),
            "digest-1".to_owned(),
            &staged_fetch_request(&target, 24),
        )
        .expect("stage read-only fetch");
    assert!(matches!(
        executor.dispatch(&http_fetch_call("fetch-bounds", "digest-1", &target, 7)),
        Ok(DispatchOutcome::Executed { .. })
    ));

    let retained = executor
        .completed_output("fetch-bounds")
        .expect("fetch retains bounded output");
    let registered_limit = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|descriptor| descriptor.family == NativeOperationFamily::HttpFetchReadOnly)
        .expect("fetch descriptor")
        .output_limit_bytes;
    assert!(retained.len() <= registered_limit);
    let retained_text = String::from_utf8_lossy(&retained).into_owned();
    assert!(retained_text.starts_with("200\n"));
    assert!(
        !retained_text.contains("secret"),
        "sensitive values must be redacted before retention: {retained_text}"
    );
}

#[test]
fn duplicate_http_fetch_dispatch_performs_exactly_one_request() {
    let transport = Arc::new(ScriptedFetchTransport::responding(200, b"body"));
    let executor = scripted_fetch_executor(Arc::clone(&transport));
    let target = format!("{FETCH_ORIGIN}/data");
    executor
        .stage_request(
            "fetch-once".to_owned(),
            "digest-1".to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("stage read-only fetch");
    let call = http_fetch_call("fetch-once", "digest-1", &target, 7);

    let first_outcome = executor.dispatch(&call).expect("first fetch");
    let second_outcome = executor.dispatch(&call).expect("absorbed duplicate");
    assert_eq!(first_outcome, second_outcome);
    assert_eq!(transport.observed_urls().len(), 1);
    assert_eq!(
        executor.query_outcome("fetch-once"),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
}

#[test]
fn http_fetch_unresolved_attempt_reconciles_indeterminate_after_restart() {
    let transport = Arc::new(ScriptedFetchTransport::failing(ReadOnlyFetchError::Timeout));
    let target = format!("{FETCH_ORIGIN}/data");
    let first = scripted_fetch_executor(Arc::clone(&transport));
    first
        .stage_request(
            "fetch-restart-unknown".to_owned(),
            "digest-1".to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("stage fetch");
    assert!(matches!(
        first.dispatch(&http_fetch_call(
            "fetch-restart-unknown",
            "digest-1",
            &target,
            7,
        )),
        Ok(DispatchOutcome::Unknown { .. })
    ));
    assert_eq!(transport.observed_urls().len(), 1);

    let restarted = scripted_fetch_executor(Arc::clone(&transport));
    restarted
        .stage_request(
            "fetch-restart-unknown".to_owned(),
            "digest-1".to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("restage durable intent");
    assert_eq!(
        restarted.query_outcome("fetch-restart-unknown"),
        Ok(ExecutorQueryResult::Indeterminate)
    );
    assert!(matches!(
        restarted.dispatch(&http_fetch_call(
            "fetch-restart-unknown",
            "digest-1",
            &target,
            7,
        )),
        Ok(DispatchOutcome::Unknown { .. })
    ));
    assert_eq!(
        transport.observed_urls().len(),
        1,
        "restart must not blindly repeat an unresolved original-key attempt"
    );
}

#[test]
fn http_fetch_missing_durable_attempt_record_fails_closed_after_restart() {
    let transport = Arc::new(ScriptedFetchTransport::failing(ReadOnlyFetchError::Timeout));
    let target = format!("{FETCH_ORIGIN}/data");
    let first = scripted_fetch_executor(Arc::clone(&transport));
    first
        .stage_request(
            "fetch-missing-state".to_owned(),
            "digest-1".to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("stage fetch");
    assert!(matches!(
        first.dispatch(&http_fetch_call(
            "fetch-missing-state",
            "digest-1",
            &target,
            7,
        )),
        Ok(DispatchOutcome::Unknown { .. })
    ));
    assert!(
        first
            .remove_durable_state("fetch-missing-state")
            .expect("remove test state")
    );

    let restarted = scripted_fetch_executor(Arc::clone(&transport));
    assert!(matches!(
        restarted.stage_request(
            "fetch-missing-state".to_owned(),
            "digest-1".to_owned(),
            &staged_fetch_request(&target, 512),
        ),
        Err(NativeToolExecutionError::ExecutorUnavailable(_))
    ));
    assert_eq!(
        restarted.query_outcome("fetch-missing-state"),
        Ok(ExecutorQueryResult::Indeterminate),
        "loss of durable attempt state must never become authoritative non-execution"
    );
}

#[test]
fn http_fetch_completed_receipt_survives_restart_without_second_request() {
    let transport = Arc::new(ScriptedFetchTransport::responding(200, b"body"));
    let target = format!("{FETCH_ORIGIN}/data");
    let first = scripted_fetch_executor(Arc::clone(&transport));
    first
        .stage_request(
            "fetch-restart-complete".to_owned(),
            "digest-1".to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("stage fetch");
    assert!(matches!(
        first.dispatch(&http_fetch_call(
            "fetch-restart-complete",
            "digest-1",
            &target,
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));

    let restarted = scripted_fetch_executor(Arc::clone(&transport));
    restarted
        .stage_request(
            "fetch-restart-complete".to_owned(),
            "digest-1".to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("restage durable intent");
    assert_eq!(
        restarted.query_outcome("fetch-restart-complete"),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
    assert!(matches!(
        restarted.dispatch(&http_fetch_call(
            "fetch-restart-complete",
            "digest-1",
            &target,
            7,
        )),
        Ok(DispatchOutcome::Executed { .. })
    ));
    assert_eq!(transport.observed_urls().len(), 1);
}

#[test]
fn http_fetch_effect_protocol_restart_keeps_unresolved_attempt_indeterminate() {
    let target = format!("{FETCH_ORIGIN}/data");
    let idempotency_key = "fetch-effect-restart-unknown";
    let parameters_digest = "sha256:fetch-effect-restart-unknown";
    let fixture = durable_effect_fixture(
        701,
        "fetch",
        &target,
        idempotency_key,
        parameters_digest,
        "http-fetch-restart-unknown",
    );
    let transport = Arc::new(ScriptedFetchTransport::failing(ReadOnlyFetchError::Timeout));
    let first_executor = scripted_fetch_executor_at_epoch(Arc::clone(&transport), 1);
    first_executor
        .stage_request(
            idempotency_key.to_owned(),
            parameters_digest.to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("stage fetch");
    let clock = FixedEffectClock(fixture.admitted_at.clone());
    let identifiers = UuidV7Generator;
    let grant = effect_grant();
    let governance_currency = GovernanceCurrency {
        revocation_epoch: 1,
        capability_set_version: 1,
    };
    let writer_lease = WriterLease { epoch: 1 };
    let first_protocol = EffectProtocol::new(
        fixture.store.as_ref(),
        &clock,
        &identifiers,
        UriRef::parse("actor://personal/daemon").expect("valid actor reference"),
        UriRef::parse("authority://personal/effect-authority").expect("valid authority reference"),
        UriRef::parse("correlation://personal/http-fetch-restart-unknown")
            .expect("valid correlation reference"),
    );
    let authorized = first_protocol
        .authorize_effect(
            &fixture.effect_object_id,
            Version::INITIAL,
            &grant,
            &governance_currency,
            &writer_lease,
        )
        .expect("authorize fetch");
    let (dispatched, outcome) = first_protocol
        .dispatch_effect(
            &fixture.effect_object_id,
            authorized.after_version,
            &grant,
            &governance_currency,
            &first_executor,
            &writer_lease,
        )
        .expect("dispatch fetch");
    assert!(matches!(outcome, DispatchOutcome::Unknown { .. }));
    let unknown = first_protocol
        .record_outcome(
            &fixture.effect_object_id,
            dispatched.after_version,
            &outcome,
            &writer_lease,
        )
        .expect("record unknown fetch");
    drop(first_protocol);
    drop(first_executor);

    let restarted_executor = scripted_fetch_executor_at_epoch(Arc::clone(&transport), 1);
    restarted_executor
        .stage_request(
            idempotency_key.to_owned(),
            parameters_digest.to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("restage durable intent");
    let restarted_protocol = EffectProtocol::new(
        fixture.store.as_ref(),
        &clock,
        &identifiers,
        UriRef::parse("actor://personal/daemon").expect("valid actor reference"),
        UriRef::parse("authority://personal/effect-authority").expect("valid authority reference"),
        UriRef::parse("correlation://personal/http-fetch-restart-unknown")
            .expect("valid correlation reference"),
    );
    let (_, query) = restarted_protocol
        .reconcile(
            &fixture.effect_object_id,
            "OUTCOME_UNKNOWN",
            unknown.after_version,
            &restarted_executor,
            &writer_lease,
        )
        .expect("reconcile after restart");
    assert_eq!(query, ExecutorQueryResult::Indeterminate);
    assert_eq!(transport.observed_urls().len(), 1);
    assert_eq!(
        fixture
            .store
            .load_object(LifecycleDomain::Task, &fixture.task_object_id)
            .expect("load task")
            .expect("task exists")
            .version,
        Version::INITIAL
    );
    std::fs::remove_file(fixture.database_path).unwrap_or(());
}

#[test]
fn http_fetch_effect_protocol_restart_recovers_completed_key_bound_receipt() {
    let target = format!("{FETCH_ORIGIN}/data");
    let idempotency_key = "fetch-effect-restart-complete";
    let parameters_digest = "sha256:fetch-effect-restart-complete";
    let fixture = durable_effect_fixture(
        711,
        "fetch",
        &target,
        idempotency_key,
        parameters_digest,
        "http-fetch-restart-complete",
    );
    let transport = Arc::new(ScriptedFetchTransport::responding(200, b"body"));
    let first_executor = scripted_fetch_executor_at_epoch(Arc::clone(&transport), 1);
    first_executor
        .stage_request(
            idempotency_key.to_owned(),
            parameters_digest.to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("stage fetch");
    let clock = FixedEffectClock(fixture.admitted_at.clone());
    let identifiers = UuidV7Generator;
    let grant = effect_grant();
    let governance_currency = GovernanceCurrency {
        revocation_epoch: 1,
        capability_set_version: 1,
    };
    let writer_lease = WriterLease { epoch: 1 };
    let first_protocol = EffectProtocol::new(
        fixture.store.as_ref(),
        &clock,
        &identifiers,
        UriRef::parse("actor://personal/daemon").expect("valid actor reference"),
        UriRef::parse("authority://personal/effect-authority").expect("valid authority reference"),
        UriRef::parse("correlation://personal/http-fetch-restart-complete")
            .expect("valid correlation reference"),
    );
    let authorized = first_protocol
        .authorize_effect(
            &fixture.effect_object_id,
            Version::INITIAL,
            &grant,
            &governance_currency,
            &writer_lease,
        )
        .expect("authorize fetch");
    let (dispatched, outcome) = {
        let lost_response = UnknownAfterNativeHttpFetchExecutor {
            native_executor: &first_executor,
        };
        first_protocol
            .dispatch_effect(
                &fixture.effect_object_id,
                authorized.after_version,
                &grant,
                &governance_currency,
                &lost_response,
                &writer_lease,
            )
            .expect("dispatch through lost-response wrapper")
    };
    assert!(matches!(outcome, DispatchOutcome::Unknown { .. }));
    let unknown = first_protocol
        .record_outcome(
            &fixture.effect_object_id,
            dispatched.after_version,
            &outcome,
            &writer_lease,
        )
        .expect("record unknown fetch");
    drop(first_protocol);
    drop(first_executor);

    let restarted_executor = scripted_fetch_executor_at_epoch(Arc::clone(&transport), 1);
    restarted_executor
        .stage_request(
            idempotency_key.to_owned(),
            parameters_digest.to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("restage durable intent");
    let restarted_protocol = EffectProtocol::new(
        fixture.store.as_ref(),
        &clock,
        &identifiers,
        UriRef::parse("actor://personal/daemon").expect("valid actor reference"),
        UriRef::parse("authority://personal/effect-authority").expect("valid authority reference"),
        UriRef::parse("correlation://personal/http-fetch-restart-complete")
            .expect("valid correlation reference"),
    );
    let (_, query) = restarted_protocol
        .reconcile(
            &fixture.effect_object_id,
            "OUTCOME_UNKNOWN",
            unknown.after_version,
            &restarted_executor,
            &writer_lease,
        )
        .expect("reconcile completed fetch after restart");
    assert_eq!(query, ExecutorQueryResult::ExecutedWithOriginalKey);
    assert_eq!(transport.observed_urls().len(), 1);
    std::fs::remove_file(fixture.database_path).unwrap_or(());
}

#[test]
fn http_fetch_classifies_transport_failures_without_inventing_a_result() {
    let target = format!("{FETCH_ORIGIN}/data");
    for (error, expects_unknown) in [
        (
            ReadOnlyFetchError::Policy {
                detail: "refused before egress",
            },
            false,
        ),
        (ReadOnlyFetchError::ResponseTooLarge, false),
        (ReadOnlyFetchError::Timeout, true),
        (
            ReadOnlyFetchError::Network {
                detail: "transport fault",
            },
            true,
        ),
    ] {
        let transport = Arc::new(ScriptedFetchTransport::failing(error));
        let executor = scripted_fetch_executor(Arc::clone(&transport));
        executor
            .stage_request(
                "fetch-failure".to_owned(),
                "digest-1".to_owned(),
                &staged_fetch_request(&target, 512),
            )
            .expect("stage read-only fetch");

        let outcome = executor
            .dispatch(&http_fetch_call("fetch-failure", "digest-1", &target, 7))
            .expect("dispatch classifies the failure");
        if expects_unknown {
            assert!(
                matches!(outcome, DispatchOutcome::Unknown { .. }),
                "a request that may have reached the origin is uncertain: {outcome:?}"
            );
        } else {
            assert!(
                matches!(outcome, DispatchOutcome::NotExecuted { .. }),
                "a refusal with nothing retained is authoritative non-execution: {outcome:?}"
            );
        }
        assert_eq!(executor.completed_output("fetch-failure"), None);
        assert_eq!(
            executor.query_outcome("fetch-failure"),
            Ok(if expects_unknown {
                ExecutorQueryResult::Indeterminate
            } else {
                ExecutorQueryResult::NotExecuted
            })
        );
    }
}

#[test]
fn http_fetch_sink_rejects_stale_fencing_before_egress() {
    let transport = Arc::new(ScriptedFetchTransport::responding(200, b"body"));
    let executor = scripted_fetch_executor(Arc::clone(&transport));
    let target = format!("{FETCH_ORIGIN}/data");
    executor
        .stage_request(
            "fetch-fenced".to_owned(),
            "digest-1".to_owned(),
            &staged_fetch_request(&target, 512),
        )
        .expect("stage read-only fetch");

    assert_eq!(
        executor.dispatch(&http_fetch_call("fetch-fenced", "digest-1", &target, 6)),
        Ok(DispatchOutcome::FencedStaleEpoch { sink_epoch: 7 })
    );
    assert_eq!(executor.fetch_count(), 0);
    assert!(transport.observed_urls().is_empty());
}

// ---------------------------------------------------------------------------
// P2-T10/D04 — assembled-executor parity for the readiness projection
// ---------------------------------------------------------------------------

/// Stage one request of `family` through whichever sink owns that family.
fn stage_family_through_its_executor(
    family: NativeOperationFamily,
    workspace_root: &std::path::Path,
) -> Result<(), NativeToolExecutionError> {
    let idempotency_key = format!("parity-{family:?}");
    let parameters_digest = "parity-digest".to_owned();
    match family {
        NativeOperationFamily::WorkspaceRead => {
            let mut request = request_for(family);
            request.target = "workspace://notes.txt".to_owned();
            request.input.clear();
            request.workspace_root = Some(workspace_root.to_path_buf());
            let validated = validate_native_tool_request(&request)?;
            NativeWorkspaceReadExecutor::new(1).stage_request(
                idempotency_key,
                parameters_digest,
                &validated,
            )
        }
        NativeOperationFamily::WorkspaceSearch => {
            let validated = staged_search_request(workspace_root, "workspace://", b"needle");
            NativeWorkspaceSearchExecutor::new(1).stage_request(
                idempotency_key,
                parameters_digest,
                &validated,
            )
        }
        NativeOperationFamily::WorkspaceWrite => {
            let validated = staged_mutation_request(
                family,
                workspace_root,
                "workspace://notes.txt",
                b"content\n",
                WorkspacePreimage::Absent,
            );
            NativeWorkspaceMutationExecutor::new(1, durable_state_store(workspace_root))
                .stage_request(idempotency_key, parameters_digest, &validated)
        }
        NativeOperationFamily::WorkspacePatch => {
            let validated = staged_mutation_request(
                family,
                workspace_root,
                "workspace://notes.txt",
                b"@@ -1 +1 @@\n-old\n+new\n",
                WorkspacePreimage::Absent,
            );
            NativeWorkspaceMutationExecutor::new(1, durable_state_store(workspace_root))
                .stage_request(idempotency_key, parameters_digest, &validated)
        }
        NativeOperationFamily::ProcessCheck => {
            let validated = validate_native_tool_request(&process_check_request())?;
            NativeProcessCheckExecutor::new(
                1,
                Arc::new(BoundedProcessCheckSupervisor::new(Duration::from_secs(1))),
                Duration::from_secs(1),
            )
            .stage_request(idempotency_key, parameters_digest, &validated)
        }
        NativeOperationFamily::HttpFetchReadOnly => {
            let validated = staged_fetch_request(&format!("{FETCH_ORIGIN}/data"), 512);
            scripted_fetch_executor(Arc::new(ScriptedFetchTransport::responding(200, b"body")))
                .stage_request(idempotency_key, parameters_digest, &validated)
        }
    }
}

/// The readiness projection is only honest if the list it derives from is.
/// This fails the moment `ASSEMBLED_EXECUTOR_FAMILIES` names a family that no
/// sink will accept.
#[test]
fn every_assembled_family_has_a_sink_that_accepts_it() {
    let temporary_workspace = TestWorkspace::new("assembled-family-parity");
    for family in ASSEMBLED_EXECUTOR_FAMILIES {
        let staged = stage_family_through_its_executor(family, &temporary_workspace.path);
        assert!(
            staged.is_ok(),
            "{family:?} is reported assembled but no sink accepts it: {staged:?}"
        );
    }
}

/// Readiness still follows the assembled set rather than registry
/// availability, and still leaves every immutable descriptor digest alone.
#[test]
fn readiness_follows_the_assembled_set_without_touching_any_descriptor_digest() {
    let digests_before = BUILTIN_TOOL_CATALOG
        .iter()
        .map(|descriptor| {
            cognitive_kernel::tool_registry::compute_descriptor_digest(descriptor)
                .expect("descriptor digest")
        })
        .collect::<Vec<_>>();

    for descriptor in BUILTIN_TOOL_CATALOG.iter() {
        assert_eq!(
            cognitive_kernel::tool_registry::tool_execution_readiness(
                descriptor,
                &ASSEMBLED_EXECUTOR_FAMILIES,
            ),
            cognitive_kernel::tool_registry::ToolExecutionReadiness::ExecutionReady,
            "{} has a sink and is enabled, so it must project as executable",
            descriptor.operation_id
        );
        // The projection is derived, not stored: drop the assembled set and
        // the same enabled descriptor immediately reads registered-only.
        assert_eq!(
            cognitive_kernel::tool_registry::tool_execution_readiness(descriptor, &[]),
            cognitive_kernel::tool_registry::ToolExecutionReadiness::RegisteredOnly,
            "{} must never forge readiness from registry availability",
            descriptor.operation_id
        );
    }

    let digests_after = BUILTIN_TOOL_CATALOG
        .iter()
        .map(|descriptor| {
            cognitive_kernel::tool_registry::compute_descriptor_digest(descriptor)
                .expect("descriptor digest")
        })
        .collect::<Vec<_>>();
    assert_eq!(digests_before, digests_after);
}

#[test]
fn non_fetch_descriptor_cannot_be_staged_for_fetch_dispatch() {
    let transport = Arc::new(ScriptedFetchTransport::responding(200, b"body"));
    let executor = scripted_fetch_executor(transport);
    let mut request = request_for(NativeOperationFamily::WorkspaceRead);
    request.input.clear();
    let validated_request = validate_native_tool_request(&request).expect("valid read request");

    assert_eq!(
        executor.stage_request(
            "read-key".to_owned(),
            "digest-1".to_owned(),
            &validated_request,
        ),
        Err(NativeToolExecutionError::UnsupportedExecutionFamily)
    );
}
