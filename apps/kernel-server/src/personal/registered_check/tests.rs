#![allow(clippy::expect_used, clippy::panic)]

use super::*;
use cognitive_kernel::executor::{DispatchOutcome, EffectExecutor, ExecutorCall};
use cognitive_kernel::tool_registry::BUILTIN_TOOL_CATALOG;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEMP_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

struct TestLayout {
    root: PathBuf,
    workspace: PathBuf,
}

impl TestLayout {
    fn new(label: &str) -> Self {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "cognitiveos-p2-t16-{label}-{}-{sequence}",
            std::process::id()
        ));
        let workspace = root.join("workspace");
        std::fs::create_dir_all(&workspace).expect("创建测试 workspace");
        Self { root, workspace }
    }

    fn state_store(&self) -> Arc<DurableExecutorStateStore> {
        Arc::new(
            DurableExecutorStateStore::open(&self.root.join("executor-state"))
                .expect("打开独立执行状态"),
        )
    }

    fn artifact_store(&self) -> ArtifactStore {
        ArtifactStore::open(self.root.join("artifacts"), 1024 * 1024)
            .expect("打开测试 ArtifactStore")
    }

    fn write_c2a_fixture(&self) {
        write_repaired_oracle_files(RepairCorpusFamily::TypeScript, &self.workspace)
            .expect("写入 TypeScript 修复 oracle");
    }
}

impl Drop for TestLayout {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[derive(Clone)]
struct FixedRunner {
    observation: RegisteredCheckObservation,
    calls: Arc<AtomicUsize>,
}

impl FixedRunner {
    fn new(observation: RegisteredCheckObservation) -> Self {
        Self {
            observation,
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl RegisteredCheckRunner for FixedRunner {
    fn run(
        &self,
        _descriptor: &RegisteredCheckDescriptor,
        _workspace_root: &Path,
    ) -> Result<RegisteredCheckObservation, RegisteredCheckError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.observation.clone())
    }
}

fn descriptor() -> RegisteredCheckDescriptor {
    RegisteredCheckRegistry::production()
        .resolve(&RegisteredCheckRunRequest::new(C2A_CHECK_ID))
        .expect("解析固定检查")
}

fn native_descriptor() -> NativeToolDescriptor {
    BUILTIN_TOOL_CATALOG
        .iter()
        .find(|candidate| candidate.family == NativeOperationFamily::RegisteredCheckRun)
        .cloned()
        .expect("RegisteredCheckRun 原生描述应存在")
}

fn successful_observation() -> RegisteredCheckObservation {
    let descriptor = descriptor();
    RegisteredCheckObservation {
        exit_code: Some(0),
        stdout: b"{\"passed\":true}".to_vec(),
        stderr: Vec::new(),
        elapsed_milliseconds: 12,
        observed_processes: 1,
        timed_out: false,
        process_tree_escaped: false,
        observed_write_paths: Vec::new(),
        network_attempted: false,
        observed_file_digests: descriptor.expected_file_digests,
    }
}

fn executor_call(key: &str, epoch: i64) -> ExecutorCall {
    ExecutorCall {
        action: "run".to_owned(),
        target: format!("{CHECK_TARGET_PREFIX}{C2A_CHECK_ID}"),
        idempotency_key: key.to_owned(),
        parameters_digest: "sha256:registered-check-parameters".to_owned(),
        authorization_digest: "sha256:registered-check-authorization".to_owned(),
        fencing_epoch: epoch,
    }
}

fn stage(executor: &NativeRegisteredCheckExecutor, key: &str) -> Result<(), RegisteredCheckError> {
    executor.stage_request(
        key.to_owned(),
        "sha256:registered-check-parameters".to_owned(),
        &native_descriptor(),
        &RegisteredCheckRunRequest::new(C2A_CHECK_ID),
    )
}

fn test_executor(
    layout: &TestLayout,
    runner: Arc<dyn RegisteredCheckRunner>,
) -> NativeRegisteredCheckExecutor {
    NativeRegisteredCheckExecutor::new(
        7,
        layout.workspace.clone(),
        layout.state_store(),
        layout.artifact_store(),
        runner,
    )
    .expect("构建固定检查 executor")
}

#[test]
fn caller_can_request_registered_check_by_check_id_only() {
    let request = RegisteredCheckRunRequest::new(C2A_CHECK_ID);
    let resolved = RegisteredCheckRegistry::production()
        .resolve(&request)
        .expect("固定登记的 C2a 检查应可仅凭 check_id 解析");

    assert_eq!(resolved.check_id(), C2A_CHECK_ID);
}

#[test]
fn request_rejects_argv_env_cwd_credentials_and_network_injection() {
    for injected in [
        br#"{"check_id":"c2a.repair.typescript","argv":["--evil"]}"#.as_slice(),
        br#"{"check_id":"c2a.repair.typescript","env":{"TOKEN":"secret"}}"#.as_slice(),
        br#"{"check_id":"c2a.repair.typescript","cwd":"../outside"}"#.as_slice(),
        br#"{"check_id":"c2a.repair.typescript","credential":"secret"}"#.as_slice(),
        br#"{"check_id":"c2a.repair.typescript","network":"unrestricted"}"#.as_slice(),
    ] {
        assert!(matches!(
            RegisteredCheckRunRequest::from_json(injected),
            Err(RegisteredCheckError::InvalidRequest(_))
        ));
    }
    assert_eq!(
        RegisteredCheckRunRequest::from_json(br#"{"check_id":"c2a.repair.typescript"}"#)
            .expect("唯一合法载荷")
            .check_id(),
        C2A_CHECK_ID
    );
}

#[test]
fn unknown_and_shell_metacharacter_check_ids_fail_closed() {
    let rust = RegisteredCheckRegistry::production()
        .resolve(&RegisteredCheckRunRequest::new(C2A_RUST_CHECK_ID))
        .expect("Rust repair check_id 必须解析");
    assert_eq!(rust.check_id(), C2A_RUST_CHECK_ID);
    assert!(matches!(
        RegisteredCheckRegistry::production()
            .resolve(&RegisteredCheckRunRequest::new("unknown.check")),
        Err(RegisteredCheckError::UnknownCheck(_))
    ));
    for injected in [
        "c2a.repair.typescript;whoami",
        "c2a.repair.typescript && cargo test",
        "$(touch escaped)",
        "c2a|powershell",
    ] {
        assert!(matches!(
            RegisteredCheckRegistry::production()
                .resolve(&RegisteredCheckRunRequest::new(injected)),
            Err(RegisteredCheckError::InvalidCheckId)
        ));
    }
}

#[test]
fn descriptor_version_or_field_drift_is_rejected() {
    let registry = RegisteredCheckRegistry::production();
    let mut version_drift = descriptor();
    version_drift.descriptor_version += 1;
    assert!(matches!(
        registry.validate_exact(&version_drift),
        Err(RegisteredCheckError::DescriptorVersionDrift { .. })
    ));

    let mut argv_drift = descriptor();
    argv_drift.argv_template.push("--injected".to_owned());
    assert!(matches!(
        registry.validate_exact(&argv_drift),
        Err(RegisteredCheckError::DescriptorDrift { .. })
    ));

    let mut digest_drift = descriptor();
    digest_drift.descriptor_digest = "sha256:drift".to_owned();
    assert!(matches!(
        registry.validate_exact(&digest_drift),
        Err(RegisteredCheckError::DescriptorDrift { .. })
    ));
}

#[test]
fn descriptor_is_fixed_non_shell_non_package_manager_and_network_denied() {
    let descriptor = descriptor();
    assert_eq!(
        descriptor.executable_identity,
        RegisteredExecutableIdentity::CurrentKernelServer
    );
    assert_eq!(
        descriptor.argv_template,
        ["--personal-registered-check-worker", C2A_CHECK_ID]
    );
    assert!(descriptor.minimal_environment.is_empty());
    assert_eq!(
        descriptor.working_directory_policy,
        RegisteredWorkingDirectoryPolicy::DaemonWorkspaceRoot
    );
    assert_eq!(descriptor.maximum_processes, 1);
    assert!(descriptor.allowed_write_roots.is_empty());
    assert_eq!(descriptor.network_policy, RegisteredNetworkPolicy::Denied);
    assert!(descriptor.argv_template.iter().all(|argument| {
        !argument.contains([';', '|', '&', '$', '`'])
            && !["cargo", "pnpm", "npm", "yarn", "powershell", "cmd", "sh"]
                .contains(&argument.as_str())
    }));
}

#[test]
fn c2a_fixture_has_a_deterministic_fixed_oracle() {
    let layout = TestLayout::new("c2a-fixture");
    layout.write_c2a_fixture();
    let passed =
        run_registered_check_worker(C2A_CHECK_ID, &layout.workspace).expect("运行固定 helper");
    assert!(passed.passed);
    assert!(String::from_utf8_lossy(&passed.bytes).contains("\"passed\":true"));

    std::fs::write(
        layout.workspace.join(C2A_SOURCE_PATH),
        b"export const add = (left: number, right: number) => left - right;\n",
    )
    .expect("写入故障版本");
    let failed =
        run_registered_check_worker(C2A_CHECK_ID, &layout.workspace).expect("运行固定 helper");
    assert!(!failed.passed);
}

#[test]
fn successful_run_publishes_cas_evidence_and_duplicate_dispatch_is_absorbed() {
    let layout = TestLayout::new("success");
    let runner = Arc::new(FixedRunner::new(successful_observation()));
    let executor = test_executor(&layout, runner.clone());
    stage(&executor, "check-success").expect("stage");
    let first = executor
        .dispatch(&executor_call("check-success", 7))
        .expect("dispatch");
    let DispatchOutcome::Executed { receipt_ref } = first else {
        panic!("应返回 Artifact receipt");
    };
    assert!(receipt_ref.starts_with("artifact://sha256/"));
    assert_eq!(
        executor.query_outcome("check-success"),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
    assert!(
        verify_registered_check_artifact(&layout.artifact_store(), &receipt_ref)
            .expect("独立校验 CAS evidence")
    );

    assert!(matches!(
        executor
            .dispatch(&executor_call("check-success", 7))
            .expect("duplicate"),
        DispatchOutcome::Executed { .. }
    ));
    assert_eq!(runner.calls(), 1);
}

#[test]
fn nonzero_exit_is_evidence_but_cannot_pass_independent_verification() {
    let layout = TestLayout::new("exit-nonzero");
    let mut observation = successful_observation();
    observation.exit_code = Some(1);
    let executor = test_executor(&layout, Arc::new(FixedRunner::new(observation)));
    stage(&executor, "exit-nonzero").expect("stage");
    let DispatchOutcome::Executed { receipt_ref } = executor
        .dispatch(&executor_call("exit-nonzero", 7))
        .expect("dispatch")
    else {
        panic!("非零退出仍应产生检查 evidence");
    };
    assert!(
        !verify_registered_check_artifact(&layout.artifact_store(), &receipt_ref)
            .expect("独立 verifier 必须给出失败")
    );
}

#[test]
fn stale_epoch_is_rejected_before_runner_access() {
    let layout = TestLayout::new("stale");
    let runner = Arc::new(FixedRunner::new(successful_observation()));
    let executor = test_executor(&layout, runner.clone());
    stage(&executor, "stale").expect("stage");
    assert_eq!(
        executor.dispatch(&executor_call("stale", 6)),
        Ok(DispatchOutcome::FencedStaleEpoch { sink_epoch: 7 })
    );
    assert_eq!(runner.calls(), 0);
}

fn assert_boundary_violation(label: &str, mutate: impl FnOnce(&mut RegisteredCheckObservation)) {
    let layout = TestLayout::new(label);
    let mut observation = successful_observation();
    mutate(&mut observation);
    let runner = Arc::new(FixedRunner::new(observation));
    let executor = test_executor(&layout, runner.clone());
    stage(&executor, label).expect("stage");
    assert!(matches!(
        executor
            .dispatch(&executor_call(label, 7))
            .expect("dispatch"),
        DispatchOutcome::Unknown { .. }
    ));
    assert_eq!(
        executor.query_outcome(label),
        Ok(ExecutorQueryResult::Indeterminate)
    );
    assert_eq!(runner.calls(), 1);
}

#[test]
fn timeout_output_orphan_write_and_network_boundaries_fail_closed() {
    assert_boundary_violation("timeout", |observation| observation.timed_out = true);
    assert_boundary_violation("oversized", |observation| {
        observation.stdout = vec![b'x'; descriptor().output_limit_bytes + 1];
    });
    assert_boundary_violation("process-tree", |observation| {
        observation.observed_processes = 2;
        observation.process_tree_escaped = true;
    });
    assert_boundary_violation("write-root", |observation| {
        observation
            .observed_write_paths
            .push("../outside.txt".to_owned());
    });
    assert_boundary_violation("network", |observation| {
        observation.network_attempted = true;
    });
}

#[test]
fn crash_before_dispatch_remains_not_executed() {
    let layout = TestLayout::new("before-dispatch");
    let runner = Arc::new(FixedRunner::new(successful_observation()));
    let executor = test_executor(&layout, runner.clone());
    stage(&executor, "before-dispatch").expect("stage");
    assert_eq!(
        executor.query_outcome("before-dispatch"),
        Ok(ExecutorQueryResult::NotExecuted)
    );
    assert_eq!(runner.calls(), 0);
}

#[test]
fn crash_mid_dispatch_is_indeterminate_after_restart_and_never_redispatched() {
    let layout = TestLayout::new("mid-dispatch");
    let first_runner = Arc::new(FixedRunner::new(successful_observation()));
    let first = test_executor(&layout, first_runner.clone());
    stage(&first, "mid-dispatch").expect("stage");
    first
        .mark_attempting_for_test("mid-dispatch")
        .expect("写入 attempting");
    drop(first);

    let restarted_runner = Arc::new(FixedRunner::new(successful_observation()));
    let restarted = test_executor(&layout, restarted_runner.clone());
    stage(&restarted, "mid-dispatch").expect("恢复 stage");
    assert_eq!(
        restarted.query_outcome("mid-dispatch"),
        Ok(ExecutorQueryResult::Indeterminate)
    );
    assert!(matches!(
        restarted
            .dispatch(&executor_call("mid-dispatch", 7))
            .expect("reconcile-only dispatch"),
        DispatchOutcome::Unknown { .. }
    ));
    assert_eq!(first_runner.calls(), 0);
    assert_eq!(restarted_runner.calls(), 0);
}

#[test]
fn crash_after_dispatch_reconciles_artifact_under_original_key() {
    let layout = TestLayout::new("after-dispatch");
    let first_runner = Arc::new(FixedRunner::new(successful_observation()));
    let first = test_executor(&layout, first_runner.clone());
    stage(&first, "after-dispatch").expect("stage");
    assert!(matches!(
        first
            .dispatch(&executor_call("after-dispatch", 7))
            .expect("dispatch"),
        DispatchOutcome::Executed { .. }
    ));
    drop(first);

    let restarted_runner = Arc::new(FixedRunner::new(successful_observation()));
    let restarted = test_executor(&layout, restarted_runner.clone());
    stage(&restarted, "after-dispatch").expect("恢复 stage");
    assert_eq!(
        restarted.query_outcome("after-dispatch"),
        Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
    );
    assert!(matches!(
        restarted
            .dispatch(&executor_call("after-dispatch", 7))
            .expect("absorbed duplicate"),
        DispatchOutcome::Executed { .. }
    ));
    assert_eq!(first_runner.calls(), 1);
    assert_eq!(restarted_runner.calls(), 0);
}

#[test]
fn original_key_cannot_be_rebound_to_another_parameters_digest() {
    let layout = TestLayout::new("key-conflict");
    let executor = test_executor(
        &layout,
        Arc::new(FixedRunner::new(successful_observation())),
    );
    stage(&executor, "same-key").expect("first stage");
    assert_eq!(
        executor.stage_request(
            "same-key".to_owned(),
            "sha256:different".to_owned(),
            &native_descriptor(),
            &RegisteredCheckRunRequest::new(C2A_CHECK_ID),
        ),
        Err(RegisteredCheckError::IdempotencyBindingConflict)
    );
}

#[test]
fn tampered_or_missing_cas_evidence_never_verifies() {
    let layout = TestLayout::new("tampered-evidence");
    assert!(matches!(
        verify_registered_check_artifact(
            &layout.artifact_store(),
            "artifact://sha256/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
        Err(RegisteredCheckError::InvalidEvidence(_))
    ));

    let bytes = br#"{"schema":"personal-registered-check-evidence/0.1","check_id":"unknown"}"#;
    let reference = layout
        .artifact_store()
        .put(bytes)
        .expect("写入伪造 evidence");
    let digest = reference.trim_start_matches("sha256:");
    assert!(
        verify_registered_check_artifact(
            &layout.artifact_store(),
            &format!("artifact://sha256/{digest}")
        )
        .is_err()
    );
}

fn corpus_looks_secret_shaped(bytes: &[u8]) -> bool {
    const NEEDLES: [&[u8]; 9] = [
        b"sk-",
        b"Bearer ",
        b"AKIA",
        b"ASIA",
        b"ghp_",
        b"xoxb-",
        b"BEGIN PRIVATE",
        b"aws_secret",
        b"api_key",
    ];
    NEEDLES
        .iter()
        .any(|needle| bytes.windows(needle.len()).any(|window| window == *needle))
}

#[test]
fn broken_reset_is_deterministic_for_typescript_and_rust() {
    for family in [RepairCorpusFamily::TypeScript, RepairCorpusFamily::Rust] {
        let layout = TestLayout::new(&format!("reset-{}", family.check_id()));
        reset_broken_repair_corpus(family, &layout.workspace).expect("第一次 broken reset");
        let first = corpus_snapshot_digest(&layout.workspace).expect("第一次 snapshot");
        reset_broken_repair_corpus(family, &layout.workspace).expect("第二次 broken reset");
        let second = corpus_snapshot_digest(&layout.workspace).expect("第二次 snapshot");
        assert_eq!(
            first,
            second,
            "{} broken reset must be deterministic",
            family.check_id()
        );
        assert_eq!(
            std::fs::read(layout.workspace.join(family.source_path())).expect("读 broken 源"),
            broken_source_bytes(family)
        );
    }
}

#[test]
fn repaired_corpus_worker_passes_and_broken_corpus_fails() {
    for family in [RepairCorpusFamily::TypeScript, RepairCorpusFamily::Rust] {
        let layout = TestLayout::new(&format!("oracle-{}", family.check_id()));
        write_repaired_oracle_files(family, &layout.workspace).expect("写入 repaired oracle");
        let passed = run_registered_check_worker(family.check_id(), &layout.workspace)
            .expect("repaired worker");
        assert!(
            passed.passed,
            "{} repaired corpus must pass the digest oracle",
            family.check_id()
        );

        reset_broken_repair_corpus(family, &layout.workspace).expect("写入 broken 源");
        let failed = run_registered_check_worker(family.check_id(), &layout.workspace)
            .expect("broken worker");
        assert!(
            !failed.passed,
            "{} broken corpus must fail the digest oracle",
            family.check_id()
        );
    }
}

#[test]
fn gutting_public_test_fails_even_when_source_repaired_and_hidden_intact() {
    for family in [RepairCorpusFamily::TypeScript, RepairCorpusFamily::Rust] {
        let layout = TestLayout::new(&format!("gut-public-{}", family.check_id()));
        write_repaired_oracle_files(family, &layout.workspace).expect("写入 repaired oracle");
        std::fs::write(
            layout.workspace.join(family.public_test_path()),
            b"// gutted public oracle\n",
        )
        .expect("削弱 public test");
        let failed = run_registered_check_worker(family.check_id(), &layout.workspace)
            .expect("gutted public worker");
        assert!(
            !failed.passed,
            "{} public-test weakening must fail even with repaired source and hidden intact",
            family.check_id()
        );
    }
}

#[test]
fn gutting_hidden_test_fails_even_when_source_repaired_and_public_intact() {
    for family in [RepairCorpusFamily::TypeScript, RepairCorpusFamily::Rust] {
        let layout = TestLayout::new(&format!("gut-hidden-{}", family.check_id()));
        write_repaired_oracle_files(family, &layout.workspace).expect("写入 repaired oracle");
        std::fs::write(
            layout.workspace.join(family.hidden_test_path()),
            b"// gutted hidden oracle\n",
        )
        .expect("削弱 hidden test");
        let failed = run_registered_check_worker(family.check_id(), &layout.workspace)
            .expect("gutted hidden worker");
        assert!(
            !failed.passed,
            "{} hidden-test weakening must fail even with repaired source and public intact",
            family.check_id()
        );
    }
}

#[test]
fn frozen_repair_corpora_contain_no_secret_shaped_bytes() {
    for bytes in frozen_repair_corpus_bytes() {
        assert!(
            !corpus_looks_secret_shaped(bytes),
            "frozen repair corpus bytes must not contain secret-shaped material"
        );
    }
}

#[test]
fn rust_and_typescript_descriptors_share_deny_policy_and_exclude_broken_source() {
    let registry = RegisteredCheckRegistry::production();
    for family in [RepairCorpusFamily::TypeScript, RepairCorpusFamily::Rust] {
        let descriptor = registry
            .resolve(&RegisteredCheckRunRequest::new(family.check_id()))
            .expect("解析 repair family");
        assert_eq!(
            descriptor.argv_template,
            [
                "--personal-registered-check-worker".to_owned(),
                family.check_id().to_owned()
            ]
        );
        assert!(descriptor.minimal_environment.is_empty());
        assert_eq!(descriptor.network_policy, RegisteredNetworkPolicy::Denied);
        assert!(descriptor.allowed_write_roots.is_empty());
        assert!(
            !descriptor
                .expected_file_digests
                .values()
                .any(|digest| digest == &check_file_digest(broken_source_bytes(family)))
        );
        let repaired_digest = check_file_digest(repaired_source_bytes(family));
        assert_eq!(
            descriptor.expected_file_digests.get(family.source_path()),
            Some(&repaired_digest)
        );
    }
    let typescript = descriptor();
    assert_eq!(typescript.descriptor_version, 2);
}
