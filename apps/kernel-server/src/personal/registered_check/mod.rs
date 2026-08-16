//! 由 daemon 固定登记的确定性检查执行边界。
//!
//! 调用方只能提交 `check_id`。可执行文件、参数、工作目录、环境、超时、
//! 输出、进程树、写入根和网络策略均来自本模块的不可变目录。

use cognitive_kernel::executor::{
    DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
};
use cognitive_kernel::ports::PortFailure;
use cognitive_kernel::tool_registry::{NativeOperationFamily, NativeToolDescriptor};
use cognitive_store::ArtifactStore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(not(test))]
use std::io::Read;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::{Command, Stdio};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

use crate::personal::tool_executor::DurableExecutorStateStore;

pub(crate) const REGISTERED_CHECK_VERIFIER_REF: &str = "verifier://personal/registered-check";
pub(crate) const REGISTERED_CHECK_VERIFIER_VERSION: &str = "v1";
pub(crate) const CHECK_TARGET_PREFIX: &str = "check://";
const CHECK_DESCRIPTOR_DIGEST_DOMAIN: &str = "personal-registered-check-descriptor/0.1";
const CHECK_FILE_DIGEST_DOMAIN: &str = "personal-registered-check-file/0.1";
const CHECK_STATE_NAMESPACE: &str = "registered-check-run";
const CHECK_STATE_SCHEMA: &str = "personal-registered-check-state/0.1";
pub(crate) const CHECK_EVIDENCE_SCHEMA: &str = "personal-registered-check-evidence/0.1";
pub(crate) const C2A_CHECK_ID: &str = "c2a.repair.typescript";
pub(crate) const C2A_RUST_CHECK_ID: &str = "c2a.repair.rust";
const C2A_SOURCE_PATH: &str = "src/repair.ts";
const C2A_TEST_PATH: &str = "tests/repair.test.ts";
const C2A_HIDDEN_TEST_PATH: &str = "tests/hidden.repair.test.ts";
const C2A_RUST_SOURCE_PATH: &str = "src/repair.rs";
const C2A_RUST_TEST_PATH: &str = "tests/repair.rs";
const C2A_RUST_HIDDEN_TEST_PATH: &str = "tests/hidden.repair.rs";
const C2A_SOURCE: &[u8] =
    b"export function add(left: number, right: number): number {\n  return left + right;\n}\n";
#[cfg(test)]
const C2A_BROKEN_SOURCE: &[u8] =
    b"export function add(left: number, right: number): number {\n  return left - right;\n}\n";
const C2A_TEST: &[u8] = b"import { add } from \"../src/repair\";\n\nif (add(2, 3) !== 5) {\n  throw new Error(\"repair failed\");\n}\n";
const C2A_HIDDEN_TEST: &[u8] = b"import { add } from \"../src/repair\";\n\nif (add(4, 1) !== 5) {\n  throw new Error(\"hidden repair failed\");\n}\n";
const C2A_RUST_SOURCE: &[u8] = b"pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n";
#[cfg(test)]
const C2A_RUST_BROKEN_SOURCE: &[u8] =
    b"pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n";
const C2A_RUST_TEST: &[u8] =
    b"fn public_oracle() {\n    if add(2, 3) != 5 {\n        panic!(\"repair failed\");\n    }\n}\n";
const C2A_RUST_HIDDEN_TEST: &[u8] = b"fn hidden_oracle() {\n    if add(4, 1) != 5 {\n        panic!(\"hidden repair failed\");\n    }\n}\n";
#[cfg(test)]
const CHECK_CORPUS_DIGEST_DOMAIN: &str = "personal-registered-check-corpus/0.1";
const MAXIMUM_WORKSPACE_ENTRIES: usize = 512;

/// 唯一允许跨越调用方边界的请求载荷。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegisteredCheckRunRequest {
    check_id: String,
}

impl RegisteredCheckRunRequest {
    pub(crate) fn new(check_id: impl Into<String>) -> Self {
        Self {
            check_id: check_id.into(),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_json(bytes: &[u8]) -> Result<Self, RegisteredCheckError> {
        serde_json::from_slice(bytes).map_err(|error| {
            RegisteredCheckError::InvalidRequest(format!(
                "registered check request is not check_id-only: {error}"
            ))
        })
    }

    pub(crate) fn check_id(&self) -> &str {
        &self.check_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RegisteredExecutableIdentity {
    CurrentKernelServer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RegisteredWorkingDirectoryPolicy {
    DaemonWorkspaceRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RegisteredNetworkPolicy {
    Denied,
}

/// daemon 编译期固定的一条检查描述。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RegisteredCheckDescriptor {
    check_id: String,
    descriptor_version: i64,
    descriptor_digest: String,
    executable_identity: RegisteredExecutableIdentity,
    argv_template: Vec<String>,
    working_directory_policy: RegisteredWorkingDirectoryPolicy,
    minimal_environment: BTreeMap<String, String>,
    timeout_milliseconds: u64,
    output_limit_bytes: usize,
    maximum_processes: usize,
    allowed_write_roots: Vec<String>,
    network_policy: RegisteredNetworkPolicy,
    expected_file_digests: BTreeMap<String, String>,
}

impl RegisteredCheckDescriptor {
    pub(crate) fn check_id(&self) -> &str {
        &self.check_id
    }

    pub(crate) fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_milliseconds)
    }

    pub(crate) fn target(&self) -> String {
        format!("{CHECK_TARGET_PREFIX}{}", self.check_id)
    }
}

/// 固定目录没有运行期注册 API。
pub(crate) struct RegisteredCheckRegistry;

impl RegisteredCheckRegistry {
    pub(crate) fn production() -> Self {
        Self
    }

    pub(crate) fn resolve(
        &self,
        request: &RegisteredCheckRunRequest,
    ) -> Result<RegisteredCheckDescriptor, RegisteredCheckError> {
        validate_check_id(request.check_id())?;
        let descriptor = REGISTERED_CHECK_CATALOG
            .iter()
            .find(|descriptor| descriptor.check_id == request.check_id)
            .cloned()
            .ok_or_else(|| RegisteredCheckError::UnknownCheck(request.check_id.clone()))?;
        self.validate_exact(&descriptor)?;
        Ok(descriptor)
    }

    pub(crate) fn resolve_target(
        &self,
        target: &str,
    ) -> Result<RegisteredCheckDescriptor, RegisteredCheckError> {
        let check_id = target.strip_prefix(CHECK_TARGET_PREFIX).ok_or_else(|| {
            RegisteredCheckError::InvalidRequest("check target is invalid".into())
        })?;
        self.resolve(&RegisteredCheckRunRequest::new(check_id))
    }

    pub(crate) fn validate_exact(
        &self,
        descriptor: &RegisteredCheckDescriptor,
    ) -> Result<(), RegisteredCheckError> {
        let catalog_descriptor = REGISTERED_CHECK_CATALOG
            .iter()
            .find(|candidate| candidate.check_id == descriptor.check_id)
            .ok_or_else(|| RegisteredCheckError::UnknownCheck(descriptor.check_id.clone()))?;
        if descriptor.descriptor_version != catalog_descriptor.descriptor_version {
            return Err(RegisteredCheckError::DescriptorVersionDrift {
                check_id: descriptor.check_id.clone(),
            });
        }
        let computed = compute_descriptor_digest(descriptor)?;
        if computed != descriptor.descriptor_digest || descriptor != catalog_descriptor {
            return Err(RegisteredCheckError::DescriptorDrift {
                check_id: descriptor.check_id.clone(),
            });
        }
        Ok(())
    }
}

static REGISTERED_CHECK_CATALOG: LazyLock<Vec<RegisteredCheckDescriptor>> = LazyLock::new(|| {
    vec![
        frozen_registered_check_descriptor(
            C2A_CHECK_ID,
            2,
            expected_digests(&[
                (C2A_SOURCE_PATH, C2A_SOURCE),
                (C2A_TEST_PATH, C2A_TEST),
                (C2A_HIDDEN_TEST_PATH, C2A_HIDDEN_TEST),
            ]),
        ),
        frozen_registered_check_descriptor(
            C2A_RUST_CHECK_ID,
            1,
            expected_digests(&[
                (C2A_RUST_SOURCE_PATH, C2A_RUST_SOURCE),
                (C2A_RUST_TEST_PATH, C2A_RUST_TEST),
                (C2A_RUST_HIDDEN_TEST_PATH, C2A_RUST_HIDDEN_TEST),
            ]),
        ),
    ]
});

fn expected_digests(files: &[(&str, &[u8])]) -> BTreeMap<String, String> {
    files
        .iter()
        .map(|(path, bytes)| ((*path).to_owned(), check_file_digest(bytes)))
        .collect()
}

fn frozen_registered_check_descriptor(
    check_id: &str,
    descriptor_version: i64,
    expected_file_digests: BTreeMap<String, String>,
) -> RegisteredCheckDescriptor {
    let mut descriptor = RegisteredCheckDescriptor {
        check_id: check_id.to_owned(),
        descriptor_version,
        descriptor_digest: String::new(),
        executable_identity: RegisteredExecutableIdentity::CurrentKernelServer,
        argv_template: vec![
            "--personal-registered-check-worker".to_owned(),
            check_id.to_owned(),
        ],
        working_directory_policy: RegisteredWorkingDirectoryPolicy::DaemonWorkspaceRoot,
        minimal_environment: BTreeMap::new(),
        timeout_milliseconds: 10_000,
        output_limit_bytes: 64 * 1024,
        maximum_processes: 1,
        allowed_write_roots: Vec::new(),
        network_policy: RegisteredNetworkPolicy::Denied,
        expected_file_digests,
    };
    if let Ok(digest) = compute_descriptor_digest(&descriptor) {
        descriptor.descriptor_digest = digest;
    }
    descriptor
}

/// Frozen TypeScript/Rust repair corpora used by the C2a journey.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RepairCorpusFamily {
    TypeScript,
    Rust,
}

#[cfg(test)]
impl RepairCorpusFamily {
    pub(crate) fn check_id(self) -> &'static str {
        match self {
            Self::TypeScript => C2A_CHECK_ID,
            Self::Rust => C2A_RUST_CHECK_ID,
        }
    }

    pub(crate) fn source_path(self) -> &'static str {
        match self {
            Self::TypeScript => C2A_SOURCE_PATH,
            Self::Rust => C2A_RUST_SOURCE_PATH,
        }
    }

    pub(crate) fn public_test_path(self) -> &'static str {
        match self {
            Self::TypeScript => C2A_TEST_PATH,
            Self::Rust => C2A_RUST_TEST_PATH,
        }
    }

    pub(crate) fn hidden_test_path(self) -> &'static str {
        match self {
            Self::TypeScript => C2A_HIDDEN_TEST_PATH,
            Self::Rust => C2A_RUST_HIDDEN_TEST_PATH,
        }
    }

    fn public_test_bytes(self) -> &'static [u8] {
        match self {
            Self::TypeScript => C2A_TEST,
            Self::Rust => C2A_RUST_TEST,
        }
    }

    fn hidden_test_bytes(self) -> &'static [u8] {
        match self {
            Self::TypeScript => C2A_HIDDEN_TEST,
            Self::Rust => C2A_RUST_HIDDEN_TEST,
        }
    }
}

#[cfg(test)]
pub(crate) fn repaired_source_bytes(family: RepairCorpusFamily) -> &'static [u8] {
    match family {
        RepairCorpusFamily::TypeScript => C2A_SOURCE,
        RepairCorpusFamily::Rust => C2A_RUST_SOURCE,
    }
}

#[cfg(test)]
pub(crate) fn broken_source_bytes(family: RepairCorpusFamily) -> &'static [u8] {
    match family {
        RepairCorpusFamily::TypeScript => C2A_BROKEN_SOURCE,
        RepairCorpusFamily::Rust => C2A_RUST_BROKEN_SOURCE,
    }
}

#[cfg(test)]
pub(crate) fn reset_broken_repair_corpus(
    family: RepairCorpusFamily,
    workspace_root: &Path,
) -> Result<(), RegisteredCheckError> {
    write_corpus_files(
        workspace_root,
        &[
            (family.source_path(), broken_source_bytes(family)),
            (family.public_test_path(), family.public_test_bytes()),
            (family.hidden_test_path(), family.hidden_test_bytes()),
        ],
    )
}

#[cfg(test)]
pub(crate) fn write_repaired_oracle_files(
    family: RepairCorpusFamily,
    workspace_root: &Path,
) -> Result<(), RegisteredCheckError> {
    write_corpus_files(
        workspace_root,
        &[
            (family.source_path(), repaired_source_bytes(family)),
            (family.public_test_path(), family.public_test_bytes()),
            (family.hidden_test_path(), family.hidden_test_bytes()),
        ],
    )
}

#[cfg(test)]
pub(crate) fn corpus_snapshot_digest(
    workspace_root: &Path,
) -> Result<String, RegisteredCheckError> {
    let snapshot = snapshot_workspace(workspace_root)?;
    let bytes =
        cognitive_contracts::canonical::canonical_bytes_of_value(&serde_json::json!(snapshot))
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
    cognitive_contracts::canonical::digest(&bytes, CHECK_CORPUS_DIGEST_DOMAIN)
        .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))
}

#[cfg(test)]
pub(crate) fn frozen_repair_corpus_bytes() -> Vec<&'static [u8]> {
    vec![
        C2A_SOURCE,
        C2A_BROKEN_SOURCE,
        C2A_TEST,
        C2A_HIDDEN_TEST,
        C2A_RUST_SOURCE,
        C2A_RUST_BROKEN_SOURCE,
        C2A_RUST_TEST,
        C2A_RUST_HIDDEN_TEST,
    ]
}

#[cfg(test)]
fn write_corpus_files(
    workspace_root: &Path,
    files: &[(&str, &[u8])],
) -> Result<(), RegisteredCheckError> {
    for (relative, bytes) in files {
        let path = workspace_root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
        }
        std::fs::write(&path, bytes)
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
    }
    Ok(())
}

fn validate_check_id(check_id: &str) -> Result<(), RegisteredCheckError> {
    if check_id.is_empty()
        || check_id.len() > 128
        || !check_id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
    {
        return Err(RegisteredCheckError::InvalidCheckId);
    }
    Ok(())
}

fn compute_descriptor_digest(
    descriptor: &RegisteredCheckDescriptor,
) -> Result<String, RegisteredCheckError> {
    let value = serde_json::json!({
        "allowed_write_roots": &descriptor.allowed_write_roots,
        "argv_template": &descriptor.argv_template,
        "check_id": &descriptor.check_id,
        "descriptor_version": descriptor.descriptor_version,
        "executable_identity": descriptor.executable_identity,
        "expected_file_digests": &descriptor.expected_file_digests,
        "maximum_processes": descriptor.maximum_processes,
        "minimal_environment": &descriptor.minimal_environment,
        "network_policy": descriptor.network_policy,
        "output_limit_bytes": descriptor.output_limit_bytes,
        "timeout_milliseconds": descriptor.timeout_milliseconds,
        "working_directory_policy": descriptor.working_directory_policy,
    });
    let bytes = cognitive_contracts::canonical::canonical_bytes_of_value(&value)
        .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
    cognitive_contracts::canonical::digest(&bytes, CHECK_DESCRIPTOR_DIGEST_DOMAIN)
        .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))
}

fn check_file_digest(bytes: &[u8]) -> String {
    cognitive_contracts::canonical::digest(bytes, CHECK_FILE_DIGEST_DOMAIN)
        .unwrap_or_else(|_| format!("sha256:{:x}", Sha256::digest(bytes)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegisteredCheckObservation {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub elapsed_milliseconds: u64,
    pub observed_processes: usize,
    pub timed_out: bool,
    pub process_tree_escaped: bool,
    pub observed_write_paths: Vec<String>,
    pub network_attempted: bool,
    pub observed_file_digests: BTreeMap<String, String>,
}

pub(crate) trait RegisteredCheckRunner: Send + Sync {
    fn run(
        &self,
        descriptor: &RegisteredCheckDescriptor,
        workspace_root: &Path,
    ) -> Result<RegisteredCheckObservation, RegisteredCheckError>;
}

pub(crate) struct SystemRegisteredCheckRunner;

impl RegisteredCheckRunner for SystemRegisteredCheckRunner {
    fn run(
        &self,
        descriptor: &RegisteredCheckDescriptor,
        workspace_root: &Path,
    ) -> Result<RegisteredCheckObservation, RegisteredCheckError> {
        RegisteredCheckRegistry::production().validate_exact(descriptor)?;
        let canonical_workspace_root = std::fs::canonicalize(workspace_root)
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
        // `cargo test --bin kernel-server` uses the libtest harness as
        // current_exe, which does not honor `--personal-registered-check-worker`.
        // The production binary still spawns that helper with env_clear; bin
        // unit tests invoke the same digest oracle in-process. Isolation spawn
        // remains covered by `tests/p2_t16_registered_check.rs`.
        #[cfg(test)]
        {
            observe_registered_check_digest_oracle(descriptor, &canonical_workspace_root)
        }
        #[cfg(not(test))]
        {
            spawn_current_kernel_server_worker(descriptor, &canonical_workspace_root)
        }
    }
}

#[cfg(test)]
fn observe_registered_check_digest_oracle(
    descriptor: &RegisteredCheckDescriptor,
    canonical_workspace_root: &Path,
) -> Result<RegisteredCheckObservation, RegisteredCheckError> {
    let before = snapshot_workspace(canonical_workspace_root)?;
    let started = Instant::now();
    let outcome = run_registered_check_worker(&descriptor.check_id, canonical_workspace_root)?;
    let after = snapshot_workspace(canonical_workspace_root)?;
    Ok(RegisteredCheckObservation {
        exit_code: Some(if outcome.passed { 0 } else { 1 }),
        stdout: outcome.bytes,
        stderr: Vec::new(),
        elapsed_milliseconds: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        observed_processes: 1,
        timed_out: false,
        process_tree_escaped: false,
        observed_write_paths: changed_paths(&before, &after),
        network_attempted: false,
        observed_file_digests: observed_oracle_files(descriptor, canonical_workspace_root)?,
    })
}

#[cfg(not(test))]
fn spawn_current_kernel_server_worker(
    descriptor: &RegisteredCheckDescriptor,
    canonical_workspace_root: &Path,
) -> Result<RegisteredCheckObservation, RegisteredCheckError> {
    let before = snapshot_workspace(canonical_workspace_root)?;
    let executable = match descriptor.executable_identity {
        RegisteredExecutableIdentity::CurrentKernelServer => std::env::current_exe()
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?,
    };
    let mut command = Command::new(executable);
    command
        .args(&descriptor.argv_template)
        .current_dir(canonical_workspace_root)
        .env_clear()
        .envs(&descriptor.minimal_environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| RegisteredCheckError::Infrastructure("stdout pipe missing".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| RegisteredCheckError::Infrastructure("stderr pipe missing".into()))?;
    let output_ceiling = descriptor.output_limit_bytes;
    let stdout_reader = std::thread::spawn(move || read_bounded_output(stdout, output_ceiling));
    let stderr_reader = std::thread::spawn(move || read_bounded_output(stderr, output_ceiling));
    let started = Instant::now();
    let (status, timed_out) = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?
        {
            break (status, false);
        }
        if started.elapsed() >= descriptor.timeout() {
            child
                .kill()
                .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
            let status = child
                .wait()
                .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
            break (status, true);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| RegisteredCheckError::Infrastructure("stdout reader panicked".into()))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| RegisteredCheckError::Infrastructure("stderr reader panicked".into()))??;
    if stdout.overflowed || stderr.overflowed {
        return Err(RegisteredCheckError::OutputTooLarge);
    }
    let after = snapshot_workspace(canonical_workspace_root)?;
    Ok(RegisteredCheckObservation {
        exit_code: status.code(),
        stdout: stdout.bytes,
        stderr: stderr.bytes,
        elapsed_milliseconds: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        // 固定 helper 的实现不含子进程或网络 API；运行期仍保留可注入观察，
        // 使任何未来实现一旦报告扩张即可 fail closed。
        observed_processes: 1,
        timed_out,
        process_tree_escaped: false,
        observed_write_paths: changed_paths(&before, &after),
        network_attempted: false,
        observed_file_digests: observed_oracle_files(descriptor, canonical_workspace_root)?,
    })
}

#[cfg(not(test))]
struct BoundedRead {
    bytes: Vec<u8>,
    overflowed: bool,
}

#[cfg(not(test))]
fn read_bounded_output(
    mut input: impl Read,
    maximum_bytes: usize,
) -> Result<BoundedRead, RegisteredCheckError> {
    let mut bytes = Vec::new();
    let limit = u64::try_from(maximum_bytes).unwrap_or(u64::MAX);
    input
        .by_ref()
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
    let overflowed = bytes.len() > maximum_bytes;
    if overflowed {
        bytes.truncate(maximum_bytes);
    }
    Ok(BoundedRead { bytes, overflowed })
}

fn snapshot_workspace(root: &Path) -> Result<BTreeMap<String, String>, RegisteredCheckError> {
    let mut pending = vec![root.to_path_buf()];
    let mut snapshot = BTreeMap::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?
        {
            let entry =
                entry.map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
            let file_type = entry
                .file_type()
                .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
            if file_type.is_symlink() {
                return Err(RegisteredCheckError::WorkspaceBoundaryViolation(
                    "workspace contains a symbolic link".into(),
                ));
            }
            if file_type.is_dir() {
                pending.push(entry.path());
                continue;
            }
            if !file_type.is_file() {
                return Err(RegisteredCheckError::WorkspaceBoundaryViolation(
                    "workspace contains a non-regular entry".into(),
                ));
            }
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = std::fs::read(entry.path())
                .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
            snapshot.insert(relative, check_file_digest(&bytes));
            if snapshot.len() > MAXIMUM_WORKSPACE_ENTRIES {
                return Err(RegisteredCheckError::WorkspaceBoundaryViolation(
                    "workspace entry ceiling exceeded".into(),
                ));
            }
        }
    }
    Ok(snapshot)
}

fn changed_paths(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) -> Vec<String> {
    before
        .keys()
        .chain(after.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|path| before.get(*path) != after.get(*path))
        .cloned()
        .collect()
}

fn observed_oracle_files(
    descriptor: &RegisteredCheckDescriptor,
    workspace_root: &Path,
) -> Result<BTreeMap<String, String>, RegisteredCheckError> {
    descriptor
        .expected_file_digests
        .keys()
        .map(|relative| {
            let path = Path::new(relative);
            if path.is_absolute()
                || path
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir))
            {
                return Err(RegisteredCheckError::WorkspaceBoundaryViolation(
                    "registered oracle path escaped the workspace".into(),
                ));
            }
            let bytes = std::fs::read(workspace_root.join(path))
                .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
            Ok((relative.clone(), check_file_digest(&bytes)))
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RegisteredCheckAttemptStatus {
    Staged,
    Attempting,
    Completed,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RegisteredCheckStateRecord {
    schema: String,
    idempotency_key: String,
    parameters_digest: String,
    target: String,
    check_id: String,
    descriptor_version: i64,
    descriptor_digest: String,
    status: RegisteredCheckAttemptStatus,
    artifact_uri: Option<String>,
    unresolved_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct StagedRegisteredCheck {
    parameters_digest: String,
    target: String,
    descriptor: RegisteredCheckDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RegisteredCheckEvidence {
    schema: String,
    check_id: String,
    descriptor_version: i64,
    descriptor_digest: String,
    idempotency_key: String,
    exit_code: Option<i32>,
    stdout_utf8: String,
    stderr_utf8: String,
    elapsed_milliseconds: u64,
    observed_processes: usize,
    timed_out: bool,
    process_tree_escaped: bool,
    observed_write_paths: Vec<String>,
    network_attempted: bool,
    observed_file_digests: BTreeMap<String, String>,
    oracle_passed: bool,
}

pub(crate) struct NativeRegisteredCheckExecutor {
    trusted_fencing_epoch: i64,
    workspace_root: PathBuf,
    state_store: Arc<DurableExecutorStateStore>,
    artifact_store: ArtifactStore,
    runner: Arc<dyn RegisteredCheckRunner>,
    staged_requests: Mutex<BTreeMap<String, StagedRegisteredCheck>>,
}

impl NativeRegisteredCheckExecutor {
    pub(crate) fn new(
        trusted_fencing_epoch: i64,
        workspace_root: PathBuf,
        state_store: Arc<DurableExecutorStateStore>,
        artifact_store: ArtifactStore,
        runner: Arc<dyn RegisteredCheckRunner>,
    ) -> Result<Self, RegisteredCheckError> {
        state_store
            .ensure_outside_workspace(&workspace_root)
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
        Ok(Self {
            trusted_fencing_epoch,
            workspace_root,
            state_store,
            artifact_store,
            runner,
            staged_requests: Mutex::new(BTreeMap::new()),
        })
    }

    pub(crate) fn stage_request(
        &self,
        idempotency_key: String,
        parameters_digest: String,
        native_descriptor: &NativeToolDescriptor,
        request: &RegisteredCheckRunRequest,
    ) -> Result<(), RegisteredCheckError> {
        if native_descriptor.family != NativeOperationFamily::RegisteredCheckRun
            || native_descriptor.action != "run"
            || idempotency_key.is_empty()
            || parameters_digest.is_empty()
        {
            return Err(RegisteredCheckError::NativeDescriptorMismatch);
        }
        let descriptor = RegisteredCheckRegistry::production().resolve(request)?;
        let staged = StagedRegisteredCheck {
            parameters_digest: parameters_digest.clone(),
            target: descriptor.target(),
            descriptor: descriptor.clone(),
        };
        let state_guard = self
            .state_store
            .lock_key(CHECK_STATE_NAMESPACE, &idempotency_key)
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
        match state_guard
            .read::<RegisteredCheckStateRecord>()
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?
        {
            Some(existing) if !state_matches(&existing, &idempotency_key, &staged) => {
                return Err(RegisteredCheckError::IdempotencyBindingConflict);
            }
            Some(_) => {}
            None => {
                if state_guard.key_previously_seen() {
                    return Err(RegisteredCheckError::UnresolvedOutcome);
                }
                state_guard
                    .write(&state_record(
                        &idempotency_key,
                        &staged,
                        RegisteredCheckAttemptStatus::Staged,
                        None,
                        None,
                    ))
                    .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
            }
        }
        let mut staged_requests = self
            .staged_requests
            .lock()
            .map_err(|_| RegisteredCheckError::Infrastructure("staging map poisoned".into()))?;
        if let Some(existing) = staged_requests.get(&idempotency_key) {
            if existing.parameters_digest != staged.parameters_digest
                || existing.target != staged.target
                || existing.descriptor != staged.descriptor
            {
                return Err(RegisteredCheckError::IdempotencyBindingConflict);
            }
            return Ok(());
        }
        staged_requests.insert(idempotency_key, staged);
        Ok(())
    }

    pub(crate) fn artifact_uri(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<String>, RegisteredCheckError> {
        let state_guard = self
            .state_store
            .lock_key(CHECK_STATE_NAMESPACE, idempotency_key)
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
        let record = state_guard
            .read::<RegisteredCheckStateRecord>()
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
        Ok(record.and_then(|record| {
            (record.status == RegisteredCheckAttemptStatus::Completed)
                .then_some(record.artifact_uri)
                .flatten()
        }))
    }

    #[cfg(test)]
    fn mark_attempting_for_test(&self, idempotency_key: &str) -> Result<(), RegisteredCheckError> {
        let staged = self
            .staged_requests
            .lock()
            .map_err(|_| RegisteredCheckError::Infrastructure("staging map poisoned".into()))?
            .get(idempotency_key)
            .cloned()
            .ok_or(RegisteredCheckError::UnresolvedOutcome)?;
        let guard = self
            .state_store
            .lock_key(CHECK_STATE_NAMESPACE, idempotency_key)
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
        guard
            .write(&state_record(
                idempotency_key,
                &staged,
                RegisteredCheckAttemptStatus::Attempting,
                None,
                None,
            ))
            .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))
    }
}

impl EffectExecutor for NativeRegisteredCheckExecutor {
    fn capabilities(&self) -> ExecutorCapabilities {
        ExecutorCapabilities {
            queryable: true,
            idempotent: true,
        }
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        if call.fencing_epoch != self.trusted_fencing_epoch {
            return Ok(DispatchOutcome::FencedStaleEpoch {
                sink_epoch: self.trusted_fencing_epoch,
            });
        }
        let staged = self
            .staged_requests
            .lock()
            .map_err(|_| port_failure("registered check staging map is poisoned"))?
            .get(&call.idempotency_key)
            .cloned();
        let Some(staged) = staged else {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "no daemon-staged registered check for original key".into(),
            });
        };
        if call.action != "run"
            || call.target != staged.target
            || call.parameters_digest != staged.parameters_digest
        {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "dispatch does not match daemon-staged registered check".into(),
            });
        }
        RegisteredCheckRegistry::production()
            .validate_exact(&staged.descriptor)
            .map_err(|error| port_failure(error.to_string()))?;
        let state_guard = self
            .state_store
            .lock_key(CHECK_STATE_NAMESPACE, &call.idempotency_key)
            .map_err(|error| port_failure(error.to_string()))?;
        let existing = state_guard
            .read::<RegisteredCheckStateRecord>()
            .map_err(|error| port_failure(error.to_string()))?
            .ok_or_else(|| port_failure("registered check state is missing"))?;
        if !state_matches(&existing, &call.idempotency_key, &staged) {
            return Err(port_failure("registered check state binding drifted"));
        }
        match existing.status {
            RegisteredCheckAttemptStatus::Completed => {
                let receipt_ref = existing
                    .artifact_uri
                    .ok_or_else(|| port_failure("completed check has no artifact"))?;
                return Ok(DispatchOutcome::Executed { receipt_ref });
            }
            RegisteredCheckAttemptStatus::Attempting | RegisteredCheckAttemptStatus::Unresolved => {
                return Ok(DispatchOutcome::Unknown {
                    detail: "registered check outcome requires original-key reconciliation".into(),
                });
            }
            RegisteredCheckAttemptStatus::Staged => {}
        }
        state_guard
            .write(&state_record(
                &call.idempotency_key,
                &staged,
                RegisteredCheckAttemptStatus::Attempting,
                None,
                None,
            ))
            .map_err(|error| port_failure(error.to_string()))?;
        let observation = match self.runner.run(&staged.descriptor, &self.workspace_root) {
            Ok(observation) => observation,
            Err(error) => {
                state_guard
                    .write(&state_record(
                        &call.idempotency_key,
                        &staged,
                        RegisteredCheckAttemptStatus::Unresolved,
                        None,
                        Some(error.to_string()),
                    ))
                    .map_err(|write_error| port_failure(write_error.to_string()))?;
                return Ok(DispatchOutcome::Unknown {
                    detail: error.to_string(),
                });
            }
        };
        if let Some(reason) = observation_boundary_violation(&staged.descriptor, &observation) {
            state_guard
                .write(&state_record(
                    &call.idempotency_key,
                    &staged,
                    RegisteredCheckAttemptStatus::Unresolved,
                    None,
                    Some(reason.clone()),
                ))
                .map_err(|error| port_failure(error.to_string()))?;
            return Ok(DispatchOutcome::Unknown { detail: reason });
        }
        let oracle_passed =
            observation.observed_file_digests == staged.descriptor.expected_file_digests;
        let evidence = RegisteredCheckEvidence {
            schema: CHECK_EVIDENCE_SCHEMA.to_owned(),
            check_id: staged.descriptor.check_id.clone(),
            descriptor_version: staged.descriptor.descriptor_version,
            descriptor_digest: staged.descriptor.descriptor_digest.clone(),
            idempotency_key: call.idempotency_key.clone(),
            exit_code: observation.exit_code,
            stdout_utf8: String::from_utf8_lossy(&observation.stdout).into_owned(),
            stderr_utf8: String::from_utf8_lossy(&observation.stderr).into_owned(),
            elapsed_milliseconds: observation.elapsed_milliseconds,
            observed_processes: observation.observed_processes,
            timed_out: observation.timed_out,
            process_tree_escaped: observation.process_tree_escaped,
            observed_write_paths: observation.observed_write_paths,
            network_attempted: observation.network_attempted,
            observed_file_digests: observation.observed_file_digests,
            oracle_passed,
        };
        let evidence_bytes =
            serde_json::to_vec(&evidence).map_err(|error| port_failure(error.to_string()))?;
        let reference = format!("sha256:{:x}", Sha256::digest(&evidence_bytes));
        let metadata = self
            .artifact_store
            .put_with_metadata(
                &reference,
                &evidence_bytes,
                "application/vnd.cognitiveos.registered-check-evidence+json",
            )
            .map_err(|error| port_failure(error.to_string()))?;
        let digest = metadata
            .reference
            .strip_prefix("sha256:")
            .ok_or_else(|| port_failure("ArtifactStore returned malformed reference"))?;
        let artifact_uri = format!("artifact://sha256/{digest}");
        state_guard
            .write(&state_record(
                &call.idempotency_key,
                &staged,
                RegisteredCheckAttemptStatus::Completed,
                Some(artifact_uri.clone()),
                None,
            ))
            .map_err(|error| port_failure(error.to_string()))?;
        Ok(DispatchOutcome::Executed {
            receipt_ref: artifact_uri,
        })
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        let state_guard = self
            .state_store
            .lock_key(CHECK_STATE_NAMESPACE, idempotency_key)
            .map_err(|error| port_failure(error.to_string()))?;
        let record = state_guard
            .read::<RegisteredCheckStateRecord>()
            .map_err(|error| port_failure(error.to_string()))?;
        Ok(match record {
            Some(record) if record.status == RegisteredCheckAttemptStatus::Completed => {
                let Some(artifact_uri) = record.artifact_uri else {
                    return Ok(ExecutorQueryResult::Indeterminate);
                };
                match self.artifact_store.contains_artifact_uri(&artifact_uri) {
                    Ok(true) => ExecutorQueryResult::ExecutedWithOriginalKey,
                    Ok(false) | Err(_) => ExecutorQueryResult::Indeterminate,
                }
            }
            Some(record)
                if matches!(
                    record.status,
                    RegisteredCheckAttemptStatus::Attempting
                        | RegisteredCheckAttemptStatus::Unresolved
                ) =>
            {
                ExecutorQueryResult::Indeterminate
            }
            Some(_) => ExecutorQueryResult::NotExecuted,
            None if state_guard.key_previously_seen() => ExecutorQueryResult::Indeterminate,
            None => ExecutorQueryResult::NotExecuted,
        })
    }
}

fn state_record(
    idempotency_key: &str,
    staged: &StagedRegisteredCheck,
    status: RegisteredCheckAttemptStatus,
    artifact_uri: Option<String>,
    unresolved_reason: Option<String>,
) -> RegisteredCheckStateRecord {
    RegisteredCheckStateRecord {
        schema: CHECK_STATE_SCHEMA.to_owned(),
        idempotency_key: idempotency_key.to_owned(),
        parameters_digest: staged.parameters_digest.clone(),
        target: staged.target.clone(),
        check_id: staged.descriptor.check_id.clone(),
        descriptor_version: staged.descriptor.descriptor_version,
        descriptor_digest: staged.descriptor.descriptor_digest.clone(),
        status,
        artifact_uri,
        unresolved_reason,
    }
}

fn state_matches(
    record: &RegisteredCheckStateRecord,
    idempotency_key: &str,
    staged: &StagedRegisteredCheck,
) -> bool {
    record.schema == CHECK_STATE_SCHEMA
        && record.idempotency_key == idempotency_key
        && record.parameters_digest == staged.parameters_digest
        && record.target == staged.target
        && record.check_id == staged.descriptor.check_id
        && record.descriptor_version == staged.descriptor.descriptor_version
        && record.descriptor_digest == staged.descriptor.descriptor_digest
}

fn observation_boundary_violation(
    descriptor: &RegisteredCheckDescriptor,
    observation: &RegisteredCheckObservation,
) -> Option<String> {
    if observation.timed_out {
        return Some("registered check timed out".into());
    }
    if observation
        .stdout
        .len()
        .saturating_add(observation.stderr.len())
        > descriptor.output_limit_bytes
    {
        return Some("registered check output exceeded its bound".into());
    }
    if observation.observed_processes > descriptor.maximum_processes
        || observation.process_tree_escaped
    {
        return Some("registered check process tree escaped its bound".into());
    }
    if observation.network_attempted || descriptor.network_policy != RegisteredNetworkPolicy::Denied
    {
        return Some("registered check attempted network access".into());
    }
    if observation.observed_write_paths.iter().any(|path| {
        !descriptor
            .allowed_write_roots
            .iter()
            .any(|root| path == root)
    }) {
        return Some("registered check wrote outside its registered roots".into());
    }
    None
}

pub(crate) fn verify_registered_check_artifact(
    artifact_store: &ArtifactStore,
    artifact_uri: &str,
) -> Result<bool, RegisteredCheckError> {
    let digest = artifact_uri
        .strip_prefix("artifact://sha256/")
        .ok_or_else(|| RegisteredCheckError::InvalidEvidence("artifact URI is invalid".into()))?;
    let bytes = artifact_store
        .get(&format!("sha256:{digest}"))
        .map_err(|error| RegisteredCheckError::InvalidEvidence(error.to_string()))?
        .ok_or_else(|| RegisteredCheckError::InvalidEvidence("artifact is missing".into()))?;
    let evidence: RegisteredCheckEvidence = serde_json::from_slice(&bytes)
        .map_err(|error| RegisteredCheckError::InvalidEvidence(error.to_string()))?;
    if evidence.schema != CHECK_EVIDENCE_SCHEMA {
        return Err(RegisteredCheckError::InvalidEvidence(
            "evidence schema drifted".into(),
        ));
    }
    let descriptor = RegisteredCheckRegistry::production()
        .resolve(&RegisteredCheckRunRequest::new(&evidence.check_id))?;
    let exact_binding = evidence.descriptor_version == descriptor.descriptor_version
        && evidence.descriptor_digest == descriptor.descriptor_digest
        && evidence.observed_file_digests == descriptor.expected_file_digests;
    let safe = evidence.exit_code == Some(0)
        && evidence.oracle_passed
        && !evidence.timed_out
        && !evidence.process_tree_escaped
        && evidence.observed_processes <= descriptor.maximum_processes
        && evidence.observed_write_paths.is_empty()
        && !evidence.network_attempted;
    Ok(exact_binding && safe)
}

pub(crate) struct RegisteredCheckWorkerOutcome {
    pub bytes: Vec<u8>,
    pub passed: bool,
}

/// 仅供当前二进制的固定 helper 参数调用；不解析额外 argv、env 或 cwd。
pub(crate) fn run_registered_check_worker(
    check_id: &str,
    workspace_root: &Path,
) -> Result<RegisteredCheckWorkerOutcome, RegisteredCheckError> {
    let descriptor =
        RegisteredCheckRegistry::production().resolve(&RegisteredCheckRunRequest::new(check_id))?;
    let observed = observed_oracle_files(&descriptor, workspace_root)?;
    let passed = observed == descriptor.expected_file_digests;
    let bytes = cognitive_contracts::canonical::canonical_bytes_of_value(&serde_json::json!({
        "check_id": descriptor.check_id,
        "descriptor_digest": descriptor.descriptor_digest,
        "passed": passed,
        "schema": "personal-registered-check-worker/0.1",
    }))
    .map_err(|error| RegisteredCheckError::Infrastructure(error.to_string()))?;
    Ok(RegisteredCheckWorkerOutcome { bytes, passed })
}

fn port_failure(detail: impl Into<String>) -> PortFailure {
    PortFailure {
        detail: detail.into(),
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum RegisteredCheckError {
    #[error("registered check request is invalid: {0}")]
    InvalidRequest(String),
    #[error("registered check id is invalid")]
    InvalidCheckId,
    #[error("registered check is unknown: {0}")]
    UnknownCheck(String),
    #[error("registered check descriptor version drifted: {check_id}")]
    DescriptorVersionDrift { check_id: String },
    #[error("registered check descriptor drifted: {check_id}")]
    DescriptorDrift { check_id: String },
    #[error("native Tool descriptor does not name RegisteredCheckRun")]
    NativeDescriptorMismatch,
    #[error("registered check idempotency key is bound to a different request")]
    IdempotencyBindingConflict,
    #[error("registered check output exceeds its immutable ceiling")]
    OutputTooLarge,
    #[error("registered check workspace boundary failed: {0}")]
    WorkspaceBoundaryViolation(String),
    #[error("registered check outcome remains unresolved")]
    UnresolvedOutcome,
    #[error("registered check evidence is invalid: {0}")]
    InvalidEvidence(String),
    #[error("registered check infrastructure is unavailable: {0}")]
    Infrastructure(String),
}

#[cfg(test)]
mod tests;
