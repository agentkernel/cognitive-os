//! Daemon-private native Tool execution admission and bounded request shape.
//!
//! This module does not grant authority and does not replace Intent/Effect
//! persistence; it converts an already daemon-bound Tool descriptor into a
//! request the persistent protocol can safely execute.

#![allow(unused)] // Some daemon-private executor families remain deferred.

use cognitive_domain::{ObjectId, Version};
use cognitive_kernel::tool_registry::{NativeOperationFamily, NativeToolDescriptor, ToolRisk};
use cognitive_kernel::{
    authz::AuthorizationGrant,
    effects::{EffectError, EffectProtocol, GovernanceCurrency, WriterLease},
    engine::CommittedTransition,
    executor::{
        DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
    },
    ports::PortFailure,
};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use thiserror::Error;

/// Bounded daemon-private input for one native Tool operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NativeToolExecutionRequest {
    pub descriptor: NativeToolDescriptor,
    pub target: String,
    pub input: Vec<u8>,
    pub workspace_root: Option<PathBuf>,
}

/// Validated execution input. The validated target is still an observation
/// boundary; authority, budget, fencing, and Effect state remain server-side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedNativeToolRequest {
    pub descriptor: NativeToolDescriptor,
    pub target: String,
    pub input: Vec<u8>,
    pub approved_workspace_root: Option<PathBuf>,
    pub resolved_workspace_path: Option<PathBuf>,
}

/// Monotonic cursor over bounded output chunks. A cursor never exposes more
/// than the descriptor ceiling and cannot replay an already acknowledged
/// chunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundedOutputCursor {
    output_limit_bytes: usize,
    next_offset: usize,
}

/// Daemon-private workspace-read sink for a request already sealed by the
/// scheduler admission path. Staging a request does not authorize execution:
/// the Effect protocol records `EXECUTING` before it calls this adapter.
///
/// The adapter retains only redacted, bounded bytes under the original
/// idempotency key. This provides a queryable, idempotent sink for recovery
/// without treating Tool output as evidence, verification, or Task progress.
pub(crate) struct NativeWorkspaceReadExecutor {
    trusted_fencing_epoch: i64,
    staged_requests: Mutex<BTreeMap<String, StagedWorkspaceReadRequest>>,
    completed_reads: Mutex<BTreeMap<String, CompletedWorkspaceRead>>,
    #[cfg(test)]
    before_read_hook: Mutex<Option<Box<dyn Fn() + Send>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StagedWorkspaceReadRequest {
    parameters_digest: String,
    target: String,
    approved_workspace_root: PathBuf,
    resolved_workspace_path: PathBuf,
    output_limit_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletedWorkspaceRead {
    receipt_ref: String,
    redacted_output: Vec<u8>,
}

impl NativeWorkspaceReadExecutor {
    pub(crate) fn new(trusted_fencing_epoch: i64) -> Self {
        Self {
            trusted_fencing_epoch,
            staged_requests: Mutex::new(BTreeMap::new()),
            completed_reads: Mutex::new(BTreeMap::new()),
            #[cfg(test)]
            before_read_hook: Mutex::new(None),
        }
    }

    /// Bind one validated workspace-read request to its durable Intent
    /// identity. The caller supplies the digest from the sealed Intent; a
    /// later dispatch must match every bound field before filesystem access.
    pub(crate) fn stage_request(
        &self,
        idempotency_key: String,
        parameters_digest: String,
        request: &ValidatedNativeToolRequest,
    ) -> Result<(), NativeToolExecutionError> {
        if request.descriptor.family != NativeOperationFamily::WorkspaceRead {
            return Err(NativeToolExecutionError::UnsupportedExecutionFamily);
        }
        let resolved_workspace_path = request
            .resolved_workspace_path
            .as_ref()
            .ok_or(NativeToolExecutionError::WorkspaceTargetRequired)?;
        let approved_workspace_root = request
            .approved_workspace_root
            .as_ref()
            .ok_or(NativeToolExecutionError::WorkspaceTargetRequired)?;
        if idempotency_key.is_empty() || parameters_digest.is_empty() {
            return Err(NativeToolExecutionError::InvalidDescriptor(
                "idempotency key and parameters digest are required".to_owned(),
            ));
        }
        let staged_request = StagedWorkspaceReadRequest {
            parameters_digest,
            target: request.target.clone(),
            approved_workspace_root: approved_workspace_root.clone(),
            resolved_workspace_path: resolved_workspace_path.clone(),
            output_limit_bytes: request.descriptor.output_limit_bytes,
        };
        let mut staged_requests = self.staged_requests.lock().map_err(|_| {
            NativeToolExecutionError::ExecutorUnavailable(
                "staged request store is poisoned".to_owned(),
            )
        })?;
        if let Some(existing_request) = staged_requests.get(&idempotency_key) {
            if existing_request != &staged_request {
                return Err(NativeToolExecutionError::IdempotencyBindingConflict);
            }
            return Ok(());
        }
        staged_requests.insert(idempotency_key, staged_request);
        Ok(())
    }

    #[cfg(test)]
    fn completed_output(&self, idempotency_key: &str) -> Option<Vec<u8>> {
        self.completed_reads
            .lock()
            .ok()
            .and_then(|completed_reads| completed_reads.get(idempotency_key).cloned())
            .map(|completed_read| completed_read.redacted_output)
    }

    #[cfg(test)]
    fn install_before_read_hook(&self, hook: impl Fn() + Send + 'static) {
        let mut before_read_hook = match self.before_read_hook.lock() {
            Ok(before_read_hook) => before_read_hook,
            Err(poisoned_before_read_hook) => poisoned_before_read_hook.into_inner(),
        };
        *before_read_hook = Some(Box::new(hook));
    }

    fn read_staged_workspace_file(
        &self,
        call: &ExecutorCall,
        staged_request: &StagedWorkspaceReadRequest,
    ) -> Result<DispatchOutcome, PortFailure> {
        if call.action != "read"
            || call.target != staged_request.target
            || call.parameters_digest != staged_request.parameters_digest
        {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "dispatch does not match the daemon-staged workspace read".to_owned(),
            });
        }
        let canonical_workspace_root =
            std::fs::canonicalize(&staged_request.approved_workspace_root).map_err(|error| {
                PortFailure {
                    detail: format!("workspace root resolution failed: {error}"),
                }
            })?;
        let canonical_target_path = std::fs::canonicalize(&staged_request.resolved_workspace_path)
            .map_err(|error| PortFailure {
                detail: format!("workspace target resolution failed: {error}"),
            })?;
        if !canonical_target_path.starts_with(&canonical_workspace_root) {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "workspace target escaped the approved root after resolution".to_owned(),
            });
        }
        #[cfg(test)]
        let before_read_hook = self
            .before_read_hook
            .lock()
            .map_err(|_| PortFailure {
                detail: "before-read hook store is poisoned".to_owned(),
            })?
            .take();
        #[cfg(test)]
        if let Some(before_read_hook) = before_read_hook {
            before_read_hook();
        }
        // Hold the completed-result ledger lock through the filesystem read so
        // concurrent calls for one idempotency key cannot both execute it.
        let mut completed_reads = self.completed_reads.lock().map_err(|_| PortFailure {
            detail: "completed read store is poisoned".to_owned(),
        })?;
        if let Some(existing_read) = completed_reads.get(&call.idempotency_key) {
            return Ok(DispatchOutcome::Executed {
                receipt_ref: existing_read.receipt_ref.clone(),
            });
        }
        let raw_output = std::fs::read(&canonical_target_path).map_err(|error| PortFailure {
            detail: format!("workspace read failed: {error}"),
        })?;
        let redacted_output = redact_sensitive_output(&String::from_utf8_lossy(&raw_output))
            .into_bytes()
            .into_iter()
            .take(staged_request.output_limit_bytes)
            .collect::<Vec<_>>();
        let receipt_ref = format!("tool-receipt://workspace-read/{}", call.idempotency_key);
        let completed_read = CompletedWorkspaceRead {
            receipt_ref: receipt_ref.clone(),
            redacted_output,
        };
        completed_reads.insert(call.idempotency_key.clone(), completed_read);
        Ok(DispatchOutcome::Executed { receipt_ref })
    }
}

impl EffectExecutor for NativeWorkspaceReadExecutor {
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
        let staged_request = self
            .staged_requests
            .lock()
            .map_err(|_| PortFailure {
                detail: "staged request store is poisoned".to_owned(),
            })?
            .get(&call.idempotency_key)
            .cloned();
        let Some(staged_request) = staged_request else {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "no daemon-staged request for idempotency key".to_owned(),
            });
        };
        self.read_staged_workspace_file(call, &staged_request)
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        let completed_reads = self.completed_reads.lock().map_err(|_| PortFailure {
            detail: "completed read store is poisoned".to_owned(),
        })?;
        Ok(if completed_reads.contains_key(idempotency_key) {
            ExecutorQueryResult::ExecutedWithOriginalKey
        } else {
            ExecutorQueryResult::NotExecuted
        })
    }
}

/// A daemon-private process observation port. The supervisor owns process
/// lifetime and timeout decisions; this executor never discovers or launches
/// operating-system processes itself.
pub(crate) trait ProcessCheckSupervisor: Send + Sync {
    fn check_process(
        &self,
        process_id: u32,
        timeout: Duration,
    ) -> Result<Vec<u8>, ProcessCheckSupervisorError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcessCheckSupervisorError {
    NotRegistered,
    Orphaned,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StagedProcessCheckRequest {
    parameters_digest: String,
    target: String,
    process_id: u32,
    output_limit_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletedProcessCheck {
    receipt_ref: String,
    redacted_output: Vec<u8>,
}

/// Testable in-process supervisor registry. Production wiring can replace it
/// with the existing daemon supervisor without changing the Effect boundary.
pub(crate) struct BoundedProcessCheckSupervisor {
    maximum_timeout: Duration,
    registered_processes: Mutex<BTreeMap<u32, RegisteredProcess>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegisteredProcess {
    output: Vec<u8>,
    required_runtime: Duration,
    alive: bool,
}

impl BoundedProcessCheckSupervisor {
    pub(crate) fn new(maximum_timeout: Duration) -> Self {
        Self {
            maximum_timeout,
            registered_processes: Mutex::new(BTreeMap::new()),
        }
    }

    #[cfg(test)]
    fn register(&self, process_id: u32, output: &[u8], required_runtime: Duration) {
        let mut registered_processes = match self.registered_processes.lock() {
            Ok(registered_processes) => registered_processes,
            Err(poisoned_registered_processes) => poisoned_registered_processes.into_inner(),
        };
        registered_processes.insert(
            process_id,
            RegisteredProcess {
                output: output.to_vec(),
                required_runtime,
                alive: true,
            },
        );
    }

    #[cfg(test)]
    fn orphan(&self, process_id: u32) {
        let mut registered_processes = match self.registered_processes.lock() {
            Ok(registered_processes) => registered_processes,
            Err(poisoned_registered_processes) => poisoned_registered_processes.into_inner(),
        };
        if let Some(process) = registered_processes.get_mut(&process_id) {
            process.alive = false;
        }
    }
}

impl ProcessCheckSupervisor for BoundedProcessCheckSupervisor {
    fn check_process(
        &self,
        process_id: u32,
        timeout: Duration,
    ) -> Result<Vec<u8>, ProcessCheckSupervisorError> {
        if timeout.is_zero() || timeout > self.maximum_timeout {
            return Err(ProcessCheckSupervisorError::TimedOut);
        }
        let registered_processes = self
            .registered_processes
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?;
        let Some(process) = registered_processes.get(&process_id) else {
            return Err(ProcessCheckSupervisorError::NotRegistered);
        };
        if !process.alive {
            return Err(ProcessCheckSupervisorError::Orphaned);
        }
        if process.required_runtime > timeout {
            return Err(ProcessCheckSupervisorError::TimedOut);
        }
        Ok(process.output.clone())
    }
}

/// Daemon-private, read-only process/check Effect sink. It has no Task
/// lifecycle input and therefore cannot report Task progress or completion.
pub(crate) struct NativeProcessCheckExecutor<S> {
    trusted_fencing_epoch: i64,
    supervisor: Arc<S>,
    timeout: Duration,
    staged_requests: Mutex<BTreeMap<String, StagedProcessCheckRequest>>,
    completed_checks: Mutex<BTreeMap<String, CompletedProcessCheck>>,
}

impl<S> NativeProcessCheckExecutor<S>
where
    S: ProcessCheckSupervisor,
{
    pub(crate) fn new(trusted_fencing_epoch: i64, supervisor: Arc<S>, timeout: Duration) -> Self {
        Self {
            trusted_fencing_epoch,
            supervisor,
            timeout,
            staged_requests: Mutex::new(BTreeMap::new()),
            completed_checks: Mutex::new(BTreeMap::new()),
        }
    }

    pub(crate) fn stage_request(
        &self,
        idempotency_key: String,
        parameters_digest: String,
        request: &ValidatedNativeToolRequest,
    ) -> Result<(), NativeToolExecutionError> {
        if request.descriptor.family != NativeOperationFamily::ProcessCheck {
            return Err(NativeToolExecutionError::UnsupportedExecutionFamily);
        }
        let process_id = parse_process_id(&request.target)?;
        if idempotency_key.is_empty() || parameters_digest.is_empty() {
            return Err(NativeToolExecutionError::InvalidDescriptor(
                "idempotency key and parameters digest are required".to_owned(),
            ));
        }
        let staged_request = StagedProcessCheckRequest {
            parameters_digest,
            target: request.target.clone(),
            process_id,
            output_limit_bytes: request.descriptor.output_limit_bytes,
        };
        let mut staged_requests = self.staged_requests.lock().map_err(|_| {
            NativeToolExecutionError::ExecutorUnavailable(
                "staged process store is poisoned".to_owned(),
            )
        })?;
        if let Some(existing_request) = staged_requests.get(&idempotency_key) {
            if existing_request != &staged_request {
                return Err(NativeToolExecutionError::IdempotencyBindingConflict);
            }
            return Ok(());
        }
        staged_requests.insert(idempotency_key, staged_request);
        Ok(())
    }

    #[cfg(test)]
    fn completed_output(&self, idempotency_key: &str) -> Option<Vec<u8>> {
        self.completed_checks
            .lock()
            .ok()
            .and_then(|checks| checks.get(idempotency_key).cloned())
            .map(|check| check.redacted_output)
    }
}

impl<S> EffectExecutor for NativeProcessCheckExecutor<S>
where
    S: ProcessCheckSupervisor,
{
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
        let staged_request = self
            .staged_requests
            .lock()
            .map_err(|_| PortFailure {
                detail: "staged process store is poisoned".to_owned(),
            })?
            .get(&call.idempotency_key)
            .cloned();
        let Some(staged_request) = staged_request else {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "no daemon-staged process check for idempotency key".to_owned(),
            });
        };
        if call.action != "check"
            || call.target != staged_request.target
            || call.parameters_digest != staged_request.parameters_digest
        {
            return Ok(DispatchOutcome::NotExecuted {
                reason: "dispatch does not match the daemon-staged process check".to_owned(),
            });
        }
        let mut completed_checks = self.completed_checks.lock().map_err(|_| PortFailure {
            detail: "completed process store is poisoned".to_owned(),
        })?;
        if let Some(existing_check) = completed_checks.get(&call.idempotency_key) {
            return Ok(DispatchOutcome::Executed {
                receipt_ref: existing_check.receipt_ref.clone(),
            });
        }
        let raw_output = self
            .supervisor
            .check_process(staged_request.process_id, self.timeout)
            .map_err(|error| PortFailure {
                detail: format!("bounded process check failed: {error:?}"),
            })?;
        let redacted_output = redact_sensitive_output(&String::from_utf8_lossy(&raw_output))
            .into_bytes()
            .into_iter()
            .take(staged_request.output_limit_bytes)
            .collect();
        let receipt_ref = format!("tool-receipt://process-check/{}", call.idempotency_key);
        completed_checks.insert(
            call.idempotency_key.clone(),
            CompletedProcessCheck {
                receipt_ref: receipt_ref.clone(),
                redacted_output,
            },
        );
        Ok(DispatchOutcome::Executed { receipt_ref })
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        let completed_checks = self.completed_checks.lock().map_err(|_| PortFailure {
            detail: "completed process store is poisoned".to_owned(),
        })?;
        Ok(if completed_checks.contains_key(idempotency_key) {
            ExecutorQueryResult::ExecutedWithOriginalKey
        } else {
            ExecutorQueryResult::NotExecuted
        })
    }
}

fn parse_process_id(target: &str) -> Result<u32, NativeToolExecutionError> {
    target
        .strip_prefix("process://")
        .ok_or(NativeToolExecutionError::InvalidProcessTarget)?
        .parse::<u32>()
        .map_err(|_| NativeToolExecutionError::InvalidProcessTarget)
}

/// Drive a staged process observation through the durable Effect protocol.
pub(crate) fn dispatch_staged_process_check_effect<S, C, G, P>(
    effect_protocol: &EffectProtocol<'_, S, C, G>,
    effect_object_id: &ObjectId,
    expected_effect_version: Version,
    grant: &AuthorizationGrant,
    governance_currency: &GovernanceCurrency,
    executor: &NativeProcessCheckExecutor<P>,
    writer_lease: &WriterLease,
) -> Result<CommittedTransition, EffectError>
where
    S: cognitive_kernel::ports::AuthorityStore + cognitive_kernel::ports::ProtocolStore,
    C: cognitive_kernel::ports::Clock,
    G: cognitive_kernel::ports::IdGenerator,
    P: ProcessCheckSupervisor,
{
    let authorized = effect_protocol.authorize_effect(
        effect_object_id,
        expected_effect_version,
        grant,
        governance_currency,
        writer_lease,
    )?;
    let (dispatched, outcome) = effect_protocol.dispatch_effect(
        effect_object_id,
        authorized.after_version,
        grant,
        governance_currency,
        executor,
        writer_lease,
    )?;
    effect_protocol.record_outcome(
        effect_object_id,
        dispatched.after_version,
        &outcome,
        writer_lease,
    )
}

/// Drive an already staged workspace read through the durable Effect protocol.
///
/// Staging binds the native request to the Intent's idempotency key and
/// parameter digest; this function is the only adapter path that can invoke
/// it. The protocol records `EXECUTING` before filesystem access and records
/// the executor outcome afterwards. It deliberately has no Task, Loop,
/// progress, evidence, or completion inputs, so a read cannot be mistaken for
/// a Task outcome.
pub(crate) fn dispatch_staged_workspace_read_effect<S, C, G>(
    effect_protocol: &EffectProtocol<'_, S, C, G>,
    effect_object_id: &ObjectId,
    expected_effect_version: Version,
    grant: &AuthorizationGrant,
    governance_currency: &GovernanceCurrency,
    executor: &NativeWorkspaceReadExecutor,
    writer_lease: &WriterLease,
) -> Result<CommittedTransition, EffectError>
where
    S: cognitive_kernel::ports::AuthorityStore + cognitive_kernel::ports::ProtocolStore,
    C: cognitive_kernel::ports::Clock,
    G: cognitive_kernel::ports::IdGenerator,
{
    let authorized = effect_protocol.authorize_effect(
        effect_object_id,
        expected_effect_version,
        grant,
        governance_currency,
        writer_lease,
    )?;
    let (dispatched, outcome) = effect_protocol.dispatch_effect(
        effect_object_id,
        authorized.after_version,
        grant,
        governance_currency,
        executor,
        writer_lease,
    )?;
    effect_protocol.record_outcome(
        effect_object_id,
        dispatched.after_version,
        &outcome,
        writer_lease,
    )
}

impl BoundedOutputCursor {
    pub(crate) fn new(output_limit_bytes: usize) -> Self {
        Self {
            output_limit_bytes,
            next_offset: 0,
        }
    }

    pub(crate) fn next_chunk(
        &mut self,
        output: &[u8],
        requested_offset: usize,
        maximum_chunk_bytes: usize,
    ) -> Result<Option<(usize, Vec<u8>)>, NativeToolExecutionError> {
        if requested_offset != self.next_offset {
            return Err(NativeToolExecutionError::InvalidDescriptor(
                "output cursor is stale or out of order".to_owned(),
            ));
        }
        let bounded_output = &output[..output.len().min(self.output_limit_bytes)];
        if self.next_offset >= bounded_output.len() {
            return Ok(None);
        }
        let chunk_end = (self.next_offset + maximum_chunk_bytes.max(1)).min(bounded_output.len());
        let chunk = bounded_output[self.next_offset..chunk_end].to_vec();
        let chunk_offset = self.next_offset;
        self.next_offset = chunk_end;
        Ok(Some((chunk_offset, chunk)))
    }
}

/// Redact values that must never enter ordinary Tool output or evidence.
pub(crate) fn redact_sensitive_output(output: &str) -> String {
    let mut redacted_output = output.to_owned();
    for sensitive_marker in ["api_key=", "API_KEY=", "token=", "TOKEN="] {
        let mut search_start = 0;
        while let Some(relative_marker_start) =
            redacted_output[search_start..].find(sensitive_marker)
        {
            let marker_start = search_start + relative_marker_start;
            let value_start = marker_start + sensitive_marker.len();
            let value_end = redacted_output[value_start..]
                .find([' ', '\n', '\r', '&'])
                .map_or(redacted_output.len(), |relative_end| {
                    value_start + relative_end
                });
            redacted_output.replace_range(value_start..value_end, "[REDACTED]");
            // Continue after the replacement. Searching from the beginning
            // would rediscover the marker we intentionally retain for context.
            search_start = value_start + "[REDACTED]".len();
        }
    }
    redacted_output
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum NativeToolExecutionError {
    #[error("descriptor binding is invalid: {0}")]
    InvalidDescriptor(String),
    #[error("tool input exceeds descriptor limit: {observed} > {limit}")]
    InputTooLarge { observed: usize, limit: usize },
    #[error("workspace target is required for this operation")]
    WorkspaceTargetRequired,
    #[error("workspace target escapes the daemon-approved workspace root")]
    WorkspaceTargetEscapesRoot,
    #[error("workspace mutation requires a non-empty input")]
    MutationInputRequired,
    #[error("network target must use HTTPS")]
    NetworkTargetMustUseHttps,
    #[error("network target contains credentials")]
    NetworkTargetContainsCredentials,
    #[error("process target is not a bounded process identifier")]
    InvalidProcessTarget,
    #[error("this executor only accepts its validated native operation family")]
    UnsupportedExecutionFamily,
    #[error("idempotency key is already bound to a different native operation")]
    IdempotencyBindingConflict,
    #[error("native Tool executor is unavailable: {0}")]
    ExecutorUnavailable(String),
}

pub(crate) fn validate_native_tool_request(
    request: &NativeToolExecutionRequest,
) -> Result<ValidatedNativeToolRequest, NativeToolExecutionError> {
    validate_descriptor(&request.descriptor)?;
    if request.input.len() > request.descriptor.input_limit_bytes {
        return Err(NativeToolExecutionError::InputTooLarge {
            observed: request.input.len(),
            limit: request.descriptor.input_limit_bytes,
        });
    }

    let resolved_workspace_path = match request.descriptor.family {
        NativeOperationFamily::WorkspaceRead
        | NativeOperationFamily::WorkspaceSearch
        | NativeOperationFamily::WorkspaceWrite
        | NativeOperationFamily::WorkspacePatch => Some(validate_workspace_target(
            &request.target,
            request.workspace_root.as_deref(),
            request.descriptor.risk,
            request.input.is_empty(),
        )?),
        NativeOperationFamily::ProcessCheck => {
            validate_process_target(&request.target)?;
            None
        }
        NativeOperationFamily::HttpFetchReadOnly => {
            validate_network_target(&request.target)?;
            None
        }
    };

    Ok(ValidatedNativeToolRequest {
        descriptor: request.descriptor.clone(),
        target: request.target.clone(),
        input: request.input.clone(),
        approved_workspace_root: request.workspace_root.clone(),
        resolved_workspace_path,
    })
}

fn validate_descriptor(descriptor: &NativeToolDescriptor) -> Result<(), NativeToolExecutionError> {
    if descriptor.operation_id.is_empty()
        || descriptor.action.is_empty()
        || descriptor.executor.is_empty()
        || descriptor.descriptor_version < 1
        || descriptor.descriptor_digest.is_empty()
    {
        return Err(NativeToolExecutionError::InvalidDescriptor(
            "descriptor identity and digest are required".to_owned(),
        ));
    }
    if !matches!(
        descriptor.availability,
        cognitive_kernel::tool_registry::ToolAvailability::Enabled
    ) {
        return Err(NativeToolExecutionError::InvalidDescriptor(
            "disabled or quarantined descriptor cannot execute".to_owned(),
        ));
    }
    Ok(())
}

fn validate_workspace_target(
    target: &str,
    workspace_root: Option<&Path>,
    risk: ToolRisk,
    input_is_empty: bool,
) -> Result<PathBuf, NativeToolExecutionError> {
    let root = workspace_root.ok_or(NativeToolExecutionError::WorkspaceTargetRequired)?;
    let relative_target = target
        .strip_prefix("workspace://")
        .ok_or(NativeToolExecutionError::WorkspaceTargetEscapesRoot)?;
    let relative_path = Path::new(relative_target);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(NativeToolExecutionError::WorkspaceTargetEscapesRoot);
    }
    if matches!(risk, ToolRisk::WorkspaceMutation) && input_is_empty {
        return Err(NativeToolExecutionError::MutationInputRequired);
    }
    Ok(root.join(relative_path))
}

fn validate_process_target(target: &str) -> Result<(), NativeToolExecutionError> {
    let process_id = target
        .strip_prefix("process://")
        .ok_or(NativeToolExecutionError::InvalidProcessTarget)?;
    if process_id.is_empty() || !process_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(NativeToolExecutionError::InvalidProcessTarget);
    }
    Ok(())
}

fn validate_network_target(target: &str) -> Result<(), NativeToolExecutionError> {
    let authority = target
        .strip_prefix("https://")
        .ok_or(NativeToolExecutionError::NetworkTargetMustUseHttps)?
        .split_once('/')
        .map_or(
            target.strip_prefix("https://").unwrap_or_default(),
            |(host, _)| host,
        );
    if authority.is_empty() || authority.contains('@') {
        return Err(NativeToolExecutionError::NetworkTargetContainsCredentials);
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use cognitive_domain::{EventId, LifecycleDomain, StateName, UriRef, WallTimestamp};
    use cognitive_kernel::authz::{
        AccessRequest, ActorChainFacts, AuthzSnapshot, MembershipFacts, ObjectGovernance,
        PrincipalFacts, authorize,
    };
    use cognitive_kernel::ports::{
        AuthorityStore, Clock, EventDraft, IntentRow, ObjectAdmission, ProtocolStore, StoredObject,
    };
    use cognitive_kernel::tool_registry::{BUILTIN_TOOL_CATALOG, ToolAvailability};
    use cognitive_store::{SqliteAuthorityStore, UuidV7Generator};
    use serde_json::json;
    use std::{
        collections::BTreeMap,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::{SystemTime, UNIX_EPOCH},
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
        let validated_request =
            validate_native_tool_request(&request).expect("valid process check");
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

    #[test]
    fn process_check_validates_bounded_target_and_executes_registered_observation() {
        let request = process_check_request();
        let validated_request =
            validate_native_tool_request(&request).expect("valid process check");
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
        let validated_request =
            validate_native_tool_request(&request).expect("valid process check");
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
        let (_supervisor, executor) = staged_process_executor(64);
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
                        expires: WallTimestamp::parse("2026-08-04T12:05:00Z")
                            .expect("valid lease end"),
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

        let matched_call =
            workspace_read_call("read-key-1", "digest-1", "workspace://notes.txt", 7);
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
        let validated_request =
            validate_native_tool_request(&request).expect("valid write request");
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
        let store =
            Arc::new(SqliteAuthorityStore::open(&database_path).expect("open authority store"));
        let task_object_id = object_id(501);
        let effect_object_id = object_id(502);
        let intent_object_id = object_id(503);
        let admitted_at =
            WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid admission time");

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
                        event_id: EventId::parse(&format!(
                            "00000000-0000-7000-a000-{event_id:012x}"
                        ))
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
        let validated_request =
            validate_native_tool_request(&request).expect("valid workspace read");
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
            UriRef::parse("authority://personal/effect-authority")
                .expect("valid authority reference"),
            UriRef::parse("correlation://personal/p2-t06-d02")
                .expect("valid correlation reference"),
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
        let store =
            Arc::new(SqliteAuthorityStore::open(&database_path).expect("open authority store"));
        let task_object_id = object_id(511);
        let effect_object_id = object_id(512);
        let intent_object_id = object_id(513);
        let admitted_at =
            WallTimestamp::parse("2026-08-04T12:02:00Z").expect("valid admission time");

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
                        event_id: EventId::parse(&format!(
                            "00000000-0000-7000-a000-{event_id:012x}"
                        ))
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
        let validated_request =
            validate_native_tool_request(&request).expect("valid workspace read");
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
            UriRef::parse("authority://personal/effect-authority")
                .expect("valid authority reference"),
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
}
