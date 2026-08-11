#![allow(unused, unused_imports)]

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

use super::*;

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
    ObservationUnavailable,
}

/// A daemon-owned source of observations for an already registered process.
///
/// The source does not grant authority and must not discover arbitrary PIDs.
/// Runtime wiring can provide a platform-specific implementation after it has
/// established process ownership and fencing. The default source below fails
/// closed rather than attempting an unsafe cross-platform PID attach.
pub(crate) trait ProcessObservationSource: Send + Sync {
    fn observe(
        &self,
        process_id: u32,
        timeout: Duration,
    ) -> Result<Vec<u8>, ProcessCheckSupervisorError>;
}

/// Production-safe default observation source. It deliberately has no OS
/// process access and therefore cannot accidentally trust an arbitrary PID.
pub(crate) struct FailClosedProcessObservationSource;

impl ProcessObservationSource for FailClosedProcessObservationSource {
    fn observe(
        &self,
        _process_id: u32,
        _timeout: Duration,
    ) -> Result<Vec<u8>, ProcessCheckSupervisorError> {
        Err(ProcessCheckSupervisorError::ObservationUnavailable)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProcessLifecycleState {
    Registered,
    Orphaned,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DaemonProcessRegistration {
    attempt_id: String,
    process_id: u32,
    fencing_epoch: i64,
    lifecycle: ProcessLifecycleState,
}

/// Daemon-private process supervisor with explicit ownership and fencing.
///
/// Registration is the only way a PID enters this supervisor. Every check
/// validates the registration, current fencing epoch, lifecycle, and timeout
/// before invoking the injected observation source. Output is bounded here,
/// before it crosses into the Effect executor.
pub(crate) struct DaemonProcessSupervisor<S> {
    maximum_timeout: Duration,
    maximum_output_bytes: usize,
    source: Arc<S>,
    registrations: Mutex<BTreeMap<u32, DaemonProcessRegistration>>,
    current_fencing_epoch: Mutex<i64>,
    is_shutdown: Mutex<bool>,
}

impl<S> DaemonProcessSupervisor<S>
where
    S: ProcessObservationSource,
{
    pub(crate) fn new(
        initial_fencing_epoch: i64,
        maximum_timeout: Duration,
        maximum_output_bytes: usize,
        source: Arc<S>,
    ) -> Self {
        Self {
            maximum_timeout,
            maximum_output_bytes,
            source,
            registrations: Mutex::new(BTreeMap::new()),
            current_fencing_epoch: Mutex::new(initial_fencing_epoch),
            is_shutdown: Mutex::new(false),
        }
    }

    pub(crate) fn register(
        &self,
        attempt_id: String,
        process_id: u32,
        fencing_epoch: i64,
    ) -> Result<(), ProcessCheckSupervisorError> {
        if attempt_id.is_empty() || process_id == 0 {
            return Err(ProcessCheckSupervisorError::NotRegistered);
        }
        if *self
            .is_shutdown
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?
        {
            return Err(ProcessCheckSupervisorError::Orphaned);
        }
        let current_fencing_epoch = *self
            .current_fencing_epoch
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?;
        if fencing_epoch != current_fencing_epoch {
            return Err(ProcessCheckSupervisorError::Orphaned);
        }
        let mut registrations = self
            .registrations
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?;
        if registrations.values().any(|registration| {
            registration.attempt_id == attempt_id
                && registration.process_id != process_id
                && registration.lifecycle != ProcessLifecycleState::Shutdown
        }) {
            return Err(ProcessCheckSupervisorError::Orphaned);
        }
        registrations.insert(
            process_id,
            DaemonProcessRegistration {
                attempt_id,
                process_id,
                fencing_epoch,
                lifecycle: ProcessLifecycleState::Registered,
            },
        );
        Ok(())
    }

    pub(crate) fn unregister(&self, process_id: u32) -> Result<(), ProcessCheckSupervisorError> {
        let mut registrations = self
            .registrations
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?;
        registrations.remove(&process_id);
        Ok(())
    }

    pub(crate) fn fence(&self, fencing_epoch: i64) -> Result<(), ProcessCheckSupervisorError> {
        if *self
            .is_shutdown
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?
        {
            return Err(ProcessCheckSupervisorError::Orphaned);
        }
        let mut current_fencing_epoch = self
            .current_fencing_epoch
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?;
        if fencing_epoch <= *current_fencing_epoch {
            return Err(ProcessCheckSupervisorError::Orphaned);
        }
        *current_fencing_epoch = fencing_epoch;
        let mut registrations = self
            .registrations
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?;
        for registration in registrations.values_mut() {
            registration.lifecycle = ProcessLifecycleState::Orphaned;
        }
        Ok(())
    }

    pub(crate) fn recover(
        &self,
        attempt_id: &str,
        process_id: u32,
        fencing_epoch: i64,
    ) -> Result<(), ProcessCheckSupervisorError> {
        self.register(attempt_id.to_owned(), process_id, fencing_epoch)
    }

    pub(crate) fn shutdown(&self) -> Result<(), ProcessCheckSupervisorError> {
        let mut is_shutdown = self
            .is_shutdown
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?;
        *is_shutdown = true;
        let mut registrations = self
            .registrations
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?;
        for registration in registrations.values_mut() {
            registration.lifecycle = ProcessLifecycleState::Shutdown;
        }
        Ok(())
    }
}

impl DaemonProcessSupervisor<FailClosedProcessObservationSource> {
    pub(crate) fn fail_closed(
        initial_fencing_epoch: i64,
        maximum_timeout: Duration,
        maximum_output_bytes: usize,
    ) -> Self {
        Self::new(
            initial_fencing_epoch,
            maximum_timeout,
            maximum_output_bytes,
            Arc::new(FailClosedProcessObservationSource),
        )
    }
}

impl<S> ProcessCheckSupervisor for DaemonProcessSupervisor<S>
where
    S: ProcessObservationSource,
{
    fn check_process(
        &self,
        process_id: u32,
        timeout: Duration,
    ) -> Result<Vec<u8>, ProcessCheckSupervisorError> {
        if timeout.is_zero() || timeout > self.maximum_timeout {
            return Err(ProcessCheckSupervisorError::TimedOut);
        }
        let current_fencing_epoch = *self
            .current_fencing_epoch
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?;
        let registration = self
            .registrations
            .lock()
            .map_err(|_| ProcessCheckSupervisorError::Orphaned)?
            .get(&process_id)
            .cloned()
            .ok_or(ProcessCheckSupervisorError::NotRegistered)?;
        if registration.process_id != process_id
            || registration.fencing_epoch != current_fencing_epoch
        {
            return Err(ProcessCheckSupervisorError::Orphaned);
        }
        if registration.lifecycle != ProcessLifecycleState::Registered {
            return Err(ProcessCheckSupervisorError::Orphaned);
        }
        let output = self.source.observe(process_id, timeout)?;
        Ok(output.into_iter().take(self.maximum_output_bytes).collect())
    }
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
    #[cfg(test)]
    access_count: std::sync::atomic::AtomicUsize,
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
            #[cfg(test)]
            access_count: std::sync::atomic::AtomicUsize::new(0),
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

    #[cfg(test)]
    fn access_count(&self) -> usize {
        self.access_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl ProcessCheckSupervisor for BoundedProcessCheckSupervisor {
    fn check_process(
        &self,
        process_id: u32,
        timeout: Duration,
    ) -> Result<Vec<u8>, ProcessCheckSupervisorError> {
        #[cfg(test)]
        self.access_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
    #[cfg(test)]
    before_check_hook: Mutex<Option<Box<dyn Fn() + Send>>>,
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
            #[cfg(test)]
            before_check_hook: Mutex::new(None),
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

    #[cfg(test)]
    fn install_before_check_hook(&self, hook: impl Fn() + Send + 'static) {
        let mut before_check_hook = match self.before_check_hook.lock() {
            Ok(before_check_hook) => before_check_hook,
            Err(poisoned_before_check_hook) => poisoned_before_check_hook.into_inner(),
        };
        *before_check_hook = Some(Box::new(hook));
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
        #[cfg(test)]
        let before_check_hook = self
            .before_check_hook
            .lock()
            .map_err(|_| PortFailure {
                detail: "before-check hook store is poisoned".to_owned(),
            })?
            .take();
        #[cfg(test)]
        if let Some(before_check_hook) = before_check_hook {
            before_check_hook();
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
