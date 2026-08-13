use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use cognitive_kernel::executor::{
    DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
};
use cognitive_kernel::ports::PortFailure;
use cognitive_kernel::tool_registry::NativeOperationFamily;

use crate::personal::scheduler_authority::ResolvedNativeWorkerDispatch;

use super::{
    NativeToolExecutionError, NativeToolExecutionRequest, NativeWorkspaceReadExecutor,
    validate_native_tool_request,
};

/// Composition-root router for daemon-staged native Tool requests.
///
/// The first production caller supports the parameter-free WorkspaceRead
/// family. Families that need a separately governed payload or preimage remain
/// fail-closed until such an immutable carrier exists; the router never invents
/// input from a digest.
pub(crate) struct ProductionNativeToolExecutorRouter {
    workspace_root: PathBuf,
    workspace_read: NativeWorkspaceReadExecutor,
    staged_families: Mutex<BTreeMap<String, NativeOperationFamily>>,
}

impl ProductionNativeToolExecutorRouter {
    pub(crate) fn open(
        trusted_fencing_epoch: i64,
        workspace_root: PathBuf,
    ) -> Result<Self, NativeToolExecutionError> {
        std::fs::create_dir_all(&workspace_root).map_err(|error| {
            NativeToolExecutionError::ExecutorUnavailable(format!(
                "create daemon-approved workspace root: {error}"
            ))
        })?;
        Ok(Self {
            workspace_root,
            workspace_read: NativeWorkspaceReadExecutor::new(trusted_fencing_epoch),
            staged_families: Mutex::new(BTreeMap::new()),
        })
    }

    pub(crate) fn stage_resolved(
        &self,
        resolved: &ResolvedNativeWorkerDispatch,
    ) -> Result<(), NativeToolExecutionError> {
        let family = resolved.native_tool.descriptor.family;
        if family != NativeOperationFamily::WorkspaceRead {
            return Err(NativeToolExecutionError::UnsupportedExecutionFamily);
        }
        let request = validate_native_tool_request(&NativeToolExecutionRequest {
            descriptor: resolved.native_tool.descriptor.clone(),
            target: resolved.candidate.target.clone(),
            input: Vec::new(),
            workspace_root: Some(self.workspace_root.clone()),
            expected_preimage: None,
        })?;
        self.workspace_read.stage_request(
            resolved.intent.idempotency_key.clone(),
            resolved.intent.parameters_digest.clone(),
            &request,
        )?;
        let mut staged_families = self.staged_families.lock().map_err(|_| {
            NativeToolExecutionError::ExecutorUnavailable(
                "native executor routing table is poisoned".to_owned(),
            )
        })?;
        if let Some(existing_family) = staged_families.get(&resolved.intent.idempotency_key) {
            if *existing_family != family {
                return Err(NativeToolExecutionError::IdempotencyBindingConflict);
            }
            return Ok(());
        }
        staged_families.insert(resolved.intent.idempotency_key.clone(), family);
        Ok(())
    }

    fn staged_family(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<NativeOperationFamily>, PortFailure> {
        self.staged_families
            .lock()
            .map_err(|_| PortFailure {
                detail: "native executor routing table is poisoned".to_owned(),
            })
            .map(|families| families.get(idempotency_key).copied())
    }

    #[cfg(test)]
    pub(crate) fn install_workspace_read_before_io_hook(&self, hook: impl Fn() + Send + 'static) {
        self.workspace_read.install_before_read_hook(hook);
    }
}

impl EffectExecutor for ProductionNativeToolExecutorRouter {
    fn capabilities(&self) -> ExecutorCapabilities {
        ExecutorCapabilities {
            queryable: true,
            idempotent: true,
        }
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        match self.staged_family(&call.idempotency_key)? {
            Some(NativeOperationFamily::WorkspaceRead) => self.workspace_read.dispatch(call),
            Some(_) => Ok(DispatchOutcome::NotExecuted {
                reason: "native family has no production request carrier".to_owned(),
            }),
            None => Ok(DispatchOutcome::NotExecuted {
                reason: "no daemon-staged native request for idempotency key".to_owned(),
            }),
        }
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        match self.staged_family(idempotency_key)? {
            Some(NativeOperationFamily::WorkspaceRead) => {
                self.workspace_read.query_outcome(idempotency_key)
            }
            Some(_) | None => Ok(ExecutorQueryResult::NotExecuted),
        }
    }
}
