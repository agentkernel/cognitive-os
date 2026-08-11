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
